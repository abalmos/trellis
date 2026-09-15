//! Read-only telemetry snapshot samplers for platform components.
//!
//! Samplers poll existing authoritative state on a fixed interval and publish
//! immutable numeric snapshots to observable gauges. They never mutate
//! business state, never hold a lease, and never run exporter work inside a
//! gauge callback. A failed poll keeps the previous values and advances only
//! the error counter; genuine zeros are published as zeros.

use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use trellis_rs::telemetry::instruments::{self, ObservableFamily};
use trellis_rs::telemetry::KeyValue;

use crate::platform::auth::{AuthorizationStateError, SqliteAuthorizationStore};
use crate::shutdown::StopHandle;
use trellis_events_runtime::storage::EventsStore;
use trellis_jobs_runtime::storage::SqliteJobsStore;

/// Snapshot poll interval.
const SAMPLER_INTERVAL: Duration = Duration::from_secs(15);
/// Total deadline for one snapshot read.
const SNAPSHOT_DEADLINE: Duration = Duration::from_secs(2);

/// Latest published values for one snapshot source.
#[derive(Default)]
struct SnapshotValues {
    observed_at: f64,
    gauges: Vec<(ObservableFamily, f64, Vec<KeyValue>)>,
}

/// One registered snapshot source with its gauge callbacks.
struct SnapshotSource {
    values: Arc<Mutex<SnapshotValues>>,
}

impl SnapshotSource {
    /// Registers the observed-time gauge plus the named gauge families.
    ///
    /// `source` is `None` for sources outside the snapshot freshness contract
    /// (for example supervisor component state).
    fn register(source: Option<&'static str>, families: &[ObservableFamily]) -> Self {
        let values = Arc::new(Mutex::new(SnapshotValues::default()));
        if let Some(source) = source {
            let observed = Arc::clone(&values);
            instruments::register_observable(
                ObservableFamily::SnapshotObservedTime,
                Arc::new(move || {
                    let Ok(guard) = observed.lock() else {
                        return Vec::new();
                    };
                    vec![(
                        guard.observed_at,
                        vec![KeyValue::new("trellis.source", source)],
                    )]
                }),
            );
        }
        for family in families {
            let values = Arc::clone(&values);
            let family = *family;
            instruments::register_observable(
                family,
                Arc::new(move || {
                    let Ok(guard) = values.lock() else {
                        return Vec::new();
                    };
                    guard
                        .gauges
                        .iter()
                        .filter(|(registered, _, _)| *registered == family)
                        .map(|(_, value, attributes)| (*value, attributes.clone()))
                        .collect()
                }),
            );
        }
        Self { values }
    }

    /// Replaces the gauge set and marks the snapshot observed.
    fn publish(&self, gauges: Vec<(ObservableFamily, f64, Vec<KeyValue>)>) {
        let Ok(mut values) = self.values.lock() else {
            return;
        };
        values.gauges = gauges;
        values.observed_at = unix_seconds();
    }
}

/// Current wall-clock time in Unix seconds.
fn unix_seconds() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0.0, |duration| duration.as_secs_f64())
}

/// Records one failed snapshot attempt with a bounded reason.
fn record_failure(source: &'static str, reason: &'static str) {
    instruments::add_counter(
        instruments::CounterFamily::SnapshotErrors,
        1,
        &[
            KeyValue::new("trellis.source", source),
            KeyValue::new("trellis.reason", reason),
        ],
    );
}

/// Runs the Jobs snapshot sampler until the runtime stops.
pub(crate) async fn run_jobs_sampler(store: SqliteJobsStore, stop: StopHandle) {
    let source = SnapshotSource::register(
        Some("jobs"),
        &[
            ObservableFamily::JobsReady,
            ObservableFamily::JobsOldestReadyAge,
            ObservableFamily::JobsDead,
            ObservableFamily::JobsWaitingRetry,
            ObservableFamily::JobsWorkerRegistrations,
        ],
    );
    loop {
        tokio::select! {
            _ = tokio::time::sleep(SAMPLER_INTERVAL) => {}
            _ = stop.stopped() => return,
        }
        let now = time::OffsetDateTime::now_utc();
        let now_nanos = now.unix_timestamp_nanos() as i64;
        let snapshot_store = store.clone();
        let deadline = tokio::time::timeout(SNAPSHOT_DEADLINE, async move {
            snapshot_store.telemetry_snapshot(now_nanos, now)
        })
        .await;
        match deadline {
            Ok(Ok(snapshot)) => {
                source.publish(vec![
                    (
                        ObservableFamily::JobsReady,
                        snapshot.ready as f64,
                        Vec::new(),
                    ),
                    (
                        ObservableFamily::JobsOldestReadyAge,
                        snapshot.oldest_ready_age_seconds,
                        Vec::new(),
                    ),
                    (ObservableFamily::JobsDead, snapshot.dead as f64, Vec::new()),
                    (
                        ObservableFamily::JobsWaitingRetry,
                        snapshot.waiting_retry as f64,
                        Vec::new(),
                    ),
                    (
                        ObservableFamily::JobsWorkerRegistrations,
                        snapshot.worker_registrations as f64,
                        Vec::new(),
                    ),
                ]);
            }
            Ok(Err(_)) => {
                record_failure("jobs", "io");
            }
            Err(_) => {
                record_failure("jobs", "timeout");
            }
        }
    }
}

