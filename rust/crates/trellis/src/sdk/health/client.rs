//! Thin typed client helpers for `trellis.health@v1`.
use crate::generated::TrellisClientError;
/// Typed API wrapper for the `trellis.health@v1` contract.
pub struct HealthClient<'a> {
    inner: &'a crate::generated::Caller,
}
impl<'a> HealthClient<'a> {
    /// Wrap an already connected low-level Trellis client.
    pub fn new(inner: &'a crate::generated::Caller) -> Self {
        Self { inner }
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
    pub(crate) _inner: &'a crate::generated::Caller,
}
impl<'a> Rpc<'a> {
    /// Access the `health` RPC group.
    pub fn health(&self) -> HealthRpc<'a> {
        HealthRpc { inner: self._inner }
    }
}
/// Typed RPC methods in the `health` group.
pub struct HealthRpc<'a> {
    inner: &'a crate::generated::Caller,
}
impl<'a> HealthRpc<'a> {
    /// Call `Health.Inspect`.
    pub async fn inspect(
        &self,
        input: &super::types::HealthInspectRequest,
    ) -> Result<
        super::types::HealthInspectResponse,
        crate::generated::CallError<super::rpc::HealthInspectError>,
    > {
        self.inner
            .call_typed::<super::rpc::HealthInspectRpc, super::rpc::HealthInspectError>(input)
            .await
    }
    /// Call `Health.Metrics`.
    pub async fn metrics(
        &self,
        input: &super::types::HealthMetricsRequest,
    ) -> Result<
        super::types::HealthMetricsResponse,
        crate::generated::CallError<super::rpc::HealthMetricsError>,
    > {
        self.inner
            .call_typed::<super::rpc::HealthMetricsRpc, super::rpc::HealthMetricsError>(input)
            .await
    }
    /// Call `Health.Query`.
    pub async fn query(
        &self,
        input: &super::types::HealthQueryRequest,
    ) -> Result<
        super::types::HealthQueryResponse,
        crate::generated::CallError<super::rpc::HealthQueryError>,
    > {
        self.inner
            .call_typed::<super::rpc::HealthQueryRpc, super::rpc::HealthQueryError>(input)
            .await
    }
}
/// Typed event surface.
pub struct Event<'a> {
    pub(crate) _inner: &'a crate::generated::Caller,
}
impl<'a> Event<'a> {
    /// Access the `health` event group.
    pub fn health(&self) -> HealthEvent<'a> {
        HealthEvent { inner: self._inner }
    }
}
/// Typed events in the `health` group.
pub struct HealthEvent<'a> {
    inner: &'a crate::generated::Caller,
}
impl<'a> HealthEvent<'a> {
    /// Access `Health.StatusChanged`.
    pub fn status_changed(&self) -> HealthStatusChangedEvent<'a> {
        HealthStatusChangedEvent { inner: self.inner }
    }
}
/// Typed `Health.StatusChanged` event operations.
pub struct HealthStatusChangedEvent<'a> {
    inner: &'a crate::generated::Caller,
}
impl<'a> HealthStatusChangedEvent<'a> {
    /// Publish `Health.StatusChanged`.
    pub async fn publish(
        &self,
        event: &super::types::HealthStatusChangedEvent,
    ) -> Result<(), TrellisClientError> {
        self.inner
            .publish::<super::events::HealthStatusChangedEventDescriptor>(event)
            .await
    }
    /// Listen for live `Health.StatusChanged` events.
    pub async fn listen<F, Fut>(&self, handler: F) -> Result<(), TrellisClientError>
    where
        F: Fn(super::types::HealthStatusChangedEvent) -> Fut,
        Fut: std::future::Future<Output = Result<(), TrellisClientError>>,
    {
        let mut stream = self
            .inner
            .subscribe::<super::events::HealthStatusChangedEventDescriptor>()
            .await?;
        while let Some(event) = futures_util::StreamExt::next(&mut stream).await {
            handler(event?).await?;
        }
        Ok(())
    }
}
/// Typed feed surface.
pub struct Feed<'a> {
    pub(crate) _inner: &'a crate::generated::Caller,
}
impl<'a> Feed<'a> {
    /// Access the `health` feed group.
    pub fn health(&self) -> HealthFeed<'a> {
        HealthFeed { inner: self._inner }
    }
}
/// Typed feeds in the `health` group.
pub struct HealthFeed<'a> {
    inner: &'a crate::generated::Caller,
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
    pub(crate) _inner: &'a crate::generated::Caller,
}
impl<'a> Operation<'a> {}
