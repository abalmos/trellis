//! Typed feed descriptors for `trellis.eventlog@v1`.
use trellis_rs::generated::FeedDescriptor;
/// Descriptor for `EventLog.Watch`.
pub struct EventLogWatchFeedDescriptor;
impl FeedDescriptor for EventLogWatchFeedDescriptor {
    type Input = super::rpc::Empty;
    type Event = super::types::EventLogWatchEvent;
    const INPUT_SCHEMA_JSON: &'static str = super::schemas::EVENT_LOG_WATCH_INPUT_SCHEMA_JSON;
    const EVENT_SCHEMA_JSON: &'static str = super::schemas::EVENT_LOG_WATCH_EVENT_SCHEMA_JSON;
    const KEY: &'static str = "EventLog.Watch";
    const SUBJECT: &'static str = "feed.v1.EventLog.Watch";
    const SUBSCRIBE_CAPABILITIES: &'static [&'static str] = &["trellis.eventlog::stream"];
}
