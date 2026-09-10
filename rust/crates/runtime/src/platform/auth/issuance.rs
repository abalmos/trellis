use trellis_protocol::{
    AuthorizationPrincipalKind, GrantSet, ParticipantKind, ParticipantResourceKind,
    PermissionTarget,
};

use super::authority::{IssuanceCredentialRecord, IssuanceSnapshot};
use super::{
    AuthorizationStateError, DeviceDelegationState, DeviceState, GrantBindingState, GrantOwnerKind,
    IssuableAuthorizationState, PrincipalKind, PrincipalState, ProvisionedIdentityKind,
    ProvisionedIdentityState, ResourceBindingState, RuntimeInstanceState, SessionState,
};

pub(super) fn resolve_snapshot(
    snapshot: IssuanceSnapshot,
    now: i64,
) -> Result<IssuableAuthorizationState, AuthorizationStateError> {
    super::domain::require_protocol_timestamp("now", now)?;
    let IssuanceSnapshot {
        connection,
        credential,
        principal,
        binding,
        participant,
        resources,
        issuer,
    } = snapshot;
    if principal.state != PrincipalState::Active {
        return Err(AuthorizationStateError::PrincipalInactive);
    }
    if issuer.state != trellis_protocol::AuthorizationIssuerState::Active {
        return Err(AuthorizationStateError::IssuerMissing);
    }
    if binding.state != GrantBindingState::Active
        || binding.expires_at.is_some_and(|expiry| now >= expiry)
    {
        return Err(AuthorizationStateError::NotAuthorized);
    }
    if participant.participant_id != binding.participant_id {
        return Err(AuthorizationStateError::ParticipantMissing);
    }
    let mut expiries = vec![binding.expires_at];
    let (principal_kind, login_session_id, identity_key_id, deployment_id, instance_id) =
        match credential {
            IssuanceCredentialRecord::Login(login) => {
                if principal.kind != PrincipalKind::User
                    || login.principal_id != principal.principal_id
                    || login.participant_id != participant.participant_id
                    || login.participant_kind != participant.participant_kind
                    || !matches!(
                        participant.participant_kind,
                        ParticipantKind::App | ParticipantKind::Agent
                    )
                    || binding.owner_kind != GrantOwnerKind::User
                    || binding.owner_id != principal.principal_id
                    || connection.session_public_key != login.session_public_key
                {
                    return Err(AuthorizationStateError::NotAuthorized);
                }
                match login.state {
                    SessionState::Active => {}
                    SessionState::Expired => return Err(AuthorizationStateError::SessionExpired),
                    SessionState::Revoked => return Err(AuthorizationStateError::SessionRevoked),
                }
                if login.expires_at.is_some_and(|expiry| now >= expiry) {
                    return Err(AuthorizationStateError::SessionExpired);
                }
                expiries.push(login.expires_at);
                (
                    AuthorizationPrincipalKind::User,
                    Some(login.session_id),
                    None,
                    None,
                    None,
                )
            }
            IssuanceCredentialRecord::Native {
                identity,
                instance,
                deployment,
                device,
                delegation,
            } => {
                if identity.state != ProvisionedIdentityState::Active
                    || identity.revoked_at.is_some()
                {
                    return Err(AuthorizationStateError::IdentityMissing);
                }
                if identity.principal_id != principal.principal_id
                    || identity.instance_id != instance.instance_id
                    || identity.deployment_id != deployment.deployment_id
                    || instance.principal_id != principal.principal_id
                    || instance.deployment_id != deployment.deployment_id
                    || deployment.participant_id != participant.participant_id
                    || deployment.participant_kind != participant.participant_kind
                    || binding.owner_kind != GrantOwnerKind::Deployment
                    || binding.owner_id != deployment.deployment_id
                {
                    return Err(AuthorizationStateError::NotAuthorized);
                }
                if instance.state != RuntimeInstanceState::Active {
                    return Err(AuthorizationStateError::InstanceInactive);
                }
                if !deployment.active || deployment.expires_at.is_some_and(|expiry| now >= expiry) {
                    return Err(AuthorizationStateError::DeploymentInactive);
                }
                expiries.push(deployment.expires_at);
                let kind = match (principal.kind, identity.kind, participant.participant_kind) {
                    (
                        PrincipalKind::Service,
                        ProvisionedIdentityKind::Service,
                        ParticipantKind::Service,
                    ) => AuthorizationPrincipalKind::Service,
                    (
                        PrincipalKind::Device,
                        ProvisionedIdentityKind::Device,
                        ParticipantKind::Device,
                    ) => {
                        let device = device.ok_or(AuthorizationStateError::DeviceInactive)?;
                        if device.state != DeviceState::Active
                            || device.principal_id != principal.principal_id
                            || device.deployment_id != deployment.deployment_id
                        {
                            return Err(AuthorizationStateError::DeviceInactive);
                        }
                        if let Some(delegation) = delegation {
                            if delegation.principal_id != principal.principal_id
                                || delegation.deployment_id != deployment.deployment_id
                                || (delegation.required
                                    && delegation.state != DeviceDelegationState::Active)
                            {
                                return Err(AuthorizationStateError::ActivationMissing);
                            }
                            if delegation.required
                                && delegation.expires_at.is_some_and(|expiry| now >= expiry)
                            {
                                return Err(AuthorizationStateError::DelegationExpired);
                            }
                            expiries.push(delegation.expires_at);
                        }
                        AuthorizationPrincipalKind::Device
                    }
                    _ => return Err(AuthorizationStateError::WrongPrincipalKind),
                };
                (
                    kind,
                    None,
                    Some(identity.identity_key_id),
                    Some(deployment.deployment_id),
                    Some(instance.instance_id),
                )
            }
        };
    let resolved = participant.resolve()?;
    let required = resolved.required_grants.permissions();
    let allowed = required
        .iter()
        .chain(
            resolved
                .optional_grant_bundles
                .values()
                .flat_map(|grant| grant.permissions()),
        )
        .collect::<Vec<_>>();
    let mut selected_permissions = Vec::new();
    let mut selected_resources = Vec::new();
    for permission in binding
        .grants
        .permissions()
        .iter()
        .filter(|permission| allowed.contains(permission))
    {
        let PermissionTarget::ParticipantResource {
            participant: owner,
            resource,
            name,
        } = permission.target()
        else {
            selected_permissions.push(permission.clone());
            continue;
        };
        let kind = match resource {
            ParticipantResourceKind::Kv => "kv",
            ParticipantResourceKind::Store => "store",
            ParticipantResourceKind::JobQueue => "jobQueue",
            ParticipantResourceKind::EventConsumer => "eventConsumer",
            ParticipantResourceKind::State => "state",
        };
        let evidence = resources
            .iter()
            .find(|evidence| {
                evidence.resource_kind == kind
                    && evidence.local_name == *name
                    && evidence.owner_participant_id == *owner
                    && evidence.state == ResourceBindingState::Available
            })
            .cloned();
        let Some(evidence) = evidence else {
            if required.contains(permission) {
                return Err(AuthorizationStateError::RequiredResourceUnavailable(
                    format!("{kind}:{name}"),
                ));
            }
            continue;
        };
        selected_permissions.push(permission.clone());
        if !selected_resources.contains(&evidence) {
            selected_resources.push(evidence);
        }
    }
    let grant_set = GrantSet::new(selected_permissions);
    let session_key_id =
        super::domain::validate_ed25519_public_key("sessionKey", &connection.session_public_key)?;
    // Installation keys can be shared by tabs; inboxes belong to logical connections.
    let inbox_prefix = format!("_INBOX.{}", connection.connection_id);
    Ok(IssuableAuthorizationState {
        principal_id: principal.principal_id,
        principal_kind,
        connection_id: connection.connection_id,
        login_session_id,
        identity_key_id,
        session_public_key: connection.session_public_key,
        session_key_id,
        inbox_prefix,
        participant,
        binding,
        deployment_id,
        instance_id,
        grant_set,
        resource_bindings: selected_resources,
        expires_at: expiries.into_iter().flatten().min(),
    })
}
