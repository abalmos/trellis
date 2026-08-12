use super::super::{AuthRpcProcessor, AuthorizationStateError, ValidatedRequest, Value};

pub(super) async fn dispatch(
    processor: &AuthRpcProcessor,
    subject: &str,
    payload: &[u8],
    caller: ValidatedRequest,
) -> Result<Value, AuthorizationStateError> {
    match subject {
        "rpc.v1.Auth.Portals.List" => processor.portals_list(payload).await,
        "rpc.v1.Auth.Portals.Get" => processor.portals_get(payload).await,
        "rpc.v1.Auth.Portals.Put" => processor.portals_put(payload, &caller).await,
        "rpc.v1.Auth.Portals.Remove" => processor.portals_remove(payload, &caller).await,
        "rpc.v1.Auth.Portals.LoginSettings.Get" => processor.portal_settings_get(payload).await,
        "rpc.v1.Auth.Portals.LoginSettings.Update" => {
            processor.portal_settings_update(payload, &caller).await
        }
        "rpc.v1.Auth.Portals.Routes.Put" => processor.portal_route_put(payload, &caller).await,
        "rpc.v1.Auth.Portals.Routes.Remove" => {
            processor.portal_route_remove(payload, &caller).await
        }
        "rpc.v1.Auth.Portals.GrantOverrides.List" => {
            processor.portal_grant_overrides_list(payload).await
        }
        "rpc.v1.Auth.Portals.GrantOverrides.Put" => {
            processor.portal_grant_overrides_put(payload, &caller).await
        }
        "rpc.v1.Auth.Portals.GrantOverrides.Remove" => {
            processor
                .portal_grant_overrides_remove(payload, &caller)
                .await
        }
        "rpc.v1.Auth.Capabilities.List" => processor.capabilities_list(payload).await,
        _ => unknown(subject),
    }
}

fn unknown(subject: &str) -> Result<Value, AuthorizationStateError> {
    Err(AuthorizationStateError::InvalidRecord(format!(
        "Auth RPC is not implemented by Rust: {subject}"
    )))
}
