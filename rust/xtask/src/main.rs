use std::env;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use clap::error::ErrorKind;
use clap::{Parser, Subcommand};
use miette::{IntoDiagnostic, Result, WrapErr};

mod release;

#[derive(Debug, Clone, Eq, PartialEq, Subcommand)]
enum XtaskCommand {
    #[command(name = "prepare")]
    Prepare,
    #[command(name = "prepare-watch")]
    PrepareWatch,
    #[command(name = "protocol-wasm")]
    ProtocolWasm,
    #[command(name = "build", disable_help_flag = true)]
    Build {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(name = "release")]
    Release {
        #[command(subcommand)]
        command: release::ReleaseCommand,
    },
}

#[derive(Debug, Parser)]
#[command(name = "cargo xtask")]
struct XtaskCli {
    #[command(subcommand)]
    command: XtaskCommand,
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error:?}");
            ExitCode::from(1)
        }
    }
}

fn run() -> Result<()> {
    let Some(command) = parse_command(env::args().skip(1))? else {
        return Ok(());
    };
    match command {
        XtaskCommand::Prepare => run_prepare(),
        XtaskCommand::PrepareWatch => run_prepare_watch(),
        XtaskCommand::ProtocolWasm => generate_protocol_wasm(),
        XtaskCommand::Build { args } => run_build(&args),
        XtaskCommand::Release { command } => release::run_release(&repo_root()?, command),
    }
}

fn parse_command<I>(args: I) -> Result<Option<XtaskCommand>>
where
    I: Iterator<Item = String>,
{
    let input_args = args.collect::<Vec<_>>();
    let build_delimiter_index = match input_args.first() {
        Some(command) if command == "build" => {
            input_args.iter().skip(1).position(|arg| arg == "--")
        }
        _ => None,
    };
    let argv = std::iter::once("cargo-xtask".to_string()).chain(input_args.iter().cloned());
    let mut command = match XtaskCli::try_parse_from(argv) {
        Ok(cli) => cli.command,
        Err(error)
            if matches!(
                error.kind(),
                ErrorKind::DisplayHelp
                    | ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand
                    | ErrorKind::DisplayVersion
            ) =>
        {
            error.print().into_diagnostic()?;
            return Ok(None);
        }
        Err(error) => return Err(miette::miette!("{error}")),
    };
    if let Some(delimiter_index) = build_delimiter_index {
        if let XtaskCommand::Build { args } = &mut command {
            if args.get(delimiter_index).map(String::as_str) != Some("--") {
                args.insert(delimiter_index, "--".to_string());
            }
        }
    }
    if let XtaskCommand::Release {
        command: release_command,
    } = &command
    {
        release::validate_release_command(release_command)?;
    }
    Ok(Some(command))
}

fn run_prepare() -> Result<()> {
    run_generate_prepare(&["--no-npm"])?;
    generate_protocol_wasm()?;
    build_embedded_browser_apps()
}

fn build_embedded_browser_apps() -> Result<()> {
    let root = repo_root()?;
    let status = Command::new("deno")
        .current_dir(root)
        .args(["task", "-c", "ts/deno.json", "browser:embedded"])
        .status()
        .into_diagnostic()
        .wrap_err("failed to build embedded browser applications")?;

    if status.success() {
        Ok(())
    } else {
        Err(miette::miette!(
            "embedded browser application build failed with {status}"
        ))
    }
}

fn generate_protocol_wasm() -> Result<()> {
    let root = repo_root()?;
    let rust = root.join("rust");
    let target = env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .map(|path| {
            if path.is_absolute() {
                path
            } else {
                rust.join(path)
            }
        })
        .unwrap_or_else(|| rust.join("target"));
    let input = target.join("wasm32-unknown-unknown/release/trellis_protocol_wasm.wasm");
    let output = root.join("ts/packages/trellis/auth/protocol_wasm");
    std::fs::create_dir_all(&output).into_diagnostic()?;

    build_protocol_wasm(&rust, false)?;
    generate_wasm_bindings(&input, &output, "trellis_protocol_wasm")?;

    build_protocol_wasm(&rust, true)?;
    generate_wasm_bindings(&input, &output, "trellis_protocol_resolver_wasm")?;
    let _ = std::fs::remove_file(output.join("trellis_protocol_wasm_bytes.ts"));
    let _ = std::fs::remove_file(output.join("trellis_protocol_resolver_wasm_bytes.ts"));
    Ok(())
}

fn build_protocol_wasm(rust: &Path, resolver: bool) -> Result<()> {
    let mut command = Command::new("cargo");
    command.current_dir(rust).args([
        "build",
        "-p",
        "trellis-protocol-wasm",
        "--target",
        "wasm32-unknown-unknown",
        "--release",
    ]);
    if resolver {
        command.args(["--features", "resolver"]);
    }
    let status = command
        .env("CARGO_PROFILE_RELEASE_OPT_LEVEL", "z")
        .env("CARGO_PROFILE_RELEASE_LTO", "fat")
        .env("CARGO_PROFILE_RELEASE_CODEGEN_UNITS", "1")
        .env("CARGO_PROFILE_RELEASE_PANIC", "abort")
        .env("CARGO_PROFILE_RELEASE_STRIP", "symbols")
        .status()
        .into_diagnostic()
        .wrap_err("failed to build protocol WASM")?;
    if status.success() {
        Ok(())
    } else {
        Err(miette::miette!("protocol WASM build failed with {status}"))
    }
}

