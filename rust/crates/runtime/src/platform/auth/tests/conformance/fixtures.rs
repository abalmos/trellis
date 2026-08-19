use std::collections::BTreeMap;

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use ed25519_dalek::SigningKey;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use trellis_protocol::{
    authorization_context_signing_digest_v1, parse_api_v1, parse_participant_v1,
    resolve_participant_v1, GrantSetV1, ParticipantKindV1, UnsignedAuthorizationContextV1,
    AUTHORIZATION_CONTEXT_FORMAT_V1,
};

use crate::platform::auth::application::repository::{SessionCreation, SessionRevocation};
use crate::platform::auth::{
    AuthorityEvidenceScope, AuthorityKind, AuthorityTarget, DependencyEvidence, DependencyState,
    DesiredAuthorityRecord, IdempotencyResultRecord, IssuableAuthorizationState,
    ParticipantBindingRecord, ParticipantBindingState, PostCommitActionKind,
    PostCommitActionRecord, SessionRecord, SessionRuntimeBinding, UserProfileRecord,
};

pub(super) const NOW: i64 = 1_800_000_000_000;

pub(super) struct ParticipantFixture {
    pub(super) binding: ParticipantBindingRecord,
    pub(super) required_grants: GrantSetV1,
    pub(super) all_grants: GrantSetV1,
    pub(super) required_dependency: DependencyEvidence,
    pub(super) optional_dependency: DependencyEvidence,
}

pub(super) fn participant_fixture() -> Result<ParticipantFixture, Box<dyn std::error::Error>> {
    participant_fixture_for(ParticipantKindV1::App, "example.app")
}

pub(super) fn participant_fixture_for(
    kind: ParticipantKindV1,
    participant_id: &str,
) -> Result<ParticipantFixture, Box<dyn std::error::Error>> {
    participant_fixture_with_resources(kind, participant_id, false)
}

pub(super) fn participant_fixture_with_resources(
    kind: ParticipantKindV1,
    participant_id: &str,
    include_resources: bool,
) -> Result<ParticipantFixture, Box<dyn std::error::Error>> {
    let required_api = parse_api_v1(&json!({
        "format": "trellis.api.v1",
        "id": "required.api@v1",
        "displayName": "Required API",
        "description": "Required API fixture.",
        "schemas": { "Input": true, "Output": true },
        "rpc": {
            "Required.Get": {
                "version": "v1",
                "input": { "schema": "Input" },
                "output": { "schema": "Output" }
            }
        }
    }))?;
    let optional_api = parse_api_v1(&json!({
        "format": "trellis.api.v1",
        "id": "optional.api@v1",
        "displayName": "Optional API",
        "description": "Optional API fixture.",
        "schemas": { "Input": true, "Output": true },
        "rpc": {
            "Optional.Get": {
                "version": "v1",
                "input": { "schema": "Input" },
                "output": { "schema": "Output" }
            }
        }
    }))?;
    let required_digest = required_api.digest()?;
    let optional_digest = optional_api.digest()?;
    let kind_name = match kind {
        ParticipantKindV1::Service => "service",
        ParticipantKindV1::App => "app",
        ParticipantKindV1::Device => "device",
        ParticipantKindV1::Agent => "agent",
    };
    let mut participant_value = json!({
        "format": "trellis.participant.v1",
        "id": participant_id,
        "displayName": "Example App",
        "description": "Authorization materialization fixture.",
        "kind": kind_name,
        "uses": {
            "required": {
                "requiredApi": {
                    "api": "required.api@v1",
                    "apiDigest": required_digest,
                    "rpc": { "call": ["Required.Get"] }
                }
            },
            "optional": {
                "optionalApi": {
                    "api": "optional.api@v1",
                    "apiDigest": optional_digest,
                    "rpc": { "call": ["Optional.Get"] }
                }
            }
        }
    });
    if include_resources {
        participant_value["schemas"] = json!({ "CacheValue": true });
        participant_value["resources"] = json!({
            "kv": {
                "cache": {
                    "purpose": "Required cache storage.",
                    "schema": { "schema": "CacheValue" },
                    "required": true
                }
            },
            "store": {
                "attachments": {
                    "purpose": "Optional attachment storage.",
                    "required": false
                }
            }
        });
    }
    let participant = parse_participant_v1(&participant_value)?;
    let mut apis = BTreeMap::new();
    apis.insert(required_api.id().to_owned(), required_api.clone());
    apis.insert(optional_api.id().to_owned(), optional_api.clone());
    let resolved = resolve_participant_v1(&participant, &apis)?;
    let required_grants = resolved.needs().required().grant_set().clone();
    let all_grants = GrantSetV1::new(
        required_grants
            .permissions()
            .iter()
            .chain(resolved.needs().optional().grant_set().permissions())
            .cloned()
            .collect(),
    );
    let mut api_values = BTreeMap::<String, Value>::new();
    for (id, api) in &apis {
        api_values.insert(id.clone(), api.normalized_value()?);
    }
    let binding = ParticipantBindingRecord {
        participant_id: participant.id().to_owned(),
        participant_kind: participant.kind(),
        artifact_digest: participant.digest()?,
        needs_digest: resolved.needs().digest()?,
        participant_json: participant.canonical_json()?,
        api_artifacts_json: serde_json::to_string(&api_values)?,
        resolved_at: NOW,
        state: ParticipantBindingState::Resolved,
        error: None,
    };
    Ok(ParticipantFixture {
        binding,
        required_grants,
        all_grants,
        required_dependency: DependencyEvidence {
            alias: "requiredApi".to_owned(),
            required: true,
            api_id: required_api.id().to_owned(),
            api_digest: required_digest,
            provider_participant_id: "required.provider".to_owned(),
            provider_deployment_id: Some("required.deployment".to_owned()),
            provider_instance_id: Some("required.instance".to_owned()),
            state: DependencyState::Available,
            observed_at: NOW,
        },
        optional_dependency: DependencyEvidence {
            alias: "optionalApi".to_owned(),
            required: false,
            api_id: optional_api.id().to_owned(),
            api_digest: optional_digest,
            provider_participant_id: "optional.provider".to_owned(),
            provider_deployment_id: Some("optional.deployment".to_owned()),
            provider_instance_id: Some("optional.instance".to_owned()),
            state: DependencyState::Available,
            observed_at: NOW,
        },
    })
}

