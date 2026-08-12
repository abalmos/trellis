use std::collections::BTreeMap;
use std::io::{self, Write};
use std::path::Path;
use std::path::PathBuf;

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine as _;
use miette::IntoDiagnostic;
use serde_json::{json, Value};
use trellis_rs::auth as authlib;
use trellis_rs::generated::Caller;

use crate::app::{connect_authenticated_cli_client, generate_session_keypair, json_value_label};
use crate::cli::*;
use crate::output;
use trellis_generate::contract_input;

const DEVICE_NAME_METADATA_KEY: &str = "name";
const DEVICE_SERIAL_METADATA_KEY: &str = "serialNumber";
const DEVICE_MODEL_METADATA_KEY: &str = "modelNumber";

pub(super) async fn run_svc(format: OutputFormat, command: SvcCommand) -> miette::Result<()> {
    match (command.id, command.command) {
        (None, SvcSubcommand::List(args)) => list_services(format, &args).await,
        (Some(id), SvcSubcommand::Resource(action)) => {
            run_svc_resource(format, SvcResourceCommand { id, action }).await
        }
        (Some(_), SvcSubcommand::List(_)) => Err(miette::miette!(
            "`list` is a top-level service command; use `trellis svc list`"
        )),
        (None, SvcSubcommand::Resource(_)) => Err(miette::miette!(
            "missing service deployment ID; use `trellis svc <ID> <COMMAND>`"
        )),
    }
}

pub(super) async fn run_dev(format: OutputFormat, command: DevCommand) -> miette::Result<()> {
    match (command.id, command.command) {
        (None, DevSubcommand::List(args)) => list_devices(format, &args).await,
        (Some(id), DevSubcommand::Resource(action)) => {
            run_dev_resource(format, DevResourceCommand { id, action }).await
        }
        (Some(_), DevSubcommand::List(_)) => Err(miette::miette!(
            "`list` is a top-level device command; use `trellis dev list`"
        )),
        (None, DevSubcommand::Resource(_)) => Err(miette::miette!(
            "missing device deployment ID; use `trellis dev <ID> <COMMAND>`"
        )),
    }
}

async fn run_svc_resource(format: OutputFormat, command: SvcResourceCommand) -> miette::Result<()> {
    match command.action {
        SvcResourceAction::Show => show_service(format, &command.id).await,
        SvcResourceAction::Create(args) => create_service(format, &command.id, &args).await,
        SvcResourceAction::Apply(args) => apply_contract(format, &command.id, &args).await,
        SvcResourceAction::Disable => toggle_service(format, &command.id, false).await,
        SvcResourceAction::Enable => toggle_service(format, &command.id, true).await,
        SvcResourceAction::Remove(args) => {
            remove_deployment(format, DeploymentKind::Service, &command.id, &args).await
        }
        SvcResourceAction::Instances(args) => service_instances(format, &command.id, &args).await,
        SvcResourceAction::Provision(args) => provision_service(format, &command.id, &args).await,
        SvcResourceAction::Authority(authority) => {
            deployment_authority(format, &command.id, authority).await
        }
    }
}

async fn run_dev_resource(format: OutputFormat, command: DevResourceCommand) -> miette::Result<()> {
    let id = command.id;
    match command.action {
        DevResourceAction::Show => show_device(format, &id).await,
        DevResourceAction::Create(args) => create_device(format, &id, &args).await,
        DevResourceAction::Apply(args) => apply_contract(format, &id, &args).await,
        DevResourceAction::Disable => toggle_device(format, &id, false).await,
        DevResourceAction::Enable => toggle_device(format, &id, true).await,
        DevResourceAction::Remove(args) => {
            remove_deployment(format, DeploymentKind::Device, &id, &args).await
        }
        DevResourceAction::Instances(args) => device_instances(format, &id, &args).await,
        DevResourceAction::Provision(args) => provision_device(format, &id, &args).await,
        DevResourceAction::Authority(command) => deployment_authority(format, &id, command).await,
        DevResourceAction::Activations(command) => dev_activations(format, &id, command).await,
        DevResourceAction::Reviews(command) => dev_reviews(format, &id, command).await,
    }
}

#[derive(Clone, Copy)]
enum DeploymentKind {
    Service,
    Device,
}

async fn list_services(format: OutputFormat, args: &SvcListArgs) -> miette::Result<()> {
    let (_state, connected) = connect_authenticated_cli_client(format).await?;
    let deployments = authlib::AuthClient::new(&connected)
        .rpc()
        .auth()
        .deployments_list(&trellis_rs::sdk::auth::types::AuthDeploymentsListRequest {
            kind: Some(trellis_rs::sdk::auth::types::AuthDeploymentsListRequestKind::Service),
            state: (!args.disabled)
                .then_some(trellis_rs::sdk::auth::types::AuthDeploymentsListRequestState::Active),
            cursor: None,
            limit: Some(100),
        })
        .await
        .into_diagnostic()?
        .entries;
    if output::is_json(format) {
        output::print_json(&json!({ "deployments": deployments }))?;
        return Ok(());
    }
    print_value_table(
        &serde_json::to_value(deployments).into_diagnostic()?,
        &["deploymentId", "state", "displayName"],
    )?;
    Ok(())
}

