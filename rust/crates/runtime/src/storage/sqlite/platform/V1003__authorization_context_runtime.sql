PRAGMA foreign_keys = ON;

CREATE TABLE auth_authorization_trust_state (
    singleton_id INTEGER PRIMARY KEY CHECK (singleton_id = 1),
    authority TEXT NOT NULL CHECK (length(authority) > 0),
    root_key_id TEXT NOT NULL CHECK (length(root_key_id) = 43),
    root_digest TEXT NOT NULL CHECK (length(root_digest) = 43),
    manifest_generation INTEGER NOT NULL CHECK (manifest_generation BETWEEN 1 AND 9007199254740991),
    manifest_digest TEXT NOT NULL CHECK (length(manifest_digest) = 43),
    active_issuer_key_id TEXT NOT NULL CHECK (length(active_issuer_key_id) = 43),
    updated_at INTEGER NOT NULL CHECK (updated_at BETWEEN 0 AND 9007199254740991),
    version INTEGER NOT NULL CHECK (version BETWEEN 1 AND 9007199254740991)
);

CREATE TABLE auth_authorization_contexts (
    context_id TEXT PRIMARY KEY CHECK (length(context_id) > 0),
    context_digest TEXT NOT NULL UNIQUE CHECK (length(context_digest) = 43),
    session_id TEXT NOT NULL REFERENCES auth_sessions(session_id),
    principal_id TEXT NOT NULL CHECK (length(principal_id) > 0),
    principal_kind TEXT NOT NULL CHECK (principal_kind IN ('user', 'service', 'device')),
    participant_id TEXT NOT NULL CHECK (length(participant_id) > 0),
    participant_artifact_digest TEXT NOT NULL CHECK (length(participant_artifact_digest) = 43),
    participant_needs_digest TEXT NOT NULL CHECK (length(participant_needs_digest) = 43),
    authority_kind TEXT NOT NULL CHECK (authority_kind IN ('identity', 'deployment')),
    authority_id TEXT NOT NULL CHECK (length(authority_id) > 0),
    authority_version INTEGER NOT NULL CHECK (authority_version BETWEEN 1 AND 9007199254740991),
    materialization_version INTEGER NOT NULL CHECK (materialization_version BETWEEN 1 AND 9007199254740991),
    deployment_id TEXT,
    instance_id TEXT,
    issuer_key_id TEXT NOT NULL CHECK (length(issuer_key_id) = 43),
    signed_context_json TEXT NOT NULL CHECK (json_valid(signed_context_json)),
    context_token TEXT NOT NULL CHECK (length(context_token) > 0),
    issuance_snapshot_token TEXT NOT NULL CHECK (length(issuance_snapshot_token) = 43),
    trust_generation INTEGER NOT NULL CHECK (trust_generation BETWEEN 1 AND 9007199254740991),
    issued_at INTEGER NOT NULL CHECK (issued_at BETWEEN 0 AND 9007199254740991),
    not_before INTEGER NOT NULL CHECK (not_before BETWEEN 0 AND 9007199254740991),
    expires_at INTEGER NOT NULL CHECK (expires_at BETWEEN 0 AND 9007199254740991),
    refresh_at INTEGER NOT NULL CHECK (refresh_at BETWEEN 0 AND 9007199254740991),
    state TEXT NOT NULL CHECK (state IN ('active', 'revoked', 'expired')),
    published_at INTEGER CHECK (published_at BETWEEN 0 AND 9007199254740991),
    revoked_at INTEGER CHECK (revoked_at BETWEEN 0 AND 9007199254740991),
    revocation_reason TEXT,
    version INTEGER NOT NULL CHECK (version BETWEEN 1 AND 9007199254740991),
    CHECK ((deployment_id IS NULL) = (instance_id IS NULL)),
    CHECK (not_before <= issued_at),
    CHECK (expires_at > not_before),
    CHECK (refresh_at BETWEEN not_before AND expires_at),
    CHECK ((state = 'revoked') = (revoked_at IS NOT NULL)),
    CHECK ((state = 'revoked') = (revocation_reason IS NOT NULL)),
    CHECK (state = 'revoked' OR (revoked_at IS NULL AND revocation_reason IS NULL))
);

CREATE INDEX auth_authorization_contexts_session_idx
    ON auth_authorization_contexts(session_id, state, expires_at);
CREATE INDEX auth_authorization_contexts_principal_idx
    ON auth_authorization_contexts(principal_id, state, expires_at);
CREATE INDEX auth_authorization_contexts_authority_idx
    ON auth_authorization_contexts(authority_kind, authority_id, state, expires_at);
CREATE INDEX auth_authorization_contexts_deployment_idx
    ON auth_authorization_contexts(deployment_id, state, expires_at);
CREATE INDEX auth_authorization_contexts_instance_idx
    ON auth_authorization_contexts(instance_id, state, expires_at);
CREATE INDEX auth_authorization_contexts_issuer_idx
    ON auth_authorization_contexts(issuer_key_id, state, expires_at);
CREATE INDEX auth_authorization_contexts_state_idx
    ON auth_authorization_contexts(state, expires_at);

ALTER TABLE auth_post_commit_actions RENAME TO auth_post_commit_actions_v1002;

CREATE TABLE auth_post_commit_actions (
    action_id TEXT PRIMARY KEY CHECK (length(action_id) = 43),
    kind TEXT NOT NULL CHECK (kind IN ('event', 'kick', 'context_publish', 'context_revoke')),
    payload_json TEXT NOT NULL CHECK (json_valid(payload_json)),
    created_at INTEGER NOT NULL CHECK (created_at BETWEEN 0 AND 9007199254740991),
    attempts INTEGER NOT NULL DEFAULT 0 CHECK (attempts >= 0),
    next_attempt_at INTEGER NOT NULL CHECK (next_attempt_at BETWEEN 0 AND 9007199254740991),
    claimed_until INTEGER CHECK (claimed_until BETWEEN 0 AND 9007199254740991),
    last_error TEXT
);

INSERT INTO auth_post_commit_actions (
    action_id,
    kind,
    payload_json,
    created_at,
    attempts,
    next_attempt_at,
    claimed_until,
    last_error
)
SELECT
    action_id,
    kind,
    payload_json,
    created_at,
    attempts,
    next_attempt_at,
    claimed_until,
    last_error
FROM auth_post_commit_actions_v1002;

DROP TABLE auth_post_commit_actions_v1002;

CREATE INDEX auth_post_commit_actions_ready_idx
    ON auth_post_commit_actions(next_attempt_at, action_id);
