//! Generated API `trellis.core@v1`.
pub const API_ID: &str = "trellis.core@v1";
pub const API_DIGEST: &str = "fXmSRF8ZanEn015Qk-M3WvUGzQ-7kguhGdItKunXQyo";
pub struct Api;
impl trellis_rs::generated::ApiDescriptor for Api {
    const ID: &'static str = API_ID;
}
pub mod errors {
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
        const TYPE: &'static str = "trellis.core@v1::UnexpectedError";
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
        const TYPE: &'static str = "trellis.core@v1::ValidationError";
    }
}
pub mod rpc {
    pub type SurfaceStatusInput = crate::__types::trellis::TrellisSurfaceStatusRequest;
    pub type SurfaceStatusOutput = crate::__types::trellis::TrellisSurfaceStatusResponse;
    pub struct SurfaceStatus;
    impl SurfaceStatus {
        pub const API_ID: &'static str = super::API_ID;
        pub const DESCRIPTOR_NAME: &'static str = "rpc.Surface.Status";
        pub const KEY: &'static str = "core.Surface.Status";
        pub const SUBJECT: &'static str = "rpc.v1.core.Surface.Status";
        pub const CALLER_CAPABILITIES: &'static [&'static str] =
            &["trellis.core@v1::authority_read"];
        pub const ERRORS: &'static [&'static str] = &[
            "trellis.core@v1::UnexpectedError",
            "trellis.core@v1::ValidationError",
        ];
        pub const DOWNLOAD: bool = false;
        pub const CURSOR_PAGINATION: bool = false;
    }
    #[derive(Clone, Debug, PartialEq)]
    pub enum SurfaceStatusError {
        UnexpectedError(super::errors::UnexpectedError),
        ValidationError(super::errors::ValidationError),
    }
    impl SurfaceStatusError {
        pub fn decode(value: serde_json::Value) -> Result<Option<Self>, serde_json::Error> {
            let error_type = value.get("type").and_then(serde_json::Value::as_str);
            match error_type {
                Some("trellis.core@v1::UnexpectedError") => {
                    trellis_rs::generated::decode_typed_error::<super::errors::UnexpectedError>(
                        value,
                    )
                    .map(|value| value.map(Self::UnexpectedError))
                }
                Some("trellis.core@v1::ValidationError") => {
                    trellis_rs::generated::decode_typed_error::<super::errors::ValidationError>(
                        value,
                    )
                    .map(|value| value.map(Self::ValidationError))
                }
                _ => Ok(None),
            }
        }
    }
    impl trellis_rs::generated::RpcDescriptor for SurfaceStatus {
        type Input = SurfaceStatusInput;
        type Output = SurfaceStatusOutput;
        type Error = SurfaceStatusError;
        const API_ID: &'static str = super::API_ID;
        const DESCRIPTOR_NAME: &'static str = Self::DESCRIPTOR_NAME;
        const SUBJECT: &'static str = Self::SUBJECT;
        const KEY: &'static str = Self::KEY;
        const CALLER_CAPABILITIES: &'static [&'static str] = Self::CALLER_CAPABILITIES;
        fn decode_error(
            value: serde_json::Value,
        ) -> Result<Option<Self::Error>, serde_json::Error> {
            SurfaceStatusError::decode(value)
        }
    }
}
pub mod operations {}
pub mod events {}
pub mod feeds {}
/// Registers metadata for every RPC in this API.
pub fn register_rpc_metadata(router: &mut trellis_rs::service::Router) {
    router.register_rpc_metadata::<rpc::SurfaceStatus>();
}
#[derive(Clone)]
pub struct Client {
    inner: trellis_rs::generated::Client,
}
impl Client {
    pub fn from_generated(inner: trellis_rs::generated::Client) -> Self {
        Self { inner }
    }
    pub async fn surface_status(
        &self,
        input: &rpc::SurfaceStatusInput,
    ) -> Result<rpc::SurfaceStatusOutput, trellis_rs::client::CallError<rpc::SurfaceStatusError>>
    {
        self.inner.call::<rpc::SurfaceStatus>(input).await
    }
}
pub struct Provider<'a, P> {
    runtime: &'a mut trellis_rs::service::ConnectedServiceRuntime<P>,
}
impl<'a, P: trellis_rs::generated::ParticipantDescriptor> Provider<'a, P> {
    pub fn new(runtime: &'a mut trellis_rs::service::ConnectedServiceRuntime<P>) -> Self {
        Self { runtime }
    }
    pub fn register_surface_status<F, Fut>(&mut self, handler: F)
    where
        F: Fn(trellis_rs::service::ServiceHandlerContext, rpc::SurfaceStatusInput) -> Fut
            + Send
            + Sync
            + 'static,
        Fut: std::future::Future<
                Output = trellis_rs::service::HandlerResult<rpc::SurfaceStatusOutput>,
            > + Send
            + 'static,
    {
        self.runtime
            .register_rpc::<rpc::SurfaceStatus, _, _>(handler);
    }
}
