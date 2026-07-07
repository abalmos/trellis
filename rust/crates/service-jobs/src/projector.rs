use futures_util::StreamExt;
use serde_json::Value;
use trellis_rs::jobs::reduce_job_event;
use trellis_rs::jobs::types::{Job, JobEvent, JobEventType};
use trellis_rs::jobs::JobsRuntime;
use trellis_rs::service::ServerError;

use crate::storage::{
    JobConcurrencyMetadata, JobProjectionMetadataPatch, JobQueuePolicyMetadata, SqliteJobsStore,
    SqliteJobsStoreError,
};

const JOBS_EVENTS_SUBJECT_WILDCARD: &str = "trellis.jobs.>";
const PROJECTOR_CONSUMER_NAME: &str = "jobs-projector";

pub struct JobsProjectorHandle {
    task: Option<tokio::task::JoinHandle<Result<(), ServerError>>>,
}

impl JobsProjectorHandle {
    pub async fn stop(self) {
        let Some(task) = self.task else {
            return;
        };
        task.abort();
        let _ = task.await;
    }

    pub(crate) fn discard_completed(&mut self) {
        self.task = None;
    }

    pub async fn wait(&mut self) -> Result<(), ServerError> {
        let Some(task) = self.task.as_mut() else {
            return Ok(());
        };
        match task.await {
            Ok(result) => result,
            Err(error) if error.is_cancelled() => Ok(()),
            Err(error) => Err(ServerError::Nats(format!(
                "projector loop task failed: {error}"
            ))),
        }
    }
}

pub async fn start_jobs_projector(
    jobs_runtime: JobsRuntime,
    store: SqliteJobsStore,
    jobs_stream: String,
) -> Result<JobsProjectorHandle, ServerError> {
    let consumer_name = projector_consumer_name(&store.projection_id().map_err(|error| {
        ServerError::Nats(format!(
            "failed to resolve Jobs projection identity: {error}"
        ))
    })?);
    let mut messages = jobs_runtime
        .filtered_messages(&jobs_stream, &consumer_name, JOBS_EVENTS_SUBJECT_WILDCARD)
        .await
        .map_err(|error| {
            ServerError::Nats(format!(
                "failed to start jobs projector consumer '{consumer_name}' message stream: {error}"
            ))
        })?;

    let task = tokio::spawn(async move {
        while let Some(message) = messages.next().await {
            let message = message.map_err(|error| {
                ServerError::Nats(format!(
                    "jobs projector failed to pull from consumer '{consumer_name}' on stream '{jobs_stream}': {error}"
                ))
            })?;
            let raw_event = match serde_json::from_slice::<Value>(message.payload()) {
                Ok(raw_event) => raw_event,
                Err(_) => {
                    let _ = message.ack().await;
                    continue;
                }
            };
            let event = match serde_json::from_value::<JobEvent>(raw_event.clone()) {
                Ok(event) => event,
                Err(_) => {
                    let _ = message.ack().await;
                    continue;
                }
            };

            project_job_event_with_payload(&store, &event, &raw_event).map_err(|error| {
                ServerError::Nats(format!(
                    "jobs projector failed to project job '{}/{}/{}': {error}",
                    event.service, event.job_type, event.job_id
                ))
            })?;
            let _ = message.ack().await;
        }
        Ok(())
    });

    Ok(JobsProjectorHandle { task: Some(task) })
}

fn projector_consumer_name(projection_id: &str) -> String {
    format!("{PROJECTOR_CONSUMER_NAME}-{projection_id}")
}

#[cfg(test)]
fn project_job_event(
    store: &SqliteJobsStore,
    event: &JobEvent,
) -> Result<Option<Job>, SqliteJobsStoreError> {
    let raw_event =
        serde_json::to_value(event).map_err(|error| SqliteJobsStoreError::EncodeJson {
            model: "job event",
            details: error.to_string(),
        })?;
    project_job_event_with_payload(store, event, &raw_event)
}

