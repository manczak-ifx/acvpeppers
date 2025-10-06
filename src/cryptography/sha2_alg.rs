use sha2::{Digest, Sha256, Sha512};
use crate::cryptography::CryptoOperation;
use crate::parser::TestCase;


pub struct SHA2 { pub algorithm: String }

impl CryptoOperation for SHA2 {
    fn execute(&self, tc: &TestCase) -> String {
        // 1) Get the hex string for the message
        //    - missing msg is allowed only if len == 0
        let raw_hex = match (tc.msg.as_deref(), tc.len) {
            (Some(s), _) => s,
            (None, Some(0)) => "",
            (None, _) => panic!("tcId {}: Missing 'msg' for non-zero 'len'", tc.tc_id),
        };

        // 2)  trim and convert any smart quotes to ASCII quotes
        let msg_hex = raw_hex.trim()
            .replace('“', "\"")
            .replace('”', "\"")
            .replace('’', "'")
            .replace('‘', "'");

        // 3) Decode hex 
        //    If hex is odd-length or has non-hex chars, error !
        let mut input = hex::decode(&msg_hex)
            .unwrap_or_else(|e| panic!("tcId {}: Invalid hex in 'msg': {} → {}", tc.tc_id, msg_hex, e));

        if let Some(bits_u32) = tc.len {
            let bits = bits_u32 as usize;

            if bits == 0 {
                input.clear();
            } else {
                let full_bytes = bits / 8;
                let rem_bits   = bits % 8;
                let needed     = full_bytes + if rem_bits > 0 { 1 } else { 0 };
                assert!(
                    input.len() >= needed,
                    "tcId {}: msg too short for len={} bits ({} bytes needed, got {}): {}",
                    tc.tc_id, bits, needed, input.len(), msg_hex
                );
                if input.len() > needed {
                    input.truncate(needed);
                }
                if rem_bits > 0 {
                    let last = needed - 1;
                    let mask: u8 = 0xFF << (8 - rem_bits);
                    input[last] &= mask;
                }
            }
        }

        

        // 5) Hash
        let digest = match self.algorithm.as_str() {
            "SHA2-256" => Sha256::digest(&input).to_vec(),
            "SHA2-512" => Sha512::digest(&input).to_vec(),
            other => panic!("tcId {}: Unsupported algorithm: {}", tc.tc_id, other),
        };

        let out = hex::encode_upper(digest);

        out
    }
}
