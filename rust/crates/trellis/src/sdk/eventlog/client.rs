//! Thin typed client helpers for `trellis.eventlog@v1`.
use crate::client::TrellisClientError;
/// Typed API wrapper for the `trellis.eventlog@v1` contract.
pub struct EventlogClient<'a> {
    inner: &'a crate::client::TrellisClient,
}
impl<'a> EventlogClient<'a> {
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
    pub fn event_log(&self) -> EventLogRpc<'a> {
        EventLogRpc { inner: self._inner }
    }
}
pub struct EventLogRpc<'a> {
    inner: &'a crate::client::TrellisClient,
}
impl<'a> EventLogRpc<'a> {
    /// Call `EventLog.Consumers.Inspect`.
    pub async fn consumers_inspect(
        &self,
        input: &super::types::EventLogConsumersInspectRequest,
    ) -> Result<super::rpc::Empty, TrellisClientError> {
        self.inner
            .call::<super::rpc::EventLogConsumersInspectRpc>(input)
            .await
    }
    /// Call `EventLog.Consumers.Query`.
    pub async fn consumers_query(
        &self,
        input: &super::types::EventLogConsumersQueryRequest,
    ) -> Result<super::types::EventLogConsumersQueryResponse, TrellisClientError> {
        self.inner
            .call::<super::rpc::EventLogConsumersQueryRpc>(input)
            .await
    }
    /// Call `EventLog.Inspect`.
    pub async fn inspect(
        &self,
        input: &super::types::EventLogInspectRequest,
    ) -> Result<super::rpc::Empty, TrellisClientError> {
        self.inner
            .call::<super::rpc::EventLogInspectRpc>(input)
            .await
    }
    /// Call `EventLog.Metrics`.
    pub async fn metrics(
        &self,
        input: &super::types::EventLogMetricsRequest,
    ) -> Result<super::types::EventLogMetricsResponse, TrellisClientError> {
        self.inner
            .call::<super::rpc::EventLogMetricsRpc>(input)
            .await
    }
    /// Call `EventLog.Query`.
    pub async fn query(
        &self,
        input: &super::types::EventLogQueryRequest,
    ) -> Result<super::types::EventLogQueryResponse, TrellisClientError> {
        self.inner.call::<super::rpc::EventLogQueryRpc>(input).await
    }
}
/// Typed event surface.
pub struct Event<'a> {
    pub(crate) _inner: &'a crate::client::TrellisClient,
}
impl<'a> Event<'a> {}
/// Typed feed surface.
pub struct Feed<'a> {
    pub(crate) _inner: &'a crate::client::TrellisClient,
}
impl<'a> Feed<'a> {
    pub fn event_log(&self) -> EventLogFeed<'a> {
        EventLogFeed { inner: self._inner }
    }
}
pub struct EventLogFeed<'a> {
    inner: &'a crate::client::TrellisClient,
}
impl<'a> EventLogFeed<'a> {
    /// Subscribe to `EventLog.Watch`.
    pub async fn watch(
        &self,
    ) -> Result<
        futures_util::stream::BoxStream<
            'static,
            Result<super::types::EventLogWatchEvent, TrellisClientError>,
        >,
        TrellisClientError,
    > {
        self.inner
            .feed::<super::feeds::EventLogWatchFeedDescriptor>(&super::rpc::Empty {})
            .await
    }
}
/// Typed operation surface.
pub struct Operation<'a> {
    pub(crate) _inner: &'a crate::client::TrellisClient,
}
impl<'a> Operation<'a> {}
