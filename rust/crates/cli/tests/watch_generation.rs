use std::{
    fs,
    path::Path,
    process::{Child, Command, Stdio},
    thread,
    time::{Duration, Instant},
};

struct ChildGuard(Child);

impl Drop for ChildGuard {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn write_api(path: &Path, id: &str, extra_field: bool) {
    fs::write(
        path.join("contract.trellis"),
        format!(
            r#"api "{id}" {{
    version "1.0.0";
    display_name "{id}";
    description "Watch fixture.";
    model Request {{}}
    model Reply {{ ok: bool; {} }}
    rpc "Status.Get" {{ version "v1"; input Request; output Reply; }}
}}
"#,
            if extra_field { "detail?: string;" } else { "" }
        ),
    )
    .unwrap();
}

fn wait_for(path: &Path, predicate: impl Fn(&[u8]) -> bool) -> Vec<u8> {
    let deadline = Instant::now() + Duration::from_secs(10);
    while Instant::now() < deadline {
        if let Ok(contents) = fs::read(path) {
            if predicate(&contents) {
                return contents;
            }
        }
        thread::sleep(Duration::from_millis(50));
    }
    panic!("timed out waiting for {}", path.display());
}

#[test]
fn watch_recovers_after_invalid_sibling_dependency_change() {
    let temp = tempfile::tempdir().unwrap();
    let a = temp.path().join("a");
    let b = temp.path().join("b");
    fs::create_dir_all(&a).unwrap();
    fs::create_dir_all(&b).unwrap();
    fs::write(
        temp.path().join("Cargo.toml"),
        "[workspace]\nmembers = [\"a\"]\nresolver = \"2\"\n\n[workspace.package]\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .unwrap();
    fs::write(
        a.join("Cargo.toml"),
        "[package]\nname = \"a\"\nversion.workspace = true\nedition.workspace = true\n",
    )
    .unwrap();
    fs::write(
        a.join("contract.trellis"),
        r#"api "example.a@v1" {
    version "1.0.0";
    display_name "A";
    description "A fixture.";
    model Request {}
    model Reply { ok: bool; }
    rpc "Status.Get" { version "v1"; input Request; output Reply; }
}
participant "example.a" service {
    implements "example.a@v1";
    use required b "example.b@v1" { call rpc "Status.Get"; }
}
"#,
    )
    .unwrap();
    fs::write(
        a.join("trellis.toml"),
        "format = 1\n\n[apis.\"example.b@v1\"]\nversion = \"1.0.0\"\npath = \"../b\"\n",
    )
    .unwrap();
    write_api(&b, "example.b@v1", false);
    fs::write(b.join("trellis.toml"), "format = 1\n").unwrap();

    let mut child = ChildGuard(
        Command::new(env!("CARGO_BIN_EXE_trellis"))
            .args(["generate", "--watch", "--root"])
            .arg(&a)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap(),
    );
    let a_stem = trellis_generation::artifacts::sdk_output_stem("example.a@v1");
    let b_stem = trellis_generation::artifacts::sdk_output_stem("example.b@v1");
    let participant_stem = trellis_generation::artifacts::sdk_output_stem("example.a");
    let participant_path = a.join(".trellis/artifacts/participants/example.a.json");
    wait_for(&a.join(".trellis/artifacts/apis/example.a@v1.json"), |_| {
        true
    });
    let a_sdk = a.join(".trellis/rust/apis").join(a_stem);
    let b_sdk = a.join(".trellis/rust/apis").join(b_stem);
    let facade = a.join(".trellis/rust/participants").join(participant_stem);
    wait_for(&a_sdk.join("Cargo.toml"), |_| true);
    wait_for(&b_sdk.join("Cargo.toml"), |_| true);
    let facade_manifest = wait_for(&facade.join("Cargo.toml"), |_| true);
    let facade_manifest: toml::Value =
        toml::from_str(&String::from_utf8(facade_manifest).unwrap()).unwrap();
    let b_crate = trellis_generation::artifacts::default_rust_crate_name_from_id("example.b@v1");
    let b_path = facade_manifest["dependencies"][b_crate.as_str()]["path"]
        .as_str()
        .unwrap();
    assert_eq!(
        facade.join(b_path).canonicalize().unwrap(),
        b_sdk.canonicalize().unwrap()
    );
    let initial = wait_for(&participant_path, |_| true);

    fs::write(b.join("contract.trellis"), "temporarily invalid").unwrap();
    thread::sleep(Duration::from_millis(600));
    assert!(child.0.try_wait().unwrap().is_none());
    assert_eq!(fs::read(&participant_path).unwrap(), initial);

    write_api(&b, "example.b@v1", true);
    let changed = wait_for(&participant_path, |contents| contents != initial);
    let participant: serde_json::Value = serde_json::from_slice(&changed).unwrap();
    let expected = trellis_idl::compile_project(&b).unwrap().apis["example.b@v1"]
        .digest()
        .unwrap();
    assert_eq!(participant["uses"]["required"]["b"]["apiDigest"], expected);
    wait_for(&b_sdk.join("src/types.rs"), |contents| {
        String::from_utf8_lossy(contents).contains("detail")
    });
}
