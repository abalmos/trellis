use std::{fs, path::Path, process::Command};

#[test]
fn safe_outputs_and_check_preserve_authored_and_last_good_files() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let manifest =
        "format = 1\nname = \"example-trellis\"\n[generate]\noutput = \"src/generated\"\n";
    fs::write(root.join("trellis.toml"), manifest).unwrap();
    fs::write(root.join("package.json"), "{}").unwrap();
    let source = "api \"example.api@v1\" { version \"1.0.0\"; display_name \"Example\"; description \"Original\"; }\n";
    fs::write(root.join("contract.trellis"), source).unwrap();
    let run = |args: &[&str]| {
        Command::new(env!("CARGO_BIN_EXE_trellis"))
            .args(args)
            .arg("--root")
            .arg(root)
            .env("TRELLIS_CACHE", root.join("unused-cache"))
            .output()
            .unwrap()
    };
    let missing = run(&["generate", "--check"]);
    assert!(!missing.status.success());
    assert!(String::from_utf8_lossy(&missing.stderr).contains("missing:"));
    assert!(!root.join("src").exists(), "check created an output parent");
    assert!(!run(&["generate", "--watch", "--check"]).status.success());

    fs::create_dir_all(root.join("src/generated/empty")).unwrap();
    fs::write(root.join("src/main.ts"), "export const authored = true;\n").unwrap();
    let generated = run(&["generate"]);
    assert!(
        generated.status.success(),
        "{}",
        String::from_utf8_lossy(&generated.stderr)
    );
    let package = root.join("src/generated/package.json");
    let original = fs::read(&package).unwrap();
    for (length, accepted) in [(214, true), (215, false)] {
        fs::write(
            root.join("trellis.toml"),
            manifest.replace("example-trellis", &"a".repeat(length)),
        )
        .unwrap();
        let output = run(&["generate"]);
        assert_eq!(
            output.status.success(),
            accepted,
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    fs::write(root.join("trellis.toml"), manifest).unwrap();
    assert!(run(&["generate"]).status.success());
    assert_eq!(fs::read(&package).unwrap(), original);
    let metadata: serde_json::Value = serde_json::from_slice(&original).unwrap();
    assert_eq!(metadata["name"], "example-trellis");
    assert_eq!(metadata["version"], "0.0.0");
    let modified = fs::metadata(&package).unwrap().modified().unwrap();
    let clean = run(&["generate", "--check"]);
    assert!(
        clean.status.success(),
        "{}",
        String::from_utf8_lossy(&clean.stderr)
    );
    assert_eq!(
        fs::metadata(&package).unwrap().modified().unwrap(),
        modified
    );

    fs::write(&package, "{}").unwrap();
    fs::remove_file(root.join("src/generated/index.js")).unwrap();
    fs::write(root.join("src/generated/extra.txt"), "unrelated").unwrap();
    let drift = run(&["generate", "--check"]);
    let error = String::from_utf8_lossy(&drift.stderr);
    assert!(!drift.status.success());
    for kind in ["missing:", "stale:", "extra:"] {
        assert!(error.contains(kind), "{error}");
    }
    assert_eq!(fs::read(&package).unwrap(), b"{}");
    assert!(!root.join("src/generated/index.js").exists());
    assert_eq!(
        fs::read(root.join("src/generated/extra.txt")).unwrap(),
        b"unrelated"
    );
    assert!(!run(&["generate"]).status.success());
    // Explicit test-owned preparation, never automatic recognition of old packages.
    fs::remove_dir_all(root.join("src/generated")).unwrap();
    assert!(run(&["generate"]).status.success());

    for destination in [
        ".",
        "src/..",
        "../outside",
        "/tmp/trellis-outside",
        "contracts",
        "src",
    ] {
        fs::write(
            root.join("trellis.toml"),
            manifest.replace("src/generated", destination),
        )
        .unwrap();
        assert!(
            !run(&["generate"]).status.success(),
            "accepted {destination}"
        );
        assert_eq!(fs::read(&package).unwrap(), original);
    }
    for name in [
        "Upper",
        "@scope/name",
        "under_score",
        "two--hyphens",
        "trailing-",
        "1first",
    ] {
        fs::write(
            root.join("trellis.toml"),
            manifest.replace("example-trellis", name),
        )
        .unwrap();
        assert!(!run(&["generate"]).status.success(), "accepted {name}");
        assert_eq!(fs::read(&package).unwrap(), original);
    }
    fs::write(
        root.join("trellis.toml"),
        format!("{manifest}\n[generate.rust]\noutput = \"rust-generated\"\n"),
    )
    .unwrap();
    assert!(!run(&["generate"]).status.success());

    fs::create_dir_all(root.join("dependencies/producer")).unwrap();
    fs::write(
        root.join("dependencies/producer/contract.trellis"),
        source.replace("example.api", "example.dependency"),
    )
    .unwrap();
    for destination in ["dependencies", "dependencies/producer/generated"] {
        fs::write(root.join("trellis.toml"), format!("{}\n[apis.\"example.dependency@v1\"]\nversion = \"1.0.0\"\npath = \"dependencies/producer\"\n", manifest.replace("src/generated", destination))).unwrap();
        assert!(
            !run(&["generate"]).status.success(),
            "accepted {destination}"
        );
        assert_eq!(fs::read(&package).unwrap(), original);
    }

    fs::write(root.join("trellis.toml"), manifest).unwrap();
    fs::write(root.join("contract.trellis"), "api {").unwrap();
    assert!(!run(&["generate"]).status.success());
    assert_eq!(fs::read(&package).unwrap(), original);
    fs::write(root.join("contract.trellis"), source).unwrap();
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(root.join("src"), root.join("linked")).unwrap();
        fs::write(
            root.join("trellis.toml"),
            manifest.replace("src/generated", "linked/generated"),
        )
        .unwrap();
        assert!(!run(&["generate"]).status.success());
        fs::write(root.join("trellis.toml"), manifest).unwrap();
        std::os::unix::fs::symlink(
            root.join("contract.trellis"),
            root.join("src/generated/link"),
        )
        .unwrap();
        assert!(!run(&["generate", "--check"]).status.success());
        assert!(!run(&["generate"]).status.success());
    }
    assert_eq!(
        fs::read_to_string(root.join("src/main.ts")).unwrap(),
        "export const authored = true;\n"
    );
    assert!(!root.join("unused-cache").exists());
    assert_eq!(metadata["trellisGenerated"], true);
}

