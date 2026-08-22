from pathlib import Path


def replace_once(path: str, old: str, new: str, label: str) -> None:
    file = Path(path)
    text = file.read_text()
    count = text.count(old)
    if count != 1:
        raise RuntimeError(f"{path}: {label}: expected one match, found {count}")
    file.write_text(text.replace(old, new, 1))


rpc = "rust/crates/trellis/tests/integration/rpc.rs"
replace_once(
    rpc,
    '''    let client_subject = client.integration_test_descriptor_subject(EntityGetRpc::SUBJECT);
    let client_capability = client.integration_test_descriptor_capability(RPC_READ_CAPABILITY);
    assert_eq!(service_subjects, [client_subject.as_str()]);
    assert_ne!(client_subject, EntityGetRpc::SUBJECT);
''',
    '''    let client_subject = EntityGetRpc::SUBJECT.to_owned();
    let client_capability = RPC_READ_CAPABILITY.to_owned();
    assert_eq!(service_subjects, [EntityGetRpc::SUBJECT]);
    assert_eq!(client_subject, EntityGetRpc::SUBJECT);
''',
    "successful RPC authored identity",
)
replace_once(
    rpc,
    '''    let client_subject = client.integration_test_descriptor_subject(EntityGetRpc::SUBJECT);
    let result = call_entity_get_expecting_error(&client, "entity-1").await;
''',
    '''    let client_subject = EntityGetRpc::SUBJECT.to_owned();
    let result = call_entity_get_expecting_error(&client, "entity-1").await;
''',
    "declared-error RPC authored subject",
)

event_consumers = "rust/crates/trellis/tests/integration/event_consumers.rs"
replace_once(
    event_consumers,
    "    let expected_subject = _runtime.integration_test_descriptor_subject(SourcePingedEvent::SUBJECT);\n",
    "    let expected_subject = SourcePingedEvent::SUBJECT.to_owned();\n",
    "missing-group expected subject",
)
# The same source line occurs a second time in the ambiguous-group test after the
# first replacement, so apply the identical exact replacement once more.
replace_once(
    event_consumers,
    "    let expected_subject = _runtime.integration_test_descriptor_subject(SourcePingedEvent::SUBJECT);\n",
    "    let expected_subject = SourcePingedEvent::SUBJECT.to_owned();\n",
    "ambiguous-group expected subject",
)
replace_once(
    event_consumers,
    "    let subject = runtime.integration_test_descriptor_subject(subject);\n",
    "    let subject = subject.to_owned();\n",
    "consumer subject filter",
)
replace_once(
    event_consumers,
    '''    let first_subject = runtime.integration_test_descriptor_subject(first_subject);
    let second_subject = runtime.integration_test_descriptor_subject(second_subject);
''',
    '''    let first_subject = first_subject.to_owned();
    let second_subject = second_subject.to_owned();
''',
    "grouped consumer subject filters",
)

prepared = "rust/crates/trellis/tests/integration/prepared_events.rs"
replace_once(
    prepared,
    "    let raw_subject = runtime.integration_test_descriptor_subject(EntityChanged::SUBJECT);\n",
    "    let raw_subject = EntityChanged::SUBJECT.to_owned();\n",
    "prepared-event raw subject",
)

for path in Path("rust/crates/trellis/tests/integration").glob("*.rs"):
    content = path.read_text()
    if "integration_test_descriptor_subject" in content or "integration_test_descriptor_capability" in content:
        raise RuntimeError(f"stale descriptor-scoping fixture hook in {path}")
