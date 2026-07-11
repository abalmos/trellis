use std::collections::BTreeMap;
use std::path::Path;
use std::sync::{Arc, Mutex};

use rusqlite::{params, params_from_iter, types::Value as SqlValue, Connection, OptionalExtension};
use serde_json::{json, Value};
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

/// SQLite-backed Event Log projection store.
#[derive(Debug, Clone)]
pub struct EventLogStore {
    connection: Arc<Mutex<Connection>>,
}

/// One projected Trellis event row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectedEvent {
    /// JetStream stream sequence.
    pub stream_sequence: u64,
    /// Trellis event id, normally from `Nats-Msg-Id`.
    pub event_id: Option<String>,
    /// Event timestamp.
    pub event_time: String,
    /// Concrete event subject.
    pub subject: String,
    /// Owner contract id resolved from the catalog, when available.
    pub owner_contract_id: Option<String>,
    /// Owner event name resolved from the catalog, when available.
    pub owner_event_name: Option<String>,
    /// Resolution status.
    pub resolution: String,
    /// Auth proof validation status.
    pub verification_status: String,
    /// Publisher kind from `Auth.Events.Validate`.
    pub publisher_kind: Option<String>,
    /// Publisher deployment id from `Auth.Events.Validate`.
    pub publisher_deployment_id: Option<String>,
    /// Publisher instance id from `Auth.Events.Validate`.
    pub publisher_instance_id: Option<String>,
    /// Publisher contract id from `Auth.Events.Validate`.
    pub publisher_contract_id: Option<String>,
    /// Publisher contract digest from `Auth.Events.Validate`.
    pub publisher_contract_digest: Option<String>,
    /// Publisher session status from `Auth.Events.Validate`.
    pub publisher_session_status: Option<String>,
    /// W3C trace id, when traceparent is present.
    pub trace_id: Option<String>,
    /// W3C traceparent header.
    pub traceparent: Option<String>,
    /// Raw payload bytes.
    pub payload_bytes: Vec<u8>,
    /// JSON object of message headers.
    pub headers_json: String,
    /// Decoded JSON payload cache.
    pub payload_json: Option<String>,
    /// UTF-8 payload cache.
    pub payload_text: Option<String>,
    /// Decode error for non-text payloads.
    pub decode_error: Option<String>,
    /// Projection timestamp.
    pub projected_at: String,
}

/// A resolved Event Log event type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventTypeRef {
    /// Owner contract id.
    pub owner_contract_id: String,
    /// Owner event name.
    pub owner_event_name: String,
}

/// Filters supported by `EventLog.Query`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EventLogFilter {
    /// Free-text search over indexed metadata fields.
    pub search: Option<String>,
    /// Exact event subject.
    pub subject: Option<String>,
    /// Owner contract id.
    pub owner_contract_id: Option<String>,
    /// Owner event name.
    pub owner_event_name: Option<String>,
    /// Event types to include.
    pub include_event_types: Vec<EventTypeRef>,
    /// Event types to exclude.
    pub exclude_event_types: Vec<EventTypeRef>,
    /// Publisher deployment id.
    pub publisher_deployment_id: Option<String>,
    /// Resolution statuses.
    pub resolution: Vec<String>,
    /// Verification statuses.
    pub verification_status: Vec<String>,
    /// Whether to include only events with a resolution or verification exception.
    pub integrity_exception_only: bool,
    /// Lower bound for event time.
    pub since: Option<String>,
    /// Page offset.
    pub offset: u64,
    /// Page limit.
    pub limit: u64,
    /// Sort field.
    pub sort_field: String,
    /// Sort direction.
    pub sort_direction: String,
}

/// Errors returned by the SQLite Event Log projection store.
#[derive(Debug, thiserror::Error)]
pub enum EventLogStoreError {
    /// SQLite operation failed.
    #[error("sqlite operation failed: {0}")]
    Sqlite(#[from] rusqlite::Error),
    /// SQLite connection lock was poisoned.
    #[error("sqlite connection lock is poisoned")]
    Poisoned,
    /// JSON encode/decode failed.
    #[error("failed to encode {model}: {details}")]
    EncodeJson {
        /// Model name.
        model: &'static str,
        /// Error details.
        details: String,
    },
    /// Stream sequence cannot be represented in SQLite's signed integer range.
    #[error("stream sequence {0} is outside SQLite integer range")]
    SequenceOutOfRange(u64),
}

impl EventLogStore {
    /// Open a store at `path` and initialize the schema if needed.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, EventLogStoreError> {
        let store = Self {
            connection: Arc::new(Mutex::new(Connection::open(path)?)),
        };
        store.initialize_schema()?;
        Ok(store)
    }

