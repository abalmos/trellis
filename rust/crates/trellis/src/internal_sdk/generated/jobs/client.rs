//! Thin typed client helpers for `trellis.jobs@v1`.
use trellis_rs::generated::TrellisClientError;
/// Typed API wrapper for the `trellis.jobs@v1` contract.
pub struct JobsClient<'a> {
    inner: &'a trellis_rs::generated::Caller,
}
impl<'a> JobsClient<'a> {
    /// Wrap an already connected low-level Trellis client.
    pub fn new(inner: &'a trellis_rs::generated::Caller) -> Self {
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
    pub(crate) _inner: &'a trellis_rs::generated::Caller,
}
impl<'a> Rpc<'a> {
    /// Access the `jobs` RPC group.
    pub fn jobs(&self) -> JobsRpc<'a> {
        JobsRpc { inner: self._inner }
    }
}
/// Typed RPC methods in the `jobs` group.
pub struct JobsRpc<'a> {
    inner: &'a trellis_rs::generated::Caller,
}
impl<'a> JobsRpc<'a> {
    /// Call `Jobs.Cancel`.
    pub async fn cancel(
        &self,
        input: &super::types::JobsCancelRequest,
    ) -> Result<
        super::types::JobsCancelResponse,
        trellis_rs::generated::CallError<super::rpc::JobsCancelError>,
    > {
        self.inner
            .call_typed::<super::rpc::JobsCancelRpc, super::rpc::JobsCancelError>(input)
            .await
    }
    /// Call `Jobs.DismissDLQ`.
    pub async fn dismiss_dlq(
        &self,
        input: &super::types::JobsDismissDLQRequest,
    ) -> Result<
        super::types::JobsDismissDLQResponse,
        trellis_rs::generated::CallError<super::rpc::JobsDismissDLQError>,
    > {
        self.inner
            .call_typed::<super::rpc::JobsDismissDLQRpc, super::rpc::JobsDismissDLQError>(input)
            .await
    }
    /// Call `Jobs.GetKey`.
    pub async fn get_key(
        &self,
        input: &super::types::JobsGetKeyRequest,
    ) -> Result<
        super::types::JobsGetKeyResponse,
        trellis_rs::generated::CallError<super::rpc::JobsGetKeyError>,
    > {
        self.inner
            .call_typed::<super::rpc::JobsGetKeyRpc, super::rpc::JobsGetKeyError>(input)
            .await
    }
    /// Call `Jobs.Inspect`.
    pub async fn inspect(
        &self,
        input: &super::types::JobsInspectRequest,
    ) -> Result<
        super::types::JobsInspectResponse,
        trellis_rs::generated::CallError<super::rpc::JobsInspectError>,
    > {
        self.inner
            .call_typed::<super::rpc::JobsInspectRpc, super::rpc::JobsInspectError>(input)
            .await
    }
    /// Call `Jobs.ListDLQ`.
    pub async fn list_dlq(
        &self,
        input: &super::types::JobsListDLQRequest,
    ) -> Result<
        super::types::JobsListDLQResponse,
        trellis_rs::generated::CallError<super::rpc::JobsListDLQError>,
    > {
        self.inner
            .call_typed::<super::rpc::JobsListDLQRpc, super::rpc::JobsListDLQError>(input)
            .await
    }
    /// Call `Jobs.ListServices`.
    pub async fn list_services(
        &self,
        input: &super::types::JobsListServicesRequest,
    ) -> Result<
        super::types::JobsListServicesResponse,
        trellis_rs::generated::CallError<super::rpc::JobsListServicesError>,
    > {
        self.inner
            .call_typed::<super::rpc::JobsListServicesRpc, super::rpc::JobsListServicesError>(input)
            .await
    }
    /// Call `Jobs.Metrics`.
    pub async fn metrics(
        &self,
        input: &super::types::JobsMetricsRequest,
    ) -> Result<
        super::types::JobsMetricsResponse,
        trellis_rs::generated::CallError<super::rpc::JobsMetricsError>,
    > {
        self.inner
            .call_typed::<super::rpc::JobsMetricsRpc, super::rpc::JobsMetricsError>(input)
            .await
    }
    /// Call `Jobs.Query`.
    pub async fn query(
        &self,
        input: &super::types::JobsQueryRequest,
    ) -> Result<
        super::types::JobsQueryResponse,
        trellis_rs::generated::CallError<super::rpc::JobsQueryError>,
    > {
        self.inner
            .call_typed::<super::rpc::JobsQueryRpc, super::rpc::JobsQueryError>(input)
            .await
    }
    /// Call `Jobs.ReplayDLQ`.
    pub async fn replay_dlq(
        &self,
        input: &super::types::JobsReplayDLQRequest,
    ) -> Result<
        super::types::JobsReplayDLQResponse,
        trellis_rs::generated::CallError<super::rpc::JobsReplayDLQError>,
    > {
        self.inner
            .call_typed::<super::rpc::JobsReplayDLQRpc, super::rpc::JobsReplayDLQError>(input)
            .await
    }
    /// Call `Jobs.Retry`.
    pub async fn retry(
        &self,
        input: &super::types::JobsRetryRequest,
    ) -> Result<
        super::types::JobsRetryResponse,
        trellis_rs::generated::CallError<super::rpc::JobsRetryError>,
    > {
        self.inner
            .call_typed::<super::rpc::JobsRetryRpc, super::rpc::JobsRetryError>(input)
            .await
    }
}
/// Typed event surface.
pub struct Event<'a> {
    pub(crate) _inner: &'a trellis_rs::generated::Caller,
}
impl<'a> Event<'a> {}
/// Typed feed surface.
pub struct Feed<'a> {
    pub(crate) _inner: &'a trellis_rs::generated::Caller,
}
impl<'a> Feed<'a> {
    /// Access the `jobs` feed group.
    pub fn jobs(&self) -> JobsFeed<'a> {
        JobsFeed { inner: self._inner }
    }
}
/// Typed feeds in the `jobs` group.
pub struct JobsFeed<'a> {
    inner: &'a trellis_rs::generated::Caller,
}
impl<'a> JobsFeed<'a> {
    /// Subscribe to `Jobs.Watch`.
    pub async fn watch(
        &self,
        input: &super::types::JobsWatchInput,
    ) -> Result<
        futures_util::stream::BoxStream<
            'static,
            Result<super::types::JobsWatchEvent, TrellisClientError>,
        >,
        TrellisClientError,
    > {
        self.inner
            .feed::<super::feeds::JobsWatchFeedDescriptor>(input)
            .await
    }
}
/// Typed operation surface.
pub struct Operation<'a> {
    pub(crate) _inner: &'a trellis_rs::generated::Caller,
}
impl<'a> Operation<'a> {}
