//! Feature check for cyclic sibling-project API dependencies.

use miette::IntoDiagnostic;
use std::{fs, time::SystemTime};
use trellis_idl::compile_project;

#[test]
fn compiles_cyclic_sibling_projects_without_recursion() -> miette::Result<()> {
    let root = std::env::temp_dir().join(format!(
        "trellis-idl-cycle-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos()
    ));
    let a = root.join("a");
    let b = root.join("b");
    fs::create_dir_all(&a).into_diagnostic()?;
    fs::create_dir_all(&b).into_diagnostic()?;
    fs::write(
        a.join("contract.trellis"),
        r#"
api "example.a@v1" {
    version "1.0.0";
    display_name "Example A";
    description "Example A API.";
    model Empty {}
    rpc "Status.Get" { version "v1"; input Empty; output Empty; }
}
participant "example.a" service {
    implements "example.a@v1";
    use required b "example.b@v1" { call rpc "Status.Get"; }
}
"#,
    )
    .into_diagnostic()?;
    fs::write(
        a.join("trellis.toml"),
        "format = 1\n\n[apis.\"example.b@v1\"]\nversion = \"1.0.0\"\npath = \"../b\"\n",
    )
    .into_diagnostic()?;
    fs::write(
        b.join("contract.trellis"),
        r#"
api "example.b@v1" {
    version "1.0.0";
    display_name "Example B";
    description "Example B API.";
    model Empty {}
    rpc "Status.Get" { version "v1"; input Empty; output Empty; }
}
participant "example.b" service {
    implements "example.b@v1";
    use required a "example.a@v1" { call rpc "Status.Get"; }
}
"#,
    )
    .into_diagnostic()?;
    fs::write(
        b.join("trellis.toml"),
        "format = 1\n\n[apis.\"example.a@v1\"]\nversion = \"1.0.0\"\npath = \"../a\"\n",
    )
    .into_diagnostic()?;

    let result = (|| {
        let compiled_a = compile_project(&a)?;
        let compiled_b = compile_project(&b)?;
        assert_eq!(compiled_a.referenced_apis.len(), 1);
        assert_eq!(compiled_b.referenced_apis.len(), 1);
        assert_eq!(compiled_a.participants.len(), 1);
        assert_eq!(compiled_b.participants.len(), 1);
        Ok(())
    })();
    fs::remove_dir_all(root).into_diagnostic()?;
    result
}
