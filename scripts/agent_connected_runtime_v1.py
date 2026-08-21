from pathlib import Path
import re

path = Path("rust/crates/trellis/src/service/runtime_facade.rs")
text = path.read_text()


def replace_once(old: str, new: str, label: str) -> None:
    global text
    count = text.count(old)
    if count != 1:
        raise RuntimeError(f"{label}: expected one occurrence, found {count}")
    text = text.replace(old, new, 1)


# A value named ConnectedServiceRuntime is always backed by a connected client.
replace_once(
    '''    /// The runtime was built without a client and cannot use the default runner.
    #[error("service runtime is missing a Trellis client")]
    MissingClient,

''',
    "",
    "remove impossible MissingClient error",
)
replace_once(
    "    client: Option<Arc<TrellisClient>>,\n    service_name: Arc<str>,",
    "    client: Arc<TrellisClient>,\n    service_name: Arc<str>,",
    "ServiceHandle client ownership",
)
replace_once(
    '''        crate::generated::Caller::from_client(Arc::clone(
            self.client
                .as_ref()
                .expect("connected service handles always include a Trellis client"),
        ))''',
    "        crate::generated::Caller::from_client(Arc::clone(&self.client))",
    "ServiceHandle caller",
)
replace_once(
    '''        &self
            .client
            .as_ref()
            .expect("connected service handles always include a Trellis client")
            .auth()
            .session_key''',
    "        &self.client.auth().session_key",
    "ServiceHandle session key",
)
replace_once(
    '''    fn client(&self) -> &Arc<TrellisClient> {
        self.client
            .as_ref()
            .expect("connected service handles always include a Trellis client")
    }
''',
    '''    fn client(&self) -> &Arc<TrellisClient> {
        &self.client
    }
''',
    "ServiceHandle client accessor",
)

replace_once(
    '''pub struct ConnectedServiceRuntime<C> {
    client: Option<Arc<TrellisClient>>,
    caller: Option<crate::generated::Caller>,''',
    '''pub struct ConnectedServiceRuntime<C> {
    client: Arc<TrellisClient>,
    caller: crate::generated::Caller,''',
    "ConnectedServiceRuntime ownership",
)

# Test-observation methods can use the invariant directly.
text = text.replace(
    '''        self.client
            .as_ref()
            .expect("connected service client is present")
            .integration_test_authorization_provider()''',
    "        self.client.integration_test_authorization_provider()",
)
text = text.replace(
    '''        self.client
            .as_ref()
            .expect("connected service client is present")
            .integration_test_nats()''',
    "        self.client.integration_test_nats()",
)
text = text.replace(
    '''        self.client
            .as_ref()
            .expect("connected service client is present")
            .request_json_value(subject, input)''',
    "        self.client.request_json_value(subject, input)",
)

replace_once(
    '''            client: Some(client),
            caller: Some(caller),''',
    '''            client,
            caller,''',
    "from_parts connected fields",
)
replace_once(
    '''    pub(crate) fn client(&self) -> &Arc<TrellisClient> {
        self.client
            .as_ref()
            .expect("connected service runtimes always include a Trellis client")
    }
''',
    '''    pub(crate) fn client(&self) -> &Arc<TrellisClient> {
        &self.client
    }
''',
    "runtime client accessor",
)
replace_once(
    '''    pub fn caller(&self) -> &crate::generated::Caller {
        self.caller
            .as_ref()
            .expect("connected service runtimes always include a caller handle")
    }
''',
    '''    pub fn caller(&self) -> &crate::generated::Caller {
        &self.caller
    }
''',
    "runtime caller accessor",
)

