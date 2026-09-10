use std::str::FromStr;

use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::{Number, Value};
use trellis_rs::service::ServerError;

pub(crate) fn generated_input(
    input: impl Serialize,
    integer_fields: &[&str],
) -> Result<Value, ServerError> {
    let mut value = serde_json::to_value(input)?;
    convert_integer_fields(&mut value, integer_fields, false);
    Ok(value)
}

pub(crate) fn generated_output<T: DeserializeOwned>(
    mut value: Value,
    integer_fields: &[&str],
) -> Result<T, ServerError> {
    convert_integer_fields(&mut value, integer_fields, true);
    serde_json::from_value(value).map_err(|error| {
        ServerError::Nats(format!("failed to encode generated Events value: {error}"))
    })
}

fn convert_integer_fields(value: &mut Value, integer_fields: &[&str], encode: bool) {
    match value {
        Value::Array(values) => {
            for value in values {
                convert_integer_fields(value, integer_fields, encode);
            }
        }
        Value::Object(object) => {
            for (key, value) in object {
                if integer_fields.contains(&key.as_str()) {
                    if encode {
                        if let Value::Number(number) = value {
                            *value = Value::String(number.to_string());
                        }
                    } else if let Value::String(text) = value {
                        if let Ok(number) = Number::from_str(text) {
                            *value = Value::Number(number);
                        }
                    }
                } else {
                    convert_integer_fields(value, integer_fields, encode);
                }
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use trellis_rs::generated::Codec;
    use trellis_runtime_apis::apis::trellis_events_v1::rpc;

    use super::generated_output;

    #[test]
    fn generated_query_output_uses_decimal_integer_codecs() {
        let output: rpc::QueryOutput = generated_output(
            json!({ "events": [], "limit": 50, "offset": 0, "total": 0 }),
            &["limit", "offset", "total"],
        )
        .expect("convert generated output");

        assert_eq!(
            output.encode().expect("encode generated output"),
            json!({ "events": [], "limit": "50", "offset": "0", "total": "0" })
        );
    }
}
