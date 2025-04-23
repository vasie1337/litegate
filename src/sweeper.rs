use crate::{
    db::{Db, Payment},
    electrum::{fee_sat_async, rpc_async},
    utils::{decrypt_wif, script_hash},
};
use anyhow::Result;
use bech32::{decode, FromBase32};
use bitcoin::Witness;
use bitcoin::{
    blockdata::{script::Script, transaction::OutPoint},
    util::{psbt::serialize::Serialize, sighash::SighashCache},
    EcdsaSighashType, PubkeyHash, Transaction, TxIn, TxOut, Txid,
};
use bitcoin::{hashes::Hash, util::address::WitnessVersion};
use ripemd::Ripemd160;
use secp256k1::{PublicKey, Secp256k1, SecretKey};
use sha2::{Digest, Sha256};
use std::{env, str::FromStr};
use tokio::{
    spawn,
    time::{interval, Duration},
};
use tracing::{debug, error, info, instrument};

fn addr_to_script(addr: &str) -> Script {
    let (_, data, _) = decode(addr).expect("bech32 decode");
    let (_, prog5) = data.split_first().expect("empty data");
    let prog = Vec::<u8>::from_base32(prog5).expect("base32 decode");
    Script::new_witness_program(WitnessVersion::V0, &prog)
}

fn hash160(data: &[u8]) -> [u8; 20] {
    let mut out = [0u8; 20];
    out.copy_from_slice(&Ripemd160::digest(&Sha256::digest(data)));
    out
}

fn p2pkh_script_code(pk: &PublicKey) -> Script {
    let h = hash160(&pk.serialize());
    let pubkey_hash = PubkeyHash::from_slice(&h).expect("valid hash160");
    Script::new_p2pkh(&pubkey_hash)
}

/// Launch the background sweeper.
pub async fn start(db: Db) {
    info!("Starting background sweeper");
    spawn(async move {
        let mut iv = interval(Duration::from_secs(10));
        loop {
            iv.tick().await;
            debug!("Sweeper tick - checking payments");
            let payments = match db.all() {
                Ok(payments) => payments,
                Err(e) => {
                    error!("Failed to fetch payments: {}", e);
                    Vec::new()
                }
            };
            for payment in &payments {
                if let Err(e) = process(&db, payment).await {
                    error!(payment_id = %payment.id, error = %e, "Failed to process payment");
                }
            }
        }
    });
}

#[instrument(skip(db, p), fields(payment_id = %p.id))]
async fn process(db: &Db, p: &Payment) -> Result<()> {
    // First check confirmations
    let hist = rpc_async(
        "blockchain.scripthash.get_history",
        &[script_hash(&p.address).into()],
    )
    .await?;
    let hdr = rpc_async("blockchain.headers.subscribe", &[]).await?;
    let tip = hdr["height"].as_u64().unwrap_or(0);

    // For a single-payment address, min block height across all txs in history is enough
    let confirmations = hist
        .as_array()
        .unwrap()
        .iter()
        .filter(|h| h["height"].as_u64().unwrap_or(0) > 0)
        .map(|h| tip - h["height"].as_u64().unwrap() + 1)
        .min()
        .unwrap_or(0);

    let needed = env::var("CONFIRMATIONS")
        .unwrap_or_else(|_| "2".to_string())
        .parse::<u64>()
        .unwrap_or(2);

    debug!(confirmations, needed, "Checking confirmations");
    if confirmations < needed {
        // Not enough confirmations yet; skip
        debug!("Not enough confirmations yet");
        return Ok(());
    }

    // Now check if the confirmed balance meets or exceeds `p.amount`
    let bal = rpc_async(
        "blockchain.scripthash.get_balance",
        &[script_hash(&p.address).into()],
    )
    .await?;
    let target_sat = (p.amount * 1e8) as u64;
    let confirmed_balance = bal["confirmed"].as_u64().unwrap_or(0);

    debug!(confirmed_balance, target_sat, "Checking balance");
    if confirmed_balance < target_sat {
        debug!("Balance insufficient");
        return Ok(());
    }

    // Collect UTXOs and sweep
    let utxos = rpc_async(
        "blockchain.scripthash.listunspent",
        &[script_hash(&p.address).into()],
    )
    .await?;

    let main_address = env::var("MAIN_ADDRESS").expect("MAIN_ADDRESS must be set");
    let main_script = addr_to_script(&main_address);
    let mut total = 0u64;
    let mut tx = Transaction {
        version: 2,
        lock_time: 0,
        input: vec![],
        output: vec![TxOut {
            value: 0,
            script_pubkey: main_script,
        }],
    };

    let mut prev_values = Vec::new();
    for u in utxos.as_array().unwrap() {
        let val = u["value"].as_u64().unwrap();
        total += val;
        prev_values.push(val);
        let txid = Txid::from_str(u["tx_hash"].as_str().unwrap())?;
        let vout = u["tx_pos"].as_u64().unwrap() as u32;
        tx.input.push(TxIn {
            previous_output: OutPoint::new(txid, vout),
            script_sig: Script::new(),
            sequence: 0xffffffff,
            witness: Witness::default(),
        });
    }

    let fee = fee_sat_async(tx.vsize() as u64 + 68).await;
    if total <= fee {
        debug!(total, fee, "Total less than or equal to fee, skipping");
        return Ok(());
    }
    tx.output[0].value = total - fee;

    let sk_hex = decrypt_wif(&p.wif_enc);
    let sk = SecretKey::from_str(&sk_hex)?;
    let secp = Secp256k1::new();
    let pk = PublicKey::from_secret_key(&secp, &sk);

    debug!(inputs = tx.input.len(), "Signing transaction");
    for (i, prev_value) in prev_values.iter().enumerate() {
        let script_code = p2pkh_script_code(&pk);
        let sighash = {
            let mut cache = SighashCache::new(&mut tx);
            cache.segwit_signature_hash(i, &script_code, *prev_value, EcdsaSighashType::All)?
        };
        let msg = secp256k1::Message::from_slice(&sighash[..])?;
        let mut sig: Vec<u8> = secp.sign_ecdsa(&msg, &sk).serialize_der().to_vec();
        sig.push(EcdsaSighashType::All.to_u32() as u8);
        tx.input[i].witness.push(sig);
        tx.input[i].witness.push(pk.serialize().to_vec());
    }

    let txid = rpc_async(
        "blockchain.transaction.broadcast",
        &[hex::encode(tx.serialize()).into()],
    )
    .await?;

    info!(txid = %txid, total_amount = total - fee, "Transaction broadcast successfully");
    let _ = db.mark_completed(&p.id);
    Ok(())
}