    /// Open an in-memory store and initialize the schema. Intended for tests.
    pub fn open_in_memory() -> Result<Self, EventLogStoreError> {
        let store = Self {
            connection: Arc::new(Mutex::new(Connection::open_in_memory()?)),
        };
        store.initialize_schema()?;
        Ok(store)
    }

    /// Initialize the projection schema. Safe to call more than once.
    pub fn initialize_schema(&self) -> Result<(), EventLogStoreError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| EventLogStoreError::Poisoned)?;
        connection.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS eventlog_projection_metadata (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS eventlog_events (
                stream_sequence INTEGER PRIMARY KEY,
                event_id TEXT,
                event_time TEXT NOT NULL,
                subject TEXT NOT NULL,
                owner_contract_id TEXT,
                owner_event_name TEXT,
                resolution TEXT NOT NULL,
                verification_status TEXT NOT NULL,
                publisher_kind TEXT,
                publisher_deployment_id TEXT,
                publisher_instance_id TEXT,
                publisher_contract_id TEXT,
                publisher_contract_digest TEXT,
                publisher_session_status TEXT,
                trace_id TEXT,
                traceparent TEXT,
                payload_size_bytes INTEGER NOT NULL,
                payload_bytes BLOB NOT NULL,
                headers_json TEXT NOT NULL,
                payload_json TEXT,
                payload_text TEXT,
                decode_error TEXT,
                projected_at TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_eventlog_events_event_id ON eventlog_events (event_id);
            CREATE INDEX IF NOT EXISTS idx_eventlog_events_event_time ON eventlog_events (event_time DESC);
            CREATE INDEX IF NOT EXISTS idx_eventlog_events_subject ON eventlog_events (subject);
            CREATE INDEX IF NOT EXISTS idx_eventlog_events_owner ON eventlog_events (owner_contract_id, owner_event_name);
            CREATE INDEX IF NOT EXISTS idx_eventlog_events_publisher_deployment ON eventlog_events (publisher_deployment_id);
            CREATE INDEX IF NOT EXISTS idx_eventlog_events_publisher_contract ON eventlog_events (publisher_contract_id);
            CREATE INDEX IF NOT EXISTS idx_eventlog_events_trace ON eventlog_events (trace_id);
            CREATE INDEX IF NOT EXISTS idx_eventlog_events_resolution ON eventlog_events (resolution);
            CREATE INDEX IF NOT EXISTS idx_eventlog_events_verification ON eventlog_events (verification_status);
            CREATE TABLE IF NOT EXISTS eventlog_consumer_samples (
                sampled_at TEXT NOT NULL,
                consumer_name TEXT NOT NULL,
                deployment_id TEXT,
                contract_id TEXT,
                group_name TEXT,
                status TEXT NOT NULL,
                pending INTEGER NOT NULL,
                ack_pending INTEGER NOT NULL,
                waiting_pulls INTEGER NOT NULL,
                redelivered INTEGER,
                oldest_pending_at TEXT,
                PRIMARY KEY (sampled_at, consumer_name)
            );
            "#,
        )?;
        let projection_id = format!("{}-{}", std::process::id(), now_timestamp_string());
        connection.execute(
            "INSERT OR IGNORE INTO eventlog_projection_metadata (key, value) VALUES ('projection_id', ?1)",
            params![projection_id],
        )?;
        connection.execute(
            "INSERT OR IGNORE INTO eventlog_projection_metadata (key, value) VALUES ('last_projected_sequence', '0')",
            [],
        )?;
        Ok(())
    }

    /// Return this projection database's stable id.
    pub fn projection_id(&self) -> Result<String, EventLogStoreError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| EventLogStoreError::Poisoned)?;
        Ok(connection.query_row(
            "SELECT value FROM eventlog_projection_metadata WHERE key = 'projection_id'",
            [],
            |row| row.get(0),
        )?)
    }

    /// Persist one projected event and advance projection metadata.
    pub fn insert_event(&self, event: &ProjectedEvent) -> Result<(), EventLogStoreError> {
        let stream_sequence_i64 = i64::try_from(event.stream_sequence)
            .map_err(|_| EventLogStoreError::SequenceOutOfRange(event.stream_sequence))?;
        let mut connection = self
            .connection
            .lock()
            .map_err(|_| EventLogStoreError::Poisoned)?;
        let transaction = connection.transaction()?;
        transaction.execute(
            r#"
            INSERT OR REPLACE INTO eventlog_events (
                stream_sequence, event_id, event_time, subject, owner_contract_id,
                owner_event_name, resolution, verification_status, publisher_kind,
                publisher_deployment_id, publisher_instance_id, publisher_contract_id,
                publisher_contract_digest, publisher_session_status, trace_id, traceparent,
                payload_size_bytes, payload_bytes, headers_json, payload_json, payload_text,
                decode_error, projected_at
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15,
                ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23
            )
            "#,
            params![
                event.stream_sequence,
                event.event_id,
                event.event_time,
                event.subject,
                event.owner_contract_id,
                event.owner_event_name,
                event.resolution,
                event.verification_status,
                event.publisher_kind,
                event.publisher_deployment_id,
                event.publisher_instance_id,
                event.publisher_contract_id,
                event.publisher_contract_digest,
                event.publisher_session_status,
                event.trace_id,
                event.traceparent,
                event.payload_bytes.len() as u64,
                event.payload_bytes,
                event.headers_json,
                event.payload_json,
                event.payload_text,
                event.decode_error,
                event.projected_at,
            ],
        )?;
        transaction.execute(
            r#"
            UPDATE eventlog_projection_metadata
            SET value = CASE
                WHEN CAST(value AS INTEGER) < ?1 THEN ?2
                ELSE value
            END
            WHERE key = 'last_projected_sequence'
            "#,
            params![stream_sequence_i64, event.stream_sequence.to_string()],
        )?;
        transaction.commit()?;
        Ok(())
    }

    /// Query projected event rows and return `(rows, total)`.
    pub fn query_events(
        &self,
        filter: &EventLogFilter,
    ) -> Result<(Vec<Value>, u64), EventLogStoreError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| EventLogStoreError::Poisoned)?;
        let (where_sql, params) = event_where_clause(filter);
        let total = connection.query_row(
            &format!("SELECT COUNT(*) FROM eventlog_events {where_sql}"),
            params_from_iter(params.clone()),
            |row| row.get::<_, u64>(0),
        )?;
        let order_field = match filter.sort_field.as_str() {
            "subject" => "subject",
            "payloadSize" => "payload_size_bytes",
            _ => "event_time",
        };
        let order_direction = if filter.sort_direction == "asc" {
            "ASC"
        } else {
            "DESC"
        };
        let mut query_params = params;
        query_params.push(SqlValue::from(filter.limit as i64));
        query_params.push(SqlValue::from(filter.offset as i64));
        let mut statement = connection.prepare(&format!(
            "SELECT stream_sequence, event_id, event_time, subject, owner_contract_id, owner_event_name, resolution, verification_status, publisher_kind, publisher_deployment_id, publisher_instance_id, publisher_contract_id, publisher_contract_digest, trace_id, payload_size_bytes, headers_json FROM eventlog_events {where_sql} ORDER BY {order_field} {order_direction}, stream_sequence DESC LIMIT ? OFFSET ?"
        ))?;
        let rows = statement
            .query_map(params_from_iter(query_params), row_to_summary_value)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok((rows, total))
    }

    /// Inspect one event by id or stream sequence.
    pub fn inspect_event(
        &self,
        event_id: Option<&str>,
        stream_sequence: Option<u64>,
    ) -> Result<Option<Value>, EventLogStoreError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| EventLogStoreError::Poisoned)?;
        let row = if let Some(sequence) = stream_sequence {
            connection
                .query_row(
                    &inspect_sql("stream_sequence = ?1"),
                    params![sequence],
                    row_to_inspect_value,
                )
                .optional()?
        } else if let Some(id) = event_id {
            connection
                .query_row(
                    &inspect_sql("event_id = ?1"),
                    params![id],
                    row_to_inspect_value,
                )
                .optional()?
        } else {
            None
        };
        let Some(mut value) = row else {
            return Ok(None);
        };
        let related = related_events(&connection, &value)?;
        if let Some(object) = value.as_object_mut() {
            object.insert("related".to_string(), Value::Array(related));
        }
        Ok(Some(value))
    }

    /// Return simple Event Log metrics from the projection.
    pub fn metrics(&self, window: Option<(&str, i64, i64)>) -> Result<Value, EventLogStoreError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| EventLogStoreError::Poisoned)?;
        let mut where_sql = String::new();
        let mut params = Vec::new();
        if let Some((since, _, _)) = window {
            where_sql.push_str("WHERE event_time >= ?");
            params.push(SqlValue::from(since.to_string()));
        }
        let total = connection.query_row(
            &format!("SELECT COUNT(*) FROM eventlog_events {where_sql}"),
            params_from_iter(params.clone()),
            |row| row.get::<_, u64>(0),
        )?;
        let unique_subjects = connection.query_row(
            &format!("SELECT COUNT(DISTINCT subject) FROM eventlog_events {where_sql}"),
            params_from_iter(params.clone()),
            |row| row.get::<_, u64>(0),
        )?;
        let payload_bytes = connection.query_row(
            &format!(
                "SELECT COALESCE(SUM(payload_size_bytes), 0) FROM eventlog_events {where_sql}"
            ),
            params_from_iter(params.clone()),
            |row| row.get::<_, u64>(0),
        )?;
        let integrity_exceptions = connection.query_row(
            &format!(
                "SELECT COUNT(*) FROM eventlog_events {where_sql} {} (resolution != 'resolved' OR verification_status != 'verified')",
                if where_sql.is_empty() { "WHERE" } else { " AND" }
            ),
            params_from_iter(params.clone()),
            |row| row.get::<_, u64>(0),
        )?;
        let event_type_where = if where_sql.is_empty() {
            "WHERE owner_contract_id IS NOT NULL AND owner_event_name IS NOT NULL".to_string()
        } else {
            format!(
                "{where_sql} AND owner_contract_id IS NOT NULL AND owner_event_name IS NOT NULL"
            )
        };
        let mut statement = connection.prepare(&format!(
            "SELECT owner_contract_id, owner_event_name, COUNT(*) FROM eventlog_events {event_type_where} GROUP BY owner_contract_id, owner_event_name ORDER BY COUNT(*) DESC, owner_contract_id, owner_event_name"
        ))?;
        let event_types = statement
            .query_map(params_from_iter(params.iter().cloned()), |row| {
                Ok(json!({
                    "ownerContractId": row.get::<_, String>(0)?,
                    "ownerEventName": row.get::<_, String>(1)?,
                    "count": row.get::<_, u64>(2)?,
                }))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        let buckets = if let Some((since, window_seconds, bucket_seconds)) = window {
            let mut bucket_params = vec![
                SqlValue::from(bucket_seconds),
                SqlValue::from(bucket_seconds),
            ];
            bucket_params.extend(params.iter().cloned());
            let mut statement = connection.prepare(&format!(
                r#"
                SELECT
                    strftime('%Y-%m-%dT%H:%M:%SZ', (CAST(strftime('%s', event_time) AS INTEGER) / ?) * ?, 'unixepoch'),
                    COUNT(*),
                    COALESCE(SUM(payload_size_bytes), 0),
                    SUM(resolution != 'resolved' OR verification_status != 'verified'),
                    SUM(resolution = 'resolved'),
                    SUM(resolution = 'unresolved'),
                    SUM(resolution = 'malformed'),
                    SUM(verification_status = 'verified'),
                    SUM(verification_status = 'missing-proof'),
                    SUM(verification_status = 'invalid-signature'),
                    SUM(verification_status = 'missing-session'),
                    SUM(verification_status = 'subject-denied'),
                    SUM(verification_status = 'outside-session-window'),
                    SUM(verification_status = 'auth-unavailable')
                FROM eventlog_events {where_sql}
                GROUP BY 1
                ORDER BY 1
                "#
            ))?;
            let buckets = statement
                .query_map(params_from_iter(bucket_params), |row| {
                    Ok(json!({
                        "start": row.get::<_, String>(0)?,
                        "total": row.get::<_, u64>(1)?,
                        "payloadSizeBytes": row.get::<_, u64>(2)?,
                        "integrityExceptions": row.get::<_, u64>(3)?,
                        "byResolution": {
                            "resolved": row.get::<_, u64>(4)?,
                            "unresolved": row.get::<_, u64>(5)?,
                            "malformed": row.get::<_, u64>(6)?,
                        },
                        "byVerificationStatus": {
                            "verified": row.get::<_, u64>(7)?,
                            "missing-proof": row.get::<_, u64>(8)?,
                            "invalid-signature": row.get::<_, u64>(9)?,
                            "missing-session": row.get::<_, u64>(10)?,
                            "subject-denied": row.get::<_, u64>(11)?,
                            "outside-session-window": row.get::<_, u64>(12)?,
                            "auth-unavailable": row.get::<_, u64>(13)?,
                        }
                    }))
                })?
                .collect::<Result<Vec<_>, _>>()?;
            let mut buckets_by_start = buckets
                .into_iter()
                .filter_map(|bucket| Some((bucket.get("start")?.as_str()?.to_string(), bucket)))
                .collect::<BTreeMap<_, _>>();
            OffsetDateTime::parse(since, &Rfc3339)
                .ok()
                .map(|since| {
                    let start = since.unix_timestamp().div_euclid(bucket_seconds) * bucket_seconds;
                    let end = (since.unix_timestamp() + window_seconds).div_euclid(bucket_seconds)
                        * bucket_seconds;
                    (start..=end)
                        .step_by(bucket_seconds as usize)
                        .filter_map(|timestamp| {
                            let start = OffsetDateTime::from_unix_timestamp(timestamp)
                                .ok()?
                                .format(&Rfc3339)
                                .ok()?;
                            Some(buckets_by_start.remove(&start).unwrap_or_else(|| {
                                json!({
                                    "start": start,
                                    "total": 0,
                                    "payloadSizeBytes": 0,
                                    "integrityExceptions": 0,
                                    "byResolution": {
                                        "resolved": 0,
                                        "unresolved": 0,
                                        "malformed": 0,
                                    },
                                    "byVerificationStatus": {
                                        "verified": 0,
                                        "missing-proof": 0,
                                        "invalid-signature": 0,
                                        "missing-session": 0,
                                        "subject-denied": 0,
                                        "outside-session-window": 0,
                                        "auth-unavailable": 0,
                                    }
                                })
                            }))
                        })
                        .collect()
                })
                .unwrap_or_else(|| buckets_by_start.into_values().collect())
        } else {
            Vec::new()
        };
        Ok(json!({
            "summary": {
                "total": total,
                "uniqueSubjects": unique_subjects,
                "payloadSizeBytes": payload_bytes,
                "integrityExceptions": integrity_exceptions,
                "byResolution": grouped_counts(&connection, "resolution", &where_sql, &params)?,
                "byVerificationStatus": grouped_counts(&connection, "verification_status", &where_sql, &params)?,
                "eventTypes": event_types,
            },
            "buckets": buckets
        }))
    }
}

