use std::collections::BTreeMap;
use std::sync::Arc;

use serde_json::{json, Value};
use trellis_rs::client::TrellisClient;
use trellis_rs::sdk::auth::rpc::AuthEventConsumersListRpc;
use trellis_rs::sdk::auth::types::{
    AuthEventConsumersListRequest, AuthEventConsumersListResponseEntriesItem,
};

use crate::projector::EventLogRuntime;

pub(crate) async fn query_consumers(
    runtime: &EventLogRuntime,
    auth_client: &Arc<TrellisClient>,
    input: &Value,
) -> Result<Value, String> {
    let offset = input.get("offset").and_then(Value::as_u64).unwrap_or(0) as usize;
    let limit = input
        .get("limit")
        .and_then(Value::as_u64)
        .unwrap_or(50)
        .clamp(1, 500) as usize;
    let statuses = input
        .get("status")
        .and_then(Value::as_array)
        .map(|values| values.iter().filter_map(Value::as_str).collect::<Vec<_>>())
        .unwrap_or_default();
    let subject = input.get("subject").and_then(Value::as_str);
    let deployment_id = input.get("deploymentId").and_then(Value::as_str);
    let expected = expected_consumers(auth_client).await?;
    let mut live = runtime
        .consumers()
        .await?
        .into_iter()
        .map(|info| (info.name.clone(), info))
        .collect::<BTreeMap<_, _>>();
    let mut rows = Vec::new();
    for consumer in expected.into_iter().filter(|consumer| {
        deployment_id.is_none_or(|deployment_id| consumer.deployment_id == deployment_id)
            && subject.is_none_or(|subject| {
                consumer.filter_subjects.iter().any(|filter| {
                    filter == subject || subject.starts_with(filter.trim_end_matches('>'))
                })
            })
    }) {
        let live_info = live.remove(&consumer.consumer_name);
        rows.push(expected_consumer_row(&consumer, live_info));
    }
    rows.extend(
        live.into_values()
            .map(|info| consumer_row(info, Some("orphaned"))),
    );
    let mut rows = rows
        .into_iter()
        .filter(|row| {
            statuses.is_empty()
                || row
                    .get("status")
                    .and_then(Value::as_str)
                    .is_some_and(|status| statuses.contains(&status))
        })
        .collect::<Vec<_>>();
    rows.sort_by_key(|row| {
        row.get("consumerName")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string()
    });
    let total = rows.len();
    let consumers = rows
        .into_iter()
        .skip(offset)
        .take(limit)
        .collect::<Vec<_>>();
    Ok(json!({ "consumers": consumers, "total": total, "offset": offset, "limit": limit }))
}

pub(crate) async fn inspect_consumer(
    runtime: &EventLogRuntime,
    auth_client: &Arc<TrellisClient>,
    input: &Value,
) -> Result<Value, String> {
    let name = input
        .get("consumerName")
        .or_else(|| input.get("name"))
        .and_then(Value::as_str)
        .ok_or_else(|| "consumerName is required".to_string())?;
    let expected = expected_consumers(auth_client)
        .await?
        .into_iter()
        .find(|consumer| consumer.consumer_name == name);
    let info = runtime.consumer(name).await.ok();
    if expected.is_none() && info.is_none() {
        return Err(format!("consumer not found: {name}"));
    }
    let live = info.as_ref().map(|info| {
        json!({
            "stream": info.stream_name,
            "consumerName": info.name,
            "pending": info.num_pending,
            "ackPending": info.num_ack_pending,
            "waitingPulls": info.num_waiting,
            "redelivered": info.num_redelivered,
            "ackWaitMs": info.config.ack_wait.as_millis() as u64,
            "maxDeliver": info.config.max_deliver,
            "filterSubject": info.config.filter_subject,
        })
    });
    let consumer = if let Some(expected) = expected.as_ref() {
        expected_consumer_row(expected, info)
    } else {
        consumer_row(info.expect("checked above"), Some("orphaned"))
    };
    Ok(json!({
        "consumer": consumer,
        "expected": expected.as_ref().map(expected_consumer_value),
        "live": live,
        "recentEvents": []
    }))
}

