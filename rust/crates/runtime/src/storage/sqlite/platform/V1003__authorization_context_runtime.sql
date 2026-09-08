PRAGMA foreign_keys = ON;

CREATE TABLE auth_authorization_issuers (
    key_id TEXT PRIMARY KEY CHECK (length(key_id) = 43),
    public_key TEXT NOT NULL CHECK (length(public_key) = 43),
    is_current INTEGER NOT NULL CHECK (is_current IN (0, 1)),
    live_until_seconds INTEGER NOT NULL DEFAULT 0 CHECK (live_until_seconds >= 0),
    created_at INTEGER NOT NULL CHECK (created_at BETWEEN 0 AND 9007199254740991),
    revoked_at INTEGER CHECK (revoked_at BETWEEN 0 AND 9007199254740991),
    CHECK (is_current = 0 OR revoked_at IS NULL)
);

CREATE UNIQUE INDEX idx_auth_authorization_current_issuer
    ON auth_authorization_issuers(is_current) WHERE is_current = 1;

CREATE TRIGGER auth_authorization_issuer_revocation_is_final
BEFORE UPDATE OF revoked_at ON auth_authorization_issuers
WHEN OLD.revoked_at IS NOT NULL AND NEW.revoked_at IS NOT OLD.revoked_at
BEGIN
    SELECT RAISE(ABORT, 'issuer revocation is irreversible');
END;

CREATE TABLE auth_authorization_contexts (
    context_digest TEXT PRIMARY KEY CHECK (length(context_digest) = 43),
    connection_id TEXT NOT NULL CHECK (length(connection_id) = 26),
    session_public_key TEXT NOT NULL CHECK (length(session_public_key) = 43),
    inbox_prefix TEXT NOT NULL CHECK (length(inbox_prefix) > 0),
    principal_id TEXT NOT NULL CHECK (length(principal_id) > 0),
    principal_kind TEXT NOT NULL CHECK (principal_kind IN ('user', 'service', 'device')),
    participant_id TEXT NOT NULL CHECK (length(participant_id) > 0),
    owner_kind TEXT NOT NULL CHECK (owner_kind IN ('user', 'deployment')),
    owner_id TEXT NOT NULL CHECK (length(owner_id) > 0),
    grant_revision INTEGER NOT NULL CHECK (grant_revision BETWEEN 1 AND 9007199254740991),
    installed_revision INTEGER NOT NULL CHECK (installed_revision BETWEEN 1 AND 9007199254740991),
    identity_key_id TEXT,
    login_session_id TEXT,
    issuer_key_id TEXT NOT NULL CHECK (length(issuer_key_id) = 43),
    signed_context_json TEXT NOT NULL CHECK (json_valid(signed_context_json)),
    issuance_snapshot_token TEXT NOT NULL CHECK (length(issuance_snapshot_token) = 43),
    issued_at INTEGER NOT NULL CHECK (issued_at BETWEEN 0 AND 9007199254740991),
    not_before INTEGER NOT NULL CHECK (not_before BETWEEN 0 AND 9007199254740991),
    refresh_at INTEGER NOT NULL CHECK (refresh_at BETWEEN 0 AND 9007199254740991),
    expires_at INTEGER NOT NULL CHECK (expires_at BETWEEN 0 AND 9007199254740991),
    state TEXT NOT NULL CHECK (state IN ('active', 'revoked', 'expired')),
    published_at INTEGER CHECK (published_at BETWEEN 0 AND 9007199254740991),
    revoked_at INTEGER CHECK (revoked_at BETWEEN 0 AND 9007199254740991),
    revocation_reason TEXT,
    version INTEGER NOT NULL CHECK (version BETWEEN 1 AND 9007199254740991),
    FOREIGN KEY (participant_id, installed_revision)
        REFERENCES auth_installed_participants(participant_id, revision),
    CHECK ((principal_kind = 'user' AND owner_kind = 'user' AND owner_id = principal_id
            AND login_session_id IS NOT NULL AND identity_key_id IS NULL)
        OR (principal_kind IN ('service', 'device') AND owner_kind = 'deployment'
            AND identity_key_id IS NOT NULL AND login_session_id IS NULL)),
    CHECK (login_session_id IS NULL OR length(login_session_id) = 26),
    CHECK (identity_key_id IS NULL OR length(identity_key_id) = 43),
    CHECK (issued_at <= refresh_at),
    CHECK (not_before <= expires_at),
    CHECK (refresh_at <= expires_at),
    CHECK ((state = 'revoked') = (revoked_at IS NOT NULL)),
    CHECK ((state = 'revoked') = (revocation_reason IS NOT NULL)),
    CHECK (state = 'revoked' OR (revoked_at IS NULL AND revocation_reason IS NULL))
);

CREATE TRIGGER auth_authorization_context_history_no_delete
BEFORE DELETE ON auth_authorization_contexts
BEGIN
    SELECT RAISE(ABORT, 'authorization context history cannot be deleted');
END;

CREATE TRIGGER auth_authorization_context_evidence_is_immutable
BEFORE UPDATE OF context_digest, connection_id, session_public_key, inbox_prefix, principal_id, principal_kind,
    participant_id, owner_kind, owner_id, grant_revision, installed_revision,
    identity_key_id, login_session_id, issuer_key_id, signed_context_json,
    issuance_snapshot_token, issued_at, not_before, refresh_at, expires_at
ON auth_authorization_contexts
BEGIN
    SELECT RAISE(ABORT, 'issued authorization evidence is immutable');
END;

CREATE TRIGGER auth_authorization_context_revocation_is_final
BEFORE UPDATE OF state, revoked_at, revocation_reason ON auth_authorization_contexts
WHEN OLD.state = 'revoked' AND (
    NEW.state IS NOT OLD.state OR NEW.revoked_at IS NOT OLD.revoked_at
    OR NEW.revocation_reason IS NOT OLD.revocation_reason
)
BEGIN
    SELECT RAISE(ABORT, 'context revocation is irreversible');
END;

CREATE INDEX auth_authorization_contexts_connection_idx
    ON auth_authorization_contexts(connection_id, state, expires_at);
CREATE INDEX auth_authorization_contexts_login_idx
    ON auth_authorization_contexts(login_session_id, state, expires_at);
CREATE INDEX auth_authorization_contexts_principal_idx
    ON auth_authorization_contexts(principal_id, state, expires_at);
CREATE INDEX auth_authorization_contexts_grant_idx
    ON auth_authorization_contexts(owner_kind, owner_id, participant_id, state, expires_at);
CREATE INDEX auth_authorization_contexts_identity_idx
    ON auth_authorization_contexts(identity_key_id, state, expires_at);
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
