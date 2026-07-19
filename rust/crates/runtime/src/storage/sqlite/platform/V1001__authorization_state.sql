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

CREATE TABLE auth_participant_bindings (
    participant_id TEXT NOT NULL CHECK (length(participant_id) > 0),
    participant_kind TEXT NOT NULL CHECK (participant_kind IN ('service', 'app', 'device', 'agent')),
    artifact_digest TEXT NOT NULL CHECK (length(artifact_digest) = 43),
    needs_digest TEXT NOT NULL CHECK (length(needs_digest) = 43),
    participant_json TEXT NOT NULL CHECK (json_valid(participant_json)),
    api_artifacts_json TEXT NOT NULL CHECK (json_valid(api_artifacts_json)),
    resolved_at INTEGER NOT NULL CHECK (resolved_at BETWEEN 0 AND 9007199254740991),
    state TEXT NOT NULL CHECK (state IN ('resolved', 'invalid')),
    error TEXT,
    CHECK ((state = 'resolved') = (error IS NULL)),
    PRIMARY KEY (participant_id, artifact_digest)
);

CREATE TABLE auth_sessions (
    session_id TEXT PRIMARY KEY CHECK (length(session_id) > 0),
    principal_id TEXT NOT NULL REFERENCES auth_principals(principal_id) ON DELETE CASCADE,
    principal_kind TEXT NOT NULL CHECK (principal_kind IN ('user', 'service', 'device')),
    participant_id TEXT NOT NULL,
    participant_kind TEXT NOT NULL CHECK (participant_kind IN ('service', 'app', 'device', 'agent')),
    participant_artifact_digest TEXT NOT NULL,
    participant_needs_digest TEXT NOT NULL CHECK (length(participant_needs_digest) = 43),
    session_public_key TEXT NOT NULL UNIQUE CHECK (length(session_public_key) = 43),
    session_key_id TEXT NOT NULL UNIQUE CHECK (length(session_key_id) = 43),
    inbox_prefix TEXT NOT NULL CHECK (length(inbox_prefix) > 0),
    state TEXT NOT NULL CHECK (state IN ('active', 'expired', 'revoked')),
    created_at INTEGER NOT NULL CHECK (created_at BETWEEN 0 AND 9007199254740991),
    last_seen_at INTEGER NOT NULL CHECK (last_seen_at BETWEEN 0 AND 9007199254740991),
    expires_at INTEGER CHECK (expires_at BETWEEN 0 AND 9007199254740991),
    revoked_at INTEGER CHECK (revoked_at BETWEEN 0 AND 9007199254740991),
    version INTEGER NOT NULL CHECK (version BETWEEN 1 AND 9007199254740991),
    FOREIGN KEY (participant_id, participant_artifact_digest)
        REFERENCES auth_participant_bindings(participant_id, artifact_digest),
    CHECK (expires_at IS NULL OR expires_at >= created_at),
    CHECK ((state = 'revoked') = (revoked_at IS NOT NULL)),
    CHECK (
        (principal_kind = 'user' AND participant_kind IN ('app', 'agent')) OR
        (principal_kind = 'service' AND participant_kind = 'service') OR
        (principal_kind = 'device' AND participant_kind = 'device')
    )
);
CREATE INDEX auth_sessions_principal_idx ON auth_sessions(principal_id);

CREATE TABLE auth_identity_authorities (
    authority_id TEXT PRIMARY KEY CHECK (length(authority_id) > 0),
    principal_id TEXT NOT NULL REFERENCES auth_principals(principal_id) ON DELETE CASCADE,
    participant_id TEXT NOT NULL,
    participant_artifact_digest TEXT NOT NULL,
    accepted_needs_digest TEXT NOT NULL CHECK (length(accepted_needs_digest) = 43),
    desired_grant_set_json TEXT NOT NULL CHECK (json_valid(desired_grant_set_json)),
    desired_capabilities_json TEXT NOT NULL CHECK (json_valid(desired_capabilities_json)),
    state TEXT NOT NULL CHECK (state IN ('pending', 'accepted', 'rejected', 'revoked', 'stale')),
    version INTEGER NOT NULL CHECK (version BETWEEN 1 AND 9007199254740991),
    created_at INTEGER NOT NULL CHECK (created_at BETWEEN 0 AND 9007199254740991),
    updated_at INTEGER NOT NULL CHECK (updated_at BETWEEN 0 AND 9007199254740991),
    expires_at INTEGER CHECK (expires_at BETWEEN 0 AND 9007199254740991),
    decision_at INTEGER CHECK (decision_at BETWEEN 0 AND 9007199254740991),
    decision_by TEXT,
    decision_reason TEXT,
    FOREIGN KEY (participant_id, participant_artifact_digest)
        REFERENCES auth_participant_bindings(participant_id, artifact_digest),
    CHECK ((decision_at IS NULL) = (decision_by IS NULL)),
    UNIQUE (principal_id, participant_id)
);