async fn list_devices(format: OutputFormat, args: &DevListArgs) -> miette::Result<()> {
    let (_state, connected) = connect_authenticated_cli_client(format).await?;
    let deployments = authlib::AuthClient::new(&connected)
        .rpc()
        .auth()
        .deployments_list(&trellis_rs::sdk::auth::types::AuthDeploymentsListRequest {
            kind: Some(trellis_rs::sdk::auth::types::AuthDeploymentsListRequestKind::Device),
            state: (!args.disabled)
                .then_some(trellis_rs::sdk::auth::types::AuthDeploymentsListRequestState::Active),
            cursor: None,
            limit: Some(100),
        })
        .await
        .into_diagnostic()?
        .entries;
    if output::is_json(format) {
        output::print_json(&json!({ "deployments": deployments }))?;
        return Ok(());
    }
    print_value_table(
        &serde_json::to_value(deployments).into_diagnostic()?,
        &["deploymentId", "state", "displayName"],
    )?;
    Ok(())
}

async fn show_service(format: OutputFormat, id: &str) -> miette::Result<()> {
    let (_state, connected) = connect_authenticated_cli_client(format).await?;
    let deployment = find_deployment(&connected, id, DeploymentKind::Service).await?;
    print_deployment_show_result(format, DeploymentKind::Service, &deployment)
}

async fn show_device(format: OutputFormat, id: &str) -> miette::Result<()> {
    let (_state, connected) = connect_authenticated_cli_client(format).await?;
    let deployment = find_deployment(&connected, id, DeploymentKind::Device).await?;
    print_deployment_show_result(format, DeploymentKind::Device, &deployment)
}

async fn create_service(
    format: OutputFormat,
    id: &str,
    _args: &SvcCreateArgs,
) -> miette::Result<()> {
    let (_state, connected) = connect_authenticated_cli_client(format).await?;
    let deployment = authlib::AuthClient::new(&connected)
        .rpc()
        .auth()
        .deployments_create(
            &trellis_rs::sdk::auth::types::AuthDeploymentsCreateRequest {
                kind: trellis_rs::sdk::auth::types::AuthDeploymentsCreateRequestKind::Service,
                display_name: id.to_owned(),
                participant_id: None,
                expires_at: None,
                requires_device_delegation: false,
                portal_id: None,
                idempotency_key: cli_idempotency_key(),
            },
        )
        .await
        .into_diagnostic()?
        .deployment;
    print_deployment_result(format, "service deployment created", &deployment)
}

async fn create_device(format: OutputFormat, id: &str, args: &DevCreateArgs) -> miette::Result<()> {
    let (_state, connected) = connect_authenticated_cli_client(format).await?;
    let deployment = authlib::AuthClient::new(&connected)
        .rpc()
        .auth()
        .deployments_create(
            &trellis_rs::sdk::auth::types::AuthDeploymentsCreateRequest {
                kind: trellis_rs::sdk::auth::types::AuthDeploymentsCreateRequestKind::Device,
                display_name: id.to_owned(),
                participant_id: None,
                expires_at: None,
                requires_device_delegation: args.review_mode.as_optional_wire_value().is_some(),
                portal_id: None,
                idempotency_key: cli_idempotency_key(),
            },
        )
        .await
        .into_diagnostic()?
        .deployment;
    print_deployment_result(format, "device deployment created", &deployment)
}

async fn apply_contract(
    format: OutputFormat,
    deployment_id: &str,
    args: &ApplyArgs,
) -> miette::Result<()> {
    let resolved = contract_input::resolve_contract_input(
        args.api.as_deref().map(Path::new),
        args.participant.as_deref().map(Path::new),
        &args
            .referenced_api
            .iter()
            .map(PathBuf::from)
            .collect::<Vec<_>>(),
        args.source.as_deref().map(Path::new),
        args.image.as_deref(),
        "CONTRACT",
        contract_input::default_image_api_path(),
    )?;
    let (_state, connected) = connect_authenticated_cli_client(format).await?;
    let participant = resolved.participant.ok_or_else(|| {
        miette::miette!("deployment apply requires a native Trellis participant artifact")
    })?;
    let Value::Object(participant_artifact) = participant.value else {
        return Err(miette::miette!("participant artifact must be an object"));
    };
    let Value::Object(api_artifact) = resolved.api.value else {
        return Err(miette::miette!("API artifact must be an object"));
    };
    let response = trellis_rs::sdk::auth::AuthClient::new(&connected)
        .rpc()
        .auth()
        .deployment_authority_plan(
            &trellis_rs::sdk::auth::types::AuthDeploymentAuthorityPlanRequest {
                deployment_id: deployment_id.to_string(),
                participant_artifact: participant_artifact.into_iter().collect(),
                referenced_api_artifacts: std::iter::once(api_artifact)
                    .chain(resolved.referenced_apis.into_iter().map(|api| {
                        api.value
                            .as_object()
                            .expect("validated API artifact is an object")
                            .clone()
                    }))
                    .map(|api| api.into_iter().collect())
                    .collect(),
                expires_at: None,
                idempotency_key: cli_idempotency_key(),
            },
        )
        .await
        .into_diagnostic()?;
    let response = serde_json::to_value(response).into_diagnostic()?;
    if output::is_json(format) {
        output::print_json(&response)?;
    } else {
        output::print_success("deployment authority plan created");
        output::print_info(&format!("deploymentId={deployment_id}"));
        output::print_info(&format!("participantDigest={}", participant.digest));
        if let Some(plan) = response.get("proposal") {
            if let Some(plan_id) = plan.get("proposalId").and_then(Value::as_str) {
                output::print_info(&format!("proposalId={plan_id}"));
            }
            if let Some(classification) = plan.get("classification").and_then(Value::as_str) {
                output::print_info(&format!("classification={classification}"));
            }
        }
    }
    Ok(())
}

