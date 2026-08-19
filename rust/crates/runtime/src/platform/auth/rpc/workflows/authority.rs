use super::super::{AuthRpcProcessor, AuthorizationStateError, ValidatedRequest, Value};

pub(super) async fn dispatch(
    processor: &AuthRpcProcessor,
    subject: &str,
    payload: &[u8],
    caller: ValidatedRequest,
) -> Result<Value, AuthorizationStateError> {
    match subject {
        "rpc.v1.Auth.DeploymentAuthority.List" => {
            processor.deployment_authority_list(payload).await
        }
        "rpc.v1.Auth.DeploymentAuthority.Get" => processor.deployment_authority_get(payload).await,
        "rpc.v1.Auth.DeploymentAuthority.Plan" => {
            processor.deployment_authority_plan(payload, &caller).await
        }
        "rpc.v1.Auth.DeploymentAuthority.Plans.List" => {
            processor.authority_plans_list(payload).await
        }
        "rpc.v1.Auth.DeploymentAuthority.Plans.Get" => processor.authority_plans_get(payload).await,
        "rpc.v1.Auth.DeploymentAuthority.AcceptUpdate" => {
            processor
                .authority_accept(
                    payload,
                    &caller,
                    crate::platform::auth::AuthorityProposalKind::Update,
                )
                .await
        }
        "rpc.v1.Auth.DeploymentAuthority.AcceptMigration" => {
            processor
                .authority_accept(
                    payload,
                    &caller,
                    crate::platform::auth::AuthorityProposalKind::Migration,
                )
                .await
        }
        "rpc.v1.Auth.DeploymentAuthority.Reject" => {
            processor.authority_reject(payload, &caller).await
        }
        "rpc.v1.Auth.DeploymentAuthority.Reconcile" => {
            processor
                .deployment_authority_reconcile(payload, &caller)
                .await
        }
        "rpc.v1.Auth.IdentityAuthority.List" => processor.identity_authority_list(payload).await,
        "rpc.v1.Auth.IdentityAuthority.Get" => processor.identity_authority_get(payload).await,
        "rpc.v1.Auth.IdentityAuthority.Revoke" => {
            processor.identity_authority_revoke(payload, &caller).await
        }
        _ => unknown(subject),
    }
}

fn unknown(subject: &str) -> Result<Value, AuthorizationStateError> {
    Err(AuthorizationStateError::InvalidRecord(format!(
        "Auth RPC is not implemented by Rust: {subject}"
    )))
}