CREATE TABLE auth_deployment_authorities (
    authority_id TEXT PRIMARY KEY CHECK (length(authority_id) > 0),
    deployment_id TEXT NOT NULL CHECK (length(deployment_id) > 0),
    participant_id TEXT NOT NULL,
    participant_kind TEXT NOT NULL CHECK (participant_kind IN ('service', 'device')),
    participant_artifact_digest TEXT NOT NULL,
    accepted_needs_digest TEXT NOT NULL CHECK (length(accepted_needs_digest) = 43),
    desired_grant_set_json TEXT NOT NULL CHECK (json_valid(desired_grant_set_json)),
    desired_capabilities_json TEXT NOT NULL CHECK (json_valid(desired_capabilities_json)),
    state TEXT NOT NULL CHECK (state IN ('pending', 'accepted', 'rejected', 'revoked', 'stale')),
    version INTEGER NOT NULL CHECK (version BETWEEN 1 AND 9007199254740991),
    created_at INTEGER NOT NULL CHECK (created_at BETWEEN 0 AND 9007199254740991),
    updated_at INTEGER NOT NULL CHECK (updated_at BETWEEN 0 AND 9007199254740991),
    expires_at INTEGER CHECK (expires_at BETWEEN 0 AND 9007199254740991),
    decision_at INTEGER CHECK (decision_at BETWEEN 0 AND 9007199254740991),
    decision_by TEXT,
    decision_reason TEXT,
    FOREIGN KEY (participant_id, participant_artifact_digest)
        REFERENCES auth_participant_bindings(participant_id, artifact_digest),
    CHECK ((decision_at IS NULL) = (decision_by IS NULL)),
    UNIQUE (deployment_id, participant_id)
);

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
    UNIQUE (instance_id, deployment_id)
);

CREATE TABLE auth_session_runtime_bindings (
    session_id TEXT PRIMARY KEY REFERENCES auth_sessions(session_id) ON DELETE CASCADE,
    deployment_id TEXT NOT NULL REFERENCES auth_deployments(deployment_id) ON DELETE CASCADE,
    instance_id TEXT NOT NULL,
    FOREIGN KEY (instance_id, deployment_id)
        REFERENCES auth_instances(instance_id, deployment_id)
);