pub(super) fn digest(byte: u8) -> String {
    URL_SAFE_NO_PAD.encode([byte; 32])
}

pub(super) fn test_session_creation(
    session: SessionRecord,
    desired_authority: Option<DesiredAuthorityRecord>,
    runtime_binding: Option<SessionRuntimeBinding>,
) -> SessionCreation {
    let scope = format!("session.create:{}", session.session_id);
    SessionCreation {
        idempotency: test_session_idempotency(&scope, &session.principal_id, session.created_at),
        session,
        previous_session: None,
        desired_authority,
        runtime_binding,
        actions: Vec::new(),
    }
}

pub(super) fn test_session_revocation(
    session: &SessionRecord,
    expected_version: u64,
    revoked_at: i64,
    request: &str,
) -> SessionRevocation {
    let event_id = test_digest(&format!("session.revoke.event:{request}"));
    let kick_id = test_digest(&format!("session.revoke.kick:{request}"));
    SessionRevocation {
        session_id: session.session_id.clone(),
        expected_version,
        revoked_at,
        idempotency: test_session_idempotency(
            &format!("session.revoke:{request}"),
            &session.principal_id,
            revoked_at,
        ),
        actions: vec![
            PostCommitActionRecord {
                predecessor_action_id: None,
                action_id: event_id,
                kind: PostCommitActionKind::Event,
                payload: json!({ "sessionId": session.session_id }),
                created_at: revoked_at,
                attempts: 0,
                next_attempt_at: revoked_at,
                claimed_until: None,
                last_error: None,
            },
            PostCommitActionRecord {
                predecessor_action_id: None,
                action_id: kick_id,
                kind: PostCommitActionKind::Kick,
                payload: json!({ "sessionId": session.session_id }),
                created_at: revoked_at,
                attempts: 0,
                next_attempt_at: revoked_at,
                claimed_until: None,
                last_error: None,
            },
        ],
    }
}

pub(super) fn test_session_idempotency(
    request: &str,
    signer_id: &str,
    created_at: i64,
) -> IdempotencyResultRecord {
    IdempotencyResultRecord {
        scope_key: test_digest(&format!("scope:{request}")),
        purpose: if request.starts_with("session.create:") {
            "session.create"
        } else {
            "session.revoke"
        }
        .to_owned(),
        signer_id: signer_id.to_owned(),
        request_id: request.to_owned(),
        request_digest: test_digest(&format!("request:{request}")),
        result: json!({ "request": request }),
        created_at,
        expires_at: created_at + 10_000,
    }
}

pub(super) fn test_digest(value: &str) -> String {
    URL_SAFE_NO_PAD.encode(Sha256::digest(value.as_bytes()))
}

pub(super) fn evidence_scope(
    kind: AuthorityKind,
    authority_id: &str,
    binding: &ParticipantBindingRecord,
) -> AuthorityEvidenceScope {
    AuthorityEvidenceScope {
        target: AuthorityTarget {
            kind,
            authority_id: authority_id.to_owned(),
        },
        participant_id: binding.participant_id.clone(),
        participant_artifact_digest: binding.artifact_digest.clone(),
        participant_needs_digest: binding.needs_digest.clone(),
    }
}

pub(super) fn session_public_key(seed: u8) -> String {
    URL_SAFE_NO_PAD.encode(
        SigningKey::from_bytes(&[seed; 32])
            .verifying_key()
            .to_bytes(),
    )
}

pub(super) fn assert_issuable_context_valid(
    state: &IssuableAuthorizationState,
) -> Result<(), Box<dyn std::error::Error>> {
    authorization_context_signing_digest_v1(&UnsignedAuthorizationContextV1 {
        format: AUTHORIZATION_CONTEXT_FORMAT_V1.to_owned(),
        authority: "test-authority".to_owned(),
        issuer_key_id: URL_SAFE_NO_PAD.encode([99_u8; 32]),
        issuer_manifest_generation: 1,
        session_id: state.session_id.clone(),
        session_key: state.session_public_key.clone(),
        principal: state.principal.clone(),
        participant: state.participant.clone(),
        authority_ref: state.authority_ref.clone(),
        deployment_id: state.deployment_id.clone(),
        instance_id: state.instance_id.clone(),
        inbox_prefix: state.inbox_prefix.clone(),
        issued_at: NOW,
        not_before: NOW,
        expires_at: NOW + 1,
        grant_set: state.grant_set.clone(),
        capabilities: state.capabilities.clone(),
        extensions: serde_json::Map::new(),
        critical: Vec::new(),
    })
    .map(|_| ())
    .map_err(Into::into)
}

pub(super) fn profile_for(principal_id: &str) -> UserProfileRecord {
    UserProfileRecord {
        principal_id: principal_id.to_owned(),
        display_name: Some("User".to_owned()),
        email: None,
        image_url: None,
        created_at: NOW,
        updated_at: NOW,
        version: 1,
    }
}