fn event_where_clause(filter: &EventLogFilter) -> (String, Vec<SqlValue>) {
    let mut clauses = Vec::new();
    let mut params = Vec::new();
    if let Some(value) = filter.subject.as_ref() {
        clauses.push("subject = ?".to_string());
        params.push(SqlValue::from(value.clone()));
    }
    if let Some(value) = filter.owner_contract_id.as_ref() {
        clauses.push("owner_contract_id = ?".to_string());
        params.push(SqlValue::from(value.clone()));
    }
    if let Some(value) = filter.owner_event_name.as_ref() {
        clauses.push("owner_event_name = ?".to_string());
        params.push(SqlValue::from(value.clone()));
    }
    if !filter.include_event_types.is_empty() {
        clauses.push(format!(
            "({})",
            vec![
                "(owner_contract_id = ? AND owner_event_name = ?)";
                filter.include_event_types.len()
            ]
            .join(" OR ")
        ));
        for event_type in &filter.include_event_types {
            params.push(SqlValue::from(event_type.owner_contract_id.clone()));
            params.push(SqlValue::from(event_type.owner_event_name.clone()));
        }
    }
    if !filter.exclude_event_types.is_empty() {
        clauses.push(format!(
            "(owner_contract_id IS NULL OR owner_event_name IS NULL OR NOT ({}))",
            vec![
                "(owner_contract_id = ? AND owner_event_name = ?)";
                filter.exclude_event_types.len()
            ]
            .join(" OR ")
        ));
        for event_type in &filter.exclude_event_types {
            params.push(SqlValue::from(event_type.owner_contract_id.clone()));
            params.push(SqlValue::from(event_type.owner_event_name.clone()));
        }
    }
    if let Some(value) = filter.publisher_deployment_id.as_ref() {
        clauses.push("publisher_deployment_id = ?".to_string());
        params.push(SqlValue::from(value.clone()));
    }
    if let Some(value) = filter.since.as_ref() {
        clauses.push("event_time >= ?".to_string());
        params.push(SqlValue::from(value.clone()));
    }
    add_in_clause("resolution", &filter.resolution, &mut clauses, &mut params);
    add_in_clause(
        "verification_status",
        &filter.verification_status,
        &mut clauses,
        &mut params,
    );
    if filter.integrity_exception_only {
        clauses.push("(resolution != 'resolved' OR verification_status != 'verified')".to_string());
    }
    if let Some(value) = filter
        .search
        .as_ref()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
    {
        clauses.push("(event_id LIKE ? OR subject LIKE ? OR owner_contract_id LIKE ? OR owner_event_name LIKE ? OR publisher_deployment_id LIKE ? OR publisher_contract_id LIKE ? OR trace_id LIKE ?)".to_string());
        let pattern = format!("%{}%", value.replace('%', "\\%"));
        for _ in 0..7 {
            params.push(SqlValue::from(pattern.clone()));
        }
    }
    if clauses.is_empty() {
        (String::new(), params)
    } else {
        (format!("WHERE {}", clauses.join(" AND ")), params)
    }
}

