use serde_json::Value;
use trellis_protocol::{parse_authorization_context_v1, SignedAuthorizationContextV1};

use super::super::TrellisClientError;

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
        let status = response.status();
        if !status.is_success() {
            return Err(TrellisClientError::BootstrapHttp {
                status: status.as_u16(),
                body: response.text().await.unwrap_or_default(),
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
) -> Result<SignedAuthorizationContextV1, TrellisClientError> {
    parse_authorization_context_v1(&bundle.context)
        .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))
}