#[tokio::test]
async fn api_only_install_and_publish_need_no_lock_or_language_name() {
    let server = registry_testkit::RegistryServer::new(registry_testkit::RegistryConfig::memory())
        .await
        .unwrap();
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    fs::write(root.join("trellis.toml"), format!("format = 1\ndefault-registry = \"local\"\n[registries.local]\nprefix = \"127.0.0.1:{}\"\n", server.port())).unwrap();
    fs::write(root.join("contract.trellis"), "api \"example.only@v1\" { version \"1.0.0\"; display_name \"Only\"; description \"API only\"; }\n").unwrap();
    fs::create_dir(root.join("nested")).unwrap();
    for marker in ["Cargo.toml", "package.json", "deno.json", "deno.jsonc"] {
        fs::write(root.join("nested").join(marker), "").unwrap();
    }
    for command in ["install", "publish"] {
        let output = tokio::process::Command::new(env!("CARGO_BIN_EXE_trellis"))
            .args([command, "--root"])
            .arg(root)
            .output()
            .await
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(!root.join("trellis.lock").exists());
        assert!(!root.join("trellis").exists());
    }
    for flags in [vec!["generate"], vec!["generate", "--check"]] {
        let output = Command::new(env!("CARGO_BIN_EXE_trellis"))
            .args(flags)
            .arg("--root")
            .arg(root)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(!root.join("trellis.lock").exists());
        assert!(!root.join("trellis").exists());
    }
    fs::write(root.join("contract.trellis"), "api {").unwrap();
    let invalid = Command::new(env!("CARGO_BIN_EXE_trellis"))
        .args(["generate", "--root"])
        .arg(root)
        .output()
        .unwrap();
    assert!(!invalid.status.success());
    assert!(String::from_utf8_lossy(&invalid.stderr).contains("contract.trellis"));
}

