from pathlib import Path


def replace_once(text: str, old: str, new: str, label: str) -> str:
    count = text.count(old)
    if count != 1:
        raise RuntimeError(f"{label}: expected one match, found {count}")
    return text.replace(old, new, 1)


path = Path("rust/crates/trellis-test/src/lib.rs")
text = path.read_text()

# The shared runtime still needs per-test *participant* identity so durable
# identity authority from a previous serial test cannot affect the next one.
# This is ordinary test data: API IDs, action names, capabilities, and subjects
# stay exactly as authored and production code has no test-specific resolver.
runtime_anchor = '''    /// Return direct SQLite access for the runtime-owned Trellis control plane.
    #[must_use]
    pub fn control_plane_sqlite(&self) -> TrellisControlPlaneSqlite {
'''
runtime_method = '''    /// Return this test case's participant artifact with unchanged API surfaces.
    pub fn case_contract(
        &self,
        contract: &TrellisTestContract,
    ) -> Result<TrellisTestContract, TrellisTestError> {
        contract.for_test_namespace(self.namespace.as_deref())
    }

'''
text = replace_once(
    text,
    runtime_anchor,
    runtime_method + runtime_anchor,
    "case contract runtime helper",
)

impl_anchor = '''    /// Return the contract ID from this test source.
    #[must_use]
    pub fn id(&self) -> &str {
        self.participant["id"]
            .as_str()
            .expect("validated test contract has an id")
    }
}

fn builtin_api_artifacts()'''
impl_replacement = '''    /// Return the participant ID from this test source.
    #[must_use]
    pub fn id(&self) -> &str {
        self.participant["id"]
            .as_str()
            .expect("validated test contract has an id")
    }

    fn for_test_namespace(&self, namespace: Option<&str>) -> Result<Self, TrellisTestError> {
        let Some(namespace) = namespace else {
            return Ok(self.clone());
        };
        let participant_id = case_participant_id(self.id(), namespace)?;
        if participant_id == self.id() {
            return Ok(self.clone());
        }

        let mut participant = self.participant.clone();
        participant["id"] = Value::String(participant_id);
        let mut apis = builtin_api_artifacts();
        add_referenced_test_apis(&self.referenced_apis, &mut apis)?;
        for api_id in native_participant_reference_ids(&participant) {
            ensure_builtin_api(&api_id, &mut apis)?;
        }
        let artifacts = trellis_rs::contracts::ContractBuilder::from_native(
            self.api.clone(),
            participant,
        )
        .referenced_apis(apis)
        .build()?;
        build_test_contract(artifacts, self.referenced_apis.clone())
    }
}

fn case_participant_id(id: &str, namespace: &str) -> Result<String, TrellisTestError> {
    let (name, version) = id.rsplit_once('@').ok_or_else(|| {
        TrellisTestError::UnexpectedResponse(format!(
            "test participant id '{id}' is not versioned"
        ))
    })?;
    let suffix = format!("-{namespace}");
    if name.ends_with(&suffix) {
        return Ok(id.to_owned());
    }
    Ok(format!("{name}{suffix}@{version}"))
}

#[cfg(test)]
mod participant_identity_tests {
    use super::*;

    #[test]
    fn test_namespace_changes_only_participant_identity() {
        let contract = TrellisTestContract::from_artifacts(
            trellis_rs::contracts::ContractBuilder::authoring(
                "trellis.integration.participant-identity-proof@v1",
                "Participant identity proof",
                "Proves shared test isolation does not rewrite API surfaces.",
                trellis_rs::contracts::ContractKind::App,
            )
            .build()
            .expect("build participant identity proof artifacts"),
        )
        .expect("build participant identity proof contract");

        let case = contract
            .for_test_namespace(Some("run-123-case-rpc"))
            .expect("build case participant");

        assert_eq!(case.api(), contract.api());
        assert_eq!(
            case.id(),
            "trellis.integration.participant-identity-proof-run-123-case-rpc@v1"
        );
        assert_ne!(case.digest(), contract.digest());

        let original = contract.participant().clone();
        let mut case_participant = case.participant().clone();
        case_participant["id"] = original["id"].clone();
        assert_eq!(case_participant, original);
    }
}

fn builtin_api_artifacts()'''
text = replace_once(text, impl_anchor, impl_replacement, "participant-only contract identity")

# These are the three product flows that previously called contract.scoped(...):
# deployment authority approval, bound client auth, and browser auth. Reapply
# only the participant-id transformation at the test harness boundary.
compiled = "        let compiled = build_test_artifacts(&contract, &mut self.api_artifacts)?;\n"
if text.count(compiled) != 3:
    raise RuntimeError(f"expected three contract compilation flows, found {text.count(compiled)}")
text = text.replace(
    compiled,
    "        let contract = contract.for_test_namespace(self.namespace.as_deref())?;\n" + compiled,
)

# No old scope vocabulary or production mutation seam may return.
for forbidden in (
    "IntegrationTestScope",
    "integration_test_scope",
    "with_integration_test_scope",
    "descriptor_subject(&self",
):
    if forbidden in text:
        raise RuntimeError(f"stale production-style test scope remains: {forbidden}")

path.write_text(text)
