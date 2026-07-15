use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine as _;
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::ProtocolError;

/// Render JSON into the canonical Trellis form used for digests.
pub fn canonicalize_json(value: &Value) -> Result<String, ProtocolError> {
    match value {
        Value::Null => Ok("null".to_string()),
        Value::Bool(value) => Ok(if *value { "true" } else { "false" }.to_string()),
        Value::String(value) => Ok(serde_json::to_string(value).expect("string serialization")),
        Value::Number(value) => {
            let rendered = value.to_string();
            if rendered == "-0" {
                return Err(ProtocolError::NonCanonicalNumber(rendered));
            }
            Ok(rendered)
        }
        Value::Array(values) => {
            let mut out = String::from("[");
            for (index, item) in values.iter().enumerate() {
                if index > 0 {
                    out.push(',');
                }
                out.push_str(&canonicalize_json(item)?);
            }
            out.push(']');
            Ok(out)
        }
        Value::Object(map) => {
            let mut keys = map.keys().collect::<Vec<_>>();
            keys.sort();

            let mut out = String::from("{");
            for (index, key) in keys.iter().enumerate() {
                if index > 0 {
                    out.push(',');
                }
                out.push_str(&serde_json::to_string(*key).expect("key serialization"));
                out.push(':');
                out.push_str(&canonicalize_json(&map[*key])?);
            }
            out.push('}');
            Ok(out)
        }
    }
}

/// Compute a base64url-encoded SHA-256 digest for text.
pub fn sha256_base64url(text: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    URL_SAFE_NO_PAD.encode(hasher.finalize())
}

/// Canonicalize JSON and return its Trellis digest.
pub fn digest_json(value: &Value) -> Result<String, ProtocolError> {
    Ok(sha256_base64url(&canonicalize_json(value)?))
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use serde_json::Value;

    use super::*;

    #[test]
    fn matches_shared_canonical_json_vectors() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../conformance/canonical-json/vectors.json");
        let fixtures: Vec<Value> =
            serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap();

        for fixture in fixtures {
            let input = fixture.get("input").unwrap();
            assert_eq!(
                canonicalize_json(input).unwrap(),
                fixture.get("canonical").and_then(Value::as_str).unwrap()
            );
            assert_eq!(
                digest_json(input).unwrap(),
                fixture.get("digest").and_then(Value::as_str).unwrap()
            );
        }
    }
}
