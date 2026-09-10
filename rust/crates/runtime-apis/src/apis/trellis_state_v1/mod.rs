//! Generated API `trellis.state@v1`.
pub const API_ID: &str = "trellis.state@v1";
pub const API_DIGEST: &str = "-uBNjrcPUFVtLKe8dzqBzmnF8jt9xcXjDtUAOhnCCZ0";
pub struct Api;
impl trellis_rs::generated::ApiDescriptor for Api {
    const ID: &'static str = API_ID;
}
pub mod errors {
    #[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
    pub struct AuthError {
        #[serde(flatten)]
        pub error: trellis_rs::generated::SerializableErrorData,
    }
    impl std::fmt::Display for AuthError {
        fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.write_str(&self.error.message)
        }
    }
    impl std::error::Error for AuthError {}
    impl trellis_rs::generated::TrellisError for AuthError {
        const TYPE: &'static str = "trellis.state@v1::AuthError";
    }
    #[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
    pub struct UnexpectedError {
        #[serde(flatten)]
        pub error: trellis_rs::generated::SerializableErrorData,
    }
    impl std::fmt::Display for UnexpectedError {
        fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.write_str(&self.error.message)
        }
    }
    impl std::error::Error for UnexpectedError {}
    impl trellis_rs::generated::TrellisError for UnexpectedError {
        const TYPE: &'static str = "trellis.state@v1::UnexpectedError";
    }
    #[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
    pub struct ValidationError {
        #[serde(flatten)]
        pub error: trellis_rs::generated::SerializableErrorData,
    }
    impl std::fmt::Display for ValidationError {
        fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.write_str(&self.error.message)
        }
    }
    impl std::error::Error for ValidationError {}
    impl trellis_rs::generated::TrellisError for ValidationError {
        const TYPE: &'static str = "trellis.state@v1::ValidationError";
    }
}
pub mod rpc {
    pub type AdminDeleteInput = crate::__types::trellis::StateAdminDeleteRequest;
    pub type AdminDeleteOutput = crate::__types::trellis::StateAdminDeleteResponse;
    pub struct AdminDelete;
    impl AdminDelete {
        pub const API_ID: &'static str = super::API_ID;
        pub const DESCRIPTOR_NAME: &'static str = "rpc.Admin.Delete";
        pub const KEY: &'static str = "state.Admin.Delete";
        pub const SUBJECT: &'static str = "rpc.v1.state.Admin.Delete";
        pub const CALLER_CAPABILITIES: &'static [&'static str] = &["trellis.state@v1::mutate"];
        pub const ERRORS: &'static [&'static str] = &[
            "trellis.state@v1::AuthError",
            "trellis.state@v1::UnexpectedError",
            "trellis.state@v1::ValidationError",
        ];
        pub const DOWNLOAD: bool = false;
        pub const CURSOR_PAGINATION: bool = false;
    }
    #[derive(Clone, Debug, PartialEq)]
    pub enum AdminDeleteError {
        AuthError(super::errors::AuthError),
        UnexpectedError(super::errors::UnexpectedError),
        ValidationError(super::errors::ValidationError),
    }
    impl AdminDeleteError {
        pub fn decode(value: serde_json::Value) -> Result<Option<Self>, serde_json::Error> {
            let error_type = value.get("type").and_then(serde_json::Value::as_str);
            match error_type {
                Some("trellis.state@v1::AuthError") => {
                    trellis_rs::generated::decode_typed_error::<super::errors::AuthError>(value)
                        .map(|value| value.map(Self::AuthError))
                }
                Some("trellis.state@v1::UnexpectedError") => {
                    trellis_rs::generated::decode_typed_error::<super::errors::UnexpectedError>(
                        value,
                    )
                    .map(|value| value.map(Self::UnexpectedError))
                }
                Some("trellis.state@v1::ValidationError") => {
                    trellis_rs::generated::decode_typed_error::<super::errors::ValidationError>(
                        value,
                    )
                    .map(|value| value.map(Self::ValidationError))
                }
                _ => Ok(None),
            }
        }
    }
    impl trellis_rs::generated::RpcDescriptor for AdminDelete {
        type Input = AdminDeleteInput;
        type Output = AdminDeleteOutput;
        type Error = AdminDeleteError;
        const API_ID: &'static str = super::API_ID;
        const DESCRIPTOR_NAME: &'static str = Self::DESCRIPTOR_NAME;
        const SUBJECT: &'static str = Self::SUBJECT;
        const KEY: &'static str = Self::KEY;
        const CALLER_CAPABILITIES: &'static [&'static str] = Self::CALLER_CAPABILITIES;
        fn decode_error(
            value: serde_json::Value,
        ) -> Result<Option<Self::Error>, serde_json::Error> {
            AdminDeleteError::decode(value)
        }
    }
    pub type AdminGetInput = crate::__types::trellis::StateAdminGetRequest;
    pub type AdminGetOutput = crate::__types::trellis::StateAdminGetResponse;
    pub struct AdminGet;
    impl AdminGet {
        pub const API_ID: &'static str = super::API_ID;
        pub const DESCRIPTOR_NAME: &'static str = "rpc.Admin.Get";
        pub const KEY: &'static str = "state.Admin.Get";
        pub const SUBJECT: &'static str = "rpc.v1.state.Admin.Get";
        pub const CALLER_CAPABILITIES: &'static [&'static str] = &["trellis.state@v1::read"];
        pub const ERRORS: &'static [&'static str] = &[
            "trellis.state@v1::AuthError",
            "trellis.state@v1::UnexpectedError",
            "trellis.state@v1::ValidationError",
        ];
        pub const DOWNLOAD: bool = false;
        pub const CURSOR_PAGINATION: bool = false;
    }
    #[derive(Clone, Debug, PartialEq)]
    pub enum AdminGetError {
        AuthError(super::errors::AuthError),
        UnexpectedError(super::errors::UnexpectedError),
        ValidationError(super::errors::ValidationError),
    }
    impl AdminGetError {
        pub fn decode(value: serde_json::Value) -> Result<Option<Self>, serde_json::Error> {
            let error_type = value.get("type").and_then(serde_json::Value::as_str);
            match error_type {
                Some("trellis.state@v1::AuthError") => {
                    trellis_rs::generated::decode_typed_error::<super::errors::AuthError>(value)
                        .map(|value| value.map(Self::AuthError))
                }
                Some("trellis.state@v1::UnexpectedError") => {
                    trellis_rs::generated::decode_typed_error::<super::errors::UnexpectedError>(
                        value,
                    )
                    .map(|value| value.map(Self::UnexpectedError))
                }
                Some("trellis.state@v1::ValidationError") => {
                    trellis_rs::generated::decode_typed_error::<super::errors::ValidationError>(
                        value,
                    )
                    .map(|value| value.map(Self::ValidationError))
                }
                _ => Ok(None),
            }
        }
    }
    impl trellis_rs::generated::RpcDescriptor for AdminGet {
        type Input = AdminGetInput;
        type Output = AdminGetOutput;
        type Error = AdminGetError;
        const API_ID: &'static str = super::API_ID;
        const DESCRIPTOR_NAME: &'static str = Self::DESCRIPTOR_NAME;
        const SUBJECT: &'static str = Self::SUBJECT;
        const KEY: &'static str = Self::KEY;
        const CALLER_CAPABILITIES: &'static [&'static str] = Self::CALLER_CAPABILITIES;
        fn decode_error(
            value: serde_json::Value,
        ) -> Result<Option<Self::Error>, serde_json::Error> {
            AdminGetError::decode(value)
        }
    }
    pub type AdminListInput = crate::__types::trellis::StateAdminListRequest;
    pub type AdminListOutput = crate::__types::trellis::StateAdminListResponse;
    pub struct AdminList;
    impl AdminList {
        pub const API_ID: &'static str = super::API_ID;
        pub const DESCRIPTOR_NAME: &'static str = "rpc.Admin.List";
        pub const KEY: &'static str = "state.Admin.List";
        pub const SUBJECT: &'static str = "rpc.v1.state.Admin.List";
        pub const CALLER_CAPABILITIES: &'static [&'static str] = &["trellis.state@v1::read"];
        pub const ERRORS: &'static [&'static str] = &[
            "trellis.state@v1::AuthError",
            "trellis.state@v1::UnexpectedError",
            "trellis.state@v1::ValidationError",
        ];
        pub const DOWNLOAD: bool = false;
        pub const CURSOR_PAGINATION: bool = false;
    }
    #[derive(Clone, Debug, PartialEq)]
    pub enum AdminListError {
        AuthError(super::errors::AuthError),
        UnexpectedError(super::errors::UnexpectedError),
        ValidationError(super::errors::ValidationError),
    }
    impl AdminListError {
        pub fn decode(value: serde_json::Value) -> Result<Option<Self>, serde_json::Error> {
            let error_type = value.get("type").and_then(serde_json::Value::as_str);
            match error_type {
                Some("trellis.state@v1::AuthError") => {
                    trellis_rs::generated::decode_typed_error::<super::errors::AuthError>(value)
                        .map(|value| value.map(Self::AuthError))
                }
                Some("trellis.state@v1::UnexpectedError") => {
                    trellis_rs::generated::decode_typed_error::<super::errors::UnexpectedError>(
                        value,
                    )
                    .map(|value| value.map(Self::UnexpectedError))
                }
                Some("trellis.state@v1::ValidationError") => {
                    trellis_rs::generated::decode_typed_error::<super::errors::ValidationError>(
                        value,
                    )
                    .map(|value| value.map(Self::ValidationError))
                }
                _ => Ok(None),
            }
        }
    }
    impl trellis_rs::generated::RpcDescriptor for AdminList {
        type Input = AdminListInput;
        type Output = AdminListOutput;
        type Error = AdminListError;
        const API_ID: &'static str = super::API_ID;
        const DESCRIPTOR_NAME: &'static str = Self::DESCRIPTOR_NAME;
        const SUBJECT: &'static str = Self::SUBJECT;
        const KEY: &'static str = Self::KEY;
        const CALLER_CAPABILITIES: &'static [&'static str] = Self::CALLER_CAPABILITIES;
        fn decode_error(
            value: serde_json::Value,
        ) -> Result<Option<Self::Error>, serde_json::Error> {
            AdminListError::decode(value)
        }
    }
    pub type DeleteInput = crate::__types::trellis::StateDeleteRequest;
    pub type DeleteOutput = crate::__types::trellis::StateDeleteResponse;
    pub struct Delete;
    impl Delete {
        pub const API_ID: &'static str = super::API_ID;
        pub const DESCRIPTOR_NAME: &'static str = "rpc.Delete";
        pub const KEY: &'static str = "state.Delete";
        pub const SUBJECT: &'static str = "rpc.v1.state.Delete";
        pub const CALLER_CAPABILITIES: &'static [&'static str] = &["trellis.state@v1::public"];
        pub const ERRORS: &'static [&'static str] = &[
            "trellis.state@v1::AuthError",
            "trellis.state@v1::UnexpectedError",
            "trellis.state@v1::ValidationError",
        ];
        pub const DOWNLOAD: bool = false;
        pub const CURSOR_PAGINATION: bool = false;
    }
    #[derive(Clone, Debug, PartialEq)]
    pub enum DeleteError {
        AuthError(super::errors::AuthError),
        UnexpectedError(super::errors::UnexpectedError),
        ValidationError(super::errors::ValidationError),
    }
    impl DeleteError {
        pub fn decode(value: serde_json::Value) -> Result<Option<Self>, serde_json::Error> {
            let error_type = value.get("type").and_then(serde_json::Value::as_str);
            match error_type {
                Some("trellis.state@v1::AuthError") => {
                    trellis_rs::generated::decode_typed_error::<super::errors::AuthError>(value)
                        .map(|value| value.map(Self::AuthError))
                }
                Some("trellis.state@v1::UnexpectedError") => {
                    trellis_rs::generated::decode_typed_error::<super::errors::UnexpectedError>(
                        value,
                    )
                    .map(|value| value.map(Self::UnexpectedError))
                }
                Some("trellis.state@v1::ValidationError") => {
                    trellis_rs::generated::decode_typed_error::<super::errors::ValidationError>(
                        value,
                    )
                    .map(|value| value.map(Self::ValidationError))
                }
                _ => Ok(None),
            }
        }
    }
    impl trellis_rs::generated::RpcDescriptor for Delete {
        type Input = DeleteInput;
        type Output = DeleteOutput;
        type Error = DeleteError;
        const API_ID: &'static str = super::API_ID;
        const DESCRIPTOR_NAME: &'static str = Self::DESCRIPTOR_NAME;
        const SUBJECT: &'static str = Self::SUBJECT;
        const KEY: &'static str = Self::KEY;
        const CALLER_CAPABILITIES: &'static [&'static str] = Self::CALLER_CAPABILITIES;
        fn decode_error(
            value: serde_json::Value,
        ) -> Result<Option<Self::Error>, serde_json::Error> {
            DeleteError::decode(value)
        }
    }
    pub type GetInput = crate::__types::trellis::StateGetRequest;
    pub type GetOutput = crate::__types::trellis::StateGetResponse;
    pub struct Get;
    impl Get {
        pub const API_ID: &'static str = super::API_ID;
        pub const DESCRIPTOR_NAME: &'static str = "rpc.Get";
        pub const KEY: &'static str = "state.Get";
        pub const SUBJECT: &'static str = "rpc.v1.state.Get";
        pub const CALLER_CAPABILITIES: &'static [&'static str] = &["trellis.state@v1::public"];
        pub const ERRORS: &'static [&'static str] = &[
            "trellis.state@v1::AuthError",
            "trellis.state@v1::UnexpectedError",
            "trellis.state@v1::ValidationError",
        ];
        pub const DOWNLOAD: bool = false;
        pub const CURSOR_PAGINATION: bool = false;
    }
    #[derive(Clone, Debug, PartialEq)]
    pub enum GetError {
        AuthError(super::errors::AuthError),
        UnexpectedError(super::errors::UnexpectedError),
        ValidationError(super::errors::ValidationError),
    }
    impl GetError {
        pub fn decode(value: serde_json::Value) -> Result<Option<Self>, serde_json::Error> {
            let error_type = value.get("type").and_then(serde_json::Value::as_str);
            match error_type {
                Some("trellis.state@v1::AuthError") => {
                    trellis_rs::generated::decode_typed_error::<super::errors::AuthError>(value)
                        .map(|value| value.map(Self::AuthError))
                }
                Some("trellis.state@v1::UnexpectedError") => {
                    trellis_rs::generated::decode_typed_error::<super::errors::UnexpectedError>(
                        value,
                    )
                    .map(|value| value.map(Self::UnexpectedError))
                }
                Some("trellis.state@v1::ValidationError") => {
                    trellis_rs::generated::decode_typed_error::<super::errors::ValidationError>(
                        value,
                    )
                    .map(|value| value.map(Self::ValidationError))
                }
                _ => Ok(None),
            }
        }
    }
    impl trellis_rs::generated::RpcDescriptor for Get {
        type Input = GetInput;
        type Output = GetOutput;
        type Error = GetError;
        const API_ID: &'static str = super::API_ID;
        const DESCRIPTOR_NAME: &'static str = Self::DESCRIPTOR_NAME;
        const SUBJECT: &'static str = Self::SUBJECT;
        const KEY: &'static str = Self::KEY;
        const CALLER_CAPABILITIES: &'static [&'static str] = Self::CALLER_CAPABILITIES;
        fn decode_error(
            value: serde_json::Value,
        ) -> Result<Option<Self::Error>, serde_json::Error> {
            GetError::decode(value)
        }
    }
    pub type ListInput = crate::__types::trellis::StateListRequest;
    pub type ListOutput = crate::__types::trellis::StateListResponse;
    pub struct List;
    impl List {
        pub const API_ID: &'static str = super::API_ID;
        pub const DESCRIPTOR_NAME: &'static str = "rpc.List";
        pub const KEY: &'static str = "state.List";
        pub const SUBJECT: &'static str = "rpc.v1.state.List";
        pub const CALLER_CAPABILITIES: &'static [&'static str] = &["trellis.state@v1::public"];
        pub const ERRORS: &'static [&'static str] = &[
            "trellis.state@v1::AuthError",
            "trellis.state@v1::UnexpectedError",
            "trellis.state@v1::ValidationError",
        ];
        pub const DOWNLOAD: bool = false;
        pub const CURSOR_PAGINATION: bool = false;
    }
    #[derive(Clone, Debug, PartialEq)]
    pub enum ListError {
        AuthError(super::errors::AuthError),
        UnexpectedError(super::errors::UnexpectedError),
        ValidationError(super::errors::ValidationError),
    }
    impl ListError {
        pub fn decode(value: serde_json::Value) -> Result<Option<Self>, serde_json::Error> {
            let error_type = value.get("type").and_then(serde_json::Value::as_str);
            match error_type {
                Some("trellis.state@v1::AuthError") => {
                    trellis_rs::generated::decode_typed_error::<super::errors::AuthError>(value)
                        .map(|value| value.map(Self::AuthError))
                }
                Some("trellis.state@v1::UnexpectedError") => {
                    trellis_rs::generated::decode_typed_error::<super::errors::UnexpectedError>(
                        value,
                    )
                    .map(|value| value.map(Self::UnexpectedError))
                }
                Some("trellis.state@v1::ValidationError") => {
                    trellis_rs::generated::decode_typed_error::<super::errors::ValidationError>(
                        value,
                    )
                    .map(|value| value.map(Self::ValidationError))
                }
                _ => Ok(None),
            }
        }
    }
    impl trellis_rs::generated::RpcDescriptor for List {
        type Input = ListInput;
        type Output = ListOutput;
        type Error = ListError;
        const API_ID: &'static str = super::API_ID;
        const DESCRIPTOR_NAME: &'static str = Self::DESCRIPTOR_NAME;
        const SUBJECT: &'static str = Self::SUBJECT;
        const KEY: &'static str = Self::KEY;
        const CALLER_CAPABILITIES: &'static [&'static str] = Self::CALLER_CAPABILITIES;
        fn decode_error(
            value: serde_json::Value,
        ) -> Result<Option<Self::Error>, serde_json::Error> {
            ListError::decode(value)
        }
    }
    pub type PutInput = crate::__types::trellis::StatePutRequest;
    pub type PutOutput = crate::__types::trellis::StatePutResponse;
    pub struct Put;
    impl Put {
        pub const API_ID: &'static str = super::API_ID;
        pub const DESCRIPTOR_NAME: &'static str = "rpc.Put";
        pub const KEY: &'static str = "state.Put";
        pub const SUBJECT: &'static str = "rpc.v1.state.Put";
        pub const CALLER_CAPABILITIES: &'static [&'static str] = &["trellis.state@v1::public"];
        pub const ERRORS: &'static [&'static str] = &[
            "trellis.state@v1::AuthError",
            "trellis.state@v1::UnexpectedError",
            "trellis.state@v1::ValidationError",
        ];
        pub const DOWNLOAD: bool = false;
        pub const CURSOR_PAGINATION: bool = false;
    }
    #[derive(Clone, Debug, PartialEq)]
    pub enum PutError {
        AuthError(super::errors::AuthError),
        UnexpectedError(super::errors::UnexpectedError),
        ValidationError(super::errors::ValidationError),
    }
    impl PutError {
        pub fn decode(value: serde_json::Value) -> Result<Option<Self>, serde_json::Error> {
            let error_type = value.get("type").and_then(serde_json::Value::as_str);
            match error_type {
                Some("trellis.state@v1::AuthError") => {
                    trellis_rs::generated::decode_typed_error::<super::errors::AuthError>(value)
                        .map(|value| value.map(Self::AuthError))
                }
                Some("trellis.state@v1::UnexpectedError") => {
                    trellis_rs::generated::decode_typed_error::<super::errors::UnexpectedError>(
                        value,
                    )
                    .map(|value| value.map(Self::UnexpectedError))
                }
                Some("trellis.state@v1::ValidationError") => {
                    trellis_rs::generated::decode_typed_error::<super::errors::ValidationError>(
                        value,
                    )
                    .map(|value| value.map(Self::ValidationError))
                }
                _ => Ok(None),
            }
        }
    }
    impl trellis_rs::generated::RpcDescriptor for Put {
        type Input = PutInput;
        type Output = PutOutput;
        type Error = PutError;
        const API_ID: &'static str = super::API_ID;
        const DESCRIPTOR_NAME: &'static str = Self::DESCRIPTOR_NAME;
        const SUBJECT: &'static str = Self::SUBJECT;
        const KEY: &'static str = Self::KEY;
        const CALLER_CAPABILITIES: &'static [&'static str] = Self::CALLER_CAPABILITIES;
        fn decode_error(
            value: serde_json::Value,
        ) -> Result<Option<Self::Error>, serde_json::Error> {
            PutError::decode(value)
        }
    }
}
pub mod operations {}
pub mod events {}
pub mod feeds {}
/// Registers metadata for every RPC in this API.
pub fn register_rpc_metadata(router: &mut trellis_rs::service::Router) {
    router.register_rpc_metadata::<rpc::AdminDelete>();
    router.register_rpc_metadata::<rpc::AdminGet>();
    router.register_rpc_metadata::<rpc::AdminList>();
    router.register_rpc_metadata::<rpc::Delete>();
    router.register_rpc_metadata::<rpc::Get>();
    router.register_rpc_metadata::<rpc::List>();
    router.register_rpc_metadata::<rpc::Put>();
}
#[derive(Clone)]
pub struct Client {
    inner: trellis_rs::generated::Client,
}
impl Client {
    pub fn from_generated(inner: trellis_rs::generated::Client) -> Self {
        Self { inner }
    }
    pub async fn admin_delete(
        &self,
        input: &rpc::AdminDeleteInput,
    ) -> Result<rpc::AdminDeleteOutput, trellis_rs::client::CallError<rpc::AdminDeleteError>> {
        self.inner.call::<rpc::AdminDelete>(input).await
    }
    pub async fn admin_get(
        &self,
        input: &rpc::AdminGetInput,
    ) -> Result<rpc::AdminGetOutput, trellis_rs::client::CallError<rpc::AdminGetError>> {
        self.inner.call::<rpc::AdminGet>(input).await
    }
    pub async fn admin_list(
        &self,
        input: &rpc::AdminListInput,
    ) -> Result<rpc::AdminListOutput, trellis_rs::client::CallError<rpc::AdminListError>> {
        self.inner.call::<rpc::AdminList>(input).await
    }
    pub async fn delete(
        &self,
        input: &rpc::DeleteInput,
    ) -> Result<rpc::DeleteOutput, trellis_rs::client::CallError<rpc::DeleteError>> {
        self.inner.call::<rpc::Delete>(input).await
    }
    pub async fn get(
        &self,
        input: &rpc::GetInput,
    ) -> Result<rpc::GetOutput, trellis_rs::client::CallError<rpc::GetError>> {
        self.inner.call::<rpc::Get>(input).await
    }
    pub async fn list(
        &self,
        input: &rpc::ListInput,
    ) -> Result<rpc::ListOutput, trellis_rs::client::CallError<rpc::ListError>> {
        self.inner.call::<rpc::List>(input).await
    }
    pub async fn put(
        &self,
        input: &rpc::PutInput,
    ) -> Result<rpc::PutOutput, trellis_rs::client::CallError<rpc::PutError>> {
        self.inner.call::<rpc::Put>(input).await
    }
}
pub struct Provider<'a, P> {
    runtime: &'a mut trellis_rs::service::ConnectedServiceRuntime<P>,
}
impl<'a, P: trellis_rs::generated::ParticipantDescriptor> Provider<'a, P> {
    pub fn new(runtime: &'a mut trellis_rs::service::ConnectedServiceRuntime<P>) -> Self {
        Self { runtime }
    }
    pub fn register_admin_delete<F, Fut>(&mut self, handler: F)
    where
        F: Fn(trellis_rs::service::ServiceHandlerContext, rpc::AdminDeleteInput) -> Fut
            + Send
            + Sync
            + 'static,
        Fut: std::future::Future<Output = trellis_rs::service::HandlerResult<rpc::AdminDeleteOutput>>
            + Send
            + 'static,
    {
        self.runtime.register_rpc::<rpc::AdminDelete, _, _>(handler);
    }
    pub fn register_admin_get<F, Fut>(&mut self, handler: F)
    where
        F: Fn(trellis_rs::service::ServiceHandlerContext, rpc::AdminGetInput) -> Fut
            + Send
            + Sync
            + 'static,
        Fut: std::future::Future<Output = trellis_rs::service::HandlerResult<rpc::AdminGetOutput>>
            + Send
            + 'static,
    {
        self.runtime.register_rpc::<rpc::AdminGet, _, _>(handler);
    }
    pub fn register_admin_list<F, Fut>(&mut self, handler: F)
    where
        F: Fn(trellis_rs::service::ServiceHandlerContext, rpc::AdminListInput) -> Fut
            + Send
            + Sync
            + 'static,
        Fut: std::future::Future<Output = trellis_rs::service::HandlerResult<rpc::AdminListOutput>>
            + Send
            + 'static,
    {
        self.runtime.register_rpc::<rpc::AdminList, _, _>(handler);
    }
    pub fn register_delete<F, Fut>(&mut self, handler: F)
    where
        F: Fn(trellis_rs::service::ServiceHandlerContext, rpc::DeleteInput) -> Fut
            + Send
            + Sync
            + 'static,
        Fut: std::future::Future<Output = trellis_rs::service::HandlerResult<rpc::DeleteOutput>>
            + Send
            + 'static,
    {
        self.runtime.register_rpc::<rpc::Delete, _, _>(handler);
    }
    pub fn register_get<F, Fut>(&mut self, handler: F)
    where
        F: Fn(trellis_rs::service::ServiceHandlerContext, rpc::GetInput) -> Fut
            + Send
            + Sync
            + 'static,
        Fut: std::future::Future<Output = trellis_rs::service::HandlerResult<rpc::GetOutput>>
            + Send
            + 'static,
    {
        self.runtime.register_rpc::<rpc::Get, _, _>(handler);
    }
    pub fn register_list<F, Fut>(&mut self, handler: F)
    where
        F: Fn(trellis_rs::service::ServiceHandlerContext, rpc::ListInput) -> Fut
            + Send
            + Sync
            + 'static,
        Fut: std::future::Future<Output = trellis_rs::service::HandlerResult<rpc::ListOutput>>
            + Send
            + 'static,
    {
        self.runtime.register_rpc::<rpc::List, _, _>(handler);
    }
    pub fn register_put<F, Fut>(&mut self, handler: F)
    where
        F: Fn(trellis_rs::service::ServiceHandlerContext, rpc::PutInput) -> Fut
            + Send
            + Sync
            + 'static,
        Fut: std::future::Future<Output = trellis_rs::service::HandlerResult<rpc::PutOutput>>
            + Send
            + 'static,
    {
        self.runtime.register_rpc::<rpc::Put, _, _>(handler);
    }
}
