use crate::{
    db::{Db, Payment},
    utils::{decrypt_wif, script_hash},
};
use anyhow::Result;
use bech32::{decode, FromBase32};
use bitcoin::util::address::WitnessVersion;
use bitcoin::{
    blockdata::{script::Script, transaction::OutPoint},
    util::psbt::serialize::Serialize,
    SigHashType, Transaction, TxIn, TxOut, Txid,
};
use secp256k1::{PublicKey, Secp256k1};
use std::str::FromStr;
use tokio::{
    spawn,
    time::{interval, Duration},
};

fn addr_to_script(addr: &str) -> Script {
    let (_, data, _) = decode(addr).expect("bech32 decode");
    let prog = Vec::<u8>::from_base32(&data).expect("base32 decode");
    Script::new_witness_program(WitnessVersion::V0, &prog)
}

/// Launch the background sweeper.
pub async fn start(db: Db) {
    spawn(async move {
        let mut iv = interval(Duration::from_secs(10));
        loop {
            iv.tick().await;
            for p in db.all() {
                if let Err(e) = process(&db, &p).await {
                    eprintln!("[Sweeper] {} {}", p.id, e);
                }
            }
        }
    });
}

async fn process(db: &Db, p: &Payment) -> Result<()> {
    let bal = crate::electrum::rpc(
        "blockchain.scripthash.get_balance",
        &[script_hash(&p.address).into()],
    )?;
    let target_sat = (p.amount * 1e8) as u64;
    if bal["confirmed"].as_u64().unwrap_or(0) < target_sat {
        return Ok(());
    }

    let utxos = crate::electrum::rpc(
        "blockchain.scripthash.listunspent",
        &[script_hash(&p.address).into()],
    )?;

    let main_script = addr_to_script("ltc1qf00w70ek4tyzgpfjpenadtjys8k2mhw7t92adp");
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

    for u in utxos.as_array().unwrap() {
        total += u["value"].as_u64().unwrap();
        let txid = Txid::from_str(u["tx_hash"].as_str().unwrap())?;
        let vout = u["tx_pos"].as_u64().unwrap() as u32;
        tx.input.push(TxIn {
            previous_output: OutPoint::new(txid, vout),
            script_sig: Script::new(),
            sequence: 0xffffffff,
            witness: bitcoin::Witness::default(),
        });
    }

    let fee = crate::electrum::fee_sat(tx.vsize() as u64 + 68);
    if total <= fee {
        return Ok(());
    }
    tx.output[0].value = total - fee;

    let sk = secp256k1::SecretKey::from_str(&decrypt_wif(&p.wif_enc))?;
    let secp = Secp256k1::new();
    let pk = PublicKey::from_secret_key(&secp, &sk);

    for i in 0..tx.input.len() {
        let sighash = tx.signature_hash(i, &Script::new(), SigHashType::All as u32);
        let sig = secp.sign_ecdsa(&secp256k1::Message::from_slice(&sighash)?, &sk);

        let mut with_type = sig.serialize_der().to_vec();
        with_type.push(SigHashType::All as u8);

        tx.input[i].witness.push(with_type);
        tx.input[i].witness.push(pk.serialize().to_vec());
    }

    let txid = crate::electrum::rpc(
        "blockchain.transaction.broadcast",
        &[hex::encode(tx.serialize()).into()],
    )?;
    println!("[Sweeper] {} broadcast {}", p.id, txid);
    db.mark_completed(&p.id);
    Ok(())
}
