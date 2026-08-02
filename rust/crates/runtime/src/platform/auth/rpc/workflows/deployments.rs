use super::super::{
    AuthRpcProcessor, AuthorizationStateError, DeploymentProfileState, ProvisionedIdentityKind,
    RuntimeInstanceState, ValidatedRequest, Value,
};

pub(super) async fn dispatch(
    processor: &AuthRpcProcessor,
    subject: &str,
    payload: &[u8],
    caller: ValidatedRequest,
) -> Result<Value, AuthorizationStateError> {
    match subject {
        "rpc.v1.Auth.Deployments.Create" => processor.deployments_create(payload, &caller).await,
        "rpc.v1.Auth.Deployments.List" => processor.deployments_list(payload).await,
        "rpc.v1.Auth.Deployments.Enable" => {
            processor
                .deployments_set_state(payload, &caller, DeploymentProfileState::Active)
                .await
        }
        "rpc.v1.Auth.Deployments.Disable" => {
            processor
                .deployments_set_state(payload, &caller, DeploymentProfileState::Disabled)
                .await
        }
        "rpc.v1.Auth.Deployments.Remove" => {
            processor
                .deployments_set_state(payload, &caller, DeploymentProfileState::Removed)
                .await
        }
        "rpc.v1.Auth.ServiceInstances.Provision" => {
            processor
                .service_instances_provision(payload, &caller)
                .await
        }
        "rpc.v1.Auth.ServiceInstances.List" => processor.service_instances_list(payload).await,
        "rpc.v1.Auth.ServiceInstances.Enable" => {
            processor
                .provisioned_instance_set_state(
                    payload,
                    &caller,
                    ProvisionedIdentityKind::Service,
                    RuntimeInstanceState::Active,
                )
                .await
        }
        "rpc.v1.Auth.ServiceInstances.Disable" => {
            processor
                .provisioned_instance_set_state(
                    payload,
                    &caller,
                    ProvisionedIdentityKind::Service,
                    RuntimeInstanceState::Disabled,
                )
                .await
        }
        "rpc.v1.Auth.ServiceInstances.Remove" => {
            processor
                .provisioned_instance_set_state(
                    payload,
                    &caller,
                    ProvisionedIdentityKind::Service,
                    RuntimeInstanceState::Revoked,
                )
                .await
        }
        _ => unknown(subject),
    }
}

fn unknown(subject: &str) -> Result<Value, AuthorizationStateError> {
    Err(AuthorizationStateError::InvalidRecord(format!(
        "Auth RPC is not implemented by Rust: {subject}"
    )))
}
