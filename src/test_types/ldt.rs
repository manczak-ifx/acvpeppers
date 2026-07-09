use std::io::{Read, Result as IoResult};
use crate::cryptography::CryptoOperation;
use crate::parser::{TestCase};
use crate::result_format::TestCaseResult;
use log::{error, warn};
use hex;

/// Repeat pattern until total_len bytes have been read.
struct RepeatingReader { pattern: Vec<u8>, pos: usize, remaining: u64 }
impl RepeatingReader {
    fn new(pattern: Vec<u8>, total_len: u64) -> Self { Self { pattern, pos: 0, remaining: total_len } }
}
impl Read for RepeatingReader {
    fn read(&mut self, out: &mut [u8]) -> IoResult<usize> {
        if self.remaining == 0 || self.pattern.is_empty() { return Ok(0); }
        let mut written = 0;
        while written < out.len() && self.remaining > 0 {
            let take = (self.pattern.len() - self.pos).min(out.len() - written);
            out[written..written + take].copy_from_slice(&self.pattern[self.pos..self.pos + take]);
            self.pos = (self.pos + take) % self.pattern.len();
            written += take;
            self.remaining -= take as u64;
        }
        Ok(written)
    }
}

pub fn execute_test(crypto: &dyn CryptoOperation, tc: &TestCase) -> TestCaseResult {
    let Some(lm) = &tc.large_msg else {
        error!("LDT: missing largeMsg (tcId {})", tc.tc_id);
        return TestCaseResult { tc_id: tc.tc_id, md: String::new() };
    };

    if !lm.expansion_technique.eq_ignore_ascii_case("repeating") {
        error!("LDT: unsupported expansionTechnique='{}' (tcId {})", lm.expansion_technique, tc.tc_id);
        return TestCaseResult { tc_id: tc.tc_id, md: String::new() };
    }

    let seed = match hex::decode(&lm.content) {
        Ok(b) => b,
        Err(e) => {
            error!("LDT: invalid largeMsg.content hex (tcId {}): {e}", tc.tc_id);
            return TestCaseResult { tc_id: tc.tc_id, md: String::new() };
        }
    };

    
    let bits = (seed.len() as u64) * 8;
    if bits != lm.content_length {
        warn!("LDT: contentLength={} != {}*8 (tcId {})", lm.content_length, seed.len(), tc.tc_id);
    }

    
    let total_len = match u64::try_from(lm.full_length) {
        Ok(v) => v,
        Err(_) => {
            error!("LDT: fullLength does not fit into u64 (tcId {})", tc.tc_id);
            return TestCaseResult { tc_id: tc.tc_id, md: String::new() };
        }
    };

    let mut reader = RepeatingReader::new(seed, total_len);

    let md = match crypto.execute_streaming(tc, &mut reader, total_len) {
        Ok(hex_digest) => hex_digest,
        Err(why) => {
            error!("LDT: algo streaming unsupported ({why}) (tcId {})", tc.tc_id);
            String::new()
        }
    };

    TestCaseResult { tc_id: tc.tc_id, md }
}