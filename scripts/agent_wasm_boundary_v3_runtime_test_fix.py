from pathlib import Path

p = Path("rust/crates/runtime/src/platform/auth/verifier.rs")
text = p.read_text()
old = '''        let mut routes = trellis_rs::service::Router::new();
        trellis_sdk_auth::api::register_rpc_metadata(&mut routes);
        let required_permission = routes
            .required_permission(subject, payload)
            .unwrap()
            .unwrap()
            .permission_atom()
            .unwrap();
'''
if text.count(old) != 1:
    raise RuntimeError("runtime verifier generated Auth metadata test anchor changed")
new = '''        let required_permission = PermissionAtomV1::new(
            PermissionTargetV1::api_surface(
                "trellis.auth@v1",
                ApiSurfaceKindV1::Rpc,
                "Auth.Sessions.Me",
            )
            .unwrap(),
            PermissionActionV1::Call,
        )
        .unwrap();
'''
text = text.replace(old, new, 1)
p.write_text(text)
