//! Typed feed descriptors for `trellis.jobs@v1`.
use trellis_rs::generated::FeedDescriptor;
/// Descriptor for `Jobs.Watch`.
pub struct JobsWatchFeedDescriptor;
impl FeedDescriptor for JobsWatchFeedDescriptor {
    type Input = super::types::JobsWatchInput;
    type Event = super::types::JobsWatchEvent;
    const INPUT_SCHEMA_JSON: &'static str = super::schemas::JOBS_WATCH_INPUT_SCHEMA_JSON;
    const EVENT_SCHEMA_JSON: &'static str = super::schemas::JOBS_WATCH_EVENT_SCHEMA_JSON;
    const KEY: &'static str = "Jobs.Watch";
    const SUBJECT: &'static str = "feed.v1.Jobs.Watch";
    const SUBSCRIBE_CAPABILITIES: &'static [&'static str] = &["trellis.jobs::stream"];
}
