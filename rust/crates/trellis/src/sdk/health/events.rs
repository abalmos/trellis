//! Typed event descriptors for `trellis.health@v1`.
use crate::client::EventDescriptor;
/// Descriptor for `Health.StatusChanged`.
pub struct HealthStatusChangedEventDescriptor;
impl EventDescriptor for HealthStatusChangedEventDescriptor {
    type Event = super::types::HealthStatusChangedEvent;
    const KEY: &'static str = "Health.StatusChanged";
    const SUBJECT: &'static str = "events.v1.Health.StatusChanged";
    const PUBLISH_CAPABILITIES: &'static [&'static str] = &[];
    const SUBSCRIBE_CAPABILITIES: &'static [&'static str] = &["trellis.health::read"];
}
