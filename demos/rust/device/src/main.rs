use std::io::{self, Write};
use std::sync::Arc;
use std::time::Duration;

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine as _;
use clap::Parser;
use futures_util::StreamExt;
use trellis_participant_demo_device::contract as device_contract;
use trellis_participant_demo_device::state::{DraftInspectionState, SelectedSiteState};
use trellis_participant_demo_device::ConnectedClient;
use trellis_rs::{
    auth::{
        check_device_activation, derive_device_identity, wait_for_device_activation,
        DeviceActivationOptions, DeviceActivationStatus,
    },
    client::{
        download_transfer_grant_from_value, DeviceConnectOptions,
        MemoryAuthorizationContextStore,
    },
};
use trellis_sdk_demo_service::types::{
    AssignmentsListRequest, EvidenceDownloadRequest, EvidenceListRequest, EvidenceUploadInput,
    ReportsGenerateInput, SitesListRequest, SitesListResponseEntriesItem,
};

const DEMO_TIMESTAMP: &str = "2026-04-30T16:00:00.000Z";
const LIST_LIMIT: i64 = 50;
const LIST_OFFSET: i64 = 0;

#[derive(Debug, Parser)]
struct Args {
    /// Trellis HTTP URL for service bootstrap mode.
    #[arg(long, env = "TRELLIS_URL")]
    trellis_url: Option<String>,

    /// Use demo-local activated-device persistence and connect flow.
    #[arg(long, env = "TRELLIS_DEMO_DEVICE")]
    device: bool,

    /// Provisioned device deployment id.
    #[arg(long, env = "TRELLIS_DEVICE_DEPLOYMENT_ID")]
    device_deployment_id: Option<String>,

    /// Provisioned device instance id.
    #[arg(long, env = "TRELLIS_DEVICE_INSTANCE_ID")]
    device_instance_id: Option<String>,

    /// Base64url device root secret printed by `trellis deploy provision`.
    #[arg(long, env = "TRELLIS_DEVICE_ROOT_SECRET")]
    device_root_secret: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    println!("Rust Field Device Demo");
    println!("Activation helper: preregistered device bootstrap enabled.");
    println!("State helper: generated device/state facade enabled.");

    let client = connect_if_configured(&args).await?;
    if let Some(client) = client.as_ref() {
        spawn_event_watchers(client).await?;
    }

    wizard_loop(client.as_ref()).await
}

async fn connect_if_configured(args: &Args) -> anyhow::Result<Option<ConnectedClient>> {
    if args.device {
        return connect_device_if_configured(args).await;
    }

    println!("No activated-device credentials provided; running offline.");
    Ok(None)
}

async fn connect_device_if_configured(args: &Args) -> anyhow::Result<Option<ConnectedClient>> {
    let trellis_url = args
        .trellis_url
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("--device requires --trellis-url"))?;
    let deployment_id = args
        .device_deployment_id
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("--device requires --device-deployment-id"))?;
    let instance_id = args
        .device_instance_id
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("--device requires --device-instance-id"))?;
    let root_secret = URL_SAFE_NO_PAD.decode(
        args.device_root_secret
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("--device requires --device-root-secret"))?,
    )?;
    let identity = derive_device_identity(&root_secret)?;
    let session_seed = URL_SAFE_NO_PAD.encode(rand::random::<[u8; 32]>());
    let activation = DeviceActivationOptions::new(
        DeviceConnectOptions::new(
            trellis_url,
            deployment_id,
            instance_id,
            device_contract::CONTRACT_ID,
            device_contract::CONTRACT_DIGEST,
            device_contract::PARTICIPANT_NEEDS_DIGEST,
            device_contract::PARTICIPANT,
            device_contract::API_JSON,
            device_contract::API_DIGEST,
            device_contract::REFERENCED_API_ARTIFACTS,
            &identity.public_identity_key,
            &identity.identity_seed_base64url,
            &session_seed,
            10_000,
            Arc::new(MemoryAuthorizationContextStore::default()),
        ),
        &identity.activation_key_base64url,
    );
    let session = match check_device_activation(&activation).await? {
        DeviceActivationStatus::Ready(session) => session,
        DeviceActivationStatus::Pending(pending) => {
            println!("Activation URL: {}", pending.activation_url);
            println!("Confirmation code: {}", pending.confirmation_code);
            wait_for_device_activation(&activation, &pending, Duration::from_secs(300)).await?
        }
    };
    Ok(Some(
        ConnectedClient::connect_activated(activation, session).await?,
    ))
}

