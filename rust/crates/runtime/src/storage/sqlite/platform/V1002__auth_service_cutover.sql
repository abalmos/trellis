PRAGMA foreign_keys = ON;

CREATE TABLE auth_user_profiles (
    principal_id TEXT PRIMARY KEY REFERENCES auth_principals(principal_id) ON DELETE CASCADE,
    display_name TEXT CHECK (display_name IS NULL OR length(display_name) > 0),
    email TEXT,
    image_url TEXT,
    created_at INTEGER NOT NULL CHECK (created_at BETWEEN 0 AND 9007199254740991),
    updated_at INTEGER NOT NULL CHECK (updated_at BETWEEN 0 AND 9007199254740991),
    version INTEGER NOT NULL CHECK (version BETWEEN 1 AND 9007199254740991)
);

CREATE TABLE auth_local_credentials (
    principal_id TEXT PRIMARY KEY REFERENCES auth_principals(principal_id) ON DELETE CASCADE,
    normalized_username TEXT NOT NULL UNIQUE CHECK (length(normalized_username) > 0),
    password_hash TEXT NOT NULL CHECK (length(password_hash) > 0),
    hash_profile INTEGER NOT NULL CHECK (hash_profile >= 1),
    failed_attempts INTEGER NOT NULL DEFAULT 0 CHECK (failed_attempts >= 0),
    locked_until INTEGER CHECK (locked_until BETWEEN 0 AND 9007199254740991),
    password_changed_at INTEGER NOT NULL CHECK (password_changed_at BETWEEN 0 AND 9007199254740991),
    updated_at INTEGER NOT NULL CHECK (updated_at BETWEEN 0 AND 9007199254740991),
    version INTEGER NOT NULL CHECK (version BETWEEN 1 AND 9007199254740991)
);

CREATE TABLE auth_login_portals (
    portal_id TEXT PRIMARY KEY CHECK (length(portal_id) > 0),
    display_name TEXT NOT NULL CHECK (length(display_name) > 0),
    entry_url TEXT,
    builtin INTEGER NOT NULL CHECK (builtin IN (0, 1)),
    disabled INTEGER NOT NULL CHECK (disabled IN (0, 1)),
    removed INTEGER NOT NULL CHECK (removed IN (0, 1)),
    local_registration_enabled INTEGER NOT NULL CHECK (local_registration_enabled IN (0, 1)),
    provider_ids_json TEXT NOT NULL CHECK (json_valid(provider_ids_json)),
    created_at INTEGER NOT NULL CHECK (created_at BETWEEN 0 AND 9007199254740991),
    updated_at INTEGER NOT NULL CHECK (updated_at BETWEEN 0 AND 9007199254740991),
    version INTEGER NOT NULL CHECK (version BETWEEN 1 AND 9007199254740991)
);

CREATE TABLE auth_login_settings (
    portal_id TEXT PRIMARY KEY REFERENCES auth_login_portals(portal_id) ON DELETE CASCADE,
    default_provider_id TEXT,
    local_login_enabled INTEGER NOT NULL CHECK (local_login_enabled IN (0, 1)),
    federated_registration_enabled INTEGER NOT NULL CHECK (federated_registration_enabled IN (0, 1)),
    provider_selection_enabled INTEGER NOT NULL CHECK (provider_selection_enabled IN (0, 1)),
    updated_at INTEGER NOT NULL CHECK (updated_at BETWEEN 0 AND 9007199254740991),
    version INTEGER NOT NULL CHECK (version BETWEEN 1 AND 9007199254740991)
);

CREATE TABLE auth_deployment_profiles (
    deployment_id TEXT PRIMARY KEY REFERENCES auth_principals(principal_id) ON DELETE CASCADE,
    kind TEXT NOT NULL CHECK (kind IN ('service', 'device')),
    display_name TEXT NOT NULL CHECK (length(display_name) > 0),
    participant_id TEXT,
    portal_id TEXT REFERENCES auth_login_portals(portal_id) ON DELETE SET NULL,
    requires_device_delegation INTEGER NOT NULL CHECK (requires_device_delegation IN (0, 1)),
    expires_at INTEGER CHECK (expires_at IS NULL OR expires_at BETWEEN 0 AND 9007199254740991),
    state TEXT NOT NULL CHECK (state IN ('active', 'disabled', 'removed')),
    created_at INTEGER NOT NULL CHECK (created_at BETWEEN 0 AND 9007199254740991),
    updated_at INTEGER NOT NULL CHECK (updated_at BETWEEN 0 AND 9007199254740991),
    version INTEGER NOT NULL CHECK (version BETWEEN 1 AND 9007199254740991)
);

