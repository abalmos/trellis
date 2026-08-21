from pathlib import Path


def replace_once(text: str, old: str, new: str, label: str) -> str:
    count = text.count(old)
    if count != 1:
        raise RuntimeError(f"{label}: expected one occurrence, found {count}")
    return text.replace(old, new, 1)


def replace_count(text: str, old: str, new: str, expected: int, label: str) -> str:
    count = text.count(old)
    if count != expected:
        raise RuntimeError(f"{label}: expected {expected} occurrences, found {count}")
    return text.replace(old, new)


# API IDs are now part of canonical NATS subjects. Their lineage therefore must
# itself be NATS-safe dot-separated tokens. This tightens an unreleased v1 rule
# rather than escaping/aliasing arbitrary old IDs.
path = Path("rust/crates/protocol/src/identifiers.rs")
text = path.read_text()
old = '''    if lineage.is_empty() || lineage.contains("@v") {
        return Err(error(
            path.to_owned(),
            "must contain one non-empty lineage before '@vN'".to_owned(),
        ));
    }
    validate_positive_decimal(path, major, error)
'''
new = '''    if lineage.is_empty() || lineage.contains("@v") {
        return Err(error(
            path.to_owned(),
            "must contain one non-empty lineage before '@vN'".to_owned(),
        ));
    }
    validate_logical_name(path, lineage, error)?;
    validate_positive_decimal(path, major, error)
'''
text = replace_once(text, old, new, "NATS-safe API lineage")
path.write_text(text)

# Canonical subjects retain the familiar family/version prefix while adding the
# exact versioned API identity before the logical surface name. Same API/action
# instances still share a queue group; different APIs cannot collide.
path = Path("rust/crates/protocol/src/subjects.rs")
text = path.read_text()
text = text.replace(
    'identifiers::{api_error, validate_logical_name, validate_version}',
    'identifiers::{api_error, validate_api_id, validate_logical_name, validate_version}',
)
text = text.replace(
    'pub fn derive_rpc_subject(version: &str, logical_name: &str) -> Result<String, ProtocolError> {\n    derive_subject("rpc", version, logical_name)\n}',
    'pub fn derive_rpc_subject(\n    api_id: &str,\n    version: &str,\n    logical_name: &str,\n) -> Result<String, ProtocolError> {\n    derive_subject("rpc", api_id, version, logical_name)\n}',
)
text = text.replace(
    'pub fn derive_operation_subject(\n    version: &str,\n    logical_name: &str,\n) -> Result<String, ProtocolError> {\n    derive_subject("operations", version, logical_name)\n}',
    'pub fn derive_operation_subject(\n    api_id: &str,\n    version: &str,\n    logical_name: &str,\n) -> Result<String, ProtocolError> {\n    derive_subject("operations", api_id, version, logical_name)\n}',
)
text = text.replace(
    'pub fn derive_event_subject(version: &str, logical_name: &str) -> Result<String, ProtocolError> {\n    derive_subject("events", version, logical_name)\n}',
    'pub fn derive_event_subject(\n    api_id: &str,\n    version: &str,\n    logical_name: &str,\n) -> Result<String, ProtocolError> {\n    derive_subject("events", api_id, version, logical_name)\n}',
)
text = text.replace(
    'pub fn derive_event_wildcard_subject(\n    version: &str,\n    logical_name: &str,\n    parameter_count: usize,\n) -> Result<String, ProtocolError> {\n    let mut subject = derive_event_subject(version, logical_name)?;',
    'pub fn derive_event_wildcard_subject(\n    api_id: &str,\n    version: &str,\n    logical_name: &str,\n    parameter_count: usize,\n) -> Result<String, ProtocolError> {\n    let mut subject = derive_event_subject(api_id, version, logical_name)?;',
)
text = text.replace(
    'pub fn derive_feed_subject(version: &str, logical_name: &str) -> Result<String, ProtocolError> {\n    derive_subject("feed", version, logical_name)\n}',
    'pub fn derive_feed_subject(\n    api_id: &str,\n    version: &str,\n    logical_name: &str,\n) -> Result<String, ProtocolError> {\n    derive_subject("feed", api_id, version, logical_name)\n}',
)
old = '''fn derive_subject(
    family: &str,
    version: &str,
    logical_name: &str,
) -> Result<String, ProtocolError> {
    validate_version("/version", version, api_error)?;
    validate_logical_name("/name", logical_name, api_error)?;
    Ok(format!("{family}.{version}.{logical_name}"))
}
'''
new = '''fn derive_subject(
    family: &str,
    api_id: &str,
    version: &str,
    logical_name: &str,
) -> Result<String, ProtocolError> {
    validate_api_id("/api", api_id, api_error)?;
    validate_version("/version", version, api_error)?;
    validate_logical_name("/name", logical_name, api_error)?;
    Ok(format!("{family}.{version}.{api_id}.{logical_name}"))
}
'''
text = replace_once(text, old, new, "qualified Rust subject builder")
old = '''    fn subject_versions_use_canonical_positive_decimals() {
        assert_eq!(
            derive_rpc_subject("v1", "Documents.Get").unwrap(),
            "rpc.v1.Documents.Get"
        );
        assert_eq!(
            derive_rpc_subject("v10", "Documents.Get").unwrap(),
            "rpc.v10.Documents.Get"
        );
        assert!(derive_rpc_subject("v01", "Documents.Get").is_err());
        assert!(derive_rpc_subject("v00", "Documents.Get").is_err());
    }
'''
new = '''    fn subjects_are_api_qualified_and_versions_are_canonical() {
        assert_eq!(
            derive_rpc_subject("example.documents@v1", "v1", "Documents.Get").unwrap(),
            "rpc.v1.example.documents@v1.Documents.Get"
        );
        assert_eq!(
            derive_rpc_subject("example.documents@v1", "v10", "Documents.Get").unwrap(),
            "rpc.v10.example.documents@v1.Documents.Get"
        );
        assert_ne!(
            derive_rpc_subject("example.documents@v1", "v1", "Entity.Get").unwrap(),
            derive_rpc_subject("example.assets@v1", "v1", "Entity.Get").unwrap(),
        );
        assert!(derive_rpc_subject("example.documents@v1", "v01", "Documents.Get").is_err());
        assert!(derive_rpc_subject("example.documents@v1", "v00", "Documents.Get").is_err());
        assert!(derive_rpc_subject("example.*@v1", "v1", "Documents.Get").is_err());
    }
'''
text = replace_once(text, old, new, "Rust subject tests")
path.write_text(text)

