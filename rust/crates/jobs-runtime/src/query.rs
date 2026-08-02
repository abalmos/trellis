//! SQLite-backed query and mutation helpers for the Jobs admin service.

use std::collections::BTreeMap;
use std::time::{Duration, Instant};

use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;
use trellis_rs::jobs::types::{
    Job, JobAdminAction, JobErrorDetail, JobEvent, JobState, JobTrigger, JobTriggerKind,
    JobWaitEdge,
};
use trellis_rs::jobs::JobsRuntime;
use trellis_rs::jobs::{
    cancelled_event_with_admin_reason, dismissed_event, is_terminal, job_event_subject,
    reduce_job_event, retried_event_with_admin_reason,
};

use trellis_rs::sdk::jobs::types::{
    JobsCancelRequest, JobsCancelResponse, JobsDismissDLQRequest, JobsDismissDLQResponse,
    JobsGetKeyRequest, JobsGetKeyResponse, JobsGetKeyResponseActiveItem,
    JobsGetKeyResponseQueuedItem, JobsInspectRequest, JobsInspectResponse,
    JobsInspectResponseErrorsItem, JobsInspectResponseRelatedItem,
    JobsInspectResponseRelatedItemContext, JobsInspectResponseRelatedItemProgress,
    JobsInspectResponseTimelineItem, JobsInspectResponseTimelineItemErrorDetail,
    JobsInspectResponseTimelineItemWaitEdge, JobsListDLQRequest, JobsListDLQResponse,
    JobsListServicesRequest, JobsListServicesResponse, JobsListServicesResponseEntriesItem,
    JobsListServicesResponseEntriesItemWorkersItem, JobsMetricsRequest, JobsMetricsResponse,
    JobsMetricsResponseBucketsItem, JobsMetricsResponseBucketsItemGroupsItem,
    JobsMetricsResponseBucketsItemGroupsItemQueueWait,
    JobsMetricsResponseBucketsItemGroupsItemRuntime, JobsMetricsResponseSummaryItem,
    JobsMetricsResponseSummaryItemQueueWait, JobsMetricsResponseSummaryItemRuntime,
    JobsQueryRequest, JobsQueryResponse, JobsQueryResponseEntriesItem,
    JobsQueryResponseEntriesItemContext, JobsQueryResponseEntriesItemProgress,
    JobsQueryResponseGroupsItem, JobsQueryResponseStats, JobsReplayDLQRequest,
    JobsReplayDLQResponse, JobsRetryRequest, JobsRetryResponse,
};

mod resources;
mod state;
mod wire;
use crate::storage::{
    JobProjectionMetadata, JobTimelineEvent, JobsMetricsBucket, JobsMetricsBucketGroup,
    JobsMetricsFilter, JobsMetricsLatency, JobsMetricsSummaryGroup, JobsWorkbenchEntry,
    JobsWorkbenchFilter, JobsWorkbenchGroup, JobsWorkbenchGroupBy, JobsWorkbenchSort,
    JobsWorkbenchSortField, JobsWorkbenchStats, ListJobsFilter, SqliteJobsStore,
    SqliteJobsStoreError,
};
use crate::worker_presence::WORKER_PRESENCE_FRESH_FOR;

pub use resources::jobs_admin_resources;
pub use resources::JobsAdminResources;
use state::{now_timestamp_string, parse_state_filter};
use wire::{
    job_to_cancel_item, job_to_dismiss_item, job_to_dlq_item, job_to_inspect_item,
    job_to_replay_item, job_to_retry_item,
};

/// Errors returned while resolving bindings, reading projection state, or publishing admin events.
#[derive(Debug, thiserror::Error)]
pub enum JobsQueryError {
    #[error("job state conflict for key '{key}': expected '{expected}', found '{actual}'")]
    JobStateConflict {
        key: String,
        expected: String,
        actual: String,
    },
    #[error("projected job not found for key '{key}'")]
    JobNotFound { key: String },
    #[error("failed to encode job event for key '{key}': {details}")]
    EncodeEvent { key: String, details: String },
    #[error("failed to publish job event on subject '{subject}': {details}")]
    PublishEvent { subject: String, details: String },
    #[error("failed to read Jobs SQLite projection: {details}")]
    ProjectionStore { details: String },
    #[error("failed to convert {model} between internal and generated wire shapes: {details}")]
    ConvertWireModel {
        model: &'static str,
        details: String,
    },
    #[error("invalid {field}: {details}")]
    Validation {
        field: &'static str,
        details: String,
    },
}

#[derive(Clone)]
pub struct JobsQuery {
    jobs_runtime: JobsRuntime,
    store: SqliteJobsStore,
}

impl JobsQuery {
    /// Create a SQLite-backed Jobs query adapter with an already-open store.
    pub fn with_store(jobs_runtime: JobsRuntime, store: SqliteJobsStore) -> Self {
        Self {
            jobs_runtime,
            store,
        }
    }

    async fn with_projection<T, F>(&self, f: F) -> Result<T, JobsQueryError>
    where
        T: Send + 'static,
        F: FnOnce(SqliteJobsStore) -> Result<T, JobsQueryError> + Send + 'static,
    {
        let store = self.store.clone();
        tokio::task::spawn_blocking(move || f(store))
            .await
            .map_err(|error| JobsQueryError::ProjectionStore {
                details: format!("projection task failed: {error}"),
            })?
    }

    /// List registered service instances grouped by service name.
    pub async fn list_services(
        &self,
        request: &JobsListServicesRequest,
    ) -> Result<JobsListServicesResponse, JobsQueryError> {
        let started = Instant::now();
        let (offset, limit) = parse_page_request(request.offset, request.limit)?;
        tracing::debug!(offset, limit, "jobs rpc list_services started");
        let now = OffsetDateTime::now_utc();
        let workers = self
            .with_projection(move |store| {
                Ok(store.list_fresh_workers(now, WORKER_PRESENCE_FRESH_FOR)?)
            })
            .await?;

        let mut grouped =
            BTreeMap::<String, Vec<JobsListServicesResponseEntriesItemWorkersItem>>::new();
        for worker in workers {
            let service_name = worker.service.clone();
            grouped
                .entry(service_name)
                .or_default()
                .push(wire::worker_presence_to_wire(&worker));
        }

        let mut services = Vec::new();
        for (name, mut workers) in grouped {
            workers.sort_by(|left, right| {
                left.job_type
                    .cmp(&right.job_type)
                    .then_with(|| left.instance_id.cmp(&right.instance_id))
            });
            services.push(JobsListServicesResponseEntriesItem {
                healthy: !workers.is_empty(),
                name,
                workers,
            });
        }
        let count = u64::try_from(services.len()).unwrap_or(u64::MAX);
        let services = services
            .into_iter()
            .skip(usize::try_from(offset).unwrap_or(usize::MAX))
            .take(usize::try_from(limit).unwrap_or(usize::MAX))
            .collect();
        let next_offset = offset.checked_add(limit).filter(|next| *next < count);
        tracing::debug!(
            count,
            offset,
            limit,
            elapsed_ms = started.elapsed().as_millis(),
            "jobs rpc list_services completed"
        );

        Ok(JobsListServicesResponse {
            count: to_wire_integer(count),
            entries: services,
            limit: to_wire_integer(limit),
            next_offset: next_offset.map(to_wire_integer),
            offset: to_wire_integer(offset),
        })
    }

