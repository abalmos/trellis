//! Client authorization: own-context management, pre-NATS trust bootstrap,
//! and connected NATS-backed provider-side resolution.

mod bootstrap_http;
mod core;
mod own_context;
mod persistence;
mod provider_cache;
mod refresh;
mod registry;
mod types;

pub use core::{
    AuthorizationVerificationCore, AuthorizationVerificationError, EventVerificationInput,
    RequestVerificationInput, VerifiedAuthorizationEvent, VerifiedAuthorizationRequest,
    VerifiedCaller,
};
pub use own_context::AuthorizationContextCache;
pub use provider_cache::AuthorizationProviderCache;
#[cfg(feature = "test-support")]
pub use provider_cache::IntegrationTestAuthorizationIoCounters;
#[cfg(feature = "runtime-internals")]
pub use provider_cache::{RuntimeAuthorizationIoCounters, RuntimeAuthorizationTrust};
pub(crate) use refresh::spawn_authorization_context_refresh_task;
#[cfg(any(test, feature = "runtime-internals"))]
pub use types::AuthorizationRegistryBinding;
pub use types::{
    AuthorizationClientState, AuthorizationClientTrustState, AuthorizationContextBundle,
    AuthorizationContextStore, AuthorizationRoutingMaterial, AuthorizationSessionBinding,
    AuthorizationTrustBundle, AuthorizationTrustPolicy,
};

pub use persistence::{FileAuthorizationContextStore, MemoryAuthorizationContextStore};

#[cfg(test)]
pub(crate) use own_context::inject_verified_for_test as inject_own_verified_for_test;
