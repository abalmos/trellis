PRAGMA foreign_keys = ON;

CREATE TABLE auth_principals (
    principal_id TEXT PRIMARY KEY CHECK (length(principal_id) > 0),
    kind TEXT NOT NULL CHECK (kind IN ('user', 'service', 'device')),
    state TEXT NOT NULL CHECK (state IN ('active', 'disabled', 'revoked')),
    created_at INTEGER NOT NULL CHECK (created_at BETWEEN 0 AND 9007199254740991),
    updated_at INTEGER NOT NULL CHECK (updated_at BETWEEN 0 AND 9007199254740991),
    version INTEGER NOT NULL CHECK (version BETWEEN 1 AND 9007199254740991),
    disabled_at INTEGER CHECK (disabled_at BETWEEN 0 AND 9007199254740991),
    revoked_at INTEGER CHECK (revoked_at BETWEEN 0 AND 9007199254740991),
    CHECK ((state = 'disabled') = (disabled_at IS NOT NULL)),
    CHECK ((state = 'revoked') = (revoked_at IS NOT NULL))
);

CREATE TABLE auth_provider_identities (
    provider TEXT NOT NULL CHECK (length(provider) > 0),
    provider_subject TEXT NOT NULL CHECK (length(provider_subject) > 0),
    principal_id TEXT NOT NULL REFERENCES auth_principals(principal_id) ON DELETE CASCADE,
    linked_at INTEGER NOT NULL CHECK (linked_at BETWEEN 0 AND 9007199254740991),
    last_seen_at INTEGER NOT NULL CHECK (last_seen_at BETWEEN 0 AND 9007199254740991),
    PRIMARY KEY (provider, provider_subject)
);

CREATE TABLE auth_installed_participants (
    participant_id TEXT NOT NULL CHECK (length(participant_id) > 0),
    revision INTEGER NOT NULL CHECK (revision BETWEEN 1 AND 9007199254740991),
    participant_kind TEXT NOT NULL CHECK (participant_kind IN ('service', 'app', 'device', 'agent')),
    artifact_digest TEXT NOT NULL CHECK (length(artifact_digest) = 43),
    needs_digest TEXT NOT NULL CHECK (length(needs_digest) = 43),
    participant_json TEXT NOT NULL CHECK (json_valid(participant_json)),
    api_artifacts_json TEXT NOT NULL CHECK (json_valid(api_artifacts_json)),
    installed_at INTEGER NOT NULL CHECK (installed_at BETWEEN 0 AND 9007199254740991),
    PRIMARY KEY (participant_id, revision)
);
CREATE TRIGGER auth_installed_participant_is_immutable
BEFORE UPDATE ON auth_installed_participants
BEGIN
    SELECT RAISE(ABORT, 'installed participant snapshots are immutable');
END;

CREATE TRIGGER auth_installed_participant_no_delete
BEFORE DELETE ON auth_installed_participants
BEGIN
    SELECT RAISE(ABORT, 'installed participant snapshots are retained');
END;

CREATE TABLE auth_grant_bindings (
    owner_kind TEXT NOT NULL CHECK (owner_kind IN ('deployment', 'user')),
    owner_id TEXT NOT NULL CHECK (length(owner_id) > 0),
    participant_id TEXT NOT NULL,
    installed_revision INTEGER NOT NULL CHECK (installed_revision BETWEEN 1 AND 9007199254740991),
    grants_json TEXT NOT NULL CHECK (json_valid(grants_json)),
    platform_privileges_json TEXT NOT NULL CHECK (json_valid(platform_privileges_json)),
    revision INTEGER NOT NULL CHECK (revision BETWEEN 1 AND 9007199254740991),
    state TEXT NOT NULL CHECK (state IN ('active', 'revoked')),
    expires_at INTEGER CHECK (expires_at BETWEEN 0 AND 9007199254740991),
    provenance_json TEXT CHECK (provenance_json IS NULL OR json_valid(provenance_json)),
    created_at INTEGER NOT NULL CHECK (created_at BETWEEN 0 AND 9007199254740991),
    updated_at INTEGER NOT NULL CHECK (updated_at BETWEEN 0 AND 9007199254740991),
    PRIMARY KEY (owner_kind, owner_id, participant_id),
    FOREIGN KEY (participant_id, installed_revision)
        REFERENCES auth_installed_participants(participant_id, revision),
    CHECK (state != 'revoked' OR (
        json_array_length(grants_json, '$.permissions') = 0
        AND json_array_length(platform_privileges_json) = 0
    )),
    CHECK (provenance_json IS NULL OR owner_kind = 'user'),
    CHECK (updated_at >= created_at)
);

CREATE TABLE auth_sessions (
    session_id TEXT PRIMARY KEY CHECK (length(session_id) = 26),
    principal_id TEXT NOT NULL REFERENCES auth_principals(principal_id),
    participant_id TEXT NOT NULL CHECK (length(participant_id) > 0),
    participant_kind TEXT NOT NULL CHECK (participant_kind IN ('app', 'agent')),
    session_public_key TEXT NOT NULL CHECK (length(session_public_key) = 43),
    session_key_id TEXT NOT NULL CHECK (length(session_key_id) = 43),
    state TEXT NOT NULL CHECK (state IN ('active', 'expired', 'revoked')),
    created_at INTEGER NOT NULL CHECK (created_at BETWEEN 0 AND 9007199254740991),
    last_authenticated_at INTEGER NOT NULL CHECK (last_authenticated_at BETWEEN 0 AND 9007199254740991),
    expires_at INTEGER CHECK (expires_at BETWEEN 0 AND 9007199254740991),
    revoked_at INTEGER CHECK (revoked_at BETWEEN 0 AND 9007199254740991),
    version INTEGER NOT NULL CHECK (version BETWEEN 1 AND 9007199254740991),
    UNIQUE (participant_id, session_public_key),
    UNIQUE (participant_id, session_key_id),
    CHECK (last_authenticated_at >= created_at),
    CHECK (expires_at IS NULL OR expires_at >= created_at),
    CHECK ((state = 'revoked') = (revoked_at IS NOT NULL))
);
CREATE INDEX auth_sessions_principal_idx ON auth_sessions(principal_id);

