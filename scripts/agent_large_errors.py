from pathlib import Path
import re


def replace_once(path: str, old: str, new: str) -> None:
    file = Path(path)
    text = file.read_text()
    count = text.count(old)
    if count != 1:
        raise RuntimeError(f"{path}: expected one occurrence, found {count}: {old[:100]!r}")
    file.write_text(text.replace(old, new, 1))


def remove_expect(path: str, lint: str, expected: int | None = None) -> int:
    file = Path(path)
    text = file.read_text()
    pattern = re.compile(
        rf"#\[expect\(\s*clippy::{re.escape(lint)},\s*reason\s*=\s*\"[^\"]*\"\s*\)\]\n",
        re.MULTILINE,
    )
    text, count = pattern.subn("", text)
    if expected is not None and count != expected:
        raise RuntimeError(f"{path}: expected {expected} clippy::{lint} expectations, found {count}")
    file.write_text(text)
    return count


# Generated callers share one typed error boundary. Keep the public error shape typed,
# but box the two payload-bearing variants so every generated RPC does not return a
# 150+ byte error value on the stack.
client_error = "rust/crates/trellis/src/client/error.rs"
replace_once(client_error, "    Declared(E),\n", "    Declared(Box<E>),\n")
replace_once(
    client_error,
    "    Validation(ValidationFailure),\n",
    "    Validation(Box<ValidationFailure>),\n",
)
replace_once(
    client_error,
    "                Ok(Some(error)) => Self::Declared(error),\n",
    "                Ok(Some(error)) => Self::Declared(Box::new(error)),\n",
)
replace_once(
    client_error,
    "                    Ok(Some(error)) => Self::Validation(ValidationFailure::Schema(error)),\n",
    "                    Ok(Some(error)) => {\n                        Self::Validation(Box::new(ValidationFailure::Schema(error)))\n                    }\n",
)
replace_once(
    client_error,
    "                        Ok(Some(error)) => Self::Validation(ValidationFailure::Validation(error)),\n",
    "                        Ok(Some(error)) => {\n                            Self::Validation(Box::new(ValidationFailure::Validation(error)))\n                        }\n",
)

# Local caller validation synthesizes the same public failure envelope, so adapt those
# two constructors and remove the result_large_err suppression that boxing makes stale.
connection = "rust/crates/trellis/src/client/connection.rs"
remove_expect(connection, "result_large_err", expected=1)
replace_once(
    connection,
    """        Err(crate::service::ServerError::Validation { issues }) => Err(CallError::Validation(\n            crate::client::ValidationFailure::Validation(crate::client::ValidationErrorPayload {\n""",
    """        Err(crate::service::ServerError::Validation { issues }) => Err(CallError::Validation(\n            Box::new(crate::client::ValidationFailure::Validation(\n                crate::client::ValidationErrorPayload {\n""",
)
replace_once(
    connection,
    """                context: None,\n                trace_id: None,\n            }),\n        )),\n        Err(crate::service::ServerError::SchemaValidation { issues }) => Err(\n            CallError::Validation(crate::client::ValidationFailure::Schema(\n""",
    """                    context: None,\n                    trace_id: None,\n                },\n            )),\n        )),\n        Err(crate::service::ServerError::SchemaValidation { issues }) => Err(\n            CallError::Validation(Box::new(crate::client::ValidationFailure::Schema(\n""",
)
replace_once(
    connection,
    """                    context: None,\n                    trace_id: None,\n                },\n            )),\n        ),\n        Err(error) => Err(CallError::Protocol(crate::client::ProtocolError::new(\n""",
    """                    context: None,\n                    trace_id: None,\n                },\n            ))),\n        ),\n        Err(error) => Err(CallError::Protocol(crate::client::ProtocolError::new(\n""",
)

# Transfer-start errors intentionally return the accepted operation reference on an
# upload failure. That reference is large state; store it behind a box instead of
# suppressing the enum-size lint.
operations = "rust/crates/trellis/src/client/operations.rs"
remove_expect(operations, "large_enum_variant", expected=1)
replace_once(
    operations,
    """pub enum OperationTransferStartError<'a, T, D> {\n    Start(TrellisClientError),\n    Upload {\n        operation_ref: OperationRef<'a, T, D>,\n        source: TrellisClientError,\n    },\n}\n""",
    """pub enum OperationTransferStartError<'a, T, D> {\n    Start(TrellisClientError),\n    Upload {\n        operation_ref: Box<OperationRef<'a, T, D>>,\n        source: TrellisClientError,\n    },\n}\n""",
)
replace_once(
    operations,
    """                return Err(OperationTransferStartError::Upload {\n                    operation_ref,\n                    source,\n                })\n""",
    """                return Err(OperationTransferStartError::Upload {\n                    operation_ref: Box::new(operation_ref),\n                    source,\n                })\n""",
)

# Service runtime errors carry a large ServerError and, for event handlers, a large
# transport context. Box those payloads at the error boundary and remove all
# result_large_err/large_enum_variant suppressions made unnecessary by the change.
runtime = "rust/crates/trellis/src/service/runtime_facade.rs"
remove_expect(runtime, "large_enum_variant", expected=1)
removed_result_expects = remove_expect(runtime, "result_large_err")
if removed_result_expects < 1:
    raise RuntimeError("expected ServiceRuntimeError result_large_err expectations to remove")
replace_once(
    runtime,
    "    Server(#[from] ServerError),\n",
    "    Server(Box<ServerError>),\n",
)
replace_once(
    runtime,
    """    EventHandler {\n        /// Handler failure returned by the service implementation.\n        source: ServerError,\n        /// Event metadata observed from the delivered message.\n        context: ServiceEventListenerContext,\n    },\n""",
    """    EventHandler {\n        /// Handler failure returned by the service implementation.\n        source: Box<ServerError>,\n        /// Event metadata observed from the delivered message.\n        context: Box<ServiceEventListenerContext>,\n    },\n""",
)
replace_once(
    runtime,
    """}\n\n/// Options for registering a service event listener.\n""",
    """}\n\nimpl From<ServerError> for ServiceRuntimeError {\n    fn from(source: ServerError) -> Self {\n        Self::Server(Box::new(source))\n    }\n}\n\n/// Options for registering a service event listener.\n""",
)
replace_once(
    runtime,
    "return Err(ServiceRuntimeError::EventHandler { source, context });",
    "return Err(ServiceRuntimeError::EventHandler {\n                            source: Box::new(source),\n                            context: Box::new(context),\n                        });",
)
replace_once(
    runtime,
    ".map_err(|source| ServiceRuntimeError::EventHandler { source, context })",
    ".map_err(|source| ServiceRuntimeError::EventHandler {\n                    source: Box::new(source),\n                    context: Box::new(context),\n                })",
)
replace_once(
    runtime,
    ".map_err(ServiceRuntimeError::Server);",
    ".map_err(ServiceRuntimeError::from);",
)
replace_once(
    runtime,
    ".map_err(ServiceRuntimeError::Server)\n",
    ".map_err(ServiceRuntimeError::from)\n",
)

print(
    "large error transform complete; removed "
    f"{removed_result_expects} ServiceRuntimeError result_large_err expectations"
)
