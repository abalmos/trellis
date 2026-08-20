from __future__ import annotations

import json
from pathlib import Path


def replace_exact(path: str, old: str, new: str = "", *, count: int = 1) -> None:
    file = Path(path)
    source = file.read_text()
    actual = source.count(old)
    if actual != count:
        raise RuntimeError(f"{path}: expected {count} occurrences, found {actual}: {old[:80]!r}")
    file.write_text(source.replace(old, new, count))


def matching_brace(source: str, opening: int) -> int:
    depth = 0
    state = "code"
    block_depth = 0
    i = opening
    while i < len(source):
        c = source[i]
        n = source[i + 1] if i + 1 < len(source) else ""
        if state == "code":
            if c == '"':
                state = "string"
            elif c == "'":
                state = "char"
            elif c == "/" and n == "/":
                state = "line_comment"
                i += 1
            elif c == "/" and n == "*":
                state = "block_comment"
                block_depth = 1
                i += 1
            elif c == "{":
                depth += 1
            elif c == "}":
                depth -= 1
                if depth == 0:
                    return i + 1
        elif state == "string":
            if c == "\\":
                i += 1
            elif c == '"':
                state = "code"
        elif state == "char":
            if c == "\\":
                i += 1
            elif c == "'":
                state = "code"
        elif state == "line_comment":
            if c == "\n":
                state = "code"
        elif state == "block_comment":
            if c == "/" and n == "*":
                block_depth += 1
                i += 1
            elif c == "*" and n == "/":
                block_depth -= 1
                i += 1
                if block_depth == 0:
                    state = "code"
        i += 1
    raise RuntimeError("unclosed Rust block")


def remove_rust_function(path: str, signature: str) -> None:
    file = Path(path)
    source = file.read_text()
    index = source.index(signature)
    start = source.rfind("\n", 0, index) + 1
    while start > 0:
        previous_end = start - 1
        previous_start = source.rfind("\n", 0, previous_end) + 1
        previous = source[previous_start:previous_end].strip()
        if previous.startswith("///") or previous.startswith("#["):
            start = previous_start
        else:
            break
    opening = source.index("{", index)
    end = matching_brace(source, opening)
    while end < len(source) and source[end] == "\n":
        end += 1
    file.write_text(source[:start] + source[end:])


def replace_rust_function(path: str, signature: str, replacement: str) -> None:
    file = Path(path)
    source = file.read_text()
    index = source.index(signature)
    start = source.rfind("\n#[tokio::test]", 0, index)
    if start < 0:
        raise RuntimeError(f"missing tokio test attribute for {signature}")
    start += 1
    opening = source.index("{", index)
    end = matching_brace(source, opening)
    while end < len(source) and source[end] == "\n":
        end += 1
    file.write_text(source[:start] + replacement.rstrip() + "\n\n" + source[end:])


def update_matrix() -> None:
    file = Path("integration/rust-runtime-test-matrix.json")
    source = file.read_text()
    marker = '    {\n      "id": "event-consumers.authorization-failures-redeliver-or-term"'
    start = source.index(marker)
    object_start = source.index("{", start)
    case, consumed = json.JSONDecoder().raw_decode(source[object_start:])
    object_end = object_start + consumed

    case["id"] = "event-consumers.invalid-authorization-proof-terms"
    case["title"] = "Invalid authorization event proofs terminate permanently"
    case["coverage"] = ["events", "event-consumers", "authorization", "term"]
    case["description"] = (
        "A legitimate signed durable event reaches its handler, while republishing the same "
        "event with a corrupted proof is permanently terminated and never reaches the handler."
    )
    case["scenario"]["given"] = [
        "a durable event consumer is ready and a publisher can emit a valid signed event"
    ]
    case["scenario"]["when"] = [
        "the valid event is delivered, then its raw payload and headers are republished with a corrupted proof"
    ]
    case["scenario"]["then"] = [
        "the legitimate event reaches the handler exactly once",
        "the corrupted event is permanently terminated",
        "the corrupted event never reaches the handler",
    ]
    case["implementations"]["rust"]["function"] = (
        "event_consumers_invalid_authorization_proof_terms"
    )

    rendered = json.dumps(case, indent=2)
    rendered = "\n".join("    " + line for line in rendered.splitlines())
    updated = source[:object_start] + rendered + source[object_end:]
    json.loads(updated)
    file.write_text(updated)


