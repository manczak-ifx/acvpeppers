pub mod registry; 
pub mod AFT;       
pub mod MCT;       
pub mod LDT;    
use crate::cryptography::CryptoOperation; 
use crate::parser::{TestCase};

use crate::result_format::TestCaseResult; 

pub trait TestExecutor {
    fn execute_test(
        &self,
        crypto: &dyn CryptoOperation,
        test_case: &TestCase,
    ) -> TestCaseResult;
}