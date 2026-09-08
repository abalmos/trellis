mod consent;
mod local;
mod oidc;
mod request;

pub(super) fn complete_participant_grants(
    binding: &crate::platform::auth::ParticipantBindingRecord,
) -> Result<trellis_protocol::GrantSet, super::HttpError> {
    let resolved = binding.resolve()?;
    let proposal = resolved.proposal();
    Ok(trellis_protocol::GrantSet::new(
        proposal
            .required()
            .grant_set()
            .permissions()
            .iter()
            .chain(proposal.optional().grant_set().permissions())
            .cloned()
            .collect(),
    ))
}

pub(super) use consent::{bind_flow, decide_approval};
pub(super) use local::{
    complete_admin_account, get_account_flow, get_flow, get_portal_flow, local_login,
    register_local, BrowserFlowResponse,
};
pub(super) use oidc::{oidc_callback, start_account_flow_oidc, start_oidc};
pub(super) use request::{
    console_index, console_page, portal_asset, portal_index, portal_page, select_device_portal,
    start_auth, web_fallback,
};
