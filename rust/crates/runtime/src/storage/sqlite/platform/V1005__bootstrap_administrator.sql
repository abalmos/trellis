CREATE TABLE auth_bootstrap_administrator (
    singleton INTEGER PRIMARY KEY CHECK (singleton = 1),
    principal_id TEXT NOT NULL UNIQUE REFERENCES auth_principals(principal_id) ON DELETE RESTRICT,
    created_at INTEGER NOT NULL CHECK (created_at BETWEEN 0 AND 9007199254740991)
);

INSERT INTO auth_bootstrap_administrator (singleton, principal_id, created_at)
SELECT 1, principal_id, created_at
FROM auth_identity_authorities
WHERE participant_id = 'trellis-platform-administration'
ORDER BY
    CASE WHEN decision_by IN ('bootstrap', 'system:first-admin') THEN 0 ELSE 1 END,
    created_at,
    authority_id
LIMIT 1;

ALTER TABLE auth_account_flows RENAME TO auth_account_flows_v1004;

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

INSERT INTO auth_account_flows (
    flow_id, kind, token_hash, target_principal_id, target_provider_id,
    return_location, payload_json, state, created_at, expires_at, consumed_at, version
)
SELECT
    flow_id,
    CASE kind WHEN 'first_admin' THEN 'admin_account' ELSE kind END,
    token_hash, target_principal_id, target_provider_id,
    return_location, payload_json, state, created_at, expires_at, consumed_at, version
FROM auth_account_flows_v1004;

DROP TABLE auth_account_flows_v1004;

CREATE INDEX auth_account_flows_target_idx
    ON auth_account_flows(target_principal_id, kind, state);
