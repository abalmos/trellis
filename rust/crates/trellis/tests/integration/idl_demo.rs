//! Out-of-tree acceptance coverage for native Trellis IDL demos.

use std::path::Path;
use std::process::Command;

use crate::support::assertions::assert_runtime_case_registered;

const CASE_ID: &str = "idl-demo.field-ops-out-of-tree";

#[test]
fn field_ops_out_of_tree() {
    assert_runtime_case_registered(CASE_ID, "idl-demo", "idl_demo");
    let repo = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .expect("trellis-rs should live under rust/crates/trellis");
    let status = Command::new("deno")
        .current_dir(repo)
        .args([
            "run",
            "-A",
            "-c",
            "ts/deno.json",
            "ts/integration/idl/runner.ts",
        ])
        .status()
        .expect("run Field Ops IDL acceptance");
    assert!(status.success(), "Field Ops IDL acceptance failed");
}