CREATE TABLE auth_portal_routes (
    route_id TEXT PRIMARY KEY CHECK (length(route_id) > 0),
    portal_id TEXT NOT NULL REFERENCES auth_login_portals(portal_id) ON DELETE CASCADE,
    participant_id TEXT,
    origin TEXT,
    deployment_id TEXT REFERENCES auth_deployments(deployment_id) ON DELETE CASCADE,
    priority INTEGER NOT NULL,
    created_at INTEGER NOT NULL CHECK (created_at BETWEEN 0 AND 9007199254740991),
    updated_at INTEGER NOT NULL CHECK (updated_at BETWEEN 0 AND 9007199254740991),
    version INTEGER NOT NULL CHECK (version BETWEEN 1 AND 9007199254740991),
    CHECK (updated_at >= created_at),
    CHECK (participant_id IS NOT NULL OR origin IS NOT NULL OR deployment_id IS NOT NULL)
);
CREATE INDEX auth_portal_routes_selection_idx
    ON auth_portal_routes(priority DESC, route_id);

CREATE TABLE auth_account_flows (
    flow_id TEXT PRIMARY KEY CHECK (length(flow_id) > 0),
    kind TEXT NOT NULL CHECK (kind IN ('admin_account', 'identity_link', 'password_reset')),
    token_hash TEXT NOT NULL UNIQUE CHECK (length(token_hash) = 43),
    target_principal_id TEXT REFERENCES auth_principals(principal_id) ON DELETE CASCADE,
    target_provider_id TEXT,
    return_location TEXT,
    payload_json TEXT NOT NULL CHECK (json_valid(payload_json)),
    state TEXT NOT NULL CHECK (state IN ('pending', 'consumed', 'expired', 'revoked')),
    created_at INTEGER NOT NULL CHECK (created_at BETWEEN 0 AND 9007199254740991),
    expires_at INTEGER NOT NULL CHECK (expires_at BETWEEN 0 AND 9007199254740991),
    consumed_at INTEGER CHECK (consumed_at BETWEEN 0 AND 9007199254740991),
    version INTEGER NOT NULL CHECK (version BETWEEN 1 AND 9007199254740991),
    CHECK (expires_at >= created_at),
    CHECK ((state = 'consumed') = (consumed_at IS NOT NULL))
);
CREATE INDEX auth_account_flows_target_idx
    ON auth_account_flows(target_principal_id, kind, state);

CREATE TABLE auth_provisioned_identities (
    identity_key_id TEXT PRIMARY KEY CHECK (length(identity_key_id) = 43),
    identity_public_key TEXT NOT NULL UNIQUE CHECK (length(identity_public_key) = 43),
    principal_id TEXT NOT NULL REFERENCES auth_principals(principal_id) ON DELETE CASCADE,
    deployment_id TEXT NOT NULL,
    instance_id TEXT NOT NULL,
    kind TEXT NOT NULL CHECK (kind IN ('service', 'device')),
    state TEXT NOT NULL CHECK (state IN ('active', 'revoked')),
    created_at INTEGER NOT NULL CHECK (created_at BETWEEN 0 AND 9007199254740991),
    revoked_at INTEGER CHECK (revoked_at BETWEEN 0 AND 9007199254740991),
    FOREIGN KEY (instance_id, deployment_id)
        REFERENCES auth_instances(instance_id, deployment_id) ON DELETE CASCADE,
    CHECK ((state = 'revoked') = (revoked_at IS NOT NULL))
);

