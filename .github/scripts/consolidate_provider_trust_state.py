from pathlib import Path

PATH = Path("rust/crates/trellis/src/client/authorization/provider_cache.rs")
source = PATH.read_text()


def replace_once(old: str, new: str, label: str) -> None:
    global source
    count = source.count(old)
    if count != 1:
        raise RuntimeError(f"{label}: expected 1 match, found {count}")
    source = source.replace(old, new, 1)


replace_once(
    """pub(crate) struct AuthorizationProviderCacheHealth {\n    pub(crate) manifest_revision: u64,\n    pub(crate) revocation_revision: u64,\n    pub(crate) last_update_at: i64,\n    pub(crate) healthy: bool,\n}\n""",
    """pub(crate) struct AuthorizationProviderCacheHealth {\n    pub(crate) manifest_revision: u64,\n    pub(crate) revocation_revision: u64,\n    pub(crate) last_update_at: i64,\n    pub(crate) healthy: bool,\n}\n\nstruct ProviderTrustState {\n    root: Option<AuthorizationTrustRootV1>,\n    policy_floor: Option<(AuthorizationVerificationPolicyV1, u64, String)>,\n    manifest: Option<(VerifiedAuthorizationIssuerManifestV1, String)>,\n    health: AuthorizationProviderCacheHealth,\n}\n""",
    "insert ProviderTrustState",
)

replace_once(
    """    root: Arc<RwLock<Option<AuthorizationTrustRootV1>>>,\n    policy_floor: Arc<RwLock<Option<(AuthorizationVerificationPolicyV1, u64, String)>>>,\n    manifest: Arc<RwLock<Option<(VerifiedAuthorizationIssuerManifestV1, String)>>>,\n""",
    """    trust: Arc<RwLock<ProviderTrustState>>,\n""",
    "replace trust fields",
)
replace_once(
    """    context_resolves: Arc<AtomicU64>,\n    health: Arc<RwLock<AuthorizationProviderCacheHealth>>,\n    ready: Arc<tokio::sync::Notify>,\n""",
    """    context_resolves: Arc<AtomicU64>,\n    ready: Arc<tokio::sync::Notify>,\n""",
    "remove standalone health field",
)

replace_once(
    """            root: Arc::new(RwLock::new(None)),\n            policy_floor: Arc::new(RwLock::new(None)),\n            manifest: Arc::new(RwLock::new(None)),\n            verified_contexts: Arc::new(RwLock::new(HashMap::new())),\n""",
    """            trust: Arc::new(RwLock::new(ProviderTrustState {\n                root: None,\n                policy_floor: None,\n                manifest: None,\n                health: AuthorizationProviderCacheHealth {\n                    manifest_revision: 0,\n                    revocation_revision: 0,\n                    last_update_at: 0,\n                    healthy: false,\n                },\n            })),\n            verified_contexts: Arc::new(RwLock::new(HashMap::new())),\n""",
    "attach trust state",
)
replace_once(
    """            context_resolves: Arc::new(AtomicU64::new(0)),\n            health: Arc::new(RwLock::new(AuthorizationProviderCacheHealth {\n                manifest_revision: 0,\n                revocation_revision: 0,\n                last_update_at: 0,\n                healthy: false,\n            })),\n            ready: Arc::new(tokio::sync::Notify::new()),\n""",
    """            context_resolves: Arc::new(AtomicU64::new(0)),\n            ready: Arc::new(tokio::sync::Notify::new()),\n""",
    "attach remove health init",
)

