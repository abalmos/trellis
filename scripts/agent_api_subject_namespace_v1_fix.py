from pathlib import Path

path = Path("rust/crates/protocol/src/lib.rs")
text = path.read_text()
text = text.replace(
    "surface version used to derive subjects such as `rpc.v1.Documents.Get`.",
    "surface version used to derive subjects such as `rpc.v1.documents@v1.Documents.Get`.",
)
text = text.replace(
    'assert_eq!(api.derived_subjects()?.rpc["Documents.Get"], "rpc.v1.Documents.Get");',
    'assert_eq!(\n//!     api.derived_subjects()?.rpc["Documents.Get"],\n//!     "rpc.v1.documents@v1.Documents.Get"\n//! );',
)
path.write_text(text)

path = Path("rust/crates/protocol/src/subjects.rs")
text = path.read_text()
for noun in ("RPC", "operation", "event base", "feed"):
    text = text.replace(
        f"/// Derive an {noun} subject from its version and logical name.",
        f"/// Derive an {noun} subject from its API id, version, and logical name.",
    )
text = text.replace(
    "/// Returns [`ProtocolError::ApiValidation`] for an invalid `vN` version or\n/// logical surface name.",
    "/// Returns [`ProtocolError::ApiValidation`] for an invalid API id, `vN` version,\n/// or logical surface name.",
)
text = text.replace(
    "/// Returns [`ProtocolError::ApiValidation`] for an invalid `vN` version or\n/// logical surface name.",
    "/// Returns [`ProtocolError::ApiValidation`] for an invalid API id, `vN` version,\n/// or logical surface name.",
)
path.write_text(text)
