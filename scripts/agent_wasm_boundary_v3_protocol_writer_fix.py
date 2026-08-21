from pathlib import Path


def replace_once(path: str, old: str, new: str, label: str) -> None:
    p = Path(path)
    text = p.read_text()
    count = text.count(old)
    if count != 1:
        raise RuntimeError(f"{label}: expected one occurrence, found {count}: {old[:120]!r}")
    p.write_text(text.replace(old, new, 1))


replace_once(
    "rust/tools/generate/src/artifacts.rs",
    "use trellis_contracts::ApiBuilder;",
    "use trellis_contracts::{canonicalize_json, ApiBuilder, ContractBuilder};",
    "contracts imports",
)

replace_once(
    "rust/tools/generate/src/artifacts.rs",
    '''pub fn write_protocol_participant(
    api_path: &Path,
    participant_path: &Path,
    protocol_participant_out: &Path,
) -> miette::Result<()> {
    let (participant_json, participant_digest) =
        trellis_codegen_rust::native_participant_artifact(api_path, participant_path, &[])
            .into_diagnostic()?;
    if let Some(parent) = protocol_participant_out.parent() {
        fs::create_dir_all(parent).into_diagnostic()?;
    }
    write_if_changed(protocol_participant_out, &format!("{participant_json}\\n"))?;
    output::print_detail(
        "participant",
        protocol_participant_out.display().to_string(),
    );
    output::print_detail("participant digest", participant_digest);
    Ok(())
}
''',
    '''pub fn write_protocol_participant(
    resolved: &ResolvedNativeInput,
    protocol_participant_out: &Path,
) -> miette::Result<()> {
    let participant = resolved
        .participant
        .as_ref()
        .ok_or_else(|| miette::miette!("missing resolved participant"))?;
    let referenced_apis = resolved
        .referenced_apis
        .iter()
        .map(|api| (api.render_model.id.clone(), api.value.clone()))
        .collect();
    let artifacts = ContractBuilder::from_native(resolved.api.value.clone(), participant.value.clone())
        .referenced_apis(referenced_apis)
        .build()
        .into_diagnostic()?;
    let participant_json =
        canonicalize_json(&artifacts.participant_value().into_diagnostic()?).into_diagnostic()?;
    let participant_digest = artifacts.participant_digest().into_diagnostic()?;
    if let Some(parent) = protocol_participant_out.parent() {
        fs::create_dir_all(parent).into_diagnostic()?;
    }
    write_if_changed(protocol_participant_out, &format!("{participant_json}\\n"))?;
    output::print_detail(
        "participant",
        protocol_participant_out.display().to_string(),
    );
    output::print_detail("participant digest", participant_digest);
    Ok(())
}
''',
    "protocol participant writer",
)

replace_once(
    "rust/tools/generate/src/planning.rs",
    '''                } else if let (Some(protocol_participant_out), Some(participant_source)) = (
                    protocol_participant_out.as_deref(),
                    resolved.participant_path.as_deref(),
                ) {
                    write_protocol_participant(
                        &resolved.api_path,
                        participant_source,
                        protocol_participant_out,
                    )?;
                }
''',
    '''                } else if let Some(protocol_participant_out) = protocol_participant_out.as_deref() {
                    write_protocol_participant(&resolved, protocol_participant_out)?;
                }
''',
    "protocol participant call",
)
