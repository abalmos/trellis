from pathlib import Path


def replace_once(text: str, old: str, new: str, label: str) -> str:
    count = text.count(old)
    if count != 1:
        raise RuntimeError(f"{label}: expected one occurrence, found {count}")
    return text.replace(old, new, 1)


# Keep the exact API identity in the canonical route without embedding '@' in a
# subject token. API `trellis.auth@v1` becomes namespace `api.trellis.auth.v1`.
path = Path("rust/crates/protocol/src/subjects.rs")
text = path.read_text()
old = '''    validate_api_id("/api", api_id, api_error)?;
    validate_version("/version", version, api_error)?;
    validate_logical_name("/name", logical_name, api_error)?;
    Ok(format!("{family}.{version}.{api_id}.{logical_name}"))
}
'''
new = '''    validate_api_id("/api", api_id, api_error)?;
    validate_version("/version", version, api_error)?;
    validate_logical_name("/name", logical_name, api_error)?;
    let (lineage, major) = api_id
        .rsplit_once("@v")
        .expect("validated API id contains @vN");
    Ok(format!(
        "{family}.{version}.api.{lineage}.v{major}.{logical_name}"
    ))
}
'''
text = replace_once(text, old, new, "Rust API subject namespace")
text = text.replace(
    '"rpc.v1.example.documents@v1.Documents.Get"',
    '"rpc.v1.api.example.documents.v1.Documents.Get"',
)
text = text.replace(
    '"rpc.v10.example.documents@v1.Documents.Get"',
    '"rpc.v10.api.example.documents.v1.Documents.Get"',
)
path.write_text(text)

path = Path("rust/crates/protocol/src/api.rs")
text = path.read_text().replace(
    "/// feeds use `feed`; each subject then includes the exact API ID before the\n    /// logical name.",
    "/// feeds use `feed`; each subject then includes a reversible `api.<lineage>.vN`\n    /// namespace derived from the exact API ID before the logical name.",
)
path.write_text(text)

path = Path("rust/crates/protocol/src/lib.rs")
text = path.read_text()
text = text.replace(
    "`rpc.v1.documents@v1.Documents.Get`",
    "`rpc.v1.api.documents.v1.Documents.Get`",
)
text = text.replace(
    '"rpc.v1.documents@v1.Documents.Get"',
    '"rpc.v1.api.documents.v1.Documents.Get"',
)
path.write_text(text)

path = Path("ts/packages/trellis/contract_support/mod.ts")
text = path.read_text()
anchor = '''function rpcSubject(
  apiId: string,
  name: string,
  version: `v${number}`,
): string {
  return `rpc.${version}.${apiId}.${name}`;
}
'''
replacement = '''function apiSubjectNamespace(apiId: string): string {
  const marker = apiId.lastIndexOf("@v");
  if (marker <= 0) {
    throw new Error(`Invalid Trellis API id '${apiId}'`);
  }
  const lineage = apiId.slice(0, marker);
  const major = apiId.slice(marker + 2);
  return `api.${lineage}.v${major}`;
}

function rpcSubject(
  apiId: string,
  name: string,
  version: `v${number}`,
): string {
  return `rpc.${version}.${apiSubjectNamespace(apiId)}.${name}`;
}
'''
text = replace_once(text, anchor, replacement, "TS API namespace helper")
text = text.replace(
    'return `operations.${version}.${apiId}.${name}`;',
    'return `operations.${version}.${apiSubjectNamespace(apiId)}.${name}`;',
)
text = text.replace(
    'return `feed.${version}.${apiId}.${name}`;',
    'return `feed.${version}.${apiSubjectNamespace(apiId)}.${name}`;',
)
text = text.replace(
    'return `events.${version}.${apiId}.${name}${suffix}`;',
    'return `events.${version}.${apiSubjectNamespace(apiId)}.${name}${suffix}`;',
)
old = '''type ProjectedOperationSubject<TId, TKey, TOperation> =
  TId extends string
    ? TOperation extends { version: infer TVersion extends `v${number}` }
      ? `operations.${TVersion}.${TId}.${Extract<TKey, string>}`
    : string
  : string;
'''
new = '''type ProjectedApiSubjectNamespace<TId> = TId extends
  `${infer TLineage}@v${infer TMajor}` ? `api.${TLineage}.v${TMajor}`
  : string;

type ProjectedOperationSubject<TId, TKey, TOperation> =
  TId extends string
    ? TOperation extends { version: infer TVersion extends `v${number}` }
      ? `operations.${TVersion}.${ProjectedApiSubjectNamespace<TId>}.${Extract<TKey, string>}`
    : string
  : string;
'''
text = replace_once(text, old, new, "TS operation namespace type")
path.write_text(text)