def main() -> None:
    provider = "rust/crates/trellis/src/client/authorization/provider_cache.rs"
    replace_exact(
        provider,
        '#[cfg(feature = "integration-test-scoping")]\nuse std::sync::atomic::AtomicBool;\n',
    )
    replace_exact(
        provider,
        '    #[cfg(feature = "integration-test-scoping")]\n    fail_next_context_read: Arc<AtomicBool>,\n'
        '    #[cfg(feature = "integration-test-scoping")]\n    fail_next_readiness_check: Arc<AtomicBool>,\n',
    )
    replace_exact(
        provider,
        '            #[cfg(feature = "integration-test-scoping")]\n'
        '            fail_next_context_read: Arc::new(AtomicBool::new(false)),\n'
        '            #[cfg(feature = "integration-test-scoping")]\n'
        '            fail_next_readiness_check: Arc::new(AtomicBool::new(false)),\n',
        count=3,
    )
    remove_rust_function(provider, "pub fn integration_test_fail_next_context_read(&self)")
    remove_rust_function(provider, "pub fn integration_test_fail_next_readiness_check(&self)")
    remove_rust_function(
        provider, "pub(crate) fn integration_test_take_readiness_failure(&self)"
    )
    replace_exact(
        provider,
        '        #[cfg(feature = "integration-test-scoping")]\n'
        '        if self.fail_next_context_read.swap(false, Ordering::Relaxed) {\n'
        '            return Err(TrellisClientError::NatsRequest(\n'
        '                "injected authorization context read failure".into(),\n'
        '            ));\n'
        '        }\n',
    )

    validator = "rust/crates/trellis/src/service/local_validator.rs"
    replace_exact(
        validator,
        '        #[cfg(feature = "integration-test-scoping")]\n'
        '        if provider.integration_test_take_readiness_failure() {\n'
        '            return Err(EventVerificationFailure::retryable(format!(\n'
        '                "local authorization context unavailable for {subject}"\n'
        '            )));\n'
        '        }\n',
    )

    replacement = r'''#[tokio::test]
async fn event_consumers_invalid_authorization_proof_terms() {
    assert_runtime_case_registered(
        "event-consumers.invalid-authorization-proof-terms",
        "event-consumers",
        "event_consumers",
    );

    let (runtime, bootstrap_url, mut admin) = start_runtime().await;
    let source_contract = test_contract(SOURCE_API_SOURCE_JSON);
    admin
        .provision_service_instance(&bootstrap_url, &source_contract, Some("source"), None)
        .await
        .expect("approve source contract");
    let consumer = connect_consumer(
        &mut admin,
        runtime.trellis_url(),
        &bootstrap_url,
        PARALLEL_DEPENDENCY_CONSUMER_JSON,
        "event-consumers-invalid-auth-proof-rust",
    )
    .await;
    let ack_observer = runtime
        .start_jetstream_ack_observer()
        .await
        .expect("start JetStream ACK observer");
    let observed = Arc::new(Mutex::new(Vec::<String>::new()));
    let listener = consumer
        .listen_event_with_api_id::<SourcePingedEvent, _, _>(
            SOURCE_CONTRACT_ID,
            {
                let observed = Arc::clone(&observed);
                move |event, _context| {
                    let observed = Arc::clone(&observed);
                    async move {
                        observed.lock().await.push(event.id);
                        Ok(())
                    }
                }
            },
            ServiceEventListenOptions {
                mode: ServiceEventListenerMode::Durable,
                group: Some("ingest".to_owned()),
                durable_name: None,
                concurrency: 1,
            },
        )
        .await
        .expect("start authorization listener");
    wait_for_waiting_count(&runtime, SourcePingedEvent::SUBJECT, 1).await;
    let durable = matching_consumers(&runtime, SourcePingedEvent::SUBJECT)
        .await
        .remove(0);
    let durable_name = consumer_name(&durable).to_owned();
    let mut raw_events = consumer
        .integration_test_nats()
        .subscribe(durable.filter_subjects[0].clone())
        .await
        .expect("subscribe to raw test events");

    let publisher = admin
        .connect_client(&bootstrap_url, &publisher_contract())
        .await
        .expect("connect event publisher");
    publisher
        .publish::<SourcePingedEvent>(&EventRecord {
            id: "rust-event-auth-valid".into(),
            value: "valid".into(),
        })
        .await
        .expect("publish valid signed event");
    let raw = tokio::time::timeout(Duration::from_secs(5), raw_events.next())
        .await
        .expect("timed out waiting for raw event")
        .expect("raw event subscription ended");
    wait_for_observed_vec_id(&observed, "rust-event-auth-valid").await;

    let mut headers = raw.headers.expect("published event headers");
    headers.insert("proof", "invalid-event-proof");
    headers.insert("Nats-Msg-Id", format!("evt_invalid_{}", ulid::Ulid::new()));
    publisher
        .integration_test_nats()
        .publish_with_headers(raw.subject, headers, raw.payload)
        .await
        .expect("publish cryptographically invalid event");
    wait_for_ack_payload(&ack_observer, &durable_name, "+TERM").await;
    assert_eq!(
        observed.lock().await.clone(),
        vec!["rust-event-auth-valid".to_owned()]
    );

    listener.abort();
    let _ = listener.await;
    ack_observer.stop().await;
}'''
    replace_rust_function(
        "rust/crates/trellis/tests/integration/event_consumers.rs",
        "async fn event_consumers_authorization_failures_redeliver_or_term()",
        replacement,
    )
    update_matrix()


if __name__ == "__main__":
    main()
