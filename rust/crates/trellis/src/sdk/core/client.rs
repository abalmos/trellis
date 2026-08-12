//! Thin typed client helpers for `trellis.core@v1`.
/// Typed API wrapper for the `trellis.core@v1` contract.
pub struct CoreClient<'a> {
    inner: &'a crate::generated::Caller,
}
impl<'a> CoreClient<'a> {
    /// Wrap an already connected low-level Trellis client.
    pub fn new(inner: &'a crate::generated::Caller) -> Self {
        Self { inner }
    }
    #[allow(dead_code)]
    pub(crate) fn inner(&self) -> &'a crate::generated::Caller {
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
    pub(crate) _inner: &'a crate::generated::Caller,
}
impl<'a> Rpc<'a> {
    /// Access the `trellis` RPC group.
    pub fn trellis(&self) -> TrellisRpc<'a> {
        TrellisRpc { inner: self._inner }
    }
}
/// Typed RPC methods in the `trellis` group.
pub struct TrellisRpc<'a> {
    inner: &'a crate::generated::Caller,
}
impl<'a> TrellisRpc<'a> {
    /// Call `Trellis.Surface.Status`.
    pub async fn surface_status(
        &self,
        input: &super::types::TrellisSurfaceStatusRequest,
    ) -> Result<
        super::types::TrellisSurfaceStatusResponse,
        crate::generated::CallError<super::rpc::TrellisSurfaceStatusError>,
    > {
        self.inner
            .call_typed::<
                super::rpc::TrellisSurfaceStatusRpc,
                super::rpc::TrellisSurfaceStatusError,
            >(input)
            .await
    }
}
/// Typed event surface.
pub struct Event<'a> {
    pub(crate) _inner: &'a crate::generated::Caller,
}
impl<'a> Event<'a> {}
/// Typed feed surface.
pub struct Feed<'a> {
    pub(crate) _inner: &'a crate::generated::Caller,
}
impl<'a> Feed<'a> {}
/// Typed operation surface.
pub struct Operation<'a> {
    pub(crate) _inner: &'a crate::generated::Caller,
}
impl<'a> Operation<'a> {}