fn add_in_clause(
    field: &str,
    values: &[String],
    clauses: &mut Vec<String>,
    params: &mut Vec<SqlValue>,
) {
    if values.is_empty() {
        return;
    }
    clauses.push(format!(
        "{field} IN ({})",
        vec!["?"; values.len()].join(", ")
    ));
    params.extend(values.iter().cloned().map(SqlValue::from));
}

fn row_to_summary_value(row: &rusqlite::Row<'_>) -> rusqlite::Result<Value> {
    let stream_sequence: u64 = row.get(0)?;
    let payload_size_bytes: u64 = row.get(14)?;
    let headers_json: String = row.get(15)?;
    let header_count = serde_json::from_str::<Value>(&headers_json)
        .ok()
        .and_then(|value| value.as_object().map(serde_json::Map::len))
        .unwrap_or(0);
    Ok(json!({
        "eventId": row.get::<_, Option<String>>(1)?.unwrap_or_default(),
        "eventTime": row.get::<_, String>(2)?,
        "streamSequence": stream_sequence,
        "subject": row.get::<_, String>(3)?,
        "ownerContractId": row.get::<_, Option<String>>(4)?,
        "ownerEventName": row.get::<_, Option<String>>(5)?,
        "resolution": row.get::<_, String>(6)?,
        "verificationStatus": row.get::<_, String>(7)?,
        "publisherKind": row.get::<_, Option<String>>(8)?,
        "publisherDeploymentId": row.get::<_, Option<String>>(9)?,
        "publisherInstanceId": row.get::<_, Option<String>>(10)?,
        "publisherContractId": row.get::<_, Option<String>>(11)?,
        "publisherContractDigest": row.get::<_, Option<String>>(12)?,
        "traceId": row.get::<_, Option<String>>(13)?,
        "payloadSizeBytes": payload_size_bytes,
        "headerCount": header_count,
    }))
}

