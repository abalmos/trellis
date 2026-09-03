//! Feature check for compiling the complete Field Ops demo service IDL.

use miette::IntoDiagnostic;
use std::{fs, path::PathBuf, time::SystemTime};
use trellis_idl::compile_project;

#[test]
fn compiles_complete_demo_service() -> miette::Result<()> {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../demos/ts/service/contract.trellis");
    let root = std::env::temp_dir().join(format!(
        "trellis-idl-demo-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos()
    ));
    fs::create_dir(&root).into_diagnostic()?;
    fs::copy(source, root.join("contract.trellis")).into_diagnostic()?;
    fs::copy(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../demos/ts/service/trellis.toml"),
        root.join("trellis.toml"),
    )
    .into_diagnostic()?;

    let result = (|| {
        let project = compile_project(&root)?;
        let api = &project.apis["demo.service@v1"];
        let api_value = api.normalized_value().into_diagnostic()?;
        assert_eq!(api.version(), "1.0.0");
        assert!(api_value["rpc"].get("Sites.List").is_some());
        assert_eq!(
            api_value["operations"]["Evidence.Upload"]["transfer"]["direction"],
            "send"
        );
        assert!(api_value["events"].get("Audit.Recorded").is_some());
        assert!(api_value["feeds"].get("Audit.Feed").is_some());

        assert_eq!(project.participants.len(), 1);
        let participant = &project.participants[0];
        let participant_value = participant.normalized_value().into_diagnostic()?;
        assert_eq!(participant.id(), "demo.service@v1");
        assert!(participant_value["resources"]["store"]
            .get("uploads")
            .is_some());
        assert!(participant_value["resources"]["kv"]
            .get("siteSummaries")
            .is_some());
        assert!(participant_value["jobQueues"]
            .get("refreshSiteSummary")
            .is_some());
        Ok(())
    })();
    fs::remove_dir_all(root).into_diagnostic()?;
    result
}