fn generate_wasm_bindings(input: &Path, output: &Path, name: &str) -> Result<()> {
    let mut bindgen = wasm_bindgen_cli_support::Bindgen::new();
    bindgen
        .input_path(input)
        .web(true)
        .map_err(|error| miette::miette!(error.to_string()))?
        .typescript(true)
        .omit_default_module_path(true)
        .out_name(name)
        .generate(output)
        .map_err(|error| miette::miette!(error.to_string()))?;
    Ok(())
}

fn run_prepare_watch() -> Result<()> {
    run_generate_prepare(&["--watch"])
}

fn run_generate_prepare(extra_args: &[&str]) -> Result<()> {
    let repo_root = repo_root()?;
    let mut args = vec![OsString::from("prepare")];
    args.extend(extra_args.iter().map(OsString::from));
    args.push(repo_root.into_os_string());
    let status = trellis_generate_runner::run_status(args)
        .into_diagnostic()
        .wrap_err("failed to run prepare workflow")?;

    if status.success() {
        Ok(())
    } else {
        Err(miette::miette!(
            "prepare workflow failed with status {status}"
        ))
    }
}

fn run_build(args: &[String]) -> Result<()> {
    run_prepare()?;
    let workspace_root = repo_root()?.join("rust");
    let mut spec = Command::new("cargo");
    spec.current_dir(&workspace_root).arg("build");
    for arg in args {
        spec.arg(arg);
    }
    let status = spec
        .status()
        .into_diagnostic()
        .wrap_err("failed to run cargo for build workflow")?;

    if status.success() {
        Ok(())
    } else {
        Err(miette::miette!(
            "build workflow failed with status {status}"
        ))
    }
}

fn repo_root() -> Result<PathBuf> {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    for ancestor in manifest_dir.ancestors() {
        if ancestor.join("rust/tools/generate/Cargo.toml").exists()
            && ancestor.join("ts/deno.json").exists()
        {
            return Ok(ancestor.to_path_buf());
        }
    }
    Err(miette::miette!(
        "failed to resolve repository root from xtask manifest"
    ))
}

#[cfg(test)]
mod tests {
    use crate::release::ReleaseCommand;

    use super::{parse_command, XtaskCommand};

    #[test]
    fn parse_prepare_command() {
        let command = parse_command(["prepare".to_string()].into_iter())
            .expect("parse prepare")
            .expect("prepare command");
        assert_eq!(command, XtaskCommand::Prepare);
    }

    #[test]
    fn parse_protocol_wasm_command() {
        let command = parse_command(["protocol-wasm".to_string()].into_iter())
            .expect("parse protocol-wasm")
            .expect("protocol-wasm command");
        assert_eq!(command, XtaskCommand::ProtocolWasm);
    }

    #[test]
    fn parse_prepare_watch_command() {
        let command = parse_command(["prepare-watch".to_string()].into_iter())
            .expect("parse prepare-watch")
            .expect("prepare-watch command");
        assert_eq!(command, XtaskCommand::PrepareWatch);
    }

    #[test]
    fn parse_build_command_preserves_passthrough_args() {
        let command = parse_command(
            ["build", "--workspace", "--release"]
                .into_iter()
                .map(str::to_string),
        )
        .expect("parse build")
        .expect("build command");
        assert_eq!(
            command,
            XtaskCommand::Build {
                args: vec!["--workspace".to_string(), "--release".to_string()],
            }
        );
    }

    #[test]
    fn parse_build_command_preserves_help_passthrough_arg() {
        let command = parse_command(["build", "--help"].into_iter().map(str::to_string))
            .expect("parse build help argument")
            .expect("build command");
        assert_eq!(
            command,
            XtaskCommand::Build {
                args: vec!["--help".to_string()],
            }
        );
    }

    #[test]
    fn parse_release_command() {
        let command = parse_command(
            ["release", "check-versions"]
                .into_iter()
                .map(str::to_string),
        )
        .expect("parse release command")
        .expect("release command");
        assert_eq!(
            command,
            XtaskCommand::Release {
                command: ReleaseCommand::CheckVersions,
            }
        );
    }

    #[test]
    fn parse_help_command_succeeds_without_exiting() {
        let command = parse_command(["--help"].into_iter().map(str::to_string))
            .expect("help should be handled successfully");
        assert!(command.is_none());
    }

    #[test]
    fn parse_build_command_preserves_argument_delimiter() {
        let command = parse_command(
            ["build", "--", "--cfg", "foo"]
                .into_iter()
                .map(str::to_string),
        )
        .expect("parse build")
        .expect("build command");
        assert_eq!(
            command,
            XtaskCommand::Build {
                args: vec!["--", "--cfg", "foo"]
                    .into_iter()
                    .map(str::to_string)
                    .collect(),
            }
        );
    }

    #[test]
    fn prepare_rejects_extra_args() {
        let error = parse_command(["prepare", "--workspace"].into_iter().map(str::to_string))
            .expect_err("prepare should reject extra args");
        assert!(error.to_string().contains("unexpected argument"));
    }

    #[test]
    fn protocol_wasm_rejects_extra_args() {
        let error = parse_command(
            ["protocol-wasm", "--workspace"]
                .into_iter()
                .map(str::to_string),
        )
        .expect_err("protocol-wasm should reject extra args");
        assert!(error.to_string().contains("unexpected argument"));
    }

    #[test]
    fn prepare_watch_rejects_extra_args() {
        let error = parse_command(
            ["prepare-watch", "--workspace"]
                .into_iter()
                .map(str::to_string),
        )
        .expect_err("prepare-watch should reject extra args");
        assert!(error.to_string().contains("unexpected argument"));
    }
}
