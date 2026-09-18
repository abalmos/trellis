//! Production-only helper for connecting a built-in live provider.
//!
//! Built-in roles that serve a live surface connect through the ordinary native
//! service bootstrap and Auth Callout using their generated participant
//! evidence and the provisioned identity seed from [`super::live_provider`].
//! The returned client is an owned public-provider transport whose installed
//! deployment is validated against the fixed reserved deployment for the role.
//!
//! Every input is either generated evidence or a provisioned native seed; this
//! helper never accepts a preverified caller or an arbitrary unsigned context.

use trellis_rs::client::{ServiceConnectWithContractOptions, TrellisClient, TrellisClientError};
use trellis_rs::generated::ParticipantDescriptor;

use super::live_provider::{LiveProviderRole, ProvisionedLiveProvider};
use super::RuntimeError;

/// Inputs for connecting one built-in live provider through normal bootstrap.
#[allow(
    dead_code,
    reason = "consumed by built-in live-router startup wiring in the Feed migration work package"
)]
pub struct BuiltinLiveProviderConnectOptions<'a> {
    /// Configured Trellis HTTP origin.
    pub trellis_url: &'a str,
    /// Connect/request timeout in milliseconds.
    pub timeout_ms: u64,
    /// Accept a non-loopback HTTP origin explicitly allow-listed by the runtime.
    pub allow_insecure_origin: bool,
}

/// Connect a built-in live provider through normal bootstrap and validate its
/// installed deployment against the fixed reserved role deployment.
///
/// # Errors
///
/// Returns [`RuntimeError::Platform`] for bootstrap failure or when the
/// installed participant/deployment does not match the provisioned role.
#[allow(
    dead_code,
    reason = "consumed by built-in live-router startup wiring in the Feed migration work package"
)]
pub async fn connect_builtin_live_provider(
    provider: &ProvisionedLiveProvider,
    options: BuiltinLiveProviderConnectOptions<'_>,
) -> Result<TrellisClient, RuntimeError> {
    let client = match provider.role {
        LiveProviderRole::Platform => {
            connect::<trellis_runtime_apis::participants::trellis_platform::Participant>(
                provider, options,
            )
            .await?
        }
        LiveProviderRole::Health => {
            connect::<trellis_runtime_apis::participants::trellis_health_runtime::Participant>(
                provider, options,
            )
            .await?
        }
    };
    let deployment_id = match provider.role {
        LiveProviderRole::Platform => expect_deployment::<
            trellis_runtime_apis::participants::trellis_platform::Participant,
        >(provider, &client)?,
        LiveProviderRole::Health => expect_deployment::<
            trellis_runtime_apis::participants::trellis_health_runtime::Participant,
        >(provider, &client)?,
    };
    let _ = deployment_id;
    Ok(client)
}

async fn connect<C: ParticipantDescriptor>(
    provider: &ProvisionedLiveProvider,
    options: BuiltinLiveProviderConnectOptions<'_>,
) -> Result<TrellisClient, RuntimeError> {
    TrellisClient::connect_service_with_contract(ServiceConnectWithContractOptions {
        trellis_url: options.trellis_url,
        participant_id: C::ID,
        participant_path: C::PATH,
        package_evidence: C::package_evidence(),
        provisioned_identity_seed_base64url: &provider.identity_seed_base64url,
        name: None,
        timeout_ms: options.timeout_ms,
        allow_insecure_origin: options.allow_insecure_origin,
    })
    .await
    .map_err(|error| RuntimeError::Platform(format!("built-in live provider connect: {error}")))
}

fn expect_deployment<C: ParticipantDescriptor>(
    provider: &ProvisionedLiveProvider,
    client: &TrellisClient,
) -> Result<String, RuntimeError> {
    let deployment_id = client
        .runtime_deployment_id()
        .map_err(|error: TrellisClientError| RuntimeError::Platform(error.to_string()))?;
    let expected = provider.role.deployment_id();
    if deployment_id != expected {
        return Err(RuntimeError::Platform(format!(
            "built-in live provider '{}' is installed under deployment '{deployment_id}' instead of '{expected}'",
            C::ID
        )));
    }
    Ok(deployment_id)
}
