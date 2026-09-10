//! Generated API `trellis.events@v1`.
pub const API_ID: &str = "trellis.events@v1";
pub const API_DIGEST: &str = "oJlY-7SaNj7LSVf14gq3H5DWYUovxBhfkVHa0ysIr1Y";
pub struct Api;
impl trellis_rs::generated::ApiDescriptor for Api {
    const ID: &'static str = API_ID;
}
pub mod errors {
    #[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
    pub struct NotFoundError {
        #[serde(flatten)]
        pub error: trellis_rs::generated::SerializableErrorData,
    }
    impl NotFoundError {
        pub fn payload(
            &self,
        ) -> Result<crate::__types::trellis::EventsNotFoundErrorData, serde_json::Error> {
            serde_json::from_value(serde_json::Value::Object(self.error.extra.clone()))
        }
    }
    impl std::fmt::Display for NotFoundError {
        fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.write_str(&self.error.message)
        }
    }
    impl std::error::Error for NotFoundError {}
    impl trellis_rs::generated::TrellisError for NotFoundError {
        const TYPE: &'static str = "trellis.events@v1::NotFoundError";
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
        const TYPE: &'static str = "trellis.events@v1::UnexpectedError";
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
        const TYPE: &'static str = "trellis.events@v1::ValidationError";
    }
}
pub mod rpc {
    pub type ConsumersInspectInput = crate::__types::trellis::EventsConsumersInspectRequest;
    pub type ConsumersInspectOutput = crate::__types::trellis::EventsConsumersInspectResponse;
    pub struct ConsumersInspect;
    impl ConsumersInspect {
        pub const API_ID: &'static str = super::API_ID;
        pub const DESCRIPTOR_NAME: &'static str = "rpc.Consumers.Inspect";
        pub const KEY: &'static str = "events.Consumers.Inspect";
        pub const SUBJECT: &'static str = "rpc.v1.events.Consumers.Inspect";
        pub const CALLER_CAPABILITIES: &'static [&'static str] = &["trellis.events@v1::read"];
        pub const ERRORS: &'static [&'static str] = &[
            "trellis.events@v1::NotFoundError",
            "trellis.events@v1::UnexpectedError",
            "trellis.events@v1::ValidationError",
        ];
        pub const DOWNLOAD: bool = false;
        pub const CURSOR_PAGINATION: bool = false;
    }
    #[derive(Clone, Debug, PartialEq)]
    pub enum ConsumersInspectError {
        NotFoundError(super::errors::NotFoundError),
        UnexpectedError(super::errors::UnexpectedError),
        ValidationError(super::errors::ValidationError),
    }
    impl ConsumersInspectError {
        pub fn decode(value: serde_json::Value) -> Result<Option<Self>, serde_json::Error> {
            let error_type = value.get("type").and_then(serde_json::Value::as_str);
            match error_type {
                Some("trellis.events@v1::NotFoundError") => {
                    trellis_rs::generated::decode_typed_error::<super::errors::NotFoundError>(value)
                        .map(|value| value.map(Self::NotFoundError))
                }
                Some("trellis.events@v1::UnexpectedError") => {
                    trellis_rs::generated::decode_typed_error::<super::errors::UnexpectedError>(
                        value,
                    )
                    .map(|value| value.map(Self::UnexpectedError))
                }
                Some("trellis.events@v1::ValidationError") => {
                    trellis_rs::generated::decode_typed_error::<super::errors::ValidationError>(
                        value,
                    )
                    .map(|value| value.map(Self::ValidationError))
                }
                _ => Ok(None),
            }
        }
    }
    impl trellis_rs::generated::RpcDescriptor for ConsumersInspect {
        type Input = ConsumersInspectInput;
        type Output = ConsumersInspectOutput;
        type Error = ConsumersInspectError;
        const API_ID: &'static str = super::API_ID;
        const DESCRIPTOR_NAME: &'static str = Self::DESCRIPTOR_NAME;
        const SUBJECT: &'static str = Self::SUBJECT;
        const KEY: &'static str = Self::KEY;
        const CALLER_CAPABILITIES: &'static [&'static str] = Self::CALLER_CAPABILITIES;
        fn decode_error(
            value: serde_json::Value,
        ) -> Result<Option<Self::Error>, serde_json::Error> {
            ConsumersInspectError::decode(value)
        }
    }
    pub type ConsumersQueryInput = crate::__types::trellis::EventsConsumersQueryRequest;
    pub type ConsumersQueryOutput = crate::__types::trellis::EventsConsumersQueryResponse;
    pub struct ConsumersQuery;
    impl ConsumersQuery {
        pub const API_ID: &'static str = super::API_ID;
        pub const DESCRIPTOR_NAME: &'static str = "rpc.Consumers.Query";
        pub const KEY: &'static str = "events.Consumers.Query";
        pub const SUBJECT: &'static str = "rpc.v1.events.Consumers.Query";
        pub const CALLER_CAPABILITIES: &'static [&'static str] = &["trellis.events@v1::read"];
        pub const ERRORS: &'static [&'static str] = &[
            "trellis.events@v1::UnexpectedError",
            "trellis.events@v1::ValidationError",
        ];
        pub const DOWNLOAD: bool = false;
        pub const CURSOR_PAGINATION: bool = false;
    }
    #[derive(Clone, Debug, PartialEq)]
    pub enum ConsumersQueryError {
        UnexpectedError(super::errors::UnexpectedError),
        ValidationError(super::errors::ValidationError),
    }
    impl ConsumersQueryError {
        pub fn decode(value: serde_json::Value) -> Result<Option<Self>, serde_json::Error> {
            let error_type = value.get("type").and_then(serde_json::Value::as_str);
            match error_type {
                Some("trellis.events@v1::UnexpectedError") => {
                    trellis_rs::generated::decode_typed_error::<super::errors::UnexpectedError>(
                        value,
                    )
                    .map(|value| value.map(Self::UnexpectedError))
                }
                Some("trellis.events@v1::ValidationError") => {
                    trellis_rs::generated::decode_typed_error::<super::errors::ValidationError>(
                        value,
                    )
                    .map(|value| value.map(Self::ValidationError))
                }
                _ => Ok(None),
            }
        }
    }
    impl trellis_rs::generated::RpcDescriptor for ConsumersQuery {
        type Input = ConsumersQueryInput;
        type Output = ConsumersQueryOutput;
        type Error = ConsumersQueryError;
        const API_ID: &'static str = super::API_ID;
        const DESCRIPTOR_NAME: &'static str = Self::DESCRIPTOR_NAME;
        const SUBJECT: &'static str = Self::SUBJECT;
        const KEY: &'static str = Self::KEY;
        const CALLER_CAPABILITIES: &'static [&'static str] = Self::CALLER_CAPABILITIES;
        fn decode_error(
            value: serde_json::Value,
        ) -> Result<Option<Self::Error>, serde_json::Error> {
            ConsumersQueryError::decode(value)
        }
    }
    pub type InspectInput = crate::__types::trellis::EventsInspectRequest;
    pub type InspectOutput = crate::__types::trellis::EventsInspectResponse;
    pub struct Inspect;
    impl Inspect {
        pub const API_ID: &'static str = super::API_ID;
        pub const DESCRIPTOR_NAME: &'static str = "rpc.Inspect";
        pub const KEY: &'static str = "events.Inspect";
        pub const SUBJECT: &'static str = "rpc.v1.events.Inspect";
        pub const CALLER_CAPABILITIES: &'static [&'static str] = &["trellis.events@v1::read"];
        pub const ERRORS: &'static [&'static str] = &[
            "trellis.events@v1::NotFoundError",
            "trellis.events@v1::UnexpectedError",
            "trellis.events@v1::ValidationError",
        ];
        pub const DOWNLOAD: bool = false;
        pub const CURSOR_PAGINATION: bool = false;
    }
    #[derive(Clone, Debug, PartialEq)]
    pub enum InspectError {
        NotFoundError(super::errors::NotFoundError),
        UnexpectedError(super::errors::UnexpectedError),
        ValidationError(super::errors::ValidationError),
    }
    impl InspectError {
        pub fn decode(value: serde_json::Value) -> Result<Option<Self>, serde_json::Error> {
            let error_type = value.get("type").and_then(serde_json::Value::as_str);
            match error_type {
                Some("trellis.events@v1::NotFoundError") => {
                    trellis_rs::generated::decode_typed_error::<super::errors::NotFoundError>(value)
                        .map(|value| value.map(Self::NotFoundError))
                }
                Some("trellis.events@v1::UnexpectedError") => {
                    trellis_rs::generated::decode_typed_error::<super::errors::UnexpectedError>(
                        value,
                    )
                    .map(|value| value.map(Self::UnexpectedError))
                }
                Some("trellis.events@v1::ValidationError") => {
                    trellis_rs::generated::decode_typed_error::<super::errors::ValidationError>(
                        value,
                    )
                    .map(|value| value.map(Self::ValidationError))
                }
                _ => Ok(None),
            }
        }
    }
    impl trellis_rs::generated::RpcDescriptor for Inspect {
        type Input = InspectInput;
        type Output = InspectOutput;
        type Error = InspectError;
        const API_ID: &'static str = super::API_ID;
        const DESCRIPTOR_NAME: &'static str = Self::DESCRIPTOR_NAME;
        const SUBJECT: &'static str = Self::SUBJECT;
        const KEY: &'static str = Self::KEY;
        const CALLER_CAPABILITIES: &'static [&'static str] = Self::CALLER_CAPABILITIES;
        fn decode_error(
            value: serde_json::Value,
        ) -> Result<Option<Self::Error>, serde_json::Error> {
            InspectError::decode(value)
        }
    }
    pub type MetricsInput = crate::__types::trellis::EventsMetricsRequest;
    pub type MetricsOutput = crate::__types::trellis::EventsMetricsResponse;
    pub struct Metrics;
    impl Metrics {
        pub const API_ID: &'static str = super::API_ID;
        pub const DESCRIPTOR_NAME: &'static str = "rpc.Metrics";
        pub const KEY: &'static str = "events.Metrics";
        pub const SUBJECT: &'static str = "rpc.v1.events.Metrics";
        pub const CALLER_CAPABILITIES: &'static [&'static str] = &["trellis.events@v1::read"];
        pub const ERRORS: &'static [&'static str] = &[
            "trellis.events@v1::UnexpectedError",
            "trellis.events@v1::ValidationError",
        ];
        pub const DOWNLOAD: bool = false;
        pub const CURSOR_PAGINATION: bool = false;
    }
    #[derive(Clone, Debug, PartialEq)]
    pub enum MetricsError {
        UnexpectedError(super::errors::UnexpectedError),
        ValidationError(super::errors::ValidationError),
    }
    impl MetricsError {
        pub fn decode(value: serde_json::Value) -> Result<Option<Self>, serde_json::Error> {
            let error_type = value.get("type").and_then(serde_json::Value::as_str);
            match error_type {
                Some("trellis.events@v1::UnexpectedError") => {
                    trellis_rs::generated::decode_typed_error::<super::errors::UnexpectedError>(
                        value,
                    )
                    .map(|value| value.map(Self::UnexpectedError))
                }
                Some("trellis.events@v1::ValidationError") => {
                    trellis_rs::generated::decode_typed_error::<super::errors::ValidationError>(
                        value,
                    )
                    .map(|value| value.map(Self::ValidationError))
                }
                _ => Ok(None),
            }
        }
    }
    impl trellis_rs::generated::RpcDescriptor for Metrics {
        type Input = MetricsInput;
        type Output = MetricsOutput;
        type Error = MetricsError;
        const API_ID: &'static str = super::API_ID;
        const DESCRIPTOR_NAME: &'static str = Self::DESCRIPTOR_NAME;
        const SUBJECT: &'static str = Self::SUBJECT;
        const KEY: &'static str = Self::KEY;
        const CALLER_CAPABILITIES: &'static [&'static str] = Self::CALLER_CAPABILITIES;
        fn decode_error(
            value: serde_json::Value,
        ) -> Result<Option<Self::Error>, serde_json::Error> {
            MetricsError::decode(value)
        }
    }
    pub type QueryInput = crate::__types::trellis::EventsQueryRequest;
    pub type QueryOutput = crate::__types::trellis::EventsQueryResponse;
    pub struct Query;
    impl Query {
        pub const API_ID: &'static str = super::API_ID;
        pub const DESCRIPTOR_NAME: &'static str = "rpc.Query";
        pub const KEY: &'static str = "events.Query";
        pub const SUBJECT: &'static str = "rpc.v1.events.Query";
        pub const CALLER_CAPABILITIES: &'static [&'static str] = &["trellis.events@v1::read"];
        pub const ERRORS: &'static [&'static str] = &[
            "trellis.events@v1::UnexpectedError",
            "trellis.events@v1::ValidationError",
        ];
        pub const DOWNLOAD: bool = false;
        pub const CURSOR_PAGINATION: bool = false;
    }
    #[derive(Clone, Debug, PartialEq)]
    pub enum QueryError {
        UnexpectedError(super::errors::UnexpectedError),
        ValidationError(super::errors::ValidationError),
    }
    impl QueryError {
        pub fn decode(value: serde_json::Value) -> Result<Option<Self>, serde_json::Error> {
            let error_type = value.get("type").and_then(serde_json::Value::as_str);
            match error_type {
                Some("trellis.events@v1::UnexpectedError") => {
                    trellis_rs::generated::decode_typed_error::<super::errors::UnexpectedError>(
                        value,
                    )
                    .map(|value| value.map(Self::UnexpectedError))
                }
                Some("trellis.events@v1::ValidationError") => {
                    trellis_rs::generated::decode_typed_error::<super::errors::ValidationError>(
                        value,
                    )
                    .map(|value| value.map(Self::ValidationError))
                }
                _ => Ok(None),
            }
        }
    }
    impl trellis_rs::generated::RpcDescriptor for Query {
        type Input = QueryInput;
        type Output = QueryOutput;
        type Error = QueryError;
        const API_ID: &'static str = super::API_ID;
        const DESCRIPTOR_NAME: &'static str = Self::DESCRIPTOR_NAME;
        const SUBJECT: &'static str = Self::SUBJECT;
        const KEY: &'static str = Self::KEY;
        const CALLER_CAPABILITIES: &'static [&'static str] = Self::CALLER_CAPABILITIES;
        fn decode_error(
            value: serde_json::Value,
        ) -> Result<Option<Self::Error>, serde_json::Error> {
            QueryError::decode(value)
        }
    }
}
pub mod operations {}
pub mod events {}
pub mod feeds {
    pub type WatchInput = crate::__types::trellis::EventsWatchRequest;
    pub type WatchEvent = crate::__types::trellis::EventsWatchFrame;
    pub struct Watch;
    impl Watch {
        pub const API_ID: &'static str = super::API_ID;
        pub const DESCRIPTOR_NAME: &'static str = "feed.Watch";
        pub const KEY: &'static str = "events.Watch";
        pub const SUBJECT: &'static str = "feed.v1.events.Watch";
        pub const SUBSCRIBE_CAPABILITIES: &'static [&'static str] = &["trellis.events@v1::stream"];
    }
    impl trellis_rs::generated::FeedDescriptor for Watch {
        type Input = WatchInput;
        type Event = WatchEvent;
        const API_ID: &'static str = super::API_ID;
        const DESCRIPTOR_NAME: &'static str = Self::DESCRIPTOR_NAME;
        const SUBJECT: &'static str = Self::SUBJECT;
        const KEY: &'static str = Self::KEY;
        const SUBSCRIBE_CAPABILITIES: &'static [&'static str] = Self::SUBSCRIBE_CAPABILITIES;
    }
}
/// Registers metadata for every RPC in this API.
pub fn register_rpc_metadata(router: &mut trellis_rs::service::Router) {
    router.register_rpc_metadata::<rpc::ConsumersInspect>();
    router.register_rpc_metadata::<rpc::ConsumersQuery>();
    router.register_rpc_metadata::<rpc::Inspect>();
    router.register_rpc_metadata::<rpc::Metrics>();
    router.register_rpc_metadata::<rpc::Query>();
}
#[derive(Clone)]
pub struct Client {
    inner: trellis_rs::generated::Client,
}
impl Client {
    pub fn from_generated(inner: trellis_rs::generated::Client) -> Self {
        Self { inner }
    }
    pub async fn consumers_inspect(
        &self,
        input: &rpc::ConsumersInspectInput,
    ) -> Result<
        rpc::ConsumersInspectOutput,
        trellis_rs::client::CallError<rpc::ConsumersInspectError>,
    > {
        self.inner.call::<rpc::ConsumersInspect>(input).await
    }
    pub async fn consumers_query(
        &self,
        input: &rpc::ConsumersQueryInput,
    ) -> Result<rpc::ConsumersQueryOutput, trellis_rs::client::CallError<rpc::ConsumersQueryError>>
    {
        self.inner.call::<rpc::ConsumersQuery>(input).await
    }
    pub async fn inspect(
        &self,
        input: &rpc::InspectInput,
    ) -> Result<rpc::InspectOutput, trellis_rs::client::CallError<rpc::InspectError>> {
        self.inner.call::<rpc::Inspect>(input).await
    }
    pub async fn metrics(
        &self,
        input: &rpc::MetricsInput,
    ) -> Result<rpc::MetricsOutput, trellis_rs::client::CallError<rpc::MetricsError>> {
        self.inner.call::<rpc::Metrics>(input).await
    }
    pub async fn query(
        &self,
        input: &rpc::QueryInput,
    ) -> Result<rpc::QueryOutput, trellis_rs::client::CallError<rpc::QueryError>> {
        self.inner.call::<rpc::Query>(input).await
    }
    pub async fn watch(
        &self,
        input: &feeds::WatchInput,
    ) -> Result<
        futures_util::stream::BoxStream<
            'static,
            Result<feeds::WatchEvent, trellis_rs::client::TrellisClientError>,
        >,
        trellis_rs::client::TrellisClientError,
    > {
        self.inner.feed::<feeds::Watch>(input).await
    }
}
pub struct Provider<'a, P> {
    runtime: &'a mut trellis_rs::service::ConnectedServiceRuntime<P>,
}
impl<'a, P: trellis_rs::generated::ParticipantDescriptor> Provider<'a, P> {
    pub fn new(runtime: &'a mut trellis_rs::service::ConnectedServiceRuntime<P>) -> Self {
        Self { runtime }
    }
    pub fn register_consumers_inspect<F, Fut>(&mut self, handler: F)
    where
        F: Fn(trellis_rs::service::ServiceHandlerContext, rpc::ConsumersInspectInput) -> Fut
            + Send
            + Sync
            + 'static,
        Fut: std::future::Future<
                Output = trellis_rs::service::HandlerResult<rpc::ConsumersInspectOutput>,
            > + Send
            + 'static,
    {
        self.runtime
            .register_rpc::<rpc::ConsumersInspect, _, _>(handler);
    }
    pub fn register_consumers_query<F, Fut>(&mut self, handler: F)
    where
        F: Fn(trellis_rs::service::ServiceHandlerContext, rpc::ConsumersQueryInput) -> Fut
            + Send
            + Sync
            + 'static,
        Fut: std::future::Future<
                Output = trellis_rs::service::HandlerResult<rpc::ConsumersQueryOutput>,
            > + Send
            + 'static,
    {
        self.runtime
            .register_rpc::<rpc::ConsumersQuery, _, _>(handler);
    }
    pub fn register_inspect<F, Fut>(&mut self, handler: F)
    where
        F: Fn(trellis_rs::service::ServiceHandlerContext, rpc::InspectInput) -> Fut
            + Send
            + Sync
            + 'static,
        Fut: std::future::Future<Output = trellis_rs::service::HandlerResult<rpc::InspectOutput>>
            + Send
            + 'static,
    {
        self.runtime.register_rpc::<rpc::Inspect, _, _>(handler);
    }
    pub fn register_metrics<F, Fut>(&mut self, handler: F)
    where
        F: Fn(trellis_rs::service::ServiceHandlerContext, rpc::MetricsInput) -> Fut
            + Send
            + Sync
            + 'static,
        Fut: std::future::Future<Output = trellis_rs::service::HandlerResult<rpc::MetricsOutput>>
            + Send
            + 'static,
    {
        self.runtime.register_rpc::<rpc::Metrics, _, _>(handler);
    }
    pub fn register_query<F, Fut>(&mut self, handler: F)
    where
        F: Fn(trellis_rs::service::ServiceHandlerContext, rpc::QueryInput) -> Fut
            + Send
            + Sync
            + 'static,
        Fut: std::future::Future<Output = trellis_rs::service::HandlerResult<rpc::QueryOutput>>
            + Send
            + 'static,
    {
        self.runtime.register_rpc::<rpc::Query, _, _>(handler);
    }
    pub fn register_watch<F, S>(&mut self, handler: F)
    where
        F: Fn(trellis_rs::service::ServiceHandlerContext, feeds::WatchInput) -> S
            + Send
            + Sync
            + 'static,
        S: futures_util::Stream<Item = Result<feeds::WatchEvent, trellis_rs::service::ServerError>>
            + Send
            + 'static,
    {
        self.runtime.register_feed::<feeds::Watch, _, _>(handler);
    }
}
