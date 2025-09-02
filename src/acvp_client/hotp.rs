use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::time::{SystemTime, UNIX_EPOCH};


pub fn hotp(key: &[u8], counter: u64, digits: u32) -> String {
 
    let mut mac = Hmac::<Sha256>::new_from_slice(key)
        .expect("HMAC can take key of any size");
    mac.update(&counter.to_be_bytes());
    let result = mac.finalize().into_bytes();

    let offset = (result[result.len() - 1] & 0x0f) as usize;
    let code = ((u32::from(result[offset]) & 0x7f) << 24)
        | ((u32::from(result[offset + 1])) << 16)
        | ((u32::from(result[offset + 2])) << 8)
        | u32::from(result[offset + 3]);

    format!("{:0width$}", code % 10_u32.pow(digits), width = digits as usize)
}


pub fn totp(key: &[u8], step: u64, digits: u32) -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time issue")
        .as_secs();
    let counter = now / step;
    hotp(key, counter, digits)
}