pub fn project_job_event_with_payload(
    store: &SqliteJobsStore,
    event: &JobEvent,
    raw_event: &Value,
) -> Result<Option<Job>, SqliteJobsStoreError> {
    let current = store.get_job(&event.service, &event.job_type, &event.job_id)?;
    let next = reduce_job_event(current.as_ref(), event).or_else(|| job_from_terminal_event(event));
    let projected = next
        .as_ref()
        .zip(current.as_ref())
        .map(|(next, current)| next != current);
    let reason = if projected == Some(false) {
        if current
            .as_ref()
            .is_some_and(|job| trellis_rs::jobs::is_terminal(job.state))
        {
            Some("terminal-precedence")
        } else {
            Some("illegal-transition")
        }
    } else if next.is_none() {
        Some("illegal-transition")
    } else {
        None
    };
    store.project_timeline_event(event, raw_event, projected.or(Some(next.is_some())), reason)?;
    let fallback_detail = event.error.as_deref().map(|message| {
        trellis_rs::jobs::types::JobErrorDetail::from_message(
            &event.service,
            &event.job_type,
            message,
        )
    });
    if let Some(detail) = event.error_detail.as_ref().or(fallback_detail.as_ref()) {
        store.upsert_error_projection(
            &event.service,
            &event.job_type,
            event.state,
            &event.timestamp,
            detail,
        )?;
    }
    let Some(next) = next else {
        return Ok(None);
    };
    store.upsert_job(&next)?;
    store.upsert_job_lineage(&next)?;
    let metadata = metadata_patch_from_event_payload(raw_event);
    store.apply_job_metadata_patch(
        &event.service,
        &event.job_type,
        &event.job_id,
        &event.timestamp,
        &metadata,
    )?;
    Ok(Some(next))
}

fn metadata_patch_from_event_payload(raw_event: &Value) -> JobProjectionMetadataPatch {
    JobProjectionMetadataPatch {
        concurrency: raw_event
            .get("concurrency")
            .and_then(concurrency_metadata_from_value),
        queue_policy: raw_event
            .get("queuePolicy")
            .and_then(queue_policy_metadata_from_value),
    }
}

fn job_from_terminal_event(event: &JobEvent) -> Option<Job> {
    if !trellis_rs::jobs::is_terminal(event.state) {
        return None;
    }

    let mut job = Job {
        id: event.job_id.clone(),
        context: event.context.clone(),
        service: event.service.clone(),
        job_type: event.job_type.clone(),
        state: event.state,
        payload: event.payload.clone().unwrap_or(Value::Null),
        result: None,
        created_at: event.timestamp.clone(),
        updated_at: event.timestamp.clone(),
        started_at: None,
        completed_at: Some(event.timestamp.clone()),
        tries: event.tries,
        max_tries: event.max_tries.unwrap_or(event.tries.max(1)),
        last_error: event.error.clone(),
        error_detail: event.error_detail.clone().or_else(|| {
            event.error.as_deref().map(|message| {
                trellis_rs::jobs::types::JobErrorDetail::from_message(
                    &event.service,
                    &event.job_type,
                    message,
                )
            })
        }),
        deadline: event.deadline.clone(),
        progress: event.progress.clone(),
        logs: event.logs.clone(),
        concurrency: event.concurrency.clone(),
        queue_policy: event.queue_policy.clone(),
        trigger: event.trigger.clone(),
        lineage: event.lineage.clone(),
    };

    if event.event_type == JobEventType::Completed {
        job.result = event.result.clone();
        job.last_error = None;
        job.error_detail = None;
    }

    Some(job)
}

fn concurrency_metadata_from_value(value: &Value) -> Option<JobConcurrencyMetadata> {
    let key = value.get("key")?.as_str()?.to_string();
    let key_hash = value.get("keyHash")?.as_str()?.to_string();
    Some(JobConcurrencyMetadata {
        key,
        key_hash,
        instance_id: optional_string(value, "instanceId"),
        heartbeat_at: optional_string(value, "heartbeatAt"),
        lease_expires_at: optional_string(value, "leaseExpiresAt"),
        stale_takeover_count: value.get("staleTakeoverCount").and_then(Value::as_u64),
    })
}

fn queue_policy_metadata_from_value(value: &Value) -> Option<JobQueuePolicyMetadata> {
    let outcome = value.get("outcome")?.as_str()?.to_string();
    Some(JobQueuePolicyMetadata {
        outcome,
        reason: optional_string(value, "reason"),
        existing_job_id: optional_string(value, "existingJobId"),
        replaced_job_id: optional_string(value, "replacedJobId"),
    })
}