async fn toggle_service(format: OutputFormat, id: &str, enable: bool) -> miette::Result<()> {
    toggle_deployment(format, id, enable, DeploymentKind::Service).await
}

async fn toggle_deployment(
    format: OutputFormat,
    id: &str,
    enable: bool,
    kind: DeploymentKind,
) -> miette::Result<()> {
    let (_state, connected) = connect_authenticated_cli_client(format).await?;
    let current = find_deployment(&connected, id, kind).await?;
    let expected_version = current
        .get("version")
        .and_then(Value::as_i64)
        .ok_or_else(|| miette::miette!("deployment response missing version"))?;
    let auth_client = authlib::AuthClient::new(&connected);
    let deployment = if enable {
        serde_json::to_value(
            auth_client
                .rpc()
                .auth()
                .deployments_enable(
                    &trellis_rs::sdk::auth::types::AuthDeploymentsEnableRequest {
                        deployment_id: id.to_owned(),
                        expected_version,
                        reason: None,
                        idempotency_key: cli_idempotency_key(),
                    },
                )
                .await
                .into_diagnostic()?
                .deployment,
        )
        .into_diagnostic()?
    } else {
        serde_json::to_value(
            auth_client
                .rpc()
                .auth()
                .deployments_disable(
                    &trellis_rs::sdk::auth::types::AuthDeploymentsDisableRequest {
                        deployment_id: id.to_owned(),
                        expected_version,
                        reason: None,
                        idempotency_key: cli_idempotency_key(),
                    },
                )
                .await
                .into_diagnostic()?
                .deployment,
        )
        .into_diagnostic()?
    };
    print_toggle_service_result(format, id, enable, &deployment)
}

async fn toggle_device(format: OutputFormat, id: &str, enable: bool) -> miette::Result<()> {
    toggle_deployment(format, id, enable, DeploymentKind::Device).await
}

async fn remove_deployment(
    format: OutputFormat,
    kind: DeploymentKind,
    id: &str,
    args: &RemoveArgs,
) -> miette::Result<()> {
    miette::ensure!(
        !output::is_json(format) || args.force,
        "use -f with --format json to skip the interactive removal review"
    );
    let label = ref_label(kind, id);
    if !output::is_json(format) && !args.force && !prompt_for_typed_identifier(&label)? {
        return Err(miette::miette!("deployment removal cancelled"));
    }
    let (_state, connected) = connect_authenticated_cli_client(format).await?;
    let current = find_deployment(&connected, id, kind).await?;
    let expected_version = current
        .get("version")
        .and_then(Value::as_i64)
        .ok_or_else(|| miette::miette!("deployment response missing version"))?;
    let response = authlib::AuthClient::new(&connected)
        .rpc()
        .auth()
        .deployments_remove(
            &trellis_rs::sdk::auth::types::AuthDeploymentsRemoveRequest {
                deployment_id: id.to_owned(),
                expected_version,
                reason: None,
                idempotency_key: cli_idempotency_key(),
            },
        )
        .await
        .into_diagnostic()?;
    print_remove_result(format, kind, id, serde_json::to_value(response).is_ok())
}

async fn service_instances(
    format: OutputFormat,
    id: &str,
    args: &SvcInstancesArgs,
) -> miette::Result<()> {
    let (_state, connected) = connect_authenticated_cli_client(format).await?;
    let instances = authlib::AuthClient::new(&connected)
        .rpc()
        .auth()
        .service_instances_list(
            &trellis_rs::sdk::auth::types::AuthServiceInstancesListRequest {
                deployment_id: Some(id.to_owned()),
                state: (!args.disabled).then_some(
                    trellis_rs::sdk::auth::types::AuthServiceInstancesListRequestState::Active,
                ),
                cursor: None,
                limit: Some(100),
            },
        )
        .await
        .into_diagnostic()?
        .entries;
    print_service_instances_result(format, instances)
}

async fn device_instances(
    format: OutputFormat,
    id: &str,
    args: &DevInstancesArgs,
) -> miette::Result<()> {
    let (_state, connected) = connect_authenticated_cli_client(format).await?;
    let instances = authlib::AuthClient::new(&connected)
        .rpc()
        .auth()
        .devices_list(&trellis_rs::sdk::auth::types::AuthDevicesListRequest {
            deployment_id: Some(id.to_owned()),
            state: args.state.map(|state| match state {
                DeviceInstanceState::Registered => {
                    trellis_rs::sdk::auth::types::AuthDevicesListRequestState::Pending
                }
                DeviceInstanceState::Activated => {
                    trellis_rs::sdk::auth::types::AuthDevicesListRequestState::Active
                }
                DeviceInstanceState::Disabled => {
                    trellis_rs::sdk::auth::types::AuthDevicesListRequestState::Disabled
                }
                DeviceInstanceState::Revoked => {
                    trellis_rs::sdk::auth::types::AuthDevicesListRequestState::Revoked
                }
            }),
            cursor: None,
            limit: Some(100),
        })
        .await
        .into_diagnostic()?
        .entries;
    print_device_instances_result(format, instances)
}