CREATE TABLE auth_device_provisioning_secrets (
    secret_id TEXT PRIMARY KEY CHECK (length(secret_id) > 0),
    instance_id TEXT NOT NULL REFERENCES auth_instances(instance_id) ON DELETE CASCADE,
    secret_hash TEXT NOT NULL UNIQUE CHECK (length(secret_hash) = 43),
    state TEXT NOT NULL CHECK (state IN ('pending', 'consumed', 'expired', 'revoked')),
    created_at INTEGER NOT NULL CHECK (created_at BETWEEN 0 AND 9007199254740991),
    expires_at INTEGER NOT NULL CHECK (expires_at BETWEEN 0 AND 9007199254740991),
    consumed_at INTEGER CHECK (consumed_at BETWEEN 0 AND 9007199254740991),
    version INTEGER NOT NULL CHECK (version BETWEEN 1 AND 9007199254740991),
    CHECK (expires_at >= created_at),
    CHECK ((state = 'consumed') = (consumed_at IS NOT NULL))
);

CREATE TABLE auth_device_activation_reviews (
    review_id TEXT PRIMARY KEY CHECK (length(review_id) > 0),
    principal_id TEXT NOT NULL,
    deployment_id TEXT NOT NULL,
    instance_id TEXT NOT NULL,
    request_digest TEXT NOT NULL CHECK (length(request_digest) = 43),
    payload_json TEXT NOT NULL CHECK (json_valid(payload_json)),
    state TEXT NOT NULL CHECK (state IN ('pending', 'approved', 'rejected', 'cancelled', 'expired')),
    requested_at INTEGER NOT NULL CHECK (requested_at BETWEEN 0 AND 9007199254740991),
    decided_at INTEGER CHECK (decided_at BETWEEN 0 AND 9007199254740991),
    decided_by TEXT,
    reason TEXT,
    version INTEGER NOT NULL CHECK (version BETWEEN 1 AND 9007199254740991),
    FOREIGN KEY (principal_id, deployment_id)
        REFERENCES auth_devices(principal_id, deployment_id) ON DELETE CASCADE,
    FOREIGN KEY (instance_id, deployment_id)
        REFERENCES auth_instances(instance_id, deployment_id) ON DELETE CASCADE,
    CHECK ((decided_at IS NULL) = (decided_by IS NULL)),
    CHECK ((state IN ('approved', 'rejected')) = (decided_at IS NOT NULL))
);

CREATE TABLE auth_idempotency_results (
    scope_key TEXT PRIMARY KEY CHECK (length(scope_key) = 43),
    purpose TEXT NOT NULL CHECK (length(purpose) > 0),
    signer_id TEXT NOT NULL CHECK (length(signer_id) > 0),
    request_id TEXT NOT NULL CHECK (length(request_id) > 0),
    request_digest TEXT NOT NULL CHECK (length(request_digest) = 43),
    result_json TEXT NOT NULL CHECK (json_valid(result_json)),
    created_at INTEGER NOT NULL CHECK (created_at BETWEEN 0 AND 9007199254740991),
    expires_at INTEGER NOT NULL CHECK (expires_at BETWEEN 0 AND 9007199254740991),
    CHECK (expires_at >= created_at),
    UNIQUE (purpose, signer_id, request_id)
);

CREATE TABLE auth_post_commit_actions (
    action_id TEXT PRIMARY KEY CHECK (length(action_id) = 43),
    kind TEXT NOT NULL CHECK (kind IN ('event', 'kick')),
    payload_json TEXT NOT NULL CHECK (json_valid(payload_json)),
    created_at INTEGER NOT NULL CHECK (created_at BETWEEN 0 AND 9007199254740991),
    attempts INTEGER NOT NULL DEFAULT 0 CHECK (attempts >= 0),
    next_attempt_at INTEGER NOT NULL CHECK (next_attempt_at BETWEEN 0 AND 9007199254740991),
    claimed_until INTEGER CHECK (claimed_until BETWEEN 0 AND 9007199254740991),
    last_error TEXT
);
CREATE INDEX auth_post_commit_actions_ready_idx
    ON auth_post_commit_actions(next_attempt_at, action_id);
