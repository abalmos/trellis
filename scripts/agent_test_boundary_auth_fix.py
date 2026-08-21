from pathlib import Path
import re

# Stable protocol identities are intentional. Reset only durable test-owned
# portal policy state between serial auth cases.
path = Path("rust/crates/trellis-test/src/lib.rs")
text = path.read_text()
old = '''    /// Create one participant-scoped login portal and route through public Auth RPCs.
    pub async fn put_test_login_portal(
        &mut self,
        bootstrap_url: &str,
        portal_id: &str,
        participant_id: &str,
        providers: Vec<String>,
    ) -> Result<(), TrellisTestError> {
        self.complete_bootstrap(bootstrap_url).await?;
        let portal = auth_sdk::types::AuthPortalsPutRequest {
            disabled: false,
            display_name: format!("Live test portal {portal_id}"),
            entry_url: None,
            expected_version: None,
            idempotency_key: random_session_seed(),
            login_settings: auth_sdk::types::AuthPortalsPutRequestLoginSettings {
                federated_registration: true,
                local_login: true,
                local_registration: true,
                providers: Some(providers),
            },
            portal_id: portal_id.to_owned(),
        };
        let route = auth_sdk::types::AuthPortalsRoutesPutRequest {
            deployment_id: None,
            expected_version: None,
            idempotency_key: random_session_seed(),
            origin: None,
            participant_id: Some(participant_id.to_owned()),
            portal_id: portal_id.to_owned(),
            priority: 100,
            route_id: None,
        };
        if let Some(proxy) = &self.admin_rpc {
            let _: Value = proxy.call("authPortalsPut", &portal).await?;
            let _: Value = proxy.call("authPortalsRoutesPut", &route).await?;
        } else {
            let client = GeneratedAuthClient::new(self.connect_admin(bootstrap_url).await?);
            client.rpc().auth().portals_put(&portal).await?;
            client.rpc().auth().portals_routes_put(&route).await?;
        }
        Ok(())
    }
'''
new = '''    /// Reset one participant-scoped login portal and stable route through public Auth RPCs.
    pub async fn put_test_login_portal(
        &mut self,
        bootstrap_url: &str,
        portal_id: &str,
        participant_id: &str,
        providers: Vec<String>,
    ) -> Result<(), TrellisTestError> {
        self.complete_bootstrap(bootstrap_url).await?;
        let list = auth_sdk::types::AuthPortalsListRequest {
            cursor: None,
            disabled: None,
            limit: Some(100),
        };
        let current: auth_sdk::types::AuthPortalsListResponse = if let Some(proxy) = &self.admin_rpc {
            proxy.call("authPortalsList", &list).await?
        } else {
            GeneratedAuthClient::new(self.connect_admin(bootstrap_url).await?)
                .rpc()
                .auth()
                .portals_list(&list)
                .await?
        };
        let expected_version = current
            .entries
            .iter()
            .find(|entry| entry.portal_id == portal_id)
            .map(|entry| entry.version);
        let portal = auth_sdk::types::AuthPortalsPutRequest {
            disabled: false,
            display_name: format!("Live test portal {portal_id}"),
            entry_url: None,
            expected_version,
            idempotency_key: random_session_seed(),
            login_settings: auth_sdk::types::AuthPortalsPutRequestLoginSettings {
                federated_registration: true,
                local_login: true,
                local_registration: true,
                providers: Some(providers),
            },
            portal_id: portal_id.to_owned(),
        };
        if let Some(proxy) = &self.admin_rpc {
            let _: Value = proxy.call("authPortalsPut", &portal).await?;
        } else {
            GeneratedAuthClient::new(self.connect_admin(bootstrap_url).await?)
                .rpc()
                .auth()
                .portals_put(&portal)
                .await?;
        }

        let get = auth_sdk::types::AuthPortalsGetRequest {
            portal_id: portal_id.to_owned(),
        };
        let current: auth_sdk::types::AuthPortalsGetResponse = if let Some(proxy) = &self.admin_rpc {
            proxy.call("authPortalsGet", &get).await?
        } else {
            GeneratedAuthClient::new(self.connect_admin(bootstrap_url).await?)
                .rpc()
                .auth()
                .portals_get(&get)
                .await?
        };
        let has_route = current.routes.iter().any(|route| {
            route.participant_id.as_deref() == Some(participant_id)
                && route.origin.is_none()
                && route.deployment_id.is_none()
                && route.priority == 100
        });
        if !has_route {
            let route = auth_sdk::types::AuthPortalsRoutesPutRequest {
                deployment_id: None,
                expected_version: None,
                idempotency_key: random_session_seed(),
                origin: None,
                participant_id: Some(participant_id.to_owned()),
                portal_id: portal_id.to_owned(),
                priority: 100,
                route_id: None,
            };
            if let Some(proxy) = &self.admin_rpc {
                let _: Value = proxy.call("authPortalsRoutesPut", &route).await?;
            } else {
                GeneratedAuthClient::new(self.connect_admin(bootstrap_url).await?)
                    .rpc()
                    .auth()
                    .portals_routes_put(&route)
                    .await?;
            }
        }

        let list = auth_sdk::types::AuthPortalsGrantOverridesListRequest {
            limit: 100,
            offset: None,
            participant_id: Some(participant_id.to_owned()),
            portal_id: Some(portal_id.to_owned()),
        };
        let overrides: auth_sdk::types::AuthPortalsGrantOverridesListResponse =
            if let Some(proxy) = &self.admin_rpc {
                proxy.call("authPortalsGrantOverridesList", &list).await?
            } else {
                GeneratedAuthClient::new(self.connect_admin(bootstrap_url).await?)
                    .rpc()
                    .auth()
                    .portals_grant_overrides_list(&list)
                    .await?
            };
        if let Some(existing) = overrides.entries.into_iter().next() {
            let remove = auth_sdk::types::AuthPortalsGrantOverridesRemoveRequest {
                expected_version: existing.version,
                idempotency_key: random_session_seed(),
                participant_id: participant_id.to_owned(),
                portal_id: portal_id.to_owned(),
            };
            if let Some(proxy) = &self.admin_rpc {
                let _: Value = proxy.call("authPortalsGrantOverridesRemove", &remove).await?;
            } else {
                GeneratedAuthClient::new(self.connect_admin(bootstrap_url).await?)
                    .rpc()
                    .auth()
                    .portals_grant_overrides_remove(&remove)
                    .await?;
            }
        }
        Ok(())
    }
'''
if text.count(old) != 1:
    raise RuntimeError(f"expected one test portal helper, found {text.count(old)}")
