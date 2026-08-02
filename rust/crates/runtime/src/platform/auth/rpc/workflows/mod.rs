mod authority;
mod deployments;
mod devices;
mod portals;
mod sessions;
mod users;

use super::{AuthRpcProcessor, AuthorizationStateError, ValidatedRequest, Value};

pub(super) async fn dispatch(
    processor: &AuthRpcProcessor,
    subject: &str,
    payload: &[u8],
    caller: ValidatedRequest,
) -> Result<Value, AuthorizationStateError> {
    if subject.starts_with("rpc.v1.Auth.Sessions.")
        || subject.starts_with("rpc.v1.Auth.Connections.")
    {
        sessions::dispatch(processor, subject, payload, caller).await
    } else if subject.starts_with("rpc.v1.Auth.Portals.")
        || subject == "rpc.v1.Auth.Capabilities.List"
    {
        portals::dispatch(processor, subject, payload, caller).await
    } else if subject.starts_with("rpc.v1.Auth.Users.")
        || subject.starts_with("rpc.v1.Auth.UserIdentities.")
    {
        users::dispatch(processor, subject, payload, caller).await
    } else if subject.starts_with("rpc.v1.Auth.Deployments.")
        || subject.starts_with("rpc.v1.Auth.ServiceInstances.")
    {
        deployments::dispatch(processor, subject, payload, caller).await
    } else if subject.starts_with("rpc.v1.Auth.Devices.")
        || subject.starts_with("rpc.v1.Auth.DeviceUserAuthorities.")
    {
        devices::dispatch(processor, subject, payload, caller).await
    } else if subject.starts_with("rpc.v1.Auth.DeploymentAuthority.")
        || subject.starts_with("rpc.v1.Auth.IdentityAuthority.")
    {
        authority::dispatch(processor, subject, payload, caller).await
    } else {
        Err(AuthorizationStateError::InvalidRecord(format!(
            "Auth RPC is not implemented by Rust: {subject}"
        )))
    }
}
