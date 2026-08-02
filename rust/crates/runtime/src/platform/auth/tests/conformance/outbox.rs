use trellis_protocol::ParticipantKindV1;

use super::fixtures::{digest, participant_fixture, session_public_key};
use crate::platform::auth::authority::validate_identity_authority;
use crate::platform::auth::domain::{
    AuthorityDecision, AuthorityState, AuthorizationStateError, IdentityAuthorityRecord,
    NewSession, PrincipalKind, SessionRecord, MAX_PROTOCOL_INTEGER,
};

#[test]
fn protocol_integer_boundaries_are_enforced() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = participant_fixture()?;
    let mut authority = IdentityAuthorityRecord {
        authority_id: "ida_boundary".to_owned(),
        principal_id: "usr_boundary".to_owned(),
        participant_id: fixture.binding.participant_id,
        participant_artifact_digest: fixture.binding.artifact_digest,
        accepted_needs_digest: fixture.binding.needs_digest,
        desired_grant_set: fixture.all_grants,
        desired_capabilities: Vec::new(),
        state: AuthorityState::Accepted,
        version: MAX_PROTOCOL_INTEGER,
        created_at: MAX_PROTOCOL_INTEGER as i64,
        updated_at: MAX_PROTOCOL_INTEGER as i64,
        expires_at: Some(MAX_PROTOCOL_INTEGER as i64),
        decision: Some(AuthorityDecision {
            decided_at: MAX_PROTOCOL_INTEGER as i64,
            decided_by: "usr_admin".to_owned(),
            reason: None,
        }),
    };
    validate_identity_authority(&mut authority)?;
    authority.version = MAX_PROTOCOL_INTEGER + 1;
    assert!(matches!(
        validate_identity_authority(&mut authority),
        Err(AuthorizationStateError::InvalidRecord(_))
    ));
    let too_large = (MAX_PROTOCOL_INTEGER + 1) as i64;
    assert!(matches!(
        SessionRecord::from_new(NewSession {
            session_id: "ses_boundary".to_owned(),
            principal_id: "usr_boundary".to_owned(),
            principal_kind: PrincipalKind::User,
            participant_id: "example.app".to_owned(),
            participant_kind: ParticipantKindV1::App,
            participant_artifact_digest: digest(1),
            participant_needs_digest: digest(2),
            session_public_key: session_public_key(31),
            inbox_prefix: "_INBOX.boundary".to_owned(),
            created_at: too_large,
            expires_at: None,
        }),
        Err(AuthorizationStateError::InvalidRecord(_))
    ));
    Ok(())
}
