from __future__ import annotations

from pathlib import Path

PATH = Path("rust/crates/trellis/src/client/authorization/provider_cache.rs")


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


def replace_exact(old: str, new: str, *, count: int = 1) -> None:
    source = PATH.read_text()
    actual = source.count(old)
    if actual != count:
        raise RuntimeError(f"expected {count} matches, found {actual}: {old[:100]!r}")
    PATH.write_text(source.replace(old, new, count))


def replace_function(signature: str, replacement: str) -> None:
    source = PATH.read_text()
    index = source.index(signature)
    start = source.rfind("\n", 0, index) + 1
    opening = source.index("{", index)
    end = matching_brace(source, opening)
    PATH.write_text(source[:start] + replacement.rstrip() + source[end:])


def main() -> None:
    replace_exact(
        "struct PendingContext {\n    lock: tokio::sync::Mutex<()>,\n}\n\n",
        "struct PendingContext {\n    lock: tokio::sync::Mutex<()>,\n}\n\n"
        "struct AuthorizationProviderState {\n"
        "    root: Option<AuthorizationTrustRootV1>,\n"
        "    policy_floor: Option<(AuthorizationVerificationPolicyV1, u64, String)>,\n"
        "    manifest: Option<(VerifiedAuthorizationIssuerManifestV1, String)>,\n"
        "    verified_contexts: HashMap<String, VerifiedAuthorizationContextV1>,\n"
        "    retention_deadlines: HashMap<String, i64>,\n"
        "    revocations: HashMap<String, i64>,\n"
        "    health: AuthorizationProviderCacheHealth,\n"
        "}\n\n",
    )
    replace_exact(
        "    root: Arc<RwLock<Option<AuthorizationTrustRootV1>>>,\n"
        "    policy_floor: Arc<RwLock<Option<(AuthorizationVerificationPolicyV1, u64, String)>>>,\n"
        "    manifest: Arc<RwLock<Option<(VerifiedAuthorizationIssuerManifestV1, String)>>>,\n"
        "    verified_contexts: Arc<RwLock<HashMap<String, VerifiedAuthorizationContextV1>>>,\n"
        "    retention_deadlines: Arc<RwLock<HashMap<String, i64>>>,\n"
        "    revocations: Arc<RwLock<HashMap<String, i64>>>,\n",
        "    state: Arc<RwLock<AuthorizationProviderState>>,\n",
    )
    replace_exact(
        "    health: Arc<RwLock<AuthorizationProviderCacheHealth>>,\n",
        "",
    )

    pending_state = """            state: Arc::new(RwLock::new(AuthorizationProviderState {
                root: None,
                policy_floor: None,
                manifest: None,
                verified_contexts: HashMap::new(),
                retention_deadlines: HashMap::new(),
                revocations: HashMap::new(),
                health: AuthorizationProviderCacheHealth {
                    manifest_revision: 0,
                    revocation_revision: 0,
                    last_update_at: 0,
                    healthy: false,
                },
            })),
"""
    replace_exact(
        "            root: Arc::new(RwLock::new(None)),\n"
        "            policy_floor: Arc::new(RwLock::new(None)),\n"
        "            manifest: Arc::new(RwLock::new(None)),\n"
        "            verified_contexts: Arc::new(RwLock::new(HashMap::new())),\n"
        "            retention_deadlines: Arc::new(RwLock::new(HashMap::new())),\n"
        "            revocations: Arc::new(RwLock::new(HashMap::new())),\n",
        pending_state,
    )
    replace_exact(
        "            health: Arc::new(RwLock::new(AuthorizationProviderCacheHealth {\n"
        "                manifest_revision: 0,\n"
        "                revocation_revision: 0,\n"
        "                last_update_at: 0,\n"
        "                healthy: false,\n"
        "            })),\n",
        "",
        count=2,
    )

    runtime_state = """            state: Arc::new(RwLock::new(AuthorizationProviderState {
                root: Some(trust.root),
                policy_floor: Some((
                    trust.policy,
                    trust.minimum_manifest_generation,
                    trust.minimum_manifest_digest,
                )),
                manifest: Some((trust.manifest, trust.manifest_digest)),
                verified_contexts: HashMap::new(),
                retention_deadlines: HashMap::new(),
                revocations: HashMap::new(),
                health: AuthorizationProviderCacheHealth {
                    manifest_revision: 0,
                    revocation_revision: 0,
                    last_update_at: 0,
                    healthy: false,
                },
            })),
"""
    replace_exact(
        "            root: Arc::new(RwLock::new(Some(trust.root))),\n"
        "            policy_floor: Arc::new(RwLock::new(Some((\n"
        "                trust.policy,\n"
        "                trust.minimum_manifest_generation,\n"
        "                trust.minimum_manifest_digest,\n"
        "            )))),\n"
        "            manifest: Arc::new(RwLock::new(Some((trust.manifest, trust.manifest_digest)))),\n"
        "            verified_contexts: Arc::new(RwLock::new(HashMap::new())),\n"
        "            retention_deadlines: Arc::new(RwLock::new(HashMap::new())),\n"
        "            revocations: Arc::new(RwLock::new(HashMap::new())),\n",
        runtime_state,
    )

    test_state = """            state: Arc::new(RwLock::new(AuthorizationProviderState {
                root: Some(root),
                policy_floor: Some((
                    policy,
                    input.minimum_manifest_generation,
                    manifest_digest.clone(),
                )),
                manifest: Some((manifest, manifest_digest)),
                verified_contexts: HashMap::new(),
                retention_deadlines: HashMap::new(),
                revocations: HashMap::new(),
                health: AuthorizationProviderCacheHealth {
                    manifest_revision: 0,
                    revocation_revision: 0,
                    last_update_at: 0,
                    healthy: true,
                },
            })),
"""
    replace_exact(
        "            root: Arc::new(RwLock::new(Some(root))),\n"
        "            policy_floor: Arc::new(RwLock::new(Some((\n"
        "                policy,\n"
        "                input.minimum_manifest_generation,\n"
        "                manifest_digest.clone(),\n"
        "            )))),\n"
        "            manifest: Arc::new(RwLock::new(Some((manifest, manifest_digest)))),\n"
        "            verified_contexts: Arc::new(RwLock::new(HashMap::new())),\n"
        "            retention_deadlines: Arc::new(RwLock::new(HashMap::new())),\n"
        "            revocations: Arc::new(RwLock::new(HashMap::new())),\n",
        test_state,
    )
    replace_exact(
        "            health: Arc::new(RwLock::new(AuthorizationProviderCacheHealth {\n"
        "                manifest_revision: 0,\n"
        "                revocation_revision: 0,\n"
        "                last_update_at: 0,\n"
        "                healthy: true,\n"
        "            })),\n",
        "",
    )

    replace_function(
        "    pub(crate) fn sync_trust_material(&self)",
        r'''    pub(crate) fn sync_trust_material(&self) -> Result<(), TrellisClientError> {
        let Some(own) = self.own.as_ref() else {
            return Ok(());
        };
        let input = own.provider_trust_input()?;
        let durable = own.durable_state()?;
        let floor_digest = if durable.as_ref().is_some_and(|state| {
            state.trust.minimum_manifest_generation == input.minimum_manifest_generation
        }) {
            durable
                .as_ref()
                .map(|state| state.trust.manifest_digest_at_minimum_generation.clone())
                .ok_or_else(|| {
                    TrellisClientError::Bootstrap("authorization trust floor is unavailable".into())
                })?
        } else {
            let manifest = parse_issuer_manifest_v1(&own.bundle()?.trust.manifest)
                .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?;
            manifest
                .digest()
                .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?
        };
        if floor_digest.is_empty() {
            return Err(TrellisClientError::Bootstrap(
                "authorization trust floor digest is empty".into(),
            ));
        }
        let root = AuthorizationTrustRootV1::parse(&input.root)
            .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?;
        let root_digest = root
            .digest()
            .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?;
        let policy = AuthorizationVerificationPolicyV1::new(
            own.corrected_now_seconds()?,
            input.policy.allowed_clock_skew_seconds,
            input.policy.maximum_context_lifetime_seconds,
            input.policy.maximum_context_bytes,
            input.policy.maximum_permissions,
            input.policy.maximum_capabilities,
            input.minimum_manifest_generation,
        )
        .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?;

        let mut state = self.write_state()?;
        if let Some(current) = state.root.as_ref() {
            if current.authority() != root.authority()
                || current.key_id() != root.key_id()
                || current
                    .digest()
                    .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?
                    != root_digest
            {
                return Err(TrellisClientError::Bootstrap(
                    "authorization trust root changed".into(),
                ));
            }
        }
        if let Some((_, generation, digest)) = state.policy_floor.as_ref() {
            if input.minimum_manifest_generation < *generation {
                return Err(TrellisClientError::Bootstrap(
                    "authorization manifest floor rolled back".into(),
                ));
            }
            if input.minimum_manifest_generation == *generation && floor_digest != *digest {
                return Err(TrellisClientError::Bootstrap(
                    "authorization manifest floor equivocates".into(),
                ));
            }
        }
        state.root = Some(root);
        state.policy_floor = Some((policy, input.minimum_manifest_generation, floor_digest));
        Ok(())
    }''',
    )

    replace_function(
        "    pub(crate) fn health(&self)",
        r'''    pub(crate) fn health(&self) -> Result<AuthorizationProviderCacheHealth, TrellisClientError> {
        Ok(self.read_state()?.health.clone())
    }''',
    )
    replace_function(
        "    pub(crate) fn verified_context_raw(",
        r'''    pub(crate) fn verified_context_raw(
        &self,
        digest: &str,
    ) -> Result<Option<VerifiedAuthorizationContextV1>, TrellisClientError> {
        Ok(self.read_state()?.verified_contexts.get(digest).cloned())
    }''',
    )
    replace_function(
        "    fn active_context_raw(",
        r'''    fn active_context_raw(
        &self,
        digest: &str,
    ) -> Result<Option<VerifiedAuthorizationContextV1>, TrellisClientError> {
        Ok(self.read_state()?.verified_contexts.get(digest).cloned())
    }''',
    )
    replace_function(
        "    pub(crate) fn revocation_time(",
        r'''    pub(crate) fn revocation_time(&self, digest: &str) -> Result<Option<i64>, TrellisClientError> {
        Ok(self.read_state()?.revocations.get(digest).copied())
    }''',
    )
    replace_function(
        "    pub fn apply_runtime_revocation(",
        r'''    pub fn apply_runtime_revocation(
        &self,
        digest: &str,
        revoked_at: i64,
    ) -> Result<(), TrellisClientError> {
        {
            let mut state = self.write_state()?;
            state
                .revocations
                .entry(digest.to_owned())
                .and_modify(|current| *current = (*current).max(revoked_at))
                .or_insert(revoked_at);
        }
        if let Some(own) = self.own.as_ref() {
            if own.context_digest().is_ok_and(|current| current == digest) {
                own.clear()?;
                own.request_refresh();
            }
        }
        Ok(())
    }''',
    )
    replace_function(
        "    pub(crate) fn policy(&self)",
        r'''    pub(crate) fn policy(&self) -> Result<AuthorizationVerificationPolicyV1, TrellisClientError> {
        let policy_floor = self
            .read_state()?
            .policy_floor
            .clone()
            .ok_or_else(|| {
                TrellisClientError::Bootstrap("authorization context unavailable".into())
            })?;
        let mut policy = policy_floor.0;
        policy.now_unix_seconds = self.now_seconds()?;
        policy.minimum_manifest_generation = policy.minimum_manifest_generation.max(policy_floor.1);
        Ok(policy)
    }''',
    )
    replace_function(
        "    fn set_healthy(&self, healthy: bool)",
        r'''    fn set_healthy(&self, healthy: bool) {
        if let Ok(mut state) = self.state.write() {
            state.health.healthy = healthy;
        }
    }''',
    )
    replace_function(
        "    fn record_healthy(&self)",
        r'''    fn record_healthy(&self) {
        let now = self.now_seconds().unwrap_or(0);
        if let Ok(mut state) = self.state.write() {
            state.health.last_update_at = now;
            state.health.healthy = true;
        }
        self.ready.notify_waiters();
    }''',
    )

    replace_function(
        "    async fn observe_manifest(",
        r'''    async fn observe_manifest(
        &self,
        pointer: &ManifestPointer,
        revision: u64,
    ) -> Result<(), TrellisClientError> {
        let (manifest_revision, was_healthy, current_generation, current_digest) = {
            let state = self.read_state()?;
            let (generation, digest) = state
                .manifest
                .as_ref()
                .map(|(manifest, digest)| (manifest.generation(), digest.clone()))
                .unwrap_or((0, String::new()));
            (
                state.health.manifest_revision,
                state.health.healthy,
                generation,
                digest,
            )
        };
        if manifest_revision >= revision {
            return Ok(());
        }
        self.check_manifest_floor(pointer.generation, &pointer.digest)?;
        let initial_snapshot = current_generation == 0;
        if pointer.generation < current_generation {
            return Err(TrellisClientError::Bootstrap(
                "manifest.current rolled back".into(),
            ));
        }
        if pointer.generation == current_generation && pointer.digest != current_digest {
            return Err(TrellisClientError::Bootstrap(
                "manifest.current equivocates at the accepted generation".into(),
            ));
        }

        if pointer.generation > current_generation {
            self.set_healthy(false);
            let Some(registry) = self.registry.as_ref() else {
                return Err(TrellisClientError::Bootstrap(
                    "authorization registry is unavailable".into(),
                ));
            };
            let key = format!("manifest.{}", pointer.generation);
            let value = registry
                .get_manifest(pointer.generation)
                .await?
                .ok_or_else(|| {
                    TrellisClientError::Bootstrap(format!("issuer manifest {key} is missing"))
                })?;
            let json: serde_json::Value = serde_json::from_slice(&value)
                .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?;
            let manifest = parse_issuer_manifest_v1(&json)
                .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?;
            let digest = manifest
                .digest()
                .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?;
            if manifest.unsigned.generation != pointer.generation || digest != pointer.digest {
                return Err(TrellisClientError::Bootstrap(
                    "manifest.current does not match its immutable manifest".into(),
                ));
            }
            let mut policy = self.policy()?;
            policy.now_unix_seconds = self.now_seconds()?;
            policy.minimum_manifest_generation =
                policy.minimum_manifest_generation.max(current_generation);
            let verified = verify_issuer_manifest_v1(&self.root_value()?, &manifest, &policy)
                .map_err(|error| {
                    TrellisClientError::Bootstrap(format!(
                        "issuer manifest {key} is not trusted: {error}"
                    ))
                })?;
            if let Some(own) = self.own.as_ref() {
                own.advance_manifest_floor(pointer.generation, &digest)
                    .await?;
            }

            let now = self.now_seconds().unwrap_or(0);
            {
                let mut state = self.write_state()?;
                let (mut policy, _, _) = state.policy_floor.clone().ok_or_else(|| {
                    TrellisClientError::Bootstrap("authorization context unavailable".into())
                })?;
                policy.minimum_manifest_generation = pointer.generation;
                state.policy_floor = Some((policy, pointer.generation, digest.clone()));
                state.manifest = Some((verified, digest));
                state.verified_contexts.clear();
                state.retention_deadlines.clear();
                state.health.manifest_revision = state.health.manifest_revision.max(revision);
                state.health.last_update_at = now;
                state.health.healthy = false;
            }
            if let Some(own) = self.own.as_ref() {
                own.request_refresh();
            }
            if !initial_snapshot {
                return Err(TrellisClientError::Bootstrap(
                    "issuer manifest advanced; restarting complete provider snapshot".into(),
                ));
            }
            return Ok(());
        }

        let now = self.now_seconds().unwrap_or(0);
        let mut state = self.write_state()?;
        state.health.manifest_revision = state.health.manifest_revision.max(revision);
        state.health.last_update_at = now;
        state.health.healthy = was_healthy;
        Ok(())
    }''',
    )

    replace_function(
        "    fn observe_revocation_value(",
        r'''    fn observe_revocation_value(
        &self,
        key: &str,
        value: &[u8],
        revision: u64,
    ) -> Result<(), TrellisClientError> {
        let digest = key
            .strip_prefix(super::registry::REVOCATION_PREFIX)
            .filter(|digest| !digest.is_empty())
            .ok_or_else(|| {
                TrellisClientError::Bootstrap("authorization revocation key is invalid".into())
            })?;
        let revoked_at = parse_revocation_record(value)?;
        let now = self.now_seconds().unwrap_or(0);
        {
            let mut state = self.write_state()?;
            state
                .revocations
                .entry(digest.to_owned())
                .and_modify(|current| *current = (*current).max(revoked_at))
                .or_insert(revoked_at);
            state.health.revocation_revision = state.health.revocation_revision.max(revision);
            state.health.last_update_at = now;
        }
        if let Some(own) = self.own.as_ref() {
            if own.context_digest().is_ok_and(|current| current == digest) {
                own.clear()?;
                own.request_refresh();
            }
        }
        Ok(())
    }''',
    )
    replace_function(
        "    fn observe_revocation_removal(",
        r'''    fn observe_revocation_removal(
        &self,
        key: &str,
        revision: u64,
    ) -> Result<(), TrellisClientError> {
        let digest = key
            .strip_prefix(super::registry::REVOCATION_PREFIX)
            .filter(|digest| !digest.is_empty())
            .ok_or_else(|| {
                TrellisClientError::Bootstrap("authorization revocation key is invalid".into())
            })?;
        // Revocation is monotonic: registry cleanup must never make a revoked context valid again.
        let _ = digest;
        let now = self.now_seconds().unwrap_or(0);
        let mut state = self.write_state()?;
        state.health.revocation_revision = state.health.revocation_revision.max(revision);
        state.health.last_update_at = now;
        Ok(())
    }''',
    )
    replace_function(
        "    fn prune_contexts(&self, now: i64)",
        r'''    fn prune_contexts(&self, now: i64) -> Result<(), TrellisClientError> {
        let mut state = self.write_state()?;
        let expired = state
            .retention_deadlines
            .iter()
            .filter(|(_, retained_until)| **retained_until <= now)
            .map(|(digest, _)| digest.clone())
            .collect::<Vec<_>>();
        for digest in expired {
            state.retention_deadlines.remove(&digest);
            state.verified_contexts.remove(&digest);
        }
        Ok(())
    }''',
    )
    replace_function(
        "    fn root_value(&self)",
        r'''    fn root_value(&self) -> Result<AuthorizationTrustRootV1, TrellisClientError> {
        self.read_state()?
            .root
            .clone()
            .ok_or_else(|| {
                TrellisClientError::Bootstrap("authorization context unavailable".into())
            })
    }''',
    )
    replace_function(
        "    fn check_manifest_floor(",
        r'''    fn check_manifest_floor(
        &self,
        generation: u64,
        digest: &str,
    ) -> Result<(), TrellisClientError> {
        let state = self.read_state()?;
        let Some((_, minimum_generation, minimum_digest)) = state.policy_floor.as_ref() else {
            return Err(TrellisClientError::Bootstrap(
                "authorization context unavailable".into(),
            ));
        };
        if generation < *minimum_generation {
            return Err(TrellisClientError::Bootstrap(
                "manifest is below the durable authorization floor".into(),
            ));
        }
        if generation == *minimum_generation && digest != minimum_digest {
            return Err(TrellisClientError::Bootstrap(
                "manifest equivocates with the durable authorization floor".into(),
            ));
        }
        Ok(())
    }''',
    )
    replace_function(
        "    pub fn runtime_current_manifest(",
        r'''    pub fn runtime_current_manifest(
        &self,
    ) -> Result<VerifiedAuthorizationIssuerManifestV1, TrellisClientError> {
        self.read_state()?
            .manifest
            .as_ref()
            .map(|(manifest, _)| manifest.clone())
            .ok_or_else(|| TrellisClientError::Bootstrap("provider manifest unavailable".into()))
    }''',
    )

    replace_function(
        "    async fn resolve_context_for(",
        r'''    async fn resolve_context_for(
        &self,
        digest: &str,
        verification_time: i64,
        historical: bool,
    ) -> Result<VerifiedAuthorizationContextV1, TrellisClientError> {
        self.prune_contexts(self.now_seconds()?)?;
        let known = self.active_context_raw(digest)?;
        if let Some(context) = known {
            return Ok(context);
        }
        let manifest_generation = self
            .read_state()?
            .manifest
            .as_ref()
            .map(|(manifest, _)| manifest.generation())
            .ok_or_else(|| {
                TrellisClientError::Bootstrap("authorization manifest is unavailable".into())
            })?;
        let pending = {
            let mut in_flight = self.in_flight.lock().map_err(|_| {
                TrellisClientError::Bootstrap("provider resolution lock poisoned".into())
            })?;
            Arc::clone(in_flight.entry(digest.to_owned()).or_insert_with(|| {
                Arc::new(PendingContext {
                    lock: tokio::sync::Mutex::new(()),
                })
            }))
        };
        let _guard = pending.lock.lock().await;
        let known = self.active_context_raw(digest)?;
        if let Some(context) = known {
            return Ok(context);
        }
        let outcome = self
            .resolve_context_once(digest, verification_time, historical)
            .await;
        self.drop_in_flight(digest, &pending);
        let context = outcome?;
        let mut state = self.write_state()?;
        if state
            .manifest
            .as_ref()
            .map(|(manifest, _)| manifest.generation())
            != Some(manifest_generation)
        {
            return Err(TrellisClientError::Bootstrap(
                "authorization manifest advanced during context resolution".into(),
            ));
        }
        if !historical {
            state
                .retention_deadlines
                .insert(digest.to_owned(), context.expires_at());
            state
                .verified_contexts
                .insert(digest.to_owned(), context.clone());
        }
        Ok(context)
    }''',
    )

    replace_exact(
        "            let current_generation = self\n"
        "                .manifest\n"
        "                .read()\n"
        "                .map_err(|_| {\n"
        "                    TrellisClientError::Bootstrap(\"provider manifest lock poisoned\".into())\n"
        "                })?\n"
        "                .as_ref()\n"
        "                .map(|(manifest, _)| manifest.generation())\n"
        "                .ok_or_else(|| {\n"
        "                    TrellisClientError::Bootstrap(\"provider manifest unavailable\".into())\n"
        "                })?;\n",
        "            let current_generation = self\n"
        "                .read_state()?\n"
        "                .manifest\n"
        "                .as_ref()\n"
        "                .map(|(manifest, _)| manifest.generation())\n"
        "                .ok_or_else(|| {\n"
        "                    TrellisClientError::Bootstrap(\"provider manifest unavailable\".into())\n"
        "                })?;\n",
    )
    replace_exact(
        "        if let Some((manifest, digest)) = self\n"
        "            .manifest\n"
        "            .read()\n"
        "            .map_err(|_| TrellisClientError::Bootstrap(\"provider manifest lock poisoned\".into()))?\n"
        "            .as_ref()\n"
        "            .filter(|(manifest, _)| manifest.generation() == generation)\n"
        "            .cloned()\n",
        "        if let Some((manifest, digest)) = self\n"
        "            .read_state()?\n"
        "            .manifest\n"
        "            .as_ref()\n"
        "            .filter(|(manifest, _)| manifest.generation() == generation)\n"
        "            .cloned()\n",
    )
    replace_function(
        "    pub(crate) fn inject_verified_for_test(",
        r'''    pub(crate) fn inject_verified_for_test(
        &self,
        digest: &str,
        verified: VerifiedAuthorizationContextV1,
        revoked_at: Option<i64>,
    ) -> Result<(), TrellisClientError> {
        let mut state = self.write_state()?;
        state
            .verified_contexts
            .insert(digest.to_owned(), verified);
        if let Some(revoked_at) = revoked_at {
            state.revocations.insert(digest.to_owned(), revoked_at);
        }
        Ok(())
    }''',
    )

    # Add the single lock access boundary before the impl closes.
    source = PATH.read_text()
    marker = "    /// Install an already-verified snapshot for unit tests without registry I/O.\n"
    insert = r'''    fn read_state(
        &self,
    ) -> Result<std::sync::RwLockReadGuard<'_, AuthorizationProviderState>, TrellisClientError> {
        self.state
            .read()
            .map_err(|_| TrellisClientError::Bootstrap("provider state lock poisoned".into()))
    }

    fn write_state(
        &self,
    ) -> Result<std::sync::RwLockWriteGuard<'_, AuthorizationProviderState>, TrellisClientError> {
        self.state
            .write()
            .map_err(|_| TrellisClientError::Bootstrap("provider state lock poisoned".into()))
    }

'''
    if source.count(marker) != 1:
        raise RuntimeError("failed to locate state-helper insertion point")
    PATH.write_text(source.replace(marker, insert + marker, 1))

    # No old fragmented state fields or poison messages should remain.
    source = PATH.read_text()
    for token in [
        "self.root",
        "self.policy_floor",
        "self.manifest",
        "self.verified_contexts",
        "self.retention_deadlines",
        "self.revocations",
        "self.health",
        "provider trust lock poisoned",
        "provider manifest lock poisoned",
        "provider context cache lock poisoned",
        "provider retention lock poisoned",
        "provider revocation lock poisoned",
        "provider health lock poisoned",
    ]:
        if token in source:
            raise RuntimeError(f"fragmented provider state remains: {token}")

    source = source.replace(
        '        "provider trust lock poisoned",\n'
        '        "provider manifest lock poisoned",\n'
        '        "provider context cache lock poisoned",\n'
        '        "provider resolution lock poisoned",\n'
        '        "provider retention lock poisoned",\n',
        '        "provider state lock poisoned",\n'
        '        "provider resolution lock poisoned",\n',
    )
    PATH.write_text(source)


if __name__ == "__main__":
    main()
