from pathlib import Path

path = Path("rust/tools/generate/src/planning.rs")
text = path.read_text()

old = '''                let metadata = generated_artifacts_metadata(
                    &resolved,
                    &native_api_digest(&resolved)?,
                    &artifact_version,
                    entry.runtime_source,
                    &trellis_package_version(),
                    entry.jsr_out.is_some(),
                    entry.npm_out.is_some(),
                    entry.cargo_out.is_some(),
                    &package_name,
                    &crate_name,
                    generator_fingerprint,
                );
                if !force
                    && generated_artifacts_are_fresh(
                        &metadata,
                        out_api,
                        entry.jsr_out.as_deref(),
                        entry.npm_out.as_deref(),
                        entry.cargo_out.as_deref(),
                    )
                {
'''
new = '''                let protocol_participant_out = match (
                    entry.protocol_participant_out.as_ref(),
                    resolved.participant.as_ref(),
                ) {
                    (Some(path), _) => Some(path.clone()),
                    (None, Some(participant)) => Some(protocol_participant_output_path(
                        out_api,
                        participant.participant.id(),
                    )?),
                    (None, None) => None,
                };
                let metadata = generated_artifacts_metadata(
                    &resolved,
                    &native_api_digest(&resolved)?,
                    &artifact_version,
                    entry.runtime_source,
                    &trellis_package_version(),
                    entry.jsr_out.is_some(),
                    entry.npm_out.is_some(),
                    entry.cargo_out.is_some(),
                    &package_name,
                    &crate_name,
                    generator_fingerprint,
                );
                if !force
                    && generated_artifacts_are_fresh(
                        &metadata,
                        out_api,
                        entry.jsr_out.as_deref(),
                        entry.npm_out.as_deref(),
                        entry.cargo_out.as_deref(),
                    )
                    && match (
                        resolved.participant.as_ref(),
                        protocol_participant_out.as_deref(),
                    ) {
                        (Some(expected), Some(path)) => {
                            protocol_participant_output_is_fresh(expected, path)?
                        }
                        (None, None) => true,
                        _ => false,
                    }
                {
'''
if text.count(old) != 1:
    raise RuntimeError(f"freshness anchor count: {text.count(old)}")
text = text.replace(old, new, 1)

old = '''                if let Some(cargo_participant_out) = &entry.cargo_participant_out {
                    let participant_source = resolved
                        .participant_path
                        .as_deref()
                        .unwrap_or(&resolved.api.path);
                    let mappings = participant_alias_mappings(entry, plan, participant_source)?;
                    write_participant_facade_outputs(
                        &resolved.api_path,
                        participant_source,
                        entry.protocol_participant_out.as_deref().ok_or_else(|| {
                            miette::miette!("missing protocol participant output")
                        })?,
                        cargo_participant_out,
'''
new = '''                if let Some(cargo_participant_out) = &entry.cargo_participant_out {
                    let participant_source = resolved
                        .participant_path
                        .as_deref()
                        .unwrap_or(&resolved.api.path);
                    let mappings = participant_alias_mappings(entry, plan, participant_source)?;
                    write_participant_facade_outputs(
                        &resolved.api_path,
                        participant_source,
                        protocol_participant_out.as_deref().ok_or_else(|| {
                            miette::miette!("missing protocol participant output")
                        })?,
                        cargo_participant_out,
'''
if text.count(old) != 1:
    raise RuntimeError(f"cargo participant anchor count: {text.count(old)}")
text = text.replace(old, new, 1)

old = '''                } else if let Some(protocol_participant_out) =
                    entry.protocol_participant_out.as_deref()
                {
                    let sibling_participant = entry
                        .discovered
                        .source_path
                        .with_file_name("trellis.participant.json");
                    let participant_source = resolved
                        .participant_path
                        .as_deref()
                        .unwrap_or(&sibling_participant);
                    write_protocol_participant(
                        &resolved.api_path,
                        participant_source,
                        protocol_participant_out,
                    )?;
                }
'''
new = '''                } else if let (Some(protocol_participant_out), Some(participant_source)) = (
                    protocol_participant_out.as_deref(),
                    resolved.participant_path.as_deref(),
                ) {
                    write_protocol_participant(
                        &resolved.api_path,
                        participant_source,
                        protocol_participant_out,
                    )?;
                }
'''
if text.count(old) != 1:
    raise RuntimeError(f"protocol participant anchor count: {text.count(old)}")
text = text.replace(old, new, 1)

marker = '''fn cleanup_legacy_protocol_outputs(plan: &[AutoPlanEntry]) -> miette::Result<()> {'''
helper = '''fn protocol_participant_output_path(
    out_api: &Path,
    participant_id: &str,
) -> miette::Result<PathBuf> {
    let protocol_root = out_api
        .parent()
        .and_then(Path::parent)
        .ok_or_else(|| miette::miette!("generated API output is outside generated/protocol/apis"))?;
    Ok(protocol_root
        .join("participants")
        .join(format!("{participant_id}.json")))
}

fn protocol_participant_output_is_fresh(
    expected: &trellis_contracts::LoadedParticipant,
    output: &Path,
) -> miette::Result<bool> {
    let Ok(existing) = trellis_contracts::load_participant_source(output) else {
        return Ok(false);
    };
    Ok(existing.participant.digest().into_diagnostic()?
        == expected.participant.digest().into_diagnostic()?)
}

'''
if text.count(marker) != 1:
    raise RuntimeError(f"helper anchor count: {text.count(marker)}")
text = text.replace(marker, helper + marker, 1)

test_marker = '''    #[test]
    fn auto_plan_orders_local_jsr_package_imports_before_dependents() {'''
test = r'''    #[test]
    fn protocol_participant_freshness_tracks_semantic_identity() {
        let temp = tempfile::tempdir().unwrap();
        let expected_path = temp.path().join("expected.participant.json");
        let output_path = temp.path().join("generated.participant.json");
        let service = r#"{"format":"trellis.participant.v1","id":"example.fixture@v1","displayName":"Example","description":"Example participant","kind":"service"}"#;
        let app = r#"{"format":"trellis.participant.v1","id":"example.fixture@v1","displayName":"Example","description":"Example participant","kind":"app"}"#;
        fs::write(&expected_path, service).unwrap();
        fs::write(&output_path, service).unwrap();
        let expected = trellis_contracts::load_participant_source(&expected_path).unwrap();
        assert!(protocol_participant_output_is_fresh(&expected, &output_path).unwrap());

        fs::write(&output_path, app).unwrap();
        assert!(!protocol_participant_output_is_fresh(&expected, &output_path).unwrap());

        fs::write(&output_path, "{").unwrap();
        assert!(!protocol_participant_output_is_fresh(&expected, &output_path).unwrap());

        fs::remove_file(&output_path).unwrap();
        assert!(!protocol_participant_output_is_fresh(&expected, &output_path).unwrap());
    }

'''
if text.count(test_marker) != 1:
    raise RuntimeError(f"test anchor count: {text.count(test_marker)}")
text = text.replace(test_marker, test + test_marker, 1)

path.write_text(text)
