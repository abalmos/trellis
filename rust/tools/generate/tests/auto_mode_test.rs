use std::fs;
use std::path::Path;
use std::process::Command;

use trellis_generate::artifacts::trellis_package_version;

fn write_ts_contract(path: &Path, id: &str, display_name: &str, kind: &str) {
    let api_digest = trellis_contracts::ApiBuilder::new(serde_json::json!({
        "format": "trellis.api.v1",
        "id": id,
        "displayName": display_name,
        "description": "Fixture API",
    }))
    .digest()
    .unwrap();
    fs::write(
        path,
        format!(
            "const API = {{\n  format: \"trellis.api.v1\",\n  id: \"{id}\",\n  displayName: \"{display_name}\",\n  description: \"Fixture API\",\n}};\nconst PARTICIPANT = {{\n  format: \"trellis.participant.v1\",\n  id: \"{id}\",\n  displayName: \"{display_name}\",\n  description: \"Fixture participant\",\n  kind: \"{kind}\",\n  implements: {{ self: {{ api: \"{id}\", apiDigest: \"{api_digest}\" }} }},\n}};\n\nexport default {{ API, PARTICIPANT }};\n"
        ),
    )
    .unwrap();
}

fn write_orders_contract(path: &Path, description: &str) {
    let api_digest = trellis_contracts::ApiBuilder::new(serde_json::json!({
        "format": "trellis.api.v1",
        "id": "trellis.orders@v1",
        "displayName": "Orders",
        "description": description,
        "schemas": {
            "Empty": { "type": "object", "properties": {}, "required": [] },
            "Order": {
                "type": "object",
                "properties": {
                    "id": { "type": "string" },
                    "revision": { "const": description }
                },
                "required": ["id", "revision"]
            }
        },
        "rpc": {
            "Orders.Get": {
                "version": "v1",
                "input": { "schema": "Empty" },
                "output": { "schema": "Order" }
            }
        },
        "operations": {},
        "events": {}
    }))
    .digest()
    .unwrap();
    fs::write(
        path,
        format!(
            r#"const API = {{
  format: "trellis.api.v1",
  id: "trellis.orders@v1",
  displayName: "Orders",
  description: "{description}",
  schemas: {{
    Empty: {{ type: "object", properties: {{}}, required: [] }},
    Order: {{
      type: "object",
      properties: {{ id: {{ type: "string" }}, revision: {{ const: "{description}" }} }},
      required: ["id", "revision"],
    }},
  }},
  rpc: {{
    "Orders.Get": {{
      version: "v1",
      input: {{ schema: "Empty" }},
      output: {{ schema: "Order" }},
    }},
  }},
  operations: {{}},
  events: {{}},
}};
const PARTICIPANT = {{
  format: "trellis.participant.v1",
  id: "trellis.orders-service@v1",
  displayName: API.displayName,
  description: "Fixture service participant",
  kind: "service",
  implements: {{ self: {{ api: API.id, apiDigest: "{api_digest}" }} }},
}};

export default {{ API, PARTICIPANT }};
"#,
        ),
    )
    .unwrap();
}

fn write_rust_contract(path: &Path, manifest_name: &str, participant_id: &str, kind: &str) {
    fs::write(
        path,
        format!(
            r#"pub fn api_artifact() -> Result<trellis_contracts::ApiArtifact, trellis_contracts::ContractsError> {{
    let mut source: serde_json::Value = serde_json::from_str(include_str!("{manifest_name}"))?;
    if let Some(source) = source.as_object_mut() {{
        source.remove("kind");
        source.remove("uses");
        for method in source.get_mut("rpc").and_then(serde_json::Value::as_object_mut).into_iter().flat_map(|rpc| rpc.values_mut()) {{
            method.as_object_mut().map(|method| method.remove("subject"));
        }}
    }}
    trellis_contracts::ApiBuilder::new(source).build()
}}

pub fn contract_artifacts() -> Result<trellis_contracts::ContractArtifacts, trellis_contracts::ContractsError> {{
    let api = api_artifact()?.normalized_value()?;
    trellis_contracts::ContractBuilder::from_api("{participant_id}", api, trellis_contracts::ContractKind::{kind})?.build()
}}
"#
        ),
    )
    .unwrap();
}

fn write_rust_manifest(path: &Path, version: &str) {
    fs::write(
        path,
        format!("[package]\nname = \"fixture\"\nversion = \"{version}\"\nedition = \"2021\"\n"),
    )
    .unwrap();
}

fn run_prepare(root: &Path) -> std::process::Output {
    trellis_generate()
        .env("TRELLIS_PREPARE_CACHE", root.join("target/prepare-cache"))
        .args(["prepare", root.to_str().unwrap()])
        .output()
        .unwrap()
}

fn run_prepare_timings(root: &Path) -> std::process::Output {
    trellis_generate()
        .env("TRELLIS_PREPARE_CACHE", root.join("target/prepare-cache"))
        .args(["prepare", "--no-npm", "--timings", root.to_str().unwrap()])
        .output()
        .unwrap()
}

fn trellis_generate() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_trellis-generate"));
    command.env("TRELLIS_TSC_BIN", fake_tsc_path());
    command.env(
        "TRELLIS_PREPARE_CACHE",
        std::env::temp_dir()
            .join("trellis-generate-test-cache")
            .join(std::process::id().to_string()),
    );
    command
}

fn fake_tsc_path() -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!(
        "trellis-generate-fake-tsc-{}.sh",
        std::process::id()
    ));
    if !path.exists() {
        write_executable(
            &path,
            r#"#!/bin/sh
set -eu
if [ "${1:-}" = "--version" ]; then
  printf 'Version 0.0.0-test\n'
  exit 0
fi
config=""
while [ "$#" -gt 0 ]; do
  if [ "$1" = "-p" ]; then
    shift
    config="$1"
  fi
  shift || true
done
if [ -z "$config" ]; then
  config="tsconfig.json"
fi
out=$(awk -F'"' '/"outDir"/ { print $4; exit }' "$config")
mkdir -p "$out"
for f in *.ts; do
  base=$(basename "$f" .ts)
  cp "$f" "$out/$base.js"
  cp "$f" "$out/$base.d.ts"
done
"#,
        );
    }
    path
}

