use std::{fs, path::Path, process::Command};

#[test]
fn participant_output_collisions_fail_before_publication() {
    let root = tempfile::tempdir().unwrap();
    fs::write(root.path().join("trellis.toml"), "format = 1\n").unwrap();
    fs::write(root.path().join("deno.json"), "{}").unwrap();
    fs::write(
        root.path().join("contract.trellis"),
        r#"
api "demo.api@v1" { version "1.0.0"; display_name "API"; description "Collision."; }
participant "demo.worker@v1" service { implements "demo.api@v1"; }
participant "demo.worker@v2" service { implements "demo.api@v1"; }
"#,
    )
    .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_trellis"))
        .args(["generate", "--root"])
        .arg(root.path())
        .output()
        .unwrap();
    assert!(
        !output.status.success(),
        "colliding participants were published"
    );
    let error = String::from_utf8_lossy(&output.stderr);
    assert!(
        error.contains("demo.worker@v1") && error.contains("demo.worker@v2"),
        "{error}"
    );
    assert!(error.contains("participants/"), "{error}");
    for directory in ["artifacts", "ts", "rust"] {
        assert!(!root.path().join(".trellis").join(directory).exists());
    }
}

#[test]
fn later_renderer_failure_preserves_usable_project_and_installed_dependencies() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .unwrap();
    let temp = tempfile::tempdir_in(repo.join("rust/target")).unwrap();
    let root = temp.path();
    fs::write(root.join("trellis.toml"), "format = 1\n").unwrap();
    fs::write(root.join("deno.json"), "{}").unwrap();
    fs::write(
        root.join("Cargo.toml"),
        r#"
[package]
name = "last-good-consumer"
version = "0.1.0"
edition = "2021"
[workspace]
exclude = [".trellis"]
[dependencies]
generated = { package = "trellis-sdk-a-good", path = ".trellis/rust/apis/a-good" }
"#,
    )
    .unwrap();
    fs::create_dir(root.join("src")).unwrap();
    fs::write(root.join("src/main.rs"), "fn main() { let value = generated::GetResponse { value: String::from(\"last good\") }; assert_eq!(value.value, \"last good\"); }\n").unwrap();
    fs::write(root.join("consumer.ts"), "import type { GetOutput } from './.trellis/ts/apis/a-good/mod.ts';\nconst value: GetOutput = { value: 'last good' };\nconsole.log(value.value);\n").unwrap();
    let source = r#"
api "a.good@v1" {
  version "1.0.0"; display_name "Good"; description "Last good consumer.";
  model Empty {}
  model Record { value: string; }
  rpc "Get" { version "v1"; input Empty; output Record; }
}
"#;
    fs::write(root.join("contract.trellis"), source).unwrap();
    let generate = || {
        Command::new(env!("CARGO_BIN_EXE_trellis"))
            .args(["generate", "--root"])
            .arg(root)
            .output()
            .unwrap()
    };
    let output = generate();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let api_path = root.join(".trellis/artifacts/apis/a.good@v1.json");
    let last_good_api = fs::read_to_string(&api_path).unwrap();
    let check_consumers = || {
        let output = Command::new("deno")
            .args(["check", "-c"])
            .arg(repo.join("ts/deno.json"))
            .arg(root.join("consumer.ts"))
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let output = Command::new(env!("CARGO"))
            .args(["run", "--quiet", "--manifest-path"])
            .arg(root.join("Cargo.toml"))
            .env(
                "CARGO_TARGET_DIR",
                repo.join("rust/target/codegen-consumers"),
            )
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    };
    check_consumers();
    fs::create_dir_all(root.join(".trellis/apis")).unwrap();
    fs::write(
        root.join(".trellis/apis/installed.txt"),
        "installed dependency",
    )
    .unwrap();
    fs::write(
        root.join("contract.trellis"),
        format!(
            "{}\n{}",
            source.replace("value: string", "value: int"),
            r#"
api "z.bad@v1" {
  version "1.0.0"; display_name "Bad"; description "Later renderer failure.";
  model Collision { sameName: string; same_name: string; }
}
"#
        ),
    )
    .unwrap();
    let output = generate();
    assert!(
        !output.status.success(),
        "colliding Rust fields were accepted"
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("generated Rust"),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(fs::read_to_string(api_path).unwrap(), last_good_api);
    assert_eq!(
        fs::read_to_string(root.join(".trellis/apis/installed.txt")).unwrap(),
        "installed dependency"
    );

    check_consumers();
}
