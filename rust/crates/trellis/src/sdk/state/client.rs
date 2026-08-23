//! Thin typed client helpers for `trellis.state@v1`.
/// Typed API wrapper for the `trellis.state@v1` contract.
pub struct StateClient<'a> {
    inner: &'a crate::generated::Caller,
}
impl<'a> StateClient<'a> {
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
    /// Access the `state` RPC group.
    pub fn state(&self) -> StateRpc<'a> {
        StateRpc { inner: self._inner }
    }
}
/// Typed RPC methods in the `state` group.
pub struct StateRpc<'a> {
    inner: &'a crate::generated::Caller,
}
impl<'a> StateRpc<'a> {
    /// Call `State.Admin.Delete`.
    pub async fn admin_delete(
        &self,
        input: &super::types::StateAdminDeleteRequest,
    ) -> Result<
        super::types::StateAdminDeleteResponse,
        crate::generated::CallError<super::rpc::StateAdminDeleteError>,
    > {
        self.inner
            .call_typed::<super::rpc::StateAdminDeleteRpc, super::rpc::StateAdminDeleteError>(input)
            .await
    }
    /// Call `State.Admin.Get`.
    pub async fn admin_get(
        &self,
        input: &super::types::StateAdminGetRequest,
    ) -> Result<
        super::types::StateAdminGetResponse,
        crate::generated::CallError<super::rpc::StateAdminGetError>,
    > {
        self.inner
            .call_typed::<super::rpc::StateAdminGetRpc, super::rpc::StateAdminGetError>(input)
            .await
    }
    /// Call `State.Admin.List`.
    pub async fn admin_list(
        &self,
        input: &super::types::StateAdminListRequest,
    ) -> Result<
        super::types::StateAdminListResponse,
        crate::generated::CallError<super::rpc::StateAdminListError>,
    > {
        self.inner
            .call_typed::<super::rpc::StateAdminListRpc, super::rpc::StateAdminListError>(input)
            .await
    }
    /// Call `State.Delete`.
    pub async fn delete(
        &self,
        input: &super::types::StateDeleteRequest,
    ) -> Result<
        super::types::StateDeleteResponse,
        crate::generated::CallError<super::rpc::StateDeleteError>,
    > {
        self.inner
            .call_typed::<super::rpc::StateDeleteRpc, super::rpc::StateDeleteError>(input)
            .await
    }
    /// Call `State.Get`.
    pub async fn get(
        &self,
        input: &super::types::StateGetRequest,
    ) -> Result<
        super::types::StateGetResponse,
        crate::generated::CallError<super::rpc::StateGetError>,
    > {
        self.inner
            .call_typed::<super::rpc::StateGetRpc, super::rpc::StateGetError>(input)
            .await
    }
    /// Call `State.List`.
    pub async fn list(
        &self,
        input: &super::types::StateListRequest,
    ) -> Result<
        super::types::StateListResponse,
        crate::generated::CallError<super::rpc::StateListError>,
    > {
        self.inner
            .call_typed::<super::rpc::StateListRpc, super::rpc::StateListError>(input)
            .await
    }
    /// Call `State.Put`.
    pub async fn put(
        &self,
        input: &super::types::StatePutRequest,
    ) -> Result<
        super::types::StatePutResponse,
        crate::generated::CallError<super::rpc::StatePutError>,
    > {
        self.inner
            .call_typed::<super::rpc::StatePutRpc, super::rpc::StatePutError>(input)
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
