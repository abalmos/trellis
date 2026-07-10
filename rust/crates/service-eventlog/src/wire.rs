use serde_json::Value;
use trellis_rs::service::{FeedDescriptor, RpcDescriptor};

const ANY_SCHEMA: &str = "{}";
const READ_CAPABILITY: &[&str] = &["trellis.eventlog::events.read"];
const STREAM_CAPABILITY: &[&str] = &["trellis.eventlog::events.stream"];

macro_rules! rpc_descriptor {
    ($name:ident, $key:literal, $subject:literal) => {
        #[derive(Debug, Clone, Copy)]
        pub(crate) struct $name;

        impl RpcDescriptor for $name {
            type Input = Value;
            type Output = Value;

            const KEY: &'static str = $key;
            const SUBJECT: &'static str = $subject;
            const CALLER_CAPABILITIES: &'static [&'static str] = READ_CAPABILITY;
            const INPUT_SCHEMA_JSON: &'static str = ANY_SCHEMA;
            const OUTPUT_SCHEMA_JSON: &'static str = ANY_SCHEMA;
        }
    };
}

rpc_descriptor!(EventLogQueryRpc, "EventLog.Query", "rpc.v1.EventLog.Query");
rpc_descriptor!(
    EventLogInspectRpc,
    "EventLog.Inspect",
    "rpc.v1.EventLog.Inspect"
);
rpc_descriptor!(
    EventLogMetricsRpc,
    "EventLog.Metrics",
    "rpc.v1.EventLog.Metrics"
);
rpc_descriptor!(
    EventLogConsumersQueryRpc,
    "EventLog.Consumers.Query",
    "rpc.v1.EventLog.Consumers.Query"
);
rpc_descriptor!(
    EventLogConsumersInspectRpc,
    "EventLog.Consumers.Inspect",
    "rpc.v1.EventLog.Consumers.Inspect"
);

#[derive(Debug, Clone, Copy)]
pub(crate) struct EventLogWatchFeed;

impl FeedDescriptor for EventLogWatchFeed {
    type Input = Value;
    type Event = Value;

    const KEY: &'static str = "EventLog.Watch";
    const SUBJECT: &'static str = "feeds.v1.EventLog.Watch";
    const SUBSCRIBE_CAPABILITIES: &'static [&'static str] = STREAM_CAPABILITY;
    const INPUT_SCHEMA_JSON: &'static str = ANY_SCHEMA;
    const EVENT_SCHEMA_JSON: &'static str = ANY_SCHEMA;
}
