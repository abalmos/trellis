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
        || subject.starts_with("rpc.v1.Auth.CapabilityGroups.")
        || subject.starts_with("rpc.v1.Auth.IdentityGrants.")
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

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use serde_json::Value;

    const DISPATCH_SOURCE: &str = concat!(
        include_str!("sessions.rs"),
        include_str!("portals.rs"),
        include_str!("users.rs"),
        include_str!("deployments.rs"),
        include_str!("devices.rs"),
        include_str!("authority.rs"),
    );

    #[test]
    fn every_native_auth_rpc_has_a_rust_dispatch_arm() {
        let api: Value = serde_json::from_str(include_str!("../../../../../trellis.api.json"))
            .expect("parse native Auth API");
        let missing = api["rpc"]
            .as_object()
            .expect("Auth RPC map")
            .keys()
            .filter(|name| !name.starts_with("_removed."))
            .map(|name| format!("rpc.v1.{name}"))
            .filter(|subject| !DISPATCH_SOURCE.contains(&format!("\"{subject}\"")))
            .collect::<BTreeSet<_>>();
        assert!(
            missing.is_empty(),
            "Auth RPCs without Rust dispatch: {missing:?}"
        );
    }
}
