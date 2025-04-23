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
use tracing::{debug, info, instrument, trace};

lazy_static! {
    static ref AES: Aes256Gcm = {
        let key_hex = env::var("AES_KEY").expect("AES_KEY env missing");
        let key_bytes = hex_decode(&key_hex).expect("AES_KEY hex decode");
        assert_eq!(key_bytes.len(), 32, "AES_KEY len != 32 bytes");
        debug!("AES key init");
        Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key_bytes))
    };
}

#[instrument(level = "debug", skip(wif))]
pub fn encrypt_wif(wif: &str) -> String {
    trace!("Encrypting WIF");
    let mut iv = [0u8; 12];
    OsRng.fill_bytes(&mut iv);
    let nonce = Nonce::from_slice(&iv);
    let mut ct = AES.encrypt(nonce, wif.as_bytes()).expect("encrypt");
    let tag = ct.split_off(ct.len() - 16);
    hex_encode([iv.to_vec(), tag, ct].concat())
}

#[instrument(level = "debug", skip(h))]
pub fn decrypt_wif(h: &str) -> String {
    trace!("Decrypting WIF");
    let buf = hex_decode(h).expect("hex decode");
    let (iv, rest) = buf.split_at(12);
    let (tag, ct) = rest.split_at(16);
    let nonce = Nonce::from_slice(iv);
    let mut data = ct.to_vec();
    data.extend_from_slice(tag);
    let dec = AES.decrypt(nonce, data.as_ref()).expect("decrypt");
    String::from_utf8(dec).expect("utf8")
}

fn hash160(b: &[u8]) -> [u8; 20] {
    let mut out = [0u8; 20];
    out.copy_from_slice(&Ripemd160::digest(&Sha256::digest(b)));
    out
}

#[instrument(level = "info")]
pub fn new_key() -> (SecretKey, String, String) {
    info!("Generating key");
    let secp = Secp256k1::new();
    let sk = SecretKey::new(&mut rand::thread_rng());
    let pk = secp256k1::PublicKey::from_secret_key(&secp, &sk);
    let prog = hash160(&pk.serialize());
    let mut data = vec![(&[0u8]).to_base32()[0]];
    data.extend_from_slice(&prog.to_base32());
    let addr = encode("ltc", data, Variant::Bech32).expect("bech32");
    debug!("addr {}", addr);
    (sk, sk.display_secret().to_string(), addr)
}

#[instrument(level = "debug", skip(addr))]
pub fn script_hash(addr: &str) -> String {
    trace!("Script hash");
    let (_, data, _) = decode(addr).expect("decode");
    let (ver, prog5) = data.split_first().expect("empty");
    assert_eq!(*ver, (&[0u8]).to_base32()[0]);
    let prog: Vec<u8> = Vec::<u8>::from_base32(prog5).expect("base32");
    let script = Script::new_witness_program(WitnessVersion::V0, &prog);
    let mut h = Sha256::digest(script.as_bytes()).to_vec();
    h.reverse();
    hex_encode(h)
}
