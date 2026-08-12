use std::process::Command;

use serde_json::Value;
use trellis_contracts::{load_sdk_source, ContractBuilder};

fn repo_root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("repo root")
}

fn evaluate_contract(source: &str) -> Value {
    let root = repo_root();
    let source = std::path::Path::new(source);
    let output = if source
        .extension()
        .is_some_and(|extension| extension == "ts")
    {
        Command::new("deno")
            .arg("eval")
            .arg(format!(
                "const m=await import('file://{}');const c=m.default;const r=c[Symbol.for('trellis.contract.runtime')];const a=[...new Map(r.actions.flatMap(({{action}})=>{{const s=action[Symbol.for('trellis.action.metadata')]?.source;return s?[[s.api.id,s.api]]:[]}})).values()];console.log(JSON.stringify({{api:c.API,participant:c.PARTICIPANT,referencedApis:a}}))",
                source.canonicalize().unwrap().display()
            ))
            .current_dir(root.join("demos/js"))
            .output()
            .expect("evaluate TypeScript contract")
    } else {
        let temp = tempfile::tempdir().expect("temp dir");
        let main = temp.path().join("src/main.rs");
        std::fs::create_dir(temp.path().join("src")).unwrap();
        std::fs::write(
            temp.path().join("Cargo.toml"),
            format!("[package]\nname='demo-parity-eval'\nversion='0.0.0'\nedition='2021'\n[dependencies]\nserde_json='1'\ntrellis-contracts={{path={:?}}}\n", root.join("rust/crates/contracts")),
        ).unwrap();
        std::fs::write(&main, format!("#[path={:?}] mod contract; fn main() -> Result<(),Box<dyn std::error::Error>> {{ let a=contract::contract_artifacts()?; println!(\"{{}}\",serde_json::to_string(&serde_json::json!({{\"api\":a.api_value()?,\"participant\":a.participant_value()?,\"referencedApis\":a.referenced_apis().values().map(|v|v.normalized_value()).collect::<Result<Vec<_>,_>>()?}}))?); Ok(()) }}", source.canonicalize().unwrap())).unwrap();
        Command::new("cargo")
            .args(["run", "--quiet"])
            .current_dir(temp.path())
            .output()
            .expect("evaluate Rust contract")
    };
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("native contract presentation")
}

fn generate_manifest(source: &str, out: &std::path::Path) {
    let output = Command::new(env!("CARGO_BIN_EXE_trellis-generate"))
        .args(["generate", "api", "--source", source, "--out"])
        .arg(out)
        .output()
        .expect("run trellis-generate api");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn generate_demo_service_jsr_package(root: &std::path::Path) {
    let output = Command::new(env!("CARGO_BIN_EXE_trellis-generate"))
        .args([
            "generate",
            "jsr",
            "--source",
            root.join("demos/js/service/contract.ts").to_str().unwrap(),
            "--out",
            root.join("demos/js/generated/packages/jsr/demo-service")
                .to_str()
                .unwrap(),
        ])
        .output()
        .expect("run trellis-generate jsr");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn assert_manifest_parity(js_source: &str, rust_source: &str) {
    let temp = tempfile::tempdir().expect("temp dir");
    let js_out = temp.path().join("js.json");
    let rust_out = temp.path().join("rust.json");

    generate_manifest(js_source, &js_out);
    generate_manifest(rust_source, &rust_out);

    let js = load_sdk_source(&js_out).expect("load js API artifact");
    let rust = load_sdk_source(&rust_out).expect("load rust API artifact");

    assert_eq!(rust.canonical, js.canonical);
    assert_eq!(rust.digest, js.digest);

    let js = evaluate_contract(js_source);
    let rust = evaluate_contract(rust_source);
    let build = |value: Value| {
        let mut builder =
            ContractBuilder::from_native(value["api"].clone(), value["participant"].clone());
        for api in value["referencedApis"].as_array().unwrap() {
            builder = builder.referenced_api(api["id"].as_str().unwrap(), api.clone());
        }
        builder.build().unwrap()
    };
    let js = build(js);
    let rust = build(rust);
    assert_eq!(
        rust.api().canonical_json().unwrap(),
        js.api().canonical_json().unwrap()
    );
    assert_eq!(rust.api_digest().unwrap(), js.api_digest().unwrap());
    assert_eq!(
        rust.participant_value().unwrap(),
        js.participant_value().unwrap()
    );
    assert_eq!(
        rust.participant().canonical_json().unwrap(),
        js.participant().canonical_json().unwrap()
    );
    assert_eq!(
        rust.participant_digest().unwrap(),
        js.participant_digest().unwrap()
    );
    assert_eq!(
        rust.participant_needs_digest().unwrap(),
        js.participant_needs_digest().unwrap()
    );
    assert_eq!(rust.required_grants(), js.required_grants());
    assert_eq!(rust.optional_grants(), js.optional_grants());
}

#[test]
fn rust_authored_demo_service_contract_matches_js_contract() {
    let root = repo_root();
    assert_manifest_parity(
        root.join("demos/js/service/contract.ts").to_str().unwrap(),
        root.join("demos/rust/contracts/service.rs")
            .to_str()
            .unwrap(),
    );
}

#[test]
fn rust_authored_demo_device_contract_matches_js_contract() {
    let root = repo_root();
    generate_demo_service_jsr_package(&root);
    assert_manifest_parity(
        root.join("demos/js/device/contract.ts").to_str().unwrap(),
        root.join("demos/rust/contracts/device.rs")
            .to_str()
            .unwrap(),
    );
}
