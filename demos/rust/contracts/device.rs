use serde_json::{json, Value};
use trellis_contracts::{
    state, ApiArtifact, ApiBuilder, ContractArtifacts, ContractBuilder, ContractKind,
    ContractStateKind, ContractsError,
};
use trellis_apis::state::api::API_JSON as STATE_API_JSON;

#[path = "service.rs"]
mod service;

/// Build the Rust-authored Field Device demo native API.
pub fn api_artifact() -> Result<ApiArtifact, ContractsError> {
    ApiBuilder::authoring(
        "trellis.demo-device@v1",
        "1.0.0",
        "Field Device Demo",
        "Activated Field Device TUI for the consolidated demo.",
    )
    .docs_with_summary(
        "Activated field device demo.",
        "Declares the Field Device demo's service usage and local state for selected sites and draft inspections.",
    )
    .schema(
        "SelectedSiteState",
        serde_json::json!({
            "type": "object",
            "required": ["siteId", "siteName", "selectedAt"],
            "properties": {
                "siteId": {"type": "string"},
                "siteName": {"type": "string"},
                "selectedAt": {"type": "string", "format": "date-time"}
            }
        }),
    )
    .schema(
        "DraftInspectionState",
        serde_json::json!({
            "type": "object",
            "required": ["inspectionId", "siteId", "checklistName", "notes", "updatedAt"],
            "properties": {
                "inspectionId": {"type": "string"},
                "siteId": {"type": "string"},
                "checklistName": {"type": "string"},
                "notes": {"type": "string"},
                "updatedAt": {"type": "string", "format": "date-time"}
            }
        }),
    )
    .state(
        "selectedSite",
        state(ContractStateKind::Value, "SelectedSiteState")
            .state_version("selected-site.v1")
            .docs_with_summary(
                "Selected site state.",
                "Stores the active site selected in the device TUI.",
            ),
    )
    .state(
        "draftInspections",
        state(ContractStateKind::Map, "DraftInspectionState")
            .state_version("draft-inspection.v1")
            .docs_with_summary(
                "Draft inspection state.",
                "Stores editable inspection draft notes keyed by inspection id.",
            ),
    )
    .build()
}

/// Build the device's native API and participant artifacts.
pub fn contract_artifacts() -> Result<ContractArtifacts, ContractsError> {
    let service_api = service::api_artifact()?.normalized_value()?;
    let state_api: Value = serde_json::from_str(STATE_API_JSON)?;
    let api = api_artifact()?;
    let api_value = api.normalized_value()?;
    let base = ContractBuilder::from_api(
        "trellis.demo-device@v1",
        api_value.clone(),
        ContractKind::Device,
    )?
    .build()?;
    let mut participant = base.participant_value()?;
    participant["uses"] = json!({
        "required": {
            "trellis.demo-service@v1": {
                "api": "trellis.demo-service@v1",
                "apiDigest": ApiBuilder::new(service_api.clone()).build()?.digest()?,
                "rpc": {"call": ["Assignments.List", "Evidence.Download", "Evidence.List", "Sites.Get", "Sites.List"]},
                "operations": {
                    "invoke": ["Evidence.Upload", "Reports.Generate", "Sites.Refresh"],
                    "observe": ["Evidence.Upload", "Reports.Generate", "Sites.Refresh"],
                    "cancel": ["Reports.Generate"]
                },
                "events": {"subscribe": ["Audit.Recorded", "Evidence.Uploaded", "Reports.Published", "Sites.Refreshed"]}
            },
            "trellis.state@v1": {
                "api": "trellis.state@v1",
                "apiDigest": ApiBuilder::new(state_api.clone()).build()?.digest()?,
                "rpc": {"call": ["State.Delete", "State.Get", "State.List", "State.Put"]}
            }
        }
    });
    ContractBuilder::from_native(api_value, participant)
        .referenced_api("trellis.demo-service@v1", service_api)
        .referenced_api("trellis.state@v1", state_api)
        .build()
}