    /// Query projected jobs using the generated `Jobs.Query` workbench wire shape.
    pub async fn query_jobs(
        &self,
        request: &JobsQueryRequest,
    ) -> Result<JobsQueryResponse, JobsQueryError> {
        let started = Instant::now();
        let (offset, limit) = parse_page_request(request.offset, request.limit)?;
        let since = parse_window_filter(request.window.as_ref().map(AsRef::as_ref))?;
        tracing::debug!(
            service = ?request.service,
            job_type = ?request.r#type,
            state = ?request.state,
            window = ?request.window,
            group_by = ?request.group_by,
            search = request.search.is_some(),
            offset,
            limit,
            "jobs rpc query started"
        );
        let filter = JobsWorkbenchFilter {
            service: request.service.clone(),
            job_type: request.r#type.clone(),
            states: parse_state_filter(request.state.as_ref())?,
            since,
            search: request.search.clone(),
            queue_key: request.queue_key.clone(),
            runtime_band: request
                .runtime_band
                .as_ref()
                .map(|value| value.as_str().to_string()),
            trigger: request.trigger.clone(),
            sort: parse_workbench_sort(request.sort.as_ref())?,
            group_by: parse_group_by(request.group_by.as_ref().map(AsRef::as_ref))?,
            offset,
            limit,
        };
        let (page, groups) = self
            .with_projection(move |store| {
                Ok((store.query_jobs(&filter)?, store.query_job_groups(&filter)?))
            })
            .await?;
        tracing::debug!(
            count = page.count,
            entries = page.entries.len(),
            groups = groups.len(),
            elapsed_ms = started.elapsed().as_millis(),
            "jobs rpc query completed"
        );

        Ok(JobsQueryResponse {
            count: to_wire_integer(page.count),
            entries: page
                .entries
                .iter()
                .map(workbench_entry_to_wire)
                .collect::<Result<Vec<_>, _>>()?,
            groups: groups
                .iter()
                .map(workbench_group_to_wire)
                .collect::<Result<Vec<_>, _>>()?,
            limit: to_wire_integer(page.limit),
            next_offset: page.next_offset.map(to_wire_integer),
            offset: to_wire_integer(page.offset),
            stats: workbench_stats_to_wire(&page.stats)?,
        })
    }

    /// Query grouped operational metrics for Jobs dashboards.
    pub async fn metrics(
        &self,
        request: &JobsMetricsRequest,
    ) -> Result<JobsMetricsResponse, JobsQueryError> {
        let started = Instant::now();
        let until = OffsetDateTime::now_utc();
        let window = parse_metrics_window(request.window.as_str())?;
        tracing::debug!(
            service = ?request.service,
            job_type = ?request.r#type,
            state = ?request.state,
            window = %request.window,
            step = %request.step,
            group_by = %request.group_by,
            "jobs rpc metrics started"
        );
        let filter = JobsMetricsFilter {
            service: request.service.clone(),
            job_type: request.r#type.clone(),
            states: parse_state_filter(request.state.as_ref())?,
            since: until - window,
            until,
            step_nanos: parse_metrics_step(request.step.as_str())?
                .whole_nanoseconds()
                .try_into()
                .map_err(
                    |error: std::num::TryFromIntError| JobsQueryError::Validation {
                        field: "step",
                        details: error.to_string(),
                    },
                )?,
            queue_key: request.queue_key.clone(),
            trigger: request.trigger.clone(),
            group_by: parse_metrics_group_by(request.group_by.as_str())?,
        };
        let page = self
            .with_projection(move |store| Ok(store.query_metrics(&filter)?))
            .await?;
        tracing::debug!(
            buckets = page.buckets.len(),
            summary = page.summary.len(),
            elapsed_ms = started.elapsed().as_millis(),
            "jobs rpc metrics completed"
        );
        Ok(JobsMetricsResponse {
            buckets: page
                .buckets
                .iter()
                .map(metrics_bucket_to_wire)
                .collect::<Result<Vec<_>, _>>()?,
            generated_at: until
                .format(&Rfc3339)
                .map_err(|error| JobsQueryError::Validation {
                    field: "generatedAt",
                    details: error.to_string(),
                })?,
            group_by: request.group_by.as_str().to_string(),
            step: request.step.as_str().to_string(),
            summary: page
                .summary
                .iter()
                .map(metrics_summary_group_to_wire)
                .collect::<Result<Vec<_>, _>>()?,
            window: request.window.as_str().to_string(),
        })
    }

    /// Fetch one projected job and its timeline by globally addressable admin job id.
    pub async fn inspect(
        &self,
        request: &JobsInspectRequest,
    ) -> Result<JobsInspectResponse, JobsQueryError> {
        let started = Instant::now();
        tracing::debug!(job_id = %request.id, "jobs rpc inspect started");
        let request_id = request.id.clone();
        let job = self
            .with_projection(move |store| {
                store
                    .get_job_by_global_id(&request_id)?
                    .ok_or(JobsQueryError::JobNotFound { key: request_id })
            })
            .await?;

        let metadata = self.job_metadata(&job).await?;
        let errors = self.error_details(&job).await?;
        let job_id = job.id.clone();
        let related_job = job.clone();
        let (lineage, related, timeline, waiting_on) = self
            .with_projection(move |store| {
                Ok((
                    store.get_job_lineage_by_global_id(&job_id)?,
                    store.list_related_jobs(&related_job, 10)?,
                    store.list_timeline_events(&job_id, 200)?,
                    store
                        .list_current_waits(&job_id)?
                        .into_iter()
                        .map(|wait| wait.wait_edge)
                        .collect::<Vec<_>>(),
                ))
            })
            .await?;
        let mut response_job = job.clone();
        response_job.waiting_on = (!waiting_on.is_empty()).then_some(waiting_on);
        tracing::debug!(
            service = %job.service,
            job_type = %job.job_type,
            job_id = %job.id,
            timeline = timeline.len(),
            related = related.len(),
            errors = errors.len(),
            elapsed_ms = started.elapsed().as_millis(),
            "jobs rpc inspect completed"
        );
        Ok(JobsInspectResponse {
            attempts: Vec::new(),
            errors,
            job: job_to_inspect_item(&response_job, &metadata)?,
            lineage: map_optional_wire(&lineage.lineage, "job inspect lineage")?,
            related: related
                .iter()
                .map(related_entry_to_wire)
                .collect::<Result<Vec<_>, _>>()?,
            timeline: timeline
                .iter()
                .map(timeline_event_to_wire)
                .collect::<Result<Vec<_>, _>>()?,
            trigger: map_optional_wire(&lineage.trigger, "job inspect trigger")?,
        })
    }

