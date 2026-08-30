CREATE TABLE auth_participant_transport_projections (
    participant_id TEXT NOT NULL,
    artifact_digest TEXT NOT NULL,
    needs_digest TEXT NOT NULL,
    projection_json TEXT NOT NULL CHECK (json_valid(projection_json)),
    PRIMARY KEY (participant_id, artifact_digest),
    FOREIGN KEY (participant_id, artifact_digest)
        REFERENCES auth_participant_bindings(participant_id, artifact_digest)
        ON DELETE CASCADE
);

ALTER TABLE auth_participant_bindings
    ADD COLUMN transport_projection_digest TEXT;

ALTER TABLE auth_authorization_contexts
    ADD COLUMN transport_permissions_json TEXT NOT NULL
        DEFAULT '{"publish":[],"subscribe":[]}';

DELETE FROM auth_post_commit_actions
WHERE kind = 'context_publish'
  AND json_extract(payload_json, '$.contextDigest') IN (
      SELECT context_digest FROM auth_authorization_contexts
  );

DELETE FROM auth_idempotency_results
WHERE purpose = 'authorizationContextIssue';

UPDATE auth_authorization_contexts
SET state = 'revoked',
    revoked_at = unixepoch(),
    revocation_reason = 'context_replaced',
    version = version + 1
WHERE state = 'active';

INSERT OR IGNORE INTO auth_post_commit_actions (
    action_id,
    kind,
    payload_json,
    created_at,
    attempts,
    next_attempt_at,
    claimed_until,
    last_error,
    predecessor_action_id
)
SELECT
    context_digest,
    'context_revoke',
    json_object(
        'format', 'trellis.authorization-context-revoke-action.v1',
        'contextDigest', context_digest,
        'reason', revocation_reason,
        'version', version
    ),
    unixepoch(),
    0,
    unixepoch(),
    NULL,
    NULL,
    NULL
FROM auth_authorization_contexts
WHERE revoked_at IS NOT NULL
  AND NOT EXISTS (
      SELECT 1
      FROM auth_post_commit_actions AS existing
      WHERE existing.kind = 'context_revoke'
        AND json_extract(existing.payload_json, '$.contextDigest') = auth_authorization_contexts.context_digest
  );
