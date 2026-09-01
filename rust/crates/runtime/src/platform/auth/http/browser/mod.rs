mod consent;
mod local;
mod oidc;
mod request;

pub(super) use consent::{bind_flow, decide_approval};
#[cfg(test)]
pub(super) use consent::{ApprovalRequest, BindRequest};
pub(super) use local::{
    complete_admin_account, get_account_flow, get_flow, local_login, register_local,
    BrowserFlowResponse,
};
#[cfg(test)]
pub(super) use local::{AdminAccountRequest, LocalRegistrationRequest};
pub(super) use oidc::{oidc_callback, start_account_flow_oidc, start_oidc};
#[cfg(test)]
pub(super) use request::AuthStartRequest;
pub(super) use request::{
    console_index, console_page, portal_asset, portal_index, portal_page, select_device_portal,
    start_auth,
};
