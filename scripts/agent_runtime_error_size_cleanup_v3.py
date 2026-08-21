from pathlib import Path
import re


def replace_once(path: str, old: str, new: str) -> None:
    p = Path(path)
    text = p.read_text()
    count = text.count(old)
    if count != 1:
        raise RuntimeError(f"{path}: expected one occurrence, found {count}")
    p.write_text(text.replace(old, new, 1))


error_path = Path("rust/crates/trellis/src/service/error.rs")
error_text = error_path.read_text()
error_text = error_text.replace(
    """    OperationMismatch {\n        operation_id: String,\n        expected_service: String,\n        expected_operation: String,\n        actual_service: String,\n        actual_operation: String,\n    },\n""",
    """    OperationMismatch {\n        operation_id: Box<str>,\n        expected_service: String,\n        expected_operation: String,\n        actual_service: String,\n        actual_operation: String,\n    },\n""",
    1,
)
error_text = error_text.replace(
    """    BootstrapBindingMismatch {\n        service_name: String,\n        expected_contract_id: String,\n        expected_contract_digest: String,\n        actual_contract_id: String,\n        actual_contract_digest: String,\n    },\n""",
    """    BootstrapBindingMismatch {\n        service_name: Box<str>,\n        expected_contract_id: String,\n        expected_contract_digest: String,\n        actual_contract_id: String,\n        actual_contract_digest: String,\n    },\n""",
    1,
)
error_text = error_text.replace(
    """    BootstrapAuthContractMismatch {\n        service_name: String,\n        expected_contract_id: String,\n        expected_contract_digest: String,\n        actual_contract_id: String,\n        actual_contract_digest: String,\n    },\n""",
    """    BootstrapAuthContractMismatch {\n        service_name: Box<str>,\n        expected_contract_id: String,\n        expected_contract_digest: String,\n        actual_contract_id: String,\n        actual_contract_digest: String,\n    },\n""",
    1,
)
error_path.write_text(error_text)

operations_path = Path("rust/crates/trellis/src/service/operations.rs")
operations = operations_path.read_text()
pattern = re.compile(r"(ServerError::OperationMismatch \{\n\s*)operation_id: ([^\n]+),")
operations, count = pattern.subn(
    lambda match: f"{match.group(1)}operation_id: ({match.group(2)}).into_boxed_str(),",
    operations,
)
if count == 0:
    raise RuntimeError("no OperationMismatch constructors found")
operations_path.write_text(operations)

bindings_path = Path("rust/crates/trellis/src/service/bindings.rs")
bindings = bindings_path.read_text()
pattern = re.compile(r"(ServerError::BootstrapBindingMismatch \{\n\s*)service_name: ([^\n]+),")
bindings, count = pattern.subn(
    lambda match: f"{match.group(1)}service_name: ({match.group(2)}).into_boxed_str(),",
    bindings,
)
if count != 1:
    raise RuntimeError(f"expected one BootstrapBindingMismatch constructor, found {count}")
bindings_path.write_text(bindings)

projector = Path("rust/crates/eventlog-runtime/src/projector.rs")
text = projector.read_text()
text = text.replace(
    """#[expect(\n    clippy::result_large_err,\n    reason = \"ServerError preserves typed projector diagnostics\"\n)]\n""",
    "",
    1,
)
old_match = """    match project_message(message, verifier).await {\n        Ok((message, event)) => {\n            match tokio::task::spawn_blocking(move || store.insert_event(&event)).await {\n                Ok(Ok(())) => {\n                    let _ = message.ack().await;\n                }\n                Ok(Err(error)) => {\n                    tracing::warn!(%error, subject = %message.subject, \"retrying Event Log persistence\");\n                    let _ = message\n                        .ack_with(AckKind::Nak(Some(Duration::from_secs(5))))\n                        .await;\n                }\n                Err(error) => {\n                    tracing::warn!(%error, subject = %message.subject, \"retrying Event Log persistence task\");\n                    let _ = message\n                        .ack_with(AckKind::Nak(Some(Duration::from_secs(5))))\n                        .await;\n                }\n            }\n        }\n        Err((message, EventVerificationFailure::Retryable(error))) => {\n            tracing::warn!(%error, subject = %message.subject, \"retrying temporarily unverifiable event log message\");\n            let _ = message\n                .ack_with(AckKind::Nak(Some(Duration::from_secs(5))))\n                .await;\n        }\n        Err((message, EventVerificationFailure::Rejected(error))) => {\n            tracing::warn!(%error, subject = %message.subject, \"dropping rejected event log message\");\n            let _ = message.ack().await;\n        }\n    }\n"""
new_match = """    match project_message_inner(&message, verifier).await {\n        Ok(event) => {\n            match tokio::task::spawn_blocking(move || store.insert_event(&event)).await {\n                Ok(Ok(())) => {\n                    let _ = message.ack().await;\n                }\n                Ok(Err(error)) => {\n                    tracing::warn!(%error, subject = %message.subject, \"retrying Event Log persistence\");\n                    let _ = message\n                        .ack_with(AckKind::Nak(Some(Duration::from_secs(5))))\n                        .await;\n                }\n                Err(error) => {\n                    tracing::warn!(%error, subject = %message.subject, \"retrying Event Log persistence task\");\n                    let _ = message\n                        .ack_with(AckKind::Nak(Some(Duration::from_secs(5))))\n                        .await;\n                }\n            }\n        }\n        Err(EventVerificationFailure::Retryable(error)) => {\n            tracing::warn!(%error, subject = %message.subject, \"retrying temporarily unverifiable event log message\");\n            let _ = message\n                .ack_with(AckKind::Nak(Some(Duration::from_secs(5))))\n                .await;\n        }\n        Err(EventVerificationFailure::Rejected(error)) => {\n            tracing::warn!(%error, subject = %message.subject, \"dropping rejected event log message\");\n            let _ = message.ack().await;\n        }\n    }\n"""
if text.count(old_match) != 1:
    raise RuntimeError("eventlog process_message match changed")
text = text.replace(old_match, new_match, 1)
old_wrapper = """async fn project_message(\n    message: jetstream::Message,\n    verifier: EventVerifier,\n) -> Result<(jetstream::Message, ProjectedEvent), (jetstream::Message, EventVerificationFailure)> {\n    match project_message_inner(&message, verifier).await {\n        Ok(event) => Ok((message, event)),\n        Err(error) => Err((message, error)),\n    }\n}\n\n"""
if text.count(old_wrapper) != 1:
    raise RuntimeError("eventlog project_message wrapper changed")
projector.write_text(text.replace(old_wrapper, "", 1))

print("Rust 1.98 error-size cleanup v3 applied")