async fn provision_service(
    format: OutputFormat,
    id: &str,
    args: &SvcProvisionArgs,
) -> miette::Result<()> {
    let (_state, connected) = connect_authenticated_cli_client(format).await?;
    let (instance_seed, instance_key, generated_seed) = if let Some(seed) = &args.instance_seed {
        let session_key = authlib::session_public_key(seed).into_diagnostic()?;
        (seed.clone(), session_key, false)
    } else {
        let (seed, key) = generate_session_keypair();
        (seed, key, true)
    };
    let instance = authlib::AuthClient::new(&connected)
        .rpc()
        .auth()
        .service_instances_provision(
            &trellis_rs::sdk::auth::types::AuthServiceInstancesProvisionRequest {
                deployment_id: id.to_string(),
                instance_id: Some(format!("inst_{}", &instance_key[..16])),
                identity_public_key: instance_key,
                participant_id: None,
                idempotency_key: cli_idempotency_key(),
            },
        )
        .await
        .into_diagnostic()?
        .instance;
    print_service_provision_result(format, &instance, generated_seed, &instance_seed)
}

async fn provision_device(
    format: OutputFormat,
    id: &str,
    args: &DevProvisionArgs,
) -> miette::Result<()> {
    let (_state, connected) = connect_authenticated_cli_client(format).await?;
    let seed: [u8; 32] = rand::random();
    let root_secret = URL_SAFE_NO_PAD.encode(seed);
    let identity = authlib::derive_device_identity(&seed).into_diagnostic()?;
    let _metadata = build_device_metadata(args)?;
    let instance = authlib::AuthClient::new(&connected)
        .rpc()
        .auth()
        .devices_provision(&trellis_rs::sdk::auth::types::AuthDevicesProvisionRequest {
            deployment_id: id.to_owned(),
            instance_id: None,
            identity_public_key: Some(identity.public_identity_key),
            participant_id: None,
            idempotency_key: cli_idempotency_key(),
        })
        .await
        .into_diagnostic()?;
    print_device_provision_result(format, &instance, &root_secret)
}

async fn dev_activations(
    format: OutputFormat,
    deployment_id: &str,
    command: DevActivationsCommand,
) -> miette::Result<()> {
    match command {
        DevActivationsCommand::List(args) => {
            let (_state, connected) = connect_authenticated_cli_client(format).await?;
            let activations = authlib::AuthClient::new(&connected)
                .rpc()
                .auth()
                .devices_list(&trellis_rs::sdk::auth::types::AuthDevicesListRequest {
                    deployment_id: Some(deployment_id.to_owned()),
                    state: args.state.map(|state| match state {
                        DeviceActivationState::Activated => {
                            trellis_rs::sdk::auth::types::AuthDevicesListRequestState::Active
                        }
                        DeviceActivationState::Revoked => {
                            trellis_rs::sdk::auth::types::AuthDevicesListRequestState::Revoked
                        }
                    }),
                    cursor: None,
                    limit: Some(100),
                })
                .await
                .into_diagnostic()?
                .entries;
            let activations = activations
                .into_iter()
                .filter(|entry| {
                    args.instance
                        .as_deref()
                        .is_none_or(|id| entry.instance_id == id)
                })
                .collect::<Vec<_>>();
            print_device_activations_result(format, activations)
        }
        DevActivationsCommand::Revoke(args) => {
            let (_state, connected) = connect_authenticated_cli_client(format).await?;
            let devices = authlib::AuthClient::new(&connected)
                .rpc()
                .auth()
                .devices_list(&trellis_rs::sdk::auth::types::AuthDevicesListRequest {
                    deployment_id: Some(deployment_id.to_owned()),
                    state: None,
                    cursor: None,
                    limit: Some(100),
                })
                .await
                .into_diagnostic()?
                .entries;
            let device = devices
                .into_iter()
                .find(|device| device.instance_id == args.instance_id)
                .ok_or_else(|| miette::miette!("device not found: {}", args.instance_id))?;
            authlib::AuthClient::new(&connected)
                .rpc()
                .auth()
                .devices_disable(&trellis_rs::sdk::auth::types::AuthDevicesDisableRequest {
                    instance_id: args.instance_id.clone(),
                    expected_version: device.version,
                    reason: Some("device activation revoked by CLI".to_owned()),
                    idempotency_key: cli_idempotency_key(),
                })
                .await
                .into_diagnostic()?;
            let success = true;
            print_revoke_activation_result(format, &args.instance_id, success)
        }
    }
}

async fn dev_reviews(
    format: OutputFormat,
    deployment_id: &str,
    command: DevReviewsCommand,
) -> miette::Result<()> {
    let (_state, connected) = connect_authenticated_cli_client(format).await?;
    let auth_client = authlib::AuthClient::new(&connected);
    match command {
        DevReviewsCommand::List(args) => {
            let reviews = auth_client
                .rpc()
                .auth()
                .device_user_authorities_reviews_list(
                    &trellis_rs::sdk::auth::types::AuthDeviceUserAuthoritiesReviewsListRequest {
                        deployment_id: Some(deployment_id.to_owned()),
                        state: args.state.map(|state| match state {
                            DeviceReviewState::Pending => trellis_rs::sdk::auth::types::AuthDeviceUserAuthoritiesReviewsListRequestState::Pending,
                            DeviceReviewState::Approved => trellis_rs::sdk::auth::types::AuthDeviceUserAuthoritiesReviewsListRequestState::Approved,
                            DeviceReviewState::Rejected => trellis_rs::sdk::auth::types::AuthDeviceUserAuthoritiesReviewsListRequestState::Rejected,
                        }),
                        cursor: None,
                        limit: Some(100),
                    },
                )
                .await
                .into_diagnostic()?
                .entries;
            let reviews = reviews
                .into_iter()
                .filter(|review| {
                    args.instance
                        .as_deref()
                        .is_none_or(|id| review.instance_id == id)
                })
                .collect::<Vec<_>>();
            print_device_reviews_result(format, reviews)
        }
        DevReviewsCommand::Approve(args) => {
            review_decide(format, auth_client, &args, "approve").await
        }
        DevReviewsCommand::Reject(args) => {
            review_decide(format, auth_client, &args, "reject").await
        }
    }
}