replace_once(
    """            root: Arc::new(RwLock::new(Some(trust.root))),\n            policy_floor: Arc::new(RwLock::new(Some((\n                trust.policy,\n                trust.minimum_manifest_generation,\n                trust.minimum_manifest_digest,\n            )))),\n            manifest: Arc::new(RwLock::new(Some((trust.manifest, trust.manifest_digest)))),\n            verified_contexts: Arc::new(RwLock::new(HashMap::new())),\n""",
    """            trust: Arc::new(RwLock::new(ProviderTrustState {\n                root: Some(trust.root),\n                policy_floor: Some((\n                    trust.policy,\n                    trust.minimum_manifest_generation,\n                    trust.minimum_manifest_digest,\n                )),\n                manifest: Some((trust.manifest, trust.manifest_digest)),\n                health: AuthorizationProviderCacheHealth {\n                    manifest_revision: 0,\n                    revocation_revision: 0,\n                    last_update_at: 0,\n                    healthy: false,\n                },\n            })),\n            verified_contexts: Arc::new(RwLock::new(HashMap::new())),\n""",
    "runtime trust state",
)
replace_once(
    """            context_resolves: Arc::new(AtomicU64::new(0)),\n            health: Arc::new(RwLock::new(AuthorizationProviderCacheHealth {\n                manifest_revision: 0,\n                revocation_revision: 0,\n                last_update_at: 0,\n                healthy: false,\n            })),\n            ready: Arc::new(tokio::sync::Notify::new()),\n""",
    """            context_resolves: Arc::new(AtomicU64::new(0)),\n            ready: Arc::new(tokio::sync::Notify::new()),\n""",
    "runtime remove health init",
)

replace_once(
    """            root: Arc::new(RwLock::new(Some(root))),\n            policy_floor: Arc::new(RwLock::new(Some((\n                policy,\n                input.minimum_manifest_generation,\n                manifest_digest.clone(),\n            )))),\n            manifest: Arc::new(RwLock::new(Some((manifest, manifest_digest)))),\n            verified_contexts: Arc::new(RwLock::new(HashMap::new())),\n""",
    """            trust: Arc::new(RwLock::new(ProviderTrustState {\n                root: Some(root),\n                policy_floor: Some((\n                    policy,\n                    input.minimum_manifest_generation,\n                    manifest_digest.clone(),\n                )),\n                manifest: Some((manifest, manifest_digest)),\n                health: AuthorizationProviderCacheHealth {\n                    manifest_revision: 0,\n                    revocation_revision: 0,\n                    last_update_at: 0,\n                    healthy: true,\n                },\n            })),\n            verified_contexts: Arc::new(RwLock::new(HashMap::new())),\n""",
    "test trust state",
)
replace_once(
    """            context_resolves: Arc::new(AtomicU64::new(0)),\n            health: Arc::new(RwLock::new(AuthorizationProviderCacheHealth {\n                manifest_revision: 0,\n                revocation_revision: 0,\n                last_update_at: 0,\n                healthy: true,\n            })),\n            ready: Arc::new(tokio::sync::Notify::new()),\n""",
    """            context_resolves: Arc::new(AtomicU64::new(0)),\n            ready: Arc::new(tokio::sync::Notify::new()),\n""",
    "test remove health init",
)

replace_once(
    """impl AuthorizationProviderCache {\n""",
    """impl AuthorizationProviderCache {\n    fn trust_read(\n        &self,\n    ) -> Result<std::sync::RwLockReadGuard<'_, ProviderTrustState>, TrellisClientError> {\n        self.trust.read().map_err(|_| {\n            TrellisClientError::Bootstrap("provider trust state lock poisoned".into())\n        })\n    }\n\n    fn trust_write(\n        &self,\n    ) -> Result<std::sync::RwLockWriteGuard<'_, ProviderTrustState>, TrellisClientError> {\n        self.trust.write().map_err(|_| {\n            TrellisClientError::Bootstrap("provider trust state lock poisoned".into())\n        })\n    }\n\n""",
    "insert trust lock helpers",
)

