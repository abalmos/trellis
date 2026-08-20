from pathlib import Path


def replace_once(path: str, old: str, new: str) -> None:
    p = Path(path)
    text = p.read_text()
    count = text.count(old)
    if count != 1:
        raise RuntimeError(f"{path}: expected one occurrence, found {count}: {old[:120]!r}")
    p.write_text(text.replace(old, new, 1))


# Keep generated caller errors small regardless of the size of a contract-declared
# error payload. These are cold error paths, so one allocation is preferable to
# making every Result-returning caller carry a large enum inline.
replace_once(
    "rust/crates/trellis/src/client/error.rs",
    '''    Declared(E),
    /// Well-formed remote error not declared by this action.
    #[error("remote error: {}", .0.format_human())]
    Remote(RemoteErrorPayload),
    /// Standard validation failure.
    #[error("validation failed")]
    Validation(ValidationFailure),
''',
    '''    Declared(Box<E>),
    /// Well-formed remote error not declared by this action.
    #[error("remote error: {}", .0.format_human())]
    Remote(Box<RemoteErrorPayload>),
    /// Standard validation failure.
    #[error("validation failed")]
    Validation(Box<ValidationFailure>),
''',
)
replace_once(
    "rust/crates/trellis/src/client/error.rs",
    "                Ok(Some(error)) => Self::Declared(error),",
    "                Ok(Some(error)) => Self::Declared(Box::new(error)),",
)
replace_once(
    "rust/crates/trellis/src/client/error.rs",
    "                    Ok(Some(error)) => Self::Validation(ValidationFailure::Schema(error)),",
    "                    Ok(Some(error)) => Self::Validation(Box::new(ValidationFailure::Schema(error))),",
)
replace_once(
    "rust/crates/trellis/src/client/error.rs",
    "                        Ok(Some(error)) => Self::Validation(ValidationFailure::Validation(error)),",
    "                        Ok(Some(error)) => Self::Validation(Box::new(ValidationFailure::Validation(error))),",
)
replace_once(
    "rust/crates/trellis/src/client/error.rs",
    "                        Ok(None) if payload.error_type().is_some() => Self::Remote(payload),",
    "                        Ok(None) if payload.error_type().is_some() => Self::Remote(Box::new(payload)),",
)

# Transfer-start errors intentionally return the accepted live reference on upload
# failure. Box the cold-path payloads rather than suppressing the enum-size lint.
replace_once(
    "rust/crates/trellis/src/client/operations.rs",
    '''/// Error returned when starting or uploading an operation transfer fails.
#[expect(
    clippy::large_enum_variant,
    reason = "upload failures return the live operation reference to the caller"
)]
pub enum OperationTransferStartError<'a, T, D> {
    Start(TrellisClientError),
    Upload {
        operation_ref: OperationRef<'a, T, D>,
        source: TrellisClientError,
    },
}
''',
    '''/// Error returned when starting or uploading an operation transfer fails.
pub enum OperationTransferStartError<'a, T, D> {
    Start(Box<TrellisClientError>),
    Upload {
        operation_ref: Box<OperationRef<'a, T, D>>,
        source: Box<TrellisClientError>,
    },
}
''',
)
replace_once(
    "rust/crates/trellis/src/client/operations.rs",
    ".map_err(OperationTransferStartError::Start)?;",
    ".map_err(|source| OperationTransferStartError::Start(Box::new(source)))?;",
)
replace_once(
    "rust/crates/trellis/src/client/operations.rs",
    '''                return Err(OperationTransferStartError::Upload {
                    operation_ref,
                    source,
                })
''',
    '''                return Err(OperationTransferStartError::Upload {
                    operation_ref: Box::new(operation_ref),
                    source: Box::new(source),
                })
''',
)

# Service facade failures keep rich typed diagnostics, but they do not need to be
# stored inline in every Result. Box only the large error payloads and preserve
# ergonomic `?` conversion with explicit From impls.
replace_once(
    "rust/crates/trellis/src/service/runtime_facade.rs",
    '''/// Errors returned by the high-level service runtime facade.
#[derive(Debug, thiserror::Error)]
#[expect(
    clippy::large_enum_variant,
    reason = "runtime errors retain typed handler context for operator diagnostics"
)]
pub enum ServiceRuntimeError {
    /// Client-side bootstrap, transport, or outbound RPC failure.
    #[error(transparent)]
    Client(#[from] TrellisClientError),

    /// Server-side handler, auth-validation, or runtime-loop failure.
    #[error(transparent)]
    Server(#[from] ServerError),

    /// A service event listener handler failed while processing a concrete event message.
    #[error("event handler failed: {source}")]
    EventHandler {
        /// Handler failure returned by the service implementation.
        source: ServerError,
        /// Event metadata observed from the delivered message.
        context: ServiceEventListenerContext,
    },
''',
    '''/// Errors returned by the high-level service runtime facade.
#[derive(Debug, thiserror::Error)]
pub enum ServiceRuntimeError {
    /// Client-side bootstrap, transport, or outbound RPC failure.
    #[error(transparent)]
    Client(Box<TrellisClientError>),

    /// Server-side handler, auth-validation, or runtime-loop failure.
    #[error(transparent)]
    Server(Box<ServerError>),

    /// A service event listener handler failed while processing a concrete event message.
    #[error("event handler failed: {source}")]
    EventHandler {
        /// Handler failure returned by the service implementation.
        source: Box<ServerError>,
        /// Event metadata observed from the delivered message.
        context: Box<ServiceEventListenerContext>,
    },
''',
)

anchor = '''}

/// Options for registering a service event listener.
'''
conversions = '''}

impl From<TrellisClientError> for ServiceRuntimeError {
    fn from(source: TrellisClientError) -> Self {
        Self::Client(Box::new(source))
    }
}

impl From<ServerError> for ServiceRuntimeError {
    fn from(source: ServerError) -> Self {
        Self::Server(Box::new(source))
    }
}

/// Options for registering a service event listener.
'''
replace_once("rust/crates/trellis/src/service/runtime_facade.rs", anchor, conversions)

path = Path("rust/crates/trellis/src/service/runtime_facade.rs")
text = path.read_text()
old = "ServiceRuntimeError::EventHandler { source, context }"
count = text.count(old)
if count != 2:
    raise RuntimeError(f"runtime_facade.rs: expected two EventHandler constructions, found {count}")
text = text.replace(
    old,
    "ServiceRuntimeError::EventHandler { source: Box::new(source), context: Box::new(context) }",
)
path.write_text(text)

print("large Result error payload cleanup transform complete")
