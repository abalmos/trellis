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

# Every source/task that actually loads runtime protocol resolution requests the
# generated WASM explicitly. Contract source evaluation itself remains WASM-free.
replace_once(
    "ts/deno.json",
    '    "protocol:wasm": "cargo run --manifest-path ../rust/xtask/Cargo.toml -- protocol-wasm",',
    '    "protocol:wasm": "cargo run --manifest-path ../rust/xtask/Cargo.toml -- protocol-wasm",\n    "prepare:runtime": "deno task prepare && deno task protocol:wasm && deno task -c portals/login/deno.json build:embedded",',
    "runtime preparation task",
)
replace_once(
    "ts/deno.json",
    '    "check:integration": "deno check -c integration/deno.json integration/all_runner.ts integration/runner.ts integration/matrix_conformance_test.ts integration/_support/runtime.ts integration/authorization_registry/*.integration_test.ts integration/rpc/_fixture.ts integration/rpc/*.integration_test.ts integration/events/_fixture.ts integration/events/*.integration_test.ts integration/operations/_fixture.ts integration/operations/*.integration_test.ts integration/feeds/_fixture.ts integration/feeds/*.integration_test.ts integration/transfer/_fixture.ts integration/transfer/*.integration_test.ts integration/resources/_fixture.ts integration/resources/*.integration_test.ts integration/state/*.integration_test.ts",',
    '    "check:integration": "deno task prepare && deno task protocol:wasm && deno check -c integration/deno.json integration/all_runner.ts integration/runner.ts integration/matrix_conformance_test.ts integration/_support/runtime.ts integration/authorization_registry/*.integration_test.ts integration/rpc/_fixture.ts integration/rpc/*.integration_test.ts integration/events/_fixture.ts integration/events/*.integration_test.ts integration/operations/_fixture.ts integration/operations/*.integration_test.ts integration/feeds/_fixture.ts integration/feeds/*.integration_test.ts integration/transfer/_fixture.ts integration/transfer/*.integration_test.ts integration/resources/_fixture.ts integration/resources/*.integration_test.ts integration/state/*.integration_test.ts",',
    "integration typecheck task",
)
replace_once(
    "ts/deno.json",
    '    "test:client-integration": "deno run -A -c integration/deno.json integration/runner.ts",',
    '    "test:client-integration": "deno task prepare:runtime && deno run -A -c integration/deno.json integration/runner.ts",',
    "client integration task",
)
replace_once(
    "ts/deno.json",
    '    "test:integration": "deno run -A -c integration/deno.json integration/all_runner.ts",',
    '    "test:integration": "deno task prepare:runtime && deno run -A -c integration/deno.json integration/all_runner.ts",',
    "integration task",
)
replace_once(
    "ts/deno.json",
    '    "test:contracts": "deno task prepare && deno test -A packages/trellis/contract_support/protocol_test.ts packages/trellis/contract_support/protocol_artifacts_test.ts packages/trellis/contract_support/descriptors_test.ts",',
    '    "test:contracts": "deno task prepare && deno task protocol:wasm && deno test -A packages/trellis/contract_support/protocol_test.ts packages/trellis/contract_support/protocol_artifacts_test.ts packages/trellis/contract_support/descriptors_test.ts",',
    "contract tests",
)

for task, command, suffix in [
    ("dev", "deno run -A vite dev", ","),
    ("build", "deno run -A vite build", ","),
    (
        "check",
        "deno run -A @sveltejs/kit sync && deno run -A svelte-check --tsconfig ./tsconfig.check.json",
        "",
    ),
]:
    replace_once(
        "ts/apps/console/deno.json",
        f'    "{task}": "deno task prepare && {command}"{suffix}',
        f'    "{task}": "deno task prepare && deno task -c ../../deno.json protocol:wasm && {command}"{suffix}',
        f"console {task} task",
    )

replace_once(
    "docs/deno.json",
    '    "docs:api": "deno task -c ../ts/deno.json prepare && deno run -A ./scripts/generate_ts_api_docs.ts",',
    '    "docs:api": "deno task -c ../ts/deno.json prepare && deno task -c ../ts/deno.json protocol:wasm && deno run -A ./scripts/generate_ts_api_docs.ts",',
    "docs API task",
)

# Pages bypasses the app/doc task wrappers to control output roots. Generate the
# focused WASM only for source trees that no longer carry the old checked-in output.
replace_once(
    ".github/workflows/pages.yml",
    '''      - uses: dtolnay/rust-toolchain@stable
''',
    '''      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: wasm32-unknown-unknown
''',
    "pages Rust WASM target",
)
replace_once(
    ".github/workflows/pages.yml",
    '''          prepare_once "${release_worktree}" "${latest_tag}"
          prepare_once "${current_root}" "${current_tag}"

          build_docs "${release_docs_root}" "${site_root}" "${release_root}/docs"
''',
    '''          prepare_once "${release_worktree}" "${latest_tag}"
          prepare_once "${current_root}" "${current_tag}"

          ensure_protocol_wasm() {
            local repo_root="$1"
            local output="${repo_root}/ts/packages/trellis/auth/protocol_wasm/trellis_protocol_wasm.js"
            if [ -s "${output}" ]; then
              return 0
            fi
            cargo run --manifest-path "${repo_root}/rust/xtask/Cargo.toml" -- protocol-wasm
          }

          ensure_protocol_wasm "${release_docs_root}"
          ensure_protocol_wasm "${release_console_root}"
          ensure_protocol_wasm "${current_root}"

          build_docs "${release_docs_root}" "${site_root}" "${release_root}/docs"
''',
    "pages protocol WASM preparation",
)

# The monolithic release verifier starts independent lanes in parallel. Generic
# preparation is intentionally WASM-free, so install the build target and
# materialize the shared runtime artifact once before Static imports the package root.
replace_once(
    "rust/xtask/src/release/runner.rs",
    '''        "repository preparation failed",
    )?;
    if working_tree_snapshot(repo_root)? != before_prepare {
''',
    '''        "repository preparation failed",
    )?;
    run_checked_command(
        repo_root,
        &CommandSpec::new("rustup", ["target", "add", "wasm32-unknown-unknown"]),
        "protocol WASM target installation failed",
    )?;
    run_checked_command(
        repo_root,
        &CommandSpec::new(
            "cargo",
            [
                "run",
                "--manifest-path",
                "xtask/Cargo.toml",
                "--",
                "protocol-wasm",
            ],
        ),
        "protocol WASM preparation failed",
    )?;
    if working_tree_snapshot(repo_root)? != before_prepare {
''',
    "release verify protocol WASM preparation",
)