async fn review_decide(
    format: OutputFormat,
    auth_client: authlib::AuthClient<'_>,
    args: &DevReviewDecisionArgs,
    decision: &str,
) -> miette::Result<()> {
    let response = auth_client
        .rpc()
        .auth()
        .device_user_authorities_reviews_list(
            &trellis_rs::sdk::auth::types::AuthDeviceUserAuthoritiesReviewsListRequest {
                deployment_id: None,
                state: None,
                cursor: None,
                limit: Some(100),
            },
        )
        .await
        .into_diagnostic()?
        .entries
        .into_iter()
        .find(|review| review.review_id == args.review_id)
        .ok_or_else(|| miette::miette!("device review not found: {}", args.review_id))?;
    let response = auth_client
        .rpc()
        .auth()
        .device_user_authorities_reviews_decide(
            &trellis_rs::sdk::auth::types::AuthDeviceUserAuthoritiesReviewsDecideRequest {
                review_id: args.review_id.clone(),
                decision: match decision {
                    "approve" => trellis_rs::sdk::auth::types::AuthDeviceUserAuthoritiesReviewsDecideRequestDecision::Approve,
                    _ => trellis_rs::sdk::auth::types::AuthDeviceUserAuthoritiesReviewsDecideRequestDecision::Reject,
                },
                expected_version: response.version,
                reason: args.reason.clone(),
                idempotency_key: cli_idempotency_key(),
            },
        )
        .await
        .into_diagnostic()?;
    if output::is_json(format) {
        output::print_json(&response)?;
    } else {
        let message = match decision {
            "approve" => "approved device review",
            "reject" => "rejected device review",
            _ => "updated device review",
        };
        output::print_success(message);
        output::print_info(&format!("reviewId={}", args.review_id));
    }
    Ok(())
}

async fn deployment_authority(
    format: OutputFormat,
    deployment_id: &str,
    command: DeploymentAuthorityCommand,
) -> miette::Result<()> {
    let (_state, connected) = connect_authenticated_cli_client(format).await?;
    let auth = trellis_rs::sdk::auth::AuthClient::new(&connected);
    match command {
        DeploymentAuthorityCommand::Show => {
            let authority_id = deployment_authority_id(&connected, deployment_id).await?;
            let response = auth
                .rpc()
                .auth()
                .deployment_authority_get(
                    &trellis_rs::sdk::auth::types::AuthDeploymentAuthorityGetRequest {
                        authority_id,
                    },
                )
                .await
                .into_diagnostic()?;
            let response = serde_json::to_value(response).into_diagnostic()?;
            print_deployment_authority_result(format, &response)
        }
        DeploymentAuthorityCommand::Plan(command) => {
            deployment_authority_plan(format, &connected, deployment_id, command).await
        }
        DeploymentAuthorityCommand::AcceptUpdate(args) => {
            let response = auth
                .rpc()
                .auth()
                .deployment_authority_accept_update(
                    &trellis_rs::sdk::auth::types::AuthDeploymentAuthorityAcceptUpdateRequest {
                        proposal_id: args.plan_id,
                        expected_base_authority_version: parse_optional_version(
                            args.expected_desired_version.as_deref(),
                        )?,
                        reason: None,
                        idempotency_key: cli_idempotency_key(),
                    },
                )
                .await
                .into_diagnostic()?;
            let response = serde_json::to_value(response).into_diagnostic()?;
            print_authority_decision_result(
                format,
                &response,
                "accepted desired authority update",
                true,
            )
        }
        DeploymentAuthorityCommand::AcceptMigration(args) => {
            let response = auth
                .rpc()
                .auth()
                .deployment_authority_accept_migration(
                    &trellis_rs::sdk::auth::types::AuthDeploymentAuthorityAcceptMigrationRequest {
                        proposal_id: args.plan_id,
                        expected_base_authority_version: parse_optional_version(
                            args.expected_desired_version.as_deref(),
                        )?,
                        reason: Some(args.acknowledgement),
                        idempotency_key: cli_idempotency_key(),
                    },
                )
                .await
                .into_diagnostic()?;
            let response = serde_json::to_value(response).into_diagnostic()?;
            print_authority_decision_result(
                format,
                &response,
                "accepted desired authority migration",
                true,
            )
        }
        DeploymentAuthorityCommand::Reject(args) => {
            let response = auth
                .rpc()
                .auth()
                .deployment_authority_reject(
                    &trellis_rs::sdk::auth::types::AuthDeploymentAuthorityRejectRequest {
                        proposal_id: args.plan_id,
                        reason: args.reason,
                        idempotency_key: cli_idempotency_key(),
                    },
                )
                .await
                .into_diagnostic()?;
            let response = serde_json::to_value(response).into_diagnostic()?;
            print_authority_decision_result(format, &response, "rejected authority plan", false)
        }
        DeploymentAuthorityCommand::Reconcile(args) => {
            let authority_id = deployment_authority_id(&connected, deployment_id).await?;
            let response = auth
                .rpc()
                .auth()
                .deployment_authority_reconcile(
                    &trellis_rs::sdk::auth::types::AuthDeploymentAuthorityReconcileRequest {
                        authority_id,
                        expected_version: parse_optional_version(args.desired_version.as_deref())?,
                        idempotency_key: cli_idempotency_key(),
                    },
                )
                .await
                .into_diagnostic()?;
            let response = serde_json::to_value(response).into_diagnostic()?;
            print_authority_decision_result(
                format,
                &response,
                "requested authority reconciliation",
                false,
            )
        }
    }
}

