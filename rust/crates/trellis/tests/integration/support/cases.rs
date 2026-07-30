use super::matrix::{
    load_client_test_matrix, load_rust_runtime_test_matrix, CompletionStatus, TestMatrix,
};

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct IntegrationCase {
    pub(crate) id: String,
    pub(crate) module: String,
    pub(crate) function: String,
}

pub(crate) fn rust_case_by_id(id: &str) -> Option<IntegrationCase> {
    implemented_case_by_id(&load_client_test_matrix().ok()?, id)
}

pub(crate) fn rust_runtime_case_by_id(id: &str) -> Option<IntegrationCase> {
    implemented_case_by_id(&load_rust_runtime_test_matrix().ok()?, id)
}

fn implemented_case_by_id(matrix: &TestMatrix, id: &str) -> Option<IntegrationCase> {
    let case = matrix.case_by_id(id)?;
    if case.completion.rust != Some(CompletionStatus::Implemented) {
        return None;
    }
    let implementation = case.implementations.as_ref()?.rust.as_ref()?;
    Some(IntegrationCase {
        id: case.id.clone(),
        module: implementation.module.clone(),
        function: implementation.function.clone(),
    })
}