async fn spawn_event_watchers(client: &ConnectedClient) -> anyhow::Result<()> {
    let mut activity = client
        .trellis_demo_service_v1()
        .subscribe_audit_recorded()
        .await?;
    tokio::spawn(async move {
        while let Some(event) = activity.next().await {
            match event {
                Ok(event) => println!("event Audit.Recorded: {}", event.message),
                Err(error) => eprintln!("activity event error: {error}"),
            }
        }
    });

    let mut evidence = client
        .trellis_demo_service_v1()
        .subscribe_evidence_uploaded()
        .await?;
    tokio::spawn(async move {
        while let Some(event) = evidence.next().await {
            match event {
                Ok(event) => println!("event Evidence.Uploaded: {}", event.key),
                Err(error) => eprintln!("evidence event error: {error}"),
            }
        }
    });

    let mut reports = client
        .trellis_demo_service_v1()
        .subscribe_reports_published()
        .await?;
    tokio::spawn(async move {
        while let Some(event) = reports.next().await {
            match event {
                Ok(event) => println!("event Reports.Published: {}", event.report_id),
                Err(error) => eprintln!("reports event error: {error}"),
            }
        }
    });

    let mut sites = client
        .trellis_demo_service_v1()
        .subscribe_sites_refreshed()
        .await?;
    tokio::spawn(async move {
        while let Some(event) = sites.next().await {
            match event {
                Ok(event) => println!("event Sites.Refreshed: {}", event.site.site_name),
                Err(error) => eprintln!("sites event error: {error}"),
            }
        }
    });

    Ok(())
}

async fn wizard_loop(client: Option<&ConnectedClient>) -> anyhow::Result<()> {
    loop {
        println!();
        println!("1. List sites");
        println!("2. List assignments");
        println!("3. List evidence");
        println!("4. Download evidence");
        println!("5. Upload evidence");
        println!("6. Generate report");
        println!("7. Quit");
        let choice = prompt("Choose a step")?;
        match choice.as_str() {
            "1" => list_sites(client).await?,
            "2" => list_assignments(client).await?,
            "3" => list_evidence(client).await?,
            "4" => download_evidence(client).await?,
            "5" => upload_evidence(client).await?,
            "6" => generate_report(client).await?,
            "7" | "q" | "quit" => return Ok(()),
            _ => println!("Unknown step"),
        }
    }
}

async fn list_sites(client: Option<&ConnectedClient>) -> anyhow::Result<()> {
    let sites = if let Some(client) = client {
        client
            .trellis_demo_service_v1()
            .sites_list(&SitesListRequest {
                limit: LIST_LIMIT,
                offset: Some(LIST_OFFSET),
            })
            .await?
            .entries
    } else {
        offline_sites()
    };
    for site in &sites {
        println!(
            "{} - {} (open: {}, overdue: {}, status: {})",
            site.site_id,
            site.site_name,
            site.open_inspections,
            site.overdue_inspections,
            site.latest_status
        );
    }
    save_selected_site(client, &sites).await?;
    Ok(())
}

async fn save_selected_site(
    client: Option<&ConnectedClient>,
    sites: &[SitesListResponseEntriesItem],
) -> anyhow::Result<()> {
    if sites.is_empty() {
        return Ok(());
    }

    let site_id = prompt("Site id to select for device state (blank to skip)")?;
    if site_id.is_empty() {
        return Ok(());
    }

    let Some(site) = sites.iter().find(|site| site.site_id == site_id) else {
        println!("No listed site matched {site_id}; selected site state unchanged.");
        return Ok(());
    };

    let selected_site = SelectedSiteState {
        site_id: site.site_id.clone(),
        site_name: site.site_name.clone(),
        selected_at: DEMO_TIMESTAMP.to_string(),
    };

    let Some(client) = client else {
        println!(
            "Offline selected site preview, not persisted: {} ({})",
            selected_site.site_name, selected_site.site_id
        );
        return Ok(());
    };

    client
        .client()
        .state()
        .selected_site()
        .put(&selected_site)
        .await?;
    println!("Selected site state saved: {}", selected_site.site_id);

    Ok(())
}

async fn list_assignments(client: Option<&ConnectedClient>) -> anyhow::Result<()> {
    if let Some(client) = client {
        let response = client
            .trellis_demo_service_v1()
            .assignments_list(&AssignmentsListRequest {
                limit: LIST_LIMIT,
                offset: Some(LIST_OFFSET),
            })
            .await?;
        for assignment in response.entries {
            println!(
                "{} - {} / {} ({})",
                assignment.inspection_id,
                assignment.site_name,
                assignment.asset_name,
                assignment.priority
            );
        }
    } else {
        println!("insp-1001 - North Ridge Substation / Transformer A (high)");
    }
    Ok(())
}

