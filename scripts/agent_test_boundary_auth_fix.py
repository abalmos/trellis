from pathlib import Path

# API/action/capability/subject identity stays exactly authored. Only participant
# identity is case-specific in the shared Rust harness.
path = Path("rust/crates/trellis/tests/integration/auth.rs")
text = path.read_text()

service_scope = '''    let read_capability = format!(
        "{}::read",
        runtime
            .scoped_contract(&service_contract)
            .expect("scope trusted-portal service contract")
            .id()
            .strip_suffix("@v1")
            .expect("versioned trusted-portal service ID")
    );
    let publish_capability = read_capability.replace("::read", "::publish");
'''
service_static = '''    let read_capability = READ_CAPABILITY.to_owned();
    let publish_capability = PUBLISH_CAPABILITY.to_owned();
'''
if text.count(service_scope) != 1:
    raise RuntimeError(
        f"expected one capability-only service scoped contract, found {text.count(service_scope)}"
    )
text = text.replace(service_scope, service_static, 1)

remaining = text.count(".scoped_contract(")
if remaining != 8:
    raise RuntimeError(f"expected eight participant scoped-contract uses, found {remaining}")
text = text.replace(".scoped_contract(", ".case_contract(")
text = text.replace("scope approved client contract", "build case approved client participant")
text = text.replace("scope optional client contract", "build case optional client participant")
text = text.replace("scope inventoried participant", "build case inventoried participant")
text = text.replace("scope routed client", "build case routed client participant")
text = text.replace("scope trusted portal client", "build case trusted portal participant")
text = text.replace(
    "scope trusted-portal client contract",
    "build case trusted-portal client participant",
)

# Platform Auth subjects were never scoped by IntegrationTestScope. Keep the
# authored platform subject and its already case-unique deployment component.
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
        raise RuntimeError(f"expected one activation event subject for {value}")
    text = text.replace(old, new, 1)

resolved_old = '''fixture
        .runtime
        .integration_test_descriptor_subject(&format!(
            "{}.{}",
            auth_sdk::events::AuthDeviceUserAuthoritiesResolvedEventDescriptor::SUBJECT,
            device.approval.deployment_id,
        ))'''
resolved_new = '''format!(
        "{}.{}",
        auth_sdk::events::AuthDeviceUserAuthoritiesResolvedEventDescriptor::SUBJECT,
        device.approval.deployment_id,
    )'''
if text.count(resolved_old) != 2:
    raise RuntimeError(
        f"expected two resolved authority event subjects, found {text.count(resolved_old)}"
    )
text = text.replace(resolved_old, resolved_new)

rpc_observer = '''        .subscribe(
            fixture
                .runtime
                .integration_test_descriptor_subject(ValueGet::SUBJECT),
        )'''
if text.count(rpc_observer) != 1:
    raise RuntimeError(f"expected one raw RPC observer, found {text.count(rpc_observer)}")
text = text.replace(rpc_observer, "        .subscribe(ValueGet::SUBJECT)", 1)

event_observer = '''        .subscribe(
            fixture
                .runtime
                .integration_test_descriptor_subject(ValueChanged::SUBJECT),
        )'''
if text.count(event_observer) != 1:
    raise RuntimeError(f"expected one raw event observer, found {text.count(event_observer)}")
text = text.replace(event_observer, "        .subscribe(ValueChanged::SUBJECT)", 1)

for token in ("scoped_contract", "integration_test_descriptor_subject"):
    if token in text:
        raise RuntimeError(f"stale auth fixture scoping token {token!r}")

path.write_text(text)