    /// Fetch projection-backed keyed-concurrency state by service, job type, and display key.
    ///
    /// This path currently reads SQLite projection state only. The Jobs admin binding does not yet
    /// expose a `JOBS_KEYS` KV handle here, so very recent runtime coordinator updates may be newer
    /// than this response until lifecycle events are projected.
    pub async fn get_key(
        &self,
        request: &JobsGetKeyRequest,
    ) -> Result<JobsGetKeyResponse, JobsQueryError> {
        let started = Instant::now();
        tracing::debug!(
            service = ?request.service,
            job_type = ?request.r#type,
            key = %request.key,
            "jobs rpc get_key started"
        );
        let service = request.service.clone();
        let job_type = request.r#type.clone();
        let request_key = request.key.clone();
        let key = self
            .with_projection(move |store| {
                store
                    .get_projected_key(&service, &job_type, &request_key)?
                    .ok_or_else(|| JobsQueryError::JobNotFound {
                        key: format!("{service}/{job_type}/{request_key}"),
                    })
            })
            .await?;
        let now = OffsetDateTime::now_utc();
        tracing::debug!(
            service = %key.service,
            job_type = %key.job_type,
            key = %key.key,
            active = key.active.len(),
            queued = key.queued.len(),
            elapsed_ms = started.elapsed().as_millis(),
            "jobs rpc get_key completed"
        );

        Ok(JobsGetKeyResponse {
            active: key
                .active
                .iter()
                .filter_map(|active| {
                    let started_at = active.started_at.clone()?;
                    let heartbeat_at = active.heartbeat_at.clone()?;
                    let lease_expires_at = active.lease_expires_at.clone()?;
                    Some(JobsGetKeyResponseActiveItem {
                        heartbeat_age_ms: heartbeat_age_ms(&heartbeat_at, now),
                        heartbeat_at,
                        instance_id: active.instance_id.clone().unwrap_or_default(),
                        job_id: active.job_id.clone(),
                        lease_expires_at,
                        started_at,
                    })
                })
                .collect(),
            key: key.key,
            key_hash: key.key_hash,
            latest_policy_reason: key.latest_policy_reason,
            queued_depth: to_wire_integer(u64::try_from(key.queued.len()).unwrap_or(u64::MAX)),
            queued: key
                .queued
                .iter()
                .map(|queued| JobsGetKeyResponseQueuedItem {
                    created_at: queued.created_at.clone(),
                    job_id: queued.job_id.clone(),
                })
                .collect(),
            service: key.service,
            stale_takeover_count: to_wire_integer(key.stale_takeover_count),
            r#type: key.job_type,
        })
    }

    /// Cancel a projected job by publishing a `cancelled` event.
    pub async fn cancel_job(
        &self,
        request: &JobsCancelRequest,
    ) -> Result<JobsCancelResponse, JobsQueryError> {
        let started = Instant::now();
        tracing::debug!(job_id = %request.id, "jobs rpc cancel started");
        let request_id = request.id.clone();
        let existing = self
            .with_projection(move |store| {
                store
                    .get_job_by_global_id(&request_id)?
                    .ok_or(JobsQueryError::JobNotFound { key: request_id })
            })
            .await?;
        let job = if is_terminal(existing.state) {
            existing
        } else {
            self.transition_job(&request.id, "pending|retry|active", |job, now| {
                if matches!(
                    job.state,
                    JobState::Pending | JobState::Retry | JobState::Active
                ) {
                    Some(cancelled_event_with_admin_reason(
                        &job.service,
                        &job.job_type,
                        &job.id,
                        &job.context,
                        job.state,
                        job.tries,
                        now,
                        request.reason.as_deref(),
                    ))
                } else {
                    None
                }
            })
            .await?
        };
        tracing::debug!(
            service = %job.service,
            job_type = %job.job_type,
            job_id = %job.id,
            state = ?job.state,
            elapsed_ms = started.elapsed().as_millis(),
            "jobs rpc cancel completed"
        );

        Ok(JobsCancelResponse {
            job: job_to_cancel_item(&job, &self.job_metadata(&job).await?)?,
        })
    }

    /// Retry a failed job by publishing a `retried` event.
    pub async fn retry_job(
        &self,
        request: &JobsRetryRequest,
    ) -> Result<JobsRetryResponse, JobsQueryError> {
        let started = Instant::now();
        tracing::debug!(job_id = %request.id, "jobs rpc retry started");
        let job = self
            .transition_job(&request.id, "failed", |job, now| match job.state {
                JobState::Failed => Some(retried_event_with_admin_reason(
                    &job.service,
                    &job.job_type,
                    &job.id,
                    &job.context,
                    job.state,
                    now,
                    Some(job.payload.clone()),
                    Some(job.max_tries),
                    job.deadline.as_deref(),
                    request.reason.as_deref(),
                )),
                _ => None,
            })
            .await?;
        tracing::debug!(
            service = %job.service,
            job_type = %job.job_type,
            job_id = %job.id,
            state = ?job.state,
            elapsed_ms = started.elapsed().as_millis(),
            "jobs rpc retry completed"
        );

        Ok(JobsRetryResponse {
            job: job_to_retry_item(&job, &self.job_metadata(&job).await?)?,
        })
    }

    /// List only jobs currently in the DLQ (`dead`) state.
    pub async fn list_dlq(
        &self,
        request: &JobsListDLQRequest,
    ) -> Result<JobsListDLQResponse, JobsQueryError> {
        let started = Instant::now();
        let (offset, limit) = parse_page_request(request.offset, request.limit)?;
        let since = parse_since_filter(request.since.as_deref())?;
        tracing::debug!(
            service = ?request.service,
            job_type = ?request.r#type,
            since = ?request.since,
            offset,
            limit,
            "jobs rpc list_dlq started"
        );
        let service = request.service.clone();
        let job_type = request.r#type.clone();
        let page = self
            .with_projection(move |store| {
                Ok(store.list_jobs(&ListJobsFilter {
                    service,
                    job_type,
                    states: Some(vec![JobState::Dead]),
                    since,
                    offset: Some(offset),
                    limit,
                })?)
            })
            .await?;
        let mut entries = Vec::new();
        for job in &page.jobs {
            let metadata = self.job_metadata(job).await?;
            entries.push(job_to_dlq_item(job, &metadata)?);
        }
        tracing::debug!(
            count = page.count,
            entries = entries.len(),
            offset = page.offset,
            limit = page.limit,
            elapsed_ms = started.elapsed().as_millis(),
            "jobs rpc list_dlq completed"
        );
        Ok(JobsListDLQResponse {
            count: to_wire_integer(page.count),
            entries,
            limit: to_wire_integer(page.limit),
            next_offset: page.next_offset.map(to_wire_integer),
            offset: to_wire_integer(page.offset),
        })
    }

    /// Replay a dead-lettered job by publishing a `retried` event.
    pub async fn replay_dlq(
        &self,
        request: &JobsReplayDLQRequest,
    ) -> Result<JobsReplayDLQResponse, JobsQueryError> {
        let started = Instant::now();
        tracing::debug!(job_id = %request.id, "jobs rpc replay_dlq started");
        let job = self
            .transition_job(&request.id, "dead", |job, now| match job.state {
                JobState::Dead => {
                    let mut event = retried_event_with_admin_reason(
                        &job.service,
                        &job.job_type,
                        &job.id,
                        &job.context,
                        job.state,
                        now,
                        Some(job.payload.clone()),
                        Some(job.max_tries),
                        job.deadline.as_deref(),
                        request.reason.as_deref(),
                    );
                    event.trigger = Some(JobTrigger {
                        kind: JobTriggerKind::ManualReplay,
                        id: None,
                        subject: None,
                        operation_id: job
                            .lineage
                            .as_ref()
                            .and_then(|lineage| lineage.operation_id.clone()),
                        parent_job_id: job
                            .lineage
                            .as_ref()
                            .and_then(|lineage| lineage.parent_job_id.clone()),
                        trace_id: Some(job.context.trace_id.clone()),
                        request_id: Some(job.context.request_id.clone()),
                    });
                    event.lineage = job.lineage.clone();
                    Some(event)
                }
                _ => None,
            })
            .await?;
        tracing::debug!(
            service = %job.service,
            job_type = %job.job_type,
            job_id = %job.id,
            state = ?job.state,
            elapsed_ms = started.elapsed().as_millis(),
            "jobs rpc replay_dlq completed"
        );

        Ok(JobsReplayDLQResponse {
            job: job_to_replay_item(&job, &self.job_metadata(&job).await?)?,
        })
    }

