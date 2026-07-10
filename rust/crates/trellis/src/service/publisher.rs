use std::sync::Arc;

use super::{EventDescriptor, ServerError};
use crate::client::{PreparedTrellisEvent, TrellisClient};

/// A descriptor-backed event publisher using the connected Trellis client.
#[derive(Clone)]
pub struct EventPublisher {
    client: Arc<TrellisClient>,
}

impl EventPublisher {
    pub(crate) fn new(client: Arc<TrellisClient>) -> Self {
        Self { client }
    }

    /// Publish one descriptor-backed event.
    pub async fn publish<D>(&self, event: &D::Event) -> Result<(), ServerError>
    where
        D: EventDescriptor,
    {
        let prepared =
            PreparedTrellisEvent::new(D::SUBJECT, bytes::Bytes::from(serde_json::to_vec(event)?));
        self.client
            .publish_prepared(&prepared)
            .await
            .map_err(|error| ServerError::Nats(error.to_string()))?;
        Ok(())
    }
}
