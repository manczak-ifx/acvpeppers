// src/cryptography/aes_ecb.rs
use crate::cryptography::CryptoOperation;
use crate::parser::TestCase;

use aes::{Aes128, Aes192, Aes256};
use cipher::{BlockDecryptMut, BlockEncryptMut, KeyInit, block_padding::NoPadding};
use ecb::{Decryptor as EcbDec, Encryptor as EcbEnc};
use hex;

pub struct AesEcbOp;

impl CryptoOperation for AesEcbOp {
    fn execute(&self, tc: &TestCase) -> String {
        let is_encrypt = tc.plaintext.is_some();

        // decode key + data
        let key = match tc.key.as_deref().and_then(|h| hex::decode(h).ok()) {
            Some(k) => k,
            None => return String::new(),
        };
        let data = match (if is_encrypt { tc.plaintext.as_deref() } else { tc.ct.as_deref() })
            .and_then(|h| hex::decode(h).ok())
        {
            Some(d) => d,
            None => return String::new(),
        };

        // input must be a multiple of 16 bytes
        if data.len() % 16 != 0 {
            return String::new();
        }

        
        let result = match (key.len(), is_encrypt) {
            // AES-128
            (16, true) => {
                let enc = match EcbEnc::<Aes128>::new_from_slice(&key) { Ok(c)=>c, Err(_)=>return String::new() };
                let mut out = vec![0u8; data.len()];
                match enc.encrypt_padded_b2b_mut::<NoPadding>(&data, &mut out) {
                    Ok(ct) => hex::encode(ct),
                    Err(_) => return String::new(),
                }
            }
            (16, false) => {
                let dec = match EcbDec::<Aes128>::new_from_slice(&key) { Ok(c)=>c, Err(_)=>return String::new() };
                let mut out = vec![0u8; data.len()];
                match dec.decrypt_padded_b2b_mut::<NoPadding>(&data, &mut out) {
                    Ok(pt) => hex::encode(pt),
                    Err(_) => return String::new(),
                }
            }

            // AES-192
            (24, true) => {
                let enc = match EcbEnc::<Aes192>::new_from_slice(&key) { Ok(c)=>c, Err(_)=>return String::new() };
                let mut out = vec![0u8; data.len()];
                match enc.encrypt_padded_b2b_mut::<NoPadding>(&data, &mut out) {
                    Ok(ct) => hex::encode(ct),
                    Err(_) => return String::new(),
                }
            }
            (24, false) => {
                let dec = match EcbDec::<Aes192>::new_from_slice(&key) { Ok(c)=>c, Err(_)=>return String::new() };
                let mut out = vec![0u8; data.len()];
                match dec.decrypt_padded_b2b_mut::<NoPadding>(&data, &mut out) {
                    Ok(pt) => hex::encode(pt),
                    Err(_) => return String::new(),
                }
            }

            // AES-256
            (32, true) => {
                let enc = match EcbEnc::<Aes256>::new_from_slice(&key) { Ok(c)=>c, Err(_)=>return String::new() };
                let mut out = vec![0u8; data.len()];
                match enc.encrypt_padded_b2b_mut::<NoPadding>(&data, &mut out) {
                    Ok(ct) => hex::encode(ct),
                    Err(_) => return String::new(),
                }
            }
            (32, false) => {
                let dec = match EcbDec::<Aes256>::new_from_slice(&key) { Ok(c)=>c, Err(_)=>return String::new() };
                let mut out = vec![0u8; data.len()];
                match dec.decrypt_padded_b2b_mut::<NoPadding>(&data, &mut out) {
                    Ok(pt) => hex::encode(pt),
                    Err(_) => return String::new(),
                }
            }

            _ => return String::new(), // unsupported key length
        };

        result
    }
}