from pathlib import Path


def replace_once(path: str, old: str, new: str) -> None:
    p = Path(path)
    text = p.read_text()
    count = text.count(old)
    if count != 1:
        raise RuntimeError(f"{path}: expected one occurrence, found {count}: {old[:100]!r}")
    p.write_text(text.replace(old, new, 1))


# Generated SDKs are outputs, not workspace prerequisites. Only Auth was actually
# consumed by production Rust, and that dependency is removed below.
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

# Router exposes the small dynamic primitive that generated descriptors already use.
router = Path("rust/crates/trellis/src/service/router.rs")
text = router.read_text()
old = '''    /// Register one generated RPC descriptor for routing metadata only.
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
new = '''    /// Register one RPC route for routing metadata only.
    ///
    /// This supports runtimes that dispatch handlers outside [`Router`] while
    /// still using its exact permission metadata. Generated descriptors delegate
    /// to the same primitive through [`Self::register_rpc_metadata`].
    pub fn register_rpc_metadata_parts(
        &mut self,
        subject: &str,
        name: &str,
        caller_capabilities: &[&str],
    ) {
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
if text.count(old) != 1:
    raise RuntimeError("Router generated metadata block changed")
router.write_text(text.replace(old, new, 1))

# Auth owns its API source already; derive route permission metadata from it directly.
rpc = Path("rust/crates/runtime/src/platform/auth/rpc/mod.rs")
text = rpc.read_text()
text = text.replace(
    "use std::sync::Arc;\n",
    "use std::collections::BTreeMap;\nuse std::sync::Arc;\n",
    1,
)
anchor = "const MAX_CONCURRENT_REQUESTS: usize = 64;\n"
helper = '''const AUTH_API_JSON: &str = include_str!("../../../../trellis.api.json");

#[derive(serde::Deserialize)]
struct AuthApiRoutes {
    id: String,
    rpc: BTreeMap<String, AuthRpcRoute>,
}

#[derive(serde::Deserialize)]
struct AuthRpcRoute {
    version: String,
}

pub(super) fn register_auth_rpc_metadata(
    routes: &mut Router,
) -> Result<(), AuthorizationStateError> {
    let api: AuthApiRoutes = serde_json::from_str(AUTH_API_JSON)
        .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
    routes.set_api_id(api.id);
    for (name, route) in api.rpc {
        routes.register_rpc_metadata_parts(
            &format!("rpc.{}.{}", route.version, name),
            &name,
            &[],
        );
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

print("runtime generated Auth SDK decoupling transform complete")
