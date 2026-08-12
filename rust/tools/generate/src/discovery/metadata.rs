use std::fs;

use miette::IntoDiagnostic;
use trellis_contracts::ContractKind;

use crate::contract_input;

use super::{DiscoveredContractSource, SourceLanguage};

pub fn discover_contract_metadata(
    contract: &DiscoveredContractSource,
) -> miette::Result<(String, ContractKind)> {
    match contract.language {
        SourceLanguage::Protocol => resolve_contract_metadata(&contract.source_path),
        SourceLanguage::TypeScript => discover_typescript_contract_metadata(&contract.source_path),
        SourceLanguage::Rust => discover_rust_contract_metadata(&contract.source_path),
    }
}

fn discover_typescript_contract_metadata(
    path: &std::path::Path,
) -> miette::Result<(String, ContractKind)> {
    let source = fs::read_to_string(path).into_diagnostic()?;
    match resolve_contract_metadata(path) {
        Ok((id, resolved_kind)) => Ok((
            id,
            discover_typescript_contract_kind(&source, path).unwrap_or(resolved_kind),
        )),
        Err(error) => Ok((
            extract_quoted_source_field(&source, "id").ok_or(error)?,
            discover_typescript_contract_kind(&source, path)?,
        )),
    }
}

fn discover_typescript_contract_kind(
    source: &str,
    path: &std::path::Path,
) -> miette::Result<ContractKind> {
    if let Some(kind) = infer_contract_kind_from_typescript_source(source) {
        return Ok(kind);
    }

    if let Some(kind) = extract_quoted_source_field(source, "kind") {
        return parse_contract_kind(&kind);
    }

    Err(miette::miette!(
        "failed to infer contract kind from {}",
        path.display()
    ))
}

fn discover_rust_contract_metadata(
    path: &std::path::Path,
) -> miette::Result<(String, ContractKind)> {
    let (id, resolved_kind) = resolve_contract_metadata(path)?;
    let source = fs::read_to_string(path).into_diagnostic()?;
    let kind = [
        ("ContractKind::Service", ContractKind::Service),
        ("ContractKind::App", ContractKind::App),
        ("ContractKind::Device", ContractKind::Device),
        ("ContractKind::Agent", ContractKind::Agent),
    ]
    .into_iter()
    .find_map(|(needle, kind)| source.contains(needle).then_some(kind))
    .unwrap_or(resolved_kind);
    Ok((id, kind))
}

fn resolve_contract_metadata(path: &std::path::Path) -> miette::Result<(String, ContractKind)> {
    let resolved = contract_input::resolve_contract_input(
        None,
        None,
        &[],
        Some(path),
        None,
        "API",
        contract_input::default_image_api_path(),
    )?;
    let participant_path = resolved.participant_path.ok_or_else(|| {
        miette::miette!("{} does not export a native participant", path.display())
    })?;
    let participant =
        trellis_contracts::load_participant_source(participant_path).into_diagnostic()?;
    Ok((resolved.api.render_model.id, participant.render_model.kind))
}

fn extract_quoted_source_field(source: &str, field: &str) -> Option<String> {
    let needle = format!("{field}:");
    let mut offset = 0;
    while let Some(found) = source[offset..].find(&needle) {
        let start = offset + found;
        let before = source[..start].chars().next_back();
        if before.is_some_and(|ch| ch == '_' || ch.is_ascii_alphanumeric()) {
            offset = start + needle.len();
            continue;
        }
        let after = source[start + needle.len()..].trim_start();
        let value = after.strip_prefix('"')?;
        let end = value.find('"')?;
        return Some(value[..end].to_string());
    }
    None
}

pub fn parse_contract_kind(value: &str) -> miette::Result<ContractKind> {
    match value {
        "service" => Ok(ContractKind::Service),
        "app" => Ok(ContractKind::App),
        "device" => Ok(ContractKind::Device),
        "agent" => Ok(ContractKind::Agent),
        _ => Err(miette::miette!("unsupported contract kind '{value}'")),
    }
}

