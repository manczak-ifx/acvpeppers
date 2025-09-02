pub mod registry; 
pub mod sha2_alg;      
pub mod hw_proxy;
use crate::parser::{TestCase};
pub trait CryptoOperation {
   
    fn execute(&self, test_case: &TestCase) -> String;
}
