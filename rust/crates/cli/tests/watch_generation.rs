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

fn wait_for(mut ready: impl FnMut() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(60);
    while !ready() {
        assert!(
            Instant::now() < deadline,
            "watch did not produce usable output"
        );
        thread::sleep(Duration::from_millis(100));
    }
}

fn check(root: &Path, config: &Path) -> bool {
    let output = Command::new("deno")
        .args(["check", "-c"])
        .arg(config)
        .arg(root.join("consumer.ts"))
        .output()
        .unwrap();
    if !output.status.success() {
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
    }
    output.status.success()
}

#[test]
fn watch_recovers_after_invalid_dependency_change() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let temp = tempfile::tempdir_in(repo.join("rust/target")).unwrap();
    let root = temp.path();
    let dependency = root.join("dependency");
    fs::create_dir(&dependency).unwrap();
    fs::write(root.join("deno.json"), "{}").unwrap();
    fs::write(
        root.join("trellis.toml"),
        "format = 1\n[apis.\"example.b@v1\"]\nversion = \"1.0.0\"\npath = \"dependency\"\n",
    )
    .unwrap();
    fs::write(dependency.join("trellis.toml"), "format = 1\n").unwrap();
    fs::write(
        root.join("contract.trellis"),
        r#"
api "example.a@v1" { version "1.0.0"; display_name "A"; description "Caller."; }
participant "example.a" app {
    implements "example.a@v1";
    use required b "example.b@v1" { call rpc "Status.Get"; }
}
"#,
    )
    .unwrap();
    let source = r#"
api "example.b@v1" {
    version "1.0.0"; display_name "B"; description "Watch dependency.";
    model Request {}
    model Reply { ok: bool; }
    rpc "Status.Get" { version "v1"; input Request; output Reply; }
}
"#;
    fs::write(dependency.join("contract.trellis"), source).unwrap();
    fs::write(root.join("consumer.ts"), "import type { StatusGetOutput } from './.trellis/ts/apis/example-b/mod.ts';\nconst value: StatusGetOutput = { ok: true };\nconsole.log(value.ok);\n").unwrap();
    let errors = root.join("watch-errors.log");
    let mut watcher = ChildGuard(
        Command::new(env!("CARGO_BIN_EXE_trellis"))
            .args(["generate", "--watch", "--root"])
            .arg(root)
            .stdout(Stdio::null())
            .stderr(fs::File::create(&errors).unwrap())
            .spawn()
            .unwrap(),
    );
    let config = repo.join("ts/deno.json");
    wait_for(|| check(root, &config));

    fs::write(
        dependency.join("contract.trellis"),
        "api \"example.b@v1\" {",
    )
    .unwrap();
    wait_for(|| {
        fs::read_to_string(&errors)
            .unwrap()
            .contains("contract.trellis")
    });
    assert!(watcher.0.try_wait().unwrap().is_none());
    assert!(
        check(root, &config),
        "invalid authoring destroyed usable output"
    );

    fs::write(root.join("consumer.ts"), "import type { StatusGetOutput } from './.trellis/ts/apis/example-b/mod.ts';\nconst value: StatusGetOutput = { ok: true, detail: 'recovered' };\nconsole.log(value.detail.toUpperCase());\n").unwrap();
    assert!(!check(root, &config), "new field must require regeneration");
    fs::write(
        dependency.join("contract.trellis"),
        source.replace("ok: bool;", "ok: bool; detail: string;"),
    )
    .unwrap();
    wait_for(|| check(root, &config));
    assert!(watcher.0.try_wait().unwrap().is_none());
}