#[test]
fn typescript_generation_needs_no_external_tools_and_is_deterministic() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    fs::write(
        root.join("trellis.toml"),
        "format = 1\nname = \"example-trellis\"\n",
    )
    .unwrap();
    fs::write(root.join("deno.json"), "{}").unwrap();
    fs::write(
        root.join("contract.trellis"),
        r#"
api "acme.example@v1" {
  version "1.0.0"; display_name "Example"; description "Native emission.";
  model Empty {}
  model Output { value: string; }
  rpc "Example.Read" { version "v1"; input Empty; output Output; }
}
"#,
    )
    .unwrap();
    let mut previous = None;
    for _ in 0..2 {
        let output = Command::new(env!("CARGO_BIN_EXE_trellis"))
            .args(["generate", "--root"])
            .arg(root)
            .env("PATH", "")
            .env("TRELLIS_CACHE", root.join("empty-cache"))
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let files = [
            "package.json",
            "index.js",
            "index.d.ts",
            "apis/acme-example/descriptors.js",
            "apis/acme-example/descriptors.d.ts",
            "artifacts/apis/acme.example@v1.json",
        ]
        .map(|file| fs::read(root.join("trellis").join(file)).unwrap());
        if let Some(previous) = previous {
            assert_eq!(files, previous);
        }
        previous = Some(files);
    }
    assert!(!root.join(".trellis").exists());
    assert!(!root.join("empty-cache").exists());
}