fn inspect_sql(predicate: &str) -> String {
    format!(
        "SELECT stream_sequence, event_id, event_time, subject, owner_contract_id, owner_event_name, resolution, verification_status, publisher_kind, publisher_deployment_id, publisher_instance_id, publisher_contract_id, publisher_contract_digest, publisher_session_status, trace_id, traceparent, payload_size_bytes, headers_json, payload_json, payload_text, decode_error, projected_at FROM eventlog_events WHERE {predicate} ORDER BY stream_sequence DESC LIMIT 1"
    )
}

fn row_to_inspect_value(row: &rusqlite::Row<'_>) -> rusqlite::Result<Value> {
    let event = row_to_summary_from_inspect_row(row)?;
    let headers_json: String = row.get(17)?;
    let payload_json: Option<String> = row.get(18)?;
    let payload_text: Option<String> = row.get(19)?;
    let decode_error: Option<String> = row.get(20)?;
    let headers = serde_json::from_str::<Value>(&headers_json).unwrap_or_else(|_| json!({}));
    let payload = payload_json
        .as_deref()
        .and_then(|value| serde_json::from_str::<Value>(value).ok());
    Ok(json!({
        "event": event,
        "headers": headers,
        "payload": payload,
        "payloadText": payload_text,
        "decodeError": decode_error,
        "proof": {
            "status": row.get::<_, String>(7)?,
            "checkedAt": row.get::<_, String>(21)?,
        },
        "owner": {
            "contractId": row.get::<_, Option<String>>(4)?,
            "eventName": row.get::<_, Option<String>>(5)?,
        },
        "publisher": {
            "kind": row.get::<_, Option<String>>(8)?,
            "deploymentId": row.get::<_, Option<String>>(9)?,
            "instanceId": row.get::<_, Option<String>>(10)?,
            "contractId": row.get::<_, Option<String>>(11)?,
            "contractDigest": row.get::<_, Option<String>>(12)?,
            "sessionStatus": row.get::<_, Option<String>>(13)?,
        },
        "related": []
    }))
}

