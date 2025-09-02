
use crate::cryptography::CryptoOperation;
use crate::parser::{TestCase};
use crate::test_types::TestExecutor;
use crate::result_format::TestCaseResult;

pub struct AFT;

impl TestExecutor for AFT {
    fn execute_test(
        &self,
        crypto: &dyn CryptoOperation,
        test_case: &TestCase,
    ) -> TestCaseResult {
        TestCaseResult {
            tc_id: test_case.tc_id,
            md: crypto.execute(test_case),
        }
    }
}
