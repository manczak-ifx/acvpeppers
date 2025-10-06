
use crate::cryptography::CryptoOperation;
use crate::parser::{TestCase};
use crate::result_format::TestCaseResult;




   pub fn execute_test(
        crypto: &dyn CryptoOperation,
        test_case: &TestCase,
    ) -> TestCaseResult {
        TestCaseResult {
            tc_id: test_case.tc_id,
            md: crypto.execute(test_case),
        }
    }

