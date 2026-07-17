//! Runtime-wide singleton ownership lifecycle.

use std::collections::{hash_map::DefaultHasher, BTreeMap};
use std::future::{poll_fn, Future};
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::time::Duration;

use async_nats::jetstream;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

use crate::leases::{LeaseError, LeaseFence, LeaseGuard, LeaseKey, LeaseManager};
use crate::shutdown::StopHandle;
use crate::supervisor::{RuntimeError, OWNERSHIP_SHUTDOWN_TIMEOUT};
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
    guards: Arc<Vec<Mutex<HeldLease>>>,
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
                source: Box::new(source),
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

        let guards = Arc::new(held.into_iter().map(Mutex::new).collect::<Vec<_>>());
        let owners = match complete_acquisition(
            renew_round(&manager, &owner_id, &guards),
            move || owners,
        )
        .await
        {
            Ok(owners) => owners,
            Err(primary) => {
                if let Err(cleanup) = release_owned(&manager, &owner_id, &guards).await {
                    tracing::error!(error = %cleanup, "failed to clean up ownership after acquisition verification");
                }
                return Err(primary);
            }
        };

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

    pub(crate) async fn shutdown(mut self) -> Result<(), RuntimeError> {
        self.renewal_stop.stop();
        let mut first_error = None;
        if !self.renewal_joined {
            match tokio::time::timeout(OWNERSHIP_SHUTDOWN_TIMEOUT, &mut self.renewal).await {
                Ok(Ok(Ok(()))) => {}
                Ok(Ok(Err(error))) => first_error = Some(error),
                Ok(Err(source)) => {
                    first_error = Some(RuntimeError::OwnerRenewalTaskFailed {
                        owner_id: self.owner_id.clone(),
                        source,
                    });
                }
                Err(_) => {
                    self.renewal.abort();
                    let _ = (&mut self.renewal).await;
                    first_error = Some(RuntimeError::OwnerRenewalShutdownTimeout {
                        owner_id: self.owner_id.clone(),
                    });
                }
            }
        }

        if let Err(error) = release_owned(&self.manager, &self.owner_id, &self.guards).await {
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
    guards: Arc<Vec<Mutex<HeldLease>>>,
    stop: StopHandle,
) -> Result<(), RuntimeError> {
    let initial_delay = manager.renew + renewal_jitter(&owner_id, manager.renew, manager.ttl);
    tokio::select! {
        () = stop.stopped() => return Ok(()),
        () = tokio::time::sleep(initial_delay) => {}
    }

    let mut next_round = tokio::time::Instant::now();
    loop {
        renew_round(&manager, &owner_id, &guards).await?;
        next_round += manager.renew;
        tokio::select! {
            () = stop.stopped() => return Ok(()),
            () = tokio::time::sleep_until(next_round) => {}
        }
    }
}

async fn renew_round(
    manager: &LeaseManager,
    owner_id: &str,
    guards: &[Mutex<HeldLease>],
) -> Result<(), RuntimeError> {
    let renewals = guards.iter().map(|lease| async move {
        let mut lease = lease.lock().await;
        let group = lease.group;
        let key = lease.guard.key().clone();
        manager
            .renew(&mut lease.guard)
            .await
            .map_err(|source| map_renewal_error(group, key, owner_id, source))
    });
    match complete_renewal_round(renewals, manager.renew).await {
        Ok(()) => Ok(()),
        Err(RenewalRoundFailure::Operation(error)) => Err(error),
        Err(RenewalRoundFailure::Timeout) => Err(RuntimeError::OwnerRenewalRoundTimeout {
            owner_id: owner_id.to_owned(),
        }),
    }
}

#[derive(Debug)]
enum RenewalRoundFailure<E> {
    Operation(E),
    Timeout,
}

async fn complete_renewal_round<F, E>(
    renewals: impl IntoIterator<Item = F>,
    timeout: Duration,
) -> Result<(), RenewalRoundFailure<E>>
where
    F: Future<Output = Result<(), E>>,
{
    let mut renewals = renewals
        .into_iter()
        .map(|renewal| Some(Box::pin(renewal)))
        .collect::<Vec<_>>();
    let round = poll_fn(move |context| {
        let mut pending = false;
        let mut first_error = None;
        for renewal in &mut renewals {
            let result = match renewal.as_mut() {
                Some(renewal) => renewal.as_mut().poll(context),
                None => continue,
            };
            match result {
                std::task::Poll::Ready(Ok(())) => *renewal = None,
                std::task::Poll::Ready(Err(error)) => {
                    *renewal = None;
                    if first_error.is_none() {
                        first_error = Some(error);
                    }
                }
                std::task::Poll::Pending => pending = true,
            }
        }
        if let Some(error) = first_error {
            std::task::Poll::Ready(Err(error))
        } else if pending {
            std::task::Poll::Pending
        } else {
            std::task::Poll::Ready(Ok(()))
        }
    });
    match tokio::time::timeout(timeout, round).await {
        Ok(Ok(())) => Ok(()),
        Ok(Err(error)) => Err(RenewalRoundFailure::Operation(error)),
        Err(_) => Err(RenewalRoundFailure::Timeout),
    }
}

async fn complete_acquisition<F, C, T, E>(verification: F, contexts: C) -> Result<T, E>
where
    F: Future<Output = Result<(), E>>,
    C: FnOnce() -> T,
{
    verification.await?;
    Ok(contexts())
}

async fn release_owned(
    manager: &LeaseManager,
    owner_id: &str,
    held: &[Mutex<HeldLease>],
) -> Result<(), RuntimeError> {
    let mut first_error = None;
    for lease in held.iter().rev() {
        let lease = lease.lock().await;
        let group = lease.group;
        let guard = lease.guard.clone();
        let key = guard.key().clone();
        let release = tokio::time::timeout(OWNERSHIP_SHUTDOWN_TIMEOUT, manager.release(guard))
            .await
            .unwrap_or_else(|_| {
                Err(LeaseError::Backend {
                    key: Some(key.clone()),
                    operation: "release",
                    message: "operation exceeded shutdown bound".to_owned(),
                })
            });
        if let Err(source) = release {
            let error = RuntimeError::OwnerRelease {
                subsystem: group.subsystem(),
                key,
                owner_id: owner_id.to_owned(),
                source: Box::new(source),
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

async fn release_held(
    manager: &LeaseManager,
    owner_id: &str,
    held: &mut Vec<HeldLease>,
) -> Result<(), RuntimeError> {
    let mut first_error = None;
    while let Some(lease) = held.pop() {
        let group = lease.group;
        let key = lease.guard.key().clone();
        let release =
            tokio::time::timeout(OWNERSHIP_SHUTDOWN_TIMEOUT, manager.release(lease.guard))
                .await
                .unwrap_or_else(|_| {
                    Err(LeaseError::Backend {
                        key: Some(key.clone()),
                        operation: "release",
                        message: "operation exceeded shutdown bound".to_owned(),
                    })
                });
        if let Err(source) = release {
            let error = RuntimeError::OwnerRelease {
                subsystem: group.subsystem(),
                key,
                owner_id: owner_id.to_owned(),
                source: Box::new(source),
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
            source: Box::new(source),
        }
    } else {
        RuntimeError::OwnerAcquire {
            subsystem: group.subsystem(),
            key,
            owner_id: owner_id.to_owned(),
            source: Box::new(source),
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
        source: Box::new(source),
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
    use std::sync::atomic::{AtomicUsize, Ordering};

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

    #[tokio::test]
    async fn final_verification_starts_all_operations_concurrently_and_is_bounded() {
        let starts = Arc::new(AtomicUsize::new(0));
        let renewals = (0..4).map(|_| {
            let starts = Arc::clone(&starts);
            async move {
                starts.fetch_add(1, Ordering::SeqCst);
                std::future::pending::<Result<(), ()>>().await
            }
        });

        let result = complete_renewal_round(renewals, Duration::from_millis(20)).await;

        assert!(matches!(result, Err(RenewalRoundFailure::Timeout)));
        assert_eq!(starts.load(Ordering::SeqCst), 4);
    }

    #[tokio::test]
    async fn final_verification_starts_every_operation_before_reporting_failure() {
        let starts = Arc::new(AtomicUsize::new(0));
        let renewals = (0..4).map(|index| {
            let starts = Arc::clone(&starts);
            async move {
                starts.fetch_add(1, Ordering::SeqCst);
                if index == 0 {
                    Err("stale")
                } else {
                    std::future::pending().await
                }
            }
        });

        let result = tokio::time::timeout(
            Duration::from_millis(100),
            complete_renewal_round(renewals, Duration::from_secs(5)),
        )
        .await
        .expect("confirmed ownership loss must not wait for stalled renewals");

        assert!(matches!(
            result,
            Err(RenewalRoundFailure::Operation("stale"))
        ));
        assert_eq!(starts.load(Ordering::SeqCst), 4);
    }

    #[tokio::test]
    async fn early_guard_does_not_age_while_a_later_verification_is_stalled() {
        let completions = Arc::new(AtomicUsize::new(0));
        let completed = {
            let completions = Arc::clone(&completions);
            async move {
                completions.fetch_add(1, Ordering::SeqCst);
                Ok(())
            }
        };
        let stalled = async { std::future::pending::<Result<(), ()>>().await };

        let result = complete_renewal_round(
            [
                futures_util::future::Either::Left(completed),
                futures_util::future::Either::Right(stalled),
            ],
            Duration::from_millis(20),
        )
        .await;

        assert!(matches!(result, Err(RenewalRoundFailure::Timeout)));
        assert_eq!(completions.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn renewal_error_is_fatal_without_reacquisition() {
        let attempts = Arc::new(AtomicUsize::new(0));
        let renewal = {
            let attempts = Arc::clone(&attempts);
            async move {
                attempts.fetch_add(1, Ordering::SeqCst);
                Err("lost")
            }
        };

        let result = complete_renewal_round([renewal], Duration::from_secs(1)).await;

        assert!(matches!(
            result,
            Err(RenewalRoundFailure::Operation("lost"))
        ));
        assert_eq!(attempts.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn failed_final_verification_does_not_return_owner_contexts() {
        let contexts_built = Arc::new(AtomicUsize::new(0));
        let build_count = Arc::clone(&contexts_built);

        let result = complete_acquisition(std::future::ready(Err("stale")), move || {
            build_count.fetch_add(1, Ordering::SeqCst);
            "owner contexts"
        })
        .await;

        assert_eq!(result, Err("stale"));
        assert_eq!(contexts_built.load(Ordering::SeqCst), 0);
    }
}
