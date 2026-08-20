from pathlib import Path

path = Path("rust/crates/trellis/src/service/runtime_facade.rs")
source = path.read_text()


def replace(old: str, new: str, count: int = 1) -> None:
    global source
    actual = source.count(old)
    if actual != count:
        raise RuntimeError(f"expected {count} matches, found {actual}: {old[:120]!r}")
    source = source.replace(old, new, count)


replace("use tokio::sync::Mutex;\n", "")
replace(
    "type SharedDurableEventListeners =\n"
    "    Arc<Mutex<BTreeMap<DurableEventListenerKey, SharedDurableEventListener>>>;\n",
    "type SharedDurableEventListeners =\n"
    "    Arc<StdMutex<BTreeMap<DurableEventListenerKey, SharedDurableEventListener>>>;\n",
)
replace(
    "impl Drop for ServiceEventListenerRegistryCleanup {\n"
    "    fn drop(&mut self) {\n"
    "        spawn_service_event_listeners_cleanup(Arc::clone(&self.event_listeners));\n"
    "    }\n"
    "}\n",
    "impl Drop for ServiceEventListenerRegistryCleanup {\n"
    "    fn drop(&mut self) {\n"
    "        remove_service_event_listeners(&self.event_listeners);\n"
    "    }\n"
    "}\n",
)
replace(
    "                spawn_service_event_listener_cleanup(registration);",
    "                remove_service_event_listener_registration(registration);",
    count=2,
)
replace(
    "    let mut listeners = event_listeners.lock().await;\n",
    "    let mut listeners = lock_service_event_listeners(&event_listeners);\n",
)

old_cleanup = '''async fn remove_service_event_listener_registration(
    registration: ServiceEventListenerRegistration,
) {
    let mut listeners = registration.event_listeners.lock().await;
    let Some(listener) = listeners.get_mut(&registration.key) else {
        return;
    };
    if let Some(handlers) = listener.handlers.get_mut(&registration.subject) {
        handlers.remove(&registration.handler_id);
        if handlers.is_empty() {
            listener.handlers.remove(&registration.subject);
        }
    }
    if listener.handlers.values().all(BTreeMap::is_empty) {
        let listener = listeners.remove(&registration.key);
        if let Some(listener) = listener {
            for handle in listener.pull_abort_handles {
                handle.abort();
            }
        }
    }
}

fn spawn_service_event_listener_cleanup(registration: ServiceEventListenerRegistration) {
    if let Ok(handle) = tokio::runtime::Handle::try_current() {
        handle.spawn(remove_service_event_listener_registration(registration));
    }
}

async fn remove_service_event_listeners(event_listeners: SharedDurableEventListeners) {
    let listeners = std::mem::take(&mut *event_listeners.lock().await);
    for (_, listener) in listeners {
        for handle in listener.pull_abort_handles {
            handle.abort();
        }
    }
}
'''
new_cleanup = '''fn lock_service_event_listeners(
    event_listeners: &SharedDurableEventListeners,
) -> std::sync::MutexGuard<'_, BTreeMap<DurableEventListenerKey, SharedDurableEventListener>> {
    event_listeners
        .lock()
        .unwrap_or_else(|error| error.into_inner())
}

fn remove_service_event_listener_registration(registration: ServiceEventListenerRegistration) {
    let mut listeners = lock_service_event_listeners(&registration.event_listeners);
    let Some(listener) = listeners.get_mut(&registration.key) else {
        return;
    };
    if let Some(handlers) = listener.handlers.get_mut(&registration.subject) {
        handlers.remove(&registration.handler_id);
        if handlers.is_empty() {
            listener.handlers.remove(&registration.subject);
        }
    }
    if listener.handlers.values().all(BTreeMap::is_empty) {
        if let Some(listener) = listeners.remove(&registration.key) {
            for handle in listener.pull_abort_handles {
                handle.abort();
            }
        }
    }
}

fn remove_service_event_listeners(event_listeners: &SharedDurableEventListeners) {
    let listeners = std::mem::take(&mut *lock_service_event_listeners(event_listeners));
    for (_, listener) in listeners {
        for handle in listener.pull_abort_handles {
            handle.abort();
        }
    }
}
'''
replace(old_cleanup, new_cleanup)

replace(
    "durable_listener_ready(&event_listeners, &key).await",
    "durable_listener_ready(&event_listeners, &key)",
    count=4,
)
replace(
    '''            let handlers = event_listeners
                .lock()
                .await
                .get(&key)
                .and_then(|listener| listener.handlers.get(message.subject()).cloned())
                .unwrap_or_default();''',
    '''            let handlers = lock_service_event_listeners(&event_listeners)
                .get(&key)
                .and_then(|listener| listener.handlers.get(message.subject()).cloned())
                .unwrap_or_default();''',
)
replace(
    '''fn spawn_service_event_listeners_cleanup(event_listeners: SharedDurableEventListeners) {
    if let Ok(handle) = tokio::runtime::Handle::try_current() {
        handle.spawn(remove_service_event_listeners(event_listeners));
    }
}

async fn durable_listener_ready(
    event_listeners: &SharedDurableEventListeners,
    key: &DurableEventListenerKey,
) -> bool {
    event_listeners
        .lock()
        .await
        .get(key)
        .map(|listener| {
            listener
                .expected_subjects
                .iter()
                .all(|subject| listener.handlers.contains_key(subject))
        })
        .unwrap_or(false)
}
''',
    '''fn durable_listener_ready(
    event_listeners: &SharedDurableEventListeners,
    key: &DurableEventListenerKey,
) -> bool {
    lock_service_event_listeners(event_listeners)
        .get(key)
        .map(|listener| {
            listener
                .expected_subjects
                .iter()
                .all(|subject| listener.handlers.contains_key(subject))
        })
        .unwrap_or(false)
}
''',
)

for forbidden in (
    "spawn_service_event_listener_cleanup",
    "spawn_service_event_listeners_cleanup",
    "remove_service_event_listener_registration(\n",
    "event_listeners.lock().await",
    "tokio::sync::Mutex",
):
    if forbidden in source:
        raise RuntimeError(f"obsolete listener teardown path remains: {forbidden}")

# The direct call form is expected and the old async signature must be gone.
if "fn remove_service_event_listener_registration(registration:" not in source:
    raise RuntimeError("synchronous registration cleanup was not installed")
if "async fn remove_service_event_listener_registration" in source:
    raise RuntimeError("async registration cleanup remains")

path.write_text(source)
