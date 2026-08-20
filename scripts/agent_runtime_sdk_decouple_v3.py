from pathlib import Path


def replace_once(path: str, old: str, new: str) -> None:
    p = Path(path)
    text = p.read_text()
    count = text.count(old)
    if count != 1:
        raise RuntimeError(f"{path}: expected one occurrence, found {count}: {old[:120]!r}")
    p.write_text(text.replace(old, new, 1))


def remove_between(path: str, start: str, end: str) -> None:
    p = Path(path)
    text = p.read_text()
    if text.count(start) != 1 or text.count(end) != 1:
        raise RuntimeError(f"{path}: helper boundary changed")
    before, rest = text.split(start, 1)
    _, after = rest.split(end, 1)
    p.write_text(before + end + after)


# Generated SDKs are build outputs, not workspace prerequisites. None of the
# root workspace dependency declarations are needed after runtime stops importing
# the generated Auth crate.
root = Path("rust/Cargo.toml")
text = root.read_text()
for line in [
    'trellis-sdk-auth = { path = "../generated/packages/cargo/auth", version = "0.12.0" }\n',
    'trellis-sdk-core = { path = "../generated/packages/cargo/trellis-core", version = "0.12.0" }\n',
    'trellis-sdk-health = { path = "../generated/packages/cargo/health", version = "0.12.0" }\n',
    'trellis-sdk-jobs = { path = "../generated/packages/cargo/jobs", version = "0.12.0" }\n',
    'trellis-sdk-state = { path = "../generated/packages/cargo/state", version = "0.12.0" }\n',
]:
    if text.count(line) != 1:
        raise RuntimeError(f"rust/Cargo.toml: expected generated SDK dependency {line!r}")
    text = text.replace(line, "")
root.write_text(text)

replace_once(
    "rust/crates/runtime/Cargo.toml",
    "trellis-sdk-auth.workspace = true\n",
    "",
)

# The canonical API artifact owns capability-to-permission matching. Both SDK
# generators previously duplicated this lookup over raw JSON, while runtime had
# to import generated descriptors to recover the same metadata.
api = Path("rust/crates/protocol/src/api.rs")
text = api.read_text()
anchor = '''    pub fn id(&self) -> &str {
        &self.id
    }
'''
method = '''    pub fn id(&self) -> &str {
        &self.id
    }

    /// Return capability names that include one exact API-surface permission.
    ///
    /// Results follow canonical capability-name order because API capabilities
    /// are normalized in a [`BTreeMap`].
    #[must_use]
    pub fn capability_names_for_surface(
        &self,
        surface: ApiSurfaceKindV1,
        name: &str,
        action: PermissionActionV1,
    ) -> Vec<String> {
        self.capabilities
            .iter()
            .filter(|(_, capability)| {
                capability.allows().iter().any(|atom| {
                    atom.action() == action
                        && atom.target().as_api_surface().is_some_and(
                            |(api, target_surface, target_name)| {
                                api == self.id
                                    && target_surface == surface
                                    && target_name == name
                            },
                        )
                })
            })
            .map(|(name, _)| name.clone())
            .collect()
    }
'''
if text.count(anchor) != 1:
    raise RuntimeError("ApiArtifactV1 id method anchor changed")
api.write_text(text.replace(anchor, method, 1))

# Re-export the typed permission selectors through contracts so code generators
# do not need a second direct protocol dependency.
replace_once(
    "rust/crates/contracts/src/lib.rs",
    "pub use trellis_protocol::ApiArtifactV1;",
    "pub use trellis_protocol::{ApiArtifactV1, ApiSurfaceKindV1, PermissionActionV1};",
)

rust_codegen = Path("rust/crates/codegen-rust/src/lib.rs")
text = rust_codegen.read_text()
old_import = '''    ContractBuilder, ContractKind, ContractsError, LoadedApi, LoadedParticipant,
    ParticipantUseRenderModel,
'''
new_import = '''    ApiSurfaceKindV1, ContractBuilder, ContractKind, ContractsError, LoadedApi,
    LoadedParticipant, ParticipantUseRenderModel, PermissionActionV1,
'''
if text.count(old_import) != 1:
    raise RuntimeError("codegen-rust contracts import changed")