start = source.index("        if let Some(current) = self\n            .root\n")
end_marker = "        *policy_floor = Some((policy, input.minimum_manifest_generation, floor_digest));\n"
end = source.index(end_marker, start) + len(end_marker)
new_sync = """        let policy = AuthorizationVerificationPolicyV1::new(\n            own.corrected_now_seconds()?,\n            input.policy.allowed_clock_skew_seconds,\n            input.policy.maximum_context_lifetime_seconds,\n            input.policy.maximum_context_bytes,\n            input.policy.maximum_permissions,\n            input.policy.maximum_capabilities,\n            input.minimum_manifest_generation,\n        )\n        .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?;\n        let mut trust = self.trust_write()?;\n        if let Some(current) = trust.root.as_ref() {\n            if current.authority() != root.authority()\n                || current.key_id() != root.key_id()\n                || current\n                    .digest()\n                    .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?\n                    != root_digest\n            {\n                return Err(TrellisClientError::Bootstrap(\n                    "authorization trust root changed".into(),\n                ));\n            }\n        }\n        if let Some((_, generation, digest)) = trust.policy_floor.as_ref() {\n            if input.minimum_manifest_generation < *generation {\n                return Err(TrellisClientError::Bootstrap(\n                    "authorization manifest floor rolled back".into(),\n                ));\n            }\n            if input.minimum_manifest_generation == *generation && floor_digest != *digest {\n                return Err(TrellisClientError::Bootstrap(\n                    "authorization manifest floor equivocates".into(),\n                ));\n            }\n        }\n        trust.root = Some(root);\n        trust.policy_floor = Some((policy, input.minimum_manifest_generation, floor_digest));\n"""
source = source[:start] + new_sync + source[end:]

replace_once(
    """        self.health\n            .read()\n            .map(|health| health.clone())\n            .map_err(|_| TrellisClientError::Bootstrap("provider health lock poisoned".into()))\n""",
    """        Ok(self.trust_read()?.health.clone())\n""",
    "health reads trust snapshot",
)
replace_once(
    """        let policy_floor = self\n            .policy_floor\n            .read()\n            .map_err(|_| TrellisClientError::Bootstrap("provider trust lock poisoned".into()))?\n            .clone()\n            .ok_or_else(|| {\n                TrellisClientError::Bootstrap("authorization context unavailable".into())\n            })?;\n""",
    """        let policy_floor = self\n            .trust_read()?\n            .policy_floor\n            .clone()\n            .ok_or_else(|| {\n                TrellisClientError::Bootstrap("authorization context unavailable".into())\n            })?;\n""",
    "policy reads trust snapshot",
)
replace_once(
    """    fn set_healthy(&self, healthy: bool) {\n        if let Ok(mut health) = self.health.write() {\n            health.healthy = healthy;\n        }\n    }\n\n    fn record_healthy(&self) {\n        if let Ok(mut health) = self.health.write() {\n            health.last_update_at = self.now_seconds().unwrap_or(0);\n            health.healthy = true;\n        }\n        self.ready.notify_waiters();\n    }\n""",
    """    fn set_healthy(&self, healthy: bool) {\n        if let Ok(mut trust) = self.trust.write() {\n            trust.health.healthy = healthy;\n        }\n    }\n\n    fn record_healthy(&self) {\n        if let Ok(mut trust) = self.trust.write() {\n            trust.health.last_update_at = self.now_seconds().unwrap_or(0);\n            trust.health.healthy = true;\n        }\n        self.ready.notify_waiters();\n    }\n""",
    "health writes trust snapshot",
)

