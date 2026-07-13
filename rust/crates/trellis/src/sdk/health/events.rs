//! Typed event descriptors for `trellis.health@v1`.
use crate::generated::EventDescriptor;
/// Descriptor for `Health.StatusChanged`.
pub struct HealthStatusChangedEventDescriptor;
impl EventDescriptor for HealthStatusChangedEventDescriptor {
    type Event = super::types::HealthStatusChangedEvent;
    const KEY: &'static str = "Health.StatusChanged";
    const SUBJECT: &'static str = "events.v1.Health.StatusChanged";
    const SUBSCRIBE_SUBJECT: &'static str = "events.v1.Health.StatusChanged";
    const EVENT_SCHEMA_JSON: &'static str = super::schemas::HEALTH_STATUS_CHANGED_EVENT_SCHEMA_JSON;
    const PUBLISH_CAPABILITIES: &'static [&'static str] = &[];
    const DELEGATED_PUBLISH: bool = false;
    const SUBSCRIBE_CAPABILITIES: &'static [&'static str] = &["trellis.health::read"];
}