async fn expected_consumers(
    auth_client: &Arc<TrellisClient>,
) -> Result<Vec<AuthEventConsumersListResponseEntriesItem>, String> {
    let mut offset = Some(0);
    let mut entries = Vec::new();
    while let Some(current_offset) = offset {
        let response = auth_client
            .call::<AuthEventConsumersListRpc>(&AuthEventConsumersListRequest {
                deployment_id: None,
                limit: 500,
                offset: Some(current_offset),
            })
            .await
            .map_err(|error| error.to_string())?;
        entries.extend(response.entries);
        offset = response.next_offset;
    }
    Ok(entries)
}

fn consumer_status(info: &async_nats::jetstream::consumer::Info) -> &'static str {
    let pending = info.num_pending;
    let ack_pending = info.num_ack_pending as u64;
    let waiting = info.num_waiting as u64;
    let redelivered = info.num_redelivered as u64;
    let max_ack_pending = info.config.max_ack_pending.max(0) as u64;
    if redelivered > 0 {
        "failing"
    } else if pending == 0 && ack_pending == 0 {
        "current"
    } else if pending > 0 && max_ack_pending > 0 && ack_pending >= max_ack_pending {
        "saturated"
    } else if pending > 0 && waiting == 0 && ack_pending == 0 {
        "inactive"
    } else if pending > 0 {
        "behind"
    } else {
        "processing"
    }
}

fn consumer_row(info: async_nats::jetstream::consumer::Info, status: Option<&str>) -> Value {
    let status = status.unwrap_or_else(|| consumer_status(&info));
    let filter_subjects = if info.config.filter_subject.is_empty() {
        Vec::<String>::new()
    } else {
        vec![info.config.filter_subject.clone()]
    };
    json!({
        "stream": info.stream_name,
        "consumerName": info.name,
        "filterSubjects": filter_subjects,
        "status": status,
        "pending": info.num_pending,
        "ackPending": info.num_ack_pending as u64,
        "waitingPulls": info.num_waiting as u64,
        "redelivered": info.num_redelivered as u64,
        "ackWaitMs": info.config.ack_wait.as_millis() as u64,
        "maxDeliver": info.config.max_deliver,
    })
}

fn expected_consumer_value(consumer: &AuthEventConsumersListResponseEntriesItem) -> Value {
    json!({
        "deploymentId": consumer.deployment_id,
        "group": consumer.group,
        "stream": consumer.stream,
        "consumerName": consumer.consumer_name,
        "filterSubjects": consumer.filter_subjects,
        "replay": consumer.replay,
        "ordering": consumer.ordering,
        "concurrency": consumer.concurrency,
        "ackWaitMs": consumer.ack_wait_ms,
        "maxDeliver": consumer.max_deliver,
        "backoffMs": consumer.backoff_ms,
    })
}

fn expected_consumer_row(
    consumer: &AuthEventConsumersListResponseEntriesItem,
    live: Option<async_nats::jetstream::consumer::Info>,
) -> Value {
    let status = live.as_ref().map(consumer_status).unwrap_or("missing");
    json!({
        "deploymentId": consumer.deployment_id,
        "group": consumer.group,
        "stream": consumer.stream,
        "consumerName": consumer.consumer_name,
        "filterSubjects": consumer.filter_subjects,
        "status": status,
        "pending": live.as_ref().map(|info| info.num_pending).unwrap_or(0),
        "ackPending": live.as_ref().map(|info| info.num_ack_pending as u64).unwrap_or(0),
        "waitingPulls": live.as_ref().map(|info| info.num_waiting as u64).unwrap_or(0),
        "redelivered": live.as_ref().map(|info| info.num_redelivered as u64).unwrap_or(0),
        "concurrency": consumer.concurrency,
        "ackWaitMs": consumer.ack_wait_ms,
        "maxDeliver": consumer.max_deliver,
    })
}
