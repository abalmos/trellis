//! Thin typed client helpers for `trellis.health@v1`.
use crate::client::TrellisClientError;
/// Typed API wrapper for the `trellis.health@v1` contract.
pub struct HealthClient<'a> {
    inner: &'a crate::client::TrellisClient,
}
impl<'a> HealthClient<'a> {
    /// Wrap an already connected low-level Trellis client.
    pub fn new(inner: &'a crate::client::TrellisClient) -> Self {
        Self { inner }
    }
    #[allow(dead_code)]
    pub(crate) fn inner(&self) -> &'a crate::client::TrellisClient {
        self.inner
    }
    /// Access typed RPC calls.
    pub fn rpc(&self) -> Rpc<'a> {
        Rpc { _inner: self.inner }
    }
    /// Access typed events.
    pub fn event(&self) -> Event<'a> {
        Event { _inner: self.inner }
    }
    /// Access typed feeds.
    pub fn feed(&self) -> Feed<'a> {
        Feed { _inner: self.inner }
    }
    /// Access typed operations.
    pub fn operation(&self) -> Operation<'a> {
        Operation { _inner: self.inner }
    }
}
/// Typed RPC surface.
pub struct Rpc<'a> {
    pub(crate) _inner: &'a crate::client::TrellisClient,
}
impl<'a> Rpc<'a> {
    pub fn health(&self) -> HealthRpc<'a> {
        HealthRpc { inner: self._inner }
    }
}
pub struct HealthRpc<'a> {
    inner: &'a crate::client::TrellisClient,
}
impl<'a> HealthRpc<'a> {
    /// Call `Health.Inspect`.
    pub async fn inspect(
        &self,
        input: &super::types::HealthInspectRequest,
    ) -> Result<super::types::HealthInspectResponse, TrellisClientError> {
        self.inner.call::<super::rpc::HealthInspectRpc>(input).await
    }
    /// Call `Health.Metrics`.
    pub async fn metrics(
        &self,
        input: &super::types::HealthMetricsRequest,
    ) -> Result<super::types::HealthMetricsResponse, TrellisClientError> {
        self.inner.call::<super::rpc::HealthMetricsRpc>(input).await
    }
    /// Call `Health.Query`.
    pub async fn query(
        &self,
        input: &super::types::HealthQueryRequest,
    ) -> Result<super::types::HealthQueryResponse, TrellisClientError> {
        self.inner.call::<super::rpc::HealthQueryRpc>(input).await
    }
}
/// Typed event surface.
pub struct Event<'a> {
    pub(crate) _inner: &'a crate::client::TrellisClient,
}
impl<'a> Event<'a> {
    pub fn health(&self) -> HealthEvent<'a> {
        HealthEvent { inner: self._inner }
    }
}
pub struct HealthEvent<'a> {
    inner: &'a crate::client::TrellisClient,
}
impl<'a> HealthEvent<'a> {
    pub fn status_changed(&self) -> HealthStatusChangedEvent<'a> {
        HealthStatusChangedEvent { inner: self.inner }
    }
}
pub struct HealthStatusChangedEvent<'a> {
    inner: &'a crate::client::TrellisClient,
}
impl<'a> HealthStatusChangedEvent<'a> {
    pub async fn publish(
        &self,
        event: &super::types::HealthStatusChangedEvent,
    ) -> Result<(), TrellisClientError> {
        self.inner
            .publish::<super::events::HealthStatusChangedEventDescriptor>(event)
            .await
    }
    pub async fn listen<F, Fut>(&self, handler: F) -> Result<(), TrellisClientError>
    where
        F: Fn(super::types::HealthStatusChangedEvent) -> Fut,
        Fut: std::future::Future<Output = Result<(), TrellisClientError>>,
    {
        let mut stream = self
            .inner
            .subscribe_with_options::<super::events::HealthStatusChangedEventDescriptor>(
                crate::client::EventSubscribeOptions {
                    stream: None,
                    mode: crate::client::EventSubscriptionMode::Ephemeral,
                    replay: crate::client::EventReplayPolicy::New,
                    durable_name: None,
                },
            )
            .await?;
        while let Some(event) = futures_util::StreamExt::next(&mut stream).await {
            handler(event?).await?;
        }
        Ok(())
    }
}
/// Typed feed surface.
pub struct Feed<'a> {
    pub(crate) _inner: &'a crate::client::TrellisClient,
}
impl<'a> Feed<'a> {
    pub fn health(&self) -> HealthFeed<'a> {
        HealthFeed { inner: self._inner }
    }
}
pub struct HealthFeed<'a> {
    inner: &'a crate::client::TrellisClient,
}
impl<'a> HealthFeed<'a> {
    /// Subscribe to `Health.Watch`.
    pub async fn watch(
        &self,
        input: &super::types::HealthWatchInput,
    ) -> Result<
        futures_util::stream::BoxStream<
            'static,
            Result<super::types::HealthWatchEvent, TrellisClientError>,
        >,
        TrellisClientError,
    > {
        self.inner
            .feed::<super::feeds::HealthWatchFeedDescriptor>(input)
            .await
    }
}
/// Typed operation surface.
pub struct Operation<'a> {
    pub(crate) _inner: &'a crate::client::TrellisClient,
}
impl<'a> Operation<'a> {}
