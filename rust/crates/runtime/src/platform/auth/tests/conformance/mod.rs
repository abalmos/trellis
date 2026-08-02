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
        super::accounts::exercise_accounts(store.clone()).await?;
        super::portals::exercise_portals(store.clone()).await?;
        super::account_flows::exercise_account_flows(store.clone()).await?;
        super::companion_provisioning::exercise_provisioning(store.clone()).await?;
        super::authority_flows::exercise_authority_flows(store).await
    }
}