    /// Dismiss a dead-lettered job by publishing a `dismissed` event.
    pub async fn dismiss_dlq(
        &self,
        request: &JobsDismissDLQRequest,
    ) -> Result<JobsDismissDLQResponse, JobsQueryError> {
        let started = Instant::now();
        tracing::debug!(job_id = %request.id, "jobs rpc dismiss_dlq started");
        let job = self
            .transition_job(&request.id, "dead", |job, now| match job.state {
                JobState::Dead => {
                    let mut event = dismissed_event(
                        &job.service,
                        &job.job_type,
                        &job.id,
                        &job.context,
                        JobState::Dead,
                        job.tries,
                        now,
                        request.reason.as_deref().or(job.last_error.as_deref()),
                    );
                    event.admin_action = request.reason.as_ref().map(|reason| JobAdminAction {
                        reason: Some(reason.clone()),
                    });
                    Some(event)
                }
                _ => None,
            })
            .await?;
        tracing::debug!(
            service = %job.service,
            job_type = %job.job_type,
            job_id = %job.id,
            state = ?job.state,
            elapsed_ms = started.elapsed().as_millis(),
            "jobs rpc dismiss_dlq completed"
        );

        Ok(JobsDismissDLQResponse {
            job: job_to_dismiss_item(&job, &self.job_metadata(&job).await?)?,
        })
    }

    async fn job_metadata(&self, job: &Job) -> Result<JobProjectionMetadata, JobsQueryError> {
        let service = job.service.clone();
        let job_type = job.job_type.clone();
        let id = job.id.clone();
        self.with_projection(move |store| {
            Ok(store
                .get_job_metadata(&service, &job_type, &id)?
                .unwrap_or_default())
        })
        .await
    }

    async fn error_details(
        &self,
        job: &Job,
    ) -> Result<Vec<JobsInspectResponseErrorsItem>, JobsQueryError> {
        let fallback_detail = job
            .last_error
            .as_deref()
            .map(|message| JobErrorDetail::from_message(&job.service, &job.job_type, message));
        let Some(detail) = job.error_detail.as_ref().or(fallback_detail.as_ref()) else {
            return Ok(Vec::new());
        };
        let fingerprint = detail.fingerprint.clone();
        let projection = self
            .with_projection(move |store| Ok(store.get_error_projection(&fingerprint)?))
            .await?;
        let mut detail = detail.clone();
        if let Some(projection) = projection {
            detail.first_seen = Some(projection.first_seen);
            detail.occurrence_count = Some(projection.occurrence_count);
        }
        Ok(vec![serde_json::from_value(
            serde_json::to_value(detail).map_err(|error| JobsQueryError::ConvertWireModel {
                model: "job error detail",
                details: error.to_string(),
            })?,
        )
        .map_err(|error| JobsQueryError::ConvertWireModel {
            model: "job error detail",
            details: error.to_string(),
        })?])
    }

    async fn transition_job<F>(
        &self,
        id: &str,
        expected_states: &str,
        build_event: F,
    ) -> Result<Job, JobsQueryError>
    where
        F: FnOnce(&Job, &str) -> Option<JobEvent>,
    {
        let id_string = id.to_string();
        let job = self
            .with_projection(move |store| {
                store
                    .get_job_by_global_id(&id_string)?
                    .ok_or(JobsQueryError::JobNotFound { key: id_string })
            })
            .await?;
        let key = projection_key(&job);

        let now = now_timestamp_string();
        let event = build_event(&job, &now).ok_or_else(|| JobsQueryError::JobStateConflict {
            key: key.clone(),
            expected: expected_states.to_string(),
            actual: format!("{:?}", job.state).to_lowercase(),
        })?;
        let subject = self.transition_event_subject(&job, &event).await;
        tracing::debug!(
            service = %job.service,
            job_type = %job.job_type,
            job_id = %job.id,
            from_state = ?job.state,
            event_type = event.event_type.as_token(),
            subject = %subject,
            "publishing jobs admin transition event"
        );
        let payload = serde_json::to_vec(&event).map_err(|error| JobsQueryError::EncodeEvent {
            key: key.clone(),
            details: error.to_string(),
        })?;

        self.jobs_runtime
            .publish_event_payload(subject.clone(), job_event_headers(&event), payload)
            .await
            .map_err(|error| JobsQueryError::PublishEvent {
                subject,
                details: error,
            })?;

        let predicted = reduce_job_event(Some(&job), &event).ok_or_else(|| {
            JobsQueryError::JobStateConflict {
                key: key.clone(),
                expected: expected_states.to_string(),
                actual: format!("{:?}", job.state).to_lowercase(),
            }
        })?;
        let projected = self.await_job_projection(&predicted, &event).await?;
        tracing::debug!(
            service = %projected.service,
            job_type = %projected.job_type,
            job_id = %projected.id,
            state = ?projected.state,
            "jobs admin transition projected"
        );

        Ok(projected)
    }

    async fn transition_event_subject(&self, job: &Job, event: &JobEvent) -> String {
        let fallback = job_event_subject(&job.service, &job.job_type, &job.id, event.event_type);
        let job_id = job.id.clone();
        let event_type = event.event_type.as_token().to_string();
        self.with_projection(move |store| {
            Ok(store
                .list_timeline_events(&job_id, 1)
                .ok()
                .and_then(|events| {
                    events.first().and_then(|timeline_event| {
                        raw_event_subject(&timeline_event.raw_event_json)
                    })
                })
                .and_then(|subject| sibling_event_subject(&subject, &event_type)))
        })
        .await
        .ok()
        .flatten()
        .unwrap_or(fallback)
    }