text = text.replace(old_import, new_import, 1)
replacements = {
    'capability_names(&loaded.value, "rpc", key, "call")': 'loaded.api.capability_names_for_surface(\n            ApiSurfaceKindV1::Rpc,\n            key,\n            PermissionActionV1::Call,\n        )',
    'capability_names(&loaded.value, "operation", key, "invoke")': 'loaded.api.capability_names_for_surface(\n            ApiSurfaceKindV1::Operation,\n            key,\n            PermissionActionV1::Invoke,\n        )',
    'capability_names(&loaded.value, "operation", key, "observe")': 'loaded.api.capability_names_for_surface(\n            ApiSurfaceKindV1::Operation,\n            key,\n            PermissionActionV1::Observe,\n        )',
    'capability_names(&loaded.value, "operation", key, "cancel")': 'loaded.api.capability_names_for_surface(\n            ApiSurfaceKindV1::Operation,\n            key,\n            PermissionActionV1::Cancel,\n        )',
    'capability_names(&loaded.value, "operation", key, "control")': 'loaded.api.capability_names_for_surface(\n            ApiSurfaceKindV1::Operation,\n            key,\n            PermissionActionV1::Control,\n        )',
    'capability_names(&loaded.value, "event", key, "publish")': 'loaded.api.capability_names_for_surface(\n            ApiSurfaceKindV1::Event,\n            key,\n            PermissionActionV1::Publish,\n        )',
    'capability_names(&loaded.value, "event", key, "subscribe")': 'loaded.api.capability_names_for_surface(\n            ApiSurfaceKindV1::Event,\n            key,\n            PermissionActionV1::Subscribe,\n        )',
    'capability_names(&loaded.value, "feed", key, "subscribe")': 'loaded.api.capability_names_for_surface(\n            ApiSurfaceKindV1::Feed,\n            key,\n            PermissionActionV1::Subscribe,\n        )',
}
for old, new in replacements.items():
    count = text.count(old)
    if count != 1:
        raise RuntimeError(f"codegen-rust expected one capability lookup {old!r}, found {count}")
    text = text.replace(old, new, 1)
rust_codegen.write_text(text)
remove_between(
    "rust/crates/codegen-rust/src/lib.rs",
    "fn capability_names(api: &Value, surface: &str, name: &str, action: &str) -> Vec<String> {",
    "fn sdk_stem_from_contract_id_pascal(contract_id: &str) -> String {",
)

# TypeScript codegen used the same duplicated raw-JSON lookup. Route it through
# the canonical protocol artifact too; generated package contents must remain
# identical and are checked by the validation workflow.
ts_codegen = Path("rust/crates/codegen-ts/src/lib.rs")
text = ts_codegen.read_text()
old_import = "use trellis_contracts::{load_sdk_source, LoadedApi};"
new_import = (
    "use trellis_contracts::{\n"
    "    load_sdk_source, ApiSurfaceKindV1, LoadedApi, PermissionActionV1,\n"
    "};"
)
if text.count(old_import) != 1:
    raise RuntimeError("codegen-ts contracts import changed")
text = text.replace(old_import, new_import, 1)
for old, new in replacements.items():
    count = text.count(old)
    if count != 1:
        raise RuntimeError(f"codegen-ts expected one capability lookup {old!r}, found {count}")
    text = text.replace(old, new, 1)
ts_codegen.write_text(text)
remove_between(
    "rust/crates/codegen-ts/src/lib.rs",
    "fn capability_names(api: &Value, surface: &str, name: &str, action: &str) -> Vec<String> {",
    "fn escape_js_string(value: &str) -> String {",
)

# Router exposes the small non-generated primitive that descriptor registration
# already reduces to. The generic capability input preserves integration-test
# scoping for both &'static str descriptor slices and owned canonical names.
router = Path("rust/crates/trellis/src/service/router.rs")
text = router.read_text()
old_caps = '''    fn descriptor_capabilities(&self, capabilities: &[&str]) -> Vec<String> {
        #[cfg(feature = "integration-test-scoping")]
        if let Some(scope) = &self.integration_test_scope {
            return capabilities
                .iter()
                .map(|capability| scope.capability(capability))
                .collect();
        }
        capabilities
            .iter()
            .map(|capability| (*capability).to_string())
            .collect()
    }
'''
new_caps = '''    fn descriptor_capabilities<T>(&self, capabilities: &[T]) -> Vec<String>
    where
        T: AsRef<str>,
    {
        #[cfg(feature = "integration-test-scoping")]
        if let Some(scope) = &self.integration_test_scope {
            return capabilities
                .iter()
                .map(|capability| scope.capability(capability.as_ref()))
                .collect();
        }
        capabilities
            .iter()
            .map(|capability| capability.as_ref().to_string())
            .collect()
    }
'''
if text.count(old_caps) != 1:
    raise RuntimeError("Router descriptor_capabilities block changed")