CREATE TRIGGER auth_sessions_immutable_identity BEFORE UPDATE ON auth_sessions
WHEN NEW.session_id IS NOT OLD.session_id OR NEW.principal_id IS NOT OLD.principal_id
  OR NEW.participant_id IS NOT OLD.participant_id OR NEW.participant_kind IS NOT OLD.participant_kind
  OR NEW.session_public_key IS NOT OLD.session_public_key OR NEW.session_key_id IS NOT OLD.session_key_id
  OR NEW.created_at IS NOT OLD.created_at OR NEW.expires_at IS NOT OLD.expires_at
BEGIN SELECT RAISE(ABORT, 'login identity and absolute expiry are immutable'); END;
CREATE TRIGGER auth_sessions_no_revival BEFORE UPDATE ON auth_sessions
WHEN (OLD.state != 'active' AND NEW.state = 'active')
  OR (OLD.state = 'revoked' AND (NEW.state != 'revoked' OR NEW.revoked_at IS NOT OLD.revoked_at))
BEGIN SELECT RAISE(ABORT, 'terminal logins cannot be revived'); END;
CREATE TRIGGER auth_sessions_no_delete BEFORE DELETE ON auth_sessions
BEGIN SELECT RAISE(ABORT, 'login tombstones are retained'); END;

CREATE TABLE auth_deployments (
    deployment_id TEXT PRIMARY KEY CHECK (length(deployment_id) > 0),
    participant_id TEXT NOT NULL CHECK (length(participant_id) > 0),
    participant_kind TEXT NOT NULL CHECK (participant_kind IN ('service', 'device')),
    state TEXT NOT NULL CHECK (state IN ('active', 'disabled', 'revoked')),
    expires_at INTEGER CHECK (expires_at BETWEEN 0 AND 9007199254740991)
);

CREATE TABLE auth_instances (
    instance_id TEXT PRIMARY KEY CHECK (length(instance_id) > 0),
    deployment_id TEXT NOT NULL REFERENCES auth_deployments(deployment_id) ON DELETE CASCADE,
    principal_id TEXT NOT NULL REFERENCES auth_principals(principal_id) ON DELETE CASCADE,
    state TEXT NOT NULL CHECK (state IN ('active', 'disabled', 'revoked', 'stale')),
    created_at INTEGER NOT NULL CHECK (created_at BETWEEN 0 AND 9007199254740991),
    updated_at INTEGER NOT NULL CHECK (updated_at BETWEEN 0 AND 9007199254740991),
    version INTEGER NOT NULL CHECK (version BETWEEN 1 AND 9007199254740991),
    UNIQUE (instance_id, deployment_id)
);

CREATE TABLE auth_devices (
    principal_id TEXT NOT NULL REFERENCES auth_principals(principal_id) ON DELETE CASCADE,
    deployment_id TEXT NOT NULL REFERENCES auth_deployments(deployment_id) ON DELETE CASCADE,
    state TEXT NOT NULL CHECK (state IN ('pending', 'active', 'disabled', 'revoked')),
    created_at INTEGER NOT NULL CHECK (created_at BETWEEN 0 AND 9007199254740991),
    updated_at INTEGER NOT NULL CHECK (updated_at BETWEEN 0 AND 9007199254740991),
    version INTEGER NOT NULL CHECK (version BETWEEN 1 AND 9007199254740991),
    PRIMARY KEY (principal_id, deployment_id)
);

CREATE TABLE auth_device_delegations (
    principal_id TEXT NOT NULL,
    deployment_id TEXT NOT NULL,
    required INTEGER NOT NULL CHECK (required IN (0, 1)),
    state TEXT NOT NULL CHECK (state IN ('active', 'missing', 'revoked')),
    expires_at INTEGER CHECK (expires_at BETWEEN 0 AND 9007199254740991),
    PRIMARY KEY (principal_id, deployment_id),
    FOREIGN KEY (principal_id, deployment_id)
        REFERENCES auth_devices(principal_id, deployment_id) ON DELETE CASCADE
);

CREATE TABLE auth_resource_binding_evidence (
    owner_kind TEXT NOT NULL CHECK (owner_kind IN ('user', 'deployment')),
    owner_id TEXT NOT NULL CHECK (length(owner_id) > 0),
    participant_id TEXT NOT NULL CHECK (length(participant_id) > 0),
    installed_revision INTEGER NOT NULL CHECK (installed_revision BETWEEN 1 AND 9007199254740991),
    resource_kind TEXT NOT NULL CHECK (length(resource_kind) > 0),
    local_name TEXT NOT NULL CHECK (length(local_name) > 0),
    binding_id TEXT NOT NULL CHECK (length(binding_id) > 0),
    provider_identity TEXT NOT NULL CHECK (length(provider_identity) > 0),
    state TEXT NOT NULL CHECK (state IN ('available', 'unavailable', 'stale')),
    materialized_at INTEGER NOT NULL CHECK (materialized_at BETWEEN 0 AND 9007199254740991),
    error TEXT,
    PRIMARY KEY (owner_kind, owner_id, participant_id, installed_revision, resource_kind, local_name),
    UNIQUE (owner_kind, owner_id, participant_id, installed_revision, binding_id),
    FOREIGN KEY (participant_id, installed_revision) REFERENCES auth_installed_participants(participant_id, revision)
);
