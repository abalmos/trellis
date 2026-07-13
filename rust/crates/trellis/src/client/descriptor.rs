use serde::{de::DeserializeOwned, Serialize};

use super::subject::{resolve_subject, SubjectError};

/// Metadata required to call one typed Trellis RPC.
pub trait RpcDescriptor {
    /// Request payload type.
    type Input: Serialize;

    /// Success payload type.
    type Output: DeserializeOwned;

    /// Logical contract key for the RPC.
    const KEY: &'static str;

    /// Concrete NATS subject for the RPC.
    const SUBJECT: &'static str;

    /// Capability requirements declared for callers.
    const CALLER_CAPABILITIES: &'static [&'static str];

    /// Known error variants declared by the contract.
    const ERRORS: &'static [&'static str];

    /// JSON Schema for the input type.
    const INPUT_SCHEMA_JSON: &'static str;

    /// JSON Schema for the output type.
    const OUTPUT_SCHEMA_JSON: &'static str;
}

/// Metadata required to publish one typed Trellis event.
pub trait EventDescriptor {
    /// Event payload type.
    type Event: Serialize + DeserializeOwned;

    /// Logical contract key for the event.
    const KEY: &'static str;

    /// Canonical NATS subject template for the event.
    const SUBJECT: &'static str;

    /// NATS wildcard subject used to subscribe to every concrete event subject.
    const SUBSCRIBE_SUBJECT: &'static str = Self::SUBJECT;

    /// JSON Schema for the event payload.
    const EVENT_SCHEMA_JSON: &'static str = "{}";

    /// Capability requirements declared for publishers.
    const PUBLISH_CAPABILITIES: &'static [&'static str];

    /// Whether the contract explicitly permits publication by dependencies.
    const DELEGATED_PUBLISH: bool = false;

    /// Capability requirements declared for subscribers.
    const SUBSCRIBE_CAPABILITIES: &'static [&'static str];

    /// Resolve the concrete publish subject from the typed event payload.
    fn publish_subject(event: &Self::Event) -> Result<String, SubjectError> {
        let value = serde_json::to_value(event)?;
        resolve_subject(Self::SUBJECT, &value)
    }
}

/// Metadata required to subscribe to one typed Trellis feed.
pub trait FeedDescriptor {
    /// Feed subscription input type.
    type Input: Serialize;

    /// Feed event payload type.
    type Event: DeserializeOwned;

    /// Logical contract key for the feed.
    const KEY: &'static str;

    /// Concrete NATS subject for the feed.
    const SUBJECT: &'static str;

    /// Capability requirements declared for subscribers.
    const SUBSCRIBE_CAPABILITIES: &'static [&'static str];

    /// JSON Schema for the input type.
    const INPUT_SCHEMA_JSON: &'static str;

    /// JSON Schema for the event type.
    const EVENT_SCHEMA_JSON: &'static str;
}
