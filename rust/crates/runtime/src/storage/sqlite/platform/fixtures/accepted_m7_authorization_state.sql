INSERT INTO auth_principals (
    principal_id, kind, state, created_at, updated_at, version, disabled_at, revoked_at
) VALUES ('dev_m7', 'device', 'active', 1000, 1000, 1, NULL, NULL);

INSERT INTO auth_provider_identities (
    provider, provider_subject, principal_id, linked_at, last_seen_at
) VALUES ('oidc-m7', 'subject-m7', 'dev_m7', 1000, 1000);

INSERT INTO auth_participant_bindings (
    participant_id, participant_kind, artifact_digest, needs_digest,
    participant_json, api_artifacts_json, resolved_at, state, error
) VALUES (
    'participant-m7', 'device',
    'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
    'bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb',
    '{}', '{}', 1000, 'resolved', NULL
);

INSERT INTO auth_sessions (
    session_id, principal_id, principal_kind, participant_id, participant_kind,
    participant_artifact_digest, participant_needs_digest, session_public_key,
    session_key_id, inbox_prefix, state, created_at, last_seen_at, expires_at,
    revoked_at, version
) VALUES (
    'ses_m7', 'dev_m7', 'device', 'participant-m7', 'device',
    'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
    'bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb',
    'ccccccccccccccccccccccccccccccccccccccccccc',
    'ddddddddddddddddddddddddddddddddddddddddddd',
    '_INBOX.ses_m7', 'active', 1000, 1000, 2000, NULL, 1
);

INSERT INTO auth_identity_authorities (
    authority_id, principal_id, participant_id, participant_artifact_digest,
    accepted_needs_digest, desired_grant_set_json, desired_capabilities_json,
    state, version, created_at, updated_at, expires_at, decision_at,
    decision_by, decision_reason
) VALUES (
    'ia_m7', 'dev_m7', 'participant-m7',
    'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
    'bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb',
    '{"permissions":[]}', '[]', 'accepted', 1, 1000, 1000, 2000,
    1000, 'reviewer-m7', 'accepted M7 fixture'
);

INSERT INTO auth_deployments (
    deployment_id, participant_id, participant_kind, state, expires_at
) VALUES ('dep_m7', 'participant-m7', 'device', 'active', 2000);

INSERT INTO auth_deployment_authorities (
    authority_id, deployment_id, participant_id, participant_kind,
    participant_artifact_digest, accepted_needs_digest, desired_grant_set_json,
    desired_capabilities_json, state, version, created_at, updated_at,
    expires_at, decision_at, decision_by, decision_reason
) VALUES (
    'da_m7', 'dep_m7', 'participant-m7', 'device',
    'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
    'bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb',
    '{"permissions":[]}', '[]', 'accepted', 1, 1000, 1000, 2000,
    1000, 'reviewer-m7', 'accepted M7 fixture'
);

INSERT INTO auth_instances (instance_id, deployment_id, principal_id, state)
VALUES ('inst_m7', 'dep_m7', 'dev_m7', 'active');

INSERT INTO auth_session_runtime_bindings (session_id, deployment_id, instance_id)
VALUES ('ses_m7', 'dep_m7', 'inst_m7');

INSERT INTO auth_devices (principal_id, deployment_id, state)
VALUES ('dev_m7', 'dep_m7', 'active');

INSERT INTO auth_device_delegations (
    principal_id, deployment_id, required, state, expires_at
) VALUES ('dev_m7', 'dep_m7', 1, 'active', 2000);

INSERT INTO auth_dependency_evidence (
    authority_kind, authority_id, participant_id, participant_artifact_digest,
    participant_needs_digest, alias, required, api_id, api_digest,
    provider_participant_id, provider_deployment_id, provider_instance_id,
    state, observed_at
) VALUES (
    'deployment', 'da_m7', 'participant-m7',
    'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
    'bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb',
    'dependency-m7', 1, 'example.api@v1',
    'eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee',
    'provider-m7', 'provider-dep-m7', 'provider-inst-m7', 'available', 1000
);

INSERT INTO auth_resource_binding_evidence (
    authority_kind, authority_id, participant_id, participant_artifact_digest,
    participant_needs_digest, resource_kind, local_name, binding_id,
    provider_identity, state, materialized_at, error
) VALUES (
    'deployment', 'da_m7', 'participant-m7',
    'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
    'bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb',
    'kv', 'cache', 'binding-m7', 'kv:cache-m7', 'available', 1000, NULL
);

INSERT INTO auth_materialized_authorities (
    materialization_id, authority_kind, authority_id, authority_version,
    materialization_version, subject_id, participant_id, participant_kind,
    participant_artifact_digest, participant_needs_digest,
    effective_grant_set_json, effective_capabilities_json, state,
    reconciled_at, error, expires_at
) VALUES (
    'mat_m7', 'deployment', 'da_m7', 1, 1, 'dep_m7',
    'participant-m7', 'device',
    'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
    'bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb',
    '{"permissions":[]}', '[]', 'available', 1000, NULL, 2000
);

INSERT INTO auth_materialized_dependencies (
    materialization_id, alias, required, api_id, api_digest,
    provider_participant_id, provider_deployment_id, provider_instance_id,
    state, observed_at
) VALUES (
    'mat_m7', 'dependency-m7', 1, 'example.api@v1',
    'eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee',
    'provider-m7', 'provider-dep-m7', 'provider-inst-m7', 'available', 1000
);

INSERT INTO auth_materialized_resource_bindings (
    materialization_id, resource_kind, local_name, binding_id,
    owner_participant_id, provider_identity, state, materialized_at, error
) VALUES (
    'mat_m7', 'kv', 'cache', 'binding-m7', 'participant-m7',
    'kv:cache-m7', 'available', 1000, NULL
);

INSERT INTO auth_transition_outbox (event_id, transition_json, created_at)
VALUES (
    'fffffffffffffffffffffffffffffffffffffffffff',
    '{"fixture":"accepted-m7"}',
    1000
);