# API-derived subjects always include the artifact's exact API ID.
path = Path("rust/crates/protocol/src/api.rs")
text = path.read_text()
text = replace_count(
    text,
    'derive_rpc_subject(&definition.version, name)?',
    'derive_rpc_subject(&self.id, &definition.version, name)?',
    1,
    "RPC subject call",
)
text = replace_count(
    text,
    'derive_operation_subject(&definition.version, name)?',
    'derive_operation_subject(&self.id, &definition.version, name)?',
    1,
    "operation subject call",
)
text = replace_count(
    text,
    'base: derive_event_subject(&definition.version, name)?,',
    'base: derive_event_subject(&self.id, &definition.version, name)?,',
    1,
    "event subject call",
)
text = replace_count(
    text,
    'derive_event_wildcard_subject(\n                            &definition.version,',
    'derive_event_wildcard_subject(\n                            &self.id,\n                            &definition.version,',
    1,
    "event wildcard call",
)
text = replace_count(
    text,
    'derive_feed_subject(&definition.version, name)?',
    'derive_feed_subject(&self.id, &definition.version, name)?',
    1,
    "feed subject call",
)
text = text.replace(
    '/// The `lineage@vN` identifier is the API-level identity. Surface-local\n/// versions independently control derived NATS subjects.',
    '/// The `lineage@vN` identifier is the API-level identity and transport namespace.\n/// Surface-local versions independently version routes within that API namespace.',
)
text = text.replace(
    '/// RPCs use `rpc`, operations use `operations`, events use `events`, and\n    /// feeds use `feed`. Event wildcard subjects append one `*` token for each',
    '/// RPCs use `rpc`, operations use `operations`, events use `events`, and\n    /// feeds use `feed`; each subject then includes the exact API ID before the\n    /// logical name. Event wildcard subjects append one `*` token for each',
)
path.write_text(text)