#[test]
fn native_runtime_packages_preserve_public_types_and_execute() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let mut apis = std::collections::BTreeMap::new();
    for project in [
        "rust/crates/runtime",
        "rust/crates/jobs-runtime",
        "rust/crates/eventlog-runtime",
    ] {
        apis.extend(
            trellis_idl::compile_apis(&trellis_idl::parse_project(&repo.join(project)).unwrap())
                .unwrap(),
        );
    }
    let participants = trellis_idl::compile_participants(
        &trellis_idl::parse_project(&repo.join("rust/crates/runtime")).unwrap(),
        &apis,
    )
    .unwrap();
    let output = tempfile::tempdir().unwrap();
    trellis_codegen_ts::generate_ts_package(
        &apis.iter().map(|(id, api)| (id.as_str(), api)).collect(),
        &participants,
        output.path(),
        "runtime-trellis",
    )
    .unwrap();
    let consumer = output.path().join("consumer.ts");
    fs::write(&consumer, r#"
import { apis, participants } from "./index.js";
import { type RpcInputOf } from "@qlever-llc/trellis";
import { runtimeApiFromActions, PARTICIPANT_KV_METADATA } from "@qlever-llc/trellis/generated";
const name: "State.Put" = apis.state.StatePut.name;
const connected: "statePut" = apis.state.StatePut.connectedName;
const stateApi = runtimeApiFromActions([apis.state.StatePut] as const);
type Request = RpcInputOf<typeof stateApi, "State.Put">;
const key: Request["key"] = "record";
// @ts-expect-error Generated RPC keys must not widen to unknown.
const badKey: Request["key"] = 123;
// @ts-expect-error Generated action identity must retain its literal type.
const badName: "Different.Put" = apis.state.StatePut.name;
type Flow = typeof participants.authRuntime.participant[typeof PARTICIPANT_KV_METADATA]["browserFlows"]["value"];
const flowState: Flow["state"] = "approved";
// @ts-expect-error Participant KV value types must retain the authored union.
const badState: Flow["state"] = "invalid-state";
if (name !== "State.Put" || connected !== "statePut" || participants.authRuntime.participant.id !== "trellis.auth-runtime") throw new Error("invalid emitted runtime exports");
void [key, badKey, badName, flowState, badState];
"#).unwrap();
    for mode in ["check", "run"] {
        let result = Command::new("deno")
            .arg(mode)
            .arg("--config")
            .arg(repo.join("ts/deno.json"))
            .arg(&consumer)
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "deno {mode} failed: {}",
            String::from_utf8_lossy(&result.stderr)
        );
    }
    let rust = tempfile::tempdir().unwrap();
    trellis_codegen_rust::generate_rust_package(
        &apis.iter().map(|(id, api)| (id.as_str(), api)).collect(),
        &participants,
        &rust.path().join("trellis"),
        "runtime-trellis",
    )
    .unwrap();
    fs::write(rust.path().join("Cargo.toml"), "[package]\nname = \"runtime-consumer\"\nversion = \"0.0.0\"\nedition = \"2021\"\n[dependencies]\nruntime-trellis = { path = \"trellis\" }\n").unwrap();
    fs::create_dir(rust.path().join("src")).unwrap();
    fs::write(rust.path().join("src/lib.rs"), "use runtime_trellis::{apis, participants};\npub fn surfaces() { let _: Option<apis::state::rpc::StatePutRpc> = None; let _: Option<participants::auth_runtime::Participant> = None; }\n").unwrap();
    let result = Command::new("cargo")
        .arg("check")
        .arg("--manifest-path")
        .arg(rust.path().join("Cargo.toml"))
        .arg("--config")
        .arg(format!(
            "patch.crates-io.trellis-rs.path={:?}",
            repo.join("rust/crates/trellis").canonicalize().unwrap()
        ))
        .env(
            "CARGO_TARGET_DIR",
            repo.join("rust/target/codegen-consumers"),
        )
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "ordinary generated Rust consumer failed: {}",
        String::from_utf8_lossy(&result.stderr)
    );
}

#[test]
fn participant_output_collisions_fail_before_publication() {
    let root = tempfile::tempdir().unwrap();
    fs::write(
        root.path().join("trellis.toml"),
        "format = 1\nname = \"collision-trellis\"\n",
    )
    .unwrap();
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
    assert!(!root.path().join("trellis").exists());
    assert!(!root.path().join(".trellis").exists());
}

