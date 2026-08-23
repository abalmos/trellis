use super::cases::{rust_case_by_id, rust_runtime_case_by_id};
use super::matrix::{load_client_test_matrix, load_rust_runtime_test_matrix};

pub(crate) fn assert_generated_service_contract<C>(contract: &trellis_test::TrellisTestContract)
where
    C: trellis_rs::service::GeneratedServiceContract,
{
    assert_eq!(C::PARTICIPANT_ID, contract.id());
    assert_eq!(C::CONTRACT_DIGEST, contract.digest());
    assert_eq!(C::PARTICIPANT_NEEDS_DIGEST, contract.needs_digest());
    assert_eq!(C::API_DIGEST, contract.api_digest());
    assert!(C::REFERENCED_API_ARTIFACTS.is_empty());
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(C::PARTICIPANT_JSON).unwrap(),
        *contract.participant()
    );
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(C::API_JSON).unwrap(),
        *contract.api()
    );
}

pub(crate) fn assert_case_registered(case_id: &str, fixture: &str, module: &str) {
    let matrix = load_client_test_matrix().expect("load shared client integration matrix");
    let matrix_case = matrix
        .case_by_id(case_id)
        .unwrap_or_else(|| panic!("matrix is missing {case_id}"));
    let local_case =
        rust_case_by_id(case_id).unwrap_or_else(|| panic!("Rust manifest is missing {case_id}"));

    assert_eq!(matrix_case.fixture, fixture);
    assert_eq!(local_case.module, module);
    trellis_test::set_current_test_tenant(format!(
        "{}::{}",
        local_case.module, local_case.function
    ));
}

pub(crate) fn assert_runtime_case_registered(case_id: &str, fixture: &str, module: &str) {
    let matrix = load_rust_runtime_test_matrix().expect("load Rust runtime test matrix");
    let matrix_case = matrix
        .case_by_id(case_id)
        .unwrap_or_else(|| panic!("service matrix is missing {case_id}"));
    let local_case = rust_runtime_case_by_id(case_id)
        .unwrap_or_else(|| panic!("Rust runtime matrix is missing {case_id}"));

    assert_eq!(matrix_case.fixture, fixture);
    assert_eq!(local_case.module, module);
    trellis_test::set_current_test_tenant(format!(
        "{}::{}",
        local_case.module, local_case.function
    ));
}
