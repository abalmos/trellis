use std::collections::BTreeMap;

use serde_json::Value;

use super::{AuthorizationStateError, ParticipantBindingRecord, ParticipantBindingState};

fn state_api_value() -> Result<Value, AuthorizationStateError> {
    let api: Value = serde_json::from_str(trellis_rs::sdk::state::API_JSON)
        .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
    trellis_protocol::lint_api_authoring(&api)
        .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
    Ok(api)
}

pub(crate) fn administration_participant_binding(
    resolved_at: i64,
) -> Result<ParticipantBindingRecord, AuthorizationStateError> {
    let api_value: Value = serde_json::from_str(trellis_rs::sdk::auth::API_JSON)
        .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
    let api = trellis_protocol::parse_api(&api_value)
        .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
    let state_api_value = state_api_value()?;
    let state_api = trellis_protocol::parse_api(&state_api_value)
        .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
    let mut participant_value: Value = serde_json::from_str(include_str!(
        "../../../../trellis/artifacts/trellis.admin.participant.json"
    ))
    .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
    participant_value["uses"]["required"]["auth"]["apiDigest"] = Value::String(
        api.digest()
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?,
    );
    participant_value["uses"]["required"]["state"]["apiDigest"] = Value::String(
        state_api
            .digest()
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?,
    );
    builtin_participant_binding(
        &serde_json::to_string(&participant_value)
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?,
        BTreeMap::from([
            (api.id().to_owned(), api_value),
            (state_api.id().to_owned(), state_api_value),
        ]),
        resolved_at,
    )
}

pub(crate) fn auth_runtime_participant_binding(
    resolved_at: i64,
) -> Result<ParticipantBindingRecord, AuthorizationStateError> {
    let api_value: Value = serde_json::from_str(trellis_rs::sdk::auth::API_JSON)
        .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
    trellis_protocol::lint_api_authoring(&api_value)
        .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
    let api = trellis_protocol::parse_api(&api_value)
        .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
    let mut participant_value: Value =
        serde_json::from_str(include_str!("../../../trellis.participant.json"))
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
    participant_value["implements"]["auth"]["apiDigest"] = Value::String(
        api.digest()
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?,
    );
    let mut api_values = BTreeMap::from([(api.id().to_owned(), api_value)]);
    for (alias, api_json) in [
        ("core", trellis_rs::sdk::core::API_JSON),
        ("eventlog", trellis_rs::sdk::eventlog::API_JSON),
        ("health", trellis_rs::sdk::health::API_JSON),
        ("jobs", trellis_rs::sdk::jobs::API_JSON),
        ("state", trellis_rs::sdk::state::API_JSON),
    ] {
        let value: Value = serde_json::from_str(api_json)
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        trellis_protocol::lint_api_authoring(&value)
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        let parsed = trellis_protocol::parse_api(&value)
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        participant_value["implements"][alias] = serde_json::json!({
            "api": parsed.id(),
            "apiDigest": parsed.digest().map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?,
        });
        api_values.insert(parsed.id().to_owned(), value);
    }
    builtin_participant_binding(
        &serde_json::to_string(&participant_value)
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?,
        api_values,
        resolved_at,
    )
}

fn builtin_participant_binding(
    participant_json: &str,
    api_values: BTreeMap<String, Value>,
    resolved_at: i64,
) -> Result<ParticipantBindingRecord, AuthorizationStateError> {
    trellis_protocol::lint_participant_authoring(
        &serde_json::from_str(participant_json)
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?,
    )
    .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
    let participant_value: Value = serde_json::from_str(participant_json)
        .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
    let participant = trellis_protocol::parse_participant(&participant_value)
        .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
    let mut apis = BTreeMap::new();
    for value in api_values.values() {
        let api = trellis_protocol::parse_api(value)
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        apis.insert(api.id().to_owned(), api);
    }
    let resolved = trellis_protocol::resolve_participant(&participant, &apis)
        .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
    Ok(ParticipantBindingRecord {
        participant_id: resolved.participant_id().to_owned(),
        participant_kind: resolved.participant_kind(),
        artifact_digest: resolved.participant_digest().to_owned(),
        needs_digest: resolved
            .needs()
            .digest()
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?,
        participant_json: participant
            .canonical_json()
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?,
        api_artifacts_json: trellis_protocol::canonicalize_json(
            &serde_json::to_value(api_values)
                .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?,
        )
        .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?,
        resolved_at,
        state: ParticipantBindingState::Resolved,
        error: None,
    })
}

#[cfg(test)]
mod state_api_digest_test {
    #[test]
    fn admin_binding_includes_state_admin_api() {
        let binding = super::administration_participant_binding(0).expect("admin binding");
        assert!(binding.api_artifacts_json.contains("trellis.state@v1"));
    }
}