#[test]
fn prepare_warm_noop_uses_resolution_cache_and_rebuilds_deleted_target() {
    let temp = tempfile::tempdir().unwrap();
    let project = temp.path().join("service");
    fs::create_dir_all(&project).unwrap();
    fs::write(project.join("deno.json"), "{\"version\":\"0.1.0\"}\n").unwrap();
    write_ts_contract(
        &project.join("contract.ts"),
        "trellis.cached@v1",
        "Cached",
        "service",
    );

    let first = run_prepare_timings(&project);
    assert!(
        first.status.success(),
        "{}",
        String::from_utf8_lossy(&first.stderr)
    );

    let second = run_prepare_timings(&project);
    assert!(
        second.status.success(),
        "{}",
        String::from_utf8_lossy(&second.stderr)
    );
    let stdout = String::from_utf8_lossy(&second.stdout);
    assert!(stdout.contains("hits                     1"), "{stdout}");
    assert!(stdout.contains("misses                   0"), "{stdout}");
    assert!(!stdout.contains("resolve TypeScript"), "{stdout}");
    assert!(!stdout.contains("subprocesses"), "{stdout}");
    assert!(
        stdout.contains("installed               files=0 bytes=0"),
        "{stdout}"
    );

    fs::remove_file(
        project
            .join("generated/packages/jsr/cached")
            .join("descriptors.ts"),
    )
    .unwrap();
    let rebuilt = run_prepare_timings(&project);
    assert!(
        rebuilt.status.success(),
        "{}",
        String::from_utf8_lossy(&rebuilt.stderr)
    );
    let stdout = String::from_utf8_lossy(&rebuilt.stdout);
    assert!(stdout.contains("hits                     1"), "{stdout}");
    assert!(stdout.contains("misses                   0"), "{stdout}");
    assert!(!stdout.contains("resolve TypeScript"), "{stdout}");
    assert!(
        stdout.contains("target jsr            generated=1"),
        "{stdout}"
    );

    let source = project.join("contract.ts");
    let unchanged = fs::read(&source).unwrap();
    std::thread::sleep(std::time::Duration::from_millis(2));
    fs::write(&source, unchanged).unwrap();
    let touched = run_prepare_timings(&project);
    assert!(
        touched.status.success(),
        "{}",
        String::from_utf8_lossy(&touched.stderr)
    );
    let stdout = String::from_utf8_lossy(&touched.stdout);
    assert!(stdout.contains("hits                     1"), "{stdout}");
    assert!(stdout.contains("hashed files                1"), "{stdout}");
    assert!(!stdout.contains("resolve TypeScript"), "{stdout}");

    fs::write(
        &source,
        format!(
            "{}\n// semantic no-op\n",
            fs::read_to_string(&source).unwrap()
        ),
    )
    .unwrap();
    let changed = run_prepare_timings(&project);
    assert!(
        changed.status.success(),
        "{}",
        String::from_utf8_lossy(&changed.stderr)
    );
    let stdout = String::from_utf8_lossy(&changed.stdout);
    assert!(stdout.contains("misses                   1"), "{stdout}");
    assert!(stdout.contains("input changed            1"), "{stdout}");
    assert!(stdout.contains("resolve TypeScript"), "{stdout}");
    assert!(
        stdout.contains("contracts               generated=0 verified=0 skipped=1"),
        "{stdout}"
    );

    let cache_entry = glob::glob(
        &project
            .join("target/prepare-cache/repositories/*/contracts/*.json")
            .to_string_lossy(),
    )
    .unwrap()
    .next()
    .unwrap()
    .unwrap();
    fs::write(&cache_entry, "not json").unwrap();
    let recovered = run_prepare_timings(&project);
    assert!(
        recovered.status.success(),
        "{}",
        String::from_utf8_lossy(&recovered.stderr)
    );
    let stdout = String::from_utf8_lossy(&recovered.stdout);
    assert!(stdout.contains("corrupt                  1"), "{stdout}");
    assert!(stdout.contains("resolve TypeScript"), "{stdout}");

    fs::write(cache_entry.with_extension("resolved.json"), "{broken").unwrap();
    let recovered = run_prepare_timings(&project);
    assert!(
        recovered.status.success(),
        "{}",
        String::from_utf8_lossy(&recovered.stderr)
    );
    let stdout = String::from_utf8_lossy(&recovered.stdout);
    assert!(stdout.contains("corrupt                  1"), "{stdout}");
    assert!(stdout.contains("resolve TypeScript"), "{stdout}");

    let cache_entry = glob::glob(
        &project
            .join("target/prepare-cache/repositories/*/contracts/*.json")
            .to_string_lossy(),
    )
    .unwrap()
    .find(|entry| {
        entry.as_ref().is_ok_and(|path| {
            !path
                .file_name()
                .is_some_and(|name| name.to_string_lossy().ends_with(".resolved.json"))
        })
    })
    .unwrap()
    .unwrap();
    let mut entry: serde_json::Value =
        serde_json::from_slice(&fs::read(&cache_entry).unwrap()).unwrap();
    entry["schema_version"] = serde_json::json!(999);
    fs::write(cache_entry, serde_json::to_vec(&entry).unwrap()).unwrap();
    let schema_miss = run_prepare_timings(&project);
    assert!(
        schema_miss.status.success(),
        "{}",
        String::from_utf8_lossy(&schema_miss.stderr)
    );
    let stdout = String::from_utf8_lossy(&schema_miss.stdout);
    assert!(stdout.contains("invalid schema           1"), "{stdout}");

    let forced = trellis_generate()
        .env(
            "TRELLIS_PREPARE_CACHE",
            project.join("target/prepare-cache"),
        )
        .args([
            "--force",
            "prepare",
            "--no-npm",
            "--timings",
            project.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        forced.status.success(),
        "{}",
        String::from_utf8_lossy(&forced.stderr)
    );
    let stdout = String::from_utf8_lossy(&forced.stdout);
    assert!(stdout.contains("forced                   1"), "{stdout}");
    assert!(stdout.contains("resolve TypeScript"), "{stdout}");
}

fn write_executable(path: &Path, script: &str) {
    fs::write(path, script).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let mut permissions = fs::metadata(path).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(path, permissions).unwrap();
    }
}

