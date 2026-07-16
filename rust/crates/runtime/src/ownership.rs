//! Runtime-wide singleton ownership lifecycle.

use std::collections::{hash_map::DefaultHasher, BTreeMap};
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::time::Duration;

use async_nats::jetstream;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

use crate::leases::{LeaseError, LeaseFence, LeaseGuard, LeaseKey, LeaseManager};
use crate::shutdown::StopHandle;
use crate::supervisor::RuntimeError;
use crate::{ResolvedLeasesConfig, RuntimeMode, SubsystemName};

/// Stable runtime singleton owner group.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum OwnerGroup {
    Platform,
    Jobs,
    Health,
    Eventlog,
}

impl OwnerGroup {
    pub(crate) fn for_mode(mode: RuntimeMode) -> &'static [Self] {
        match mode {
            RuntimeMode::All => &[Self::Platform, Self::Jobs, Self::Health, Self::Eventlog],
            RuntimeMode::Platform => &[Self::Platform],
            RuntimeMode::Jobs => &[Self::Jobs],
            RuntimeMode::Health => &[Self::Health],
            RuntimeMode::Eventlog => &[Self::Eventlog],
        }
    }

    pub(crate) const fn key(self) -> &'static str {
        match self {
            Self::Platform => "platform.owner",
            Self::Jobs => "jobs.owner",
            Self::Health => "health.owner",
            Self::Eventlog => "eventlog.owner",
        }
    }

    pub(crate) const fn subsystem(self) -> SubsystemName {
        match self {
            Self::Platform => SubsystemName::Platform,
            Self::Jobs => SubsystemName::Jobs,
            Self::Health => SubsystemName::Health,
            Self::Eventlog => SubsystemName::Eventlog,
        }
    }
}

/// Fixed ownership information passed to a selected subsystem.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct OwnerContext {
    pub(crate) group: OwnerGroup,
    pub(crate) key: LeaseKey,
    pub(crate) fence: LeaseFence,
}

#[derive(Debug)]
struct HeldLease {
    group: OwnerGroup,
    guard: LeaseGuard,
}

/// Owns all selected runtime leases and their supervised renewal task.
#[derive(Debug)]
pub(crate) struct RuntimeOwnership {
    manager: LeaseManager,
    owner_id: String,
    owners: BTreeMap<OwnerGroup, OwnerContext>,
    guards: Arc<Mutex<Vec<HeldLease>>>,
    renewal_stop: StopHandle,
    renewal: JoinHandle<Result<(), RuntimeError>>,
    renewal_joined: bool,
}

impl RuntimeOwnership {
    pub(crate) async fn acquire(
        client: async_nats::Client,
        config: &ResolvedLeasesConfig,
        owner_id: String,
        mode: RuntimeMode,
    ) -> Result<Self, RuntimeError> {
        let manager = LeaseManager::open(jetstream::new(client), config, owner_id.clone())
            .await
            .map_err(|source| RuntimeError::LeaseBucketOpen {
                owner_id: owner_id.clone(),
                source,
            })?;
        let mut held = Vec::new();
        let mut owners = BTreeMap::new();

        for &group in OwnerGroup::for_mode(mode) {
            let key = LeaseKey::new(group.key());
            match manager.acquire(key.clone()).await {
                Ok(guard) => {
                    owners.insert(
                        group,
                        OwnerContext {
                            group,
                            key,
                            fence: guard.fence(),
                        },
                    );
                    held.push(HeldLease { group, guard });
                }
                Err(source) => {
                    let primary = map_acquisition_error(group, key, &owner_id, source);
                    if let Err(cleanup) = release_held(&manager, &owner_id, &mut held).await {
                        tracing::error!(error = %cleanup, "failed to clean up partially acquired runtime ownership");
                    }
                    return Err(primary);
                }
            }
        }

        let guards = Arc::new(Mutex::new(held));
        let renewal_stop = StopHandle::new();
        let renewal = tokio::spawn(renew_owned(
            manager.clone(),
            owner_id.clone(),
            Arc::clone(&guards),
            renewal_stop.clone(),
        ));

        Ok(Self {
            manager,
            owner_id,
            owners,
            guards,
            renewal_stop,
            renewal,
            renewal_joined: false,
        })
    }

    pub(crate) fn contexts(&self) -> BTreeMap<OwnerGroup, OwnerContext> {
        self.owners.clone()
    }

    pub(crate) async fn wait_for_renewal_failure(&mut self) -> RuntimeError {
        let result = (&mut self.renewal).await;
        self.renewal_joined = true;
        match result {
            Ok(Err(error)) => error,
            Ok(Ok(())) => RuntimeError::OwnerRenewalTaskExited {
                owner_id: self.owner_id.clone(),
            },
            Err(source) => RuntimeError::OwnerRenewalTaskFailed {
                owner_id: self.owner_id.clone(),
                source,
            },
        }
    }

    pub(crate) async fn shutdown(self) -> Result<(), RuntimeError> {
        self.renewal_stop.stop();
        let mut first_error = None;
        if !self.renewal_joined {
            match self.renewal.await {
                Ok(Ok(())) => {}
                Ok(Err(error)) => first_error = Some(error),
                Err(source) => {
                    first_error = Some(RuntimeError::OwnerRenewalTaskFailed {
                        owner_id: self.owner_id.clone(),
                        source,
                    });
                }
            }
        }

        let mut guards = self.guards.lock().await;
        if let Err(error) = release_held(&self.manager, &self.owner_id, &mut guards).await {
            if first_error.is_some() {
                tracing::error!(error = %error, "runtime ownership release failed during shutdown");
            } else {
                first_error = Some(error);
            }
        }
        first_error.map_or(Ok(()), Err)
    }
}