path.write_text(text.replace(old, new, 1))

# The auth fixture no longer needs any case-specific contract identity. Directly
# use the authored contract IDs/digests and the real platform event subjects.
path = Path("rust/crates/trellis/tests/integration/auth.rs")
text = path.read_text()
old = '''    let expected_contract = fixture
        .runtime
        .scoped_contract(&fixture.client_contract)
        .expect("scope inventoried participant");
    let expected_needs_digest = expected_contract.needs_digest();
'''
new = '''    let expected_needs_digest = fixture.client_contract.needs_digest();
'''
if text.count(old) != 1:
    raise RuntimeError(f"expected one inventoried scoped contract, found {text.count(old)}")
text = text.replace(old, new, 1)

pattern = re.compile(
    r'''(?:fixture\s*\.\s*runtime|runtime)\s*\.\s*scoped_contract\(\s*&(?P<contract>(?:fixture\.)?[A-Za-z_][A-Za-z0-9_]*)\s*\)\s*\.\s*expect\("[^"]*"\)''',
    re.MULTILINE,
)
text, count = pattern.subn(lambda match: match.group("contract"), text)
if count != 8:
    raise RuntimeError(f"expected eight remaining scoped contract uses, found {count}")

for value in ("device.approval.deployment_id", "no_review.approval.deployment_id"):
    old = f'''fixture
        .runtime
        .integration_test_descriptor_subject(&format!(
            "events.v1.Auth.DeviceUserAuthorities.*.{{}}",
            {value},
        ))'''
    new = f'''format!(
        "events.v1.Auth.DeviceUserAuthorities.*.{{}}",
        {value},
    )'''
    if text.count(old) != 1:
        raise RuntimeError(f"expected one scoped activation event subject for {value}")
    text = text.replace(old, new, 1)

for token in ("scoped_contract", "integration_test_descriptor_subject"):
    if token in text:
        raise RuntimeError(f"stale auth fixture scoping token {token!r}")

path.write_text(text)
