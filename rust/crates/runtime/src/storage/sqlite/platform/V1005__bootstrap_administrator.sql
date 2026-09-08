CREATE TABLE auth_bootstrap_administrator (
    singleton INTEGER PRIMARY KEY CHECK (singleton = 1),
    principal_id TEXT NOT NULL UNIQUE REFERENCES auth_principals(principal_id) ON DELETE RESTRICT,
    created_at INTEGER NOT NULL CHECK (created_at BETWEEN 0 AND 9007199254740991)
);