# The injected runner and disconnected test runtime existed only to unit-test
# registration plumbing. Normal live integration already exercises the real loop.
start = text.index("    /// Run registered subjects using the default NATS request loop.\n")
end = text.index("    /// Return a cloneable service handle for generated participant code.\n", start)
run_impl = '''    /// Run registered subjects using the authenticated NATS request loop.
    pub async fn run(self) -> Result<(), ServiceRuntimeError> {
        let subjects = self.registered_subjects.into_iter().collect::<Vec<_>>();
        let job_hosts = self.job_hosts;
        let auth = self.auth;
        let host = bootstrap_service_host(
            &self.service_name,
            self.binding.bootstrap_binding(),
            self.router,
            auth,
        );
        let client = self.client;
        let serve = async move {
            if subjects.is_empty() {
                std::future::pending::<()>().await;
            }
            let subject_refs = subjects.iter().map(String::as_str).collect::<Vec<_>>();
            run_multi_subject_service(client.nats().clone(), &subject_refs, host)
                .await
                .map_err(ServiceRuntimeError::from)
        };
        if job_hosts.is_empty() {
            return serve.await;
        }
        let workers = async {
            futures_util::future::try_join_all(job_hosts.into_iter().map(WorkerHostHandle::join))
                .await
                .map_err(ServiceRuntimeError::JobWorker)?;
            Ok(())
        };
        tokio::try_join!(serve, workers)?;
        Ok(())
    }

'''
text = text[:start] + run_impl + text[end:]

replace_once(
    '''        ServiceHandle {
            client: self.client.as_ref().map(Arc::clone),''',
    '''        ServiceHandle {
            client: Arc::clone(&self.client),''',
    "generated handle client",
)

# Delete the constructor that created an impossible disconnected "connected" runtime.
text, count = re.subn(
    r'\n    #\[cfg\(test\)\]\n'
    r'    fn from_test_binding\(service_name: impl Into<String>, binding: CoreBootstrapBinding\) -> Self\n'
    r'    where\n'
    r'        C: GeneratedServiceContract,\n'
    r'    \{[\s\S]*?\n    \}\n',
    "\n",
    text,
    count=1,
)
if count != 1:
    raise RuntimeError(f"from_test_binding: expected one block, found {count}")

# Delete the runner abstraction and EmptyHandler test adapter now that run always
# owns the concrete product transport.
text, count = re.subn(
    r'\n/// Runner seam for tests and alternate service loop implementations\.[\s\S]*?\nfn parse_bootstrap_binding\(',
    "\nfn parse_bootstrap_binding(",
    text,
    count=1,
)
if count != 1:
    raise RuntimeError(f"ServiceRuntimeRunner block: expected one block, found {count}")

# Unit tests that require an impossible disconnected runtime are deleted; small
# pure binding/concurrency tests remain. Live integration owns runtime behavior.
test_start = text.index("    #[derive(Debug, Clone, Serialize, Deserialize)]\n    struct PingInput")
test_end = text.index("    fn binding() -> CoreBootstrapBinding {", test_start)
text = text[:test_start] + text[test_end:]
for name in (
    "registration_records_subjects",
    "watch_operation_registration_records_data_and_control_subjects",
    "resource_binding_accessors_return_typed_resources",
    "run_passes_registered_subjects_to_runner",
    "injected_client_and_binding_path_builds_runtime",
):
    pattern = rf'\n    #\[(?:tokio::)?test\]\n    (?:async )?fn {name}\(\) \{{[\s\S]*?\n    \}}\n'
    text, count = re.subn(pattern, "\n", text, count=1)
    if count != 1:
        raise RuntimeError(f"{name}: expected one test block, found {count}")

text = text.replace("    use futures_util::future::ready;\n", "")
text = text.replace("    use serde::{Deserialize, Serialize};\n", "")
text = text.replace("    use std::sync::Mutex;\n", "")

for token in (
    "MissingClient",
    "ServiceRuntimeRunner",
    "DefaultServiceRunner",
    "RecordingRunner",
    "EmptyHandler",
    "from_test_binding",
    "client: Option<Arc<TrellisClient>>",
    "caller: Option<crate::generated::Caller>",
    "run_with_runner",
):
    if token in text:
        raise RuntimeError(f"stale disconnected-runtime token {token!r}")

path.write_text(text)
