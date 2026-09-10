use async_trait::async_trait;

use super::{
    AuthorizationStateError, GrantBinding, GrantBindingReplacement, GrantOwnerKind,
    IdempotencyResultRecord, ParticipantBindingRecord,
};
use serde_json::Value;

#[async_trait]
pub(crate) trait GrantRepository: Send + Sync {
    async fn get_installed_participant_record(
        &self,
        participant_id: String,
        revision: Option<u64>,
    ) -> Result<Option<(u64, ParticipantBindingRecord)>, AuthorizationStateError>;

    async fn accept_presented_package(
        &self,
        input: super::evidence::PackageEvidenceInput,
        now: i64,
    ) -> Result<ParticipantBindingRecord, AuthorizationStateError>;

    async fn get_credential_participant_assignment(
        &self,
        identity_key_id: String,
    ) -> Result<Option<String>, AuthorizationStateError>;

    async fn get_grant_binding(
        &self,
        owner_kind: GrantOwnerKind,
        owner_id: String,
        participant_id: String,
    ) -> Result<Option<GrantBinding>, AuthorizationStateError>;

    async fn set_grant_binding(
        &self,
        replacement: GrantBindingReplacement,
        idempotency: IdempotencyResultRecord,
    ) -> Result<Value, AuthorizationStateError>;

    async fn set_portal_grant_binding(
        &self,
        replacement: GrantBindingReplacement,
        policy: super::PortalPolicySnapshot,
        idempotency: IdempotencyResultRecord,
    ) -> Result<Value, AuthorizationStateError>;

    async fn revoke_portal_grant_binding(
        &self,
        owner_id: String,
        participant_id: String,
        expected_revision: u64,
        policy: super::PortalPolicySnapshot,
        idempotency: IdempotencyResultRecord,
    ) -> Result<Value, AuthorizationStateError>;
}