fn row_to_summary_from_inspect_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Value> {
    let payload_size_bytes: u64 = row.get(16)?;
    let headers_json: String = row.get(17)?;
    let header_count = serde_json::from_str::<Value>(&headers_json)
        .ok()
        .and_then(|value| value.as_object().map(serde_json::Map::len))
        .unwrap_or(0);
    Ok(json!({
        "eventId": row.get::<_, Option<String>>(1)?.unwrap_or_default(),
        "eventTime": row.get::<_, String>(2)?,
        "streamSequence": row.get::<_, u64>(0)?,
        "subject": row.get::<_, String>(3)?,
        "ownerContractId": row.get::<_, Option<String>>(4)?,
        "ownerEventName": row.get::<_, Option<String>>(5)?,
        "resolution": row.get::<_, String>(6)?,
        "verificationStatus": row.get::<_, String>(7)?,
        "publisherKind": row.get::<_, Option<String>>(8)?,
        "publisherDeploymentId": row.get::<_, Option<String>>(9)?,
        "publisherInstanceId": row.get::<_, Option<String>>(10)?,
        "publisherContractId": row.get::<_, Option<String>>(11)?,
        "publisherContractDigest": row.get::<_, Option<String>>(12)?,
        "traceId": row.get::<_, Option<String>>(14)?,
        "payloadSizeBytes": payload_size_bytes,
        "headerCount": header_count,
    }))
}

