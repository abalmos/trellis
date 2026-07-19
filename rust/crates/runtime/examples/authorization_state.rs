//! End-to-end Rust authorization-state and materialization example.

use std::collections::BTreeMap;

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use ed25519_dalek::SigningKey;
use serde_json::{json, Value};
use trellis_protocol::{
    parse_api_v1, parse_participant_v1, resolve_participant_v1, GrantSetV1, ParticipantKindV1,
};
use trellis_runtime::platform::auth::{
    AuthorityDecision, AuthorityEvidenceScope, AuthorityKind, AuthorityState, AuthorityTarget,
    AuthorizationStateError, AuthorizationStateService, DependencyEvidence, DependencyState,
    EvidenceRepository, IdentityAuthorityRecord, IdentityAuthorityRepository,
    InMemoryAuthorizationStore, NewSession, ParticipantBindingRecord, ParticipantBindingRepository,
    ParticipantBindingState, PrincipalAuthorizationChange, PrincipalKind, PrincipalRecord,
    PrincipalRepository, PrincipalState, SessionRecord, SessionRepository,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let now = 1_800_000_000_000_i64;
    let required_api = parse_api_v1(&api("required.api@v1", "Required.Get"))?;
    let optional_api = parse_api_v1(&api("optional.api@v1", "Optional.Get"))?;
    let required_digest = required_api.digest()?;
    let optional_digest = optional_api.digest()?;
    let participant = parse_participant_v1(&json!({
        "format": "trellis.participant.v1",
        "id": "example.app",
        "displayName": "Example App",
        "description": "Authorization-state example.",
        "kind": "app",
        "uses": {
            "required": {
                "requiredApi": {
                    "api": required_api.id(),
                    "apiDigest": required_digest,
                    "rpc": { "call": ["Required.Get"] }
                }
            },
            "optional": {
                "optionalApi": {
                    "api": optional_api.id(),
                    "apiDigest": optional_digest,
                    "rpc": { "call": ["Optional.Get"] }
                }
            }
        }
    }))?;
    let mut apis = BTreeMap::new();
    apis.insert(required_api.id().to_owned(), required_api.clone());
    apis.insert(optional_api.id().to_owned(), optional_api.clone());
    let resolved = resolve_participant_v1(&participant, &apis)?;
    let mut api_json = BTreeMap::<String, Value>::new();
    for (id, api) in &apis {
        api_json.insert(id.clone(), api.normalized_value()?);
    }
    let binding = ParticipantBindingRecord {
        participant_id: participant.id().to_owned(),
        participant_kind: participant.kind(),
        artifact_digest: participant.digest()?,
        needs_digest: resolved.needs().digest()?,
        participant_json: participant.canonical_json()?,
        api_artifacts_json: serde_json::to_string(&api_json)?,
        resolved_at: now,
        state: ParticipantBindingState::Resolved,
        error: None,
    };

    let store = InMemoryAuthorizationStore::default();
    store.put_participant_binding(binding.clone()).await?;
    store
        .create_principal(PrincipalRecord {
            principal_id: "usr_example".to_owned(),
            kind: PrincipalKind::User,
            state: PrincipalState::Active,
            created_at: now,
            updated_at: now,
            version: 1,
            disabled_at: None,
            revoked_at: None,
        })
        .await?;
    let session = SessionRecord::from_new(NewSession {
        session_id: "ses_example".to_owned(),
        principal_id: "usr_example".to_owned(),
        principal_kind: PrincipalKind::User,
        participant_id: binding.participant_id.clone(),
        participant_kind: ParticipantKindV1::App,
        participant_artifact_digest: binding.artifact_digest.clone(),
        participant_needs_digest: binding.needs_digest.clone(),
        session_public_key: URL_SAFE_NO_PAD.encode(
            SigningKey::from_bytes(&[7_u8; 32])
                .verifying_key()
                .to_bytes(),
        ),
        inbox_prefix: "_INBOX.example".to_owned(),
        created_at: now,
        expires_at: None,
    })?;
    store.create_session(session.clone()).await?;

    let accepted_grants = GrantSetV1::new(
        resolved
            .needs()
            .required()
            .grant_set()
            .permissions()
            .iter()
            .chain(resolved.needs().optional().grant_set().permissions())
            .cloned()
            .collect(),
    );
    store
        .put_identity_authority(
            IdentityAuthorityRecord {
                authority_id: "ida_example".to_owned(),
                principal_id: "usr_example".to_owned(),
                participant_id: binding.participant_id.clone(),
                participant_artifact_digest: binding.artifact_digest.clone(),
                accepted_needs_digest: binding.needs_digest.clone(),
                desired_grant_set: accepted_grants.clone(),
                desired_capabilities: Vec::new(),
                state: AuthorityState::Accepted,
                version: 1,
                created_at: now,
                updated_at: now,
                expires_at: None,
                decision: Some(AuthorityDecision {
                    decided_at: now,
                    decided_by: "usr_owner".to_owned(),
                    reason: None,
                }),
            },
            None,
        )
        .await?;

    let facade = AuthorizationStateService::new(store.clone());
    let target = AuthorityTarget::new(AuthorityKind::Identity, "ida_example")?;
    let scope = AuthorityEvidenceScope {
        target: target.clone(),
        participant_id: binding.participant_id.clone(),
        participant_artifact_digest: binding.artifact_digest.clone(),
        participant_needs_digest: binding.needs_digest.clone(),
    };
    facade.reconcile_authority(&target, now).await?;
    assert_eq!(
        facade
            .resolve_issuable_state(&session.session_id, now)
            .await,
        Err(AuthorizationStateError::MaterializationStale)
    );
    store
        .replace_dependency_evidence(
            scope.clone(),
            vec![dependency(
                "requiredApi",
                true,
                required_api.id(),
                required_digest.clone(),
                now,
            )],
        )
        .await?;
    facade.reconcile_authority(&target, now).await?;
    let required_only = facade
        .resolve_issuable_state(&session.session_id, now)
        .await?;
    assert_eq!(
        &required_only.grant_set,
        resolved.needs().required().grant_set()
    );

    store
        .replace_dependency_evidence(
            scope,
            vec![
                dependency("requiredApi", true, required_api.id(), required_digest, now),
                dependency(
                    "optionalApi",
                    false,
                    optional_api.id(),
                    optional_digest,
                    now,
                ),
            ],
        )
        .await?;
    facade.reconcile_authority(&target, now).await?;
    let complete = facade
        .resolve_issuable_state(&session.session_id, now)
        .await?;
    assert_eq!(complete.grant_set, accepted_grants);
    assert_eq!(complete.authority_ref.version, 1);

    assert_eq!(
        store
            .update_principal_authorization_state(
                "usr_example",
                9,
                PrincipalAuthorizationChange {
                    state: PrincipalState::Disabled,
                    changed_at: now,
                },
            )
            .await,
        Err(AuthorizationStateError::StorageConflict)
    );
    Ok(())
}

fn api(id: &str, rpc: &str) -> Value {
    let mut methods = serde_json::Map::new();
    methods.insert(
        rpc.to_owned(),
        json!({
            "version": "v1",
            "input": { "schema": "Input" },
            "output": { "schema": "Output" }
        }),
    );
    json!({
        "format": "trellis.api.v1",
        "id": id,
        "displayName": id,
        "description": "Authorization-state example API.",
        "schemas": { "Input": true, "Output": true },
        "rpc": methods
    })
}

fn dependency(
    alias: &str,
    required: bool,
    api_id: &str,
    api_digest: String,
    now: i64,
) -> DependencyEvidence {
    DependencyEvidence {
        alias: alias.to_owned(),
        required,
        api_id: api_id.to_owned(),
        api_digest,
        provider_participant_id: "provider.service".to_owned(),
        provider_deployment_id: Some("provider.deployment".to_owned()),
        provider_instance_id: Some("provider.instance".to_owned()),
        state: DependencyState::Available,
        observed_at: now,
    }
}
