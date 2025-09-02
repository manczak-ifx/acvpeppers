use sha2::{Digest, Sha256, Sha512};
use crate::cryptography::CryptoOperation;
use crate::parser::TestCase;
use std::fs::OpenOptions;
use std::io::Write;

pub struct SHA2 { pub algorithm: String }


impl CryptoOperation for SHA2 {
    fn execute(&self, tc: &TestCase) -> String {
        // missing msg = empty only if len == 0
        let msg_hex = match (tc.msg.as_deref(), tc.len) {
            (Some(s), _) => s,                  
            (None, Some(0)) => "",            
            (None, _) => panic!("Missing 'msg' for non-zero length test"),
        };

        
        let mut input = hex::decode(msg_hex).expect("Invalid hex in 'msg'");

        if let Some(bits) = tc.len {
            let bits = bits as usize;
            if bits == 0 {
                input.clear(); 
            } else {
                let full_bytes = bits / 8;
                let rem_bits   = bits % 8;
                let needed = full_bytes + if rem_bits > 0 { 1 } else { 0 };

                if input.len() > needed {
                    input.truncate(needed);
                } else if input.len() < needed {
                    input.resize(needed, 0);
                }
                if rem_bits > 0 {
                    let mask: u8 = 0xFF << (8 - rem_bits);
                    input[full_bytes] &= mask;
                }
            }
        }
          
    log_to_file(&format!("tcId {} → digest = {}", tc.tc_id, hex::encode_upper(&input)));
        let digest = match self.algorithm.as_str() {
            "SHA2-256" => sha2::Sha256::digest(&input).to_vec(),
            "SHA2-512" => sha2::Sha512::digest(&input).to_vec(),
            other => panic!("Unsupported algorithm: {other}"),
        };

        hex::encode_upper(digest)
    }
}

fn log_to_file(msg: &str) {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("acvp_debug.log")
        .unwrap();
    writeln!(file, "{}", msg).unwrap();
}