async fn list_evidence(client: Option<&ConnectedClient>) -> anyhow::Result<()> {
    if let Some(client) = client {
        let response = client
            .trellis_demo_service_v1()
            .evidence_list(&EvidenceListRequest {
                limit: LIST_LIMIT,
                offset: Some(LIST_OFFSET),
                prefix: None,
            })
            .await?;
        for evidence in response.entries {
            println!("{} - {} bytes", evidence.key, evidence.size);
        }
    } else {
        println!("site-north/transformer-a/photo.txt - 42 bytes");
    }
    Ok(())
}

async fn download_evidence(client: Option<&ConnectedClient>) -> anyhow::Result<()> {
    let key = prompt("Evidence key")?;
    let Some(client) = client else {
        println!("Offline mode cannot download transfer bytes.");
        return Ok(());
    };

    let service = client.trellis_demo_service_v1();
    let response = service
        .evidence_download(&EvidenceDownloadRequest { key })
        .await?;
    let grant = download_transfer_grant_from_value(serde_json::to_value(response.transfer)?)?;
    let bytes = service.download_transfer(&grant).await?;
    println!("Downloaded {} bytes for {}", bytes.len(), grant.info.key);
    Ok(())
}

async fn upload_evidence(client: Option<&ConnectedClient>) -> anyhow::Result<()> {
    let key = prompt("Evidence key")?;
    let content = prompt("Text content")?;
    let input = EvidenceUploadInput {
        content_type: Some("text/plain".to_string()),
        evidence_type: "photo".to_string(),
        key: key.clone(),
        metadata: None,
    };

    let Some(client) = client else {
        println!(
            "Offline evidence upload preview, not persisted: {} ({} bytes)",
            key,
            content.len()
        );
        return Ok(());
    };

    let started = client
        .trellis_demo_service_v1()
        .evidence_upload()
        .input(&input)
        .transfer(content.as_bytes())
        .start()
        .await
        .map_err(|error| anyhow::anyhow!("evidence upload failed: {}", error.source()))?;

    let file = started.file_info();
    println!(
        "Uploaded {} ({} bytes, content type: {})",
        file.key,
        file.size,
        file.content_type.as_deref().unwrap_or("unknown")
    );

    let snapshot = started.operation_ref().wait().await?;
    if let Some(output) = snapshot.output {
        println!("Evidence upload operation completed: {:?}", output);
    } else {
        println!("Evidence upload operation completed.");
    }

    Ok(())
}

async fn generate_report(client: Option<&ConnectedClient>) -> anyhow::Result<()> {
    let inspection_id = prompt("Inspection id")?;
    let site_id = prompt("Site id for draft state")?;
    let checklist_name = prompt("Checklist name for draft state")?;
    let comment = prompt("Report comment")?;
    save_draft_inspection(
        client,
        DraftInspectionState {
            inspection_id: inspection_id.clone(),
            site_id,
            checklist_name,
            notes: comment.clone(),
            updated_at: DEMO_TIMESTAMP.to_string(),
        },
    )
    .await?;

    let Some(client) = client else {
        println!("Offline report draft captured for {inspection_id}: {comment}");
        return Ok(());
    };

    let operation = client
        .trellis_demo_service_v1()
        .reports_generate()
        .start(&ReportsGenerateInput {
            inspection_id,
            report_comment: comment,
        })
        .await?;
    let snapshot = operation.wait().await?;
    println!("Report operation completed: {:?}", snapshot.output);
    Ok(())
}

async fn save_draft_inspection(
    client: Option<&ConnectedClient>,
    draft: DraftInspectionState,
) -> anyhow::Result<()> {
    let Some(client) = client else {
        println!(
            "Offline draft inspection preview, not persisted: {} ({})",
            draft.inspection_id, draft.site_id
        );
        return Ok(());
    };

    client
        .client()
        .state()
        .draft_inspections()
        .put(&draft.inspection_id, &draft)
        .await?;
    println!("Draft inspection state saved: {}", draft.inspection_id);

    Ok(())
}

fn prompt(label: &str) -> anyhow::Result<String> {
    print!("{label}: ");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

fn offline_sites() -> Vec<SitesListResponseEntriesItem> {
    vec![SitesListResponseEntriesItem {
        site_id: "site-north".to_string(),
        site_name: "North Ridge Substation".to_string(),
        open_inspections: 2,
        overdue_inspections: 1,
        latest_status: "attention".to_string(),
        last_report_at: DEMO_TIMESTAMP.to_string(),
    }]
}