#[test]
fn later_renderer_failure_preserves_usable_packages_and_user_files() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .unwrap();
    let temp = tempfile::tempdir_in(repo.join("rust/target")).unwrap();
    let root = temp.path();
    fs::write(root.join("trellis.toml"), "format = 1\nname = \"last-good-trellis\"\n[generate.rust]\noutput = \"crates/last-good-trellis\"\n[generate.typescript]\noutput = \"packages/last-good-trellis\"\n").unwrap();
    fs::write(root.join("deno.json"), "{}").unwrap();
    fs::write(
        root.join("Cargo.toml"),
        r#"
[package]
name = "last-good-consumer"
version = "0.1.0"
edition = "2021"
[workspace]
[dependencies]
generated = { package = "last-good-trellis", path = "crates/last-good-trellis" }
"#,
    )
    .unwrap();
    fs::create_dir(root.join("src")).unwrap();
    fs::write(root.join("src/main.rs"), "fn main() { let value = generated::apis::a_good::GetResponse { value: String::from(\"last good\") }; assert_eq!(value.value, \"last good\"); }\n").unwrap();
    fs::write(root.join("consumer.ts"), "import type { apis } from './packages/last-good-trellis/index.js';\nconst value: apis.aGood.GetOutput = { value: 'last good' };\nconsole.log(value.value);\n").unwrap();
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
    let manifest = fs::read_to_string(root.join("trellis.toml")).unwrap();
    let rust_package = root.join("crates/last-good-trellis/Cargo.toml");
    let original_package = fs::read(&rust_package).unwrap();
    let metadata: toml::Value = toml::from_slice(&original_package).unwrap();
    assert_eq!(
        metadata["package"]["metadata"]["trellis"]["generated"].as_bool(),
        Some(true)
    );
    assert_eq!(metadata["package"]["version"].as_str(), Some("0.0.0"));
    for invalid in [
        manifest.replace("packages/last-good-trellis", "crates/last-good-trellis"),
        manifest.replace(
            "packages/last-good-trellis",
            "crates/last-good-trellis/nested",
        ),
        "format = 1\nname = \"last-good-trellis\"\n[generate]\noutput = \"shared\"\n".into(),
        "format = 1\nname = \"last-good-trellis\"\n[generate.rust]\noutput = \"rust-only\"\n"
            .into(),
    ] {
        fs::write(root.join("trellis.toml"), invalid).unwrap();
        assert!(!generate().status.success());
        assert_eq!(fs::read(&rust_package).unwrap(), original_package);
    }
    fs::write(root.join("trellis.toml"), manifest).unwrap();
    for package in ["crates/last-good-trellis", "packages/last-good-trellis"] {
        fs::write(root.join(package).join("user.txt"), "user file").unwrap();
    }
    let repeated = generate();
    assert!(
        !repeated.status.success(),
        "{}",
        String::from_utf8_lossy(&repeated.stderr)
    );
    for package in ["crates/last-good-trellis", "packages/last-good-trellis"] {
        assert_eq!(
            fs::read_to_string(root.join(package).join("user.txt")).unwrap(),
            "user file"
        );
        fs::remove_file(root.join(package).join("user.txt")).unwrap();
    }
    let api_path = root.join("crates/last-good-trellis/artifacts/apis/a.good@v1.json");
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
            .arg("--config")
            .arg(format!(
                "patch.crates-io.trellis-rs.path={:?}",
                repo.join("rust/crates/trellis")
            ))
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
    assert!(!root.join(".trellis").exists());

    check_consumers();
    fs::write(root.join("contract.trellis"), source).unwrap();
    let recovered = generate();
    assert!(
        recovered.status.success(),
        "{}",
        String::from_utf8_lossy(&recovered.stderr)
    );

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let directory = root.join("packages/last-good-trellis/artifacts/apis");
        let permissions = fs::metadata(&directory).unwrap().permissions();
        fs::write(
            root.join("contract.trellis"),
            source.replace("value: string", "value: int"),
        )
        .unwrap();
        fs::set_permissions(&directory, fs::Permissions::from_mode(0o555)).unwrap();
        let failed = generate();
        fs::set_permissions(&directory, permissions).unwrap();
        assert!(
            !failed.status.success(),
            "read-only publication destination was accepted"
        );
        assert_eq!(
            fs::read_to_string(root.join("crates/last-good-trellis/artifacts/apis/a.good@v1.json"))
                .unwrap(),
            last_good_api
        );
        check_consumers();
        fs::write(root.join("contract.trellis"), source).unwrap();
    }

    // A canonical artifact alone does not establish ownership of its directory.
    let unowned = root.join("unowned/artifacts/apis/a.good@v1.json");
    fs::create_dir_all(unowned.parent().unwrap()).unwrap();
    fs::write(&unowned, &last_good_api).unwrap();
    let manifest = fs::read_to_string(root.join("trellis.toml")).unwrap();
    fs::write(
        root.join("trellis.toml"),
        manifest.replace("crates/last-good-trellis", "unowned"),
    )
    .unwrap();
    let collision = generate();
    assert!(!collision.status.success());
    assert!(
        String::from_utf8_lossy(&collision.stderr).contains("refusing to overwrite unrelated file")
    );
    assert_eq!(fs::read_to_string(unowned).unwrap(), last_good_api);
    assert!(!root.join("unowned/Cargo.toml").exists());
}
