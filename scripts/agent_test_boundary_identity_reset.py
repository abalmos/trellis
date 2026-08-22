from pathlib import Path


def replace_once(text: str, old: str, new: str, label: str) -> str:
    count = text.count(old)
    if count != 1:
        raise RuntimeError(f"{label}: expected one match, found {count}")
    return text.replace(old, new, 1)


path = Path("rust/crates/trellis-test/src/lib.rs")
text = path.read_text()

# Stable participant identity is now product identity again. Shared fixture
# processes reuse one administrator across serial tests, so clear authority
# inherited from a previous test before this test first authenticates that
# participant. Keep this entirely in the test harness and use public Auth RPCs.
text = replace_once(
    text,
    "use std::collections::{BTreeMap, HashMap};",
    "use std::collections::{BTreeMap, HashMap, HashSet};",
    "HashSet import",
)
text = replace_once(
    text,
    "    api_artifacts: std::collections::BTreeMap<String, Value>,\n    admin_rpc: Option<AdminRpcProxy>,",
    "    api_artifacts: std::collections::BTreeMap<String, Value>,\n    prepared_identity_participants: HashSet<String>,\n    admin_rpc: Option<AdminRpcProxy>,",
    "admin participant reset set",
)
text = replace_once(
    text,
    "            api_artifacts: builtin_api_artifacts(),\n            admin_rpc: options.admin_rpc,",
    "            api_artifacts: builtin_api_artifacts(),\n            prepared_identity_participants: HashSet::new(),\n            admin_rpc: options.admin_rpc,",
    "admin participant reset initializer",
)

method_anchor = "    /// Complete first-admin bootstrap with the supplied bootstrap URL.\n"
method = '''    async fn reset_shared_identity_authority_once(
        &mut self,
        participant_id: &str,
    ) -> Result<(), TrellisTestError> {
        if self.prepared_identity_participants.contains(participant_id) {
            return Ok(());
        }
        let Some(proxy) = self.admin_rpc.clone() else {
            return Ok(());
        };

        loop {
            let request = auth_sdk::types::AuthIdentityAuthorityListRequest {
                cursor: None,
                limit: Some(100),
                participant_id: Some(participant_id.to_owned()),
                principal_id: None,
                state: Some(auth_sdk::types::AuthIdentityAuthorityListRequestState::Accepted),
            };
            let response: auth_sdk::types::AuthIdentityAuthorityListResponse =
                proxy.call("authIdentityAuthorityList", &request).await?;
            if response.entries.is_empty() {
                break;
            }
            for authority in response.entries {
                let request = auth_sdk::types::AuthIdentityAuthorityRevokeRequest {
                    authority_id: authority.authority_id,
                    expected_version: authority.version,
                    idempotency_key: random_session_seed(),
                    reason: Some("reset by Rust integration fixture".to_owned()),
                };
                let _: auth_sdk::types::AuthIdentityAuthorityRevokeResponse =
                    proxy.call("authIdentityAuthorityRevoke", &request).await?;
            }
        }

        self.prepared_identity_participants
            .insert(participant_id.to_owned());
        Ok(())
    }

'''
text = replace_once(text, method_anchor, method + method_anchor, "identity reset method")

compiled_anchor = "        let compiled = build_test_artifacts(&contract, &mut self.api_artifacts)?;\n"
compiled_insert = '''        let compiled = build_test_artifacts(&contract, &mut self.api_artifacts)?;
        let participant_id = compiled.participant_value()?["id"]
            .as_str()
            .expect("compiled participant has an id")
            .to_owned();
        self.reset_shared_identity_authority_once(&participant_id)
            .await?;
'''
# approve_contract also builds artifacts. Only the user/client flow must reset
# identity authority, so anchor the occurrence after connect_client_with_registration.
function_start = text.find("    async fn connect_client_with_registration(")
if function_start == -1:
    raise RuntimeError("connect_client_with_registration not found")
compiled_at = text.find(compiled_anchor, function_start)
if compiled_at == -1:
    raise RuntimeError("client compiled-artifact anchor not found")
text = text[:compiled_at] + text[compiled_at:].replace(compiled_anchor, compiled_insert, 1)

old_inner = '''                let participant_id = compiled.participant_value()?["id"]
                    .as_str()
                    .expect("compiled participant has an id")
                    .to_owned();
                self.put_portal_grant_override(
'''
text = replace_once(
    text,
    old_inner,
    "                self.put_portal_grant_override(\n",
    "reuse stable client participant id",
)

# The reset is test-harness behavior only and must never become another
# production connection option or protocol identity mutator.
for forbidden in (
    "with_integration_test_scope",
    "IntegrationTestScope",
    "integration_test_scope",
):
    if forbidden in text:
        raise RuntimeError(f"stale test protocol scope remains after identity reset: {forbidden}")

path.write_text(text)
