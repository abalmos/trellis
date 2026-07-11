//! Typed feed descriptors for `trellis.health@v1`.
use crate::client::FeedDescriptor;
/// Descriptor for `Health.Watch`.
pub struct HealthWatchFeedDescriptor;
impl FeedDescriptor for HealthWatchFeedDescriptor {
    type Input = super::types::HealthWatchInput;
    type Event = super::types::HealthWatchEvent;
    const INPUT_SCHEMA_JSON: &'static str = super::schemas::HEALTH_WATCH_INPUT_SCHEMA_JSON;
    const EVENT_SCHEMA_JSON: &'static str = super::schemas::HEALTH_WATCH_EVENT_SCHEMA_JSON;
    const KEY: &'static str = "Health.Watch";
    const SUBJECT: &'static str = "feed.v1.Health.Watch";
    const SUBSCRIBE_CAPABILITIES: &'static [&'static str] = &["trellis.health::read"];
}