replace_once(
    """        if self.health()?.manifest_revision >= revision {\n            return Ok(());\n        }\n        let was_healthy = self.health()?.healthy;\n        let (current_generation, current_digest) = {\n            let manifest = self.manifest.read().map_err(|_| {\n                TrellisClientError::Bootstrap("provider manifest lock poisoned".into())\n            })?;\n            manifest\n                .as_ref()\n                .map(|(manifest, digest)| (manifest.generation(), digest.clone()))\n                .unwrap_or((0, String::new()))\n        };\n""",
    """        let (was_healthy, current_generation, current_digest) = {\n            let trust = self.trust_read()?;\n            if trust.health.manifest_revision >= revision {\n                return Ok(());\n            }\n            let (generation, digest) = trust\n                .manifest\n                .as_ref()\n                .map(|(manifest, digest)| (manifest.generation(), digest.clone()))\n                .unwrap_or((0, String::new()));\n            (trust.health.healthy, generation, digest)\n        };\n""",
    "observe manifest trust snapshot",
)
replace_once(
    """            {\n                let mut floor = self.policy_floor.write().map_err(|_| {\n                    TrellisClientError::Bootstrap("provider trust lock poisoned".into())\n                })?;\n                let (mut policy, _, _) = floor.clone().ok_or_else(|| {\n                    TrellisClientError::Bootstrap("authorization context unavailable".into())\n                })?;\n                policy.minimum_manifest_generation = pointer.generation;\n                *floor = Some((policy, pointer.generation, digest.clone()));\n            }\n            *self.manifest.write().map_err(|_| {\n                TrellisClientError::Bootstrap("provider manifest lock poisoned".into())\n            })? = Some((verified, digest));\n""",
    """            {\n                let mut trust = self.trust_write()?;\n                let (mut policy, _, _) = trust.policy_floor.clone().ok_or_else(|| {\n                    TrellisClientError::Bootstrap("authorization context unavailable".into())\n                })?;\n                policy.minimum_manifest_generation = pointer.generation;\n                trust.policy_floor = Some((policy, pointer.generation, digest.clone()));\n                trust.manifest = Some((verified, digest));\n            }\n""",
    "advance manifest trust snapshot",
)
replace_once(
    """        {\n            let mut health = self.health.write().map_err(|_| {\n                TrellisClientError::Bootstrap("provider health lock poisoned".into())\n            })?;\n            health.manifest_revision = health.manifest_revision.max(revision);\n            health.last_update_at = self.now_seconds().unwrap_or(0);\n            health.healthy = was_healthy && !advanced;\n        }\n""",
    """        {\n            let mut trust = self.trust_write()?;\n            trust.health.manifest_revision = trust.health.manifest_revision.max(revision);\n            trust.health.last_update_at = self.now_seconds().unwrap_or(0);\n            trust.health.healthy = was_healthy && !advanced;\n        }\n""",
    "record manifest health in trust snapshot",
)

for old, new, label in [
    (
        """        if let Ok(mut health) = self.health.write() {\n            health.revocation_revision = health.revocation_revision.max(revision);\n            health.last_update_at = self.now_seconds().unwrap_or(0);\n        }\n""",
        """        if let Ok(mut trust) = self.trust.write() {\n            trust.health.revocation_revision = trust.health.revocation_revision.max(revision);\n            trust.health.last_update_at = self.now_seconds().unwrap_or(0);\n        }\n""",
        "revocation health update",
    ),
]:
    count = source.count(old)
    if count != 2:
        raise RuntimeError(f"{label}: expected 2 matches, found {count}")
    source = source.replace(old, new, 2)

replace_once(
    """        self.root\n            .read()\n            .map_err(|_| TrellisClientError::Bootstrap("provider trust lock poisoned".into()))?\n            .clone()\n            .ok_or_else(|| {\n                TrellisClientError::Bootstrap("authorization context unavailable".into())\n            })\n""",
    """        self.trust_read()?\n            .root\n            .clone()\n            .ok_or_else(|| {\n                TrellisClientError::Bootstrap("authorization context unavailable".into())\n            })\n""",
    "root reads trust snapshot",
)
replace_once(
    """        let policy_floor = self\n            .policy_floor\n            .read()\n            .map_err(|_| TrellisClientError::Bootstrap("provider trust lock poisoned".into()))?;\n        let Some((_, minimum_generation, minimum_digest)) = policy_floor.as_ref() else {\n""",
    """        let trust = self.trust_read()?;\n        let Some((_, minimum_generation, minimum_digest)) = trust.policy_floor.as_ref() else {\n""",
    "manifest floor reads trust snapshot",
)
replace_once(
    """        self.manifest\n            .read()\n            .map_err(|_| TrellisClientError::Bootstrap("provider manifest lock poisoned".into()))?\n            .as_ref()\n            .map(|(manifest, _)| manifest.clone())\n            .ok_or_else(|| TrellisClientError::Bootstrap("provider manifest unavailable".into()))\n""",
    """        self.trust_read()?\n            .manifest\n            .as_ref()\n            .map(|(manifest, _)| manifest.clone())\n            .ok_or_else(|| TrellisClientError::Bootstrap("provider manifest unavailable".into()))\n""",
    "runtime manifest reads trust snapshot",
)