#[test]
fn explicit_generate_all_emits_buildable_sdk_packages() {
    let temp = tempfile::tempdir().unwrap();
    let project = temp.path().join("service");
    let manifest_path = temp.path().join("trellis.orders@v1.json");
    let ts_out = temp.path().join("ts");
    let rust_out = temp.path().join("rust");
    fs::create_dir_all(project.join("contracts")).unwrap();
    fs::write(
        project.join("deno.json"),
        "{\n  \"version\": \"0.4.0\"\n}\n",
    )
    .unwrap();
    write_ts_contract(
        &project.join("contracts/orders.ts"),
        "trellis.orders@v1",
        "Orders",
        "service",
    );

    let output = trellis_generate()
        .args([
            "generate",
            "all",
            "--source",
            project.join("contracts/orders.ts").to_str().unwrap(),
            "--out-api",
            manifest_path.to_str().unwrap(),
            "--jsr-out",
            ts_out.to_str().unwrap(),
            "--cargo-out",
            rust_out.to_str().unwrap(),
            "--package-name",
            "@qlever-llc/trellis-sdk-orders-test",
            "--crate-name",
            "trellis-sdk-orders-test",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(manifest_path.exists());
    assert!(ts_out.join("mod.ts").exists());
    assert!(rust_out.join("Cargo.toml").exists());
}

#[test]
fn explicit_generate_all_defaults_out_of_tree_package_to_trellis_sdk_scope() {
    let temp = tempfile::tempdir().unwrap();
    let project = temp.path().join("service");
    let manifest_path = temp.path().join("krishi.cloud@v1.json");
    let ts_out = temp.path().join("ts");
    fs::create_dir_all(project.join("contracts")).unwrap();
    fs::write(
        project.join("deno.json"),
        "{\n  \"version\": \"0.4.0\"\n}\n",
    )
    .unwrap();
    write_ts_contract(
        &project.join("contracts/cloud.ts"),
        "krishi.cloud@v1",
        "Cloud",
        "service",
    );

    let output = trellis_generate()
        .args([
            "generate",
            "all",
            "--source",
            project.join("contracts/cloud.ts").to_str().unwrap(),
            "--out-api",
            manifest_path.to_str().unwrap(),
            "--jsr-out",
            ts_out.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let deno = fs::read_to_string(ts_out.join("deno.json")).unwrap();
    assert!(deno.contains("\"name\": \"@trellis-sdk/krishi-cloud\""));
    assert!(deno.contains(&format!(
        "jsr:@qlever-llc/trellis@^{}",
        trellis_package_version()
    )));
    assert!(!deno.contains("npm:@qlever-llc/trellis"));
    assert!(!deno.contains("@qlever-llc/trellis-generated-krishi-cloud"));
}

#[test]
fn prepare_bootstraps_repo_without_discover_summary() {
    let temp = tempfile::tempdir().unwrap();
    let services = temp.path().join("services/orders");
    let apps = temp.path().join("apps/dashboard");
    fs::create_dir_all(services.join("contracts")).unwrap();
    fs::create_dir_all(apps.join("contracts")).unwrap();
    fs::write(
        services.join("deno.json"),
        "{\n  \"version\": \"0.4.0\"\n}\n",
    )
    .unwrap();
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .unwrap();
    fs::write(
        apps.join("deno.json"),
        format!(
            "{{\n  \"version\": \"0.4.0\",\n  \"imports\": {{\n    \"@trellis-sdk/trellis-orders\": \"../../generated/packages/jsr/orders/mod.ts\",\n    \"@qlever-llc/trellis\": \"file://{}/ts/packages/trellis/index.ts\",\n    \"@qlever-llc/trellis/\": \"file://{}/ts/packages/trellis/\"\n  }}\n}}\n",
            repo_root.display(),
            repo_root.display()
        ),
    )
    .unwrap();
    write_orders_contract(&services.join("contracts/orders.ts"), "Fixture contract");
    fs::write(
        apps.join("contracts/dashboard.ts"),
        r#"import * as Orders from "@trellis-sdk/trellis-orders";
import { defineAppContract } from "@qlever-llc/trellis";
void Orders;
export default defineAppContract(() => ({
  id: "trellis.dashboard-app@v1",
  apiId: "trellis.dashboard@v1",
  displayName: "Dashboard",
  description: "Fixture contract",
}));
"#,
    )
    .unwrap();

    let output = run_prepare_timings(temp.path());
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(!stdout.contains("Plan"));
    assert!(temp
        .path()
        .join("generated/protocol/apis/trellis.orders@v1.json")
        .exists());
    assert!(temp
        .path()
        .join("generated/protocol/apis/trellis.dashboard@v1.json")
        .exists());
    assert!(temp
        .path()
        .join("generated/protocol/participants/trellis.dashboard-app@v1.json")
        .exists());
    assert!(!temp
        .path()
        .join("generated/protocol/participants/trellis.dashboard@v1.json")
        .exists());
    assert!(temp
        .path()
        .join("generated/packages/jsr/dashboard/descriptors.ts")
        .exists());
    let descriptors = fs::read_to_string(
        temp.path()
            .join("generated/packages/jsr/dashboard/descriptors.ts"),
    )
    .unwrap();
    assert!(!descriptors.contains("Orders.Get"));
    assert!(!descriptors.contains("../orders"));
    assert!(!temp
        .path()
        .join("generated/packages/jsr/dashboard/client.ts")
        .exists());
    assert!(!temp
        .path()
        .join("generated/packages/cargo/dashboard/Cargo.toml")
        .exists());

    let warm = run_prepare_timings(temp.path());
    assert!(
        warm.status.success(),
        "{}",
        String::from_utf8_lossy(&warm.stderr)
    );
    let stdout = String::from_utf8_lossy(&warm.stdout);
    assert!(stdout.contains("hits                     2"), "{stdout}");
    assert!(stdout.contains("misses                   0"), "{stdout}");
    assert!(!stdout.contains("resolve TypeScript"), "{stdout}");

    let orders = services.join("contracts/orders.ts");
    fs::write(
        &orders,
        format!(
            "{}\n// semantic no-op\n",
            fs::read_to_string(&orders).unwrap()
        ),
    )
    .unwrap();
    let changed = run_prepare_timings(temp.path());
    assert!(
        changed.status.success(),
        "{}",
        String::from_utf8_lossy(&changed.stderr)
    );
    let stdout = String::from_utf8_lossy(&changed.stdout);
    assert!(stdout.contains("hits                     2"), "{stdout}");
    assert!(stdout.contains("misses                   1"), "{stdout}");
    let resolutions = stdout.split("contract resolution").nth(1).unwrap();
    assert!(resolutions.contains("orders.ts"), "{stdout}");
    assert!(!resolutions.contains("dashboard.ts"), "{stdout}");

    write_orders_contract(&orders, "Changed fixture contract");
    let semantic_change = run_prepare_timings(temp.path());
    assert!(
        semantic_change.status.success(),
        "{}",
        String::from_utf8_lossy(&semantic_change.stderr)
    );
    let stdout = String::from_utf8_lossy(&semantic_change.stdout);
    assert!(stdout.contains("misses                   2"), "{stdout}");
    assert!(stdout.contains("dependency changed       1"), "{stdout}");
    let resolutions = stdout.split("contract resolution").nth(1).unwrap();
    assert!(resolutions.contains("orders.ts"), "{stdout}");
    assert!(resolutions.contains("dashboard.ts"), "{stdout}");
}

#[test]
fn prepare_warns_for_public_closed_intersect_schemas() {
    let temp = tempfile::tempdir().unwrap();
    let service = temp.path().join("services/orders");
    fs::create_dir_all(service.join("contracts")).unwrap();
    fs::write(
        service.join("deno.json"),
        "{\n  \"version\": \"0.4.0\"\n}\n",
    )
    .unwrap();
    fs::write(
        service.join("contracts/orders.ts"),
        r#"const API = {
  format: "trellis.api.v1",
  id: "trellis.orders@v1",
  displayName: "Orders",
  description: "Fixture contract",
  schemas: {
    Merged: {
      allOf: [
        {
          type: "object",
          properties: { id: { type: "string" } },
          additionalProperties: false,
        },
        {
          type: "object",
          properties: { status: { type: "string" } },
          additionalProperties: false,
        },
      ],
    },
  },
  rpc: {
    "Orders.Get": {
      version: "v1",
      input: { schema: "Merged" },
      output: { schema: "Merged" },
    },
  },
};
const PARTICIPANT = {
  format: "trellis.participant.v1",
  id: API.id,
  displayName: API.displayName,
  description: "Fixture service participant",
  kind: "service",
  implements: { self: { api: API.id, apiDigest: "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA" } },
};

export default { API, PARTICIPANT };
"#,
    )
    .unwrap();

    let output = run_prepare(temp.path());
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("wire schemas that accept objects must leave"));
}

#[test]
fn prepare_generates_rust_participant_facade_for_local_device_uses() {
    let temp = tempfile::tempdir().unwrap();
    let service = temp.path().join("service");
    let inventory = temp.path().join("inventory");
    let device = temp.path().join("device");
    fs::create_dir_all(service.join("contracts")).unwrap();
    fs::create_dir_all(inventory.join("contracts")).unwrap();
    fs::create_dir_all(device.join("contracts")).unwrap();
    write_rust_manifest(&service.join("Cargo.toml"), "0.4.0");
    write_rust_manifest(&inventory.join("Cargo.toml"), "0.4.0");
    write_rust_manifest(&device.join("Cargo.toml"), "0.4.0");
    fs::write(
        service.join("contracts/orders.json"),
        r#"{
  "format": "trellis.api.v1",
  "id": "trellis.orders@v1",
  "displayName": "Orders",
  "description": "Fixture service contract",
  "kind": "service",
  "schemas": {
    "Empty": { "type": "object", "properties": {}, "required": [] },
    "Order": { "type": "object", "properties": { "id": { "type": "string" } }, "required": ["id"] }
  },
  "rpc": {
    "Orders.Get": {
      "version": "v1",
      "subject": "rpc.v1.Orders.Get",
      "input": { "schema": "Empty" },
      "output": { "schema": "Order" }
    }
  }
}
"#,
    )
    .unwrap();
    fs::write(
        inventory.join("contracts/inventory.json"),
        r#"{
  "format": "trellis.api.v1",
  "id": "trellis.inventory@v1",
  "displayName": "Inventory",
  "description": "Fixture inventory contract",
  "kind": "service",
  "schemas": {
    "Empty": { "type": "object", "properties": {}, "required": [] },
    "Stock": { "type": "object", "properties": { "sku": { "type": "string" } }, "required": ["sku"] }
  },
  "rpc": {
    "Inventory.Get": {
      "version": "v1",
      "subject": "rpc.v1.Inventory.Get",
      "input": { "schema": "Empty" },
      "output": { "schema": "Stock" }
    }
  }
}
"#,
    )
    .unwrap();
    fs::write(
        device.join("contracts/device.json"),
        r#"{
  "format": "trellis.api.v1",
  "id": "trellis.device@v1",
  "displayName": "Device",
  "description": "Fixture device contract",
  "kind": "device",
  "uses": {
    "required": {
      "orders": {
        "contract": "trellis.orders@v1",
        "rpc": { "call": ["Orders.Get"] }
      },
      "inventory": {
        "contract": "trellis.inventory@v1",
        "rpc": { "call": ["Inventory.Get"] }
      }
    }
  }
}
"#,
    )
    .unwrap();
    write_rust_contract(
        &service.join("contracts/orders.rs"),
        "orders.json",
        "trellis.orders-service@v1",
        "Service",
    );
    write_rust_contract(
        &inventory.join("contracts/inventory.rs"),
        "inventory.json",
        "trellis.inventory-service@v1",
        "Service",
    );
    write_rust_contract(
        &device.join("contracts/device.rs"),
        "device.json",
        "trellis.device@v1",
        "Device",
    );
    let legacy_participant = temp
        .path()
        .join("generated/protocol/participants/trellis.orders@v1.json");
    fs::create_dir_all(legacy_participant.parent().unwrap()).unwrap();
    fs::write(
        &legacy_participant,
        r#"{
  "format": "trellis.participant.v1",
  "id": "trellis.orders@v1",
  "implements": {
    "self": {
      "api": "trellis.orders@v1",
      "apiDigest": "legacy"
    }
  }
}"#,
    )
    .unwrap();
    let legacy_facade = temp
        .path()
        .join("generated/packages/cargo-participants/orders");
    fs::create_dir_all(legacy_facade.join("src")).unwrap();
    fs::write(
        legacy_facade.join("src/contract.rs"),
        "pub const CONTRACT_ID: &str = \"trellis.orders@v1\";\n",
    )
    .unwrap();

    let output = run_prepare(temp.path());
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(temp
        .path()
        .join("generated/packages/cargo/orders/Cargo.toml")
        .exists());
    let service_participant = temp
        .path()
        .join("generated/packages/cargo-participants/orders-service");
    assert!(
        fs::read_to_string(service_participant.join("src/contract.rs"))
            .unwrap()
            .contains("pub const CONTRACT_ID: &str = \"trellis.orders-service@v1\";")
    );
    assert!(fs::read_to_string(
        temp.path()
            .join("generated/packages/cargo/orders/src/api.rs")
    )
    .unwrap()
    .contains("pub const API_ID: &str = \"trellis.orders@v1\";"));
    assert!(temp
        .path()
        .join("generated/protocol/participants/trellis.orders-service@v1.json")
        .exists());
    assert!(!legacy_participant.exists());
    assert!(!legacy_facade.exists());
    write_rust_contract(
        &service.join("contracts/orders.rs"),
        "orders.json",
        "trellis.orders-renamed@v1",
        "Service",
    );
    let output = run_prepare(temp.path());
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!service_participant.exists());
    assert!(!temp
        .path()
        .join("generated/protocol/participants/trellis.orders-service@v1.json")
        .exists());
    assert!(temp
        .path()
        .join("generated/packages/cargo-participants/orders-renamed")
        .exists());
    assert!(temp
        .path()
        .join("generated/protocol/participants/trellis.orders-renamed@v1.json")
        .exists());
    let participant = temp
        .path()
        .join("generated/packages/cargo-participants/device");
    assert!(participant.join("Cargo.toml").exists());
    assert!(participant.join("src/lib.rs").exists());
    assert_eq!(fs::read_dir(participant.join("apis")).unwrap().count(), 0);
    assert!(temp
        .path()
        .join("generated/packages/cargo/device/Cargo.toml")
        .exists());

    let cargo_toml = fs::read_to_string(participant.join("Cargo.toml")).unwrap();
    assert!(cargo_toml.contains("name = \"trellis-participant-device\""));
    assert!(!cargo_toml.contains("trellis-sdk-orders"));
    assert!(!cargo_toml.contains("trellis-sdk-inventory"));
    assert!(!cargo_toml.contains("trellis-sdk-auth"));

    assert!(!participant.join("build.rs").exists());
    assert!(!participant.join("src/uses/orders.rs").exists());
    assert!(!participant.join("src/uses/inventory.rs").exists());

    fs::write(
        device.join("contracts/device.json"),
        r#"{
  "format": "trellis.api.v1",
  "id": "trellis.device@v1",
  "displayName": "Device",
  "description": "Fixture device contract",
  "kind": "device",
  "uses": {
    "required": {
      "orders": {
        "contract": "trellis.orders-remote@v1",
        "rpc": { "call": ["Orders.Get"] }
      },
      "inventory": {
        "contract": "trellis.inventory@v1",
        "rpc": { "call": ["Inventory.Get"] }
      }
    }
  }
}
"#,
    )
    .unwrap();

    let output = run_prepare(temp.path());
    assert!(output.status.success());

    fs::write(
        device.join("contracts/device.json"),
        r#"{
  "format": "trellis.api.v1",
  "id": "trellis.device@v1",
  "displayName": "Device",
  "description": "Fixture device contract",
  "kind": "device",
  "uses": {
    "required": {
      "orders": {
        "contract": "trellis.orders-remote@v1",
        "rpc": { "call": ["Orders.Get"] }
      },
      "inventory": {
        "contract": "trellis.inventory-remote@v1",
        "rpc": { "call": ["Inventory.Get"] }
      }
    }
  }
}
"#,
    )
    .unwrap();

    let output = run_prepare(temp.path());
    assert!(output.status.success());
}

