//! Platform subsystem scaffold.

/// Rust-owned authorization state and materialization.
pub mod auth;
pub mod auth_callout;
pub mod bootstrap;
pub mod catalog;

use auth::{
    authorization_reconciliation_channel, AuthorizationStateService, SqliteAuthorizationStore,
};

use crate::shutdown::StopHandle;
use crate::supervisor::{RuntimeContext, RuntimeError, SubsystemHandle};
use crate::SubsystemName;

pub(crate) async fn start(context: &RuntimeContext) -> Result<SubsystemHandle, RuntimeError> {
    let _owner = context.owner(crate::ownership::OwnerGroup::Platform)?;
    let auth_store = SqliteAuthorizationStore::open(context.stores.platform()?)?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|error| RuntimeError::Platform(error.to_string()))?
        .as_millis()
        .try_into()
        .map_err(|_| RuntimeError::Platform("current time exceeds i64 milliseconds".to_owned()))?;
    let authorization = AuthorizationStateService::new(auth_store);
    authorization
        .reconcile_all(now)
        .await
        .map_err(|error| RuntimeError::Platform(error.to_string()))?;
    let (reconciliation, reconciliation_worker) =
        authorization_reconciliation_channel(authorization.clone(), 256);
    let stop = StopHandle::new();
    let task_stop = stop.clone();
    let join = tokio::spawn(async move {
        let _authorization = authorization;
        let _reconciliation = reconciliation;
        reconciliation_worker
            .run(task_stop)
            .await
            .map_err(|error| RuntimeError::Platform(error.to_string()))
    });

    Ok(SubsystemHandle {
        name: SubsystemName::Platform,
        stop,
        join,
    })
}
