PRAGMA foreign_keys = ON;

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