#[test]
fn prepare_generates_rust_participant_facade_without_uses() {
    let temp = tempfile::tempdir().unwrap();
    let device = temp.path().join("device");
    fs::create_dir_all(device.join("contracts")).unwrap();
    write_rust_manifest(&device.join("Cargo.toml"), "0.4.0");
    fs::write(
        device.join("contracts/device.json"),
        r#"{
  "format": "trellis.api.v1",
  "id": "trellis.device@v1",
  "displayName": "Device",
  "description": "Fixture device contract",
  "kind": "device"
}
"#,
    )
    .unwrap();
    write_rust_contract(
        &device.join("contracts/device.rs"),
        "device.json",
        "trellis.device@v1",
        "Device",
    );

    let output = run_prepare(temp.path());
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(temp
        .path()
        .join("generated/packages/cargo-participants/device/Cargo.toml")
        .exists());
    assert!(temp
        .path()
        .join("generated/packages/cargo/device/Cargo.toml")
        .exists());
}

#[test]
fn prepare_accepts_custom_output_root() {
    let temp = tempfile::tempdir().unwrap();
    let service = temp.path().join("service");
    let out = temp.path().join("artifacts");
    fs::create_dir_all(service.join("contracts")).unwrap();
    fs::write(
        service.join("deno.json"),
        "{\n  \"version\": \"0.4.0\"\n}\n",
    )
    .unwrap();
    write_ts_contract(
        &service.join("contracts/orders.ts"),
        "trellis.orders@v1",
        "Orders",
        "service",
    );

    let output = trellis_generate()
        .args([
            "prepare",
            service.to_str().unwrap(),
            "--out",
            out.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(out
        .join("generated/protocol/apis/trellis.orders@v1.json")
        .exists());
    assert!(out.join("generated/packages/jsr/orders/mod.ts").exists());
    assert!(out
        .join("generated/packages/npm/orders/package.json")
        .exists());
    assert!(out
        .join("generated/packages/cargo/orders/Cargo.toml")
        .exists());
    assert!(!service.join("generated").exists());
}

#[test]
fn prepare_ignores_sveltekit_lib_contract() {
    let temp = tempfile::tempdir().unwrap();
    let app = temp.path().join("ts/apps/console");
    fs::create_dir_all(app.join("src/lib")).unwrap();
    fs::write(
        temp.path().join("ts/deno.json"),
        "{\n  \"version\": \"0.4.0\",\n  \"workspace\": [\"./apps/console\"]\n}\n",
    )
    .unwrap();
    fs::write(
        app.join("package.json"),
        "{\n  \"name\": \"console\",\n  \"version\": \"0.4.0\",\n  \"type\": \"module\"\n}\n",
    )
    .unwrap();
    write_ts_contract(
        &app.join("src/lib/contract.ts"),
        "trellis.console@v1",
        "Console",
        "app",
    );

    let output = trellis_generate()
        .args(["prepare", temp.path().to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("No contracts found."));
    assert!(!temp
        .path()
        .join("generated/protocol/apis/trellis.console@v1.json")
        .exists());
    assert!(!temp
        .path()
        .join("ts/generated/packages/jsr/console")
        .exists());
    assert!(!temp
        .path()
        .join("generated/packages/cargo/console/Cargo.toml")
        .exists());
}

#[test]
fn prepare_writes_demo_typescript_sdks_inside_demos_ts_workspace() {
    let temp = tempfile::tempdir().unwrap();
    let demos_root = temp.path().join("demos");
    let ts_workspace = demos_root.join("ts");
    let service = ts_workspace.join("rpc/service");
    fs::create_dir_all(service.join("contracts")).unwrap();
    fs::write(
        ts_workspace.join("deno.json"),
        "{\n  \"version\": \"0.4.0\",\n  \"workspace\": [\"./rpc/service\"]\n}\n",
    )
    .unwrap();
    fs::write(
        service.join("deno.json"),
        "{\n  \"version\": \"0.4.0\"\n}\n",
    )
    .unwrap();
    write_ts_contract(
        &service.join("contracts/demo_rpc_service.ts"),
        "trellis.demo-rpc-service@v1",
        "Demo RPC Service",
        "service",
    );

    let output = trellis_generate()
        .args(["prepare", demos_root.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(demos_root
        .join("generated/protocol/apis/trellis.demo-rpc-service@v1.json")
        .exists());
    assert!(demos_root
        .join("ts/generated/packages/jsr/demo-rpc-service/mod.ts")
        .exists());
    assert!(demos_root
        .join("ts/generated/packages/jsr/demo-rpc-service/descriptors.ts")
        .exists());
    assert!(demos_root
        .join("generated/packages/cargo/demo-rpc-service/Cargo.toml")
        .exists());
}

#[test]
fn prepare_in_local_runtime_repo_keeps_typescript_package_specifiers() {
    let temp = tempfile::tempdir().unwrap();
    let repo = temp.path();
    let app = repo.join("apps/dashboard");
    fs::create_dir_all(app.join("contracts")).unwrap();
    fs::create_dir_all(repo.join("ts/packages/trellis")).unwrap();
    fs::create_dir_all(repo.join("rust")).unwrap();
    fs::write(repo.join("ts/deno.json"), "{}\n").unwrap();
    fs::write(
        repo.join("rust/Cargo.toml"),
        "[workspace]\nmembers = []\nresolver = \"2\"\n",
    )
    .unwrap();
    fs::write(app.join("deno.json"), "{\n  \"version\": \"0.4.0\"\n}\n").unwrap();
    fs::write(
        app.join("contracts/dashboard.ts"),
        r#"const API = {
  format: "trellis.api.v1",
  id: "trellis.dashboard@v1",
  displayName: "Dashboard",
  description: "Fixture contract",
  schemas: {
    Empty: {
      type: "object",
      properties: {},
      required: [],
    },
  },
  rpc: {
    "Dashboard.Ping": {
      version: "v1",
      input: { schema: "Empty" },
      output: { schema: "Empty" },
    },
  },
  operations: {},
  events: {},
};
const PARTICIPANT = {
  format: "trellis.participant.v1",
  id: API.id,
  displayName: API.displayName,
  description: "Fixture app participant",
  kind: "app",
  implements: { self: { api: API.id, apiDigest: "va2NvbdTudYLfxVBLgiwZXyEXCUvKB-ilGDWl_4yklQ" } },
};

export default { API, PARTICIPANT };
"#,
    )
    .unwrap();

    let output = trellis_generate()
        .args(["prepare", repo.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let sdk = repo.join("generated/packages/jsr/dashboard");
    let descriptors = fs::read_to_string(sdk.join("descriptors.ts")).unwrap();
    let api = fs::read_to_string(sdk.join("api.ts")).unwrap();
    let types = fs::read_to_string(sdk.join("types.ts")).unwrap();
    let deno = fs::read_to_string(sdk.join("deno.json")).unwrap();
    let combined = format!("{descriptors}\n{api}\n{types}\n{deno}");

    assert!(descriptors.contains("@qlever-llc/trellis/contracts"));
    assert!(api.contains("export const API_ID"));
    assert!(!sdk.join("scripts/build_npm.ts").exists());
    assert!(!deno.contains("build:npm"));
    assert!(!combined.contains("ts/packages/trellis"));
    assert!(!combined.contains("file:"));
}

#[test]
fn local_mode_generates_app_typescript_client_without_rust_sdk() {
    let temp = tempfile::tempdir().unwrap();
    let project = temp.path().join("app");
    fs::create_dir_all(project.join("contracts")).unwrap();
    fs::write(
        project.join("deno.json"),
        "{\n  \"version\": \"0.4.0\"\n}\n",
    )
    .unwrap();
    write_ts_contract(
        &project.join("contracts/dashboard.ts"),
        "trellis.dashboard@v1",
        "Dashboard",
        "app",
    );

    let output = trellis_generate().current_dir(&project).output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("generate trellis.dashboard@v1"));
    assert!(stdout.contains("generated contract artifacts for trellis.dashboard@v1"));
    assert!(project
        .join("generated/protocol/apis/trellis.dashboard@v1.json")
        .exists());
    assert!(project
        .join("generated/packages/jsr/dashboard/descriptors.ts")
        .exists());
    assert!(project
        .join("generated/packages/jsr/dashboard/mod.ts")
        .exists());
    assert!(project
        .join("generated/packages/npm/dashboard/package.json")
        .exists());
    assert!(!project.join("generated/packages/cargo/dashboard").exists());
}

#[test]
fn local_mode_generates_service_artifacts_from_nearest_project_root() {
    let temp = tempfile::tempdir().unwrap();
    let project = temp.path().join("service");
    fs::create_dir_all(project.join("contracts")).unwrap();
    fs::create_dir_all(project.join("src/nested")).unwrap();
    fs::write(
        project.join("deno.json"),
        "{\n  \"version\": \"0.4.0\"\n}\n",
    )
    .unwrap();
    write_ts_contract(
        &project.join("contracts/orders.ts"),
        "trellis.orders@v1",
        "Orders",
        "service",
    );

    let output = trellis_generate()
        .current_dir(project.join("src/nested"))
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(project
        .join("generated/protocol/apis/trellis.orders@v1.json")
        .exists());
    assert!(project
        .join("generated/packages/jsr/orders/mod.ts")
        .exists());
    assert!(project
        .join("generated/packages/npm/orders/package.json")
        .exists());
    assert!(project
        .join("generated/packages/cargo/orders/Cargo.toml")
        .exists());
    assert!(String::from_utf8(output.stdout)
        .unwrap()
        .contains("Trellis Generate"));
}

#[test]
fn local_mode_generates_service_artifacts_from_top_level_contract_ts() {
    let temp = tempfile::tempdir().unwrap();
    let project = temp.path().join("service");
    fs::create_dir_all(project.join("src/nested")).unwrap();
    fs::write(
        project.join("package.json"),
        "{\n  \"name\": \"service\",\n  \"version\": \"0.4.0\",\n  \"type\": \"module\"\n}\n",
    )
    .unwrap();
    write_ts_contract(
        &project.join("contract.ts"),
        "trellis.top-level-orders@v1",
        "Top Level Orders",
        "service",
    );

    let output = trellis_generate()
        .current_dir(project.join("src/nested"))
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(project
        .join("generated/protocol/apis/trellis.top-level-orders@v1.json")
        .exists());
    assert!(project
        .join("generated/packages/jsr/top-level-orders/mod.ts")
        .exists());
    assert!(project
        .join("generated/packages/cargo/top-level-orders/Cargo.toml")
        .exists());
}

#[test]
fn local_mode_generates_service_artifacts_from_node_project_contracts() {
    let temp = tempfile::tempdir().unwrap();
    let project = temp.path().join("node-service");
    let tsx_path = temp.path().join("fake-tsx.sh");
    let tsc_path = temp.path().join("fake-tsc.sh");
    let npm_out = project.join("generated/packages/npm/node-orders");
    let support = project.join("node_modules/contract-support");
    fs::create_dir_all(project.join("contracts")).unwrap();
    fs::create_dir_all(&support).unwrap();
    fs::write(
        project.join("package.json"),
        "{\n  \"name\": \"node-service\",\n  \"version\": \"0.4.0\",\n  \"type\": \"module\"\n}\n",
    )
    .unwrap();
    fs::write(
        support.join("package.json"),
        "{\n  \"name\": \"contract-support\",\n  \"type\": \"module\",\n  \"exports\": \"./index.js\"\n}\n",
    )
    .unwrap();
    fs::write(
        support.join("index.js"),
        "export const CONTRACT_ID = 'trellis.node-orders@v1';\nexport const CONTRACT_KIND = 'service';\n",
    )
    .unwrap();
    fs::write(
        project.join("contracts/orders.ts"),
        concat!(
            "import { CONTRACT_ID, CONTRACT_KIND } from 'contract-support';\n",
            "export const API = {\n",
            "  format: 'trellis.api.v1',\n",
            "  id: CONTRACT_ID,\n",
            "  displayName: 'Node Orders',\n",
            "  description: 'Orders from node project',\n",
            "};\n",
            "export const PARTICIPANT = {\n",
            "  format: 'trellis.participant.v1',\n",
            "  id: CONTRACT_ID,\n",
            "  displayName: 'Node Orders',\n",
            "  description: 'Node Orders service',\n",
            "  kind: CONTRACT_KIND,\n",
            "  implements: { self: { api: CONTRACT_ID, apiDigest: '7aZlI7NfeGJOqx4ypFKNCcsAf2CEL4PqJpY6IQfQLTQ' } },\n",
            "};\n",
            "export default { API, PARTICIPANT };\n",
        ),
    )
    .unwrap();

    write_executable(
        &tsx_path,
        "#!/bin/sh
printf '{\"api\":{\"format\":\"trellis.api.v1\",\"id\":\"trellis.node-orders@v1\",\"displayName\":\"Node Orders\",\"description\":\"Orders from node project\"},\"participant\":{\"format\":\"trellis.participant.v1\",\"id\":\"trellis.node-orders@v1\",\"displayName\":\"Node Orders\",\"description\":\"Node Orders service\",\"kind\":\"service\",\"implements\":{\"self\":{\"api\":\"trellis.node-orders@v1\",\"apiDigest\":\"7aZlI7NfeGJOqx4ypFKNCcsAf2CEL4PqJpY6IQfQLTQ\"}}}}'
",
    );
    write_executable(
        &tsc_path,
        "#!/bin/sh
set -eu
if [ \"${1:-}\" = \"--version\" ]; then
  printf 'Version 0.0.0-test\\n'
  exit 0
fi
config=tsconfig.json
while [ \"$#\" -gt 0 ]; do
  if [ \"$1\" = \"-p\" ]; then
    shift
    config=\"$1\"
  fi
  shift || true
done
out=$(awk -F'\"' '/\"outDir\"/ { print $4; exit }' \"$config\")
mkdir -p \"$out\"
for f in *.ts; do
  base=$(basename \"$f\" .ts)
  cp \"$f\" \"$out/$base.js\"
  cp \"$f\" \"$out/$base.d.ts\"
done
",
    );

    let output = trellis_generate()
        .current_dir(&project)
        .env("TRELLIS_TSX_BIN", &tsx_path)
        .env("TRELLIS_TSC_BIN", &tsc_path)
        .env("TRELLIS_DENO_BIN", "disabled")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(project
        .join("generated/protocol/apis/trellis.node-orders@v1.json")
        .exists());
    assert!(project
        .join("generated/packages/jsr/node-orders/mod.ts")
        .exists());
    assert!(npm_out.join("package.json").exists());
    assert!(npm_out.join("README.md").exists());
    assert!(npm_out.join("esm/mod.js").exists());
    assert!(npm_out.join("esm/mod.d.ts").exists());
    assert!(project
        .join("generated/packages/cargo/node-orders/Cargo.toml")
        .exists());
}

#[test]
fn discover_mode_summarizes_generation_actions_for_service_and_app_contracts() {
    let temp = tempfile::tempdir().unwrap();
    let services = temp.path().join("services/orders");
    let apps = temp.path().join("apps/dashboard");
    fs::create_dir_all(services.join("contracts")).unwrap();
    fs::create_dir_all(apps.join("contracts")).unwrap();
    fs::write(
        services.join("deno.json"),
        "{\n  \"version\": \"0.4.0\"\n}\n",
    )
    .unwrap();
    fs::write(apps.join("deno.json"), "{\n  \"version\": \"0.4.0\"\n}\n").unwrap();
    write_ts_contract(
        &services.join("contracts/orders.ts"),
        "trellis.orders@v1",
        "Orders",
        "service",
    );
    write_ts_contract(
        &apps.join("contracts/dashboard.ts"),
        "trellis.dashboard@v1",
        "Dashboard",
        "app",
    );

    let output = trellis_generate()
        .args(["discover", temp.path().to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Plan"));
    assert!(stdout.contains("trellis.orders@v1"));
    assert!(stdout.contains("generate"));
    assert!(stdout.contains("trellis.dashboard@v1"));
    assert!(stdout.contains("generate"));
    assert!(temp
        .path()
        .join("generated/protocol/apis/trellis.orders@v1.json")
        .exists());
    assert!(!apps.join("generated").exists());
}

#[test]
fn discover_mode_supports_top_level_contract_js() {
    let temp = tempfile::tempdir().unwrap();
    let app = temp.path().join("apps/dashboard");
    fs::create_dir_all(&app).unwrap();
    fs::write(
        app.join("package.json"),
        "{\n  \"name\": \"dashboard\",\n  \"version\": \"0.4.0\",\n  \"type\": \"module\"\n}\n",
    )
    .unwrap();
    write_ts_contract(
        &app.join("contract.js"),
        "trellis.dashboard-js@v1",
        "Dashboard JS",
        "app",
    );

    let output = trellis_generate()
        .args(["discover", temp.path().to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Plan"));
    assert!(stdout.contains("trellis.dashboard-js@v1"));
    assert!(stdout.contains("generate"));
}

#[test]
fn prepare_mode_supports_top_level_contract_js() {
    let temp = tempfile::tempdir().unwrap();
    let service = temp.path().join("services/orders");
    fs::create_dir_all(&service).unwrap();
    fs::write(
        service.join("package.json"),
        "{\n  \"name\": \"orders\",\n  \"version\": \"0.4.0\",\n  \"type\": \"module\"\n}\n",
    )
    .unwrap();
    write_ts_contract(
        &service.join("contract.js"),
        "trellis.orders-js@v1",
        "Orders JS",
        "service",
    );

    let output = trellis_generate()
        .args(["prepare", temp.path().to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(temp
        .path()
        .join("generated/protocol/apis/trellis.orders-js@v1.json")
        .exists());
    assert!(temp
        .path()
        .join("generated/packages/jsr/orders-js/mod.ts")
        .exists());
    assert!(temp
        .path()
        .join("generated/packages/cargo/orders-js/Cargo.toml")
        .exists());
}

#[test]
fn local_mode_fails_for_duplicate_contract_layouts() {
    let temp = tempfile::tempdir().unwrap();
    let project = temp.path().join("service");
    fs::create_dir_all(project.join("contracts")).unwrap();
    fs::write(
        project.join("package.json"),
        "{\n  \"name\": \"service\",\n  \"version\": \"0.4.0\",\n  \"type\": \"module\"\n}\n",
    )
    .unwrap();
    write_ts_contract(
        &project.join("contracts/orders.ts"),
        "trellis.orders@v1",
        "Orders",
        "service",
    );
    write_ts_contract(
        &project.join("contract.ts"),
        "trellis.orders-top-level@v1",
        "Orders Top Level",
        "service",
    );

    let output = trellis_generate().current_dir(&project).output().unwrap();
    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("has both contracts/"));
    assert!(stderr.contains("choose one layout"));
}

#[test]
fn local_mode_generates_service_artifacts_from_rust_contract_sources() {
    let temp = tempfile::tempdir().unwrap();
    let project = temp.path().join("rust-service");
    fs::create_dir_all(project.join("contracts")).unwrap();
    fs::write(
        project.join("Cargo.toml"),
        "[package]\nname = \"rust-service\"\nversion = \"0.4.0\"\nedition = \"2021\"\n\n[dependencies]\n",
    )
    .unwrap();
    write_rust_contract(
        &project.join("contracts/service.rs"),
        "service.manifest.json",
        "trellis.rust-service-participant@v1",
        "Service",
    );
    fs::write(
        project.join("contracts/service.manifest.json"),
        concat!(
            "{\n",
            "  \"format\": \"trellis.api.v1\",\n",
            "  \"id\": \"trellis.rust-service@v1\",\n",
            "  \"displayName\": \"Rust Service\",\n",
            "  \"description\": \"Fixture contract\",\n",
            "  \"kind\": \"service\"\n",
            "}\n"
        ),
    )
    .unwrap();

    let output = trellis_generate().current_dir(&project).output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(project
        .join("generated/protocol/apis/trellis.rust-service@v1.json")
        .exists());
    assert!(project
        .join("generated/packages/jsr/rust-service/mod.ts")
        .exists());
    assert!(project
        .join("generated/packages/cargo/rust-service/Cargo.toml")
        .exists());
}

#[test]
fn local_mode_skips_when_generated_artifacts_are_up_to_date() {
    let temp = tempfile::tempdir().unwrap();
    let project = temp.path().join("service");
    fs::create_dir_all(project.join("contracts")).unwrap();
    fs::write(
        project.join("deno.json"),
        "{\n  \"version\": \"0.4.0\"\n}\n",
    )
    .unwrap();
    write_ts_contract(
        &project.join("contracts/orders.ts"),
        "trellis.orders@v1",
        "Orders",
        "service",
    );

    let first = trellis_generate().current_dir(&project).output().unwrap();
    assert!(
        first.status.success(),
        "{}",
        String::from_utf8_lossy(&first.stderr)
    );

    let metadata = project.join("generated/protocol/apis/trellis.orders@v1.trellis-generate.json");
    assert!(metadata.exists());

    let second = trellis_generate()
        .args(["prepare", "--timings"])
        .current_dir(&project)
        .output()
        .unwrap();
    assert!(
        second.status.success(),
        "{}",
        String::from_utf8_lossy(&second.stderr)
    );

    let stdout = String::from_utf8(second.stdout).unwrap();
    assert!(stdout.contains("artifacts already up to date for trellis.orders@v1"));
    assert!(!stdout.contains("generated contract artifacts for trellis.orders@v1"));
    assert!(stdout.contains("Timings"));
    assert!(stdout.contains("contracts               generated=0 verified=0 skipped=1"));
    assert!(stdout.contains("installed               files=0 bytes=0"));
}

#[test]
fn local_mode_force_regenerates_when_generated_artifacts_are_up_to_date() {
    let temp = tempfile::tempdir().unwrap();
    let project = temp.path().join("service");
    fs::create_dir_all(project.join("contracts")).unwrap();
    fs::write(
        project.join("deno.json"),
        "{\n  \"version\": \"0.4.0\"\n}\n",
    )
    .unwrap();
    write_ts_contract(
        &project.join("contracts/orders.ts"),
        "trellis.orders@v1",
        "Orders",
        "service",
    );

    let first = trellis_generate().current_dir(&project).output().unwrap();
    assert!(
        first.status.success(),
        "{}",
        String::from_utf8_lossy(&first.stderr)
    );

    let second = trellis_generate()
        .current_dir(&project)
        .arg("--force")
        .output()
        .unwrap();
    assert!(
        second.status.success(),
        "{}",
        String::from_utf8_lossy(&second.stderr)
    );

    let stdout = String::from_utf8(second.stdout).unwrap();
    assert!(stdout.contains("generated contract artifacts for trellis.orders@v1"));
    assert!(!stdout.contains("artifacts already up to date for trellis.orders@v1"));
}

#[test]
fn local_mode_regenerates_when_a_key_output_is_missing() {
    let temp = tempfile::tempdir().unwrap();
    let project = temp.path().join("service");
    fs::create_dir_all(project.join("contracts")).unwrap();
    fs::write(
        project.join("deno.json"),
        "{\n  \"version\": \"0.4.0\"\n}\n",
    )
    .unwrap();
    write_ts_contract(
        &project.join("contracts/orders.ts"),
        "trellis.orders@v1",
        "Orders",
        "service",
    );

    let first = trellis_generate().current_dir(&project).output().unwrap();
    assert!(
        first.status.success(),
        "{}",
        String::from_utf8_lossy(&first.stderr)
    );

    fs::remove_file(project.join("generated/packages/jsr/orders/descriptors.ts")).unwrap();

    let second = trellis_generate().current_dir(&project).output().unwrap();
    assert!(
        second.status.success(),
        "{}",
        String::from_utf8_lossy(&second.stderr)
    );

    let stdout = String::from_utf8(second.stdout).unwrap();
    assert!(stdout.contains("generated contract artifacts for trellis.orders@v1"));
}

#[test]
fn local_mode_regenerates_when_rust_sdk_cargo_toml_is_invalid() {
    let temp = tempfile::tempdir().unwrap();
    let project = temp.path().join("service");
    fs::create_dir_all(project.join("contracts")).unwrap();
    fs::write(
        project.join("deno.json"),
        "{\n  \"version\": \"0.4.0\"\n}\n",
    )
    .unwrap();
    write_ts_contract(
        &project.join("contracts/orders.ts"),
        "trellis.orders@v1",
        "Orders",
        "service",
    );

    let first = trellis_generate().current_dir(&project).output().unwrap();
    assert!(
        first.status.success(),
        "{}",
        String::from_utf8_lossy(&first.stderr)
    );

    let cargo_toml = project.join("generated/packages/cargo/orders/Cargo.toml");
    fs::write(
        &cargo_toml,
        concat!(
            "[package]\n",
            "name = \"trellis-sdk-orders\"\n",
            "version = \"0.4.0\"\n",
            "edition = \"2021\"\n\n",
            "[dependencies]\n",
            "trellis-client = \"0.4.0\"\n",
        ),
    )
    .unwrap();

    let second = trellis_generate().current_dir(&project).output().unwrap();
    assert!(
        second.status.success(),
        "{}",
        String::from_utf8_lossy(&second.stderr)
    );

    let stdout = String::from_utf8(second.stdout).unwrap();
    assert!(stdout.contains("generated contract artifacts for trellis.orders@v1"));
    assert!(!stdout.contains("artifacts already up to date for trellis.orders@v1"));

    let repaired = fs::read_to_string(&cargo_toml).unwrap();
    assert!(repaired.contains("serde = { version = \"1.0\""));
    assert!(repaired.contains("serde_json = \"1.0\""));
    assert!(repaired.contains("trellis-rs = \"0.4.0\""));
    assert!(repaired.contains("trellis-contracts = \"0.4.0\""));
    assert!(!repaired.contains("trellis-client"));
    assert!(!repaired.contains("trellis-service"));
}

#[test]
fn generate_all_skips_when_metadata_matches_outputs() {
    let temp = tempfile::tempdir().unwrap();
    let project = temp.path().join("service");
    let manifest_path = temp.path().join("trellis.orders@v1.json");
    let ts_out = temp.path().join("ts");
    let rust_out = temp.path().join("rust");
    fs::create_dir_all(project.join("contracts")).unwrap();
    fs::write(
        project.join("deno.json"),
        "{\n  \"version\": \"0.4.0\"\n}\n",
    )
    .unwrap();
    write_ts_contract(
        &project.join("contracts/orders.ts"),
        "trellis.orders@v1",
        "Orders",
        "service",
    );

    let first = trellis_generate()
        .args([
            "generate",
            "all",
            "--source",
            project.join("contracts/orders.ts").to_str().unwrap(),
            "--out-api",
            manifest_path.to_str().unwrap(),
            "--jsr-out",
            ts_out.to_str().unwrap(),
            "--cargo-out",
            rust_out.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        first.status.success(),
        "{}",
        String::from_utf8_lossy(&first.stderr)
    );

    let second = trellis_generate()
        .args([
            "generate",
            "all",
            "--source",
            project.join("contracts/orders.ts").to_str().unwrap(),
            "--out-api",
            manifest_path.to_str().unwrap(),
            "--jsr-out",
            ts_out.to_str().unwrap(),
            "--cargo-out",
            rust_out.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        second.status.success(),
        "{}",
        String::from_utf8_lossy(&second.stderr)
    );

    let stdout = String::from_utf8(second.stdout).unwrap();
    assert!(stdout.contains("artifacts already up to date for trellis.orders@v1"));
}