async fn deployment_authority_plan(
    format: OutputFormat,
    connected: &Caller,
    deployment_id: &str,
    command: AuthorityPlanCommand,
) -> miette::Result<()> {
    match command {
        AuthorityPlanCommand::List(args) => {
            if args.classification.is_some() {
                return Err(miette::miette!(
                    "classification filtering was removed; filter proposals by state"
                ));
            }
            let response = trellis_rs::sdk::auth::AuthClient::new(connected)
                .rpc()
                .auth()
                .deployment_authority_plans_list(&trellis_rs::sdk::auth::types::AuthDeploymentAuthorityPlansListRequest {
                    deployment_id: Some(deployment_id.to_string()),
                    limit: Some(100),
                    cursor: None,
                    state: args.state.map(|state| match state {
                        DeploymentAuthorityPlanState::Pending => trellis_rs::sdk::auth::types::AuthDeploymentAuthorityPlansListRequestState::Pending,
                        DeploymentAuthorityPlanState::Accepted => trellis_rs::sdk::auth::types::AuthDeploymentAuthorityPlansListRequestState::Accepted,
                        DeploymentAuthorityPlanState::Rejected => trellis_rs::sdk::auth::types::AuthDeploymentAuthorityPlansListRequestState::Rejected,
                        DeploymentAuthorityPlanState::Expired => trellis_rs::sdk::auth::types::AuthDeploymentAuthorityPlansListRequestState::Expired,
                    }),
                })
                .await
                .into_diagnostic()?;
            let response = serde_json::to_value(response).into_diagnostic()?;
            print_deployment_authority_plans_result(format, &response)
        }
        AuthorityPlanCommand::Show(args) => {
            let response = trellis_rs::sdk::auth::AuthClient::new(connected)
                .rpc()
                .auth()
                .deployment_authority_plans_get(
                    &trellis_rs::sdk::auth::types::AuthDeploymentAuthorityPlansGetRequest {
                        proposal_id: args.plan_id,
                    },
                )
                .await
                .into_diagnostic()?;
            let response = serde_json::to_value(response).into_diagnostic()?;
            print_deployment_authority_result(format, &response)
        }
    }
}

fn print_deployment_authority_result(format: OutputFormat, response: &Value) -> miette::Result<()> {
    if output::is_json(format) {
        output::print_json(&response)?;
    } else {
        output::print_json(response)?;
    }
    Ok(())
}

fn print_deployment_authority_plans_result(
    format: OutputFormat,
    response: &Value,
) -> miette::Result<()> {
    if output::is_json(format) {
        output::print_json(response)?;
    } else {
        let entries = response.get("entries").unwrap_or(&Value::Null);
        print_value_table(
            entries,
            &[
                "planId",
                "deploymentId",
                "classification",
                "state",
                "createdAt",
                "expiresAt",
            ],
        )?;
    }
    Ok(())
}

fn print_authority_decision_result(
    format: OutputFormat,
    response: &Value,
    message: &str,
    reconciliation_queued: bool,
) -> miette::Result<()> {
    if output::is_json(format) {
        output::print_json(response)?;
    } else {
        output::print_success(message);
        if let Some(authority) = response.get("authority") {
            if let Some(deployment_id) = authority.get("deploymentId").and_then(Value::as_str) {
                output::print_info(&format!("deploymentId={deployment_id}"));
            }
            if let Some(version) = authority_desired_version(response) {
                output::print_info(&format!("desiredVersion={version}"));
            }
        }
        if reconciliation_queued {
            output::print_info("reconciliation=triggered");
        }
    }
    Ok(())
}

fn authority_desired_version(response: &Value) -> Option<&str> {
    response
        .get("desiredVersion")
        .and_then(Value::as_str)
        .or_else(|| {
            response
                .get("authority")
                .and_then(|authority| authority.get("desiredVersion"))
                .and_then(Value::as_str)
        })
        .or_else(|| {
            response
                .get("authority")
                .and_then(|authority| authority.get("version"))
                .and_then(Value::as_str)
        })
}

fn print_deployment_show_result<T: serde::Serialize>(
    format: OutputFormat,
    kind: DeploymentKind,
    deployment: &T,
) -> miette::Result<()> {
    if output::is_json(format) {
        output::print_json(&json!({ "deployment": deployment }))?;
        return Ok(());
    }

    let value = serde_json::to_value(deployment).into_diagnostic()?;
    output::print_info(&format!(
        "ref={}",
        ref_label(kind, &value_string(&value, "deploymentId"))
    ));
    print_value_field(&value, "disabled");
    print_value_field(&value, "namespaces");
    print_value_field(&value, "reviewMode");
    Ok(())
}

