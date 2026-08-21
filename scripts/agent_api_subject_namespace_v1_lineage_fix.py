from pathlib import Path


def replace_once(text: str, old: str, new: str, label: str) -> str:
    count = text.count(old)
    if count != 1:
        raise RuntimeError(f"{label}: expected one occurrence, found {count}")
    return text.replace(old, new, 1)


path = Path("rust/crates/protocol/src/identifiers.rs")
text = path.read_text()
old = '''    validate_logical_name(path, lineage, error)?;
    validate_positive_decimal(path, major, error)
'''
new = '''    validate_logical_name(path, lineage, error)?;
    if lineage.split('.').any(|token| {
        !token
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
    }) {
        return Err(error(
            path.to_owned(),
            "API lineage tokens must use only ASCII letters, digits, '-' or '_'".to_owned(),
        ));
    }
    validate_positive_decimal(path, major, error)
'''
text = replace_once(text, old, new, "conservative API lineage")
path.write_text(text)

path = Path("rust/crates/protocol/src/subjects.rs")
text = path.read_text()
old = '''        assert!(derive_rpc_subject("example.*@v1", "v1", "Documents.Get").is_err());
'''
new = '''        assert!(derive_rpc_subject("example.*@v1", "v1", "Documents.Get").is_err());
        assert!(derive_rpc_subject("example/@v1", "v1", "Documents.Get").is_err());
        assert!(derive_rpc_subject("example.☃@v1", "v1", "Documents.Get").is_err());
'''
text = replace_once(text, old, new, "API lineage subject tests")
path.write_text(text)

path = Path("ts/packages/trellis/contract_support/mod.ts")
text = path.read_text()
old = '''  const lineage = apiId.slice(0, marker);
  const major = apiId.slice(marker + 2);
  return `api.${lineage}.v${major}`;
'''
new = '''  const lineage = apiId.slice(0, marker);
  const major = apiId.slice(marker + 2);
  if (
    !/^[A-Za-z0-9_-]+(?:\\.[A-Za-z0-9_-]+)*$/.test(lineage) ||
    !/^[1-9][0-9]*$/.test(major)
  ) {
    throw new Error(`Invalid Trellis API id '${apiId}'`);
  }
  return `api.${lineage}.v${major}`;
'''
text = replace_once(text, old, new, "TS conservative API lineage")
path.write_text(text)