CREATE TABLE auth_devices (
    principal_id TEXT NOT NULL REFERENCES auth_principals(principal_id) ON DELETE CASCADE,
    deployment_id TEXT NOT NULL REFERENCES auth_deployments(deployment_id) ON DELETE CASCADE,
    state TEXT NOT NULL CHECK (state IN ('active', 'disabled', 'revoked')),
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

CREATE TABLE auth_dependency_evidence (
    authority_kind TEXT NOT NULL CHECK (authority_kind IN ('identity', 'deployment')),
    authority_id TEXT NOT NULL CHECK (length(authority_id) > 0),
    participant_id TEXT NOT NULL CHECK (length(participant_id) > 0),
    participant_artifact_digest TEXT NOT NULL CHECK (length(participant_artifact_digest) = 43),
    participant_needs_digest TEXT NOT NULL CHECK (length(participant_needs_digest) = 43),
    alias TEXT NOT NULL CHECK (length(alias) > 0),
    required INTEGER NOT NULL CHECK (required IN (0, 1)),
    api_id TEXT NOT NULL CHECK (length(api_id) > 0),
    api_digest TEXT NOT NULL CHECK (length(api_digest) = 43),
    provider_participant_id TEXT NOT NULL CHECK (length(provider_participant_id) > 0),
    provider_deployment_id TEXT,
    provider_instance_id TEXT,
    state TEXT NOT NULL CHECK (state IN ('available', 'unavailable', 'stale')),
    observed_at INTEGER NOT NULL CHECK (observed_at BETWEEN 0 AND 9007199254740991),
    PRIMARY KEY (authority_kind, authority_id, alias, required)
);

CREATE TABLE auth_resource_binding_evidence (
    authority_kind TEXT NOT NULL CHECK (authority_kind IN ('identity', 'deployment')),
    authority_id TEXT NOT NULL CHECK (length(authority_id) > 0),
    participant_id TEXT NOT NULL CHECK (length(participant_id) > 0),
    participant_artifact_digest TEXT NOT NULL CHECK (length(participant_artifact_digest) = 43),
    participant_needs_digest TEXT NOT NULL CHECK (length(participant_needs_digest) = 43),
    resource_kind TEXT NOT NULL CHECK (length(resource_kind) > 0),
    local_name TEXT NOT NULL CHECK (length(local_name) > 0),
    binding_id TEXT NOT NULL CHECK (length(binding_id) > 0),
    provider_identity TEXT NOT NULL CHECK (length(provider_identity) > 0),
    state TEXT NOT NULL CHECK (state IN ('available', 'unavailable', 'stale')),
    materialized_at INTEGER NOT NULL CHECK (materialized_at BETWEEN 0 AND 9007199254740991),
    error TEXT,
    PRIMARY KEY (authority_kind, authority_id, resource_kind, local_name),
    UNIQUE (authority_kind, authority_id, binding_id)
);

CREATE TABLE auth_materialized_authorities (
    materialization_id TEXT NOT NULL UNIQUE CHECK (length(materialization_id) > 0),
    authority_kind TEXT NOT NULL CHECK (authority_kind IN ('identity', 'deployment')),
    authority_id TEXT NOT NULL CHECK (length(authority_id) > 0),
    authority_version INTEGER NOT NULL CHECK (authority_version BETWEEN 1 AND 9007199254740991),
    materialization_version INTEGER NOT NULL CHECK (materialization_version BETWEEN 1 AND 9007199254740991),
    subject_id TEXT NOT NULL CHECK (length(subject_id) > 0),
    participant_id TEXT NOT NULL,
    participant_kind TEXT NOT NULL CHECK (participant_kind IN ('service', 'app', 'device', 'agent')),
    participant_artifact_digest TEXT NOT NULL,
    participant_needs_digest TEXT NOT NULL CHECK (length(participant_needs_digest) = 43),
    effective_grant_set_json TEXT NOT NULL CHECK (json_valid(effective_grant_set_json)),
    effective_capabilities_json TEXT NOT NULL CHECK (json_valid(effective_capabilities_json)),
    state TEXT NOT NULL CHECK (state IN ('available', 'unavailable', 'error')),
    reconciled_at INTEGER CHECK (reconciled_at BETWEEN 0 AND 9007199254740991),
    error TEXT,
    expires_at INTEGER CHECK (expires_at BETWEEN 0 AND 9007199254740991),
    CHECK ((state = 'available') = (error IS NULL)),
    PRIMARY KEY (authority_kind, authority_id)
);

CREATE TABLE auth_materialized_dependencies (
    materialization_id TEXT NOT NULL REFERENCES auth_materialized_authorities(materialization_id) ON DELETE CASCADE,
    alias TEXT NOT NULL,
    required INTEGER NOT NULL CHECK (required IN (0, 1)),
    api_id TEXT NOT NULL,
    api_digest TEXT NOT NULL,
    provider_participant_id TEXT NOT NULL,
    provider_deployment_id TEXT,
    provider_instance_id TEXT,
    state TEXT NOT NULL,
    observed_at INTEGER NOT NULL CHECK (observed_at BETWEEN 0 AND 9007199254740991),
    PRIMARY KEY (materialization_id, alias, required)
);

CREATE TABLE auth_materialized_resource_bindings (
    materialization_id TEXT NOT NULL REFERENCES auth_materialized_authorities(materialization_id) ON DELETE CASCADE,
    resource_kind TEXT NOT NULL,
    local_name TEXT NOT NULL,
    binding_id TEXT NOT NULL,
    owner_participant_id TEXT NOT NULL,
    provider_identity TEXT NOT NULL,
    state TEXT NOT NULL,
    materialized_at INTEGER NOT NULL CHECK (materialized_at BETWEEN 0 AND 9007199254740991),
    error TEXT,
    PRIMARY KEY (materialization_id, resource_kind, local_name)
);

CREATE TABLE auth_transition_outbox (
    event_id TEXT PRIMARY KEY CHECK (length(event_id) = 43),
    transition_json TEXT NOT NULL CHECK (json_valid(transition_json)),
    created_at INTEGER NOT NULL CHECK (created_at BETWEEN 0 AND 9007199254740991)
);