replace_once(
    """        let manifest_generation = self\n            .manifest\n            .read()\n            .map_err(|_| TrellisClientError::Bootstrap("provider manifest lock poisoned".into()))?\n            .as_ref()\n            .map(|(manifest, _)| manifest.generation())\n            .ok_or_else(|| {\n                TrellisClientError::Bootstrap("authorization manifest is unavailable".into())\n            })?;\n""",
    """        let manifest_generation = self\n            .trust_read()?\n            .manifest\n            .as_ref()\n            .map(|(manifest, _)| manifest.generation())\n            .ok_or_else(|| {\n                TrellisClientError::Bootstrap("authorization manifest is unavailable".into())\n            })?;\n""",
    "capture manifest generation from trust snapshot",
)
replace_once(
    """        let manifest = self\n            .manifest\n            .read()\n            .map_err(|_| TrellisClientError::Bootstrap("provider manifest lock poisoned".into()))?;\n        if manifest.as_ref().map(|(manifest, _)| manifest.generation()) != Some(manifest_generation)\n        {\n""",
    """        let trust = self.trust_read()?;\n        if trust\n            .manifest\n            .as_ref()\n            .map(|(manifest, _)| manifest.generation())\n            != Some(manifest_generation)\n        {\n""",
    "verify manifest generation from trust snapshot",
)
replace_once("        drop(manifest);\n", "        drop(trust);\n", "drop trust snapshot")

replace_once(
    """            let current_generation = self\n                .manifest\n                .read()\n                .map_err(|_| {\n                    TrellisClientError::Bootstrap("provider manifest lock poisoned".into())\n                })?\n                .as_ref()\n                .map(|(manifest, _)| manifest.generation())\n                .ok_or_else(|| {\n                    TrellisClientError::Bootstrap("provider manifest unavailable".into())\n                })?;\n""",
    """            let current_generation = self\n                .trust_read()?\n                .manifest\n                .as_ref()\n                .map(|(manifest, _)| manifest.generation())\n                .ok_or_else(|| {\n                    TrellisClientError::Bootstrap("provider manifest unavailable".into())\n                })?;\n""",
    "resolve context current generation",
)
replace_once(
    """        if let Some((manifest, digest)) = self\n            .manifest\n            .read()\n            .map_err(|_| TrellisClientError::Bootstrap("provider manifest lock poisoned".into()))?\n            .as_ref()\n            .filter(|(manifest, _)| manifest.generation() == generation)\n            .cloned()\n        {\n""",
    """        if let Some((manifest, digest)) = self\n            .trust_read()?\n            .manifest\n            .as_ref()\n            .filter(|(manifest, _)| manifest.generation() == generation)\n            .cloned()\n        {\n""",
    "resolve manifest from trust snapshot",
)

source = source.replace(
    '"provider trust lock poisoned",\n        "provider manifest lock poisoned",',
    '"provider trust state lock poisoned",\n        "provider trust lock poisoned",\n        "provider manifest lock poisoned",',
    1,
)

for forbidden in ["self.root", "self.policy_floor", "self.manifest", "self.health"]:
    if forbidden in source:
        raise RuntimeError(f"standalone provider trust field remains: {forbidden}")

PATH.write_text(source)