async fn renew_owned(
    manager: LeaseManager,
    owner_id: String,
    guards: Arc<Mutex<Vec<HeldLease>>>,
    stop: StopHandle,
) -> Result<(), RuntimeError> {
    let initial_delay = manager.renew + renewal_jitter(&owner_id, manager.renew, manager.ttl);
    tokio::select! {
        () = stop.stopped() => return Ok(()),
        () = tokio::time::sleep(initial_delay) => {}
    }

    loop {
        {
            let mut held = guards.lock().await;
            for lease in held.iter_mut() {
                if let Err(source) = manager.renew(&mut lease.guard).await {
                    return Err(map_renewal_error(
                        lease.group,
                        lease.guard.key().clone(),
                        &owner_id,
                        source,
                    ));
                }
            }
        }
        tokio::select! {
            () = stop.stopped() => return Ok(()),
            () = tokio::time::sleep(manager.renew) => {}
        }
    }
}

async fn release_held(
    manager: &LeaseManager,
    owner_id: &str,
    held: &mut Vec<HeldLease>,
) -> Result<(), RuntimeError> {
    let mut first_error = None;
    while let Some(lease) = held.pop() {
        let group = lease.group;
        let key = lease.guard.key().clone();
        if let Err(source) = manager.release(lease.guard).await {
            let error = RuntimeError::OwnerRelease {
                subsystem: group.subsystem(),
                key,
                owner_id: owner_id.to_owned(),
                source,
            };
            if first_error.is_some() {
                tracing::error!(error = %error, "additional runtime owner release failed");
            } else {
                first_error = Some(error);
            }
        }
    }
    first_error.map_or(Ok(()), Err)
}

fn map_acquisition_error(
    group: OwnerGroup,
    key: LeaseKey,
    owner_id: &str,
    source: LeaseError,
) -> RuntimeError {
    if matches!(source, LeaseError::Held { .. }) {
        RuntimeError::OwnerHeld {
            subsystem: group.subsystem(),
            key,
            owner_id: owner_id.to_owned(),
            source,
        }
    } else {
        RuntimeError::OwnerAcquire {
            subsystem: group.subsystem(),
            key,
            owner_id: owner_id.to_owned(),
            source,
        }
    }
}

fn map_renewal_error(
    group: OwnerGroup,
    key: LeaseKey,
    owner_id: &str,
    source: LeaseError,
) -> RuntimeError {
    RuntimeError::OwnerRenewal {
        subsystem: group.subsystem(),
        key,
        owner_id: owner_id.to_owned(),
        source,
    }
}

fn renewal_jitter(owner_id: &str, renew: Duration, ttl: Duration) -> Duration {
    let margin = ttl
        .saturating_sub(renew)
        .saturating_sub(Duration::from_millis(1));
    let bound = (renew / 5).min(margin).as_millis() as u64;
    if bound == 0 {
        return Duration::ZERO;
    }
    let mut hasher = DefaultHasher::new();
    owner_id.hash(&mut hasher);
    Duration::from_millis(hasher.finish() % (bound + 1))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mode_owner_groups_use_stable_subsystem_order_and_keys() {
        assert_eq!(
            OwnerGroup::for_mode(RuntimeMode::All),
            &[
                OwnerGroup::Platform,
                OwnerGroup::Jobs,
                OwnerGroup::Health,
                OwnerGroup::Eventlog,
            ]
        );
        assert_eq!(OwnerGroup::Platform.key(), "platform.owner");
        assert_eq!(OwnerGroup::Jobs.key(), "jobs.owner");
        assert_eq!(OwnerGroup::Health.key(), "health.owner");
        assert_eq!(OwnerGroup::Eventlog.key(), "eventlog.owner");
    }

    #[test]
    fn partial_acquisition_cleanup_plans_reverse_order() {
        let acquired = &OwnerGroup::for_mode(RuntimeMode::All)[..2];
        assert_eq!(
            acquired.iter().rev().copied().collect::<Vec<_>>(),
            vec![OwnerGroup::Jobs, OwnerGroup::Platform]
        );
    }

    #[test]
    fn owner_held_and_lease_loss_map_to_typed_errors() {
        let key = LeaseKey::new("jobs.owner");
        let held = map_acquisition_error(
            OwnerGroup::Jobs,
            key.clone(),
            "owner-1",
            LeaseError::Held { key: key.clone() },
        );
        assert!(matches!(held, RuntimeError::OwnerHeld { .. }));

        let lost = map_renewal_error(
            OwnerGroup::Jobs,
            key.clone(),
            "owner-1",
            LeaseError::Stale { key },
        );
        assert!(matches!(lost, RuntimeError::OwnerRenewal { .. }));
    }

    #[test]
    fn renewal_jitter_is_bounded_by_safe_margin() {
        let renew = Duration::from_secs(5);
        let ttl = Duration::from_secs(15);
        let jitter = renewal_jitter("owner-1", renew, ttl);
        assert!(jitter <= Duration::from_secs(1));
        assert!(renew + jitter < ttl);
    }
}
