use rayon::prelude::*;
use std::sync::Arc;

use crate::cryptography::{CryptoOperation};
use crate::cryptography::registry::CryptoRegistry;
use crate::parser::{TestGroup, TestType, TestVectorSet};
use crate::result_format::{TestResultSet, TestGroupResult, TestCaseResult};

pub fn execute_test_vector(registry: &CryptoRegistry, tvs: &TestVectorSet) -> TestResultSet {
    let test_groups: Vec<TestGroupResult> = tvs.test_groups
        .par_iter()
        .map(|group: &TestGroup| {
            // shared read-only
            let crypto: Arc<dyn CryptoOperation + Send + Sync> =
                registry.resolve(&tvs.algorithm).expect("algorithm not found");

            let exec: fn(&dyn CryptoOperation, &_) -> TestCaseResult = match group.test_type {
                TestType::AFT => crate::test_types::aft::execute_test,
                TestType::MCT => crate::test_types::mct::execute_test, 
                TestType::LDT => crate::test_types::ldt::execute_test,
            };
            let tests: Vec<TestCaseResult> = group.tests
                .par_iter()
                .map(|tc| exec(crypto.as_ref(), tc))
                .collect();

            TestGroupResult { tg_id: group.tg_id, tests }
        })
        .collect();

    TestResultSet {
        vs_id: tvs.vs_id,
        algorithm: tvs.algorithm.clone(),
        test_groups,
        revision: tvs.revision.clone(),
    }
}