fn optional_string(value: &Value, field: &str) -> Option<String> {
    value.get(field)?.as_str().map(str::to_string)
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use trellis_rs::jobs::events::{
        cancelled_event_with_admin_reason, created_event, failed_event,
        started_event_with_concurrency,
    };
    use trellis_rs::jobs::types::{
        JobConcurrency, JobContext, JobLineage, JobState, JobTrigger, JobTriggerKind,
    };

    use super::*;

    #[test]
    fn projector_consumer_name_includes_projection_identity() {
        assert_eq!(
            projector_consumer_name("0123456789abcdef"),
            "jobs-projector-0123456789abcdef"
        );
    }

    #[test]
    fn project_job_event_upserts_sql_projection() {
        let store = SqliteJobsStore::open_in_memory().expect("store should open");
        let event = created_event(
            "documents",
            "document-process",
            "job-1",
            &context(),
            json!({ "documentId": "doc-1" }),
            3,
            "2026-03-28T12:00:00.000Z",
            None,
        );

        let projected = project_job_event(&store, &event)
            .expect("projection should succeed")
            .expect("event should reduce");

        assert_eq!(projected.state, JobState::Pending);
        let stored = store
            .get_job("documents", "document-process", "job-1")
            .expect("get should succeed")
            .expect("job should be stored");
        assert_eq!(stored.id, "job-1");
        assert_eq!(stored.payload, json!({ "documentId": "doc-1" }));
    }

    #[test]
    fn project_terminal_event_without_created_keeps_evidence_visible() {
        let store = SqliteJobsStore::open_in_memory().expect("store should open");
        let event = failed_event(
            "documents",
            "document-process",
            "old-job",
            &context(),
            JobState::Active,
            1,
            "2026-03-28T12:05:00.000Z",
            "boom",
        );

        let projected = project_job_event(&store, &event)
            .expect("projection should succeed")
            .expect("terminal evidence should reduce");

        assert_eq!(projected.id, "old-job");
        assert_eq!(projected.state, JobState::Failed);
        assert_eq!(projected.payload, Value::Null);
        assert_eq!(projected.last_error.as_deref(), Some("boom"));
        let stored = store
            .get_job_by_global_id("old-job")
            .expect("lookup should succeed")
            .expect("synthetic terminal job should be stored");
        assert_eq!(stored.state, JobState::Failed);
    }

    #[test]
    fn project_job_event_projects_keyed_concurrency_metadata() {
        let store = SqliteJobsStore::open_in_memory().expect("store should open");
        let event = created_event(
            "documents",
            "document-process",
            "job-1",
            &context(),
            json!({ "documentId": "doc-1" }),
            3,
            "2026-03-28T12:00:00.000Z",
            None,
        );
        let mut raw_event = serde_json::to_value(&event).expect("event should encode");
        raw_event["concurrency"] = json!({
            "key": "tenant-1:document:doc-1",
            "keyHash": "hash-1",
            "instanceId": "worker-1",
            "heartbeatAt": "2026-03-28T12:00:30.000Z",
            "leaseExpiresAt": "2026-03-28T12:02:30.000Z",
            "staleTakeoverCount": 2
        });

        project_job_event_with_payload(&store, &event, &raw_event)
            .expect("projection should succeed");

        let metadata = store
            .get_job_metadata("documents", "document-process", "job-1")
            .expect("metadata get should succeed")
            .expect("metadata should exist");
        let concurrency = metadata.concurrency.expect("concurrency should project");
        assert_eq!(concurrency.key, "tenant-1:document:doc-1");
        assert_eq!(concurrency.key_hash, "hash-1");
        assert_eq!(concurrency.instance_id.as_deref(), Some("worker-1"));
        assert_eq!(concurrency.stale_takeover_count, Some(2));
    }

    #[test]
    fn project_job_event_projects_trigger_and_lineage() {
        let store = SqliteJobsStore::open_in_memory().expect("store should open");
        let mut event = created_event(
            "documents",
            "document-process",
            "child-job",
            &context(),
            json!({ "documentId": "doc-1" }),
            3,
            "2026-03-28T12:00:00.000Z",
            None,
        );
        event.trigger = Some(JobTrigger {
            kind: JobTriggerKind::ParentJob,
            id: None,
            subject: None,
            operation_id: None,
            parent_job_id: Some("parent-job".to_string()),
            trace_id: Some("0123456789abcdef0123456789abcdef".to_string()),
            request_id: Some("request-job-1".to_string()),
        });
        event.lineage = Some(JobLineage {
            parent_job_id: Some("parent-job".to_string()),
            root_job_id: Some("root-job".to_string()),
            operation_id: None,
            related_keys: None,
        });

        project_job_event(&store, &event).expect("projection should succeed");

        let projected = store
            .get_job_lineage_by_global_id("child-job")
            .expect("lineage should read");
        assert_eq!(
            projected.trigger.map(|trigger| trigger.kind),
            Some(JobTriggerKind::ParentJob)
        );
        assert_eq!(
            projected.lineage.and_then(|lineage| lineage.parent_job_id),
            Some("parent-job".to_string())
        );
    }

    #[test]
    fn project_started_event_projects_active_key_instance_id() {
        let store = SqliteJobsStore::open_in_memory().expect("store should open");
        let created = created_event(
            "documents",
            "document-process",
            "job-1",
            &context(),
            json!({ "documentId": "doc-1" }),
            3,
            "2026-03-28T12:00:00.000Z",
            None,
        );
        project_job_event(&store, &created).expect("created projection should succeed");
        let started = started_event_with_concurrency(
            "documents",
            "document-process",
            "job-1",
            &context(),
            JobState::Pending,
            1,
            "2026-03-28T12:01:00.000Z",
            JobConcurrency {
                key: "tenant-1:document:doc-1".to_string(),
                key_hash: "hash-1".to_string(),
                instance_id: Some("worker-1".to_string()),
                slot_token: Some("slot-1".to_string()),
                heartbeat_at: Some("2026-03-28T12:01:00.000Z".to_string()),
                lease_expires_at: Some("2026-03-28T12:03:00.000Z".to_string()),
                stale_takeover_count: Some(0),
            },
        );

        project_job_event(&store, &started).expect("started projection should succeed");

        let key = store
            .get_projected_key("documents", "document-process", "tenant-1:document:doc-1")
            .expect("key query should succeed")
            .expect("key should exist");
        assert_eq!(key.active.len(), 1);
        assert_eq!(key.active[0].job_id, "job-1");
        assert_eq!(key.active[0].instance_id.as_deref(), Some("worker-1"));
    }

    #[test]
    fn project_job_event_records_timeline_in_sequence_order() {
        let store = SqliteJobsStore::open_in_memory().expect("store should open");
        let created = created_event(
            "documents",
            "document-process",
            "job-1",
            &context(),
            json!({ "documentId": "doc-1" }),
            3,
            "2026-03-28T12:00:00.000Z",
            None,
        );
        let started = started_event_with_concurrency(
            "documents",
            "document-process",
            "job-1",
            &context(),
            JobState::Pending,
            1,
            "2026-03-28T12:00:00.000Z",
            JobConcurrency {
                key: "tenant-1:document:doc-1".to_string(),
                key_hash: "hash-1".to_string(),
                instance_id: Some("worker-1".to_string()),
                slot_token: Some("slot-1".to_string()),
                heartbeat_at: None,
                lease_expires_at: None,
                stale_takeover_count: None,
            },
        );

        project_job_event(&store, &created).expect("created projection should succeed");
        project_job_event(&store, &started).expect("started projection should succeed");

        let timeline = store
            .list_timeline_events("job-1", 10)
            .expect("timeline should list");
        assert_eq!(timeline.len(), 2);
        assert_eq!(timeline[0].sequence, 1);
        assert_eq!(timeline[0].event_type, "created");
        assert_eq!(timeline[1].sequence, 2);
        assert_eq!(timeline[1].event_type, "started");
        assert_eq!(timeline[1].worker_instance_id.as_deref(), Some("worker-1"));
    }

    #[test]
    fn project_admin_reason_into_timeline_message() {
        let store = SqliteJobsStore::open_in_memory().expect("store should open");
        let created = created_event(
            "documents",
            "document-process",
            "job-1",
            &context(),
            json!({ "documentId": "doc-1" }),
            3,
            "2026-03-28T12:00:00.000Z",
            None,
        );
        project_job_event(&store, &created).expect("created projection should succeed");
        let cancelled = cancelled_event_with_admin_reason(
            "documents",
            "document-process",
            "job-1",
            &context(),
            JobState::Pending,
            0,
            "2026-03-28T12:01:00.000Z",
            Some("operator requested maintenance"),
        );

        project_job_event(&store, &cancelled).expect("cancelled projection should succeed");

        let timeline = store
            .list_timeline_events("job-1", 10)
            .expect("timeline should list");
        let admin_event = timeline
            .iter()
            .find(|event| event.event_type == "cancelled")
            .expect("admin action should appear in timeline");
        assert_eq!(admin_event.state, "cancelled");
        assert_eq!(
            admin_event.message.as_deref(),
            Some("operator requested maintenance")
        );
    }

    #[test]
    fn project_job_event_projects_queue_policy_reason_metadata() {
        let store = SqliteJobsStore::open_in_memory().expect("store should open");
        let event = created_event(
            "documents",
            "document-process",
            "job-2",
            &context(),
            json!({ "documentId": "doc-2" }),
            3,
            "2026-03-28T12:00:00.000Z",
            None,
        );
        let mut raw_event = serde_json::to_value(&event).expect("event should encode");
        raw_event["queuePolicy"] = json!({
            "outcome": "coalesced",
            "reason": "queue-full",
            "existingJobId": "job-1"
        });

        project_job_event_with_payload(&store, &event, &raw_event)
            .expect("projection should succeed");

        let metadata = store
            .get_job_metadata("documents", "document-process", "job-2")
            .expect("metadata get should succeed")
            .expect("metadata should exist");
        let queue_policy = metadata.queue_policy.expect("queue policy should project");
        assert_eq!(queue_policy.outcome, "coalesced");
        assert_eq!(queue_policy.reason.as_deref(), Some("queue-full"));
        assert_eq!(queue_policy.existing_job_id.as_deref(), Some("job-1"));
    }

    #[test]
    fn project_job_event_projects_failed_error_detail_and_aggregate() {
        let store = SqliteJobsStore::open_in_memory().expect("store should open");
        let created = created_event(
            "documents",
            "document-process",
            "job-1",
            &context(),
            json!({ "documentId": "doc-1" }),
            3,
            "2026-03-28T12:00:00.000Z",
            None,
        );
        project_job_event(&store, &created).expect("created projection should succeed");
        let started = started_event_with_concurrency(
            "documents",
            "document-process",
            "job-1",
            &context(),
            JobState::Pending,
            1,
            "2026-03-28T12:01:00.000Z",
            JobConcurrency {
                key: "tenant-1:document:doc-1".to_string(),
                key_hash: "hash-1".to_string(),
                instance_id: Some("worker-1".to_string()),
                slot_token: Some("slot-1".to_string()),
                heartbeat_at: None,
                lease_expires_at: None,
                stale_takeover_count: None,
            },
        );
        project_job_event(&store, &started).expect("started projection should succeed");
        let failed = failed_event(
            "documents",
            "document-process",
            "job-1",
            &context(),
            JobState::Active,
            1,
            "2026-03-28T12:02:00.000Z",
            "boom\njob id 123",
        );

        let projected = project_job_event(&store, &failed)
            .expect("failed projection should succeed")
            .expect("failed event should reduce");

        let detail = projected.error_detail.expect("error detail should project");
        assert_eq!(detail.message, "boom\njob id 123");
        let aggregate = store
            .get_error_projection(&detail.fingerprint)
            .expect("error projection should read")
            .expect("error projection should exist");
        assert_eq!(aggregate.message, "boom\njob id 123");
        assert_eq!(aggregate.occurrence_count, 1);
    }

    fn context() -> JobContext {
        JobContext {
            request_id: "request-job-1".to_string(),
            trace_id: "0123456789abcdef0123456789abcdef".to_string(),
            traceparent: "00-0123456789abcdef0123456789abcdef-0123456789abcdef-01".to_string(),
            tracestate: None,
        }
    }
}