text = text.replace(old_caps, new_caps, 1)
old_metadata = '''    /// Register one generated RPC descriptor for routing metadata only.
    ///
    /// This is used by runtimes whose handler dispatch predates the typed
    /// router but still must consume the router's exact permission metadata.
    pub fn register_rpc_metadata<D>(&mut self)
    where
        D: RpcDescriptor + 'static,
    {
        let capabilities = self.descriptor_capabilities(D::CALLER_CAPABILITIES);
        self.handlers.insert(
            self.descriptor_subject(D::SUBJECT),
            Route {
                capabilities: RouteCapabilities::Static(capabilities),
                permission: RoutePermissionSpec::Static(
                    ApiSurfaceKindV1::Rpc,
                    self.descriptor_name(D::KEY),
                    PermissionActionV1::Call,
                ),
                handler: Box::new(|_, _| {
                    Box::pin(async {
                        Err(ServerError::Nats(
                            "routing-metadata-only handler cannot execute".to_owned(),
                        ))
                    })
                }),
            },
        );
    }
'''
new_metadata = '''    /// Register one RPC route for routing metadata only.
    ///
    /// This supports runtimes that dispatch handlers outside [`Router`] while
    /// still using its exact permission metadata. Generated descriptors reduce
    /// to this same primitive through [`Self::register_rpc_metadata`].
    pub fn register_rpc_metadata_parts<T>(
        &mut self,
        subject: &str,
        name: &str,
        caller_capabilities: &[T],
    )
    where
        T: AsRef<str>,
    {
        let capabilities = self.descriptor_capabilities(caller_capabilities);
        self.handlers.insert(
            self.descriptor_subject(subject),
            Route {
                capabilities: RouteCapabilities::Static(capabilities),
                permission: RoutePermissionSpec::Static(
                    ApiSurfaceKindV1::Rpc,
                    self.descriptor_name(name),
                    PermissionActionV1::Call,
                ),
                handler: Box::new(|_, _| {
                    Box::pin(async {
                        Err(ServerError::Nats(
                            "routing-metadata-only handler cannot execute".to_owned(),
                        ))
                    })
                }),
            },
        );
    }

    /// Register one generated RPC descriptor for routing metadata only.
    pub fn register_rpc_metadata<D>(&mut self)
    where
        D: RpcDescriptor + 'static,
    {
        self.register_rpc_metadata_parts(D::SUBJECT, D::KEY, D::CALLER_CAPABILITIES);
    }
'''
if text.count(old_metadata) != 1:
    raise RuntimeError("Router generated metadata block changed")
router.write_text(text.replace(old_metadata, new_metadata, 1))

# Auth owns its canonical API source. Parse it through protocol, derive subjects
# and capability names through the same methods used by codegen, and register
# routing metadata without importing a generated crate.
rpc = Path("rust/crates/runtime/src/platform/auth/rpc/mod.rs")
text = rpc.read_text()
old_protocol_import = '''use trellis_protocol::{
    parse_session_proof_v1, session_proof_request_digest_v1, verify_session_proof_v1,
    AuthorizationPrincipalKindV1, SessionProofInputV1, SessionProofPolicyV1,
};
'''
new_protocol_import = '''use trellis_protocol::{
    parse_api_v1, parse_session_proof_v1, session_proof_request_digest_v1,
    verify_session_proof_v1, ApiSurfaceKindV1, AuthorizationPrincipalKindV1, PermissionActionV1,
    SessionProofInputV1, SessionProofPolicyV1,
};
'''
if text.count(old_protocol_import) != 1:
    raise RuntimeError("Auth RPC protocol import changed")
text = text.replace(old_protocol_import, new_protocol_import, 1)
anchor = "const MAX_CONCURRENT_REQUESTS: usize = 64;\n"
helper = '''const AUTH_API_JSON: &str = include_str!("../../../../trellis.api.json");

pub(super) fn register_auth_rpc_metadata(
    routes: &mut Router,
) -> Result<(), AuthorizationStateError> {
    let value: Value = serde_json::from_str(AUTH_API_JSON)
        .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
    let api = parse_api_v1(&value)
        .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
    let subjects = api
        .derived_subjects()
        .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
    routes.set_api_id(api.id());
    for (name, subject) in subjects.rpc {
        let capabilities = api.capability_names_for_surface(
            ApiSurfaceKindV1::Rpc,
            &name,
            PermissionActionV1::Call,
        );
        routes.register_rpc_metadata_parts(&subject, &name, &capabilities);
    }
    Ok(())
}

'''
if text.count(anchor) != 1:
    raise RuntimeError("Auth RPC constant anchor changed")
text = text.replace(anchor, anchor + "\n" + helper, 1)
old_call = "        trellis_sdk_auth::api::register_rpc_metadata(&mut routes);"
if text.count(old_call) != 3:
    raise RuntimeError(f"expected three Auth SDK metadata registrations, found {text.count(old_call)}")
text = text.replace(old_call, "        register_auth_rpc_metadata(&mut routes)?;", 1)
text = text.replace(old_call, "        register_auth_rpc_metadata(&mut routes).unwrap();")
rpc.write_text(text)

replace_once(
    "rust/crates/runtime/src/platform/auth/verifier.rs",
    "        trellis_sdk_auth::api::register_rpc_metadata(&mut routes);",
    "        super::rpc::register_auth_rpc_metadata(&mut routes).unwrap();",
)

print("runtime generated Auth SDK decoupling v3 transform complete")
