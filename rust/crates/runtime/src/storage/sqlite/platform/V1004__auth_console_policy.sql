PRAGMA foreign_keys = ON;

ALTER TABLE auth_post_commit_actions
ADD COLUMN predecessor_action_id TEXT;
CREATE INDEX auth_post_commit_actions_predecessor
ON auth_post_commit_actions(predecessor_action_id);

ALTER TABLE auth_deployment_profiles ADD COLUMN review_mode TEXT;
UPDATE auth_deployment_profiles
SET review_mode = 'none'
WHERE kind = 'device';
CREATE TRIGGER auth_deployment_profiles_review_mode_insert
BEFORE INSERT ON auth_deployment_profiles
WHEN (NEW.kind = 'device' AND (NEW.review_mode IS NULL OR NEW.review_mode NOT IN ('none', 'required')))
    OR (NEW.kind = 'service' AND NEW.review_mode IS NOT NULL)
BEGIN
    SELECT RAISE(ABORT, 'deployment review policy is invalid');
END;
CREATE TRIGGER auth_deployment_profiles_review_mode_update
BEFORE UPDATE OF kind, review_mode ON auth_deployment_profiles
WHEN (NEW.kind = 'device' AND (NEW.review_mode IS NULL OR NEW.review_mode NOT IN ('none', 'required')))
    OR (NEW.kind = 'service' AND NEW.review_mode IS NOT NULL)
BEGIN
    SELECT RAISE(ABORT, 'deployment review policy is invalid');
END;

CREATE TABLE auth_device_activation_reviews_v1004 (
    review_id TEXT PRIMARY KEY CHECK (length(review_id) > 0),
    principal_id TEXT NOT NULL,
    deployment_id TEXT NOT NULL,
    instance_id TEXT NOT NULL,
    request_digest TEXT NOT NULL CHECK (length(request_digest) = 43),
    payload_json TEXT NOT NULL CHECK (json_valid(payload_json)),
    state TEXT NOT NULL CHECK (state IN ('pending', 'approved', 'rejected', 'expired')),
    requested_at INTEGER NOT NULL CHECK (requested_at BETWEEN 0 AND 9007199254740991),
    expires_at INTEGER NOT NULL CHECK (expires_at BETWEEN 0 AND 9007199254740991),
    activated_by_user_principal_id TEXT,
    decided_at INTEGER CHECK (decided_at BETWEEN 0 AND 9007199254740991),
    decided_by TEXT,
    reason TEXT,
    version INTEGER NOT NULL CHECK (version BETWEEN 1 AND 9007199254740991),
    FOREIGN KEY (principal_id, deployment_id)
        REFERENCES auth_devices(principal_id, deployment_id) ON DELETE CASCADE,
    FOREIGN KEY (instance_id, deployment_id)
        REFERENCES auth_instances(instance_id, deployment_id) ON DELETE CASCADE,
    CHECK (expires_at >= requested_at),
    CHECK ((decided_at IS NULL) = (decided_by IS NULL)),
    CHECK (state NOT IN ('approved', 'rejected') OR decided_at IS NOT NULL),
    CHECK (state != 'pending' OR decided_at IS NULL)
);
INSERT INTO auth_device_activation_reviews_v1004 (
    review_id,
    principal_id,
    deployment_id,
    instance_id,
    request_digest,
    payload_json,
    state,
    requested_at,
    expires_at,
    activated_by_user_principal_id,
    decided_at,
    decided_by,
    reason,
    version
)
SELECT
    review_id,
    principal_id,
    deployment_id,
    instance_id,
    request_digest,
    payload_json,
    CASE state WHEN 'cancelled' THEN 'expired' ELSE state END,
    requested_at,
    CAST(json_extract(payload_json, '$.expiresAt') AS INTEGER),
    NULL,
    decided_at,
    decided_by,
    reason,
    version
FROM auth_device_activation_reviews;
DROP TABLE auth_device_activation_reviews;
ALTER TABLE auth_device_activation_reviews_v1004 RENAME TO auth_device_activation_reviews;
CREATE INDEX auth_device_activation_reviews_state_expiry
ON auth_device_activation_reviews(state, expires_at);

CREATE TABLE auth_capability_groups (
    group_key TEXT PRIMARY KEY CHECK (length(group_key) > 0),
    display_name TEXT NOT NULL CHECK (length(display_name) > 0),
    description TEXT NOT NULL CHECK (length(description) > 0),
    capabilities_json TEXT NOT NULL CHECK (json_valid(capabilities_json)),
    included_groups_json TEXT NOT NULL CHECK (json_valid(included_groups_json)),
    created_at INTEGER NOT NULL CHECK (created_at BETWEEN 0 AND 9007199254740991),
    updated_at INTEGER NOT NULL CHECK (updated_at BETWEEN 0 AND 9007199254740991),
    version INTEGER NOT NULL CHECK (version BETWEEN 1 AND 9007199254740991)
);

CREATE TABLE auth_portal_grant_overrides (
    portal_id TEXT NOT NULL REFERENCES auth_login_portals(portal_id) ON DELETE CASCADE,
    participant_id TEXT NOT NULL,
    direct_capabilities_json TEXT NOT NULL CHECK (json_valid(direct_capabilities_json)),
    capability_group_keys_json TEXT NOT NULL CHECK (json_valid(capability_group_keys_json)),
    role_mappings_json TEXT NOT NULL CHECK (json_valid(role_mappings_json)),
    created_at INTEGER NOT NULL CHECK (created_at BETWEEN 0 AND 9007199254740991),
    updated_at INTEGER NOT NULL CHECK (updated_at BETWEEN 0 AND 9007199254740991),
    version INTEGER NOT NULL CHECK (version BETWEEN 1 AND 9007199254740991),
    PRIMARY KEY (portal_id, participant_id)
);

CREATE TABLE auth_portal_authority_bindings (
    principal_id TEXT NOT NULL REFERENCES auth_principals(principal_id) ON DELETE CASCADE,
    participant_id TEXT NOT NULL,
    authority_id TEXT NOT NULL REFERENCES auth_identity_authorities(authority_id) ON DELETE CASCADE,
    portal_id TEXT NOT NULL REFERENCES auth_login_portals(portal_id) ON DELETE CASCADE,
    provider_id TEXT NOT NULL CHECK (length(provider_id) > 0),
    roles_json TEXT NOT NULL CHECK (json_valid(roles_json)),
    effective_policy_digest TEXT NOT NULL CHECK (length(effective_policy_digest) = 43),
    authority_version INTEGER NOT NULL CHECK (authority_version BETWEEN 1 AND 9007199254740991),
    updated_at INTEGER NOT NULL CHECK (updated_at BETWEEN 0 AND 9007199254740991),
    PRIMARY KEY (principal_id, participant_id)
);
CREATE INDEX auth_portal_authority_bindings_portal_idx
    ON auth_portal_authority_bindings(portal_id, participant_id);