fn infer_contract_kind_from_typescript_source(source: &str) -> Option<ContractKind> {
    const HELPER_KINDS: [(&str, ContractKind); 4] = [
        ("defineServiceContract(", ContractKind::Service),
        ("defineAppContract(", ContractKind::App),
        ("defineDeviceContract(", ContractKind::Device),
        ("defineAgentContract(", ContractKind::Agent),
    ];

    HELPER_KINDS
        .into_iter()
        .find_map(|(needle, kind)| source.contains(needle).then_some(kind))
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    #[test]
    fn discovers_typescript_metadata_via_deno_resolution() {
        let _env_lock = crate::contract_input::test_env_lock();
        let temp = TempDir::new().unwrap();
        let project = temp.path().join("node-service");
        let contracts = project.join("contracts");
        fs::create_dir_all(&contracts).unwrap();
        fs::write(
            project.join("package.json"),
            "{\n  \"name\": \"node-service\",\n  \"version\": \"0.4.0\",\n  \"type\": \"module\"\n}\n",
        )
        .unwrap();
        fs::write(
            contracts.join("orders.ts"),
            concat!(
                "const API_ID = ['trellis', 'node-orders@v1'].join('.');\n",
                "const API = {\n",
                "  format: 'trellis.api.v1',\n",
                "  id: API_ID,\n",
                "  displayName: 'Orders',\n",
                "  description: 'Orders',\n",
                "};\n",
                "const PARTICIPANT = {\n",
                "  format: 'trellis.participant.v1',\n",
                "  id: API_ID,\n",
                "  displayName: 'Orders',\n",
                "  description: 'Orders service',\n",
                "  kind: 'service',\n",
                "  implements: { self: { api: API_ID, apiDigest: 'AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA' } },\n",
                "};\n",
                "export default { API, PARTICIPANT };\n",
            ),
        )
        .unwrap();

        let discovered = DiscoveredContractSource {
            project_root: project.clone(),
            manifest_path: project.join("package.json"),
            source_path: contracts.join("orders.ts"),
            language: SourceLanguage::TypeScript,
        };

        let (id, kind) = discover_contract_metadata(&discovered).unwrap();
        assert_eq!(id, "trellis.node-orders@v1");
        assert_eq!(kind, ContractKind::Service);
    }

    #[test]
    fn falls_back_to_static_typescript_metadata_when_runtime_resolution_fails() {
        let _env_lock = crate::contract_input::test_env_lock();
        let temp = TempDir::new().unwrap();
        let project = temp.path().join("audit-app");
        let contracts = project.join("contracts");
        fs::create_dir_all(&contracts).unwrap();
        fs::write(
            project.join("package.json"),
            "{\n  \"name\": \"audit-app\",\n  \"version\": \"0.4.0\",\n  \"type\": \"module\"\n}\n",
        )
        .unwrap();
        fs::write(
            contracts.join("audit_app.ts"),
            concat!(
                "import { defineAppContract } from '@qlever-llc/trellis/contracts';\n",
                "import { auth } from '@qlever-llc/trellis/sdk/auth';\n",
                "export const auditApp = defineAppContract(() => ({\n",
                "  id: \"trellis.audit-app@v1\",\n",
                "  displayName: \"Audit App\",\n",
                "  description: \"Audit UI\",\n",
                "  uses: { auth },\n",
                "}));\n",
                "export default auditApp;\n",
            ),
        )
        .unwrap();

        let discovered = DiscoveredContractSource {
            project_root: project.clone(),
            manifest_path: project.join("package.json"),
            source_path: contracts.join("audit_app.ts"),
            language: SourceLanguage::TypeScript,
        };

        let (id, kind) = discover_contract_metadata(&discovered).unwrap();
        assert_eq!(id, "trellis.audit-app@v1");
        assert_eq!(kind, ContractKind::App);
    }

    #[test]
    fn helper_inference_wins_over_nested_state_kind_in_typescript_fallback() {
        let _env_lock = crate::contract_input::test_env_lock();
        let temp = TempDir::new().unwrap();
        let project = temp.path().join("inspection-app");
        let contracts = project.join("contracts");
        fs::create_dir_all(&contracts).unwrap();
        fs::write(
            project.join("package.json"),
            "{\n  \"name\": \"inspection-app\",\n  \"version\": \"0.4.0\",\n  \"type\": \"module\"\n}\n",
        )
        .unwrap();
        fs::write(
            contracts.join("inspection_app.ts"),
            concat!(
                "import { defineAppContract } from '@qlever-llc/trellis/contracts';\n",
                "export const inspectionApp = defineAppContract({ schemas: {} }, () => ({\n",
                "  id: \"trellis.inspection-app@v1\",\n",
                "  displayName: \"Inspection App\",\n",
                "  description: \"Inspection UI\",\n",
                "  state: {\n",
                "    inspectionContext: {\n",
                "      kind: \"map\",\n",
                "      schema: { schema: \"InspectionContext\" },\n",
                "    },\n",
                "  },\n",
                "}));\n",
                "export default inspectionApp;\n",
            ),
        )
        .unwrap();

        let discovered = DiscoveredContractSource {
            project_root: project.clone(),
            manifest_path: project.join("package.json"),
            source_path: contracts.join("inspection_app.ts"),
            language: SourceLanguage::TypeScript,
        };

        let (id, kind) = discover_contract_metadata(&discovered).unwrap();
        assert_eq!(id, "trellis.inspection-app@v1");
        assert_eq!(kind, ContractKind::App);
    }
}