fn print_toggle_service_result<T: serde::Serialize>(
    format: OutputFormat,
    id: &str,
    enable: bool,
    deployment: &T,
) -> miette::Result<()> {
    if output::is_json(format) {
        output::print_json(&json!({ "deployment": deployment }))?;
        return Ok(());
    }

    print_toggle_text(DeploymentKind::Service, id, enable);
    Ok(())
}

fn print_toggle_text(kind: DeploymentKind, id: &str, enable: bool) {
    let state = if enable { "enabled" } else { "disabled" };
    output::print_success(&format!("{state} deployment"));
    output::print_info(&format!("ref={}", ref_label(kind, id)));
}

fn print_remove_result(
    format: OutputFormat,
    kind: DeploymentKind,
    id: &str,
    success: bool,
) -> miette::Result<()> {
    if output::is_json(format) {
        output::print_json(&json!({ "success": success, "deploymentId": id }))?;
        return Ok(());
    }

    if success {
        output::print_success("removed deployment");
    } else {
        output::print_info("no matching deployment removed");
    }
    output::print_info(&format!("ref={}", ref_label(kind, id)));
    Ok(())
}

fn print_service_instances_result<T: serde::Serialize>(
    format: OutputFormat,
    instances: T,
) -> miette::Result<()> {
    if output::is_json(format) {
        output::print_json(&json!({ "instances": instances }))?;
        return Ok(());
    }

    print_value_table(
        &serde_json::to_value(instances).into_diagnostic()?,
        &["instanceId", "deploymentId", "disabled"],
    )
}

fn print_device_instances_result<T: serde::Serialize>(
    format: OutputFormat,
    instances: T,
) -> miette::Result<()> {
    if output::is_json(format) {
        output::print_json(&json!({ "instances": instances }))?;
        return Ok(());
    }

    print_value_table(
        &serde_json::to_value(instances).into_diagnostic()?,
        &[
            "instanceId",
            "deploymentId",
            "state",
            "publicIdentityKey",
            "name",
            "serialNumber",
            "modelNumber",
        ],
    )
}

fn print_service_provision_result<T: serde::Serialize>(
    format: OutputFormat,
    instance: &T,
    generated_seed: bool,
    instance_seed: &str,
) -> miette::Result<()> {
    if output::is_json(format) {
        output::print_json(
            &json!({ "instance": instance, "generatedSeed": generated_seed, "instanceSeed": generated_seed.then_some(instance_seed) }),
        )?;
        return Ok(());
    }

    output::print_success("provisioned service instance");
    let value = serde_json::to_value(instance).into_diagnostic()?;
    print_value_field(&value, "instanceId");
    print_value_field(&value, "deploymentId");
    print_value_field(&value, "instanceKey");
    if generated_seed {
        output::print_info(&format!("instanceSeed={instance_seed}"));
    }
    Ok(())
}

fn print_device_provision_result<T: serde::Serialize>(
    format: OutputFormat,
    instance: &T,
    root_secret: &str,
) -> miette::Result<()> {
    if output::is_json(format) {
        output::print_json(&json!({ "instance": instance, "rootSecret": root_secret }))?;
        return Ok(());
    }

    output::print_success("provisioned device instance");
    let value = serde_json::to_value(instance).into_diagnostic()?;
    print_value_field(&value, "instanceId");
    print_value_field(&value, "deploymentId");
    print_value_field(&value, "publicIdentityKey");
    output::print_info(&format!("rootSecret={root_secret}"));
    Ok(())
}

fn print_device_activations_result<T: serde::Serialize>(
    format: OutputFormat,
    activations: T,
) -> miette::Result<()> {
    if output::is_json(format) {
        output::print_json(&json!({ "activations": activations }))?;
        return Ok(());
    }

    print_value_table(
        &serde_json::to_value(activations).into_diagnostic()?,
        &[
            "instanceId",
            "deploymentId",
            "state",
            "activatedAt",
            "revokedAt",
        ],
    )
}

fn print_revoke_activation_result(
    format: OutputFormat,
    instance_id: &str,
    success: bool,
) -> miette::Result<()> {
    if output::is_json(format) {
        output::print_json(&json!({ "success": success, "instanceId": instance_id }))?;
        return Ok(());
    }

    if success {
        output::print_success("revoked device activation");
    } else {
        output::print_info("no matching activation revoked");
    }
    output::print_info(&format!("instanceId={instance_id}"));
    Ok(())
}

fn print_device_reviews_result<T: serde::Serialize>(
    format: OutputFormat,
    reviews: T,
) -> miette::Result<()> {
    if output::is_json(format) {
        output::print_json(&json!({ "reviews": reviews }))?;
        return Ok(());
    }

    print_value_table(
        &serde_json::to_value(reviews).into_diagnostic()?,
        &[
            "reviewId",
            "instanceId",
            "deploymentId",
            "state",
            "createdAt",
        ],
    )
}