/// Runs the Events dead-letter snapshot sampler until the runtime stops.
pub(crate) async fn run_events_sampler(store: EventsStore, stop: StopHandle) {
    let source = SnapshotSource::register(Some("events_dlq"), &[ObservableFamily::DeadLetters]);
    loop {
        tokio::select! {
            _ = tokio::time::sleep(SAMPLER_INTERVAL) => {}
            _ = stop.stopped() => return,
        }
        match store.telemetry_snapshot() {
            Ok(snapshot) => source.publish(vec![
                (
                    ObservableFamily::DeadLetters,
                    snapshot.open as f64,
                    vec![KeyValue::new("trellis.state", "open")],
                ),
                (
                    ObservableFamily::DeadLetters,
                    snapshot.replay_pending as f64,
                    vec![KeyValue::new("trellis.state", "replay_pending")],
                ),
                (
                    ObservableFamily::DeadLetters,
                    snapshot.replaying as f64,
                    vec![KeyValue::new("trellis.state", "replaying")],
                ),
            ]),
            Err(_) => record_failure("events_dlq", "io"),
        }
    }
}

/// Publishes selected component readiness and observation time.
pub(crate) fn spawn_component_sampler(
    components: Vec<&'static str>,
    stop: StopHandle,
) -> tokio::task::JoinHandle<()> {
    let source = SnapshotSource::register(
        None,
        &[
            ObservableFamily::ComponentReady,
            ObservableFamily::ComponentObservedTime,
        ],
    );
    tokio::spawn(async move {
        loop {
            let now = unix_seconds();
            let mut gauges = Vec::new();
            for component in &components {
                gauges.push((
                    ObservableFamily::ComponentReady,
                    1.0,
                    vec![KeyValue::new("trellis.component", *component)],
                ));
                gauges.push((
                    ObservableFamily::ComponentObservedTime,
                    now,
                    vec![KeyValue::new("trellis.component", *component)],
                ));
            }
            source.publish(gauges);
            tokio::select! {
                _ = tokio::time::sleep(SAMPLER_INTERVAL) => {}
                _ = stop.stopped() => return,
            }
        }
    })
}

/// Runs the platform Auth snapshot sampler until the runtime stops.
pub(crate) async fn run_auth_sampler(
    store: SqliteAuthorizationStore,
    stop: StopHandle,
) -> Result<(), AuthorizationStateError> {
    let source = SnapshotSource::register(
        Some("auth"),
        &[
            ObservableFamily::AuthPostCommitPending,
            ObservableFamily::AuthPostCommitOldestAge,
            ObservableFamily::ResourceBindings,
        ],
    );
    loop {
        tokio::select! {
            _ = tokio::time::sleep(SAMPLER_INTERVAL) => {}
            _ = stop.stopped() => return Ok(()),
        }
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |duration| duration.as_millis() as i64);
        let deadline =
            tokio::time::timeout(SNAPSHOT_DEADLINE, store.telemetry_snapshot(now_ms)).await;
        match deadline {
            Ok(Ok(snapshot)) => {
                let mut gauges = vec![
                    (
                        ObservableFamily::AuthPostCommitPending,
                        snapshot.post_commit_pending as f64,
                        Vec::new(),
                    ),
                    (
                        ObservableFamily::AuthPostCommitOldestAge,
                        snapshot.post_commit_oldest_age_seconds,
                        Vec::new(),
                    ),
                ];
                for (kind, state, count) in &snapshot.resources {
                    gauges.push((
                        ObservableFamily::ResourceBindings,
                        *count as f64,
                        vec![
                            KeyValue::new("trellis.kind", kind.clone()),
                            KeyValue::new("trellis.state", state.clone()),
                        ],
                    ));
                }
                source.publish(gauges);
            }
            Ok(Err(_)) => {
                record_failure("auth", "io");
            }
            Err(_) => {
                record_failure("auth", "timeout");
            }
        }
    }
}
