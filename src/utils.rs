use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use bech32::{decode, encode, FromBase32, ToBase32, Variant};
use bitcoin::blockdata::script::Script;
use bitcoin::util::address::WitnessVersion;
use hex::{decode as hex_decode, encode as hex_encode};
use lazy_static::lazy_static;
use rand::RngCore;
use ripemd::Ripemd160;
use secp256k1::{Secp256k1, SecretKey};
use sha2::{Digest, Sha256};
use std::env;
use tracing::{debug, error, info, instrument, trace};

lazy_static! {
    static ref AES: Aes256Gcm = {
        match env::var("WIF_KEY") {
            Ok(key) => match hex_decode(&key) {
                Ok(k) => {
                    debug!("AES key initialized successfully");
                    Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&k))
                }
                Err(e) => {
                    error!("Failed to decode WIF_KEY: {}", e);
                    panic!("Failed to decode WIF_KEY: {}", e);
                }
            },
            Err(e) => {
                error!("WIF_KEY environment variable not set: {}", e);
                panic!("WIF_KEY environment variable not set: {}", e);
            }
        }
    };
}

#[instrument(level = "debug", skip(wif))]
pub fn encrypt_wif(wif: &str) -> String {
    trace!("Encrypting WIF");
    let mut iv = [0u8; 12];
    OsRng.fill_bytes(&mut iv);
    let nonce = Nonce::from_slice(&iv);

    match AES.encrypt(nonce, wif.as_bytes()) {
        Ok(mut ct) => {
            let tag = ct.split_off(ct.len() - 16);
            let result = hex_encode([iv.to_vec(), tag, ct].concat());
            debug!("WIF encrypted successfully");
            result
        }
        Err(e) => {
            error!("Failed to encrypt WIF: {}", e);
            panic!("Encryption failed: {}", e);
        }
    }
}

#[instrument(level = "debug", skip(h))]
pub fn decrypt_wif(h: &str) -> String {
    trace!("Decrypting WIF");
    let buf = match hex_decode(h) {
        Ok(b) => b,
        Err(e) => {
            error!("Failed to hex decode encrypted WIF: {}", e);
            panic!("Failed to hex decode encrypted WIF: {}", e);
        }
    };

    let (iv, rest) = buf.split_at(12);
    let (tag, ct) = rest.split_at(16);
    let nonce = Nonce::from_slice(iv);
    let mut data = ct.to_vec();
    data.extend_from_slice(tag);

    match AES.decrypt(nonce, data.as_ref()) {
        Ok(decrypted) => match String::from_utf8(decrypted) {
            Ok(wif) => {
                debug!("WIF decrypted successfully");
                wif
            }
            Err(e) => {
                error!("Failed to convert decrypted WIF to UTF-8: {}", e);
                panic!("Failed to convert decrypted WIF to UTF-8: {}", e);
            }
        },
        Err(e) => {
            error!("Failed to decrypt WIF: {}", e);
            panic!("Decryption failed: {}", e);
        }
    }
}

fn hash160(b: &[u8]) -> [u8; 20] {
    trace!("Computing hash160");
    let mut out = [0u8; 20];
    out.copy_from_slice(&Ripemd160::digest(&Sha256::digest(b)));
    out
}

#[instrument(level = "info")]
pub fn new_key() -> (SecretKey, String, String) {
    info!("Generating new key pair");
    let secp = Secp256k1::new();
    let sk = SecretKey::new(&mut rand::thread_rng());
    let pk = secp256k1::PublicKey::from_secret_key(&secp, &sk);
    let prog = hash160(&pk.serialize());

    // prepend witness version (0) before convertbits
    let mut data = vec![(&[0u8]).to_base32()[0]];
    data.extend_from_slice(&prog.to_base32());

    match encode("ltc", data, Variant::Bech32) {
        Ok(addr) => {
            info!("New key pair generated successfully");
            debug!("Generated address: {}", addr);
            (sk, sk.display_secret().to_string(), addr)
        }
        Err(e) => {
            error!("Failed to encode address: {}", e);
            panic!("Failed to encode address: {}", e);
        }
    }
}

/// Electrum‐X script‑hash (sha256(scriptPubKey) reversed, hex‑le).
#[instrument(level = "debug", skip(addr))]
pub fn script_hash(addr: &str) -> String {
    trace!("Computing script hash for address");
    let (hrp, data, variant) = match decode(addr) {
        Ok(result) => result,
        Err(e) => {
            error!("Failed to decode address: {}", e);
            panic!("Failed to decode address: {}", e);
        }
    };

    debug!("Address decoded: hrp={}, variant={:?}", hrp, variant);
    let (ver, prog5) = data.split_first().expect("empty data");
    assert_eq!(
        *ver,
        (&[0u8]).to_base32()[0],
        "Only witness version 0 is supported"
    );

    let prog: Vec<u8> = match Vec::<u8>::from_base32(prog5) {
        Ok(p) => p,
        Err(e) => {
            error!("Failed to convert from base32: {}", e);
            panic!("Failed to convert from base32: {}", e);
        }
    };

    let script = Script::new_witness_program(WitnessVersion::V0, &prog);
    let mut h = Sha256::digest(script.as_bytes()).to_vec();
    h.reverse();

    let result = hex_encode(h);
    debug!("Script hash computed successfully");
    result
}