fn related_events(
    connection: &Connection,
    value: &Value,
) -> Result<Vec<Value>, EventLogStoreError> {
    let Some(event) = value.get("event") else {
        return Ok(Vec::new());
    };
    let stream_sequence = event
        .get("streamSequence")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let trace_id = event.get("traceId").and_then(Value::as_str);
    let subject = event.get("subject").and_then(Value::as_str);
    let publisher = event.get("publisherDeploymentId").and_then(Value::as_str);
    let mut clauses = Vec::new();
    let mut params = Vec::new();
    if let Some(trace_id) = trace_id {
        clauses.push("trace_id = ?");
        params.push(SqlValue::from(trace_id.to_string()));
    }
    if let Some(subject) = subject {
        clauses.push("subject = ?");
        params.push(SqlValue::from(subject.to_string()));
    }
    if let Some(publisher) = publisher {
        clauses.push("publisher_deployment_id = ?");
        params.push(SqlValue::from(publisher.to_string()));
    }
    if clauses.is_empty() {
        return Ok(Vec::new());
    }
    params.push(SqlValue::from(stream_sequence as i64));
    let mut statement = connection.prepare(&format!(
        "SELECT event_id, event_time, subject, trace_id, publisher_deployment_id FROM eventlog_events WHERE ({}) AND stream_sequence != ? ORDER BY event_time DESC LIMIT 20",
        clauses.join(" OR ")
    ))?;
    let related = statement
        .query_map(params_from_iter(params), |row| {
            let row_trace: Option<String> = row.get(3)?;
            let row_publisher: Option<String> = row.get(4)?;
            let row_subject: String = row.get(2)?;
            let matched_by = if trace_id.is_some() && row_trace.as_deref() == trace_id {
                "trace"
            } else if publisher.is_some() && row_publisher.as_deref() == publisher {
                "publisher"
            } else if subject.is_some() && row_subject == subject.unwrap_or_default() {
                "subject"
            } else {
                "subject"
            };
            Ok(json!({
                "eventId": row.get::<_, Option<String>>(0)?.unwrap_or_default(),
                "eventTime": row.get::<_, String>(1)?,
                "subject": row_subject,
                "matchedBy": matched_by,
            }))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(related)
}

fn grouped_counts(
    connection: &Connection,
    field: &str,
    where_sql: &str,
    params: &[SqlValue],
) -> Result<Value, EventLogStoreError> {
    let mut statement = connection.prepare(&format!(
        "SELECT {field}, COUNT(*) FROM eventlog_events {where_sql} GROUP BY {field}"
    ))?;
    let mut object = serde_json::Map::new();
    for row in statement.query_map(params_from_iter(params.iter().cloned()), |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, u64>(1)?))
    })? {
        let (key, count) = row?;
        object.insert(key, json!(count));
    }
    Ok(Value::Object(object))
}

/// Return the current UTC timestamp in RFC3339 form.
pub fn now_timestamp_string() -> String {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string())
}

