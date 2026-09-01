use serde_json::Value;
use trellis_protocol::{parse_authorization_context, SignedAuthorizationContext};

use super::super::{decode_trellis_http_error, TrellisClientError};

/// Returns the canonical origin used to scope persisted client authorization state.
pub fn canonical_trellis_origin(trellis_url: &str) -> Result<String, TrellisClientError> {
    let url = reqwest::Url::parse(trellis_url)
        .map_err(|error| TrellisClientError::Bootstrap(format!("invalid Trellis URL: {error}")))?;
    let origin = url.origin().ascii_serialization();
    if origin == "null" {
        return Err(TrellisClientError::Bootstrap(
            "Trellis URL must have an HTTP(S) origin".into(),
        ));
    }
    Ok(origin)
}

/// Pre-NATS HTTP credential and context recovery client.
///
/// This client performs HTTP only for credential or context recovery.
/// Connected provider-side resolution never uses it.
#[derive(Clone, Debug)]
pub(crate) struct BootstrapHttp {
    base: reqwest::Url,
    client: reqwest::Client,
}

impl BootstrapHttp {
    pub(crate) fn new(trellis_url: &str) -> Result<Self, TrellisClientError> {
        let base = reqwest::Url::parse(trellis_url)
            .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?;
        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?;
        Ok(Self { base, client })
    }

    /// POST a JSON body to a same-origin path and return the JSON response.
    pub(crate) async fn post_json(
        &self,
        path: &str,
        body: &impl serde::Serialize,
    ) -> Result<Value, TrellisClientError> {
        let url = self
            .base
            .join(path)
            .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?;
        let response = self
            .client
            .post(url.clone())
            .json(body)
            .send()
            .await
            .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?;
        if !response.status().is_success() {
            let error = decode_trellis_http_error(response).await;
            return Err(TrellisClientError::BootstrapHttp {
                status: error.status,
                code: error.code,
            });
        }
        response
            .json()
            .await
            .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))
    }
}

/// Parse the signed context from an installed bundle.
pub(crate) fn persisted_signed_context(
    bundle: &super::types::AuthorizationContextBundle,
) -> Result<SignedAuthorizationContext, TrellisClientError> {
    parse_authorization_context(&bundle.context)
        .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))
}
