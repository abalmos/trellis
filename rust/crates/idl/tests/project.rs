//! Public project compilation and authoring diagnostics.

use std::{fs, path::PathBuf};

#[test]
fn compiles_runtime_acceptance_project() {
    let root =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../integration/fixtures/runtime");
    let project = trellis_idl::compile_project(&root).unwrap();
    assert!(project.apis.contains_key("test.runtime@v1"));
    assert!(project
        .participants
        .iter()
        .any(|participant| participant.id() == "test.provider"));
}

#[test]
fn malformed_idl_reports_source_and_span() {
    let root = tempfile::tempdir().unwrap();
    fs::write(root.path().join("trellis.toml"), "format = 1\n").unwrap();
    fs::write(
        root.path().join("contract.trellis"),
        "api \"example@v1\" { version \"1.0.0\" }",
    )
    .unwrap();
    let error = trellis_idl::compile_project(root.path()).unwrap_err();
    let diagnostic = format!("{error:?}");
    assert!(diagnostic.contains("contract.trellis"), "{diagnostic}");
    assert!(diagnostic.contains("expected"), "{diagnostic}");
    assert!(
        error
            .labels()
            .is_some_and(|mut labels| labels.next().is_some()),
        "{diagnostic}"
    );
}
