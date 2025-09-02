use crate::cryptography::registry::{CryptoRegistry};
use crate::test_types::registry::{TestTypeRegistry};
use crate::parser::{TestVectorSet, TestGroup, TestCase};
use crate::result_format::{TestResultSet, TestGroupResult, TestCaseResult};
fn execute_test_case(
    crypto: &dyn crate::cryptography::CryptoOperation,
    test_type: &dyn crate::test_types::TestExecutor,
    test_case: &TestCase,
) -> TestCaseResult {
    test_type.execute_test(crypto, test_case)
}
fn process_test_group(
    registry: &CryptoRegistry,
    test_type_registry: &TestTypeRegistry,
    group: &TestGroup,
    algorithm: &str,
) -> TestGroupResult {
    let crypto = registry
        .resolve(algorithm)
        .expect(&format!("Algorithm '{}' not found", algorithm));
    let test_type = test_type_registry
        .resolve(&group.test_type)
        .expect(&format!("Test type '{}' not found", group.test_type));

    let test_results = group
        .tests
        .iter()
        .map(|test_case| execute_test_case(crypto.as_ref(), test_type.as_ref(), test_case))
        .collect();

    TestGroupResult {
        tg_id: group.tg_id,
        tests: test_results,
    }
}
pub fn execute_test_vector(
    registry: &CryptoRegistry,
    test_type_registry: &TestTypeRegistry,
    test_vector: &TestVectorSet,
) -> TestResultSet {
    let test_group_results = test_vector
        .test_groups
        .iter()
        .map(|group| process_test_group(registry, test_type_registry, group, &test_vector.algorithm))
        .collect();
    TestResultSet {
        vs_id: test_vector.vs_id,
        algorithm: test_vector.algorithm.clone(),
        test_groups: test_group_results,
        revision: test_vector.revision.clone(),
    }
}

