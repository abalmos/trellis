from pathlib import Path
import re

# The private test harness should connect services through the same public
# ServiceConnectOptions path as production code, not a generated raw escape hatch.
path = Path("rust/crates/trellis-test/src/lib.rs")
text = path.read_text()
old = '''pub async fn connect_service_runtime<C>(
    trellis_url: &str,
    key: &TrellisTestServiceKey,
) -> Result<trellis_rs::service::ConnectedServiceRuntime<C>, trellis_rs::service::ServiceRuntimeError>
{
    let session_seed = random_session_seed();
    trellis_rs::generated::test_connect_service_runtime(
        trellis_url,
        &key.participant_id,
        &key.participant_digest,
        &key.participant_json,
        &key.api_json,
        &key.api_digest,
        &key.referenced_api_artifacts
            .iter()
            .map(|(json, digest)| (json.as_str(), digest.as_str()))
            .collect::<Vec<_>>(),
        &key.deployment_id,
        &key.instance_id,
        &key.identity_seed,
        &key.participant_needs_digest,
        &session_seed,
        key.integration_test_scope.clone(),
    )
    .await
}
'''
new = '''pub async fn connect_service_runtime<C>(
    trellis_url: &str,
    key: &TrellisTestServiceKey,
) -> Result<trellis_rs::service::ConnectedServiceRuntime<C>, trellis_rs::service::ServiceRuntimeError>
{
    let referenced_api_artifacts = key
        .referenced_api_artifacts
        .iter()
        .map(|(json, digest)| (json.as_str(), digest.as_str()))
        .collect::<Vec<_>>();
    let session_seed = random_session_seed();
    let options = trellis_rs::service::ServiceConnectOptions::new(
        trellis_url,
        &key.instance_id,
        &key.deployment_id,
        &key.participant_id,
        &key.participant_digest,
        &key.participant_needs_digest,
        &key.participant_json,
        &key.api_json,
        &key.api_digest,
        &referenced_api_artifacts,
        &key.identity_seed,
        &session_seed,
        Arc::new(trellis_rs::client::MemoryAuthorizationContextStore::default()),
    )
    .with_timeout_ms(30_000);
    let options = match key.integration_test_scope.clone() {
        Some(scope) => options.with_integration_test_scope(scope),
        None => options,
    };
    trellis_rs::service::ConnectedServiceRuntime::<C>::connect(options).await
}
'''
if text.count(old) != 1:
    raise RuntimeError(f"trellis-test service connector changed: found {text.count(old)}")
path.write_text(text.replace(old, new, 1))

# The generated test connector and injected post-bootstrap constructor are now dead.
path = Path("rust/crates/trellis/src/generated.rs")
text = path.read_text()
marker = "/// Connect an ad hoc generated service runtime for Trellis integration tests.\n"
if text.count(marker) != 1:
    raise RuntimeError(f"generated test connector marker changed: found {text.count(marker)}")
path.write_text(text[: text.index(marker)].rstrip() + "\n")

path = Path("rust/crates/trellis/src/service/runtime_facade.rs")
text = path.read_text()
text, count = re.subn(
    r'\n    /// Build a connected runtime from a service client that already completed bootstrap\.\n'
    r'    #\[cfg\(feature = "test-support"\)\]\n'
    r'    pub\(crate\) fn from_connected_client\([\s\S]*?\n    \}\n',
    "\n",
    text,
    count=1,
)
if count != 1:
    raise RuntimeError(f"from_connected_client: expected one block, found {count}")
if "from_connected_client" in text:
    raise RuntimeError("from_connected_client survived service cleanup")
path.write_text(text)

for candidate in (
    Path("rust/crates/trellis/src/generated.rs"),
    Path("rust/crates/trellis-test/src/lib.rs"),
):
    if "test_connect_service_runtime" in candidate.read_text():
        raise RuntimeError(f"raw generated service connector survived in {candidate}")