# TypeScript authoring must use exactly the same canonical subject rule. Source
# contracts do not get a private subject override: the protocol artifact owns
# route identity.
path = Path("ts/packages/trellis/contract_support/mod.ts")
text = path.read_text()
text = replace_count(text, '  subject?: string;\n', '', 4, "remove TS source subject overrides")
old = '''function rpcSubject(name: string, version: `v${number}`): string {
  return `rpc.${version}.${name}`;
}

function operationSubject(name: string, version: `v${number}`): string {
  return `operations.${version}.${name}`;
}

function feedSubject(name: string, version: `v${number}`): string {
  return `feed.${version}.${name}`;
}

function eventSubject(
  name: string,
  version: `v${number}`,
  params: readonly SubjectParam[] | undefined,
): string {
'''
new = '''function rpcSubject(
  apiId: string,
  name: string,
  version: `v${number}`,
): string {
  return `rpc.${version}.${apiId}.${name}`;
}

function operationSubject(
  apiId: string,
  name: string,
  version: `v${number}`,
): string {
  return `operations.${version}.${apiId}.${name}`;
}

function feedSubject(
  apiId: string,
  name: string,
  version: `v${number}`,
): string {
  return `feed.${version}.${apiId}.${name}`;
}

function eventSubject(
  apiId: string,
  name: string,
  version: `v${number}`,
  params: readonly SubjectParam[] | undefined,
): string {
'''
text = replace_once(text, old, new, "TS subject helpers")
text = replace_once(
    text,
    '  return `events.${version}.${name}${suffix}`;',
    '  return `events.${version}.${apiId}.${name}${suffix}`;',
    "TS event subject body",
)
text = replace_once(
    text,
    'subject: rpcSubject(name, "v1"),',
    'subject: rpcSubject(TRELLIS_STATE_CONTRACT_ID, name, "v1"),',
    "State RPC subject",
)
text = replace_count(
    text,
    'subject: rpcSubject(name, method.version),',
    'subject: rpcSubject(source.id, name, method.version),',
    1,
    "runtime RPC subject",
)
text = replace_count(
    text,
    '{ ...method, subject: rpcSubject(name, method.version) },',
    '{ ...method, subject: rpcSubject(source.id, name, method.version) },',
    1,
    "artifact RPC subject",
)
text = replace_count(
    text,
    'subject: operationSubject(name, operation.version),',
    'subject: operationSubject(source.id, name, operation.version),',
    2,
    "operation subjects",
)
text = replace_count(
    text,
    'subject: feedSubject(name, feed.version),',
    'subject: feedSubject(source.id, name, feed.version),',
    1,
    "runtime feed subject",
)
text = replace_count(
    text,
    '{ ...feed, subject: feedSubject(name, feed.version) },',
    '{ ...feed, subject: feedSubject(source.id, name, feed.version) },',
    1,
    "artifact feed subject",
)
text = replace_count(
    text,
    'subject: eventSubject(name, event.version, event.params),',
    'subject: eventSubject(source.id, name, event.version, event.params),',
    2,
    "event subjects",
)
old = '''type ProjectedOperationSubject<TKey, TOperation> = TOperation extends {
  subject: infer TSubject extends string;
} ? TSubject
  : TOperation extends { version: infer TVersion extends `v${number}` }
    ? `operations.${TVersion}.${Extract<TKey, string>}`
  : string;
'''
new = '''type ProjectedOperationSubject<TId, TKey, TOperation> =
  TId extends string
    ? TOperation extends { version: infer TVersion extends `v${number}` }
      ? `operations.${TVersion}.${TId}.${Extract<TKey, string>}`
    : string
  : string;
'''
text = replace_once(text, old, new, "operation subject type")
text = replace_once(
    text,
    'subject: ProjectedOperationSubject<K, T[K]>;',
    'subject: ProjectedOperationSubject<TId, K, T[K]>;',
    "operation subject type use",
)
path.write_text(text)

# Fail early on the old unqualified canonical subject helper shape or source-only
# custom transport override. Test fixtures may still contain old expected strings;
# the real test/generation gates will identify those precisely.
for candidate in [Path("rust/crates/protocol/src/subjects.rs"), Path("ts/packages/trellis/contract_support/mod.ts")]:
    content = candidate.read_text()
    for stale in (
        'return `rpc.${version}.${name}`',
        'return `operations.${version}.${name}`',
        'return `events.${version}.${name}',
        'return `feed.${version}.${name}`',
    ):
        if stale in content:
            raise RuntimeError(f"stale unqualified subject derivation in {candidate}: {stale}")