#[cfg(test)]
mod tests {
    use super::{EventLogFilter, EventLogStore, EventTypeRef, ProjectedEvent};

    fn event(stream_sequence: u64) -> ProjectedEvent {
        ProjectedEvent {
            stream_sequence,
            event_id: Some(format!("event-{stream_sequence}")),
            event_time: "2026-01-01T00:00:00Z".to_string(),
            subject: "events.v1.Test.Created".to_string(),
            owner_contract_id: None,
            owner_event_name: None,
            resolution: "unresolved".to_string(),
            verification_status: "valid".to_string(),
            publisher_kind: None,
            publisher_deployment_id: None,
            publisher_instance_id: None,
            publisher_contract_id: None,
            publisher_contract_digest: None,
            publisher_session_status: None,
            trace_id: None,
            traceparent: None,
            payload_bytes: b"{}".to_vec(),
            headers_json: "{}".to_string(),
            payload_json: Some("{}".to_string()),
            payload_text: Some("{}".to_string()),
            decode_error: None,
            projected_at: "2026-01-01T00:00:00Z".to_string(),
        }
    }

    #[test]
    fn insert_event_keeps_projection_watermark_monotonic() {
        let store = EventLogStore::open_in_memory().expect("open store");

        store
            .insert_event(&event(20))
            .expect("insert high sequence");
        store
            .insert_event(&event(10))
            .expect("insert lower sequence");

        let connection = store.connection.lock().expect("lock store");
        let value: String = connection
            .query_row(
                "SELECT value FROM eventlog_projection_metadata WHERE key = 'last_projected_sequence'",
                [],
                |row| row.get(0),
            )
            .expect("read watermark");
        assert_eq!(value, "20");
    }

    #[test]
    fn event_type_filters_and_metrics_use_resolved_type_pairs() {
        let store = EventLogStore::open_in_memory().expect("open store");
        for (sequence, event_name) in [(1, Some("Created")), (2, Some("Updated")), (3, None)] {
            let mut projected = event(sequence);
            projected.owner_contract_id = event_name.map(|_| "test.events@v1".to_string());
            projected.owner_event_name = event_name.map(str::to_string);
            projected.resolution = if event_name.is_some() {
                "resolved"
            } else {
                "unresolved"
            }
            .to_string();
            projected.verification_status = if sequence == 2 {
                "invalid-signature"
            } else {
                "verified"
            }
            .to_string();
            store.insert_event(&projected).expect("insert event");
        }

        let event_type = |name: &str| EventTypeRef {
            owner_contract_id: "test.events@v1".to_string(),
            owner_event_name: name.to_string(),
        };
        let filter = EventLogFilter {
            include_event_types: vec![event_type("Created"), event_type("Updated")],
            exclude_event_types: vec![event_type("Updated")],
            limit: 100,
            ..EventLogFilter::default()
        };
        let (events, total) = store.query_events(&filter).expect("query included events");
        assert_eq!(total, 1);
        assert_eq!(events[0]["ownerEventName"], "Created");

        let filter = EventLogFilter {
            exclude_event_types: vec![event_type("Created")],
            limit: 100,
            ..EventLogFilter::default()
        };
        let (_, total) = store.query_events(&filter).expect("query excluded events");
        assert_eq!(
            total, 2,
            "excluding a resolved type keeps unresolved events"
        );

        let metrics = store.metrics(None).expect("query metrics");
        assert_eq!(
            metrics["summary"]["eventTypes"].as_array().map(Vec::len),
            Some(2)
        );

        let metrics = store
            .metrics(Some(("2025-12-31T23:00:00Z", 60 * 60, 5 * 60)))
            .expect("query bucketed metrics");
        let event_bucket = metrics["buckets"]
            .as_array()
            .and_then(|buckets| buckets.iter().find(|bucket| bucket["total"] == 3))
            .expect("find populated bucket");
        assert_eq!(metrics["buckets"].as_array().map(Vec::len), Some(13));
        assert_eq!(metrics["summary"]["integrityExceptions"], 2);
        assert_eq!(event_bucket["integrityExceptions"], 2);
        assert_eq!(event_bucket["byResolution"]["unresolved"], 1);
        assert_eq!(event_bucket["byVerificationStatus"]["invalid-signature"], 1);

        let filter = EventLogFilter {
            integrity_exception_only: true,
            limit: 100,
            ..EventLogFilter::default()
        };
        let (_, total) = store
            .query_events(&filter)
            .expect("query integrity exceptions");
        assert_eq!(total, 2);
    }
}
