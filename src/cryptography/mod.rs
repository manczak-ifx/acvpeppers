pub mod registry; 
pub mod sha2_alg;      
pub mod hw_proxy;
pub mod aes_ecb;
use crate::parser::{TestCase};
use std::io::Read;

pub trait CryptoOperation: Send + Sync {
    /// AFT/MCT
    fn execute(&self, test_case: &TestCase) -> String;

    /// LDT
    fn execute_streaming(
        &self,
        _test_case: &TestCase,
        _reader: &mut dyn Read,
        _total_len: u64,
    ) -> Result<String, &'static str> {
        Err("streaming not supported")
    }
}