fn print_value_table(value: &Value, columns: &[&str]) -> miette::Result<()> {
    let rows = value
        .as_array()
        .map(|items| {
            items
                .iter()
                .map(|item| {
                    columns
                        .iter()
                        .map(|column| value_string(item, column))
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    println!("{}", output::table(columns, rows));
    Ok(())
}

fn print_value_field(value: &Value, field: &str) {
    let rendered = value_string(value, field);
    if !rendered.is_empty() {
        output::print_info(&format!("{field}={rendered}"));
    }
}

fn value_string(value: &Value, field: &str) -> String {
    match value.get(field) {
        Some(Value::String(value)) => value.clone(),
        Some(Value::Bool(value)) => value.to_string(),
        Some(Value::Number(value)) => value.to_string(),
        Some(Value::Array(values)) => values
            .iter()
            .map(json_value_label)
            .collect::<Vec<_>>()
            .join(","),
        Some(Value::Object(_)) => value.get(field).map(json_value_label).unwrap_or_default(),
        Some(Value::Null) | None => String::new(),
    }
}

fn print_deployment_result<T: serde::Serialize>(
    format: OutputFormat,
    message: &str,
    deployment: &T,
) -> miette::Result<()> {
    if output::is_json(format) {
        output::print_json(&json!({ "deployment": deployment }))?;
    } else {
        output::print_success(message);
    }
    Ok(())
}

fn ref_label(kind: DeploymentKind, id: &str) -> String {
    let prefix = match kind {
        DeploymentKind::Service => "svc",
        DeploymentKind::Device => "dev",
    };
    format!("{prefix}/{id}")
}

fn prompt_for_typed_identifier(identifier: &str) -> miette::Result<bool> {
    print!("Type {identifier} to confirm: ");
    io::stdout().flush().into_diagnostic()?;
    let mut line = String::new();
    io::stdin().read_line(&mut line).into_diagnostic()?;
    Ok(line.trim() == identifier)
}

fn build_device_metadata(
    args: &DevProvisionArgs,
) -> miette::Result<Option<BTreeMap<String, String>>> {
    let mut metadata = BTreeMap::new();
    if let Some(name) = &args.name {
        metadata.insert(DEVICE_NAME_METADATA_KEY.to_string(), name.clone());
    }
    if let Some(serial_number) = &args.serial_number {
        metadata.insert(
            DEVICE_SERIAL_METADATA_KEY.to_string(),
            serial_number.clone(),
        );
    }
    if let Some(model_number) = &args.model_number {
        metadata.insert(DEVICE_MODEL_METADATA_KEY.to_string(), model_number.clone());
    }
    for entry in &args.metadata {
        let Some((key, value)) = entry.split_once('=') else {
            return Err(miette::miette!("metadata entries must use KEY=VALUE"));
        };
        if key.is_empty() {
            return Err(miette::miette!("metadata key must not be empty"));
        }
        metadata.insert(key.to_string(), value.to_string());
    }
    Ok((!metadata.is_empty()).then_some(metadata))
}

fn cli_idempotency_key() -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("cli_{nanos:x}")
}

fn parse_optional_version(value: Option<&str>) -> miette::Result<Option<i64>> {
    value.map(str::parse::<i64>).transpose().into_diagnostic()
}

async fn find_deployment(
    connected: &Caller,
    deployment_id: &str,
    kind: DeploymentKind,
) -> miette::Result<Value> {
    let kind = match kind {
        DeploymentKind::Service => {
            trellis_rs::sdk::auth::types::AuthDeploymentsListRequestKind::Service
        }
        DeploymentKind::Device => {
            trellis_rs::sdk::auth::types::AuthDeploymentsListRequestKind::Device
        }
    };
    let entries = trellis_rs::sdk::auth::AuthClient::new(connected)
        .rpc()
        .auth()
        .deployments_list(&trellis_rs::sdk::auth::types::AuthDeploymentsListRequest {
            kind: Some(kind),
            state: None,
            cursor: None,
            limit: Some(100),
        })
        .await
        .into_diagnostic()?
        .entries;
    entries
        .into_iter()
        .map(serde_json::to_value)
        .collect::<Result<Vec<_>, _>>()
        .into_diagnostic()?
        .into_iter()
        .find(|entry| entry.get("deploymentId").and_then(Value::as_str) == Some(deployment_id))
        .ok_or_else(|| miette::miette!("deployment not found: {deployment_id}"))
}

async fn deployment_authority_id(
    connected: &Caller,
    deployment_id: &str,
) -> miette::Result<String> {
    let entries = trellis_rs::sdk::auth::AuthClient::new(connected)
        .rpc()
        .auth()
        .deployment_authority_list(
            &trellis_rs::sdk::auth::types::AuthDeploymentAuthorityListRequest {
                deployment_id: Some(deployment_id.to_owned()),
                participant_id: None,
                state: None,
                cursor: None,
                limit: Some(100),
            },
        )
        .await
        .into_diagnostic()?
        .entries;
    let entries = serde_json::to_value(entries).into_diagnostic()?;
    entries
        .as_array()
        .and_then(|entries| entries.first())
        .and_then(|entry| entry.get("authorityId"))
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| miette::miette!("deployment has no authority: {deployment_id}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn authority_desired_version_prefers_explicit_response_value() {
        let response = json!({
            "desiredVersion": "desired-new",
            "authority": {
                "version": "authority-version",
                "desiredVersion": "authority-desired"
            }
        });

        assert_eq!(authority_desired_version(&response), Some("desired-new"));
    }

    #[test]
    fn authority_desired_version_falls_back_to_authority_version() {
        let response = json!({
            "authority": {
                "version": "authority-version"
            }
        });

        assert_eq!(
            authority_desired_version(&response),
            Some("authority-version")
        );
    }
}