    async fn await_job_projection(
        &self,
        predicted: &Job,
        event: &JobEvent,
    ) -> Result<Job, JobsQueryError> {
        let started = Instant::now();
        for attempt in 0..20 {
            let service = predicted.service.clone();
            let job_type = predicted.job_type.clone();
            let id = predicted.id.clone();
            if let Some(job) = self
                .with_projection(move |store| Ok(store.get_job(&service, &job_type, &id)?))
                .await?
            {
                if plan_mutation_response(Some(&job), predicted, event, false)
                    == MutationResponsePlan::ReturnProjected
                {
                    tracing::debug!(
                        service = %job.service,
                        job_type = %job.job_type,
                        job_id = %job.id,
                        attempts = attempt + 1,
                        elapsed_ms = started.elapsed().as_millis(),
                        "observed projected jobs admin transition"
                    );
                    return Ok(job);
                }
            }
            tokio::time::sleep(Duration::from_millis(25)).await;
        }

        tracing::debug!(
            service = %predicted.service,
            job_type = %predicted.job_type,
            job_id = %predicted.id,
            event_type = event.event_type.as_token(),
            elapsed_ms = started.elapsed().as_millis(),
            "returning predicted jobs admin transition before projection caught up"
        );
        Ok(predicted.clone())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MutationResponsePlan {
    ReturnProjected,
    ReturnPredicted,
    Wait,
}

fn plan_mutation_response(
    projected: Option<&Job>,
    predicted: &Job,
    event: &JobEvent,
    projection_lagged: bool,
) -> MutationResponsePlan {
    if let Some(projected) = projected {
        if projected.state == predicted.state && projected.updated_at == predicted.updated_at {
            return MutationResponsePlan::ReturnProjected;
        }
        if is_terminal_noop_projection(projected, event) {
            return MutationResponsePlan::ReturnProjected;
        }
    }

    if projection_lagged {
        MutationResponsePlan::ReturnPredicted
    } else {
        MutationResponsePlan::Wait
    }
}

fn is_terminal_noop_projection(projected: &Job, event: &JobEvent) -> bool {
    is_terminal(projected.state)
        && reduce_job_event(Some(projected), event).is_some_and(|next| next == projected.clone())
}

fn job_event_headers(event: &JobEvent) -> trellis_rs::jobs::JobEventHeaders {
    trellis_rs::jobs::JobEventHeaders::from(&event.context)
}

impl From<SqliteJobsStoreError> for JobsQueryError {
    fn from(error: SqliteJobsStoreError) -> Self {
        match error {
            SqliteJobsStoreError::Validation { field, details } => {
                Self::Validation { field, details }
            }
            other => Self::ProjectionStore {
                details: other.to_string(),
            },
        }
    }
}

fn parse_page_request(offset: Option<i64>, limit: i64) -> Result<(u64, u64), JobsQueryError> {
    let offset = match offset {
        Some(offset) => parse_non_negative_integer("offset", offset)?,
        None => 0,
    };
    let limit = parse_positive_integer("limit", limit)?;
    Ok((offset, limit))
}

fn parse_since_filter(value: Option<&str>) -> Result<Option<OffsetDateTime>, JobsQueryError> {
    value
        .map(|since| {
            OffsetDateTime::parse(since, &Rfc3339).map_err(|error| JobsQueryError::Validation {
                field: "since",
                details: error.to_string(),
            })
        })
        .transpose()
}

fn parse_window_filter(value: Option<&str>) -> Result<Option<OffsetDateTime>, JobsQueryError> {
    let Some(value) = value else {
        return Ok(None);
    };
    let duration = match value {
        "1h" => time::Duration::hours(1),
        "24h" => time::Duration::hours(24),
        "7d" => time::Duration::days(7),
        other => {
            return Err(JobsQueryError::Validation {
                field: "window",
                details: format!("unsupported window '{other}'"),
            })
        }
    };
    Ok(Some(OffsetDateTime::now_utc() - duration))
}

fn parse_metrics_window(value: &str) -> Result<time::Duration, JobsQueryError> {
    match value {
        "15m" => Ok(time::Duration::minutes(15)),
        "1h" => Ok(time::Duration::hours(1)),
        "6h" => Ok(time::Duration::hours(6)),
        "24h" => Ok(time::Duration::hours(24)),
        "7d" => Ok(time::Duration::days(7)),
        other => Err(JobsQueryError::Validation {
            field: "window",
            details: format!("unsupported window '{other}'"),
        }),
    }
}

fn parse_metrics_step(value: &str) -> Result<time::Duration, JobsQueryError> {
    match value {
        "1m" => Ok(time::Duration::minutes(1)),
        "5m" => Ok(time::Duration::minutes(5)),
        "15m" => Ok(time::Duration::minutes(15)),
        "1h" => Ok(time::Duration::hours(1)),
        "6h" => Ok(time::Duration::hours(6)),
        "1d" => Ok(time::Duration::days(1)),
        other => Err(JobsQueryError::Validation {
            field: "step",
            details: format!("unsupported step '{other}'"),
        }),
    }
}

fn parse_metrics_group_by(value: &str) -> Result<JobsWorkbenchGroupBy, JobsQueryError> {
    match value {
        "service" => Ok(JobsWorkbenchGroupBy::Service),
        "type" => Ok(JobsWorkbenchGroupBy::Type),
        "state" => Ok(JobsWorkbenchGroupBy::State),
        "queueKey" => Ok(JobsWorkbenchGroupBy::QueueKey),
        "trigger" => Ok(JobsWorkbenchGroupBy::Trigger),
        other => Err(JobsQueryError::Validation {
            field: "groupBy",
            details: format!("unsupported group '{other}'"),
        }),
    }
}

fn parse_workbench_sort(
    sort: Option<&trellis_rs::sdk::jobs::types::JobsQueryRequestSort>,
) -> Result<JobsWorkbenchSort, JobsQueryError> {
    let Some(sort) = sort else {
        return Ok(JobsWorkbenchSort::default());
    };
    let field = match sort.field.as_str() {
        "updatedAt" => JobsWorkbenchSortField::UpdatedAt,
        "queueAge" => JobsWorkbenchSortField::QueueAge,
        "runtime" => JobsWorkbenchSortField::Runtime,
        "retries" => JobsWorkbenchSortField::Retries,
        "depth" => JobsWorkbenchSortField::Depth,
        "failureRate" => JobsWorkbenchSortField::FailureRate,
        other => {
            return Err(JobsQueryError::Validation {
                field: "sort.field",
                details: format!("unsupported sort field '{other}'"),
            })
        }
    };
    let descending = match sort
        .direction
        .as_ref()
        .map(|value| value.as_str())
        .unwrap_or("desc")
    {
        "asc" => false,
        "desc" => true,
        other => {
            return Err(JobsQueryError::Validation {
                field: "sort.direction",
                details: format!("unsupported sort direction '{other}'"),
            })
        }
    };
    Ok(JobsWorkbenchSort { field, descending })
}

fn parse_group_by(value: Option<&str>) -> Result<Option<JobsWorkbenchGroupBy>, JobsQueryError> {
    value
        .map(|value| match value {
            "service" => Ok(JobsWorkbenchGroupBy::Service),
            "type" => Ok(JobsWorkbenchGroupBy::Type),
            "state" => Ok(JobsWorkbenchGroupBy::State),
            "queueKey" => Ok(JobsWorkbenchGroupBy::QueueKey),
            "runtimeBand" => Ok(JobsWorkbenchGroupBy::RuntimeBand),
            "trigger" => Ok(JobsWorkbenchGroupBy::Trigger),
            other => Err(JobsQueryError::Validation {
                field: "groupBy",
                details: format!("unsupported group '{other}'"),
            }),
        })
        .transpose()
}

fn workbench_entry_to_wire(
    entry: &JobsWorkbenchEntry,
) -> Result<JobsQueryResponseEntriesItem, JobsQueryError> {
    let job = &entry.job;
    Ok(JobsQueryResponseEntriesItem {
        completed_at: job.completed_at.clone(),
        context: Some(JobsQueryResponseEntriesItemContext {
            request_id: job.context.request_id.clone(),
            trace_id: job.context.trace_id.clone(),
            traceparent: job.context.traceparent.clone(),
            tracestate: job.context.tracestate.clone(),
        }),
        created_at: job.created_at.clone(),
        error_fingerprint: entry.last_error_fingerprint.clone(),
        id: job.id.clone(),
        last_error: job.last_error.clone(),
        lineage: map_optional_wire(&job.lineage, "job query lineage")?,
        max_tries: to_wire_integer(job.max_tries),
        progress: job
            .progress
            .as_ref()
            .map(|progress| JobsQueryResponseEntriesItemProgress {
                current: progress.current.map(to_wire_integer),
                message: progress.message.clone(),
                step: progress.step.clone(),
                total: progress.total.map(to_wire_integer),
            }),
        queue_age_ms: entry
            .queue_age_anchor_nanos
            .map(|anchor| {
                (OffsetDateTime::now_utc().unix_timestamp_nanos() - i128::from(anchor)) / 1_000_000
            })
            .and_then(|millis| i64::try_from(millis.max(0)).ok()),
        queue_key: entry.queue_key.clone(),
        runtime_band: entry.runtime_band.clone(),
        runtime_ms: entry.runtime_ms,
        service: job.service.clone(),
        started_at: job.started_at.clone(),
        state: serde_json::from_value(serde_json::to_value(job.state).map_err(|error| {
            JobsQueryError::ConvertWireModel {
                model: "job query state",
                details: error.to_string(),
            }
        })?)
        .map_err(|error| JobsQueryError::ConvertWireModel {
            model: "job query state",
            details: error.to_string(),
        })?,
        tries: to_wire_integer(job.tries),
        trigger: map_optional_wire(&job.trigger, "job query trigger")?,
        r#type: job.job_type.clone(),
        updated_at: job.updated_at.clone(),
        waiting_on: map_wait_edges(&entry.waiting_on, "job query waitingOn")?,
    })
}

fn related_entry_to_wire(
    entry: &JobsWorkbenchEntry,
) -> Result<JobsInspectResponseRelatedItem, JobsQueryError> {
    let job = &entry.job;
    Ok(JobsInspectResponseRelatedItem {
        completed_at: job.completed_at.clone(),
        context: Some(JobsInspectResponseRelatedItemContext {
            request_id: job.context.request_id.clone(),
            trace_id: job.context.trace_id.clone(),
            traceparent: job.context.traceparent.clone(),
            tracestate: job.context.tracestate.clone(),
        }),
        created_at: job.created_at.clone(),
        error_fingerprint: entry.last_error_fingerprint.clone(),
        id: job.id.clone(),
        last_error: job.last_error.clone(),
        lineage: map_optional_wire(&job.lineage, "job inspect related lineage")?,
        matched_by: map_optional_wire(&entry.matched_by, "job inspect related matchedBy")?,
        max_tries: to_wire_integer(job.max_tries),
        progress: job
            .progress
            .as_ref()
            .map(|progress| JobsInspectResponseRelatedItemProgress {
                current: progress.current.map(to_wire_integer),
                message: progress.message.clone(),
                step: progress.step.clone(),
                total: progress.total.map(to_wire_integer),
            }),
        queue_age_ms: entry
            .queue_age_anchor_nanos
            .map(|anchor| {
                (OffsetDateTime::now_utc().unix_timestamp_nanos() - i128::from(anchor)) / 1_000_000
            })
            .and_then(|millis| i64::try_from(millis.max(0)).ok()),
        queue_key: entry.queue_key.clone(),
        runtime_band: entry.runtime_band.clone(),
        runtime_ms: entry.runtime_ms,
        service: job.service.clone(),
        started_at: job.started_at.clone(),
        state: serde_json::from_value(serde_json::to_value(job.state).map_err(|error| {
            JobsQueryError::ConvertWireModel {
                model: "job inspect related state",
                details: error.to_string(),
            }
        })?)
        .map_err(|error| JobsQueryError::ConvertWireModel {
            model: "job inspect related state",
            details: error.to_string(),
        })?,
        tries: to_wire_integer(job.tries),
        trigger: map_optional_wire(&job.trigger, "job inspect related trigger")?,
        r#type: job.job_type.clone(),
        updated_at: job.updated_at.clone(),
        waiting_on: map_wait_edges(&entry.waiting_on, "job inspect related waitingOn")?,
    })
}

fn timeline_event_to_wire(
    event: &JobTimelineEvent,
) -> Result<JobsInspectResponseTimelineItem, JobsQueryError> {
    let raw_event: serde_json::Value =
        serde_json::from_str(&event.raw_event_json).map_err(|error| {
            JobsQueryError::ConvertWireModel {
                model: "job timeline raw event",
                details: error.to_string(),
            }
        })?;
    Ok(JobsInspectResponseTimelineItem {
        error: event.error_message.clone(),
        error_detail: timeline_error_detail(event)?,
        logs: decode_optional_json(&event.logs_json, "job timeline logs")?,
        message: event.message.clone(),
        previous_state: map_optional_wire(&event.previous_state, "job timeline previousState")?,
        progress: decode_optional_json(&event.progress_json, "job timeline progress")?,
        projected: event.projected,
        raw_event: Some(raw_event.clone()),
        reason: event.reason.clone(),
        sequence: to_wire_integer(event.sequence),
        state: map_wire(&event.state, "job timeline state")?,
        timestamp: event.timestamp.clone(),
        tries: Some(to_wire_integer(event.tries)),
        r#type: event.event_type.clone(),
        wait_edge: raw_event
            .get("waitEdge")
            .cloned()
            .map(serde_json::from_value::<JobsInspectResponseTimelineItemWaitEdge>)
            .transpose()
            .map_err(|error| JobsQueryError::ConvertWireModel {
                model: "job timeline waitEdge",
                details: error.to_string(),
            })?,
        worker_instance_id: event.worker_instance_id.clone(),
    })
}

fn map_wait_edges<T>(
    value: &[JobWaitEdge],
    model: &'static str,
) -> Result<Option<Vec<T>>, JobsQueryError>
where
    T: serde::de::DeserializeOwned,
{
    if value.is_empty() {
        return Ok(None);
    }
    serde_json::from_value(serde_json::to_value(value).map_err(|error| {
        JobsQueryError::ConvertWireModel {
            model,
            details: error.to_string(),
        }
    })?)
    .map(Some)
    .map_err(|error| JobsQueryError::ConvertWireModel {
        model,
        details: error.to_string(),
    })
}

fn timeline_error_detail(
    event: &JobTimelineEvent,
) -> Result<Option<JobsInspectResponseTimelineItemErrorDetail>, JobsQueryError> {
    let raw_event: serde_json::Value =
        serde_json::from_str(&event.raw_event_json).map_err(|error| {
            JobsQueryError::ConvertWireModel {
                model: "job timeline raw event",
                details: error.to_string(),
            }
        })?;
    if let Some(detail) = raw_event.get("errorDetail") {
        return serde_json::from_value(detail.clone())
            .map(Some)
            .map_err(|error| JobsQueryError::ConvertWireModel {
                model: "job timeline error detail",
                details: error.to_string(),
            });
    }
    let Some(message) = event.error_message.as_deref() else {
        return Ok(None);
    };
    let service = raw_event
        .get("service")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default();
    let job_type = raw_event
        .get("jobType")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default();
    serde_json::from_value(
        serde_json::to_value(JobErrorDetail::from_message(service, job_type, message)).map_err(
            |error| JobsQueryError::ConvertWireModel {
                model: "job timeline error detail",
                details: error.to_string(),
            },
        )?,
    )
    .map(Some)
    .map_err(|error| JobsQueryError::ConvertWireModel {
        model: "job timeline error detail",
        details: error.to_string(),
    })
}

fn decode_optional_json<T>(
    json: &Option<String>,
    model: &'static str,
) -> Result<Option<T>, JobsQueryError>
where
    T: serde::de::DeserializeOwned,
{
    json.as_ref()
        .map(|json| {
            serde_json::from_str(json).map_err(|error| JobsQueryError::ConvertWireModel {
                model,
                details: error.to_string(),
            })
        })
        .transpose()
}

fn map_optional_wire<T, U>(
    value: &Option<T>,
    model: &'static str,
) -> Result<Option<U>, JobsQueryError>
where
    T: serde::Serialize,
    U: serde::de::DeserializeOwned,
{
    value
        .as_ref()
        .map(|value| {
            serde_json::from_value(serde_json::to_value(value).map_err(|error| {
                JobsQueryError::ConvertWireModel {
                    model,
                    details: error.to_string(),
                }
            })?)
            .map_err(|error| JobsQueryError::ConvertWireModel {
                model,
                details: error.to_string(),
            })
        })
        .transpose()
}

fn map_wire<T: serde::Serialize, U: serde::de::DeserializeOwned>(
    value: &T,
    model: &'static str,
) -> Result<U, JobsQueryError> {
    serde_json::from_value(serde_json::to_value(value).map_err(|error| {
        JobsQueryError::ConvertWireModel {
            model,
            details: error.to_string(),
        }
    })?)
    .map_err(|error| JobsQueryError::ConvertWireModel {
        model,
        details: error.to_string(),
    })
}

fn workbench_group_to_wire(
    group: &JobsWorkbenchGroup,
) -> Result<JobsQueryResponseGroupsItem, JobsQueryError> {
    Ok(JobsQueryResponseGroupsItem {
        count: to_wire_integer(group.count),
        depth: group.depth.map(to_wire_integer),
        failure_rate: group.failure_rate,
        key: group.key.clone(),
        label: group.label.clone(),
        latest_updated_at: group.latest_updated_at.clone(),
        oldest_created_at: group.oldest_created_at.clone(),
        state: map_optional_wire(&group.state, "job query group state")?,
    })
}

fn workbench_stats_to_wire(
    stats: &JobsWorkbenchStats,
) -> Result<JobsQueryResponseStats, JobsQueryError> {
    Ok(JobsQueryResponseStats {
        by_state: stats
            .by_state
            .iter()
            .map(|(key, value)| Ok((key.clone(), to_wire_integer(*value))))
            .collect::<Result<BTreeMap<_, _>, JobsQueryError>>()?,
        dead: stats.dead.map(to_wire_integer),
        failed: stats.failed.map(to_wire_integer),
        queued: stats.queued.map(to_wire_integer),
        running: stats.running.map(to_wire_integer),
        slow: stats.slow.map(to_wire_integer),
        total: to_wire_integer(stats.total),
    })
}

fn metrics_latency_to_summary_wire(
    latency: &JobsMetricsLatency,
) -> JobsMetricsResponseSummaryItemRuntime {
    JobsMetricsResponseSummaryItemRuntime {
        count: to_wire_integer(latency.count),
        max_ms: latency.max_ms.map(to_wire_integer),
        p50_ms: latency.p50_ms.map(to_wire_integer),
        p95_ms: latency.p95_ms.map(to_wire_integer),
    }
}

fn metrics_latency_to_summary_queue_wire(
    latency: &JobsMetricsLatency,
) -> JobsMetricsResponseSummaryItemQueueWait {
    JobsMetricsResponseSummaryItemQueueWait {
        count: to_wire_integer(latency.count),
        max_ms: latency.max_ms.map(to_wire_integer),
        p50_ms: latency.p50_ms.map(to_wire_integer),
        p95_ms: latency.p95_ms.map(to_wire_integer),
    }
}

fn metrics_latency_to_bucket_wire(
    latency: &JobsMetricsLatency,
) -> JobsMetricsResponseBucketsItemGroupsItemRuntime {
    JobsMetricsResponseBucketsItemGroupsItemRuntime {
        count: to_wire_integer(latency.count),
        max_ms: latency.max_ms.map(to_wire_integer),
        p50_ms: latency.p50_ms.map(to_wire_integer),
        p95_ms: latency.p95_ms.map(to_wire_integer),
    }
}

fn metrics_latency_to_bucket_queue_wire(
    latency: &JobsMetricsLatency,
) -> JobsMetricsResponseBucketsItemGroupsItemQueueWait {
    JobsMetricsResponseBucketsItemGroupsItemQueueWait {
        count: to_wire_integer(latency.count),
        max_ms: latency.max_ms.map(to_wire_integer),
        p50_ms: latency.p50_ms.map(to_wire_integer),
        p95_ms: latency.p95_ms.map(to_wire_integer),
    }
}

fn metrics_summary_group_to_wire(
    group: &JobsMetricsSummaryGroup,
) -> Result<JobsMetricsResponseSummaryItem, JobsQueryError> {
    Ok(JobsMetricsResponseSummaryItem {
        by_state: group
            .by_state
            .iter()
            .map(|(key, value)| Ok((key.clone(), to_wire_integer(*value))))
            .collect::<Result<BTreeMap<_, _>, JobsQueryError>>()?,
        dead: group.dead.map(to_wire_integer),
        failed: group.failed.map(to_wire_integer),
        failure_rate: group.failure_rate,
        key: group.key.clone(),
        label: group.label.clone(),
        latest_updated_at: group.latest_updated_at.clone(),
        oldest_created_at: group.oldest_created_at.clone(),
        queue_wait: metrics_latency_to_summary_queue_wire(&group.queue_wait),
        queued: group.queued.map(to_wire_integer),
        running: group.running.map(to_wire_integer),
        runtime: metrics_latency_to_summary_wire(&group.runtime),
        slow: group.slow.map(to_wire_integer),
        total: to_wire_integer(group.total),
    })
}

fn metrics_bucket_to_wire(
    bucket: &JobsMetricsBucket,
) -> Result<JobsMetricsResponseBucketsItem, JobsQueryError> {
    Ok(JobsMetricsResponseBucketsItem {
        end: bucket.end.clone(),
        groups: bucket
            .groups
            .iter()
            .map(metrics_bucket_group_to_wire)
            .collect::<Result<Vec<_>, _>>()?,
        start: bucket.start.clone(),
    })
}

fn metrics_bucket_group_to_wire(
    group: &JobsMetricsBucketGroup,
) -> Result<JobsMetricsResponseBucketsItemGroupsItem, JobsQueryError> {
    Ok(JobsMetricsResponseBucketsItemGroupsItem {
        cancelled: to_wire_integer(group.cancelled),
        completed: to_wire_integer(group.completed),
        dead: to_wire_integer(group.dead),
        dismissed: to_wire_integer(group.dismissed),
        failed: to_wire_integer(group.failed),
        key: group.key.clone(),
        label: group.label.clone(),
        queue_wait: metrics_latency_to_bucket_queue_wire(&group.queue_wait),
        retried: to_wire_integer(group.retried),
        runtime: metrics_latency_to_bucket_wire(&group.runtime),
        started: to_wire_integer(group.started),
        submitted: to_wire_integer(group.submitted),
    })
}

fn heartbeat_age_ms(heartbeat_at: &str, now: OffsetDateTime) -> i64 {
    let Some(heartbeat_at) = OffsetDateTime::parse(heartbeat_at, &Rfc3339).ok() else {
        return 0;
    };
    let age = (now - heartbeat_at).whole_milliseconds();
    if age < 0 {
        0
    } else {
        i64::try_from(age).unwrap_or(i64::MAX)
    }
}

fn parse_positive_integer(field: &'static str, value: i64) -> Result<u64, JobsQueryError> {
    if value < 1 {
        return Err(JobsQueryError::ConvertWireModel {
            model: field,
            details: "must be at least 1".to_string(),
        });
    }
    u64::try_from(value).map_err(|error| JobsQueryError::ConvertWireModel {
        model: field,
        details: error.to_string(),
    })
}

fn parse_non_negative_integer(field: &'static str, value: i64) -> Result<u64, JobsQueryError> {
    if value < 0 {
        return Err(JobsQueryError::ConvertWireModel {
            model: field,
            details: "must be non-negative".to_string(),
        });
    }
    u64::try_from(value).map_err(|error| JobsQueryError::ConvertWireModel {
        model: field,
        details: error.to_string(),
    })
}

fn to_wire_integer(value: u64) -> i64 {
    i64::try_from(value).unwrap_or(i64::MAX)
}

fn projection_key(job: &Job) -> String {
    format!("{}/{}/{}", job.service, job.job_type, job.id)
}

fn raw_event_subject(raw_event_json: &str) -> Option<String> {
    serde_json::from_str::<serde_json::Value>(raw_event_json)
        .ok()?
        .get("_trellisSubject")?
        .as_str()
        .map(str::to_string)
}

fn sibling_event_subject(subject: &str, event_type: &str) -> Option<String> {
    Some(format!("{}.{}", subject.rsplit_once('.')?.0, event_type))
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use trellis_rs::jobs::types::{
        Job, JobContext, JobEvent, JobState, JobWaitEdge, JobWaitTarget, JobWaitTargetKind,
    };
    use trellis_rs::jobs::{cancelled_event, retried_event};

    use super::parse_since_filter;
    use super::{
        dismissed_event, jobs_admin_resources, plan_mutation_response, reduce_job_event,
        timeline_event_to_wire, workbench_entry_to_wire, JobsQueryError, MutationResponsePlan,
    };
    use crate::storage::{JobTimelineEvent, JobsWorkbenchEntry};

    #[test]
    fn jobs_admin_resources_use_builtin_stream_names() {
        let resources = jobs_admin_resources();

        assert_eq!(resources.jobs_stream, "JOBS");
        assert_eq!(resources.jobs_advisories_stream, "JOBS_ADVISORIES");
    }

    fn sample_job(state: JobState) -> Job {
        Job {
            id: "job-1".to_string(),
            context: JobContext {
                request_id: "request-job-1".to_string(),
                trace_id: "0123456789abcdef0123456789abcdef".to_string(),
                traceparent: "00-0123456789abcdef0123456789abcdef-0123456789abcdef-01".to_string(),
                tracestate: None,
            },
            service: "documents".to_string(),
            job_type: "import".to_string(),
            state,
            payload: json!({ "documentId": "doc-1" }),
            result: None,
            created_at: "2026-03-28T12:00:00Z".to_string(),
            updated_at: "2026-03-28T12:00:00Z".to_string(),
            started_at: None,
            completed_at: None,
            tries: 1,
            max_tries: 5,
            last_error: None,
            error_detail: None,
            deadline: Some("2026-03-29T12:00:00Z".to_string()),
            progress: None,
            logs: None,
            concurrency: None,
            queue_policy: None,
            trigger: None,
            lineage: None,
            waiting_on: None,
        }
    }

    fn sample_wait_edge() -> JobWaitEdge {
        JobWaitEdge {
            id: "wait-1".to_string(),
            target: JobWaitTarget {
                kind: JobWaitTargetKind::Job,
                id: Some("child-job".to_string()),
                operation_id: None,
                label: None,
                service: Some("documents".to_string()),
                target_type: Some("import".to_string()),
                system: None,
                operation: None,
                key: None,
            },
            started_at: "2026-03-28T12:01:00Z".to_string(),
            label: Some("child job".to_string()),
        }
    }

    fn predicted_job(job: &Job, event: &JobEvent) -> Job {
        reduce_job_event(Some(job), event).expect("admin event should reduce")
    }

    #[test]
    fn cancel_response_plan_returns_predicted_job_when_projection_lags() {
        let job = sample_job(JobState::Pending);
        let event = cancelled_event(
            &job.service,
            &job.job_type,
            &job.id,
            &job.context,
            job.state,
            job.tries,
            "2026-03-28T12:01:00Z",
        );
        let predicted = predicted_job(&job, &event);

        assert_eq!(predicted.state, JobState::Cancelled);
        assert_eq!(
            plan_mutation_response(None, &predicted, &event, true),
            MutationResponsePlan::ReturnPredicted
        );
    }

    #[test]
    fn retry_response_plan_returns_predicted_job_when_projection_lags() {
        let job = sample_job(JobState::Failed);
        let event = retried_event(
            &job.service,
            &job.job_type,
            &job.id,
            &job.context,
            job.state,
            "2026-03-28T12:01:00Z",
            Some(job.payload.clone()),
            Some(job.max_tries),
            job.deadline.as_deref(),
        );
        let predicted = predicted_job(&job, &event);

        assert_eq!(predicted.state, JobState::Pending);
        assert_eq!(
            plan_mutation_response(None, &predicted, &event, true),
            MutationResponsePlan::ReturnPredicted
        );
    }

    #[test]
    fn replay_response_plan_returns_predicted_job_when_projection_lags() {
        let job = sample_job(JobState::Dead);
        let event = retried_event(
            &job.service,
            &job.job_type,
            &job.id,
            &job.context,
            job.state,
            "2026-03-28T12:01:00Z",
            Some(job.payload.clone()),
            Some(job.max_tries),
            job.deadline.as_deref(),
        );
        let predicted = predicted_job(&job, &event);

        assert_eq!(predicted.state, JobState::Pending);
        assert_eq!(
            plan_mutation_response(None, &predicted, &event, true),
            MutationResponsePlan::ReturnPredicted
        );
    }

    #[test]
    fn dismiss_response_plan_returns_predicted_job_when_projection_lags() {
        let job = sample_job(JobState::Dead);
        let event = dismissed_event(
            &job.service,
            &job.job_type,
            &job.id,
            &job.context,
            JobState::Dead,
            job.tries,
            "2026-03-28T12:01:00Z",
            job.last_error.as_deref(),
        );
        let predicted = predicted_job(&job, &event);

        assert_eq!(predicted.state, JobState::Dismissed);
        assert_eq!(
            plan_mutation_response(None, &predicted, &event, true),
            MutationResponsePlan::ReturnPredicted
        );
    }

    #[test]
    fn terminal_projection_race_returns_projected_terminal_job() {
        let job = sample_job(JobState::Pending);
        let event = cancelled_event(
            &job.service,
            &job.job_type,
            &job.id,
            &job.context,
            job.state,
            job.tries,
            "2026-03-28T12:01:00Z",
        );
        let predicted = predicted_job(&job, &event);
        let mut projected = job.clone();
        projected.state = JobState::Completed;
        projected.updated_at = "2026-03-28T12:00:30Z".to_string();
        projected.completed_at = Some(projected.updated_at.clone());

        assert_eq!(
            plan_mutation_response(Some(&projected), &predicted, &event, false),
            MutationResponsePlan::ReturnProjected
        );
    }

    #[test]
    fn query_wire_maps_waiting_on_and_timeline_wait_edge() {
        let entry = JobsWorkbenchEntry {
            job: sample_job(JobState::Active),
            runtime_ms: None,
            queue_age_anchor_nanos: None,
            queue_key: None,
            runtime_band: None,
            last_error_fingerprint: None,
            matched_by: None,
            waiting_on: vec![sample_wait_edge()],
        };

        let row = workbench_entry_to_wire(&entry).expect("query row should map");
        assert_eq!(
            row.waiting_on
                .as_ref()
                .and_then(|waits| waits.first())
                .map(|wait| wait.id.as_str()),
            Some("wait-1")
        );

        let timeline = JobTimelineEvent {
            sequence: 1,
            event_type: "waiting".to_string(),
            state: "active".to_string(),
            previous_state: Some("active".to_string()),
            timestamp: "2026-03-28T12:01:00Z".to_string(),
            tries: 1,
            message: None,
            error_message: None,
            progress_json: None,
            logs_json: None,
            worker_instance_id: None,
            raw_event_json: json!({ "waitEdge": sample_wait_edge() }).to_string(),
            projected: Some(true),
            reason: None,
        };

        let timeline_row = timeline_event_to_wire(&timeline).expect("timeline row should map");
        assert_eq!(
            timeline_row.wait_edge.map(|wait| wait.id),
            Some("wait-1".to_string())
        );
    }

    #[test]
    fn parse_since_filter_accepts_rfc3339_offset_timestamps() {
        let parsed = parse_since_filter(Some("2025-12-31T19:00:30-05:00"))
            .expect("offset timestamp should parse")
            .expect("since should be present");

        assert_eq!(parsed.unix_timestamp(), 1_767_225_630);
    }

    #[test]
    fn parse_since_filter_rejects_invalid_timestamps_as_validation_errors() {
        let error =
            parse_since_filter(Some("not-a-timestamp")).expect_err("invalid timestamp should fail");

        assert!(matches!(
            error,
            JobsQueryError::Validation { field: "since", .. }
        ));
    }
}
