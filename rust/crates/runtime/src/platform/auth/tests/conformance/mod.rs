pub(super) mod account_flows;
pub(super) mod accounts;
mod authority;
pub(super) mod authority_flows;
pub(super) mod companion_provisioning;
mod deployments;
mod evidence;
mod fixtures;
mod outbox;
pub(super) mod portals;
mod provisioning;
mod sessions;

pub(super) mod companion {
    use crate::platform::auth::SqliteAuthorizationStore;

    pub(super) async fn exercise_companion_repositories(
        store: SqliteAuthorizationStore,
    ) -> Result<(), Box<dyn std::error::Error>> {
        super::accounts::exercise_accounts(store.clone())
            .await
            .map_err(|error| format!("accounts: {error}"))?;
        super::portals::exercise_portals(store.clone())
            .await
            .map_err(|error| format!("portals: {error}"))?;
        super::account_flows::exercise_account_flows(store.clone())
            .await
            .map_err(|error| format!("account flows: {error}"))?;
        super::companion_provisioning::exercise_provisioning(store.clone())
            .await
            .map_err(|error| format!("provisioning: {error}"))?;
        super::authority_flows::exercise_authority_flows(store)
            .await
            .map_err(|error| format!("authority flows: {error}").into())
    }
}
