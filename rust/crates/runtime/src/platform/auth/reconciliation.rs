use std::time::{SystemTime, UNIX_EPOCH};

use tokio::sync::mpsc;

use super::{
    AuthorityTarget, AuthorizationStateError, AuthorizationStateService, ContextRepository,
};
use crate::shutdown::StopHandle;

/// Authorization state change that requires authority-level reconciliation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReconciliationCause {
    /// Desired identity or deployment authority changed.
    DesiredAuthorityChanged,
    /// An identity-authority principal changed.
    PrincipalChanged,
    /// An exact participant binding changed.
    ParticipantChanged,
    /// Deployment-level state changed.
    DeploymentChanged,
    /// Required or optional dependency evidence changed.
    DependencyEvidenceChanged,
    /// Resource-binding evidence changed.
    ResourceEvidenceChanged,
    /// An authority or deployment expiry became due.
    ExpiryReached,
    /// Startup convergence found an authority requiring reconciliation.
    StartupRepair,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ReconciliationRequest {
    target: AuthorityTarget,
    cause: ReconciliationCause,
}

/// Bounded event-driven handle for scheduling typed authority reconciliation.
#[derive(Clone, Debug)]
pub struct AuthorizationReconciliationHandle {
    sender: mpsc::Sender<ReconciliationRequest>,
}

impl AuthorizationReconciliationHandle {
    /// Schedule one authority after a meaningful authority-level input change.
    ///
    /// # Errors
    ///
    /// Returns [`AuthorizationStateError::Storage`] when the owning platform task
    /// has stopped or bounded backpressure prevents accepting the trigger.
    pub async fn reconcile(
        &self,
        target: AuthorityTarget,
        cause: ReconciliationCause,
    ) -> Result<(), AuthorizationStateError> {
        self.sender
            .send(ReconciliationRequest { target, cause })
            .await
            .map_err(|_| {
                AuthorizationStateError::Storage(
                    "authorization reconciliation task is unavailable".to_owned(),
                )
            })
    }
}

pub(crate) struct AuthorizationReconciliationWorker<S> {
    service: AuthorizationStateService<S>,
    receiver: mpsc::Receiver<ReconciliationRequest>,
}

pub(crate) fn authorization_reconciliation_channel<S>(
    service: AuthorizationStateService<S>,
    capacity: usize,
) -> (
    AuthorizationReconciliationHandle,
    AuthorizationReconciliationWorker<S>,
) {
    let (sender, receiver) = mpsc::channel(capacity);
    (
        AuthorizationReconciliationHandle { sender },
        AuthorizationReconciliationWorker { service, receiver },
    )
}

impl<S> AuthorizationReconciliationWorker<S>
where
    S: ContextRepository,
{
    pub(crate) async fn run(mut self, stop: StopHandle) -> Result<(), AuthorizationStateError> {
        loop {
            let request = tokio::select! {
                biased;
                () = stop.stopped() => return Ok(()),
                request = self.receiver.recv() => request.ok_or_else(|| {
                    AuthorizationStateError::Storage(
                        "authorization reconciliation trigger channel closed".to_owned(),
                    )
                })?,
            };
            let now = unix_time_millis()?;
            self.service
                .reconcile_authority(&request.target, now)
                .await?;
        }
    }
}

pub(crate) fn unix_time_millis() -> Result<i64, AuthorizationStateError> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| {
            AuthorizationStateError::Storage(format!("system clock precedes Unix epoch: {error}"))
        })?;
    i64::try_from(duration.as_millis()).map_err(|_| {
        AuthorizationStateError::Storage("system time exceeds signed milliseconds".to_owned())
    })
}
