use std::fs;
use std::path::{Path, PathBuf};

use miette::{miette, IntoDiagnostic};
use trellis_local_nats::{ManagedNatsServer, NatsServerBinary};
use trellis_runtime::{NatsEndpointOverride, RuntimeOptions};

use crate::cli::*;

/// NATS client port of the `trellis init config` layout and the local render.
const NATS_PORT: u16 = 4222;
/// NATS HTTP monitoring port of the local render.
const NATS_HTTP_PORT: u16 = 8222;
/// NATS websocket port of the `trellis init config` layout and the local render.
const NATS_WS_PORT: u16 = 8080;

pub(super) async fn run(format: OutputFormat, args: ServerArgs) -> miette::Result<()> {
    let mut managed = None;
    if args.nats.is_some() {
        // Lifecycle lines go to stderr so `--format json` keeps stdout JSON-only.
        eprintln!(
            "using external NATS server at {}",
            args.nats.as_deref().expect("checked")
        );
    } else {
        if let Some(binary) = &args.nats_binary {
            eprintln!("using nats-server binary at {}", binary.display());
        }
        let server = prepare_managed_nats(&args)?;
        eprintln!(
            "started managed NATS server at {} (pid {})",
            server.url(),
            server.pid()
        );
        managed = Some(server);
    }

    let run_result = run_runtime(format, &args, &endpoint_override(&args)).await;
    if let Some(server) = managed.as_mut() {
        let stop_result = server.stop();
        if run_result.is_ok() {
            eprintln!("stopped managed NATS server");
            stop_result.into_diagnostic()?;
        } else if let Err(stop_error) = stop_result {
            eprintln!("failed to stop managed NATS server: {stop_error}");
        }
    }
    run_result
}

/// NATS endpoint replacement for `trellis server`: managed mode points the runtime and the
/// advertised client endpoints at the local server; `--nats URL` replaces the native
/// endpoint only and keeps the configured websocket (external deployments).
fn endpoint_override(args: &ServerArgs) -> NatsEndpointOverride {
    match &args.nats {
        Some(url) => NatsEndpointOverride {
            servers: url.clone(),
            websocket: None,
        },
        None => NatsEndpointOverride {
            servers: format!("nats://127.0.0.1:{NATS_PORT}"),
            websocket: Some(format!("ws://127.0.0.1:{NATS_WS_PORT}")),
        },
    }
}

async fn run_runtime(
    format: OutputFormat,
    args: &ServerArgs,
    nats_override: &NatsEndpointOverride,
) -> miette::Result<()> {
    if args.check {
        let report = trellis_runtime::check_with_nats_servers(
            args.mode,
            &args.config,
            Some(&nats_override.servers),
        )
        .await
        .into_diagnostic()?;
        let valid = report.valid;
        super::print_check_report(format, &report)?;
        if !valid {
            return Err(miette!("runtime preflight checks failed"));
        }
        return Ok(());
    }
    trellis_runtime::run(RuntimeOptions {
        mode: args.mode,
        config_path: args.config.clone(),
        rotate_first_admin: args.rotate_first_admin,
        nats_override: Some(nats_override.clone()),
    })
    .await
    .into_diagnostic()?;
    Ok(())
}

fn prepare_managed_nats(args: &ServerArgs) -> miette::Result<ManagedNatsServer> {
    let layout = resolve_managed_nats_layout(&args.config, args.nats_state_dir.as_deref())?;
    render_managed_nats_files(&layout)?;
    let binary = resolve_nats_binary(args)?;
    let server = ManagedNatsServer::start(
        &binary,
        &layout.local_config,
        NATS_PORT,
        NATS_HTTP_PORT,
        NATS_WS_PORT,
        &layout.pid_file,
    )
    .into_diagnostic()?;
    Ok(server)
}

/// Read-only managed-NATS source material plus the state directory holding every
/// generated/mutable file the managed server needs.
struct ManagedNatsLayout {
    /// Bundle nats dir (`<config_dir>/../nats`): nats.conf/jwt.conf/creds, read-only.
    source_dir: PathBuf,
    /// `nats.local.conf` (the effective server config, in the state dir).
    local_config: PathBuf,
    /// `jwt.local.conf` (jwt.conf with the resolver dir rewritten to the state dir).
    jwt_local_config: PathBuf,
    /// Pid file recording the managed nats-server process.
    pid_file: PathBuf,
    /// JetStream store dir and full-resolver dir under the state dir.
    store_dir: PathBuf,
    data_jwt_dir: PathBuf,
}

/// Resolve the managed-NATS layout: source material comes from the bundle's `nats/` dir
/// (read-only), and every generated/mutable file lives under the state dir, which defaults
/// to the bundle `nats/` dir to preserve host/dev behavior.
fn resolve_managed_nats_layout(
    config_path: &Path,
    state_dir: Option<&Path>,
) -> miette::Result<ManagedNatsLayout> {
    let source_dir = locate_nats_dir(config_path)?;
    let nats_conf = source_dir.join("nats.conf");
    let jwt_conf = source_dir.join("jwt.conf");
    if !nats_conf.is_file() || !jwt_conf.is_file() {
        return Err(miette!(
            "managed NATS requires {} and {}; run `trellis init config` first",
            nats_conf.display(),
            jwt_conf.display()
        ));
    }
    let source_dir = nats_conf
        .parent()
        .expect("nats.conf path has a parent")
        .canonicalize()
        .into_diagnostic()?;
    let state_dir = state_dir.map_or_else(
        || source_dir.clone(),
        |dir| fs::canonicalize(dir).unwrap_or_else(|_| dir.to_path_buf()),
    );
    Ok(ManagedNatsLayout {
        source_dir,
        local_config: state_dir.join("nats.local.conf"),
        jwt_local_config: state_dir.join("jwt.local.conf"),
        pid_file: state_dir.join("nats-server.pid"),
        store_dir: state_dir.join("data"),
        data_jwt_dir: state_dir.join("data/jwt"),
    })
}

/// Write the generated managed-NATS files into the state dir: `jwt.local.conf` (jwt.conf
/// with only the resolver `dir:` rewritten to the state path) and `nats.local.conf`
/// (loopback listeners, JetStream store and the jwt include both state-relative). The
/// source bundle files are only read, never written.
fn render_managed_nats_files(layout: &ManagedNatsLayout) -> miette::Result<()> {
    fs::create_dir_all(&layout.data_jwt_dir).into_diagnostic()?;

    let server_name = read_server_name(&layout.source_dir.join("nats.conf"))?;
    let rewritten_jwt = rewrite_jwt_dir(
        &fs::read_to_string(layout.source_dir.join("jwt.conf")).into_diagnostic()?,
        &layout.data_jwt_dir,
    );
    fs::write(&layout.jwt_local_config, rewritten_jwt).into_diagnostic()?;

    // nats-server resolves `include` paths relative to the including config file's
    // directory (and path-joins absolute paths onto it), so the include must be relative
    // to the state dir, where both generated configs live.
    let jwt_include = format!(
        "./{}",
        layout
            .jwt_local_config
            .file_name()
            .expect("jwt.local.conf has a file name")
            .to_string_lossy()
    );
    fs::write(
        &layout.local_config,
        trellis_bootstrap::render_local_nats_config(
            &server_name,
            &layout.store_dir.display().to_string(),
            &jwt_include,
        ),
    )
    .into_diagnostic()?;
    Ok(())
}

/// Resolve the managed nats-server binary: `--nats-binary` uses the given pre-installed
/// binary (validated, no download); otherwise the pinned release is downloaded and cached.
fn resolve_nats_binary(args: &ServerArgs) -> miette::Result<PathBuf> {
    match &args.nats_binary {
        Some(path) => NatsServerBinary::from_path(path).into_diagnostic(),
        None => NatsServerBinary::ensure(args.cache_dir.as_deref()).into_diagnostic(),
    }
}

fn locate_nats_dir(config_path: &Path) -> miette::Result<PathBuf> {
    let config_dir = config_path
        .parent()
        .ok_or_else(|| miette!("--config must include a parent directory"))?;
    Ok(config_dir.join("..").join("nats"))
}

fn read_server_name(nats_conf: &Path) -> miette::Result<String> {
    let contents = fs::read_to_string(nats_conf).into_diagnostic()?;
    Ok(contents
        .lines()
        .find_map(|line| {
            line.trim()
                .strip_prefix("server_name:")
                .map(str::trim)
                .map(str::to_string)
        })
        .unwrap_or_else(|| "trellis".to_string()))
}

/// Rewrite only the `dir:` line of the generated JWT resolver config.
fn rewrite_jwt_dir(jwt_conf: &str, data_jwt_dir: &Path) -> String {
    let dir = data_jwt_dir.display().to_string();
    jwt_conf
        .lines()
        .map(|line| {
            if line.trim_start().starts_with("dir:") {
                format!("dir: {dir}")
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn endpoint_override_managed_replaces_native_and_websocket() {
        let args = ServerArgs {
            mode: trellis_runtime::RuntimeMode::All,
            config: PathBuf::from("config.toml"),
            nats: None,
            rotate_first_admin: false,
            check: false,
            cache_dir: None,
            nats_binary: None,
            nats_state_dir: None,
        };
        let override_ = endpoint_override(&args);
        assert_eq!(override_.servers, "nats://127.0.0.1:4222");
        assert_eq!(override_.websocket.as_deref(), Some("ws://127.0.0.1:8080"));
    }

    #[test]
    fn endpoint_override_external_replaces_native_only() {
        let args = ServerArgs {
            mode: trellis_runtime::RuntimeMode::All,
            config: PathBuf::from("config.toml"),
            nats: Some("nats://external.example:4222".to_string()),
            rotate_first_admin: false,
            check: false,
            cache_dir: None,
            nats_binary: None,
            nats_state_dir: None,
        };
        let override_ = endpoint_override(&args);
        assert_eq!(override_.servers, "nats://external.example:4222");
        assert_eq!(override_.websocket, None);
    }

    #[test]
    fn rewrite_jwt_dir_replaces_only_the_dir_line() {
        let input = "operator: abc\nresolver: {\n  type: full\n  dir: /data/jwt\n}\n\nresolver_preload: {\n  A: jwt\n}\n";
        let rewritten = rewrite_jwt_dir(input, Path::new("/tmp/trellis/nats/data/jwt"));

        assert!(rewritten.contains("dir: /tmp/trellis/nats/data/jwt"));
        assert!(rewritten.contains("operator: abc"));
        assert!(rewritten.contains("type: full"));
        assert!(rewritten.contains("A: jwt"));
        assert!(!rewritten.contains("dir: /data/jwt"));
    }

    #[test]
    fn locate_nats_dir_resolves_config_sibling() {
        assert_eq!(
            locate_nats_dir(Path::new("trellis/config.toml")).expect("locate nats dir"),
            PathBuf::from("trellis/../nats")
        );
    }

    /// A minimal bundle: nats/nats.conf + nats/jwt.conf plus a marker file that must
    /// survive untouched when the state dir is separate.
    fn fixture_bundle(temp: &std::path::Path) -> PathBuf {
        let bundle = temp.join("bundle");
        fs::create_dir_all(bundle.join("nats")).expect("create nats dir");
        fs::create_dir_all(bundle.join("trellis")).expect("create trellis dir");
        fs::write(
            bundle.join("nats/nats.conf"),
            "server_name: nats-local\nlisten: 0.0.0.0:4222\n",
        )
        .expect("write nats.conf");
        fs::write(
            bundle.join("nats/jwt.conf"),
            "operator: abc\nresolver: {\n  type: full\n  dir: /data/jwt\n}\n",
        )
        .expect("write jwt.conf");
        fs::write(bundle.join("nats/marker"), b"source").expect("write source marker");
        bundle
    }

    #[test]
    fn managed_nats_files_default_to_the_source_nats_dir() {
        let temp = tempfile::tempdir().expect("temp dir");
        let bundle = fixture_bundle(temp.path());
        let layout = resolve_managed_nats_layout(&bundle.join("trellis/config.toml"), None)
            .expect("resolve layout");
        assert_eq!(
            layout.local_config,
            layout.source_dir.join("nats.local.conf")
        );
        assert_eq!(layout.pid_file, layout.source_dir.join("nats-server.pid"));

        render_managed_nats_files(&layout).expect("render files");
        assert!(layout.local_config.is_file());
        assert!(layout.jwt_local_config.is_file());
        assert!(layout.data_jwt_dir.is_dir());
    }

    #[test]
    fn managed_nats_state_dir_keeps_source_read_only_and_writes_state_only() {
        let temp = tempfile::tempdir().expect("temp dir");
        let bundle = fixture_bundle(temp.path());
        let source = bundle.join("nats");
        let source_entries_before = fs::read_dir(&source)
            .expect("read source dir")
            .filter_map(Result::ok)
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .collect::<Vec<_>>();
        let nats_conf_before = fs::read(source.join("nats.conf")).expect("read nats.conf before");
        let jwt_conf_before = fs::read(source.join("jwt.conf")).expect("read jwt.conf before");

        let state = temp.path().join("state");
        let layout = resolve_managed_nats_layout(&bundle.join("trellis/config.toml"), Some(&state))
            .expect("resolve layout with state dir");
        assert_ne!(
            layout.local_config,
            layout.source_dir.join("nats.local.conf")
        );
        assert_eq!(layout.pid_file, state.join("nats-server.pid"));
        assert_eq!(layout.store_dir, state.join("data"));
        assert_eq!(layout.data_jwt_dir, state.join("data/jwt"));

        render_managed_nats_files(&layout).expect("render files");

        // Every generated file lives under the state dir only.
        assert!(layout.local_config.is_file());
        assert!(layout.jwt_local_config.is_file());
        assert!(layout.data_jwt_dir.is_dir());
        let state_entries = fs::read_dir(&state)
            .expect("read state dir")
            .filter_map(Result::ok)
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .collect::<Vec<_>>();
        assert!(
            state_entries.contains(&"nats.local.conf".to_string()),
            "state dir must hold nats.local.conf: {state_entries:?}"
        );
        assert!(
            state_entries.contains(&"jwt.local.conf".to_string()),
            "state dir must hold jwt.local.conf: {state_entries:?}"
        );

        // The source bundle is untouched: same entries, same bytes.
        let source_entries_after = fs::read_dir(&source)
            .expect("read source dir")
            .filter_map(Result::ok)
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .collect::<Vec<_>>();
        assert_eq!(
            source_entries_before, source_entries_after,
            "source nats dir must not gain or lose entries"
        );
        assert_eq!(
            fs::read(source.join("nats.conf")).expect("read nats.conf after"),
            nats_conf_before,
            "source nats.conf must be byte-identical"
        );
        assert_eq!(
            fs::read(source.join("jwt.conf")).expect("read jwt.conf after"),
            jwt_conf_before,
            "source jwt.conf must be byte-identical"
        );
    }

    #[test]
    fn managed_nats_local_config_references_only_state_paths() {
        let temp = tempfile::tempdir().expect("temp dir");
        let bundle = fixture_bundle(temp.path());
        let state = temp.path().join("state");
        let layout = resolve_managed_nats_layout(&bundle.join("trellis/config.toml"), Some(&state))
            .expect("resolve layout");
        render_managed_nats_files(&layout).expect("render files");

        let local_config = fs::read_to_string(&layout.local_config).expect("read local config");
        let store_dir = layout.store_dir.display().to_string();
        assert!(
            local_config.contains(&format!("store_dir: {store_dir}")),
            "JetStream store must point at the state dir: {local_config}"
        );
        assert!(
            local_config.contains("include ./jwt.local.conf"),
            "the jwt include must be relative to the state dir: {local_config}"
        );
        assert!(
            !local_config.contains(layout.source_dir.display().to_string().as_str()),
            "local config must not reference the source dir: {local_config}"
        );

        let jwt_local = fs::read_to_string(&layout.jwt_local_config).expect("read jwt.local.conf");
        assert!(
            jwt_local.contains(&format!("dir: {}", layout.data_jwt_dir.display())),
            "resolver dir must point at the state data/jwt: {jwt_local}"
        );
        assert!(
            jwt_local.contains("operator: abc"),
            "jwt.local.conf keeps the read-only source material: {jwt_local}"
        );
    }

    #[test]
    fn nats_binary_resolves_without_creating_cache() {
        let temp = tempfile::tempdir().expect("temp dir");
        let binary = temp.path().join("nats-server");
        fs::write(&binary, "#!/bin/sh\n").expect("write fake binary");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt as _;
            fs::set_permissions(&binary, fs::Permissions::from_mode(0o755)).expect("chmod");
        }
        let cache_dir = temp.path().join("cache");
        let args = ServerArgs {
            mode: trellis_runtime::RuntimeMode::All,
            config: PathBuf::from("config.toml"),
            nats: None,
            rotate_first_admin: false,
            check: false,
            cache_dir: Some(cache_dir.clone()),
            nats_binary: Some(binary.clone()),
            nats_state_dir: None,
        };

        let resolved = resolve_nats_binary(&args).expect("resolve nats binary");
        assert_eq!(resolved, fs::canonicalize(&binary).expect("canonical path"));
        assert!(
            !cache_dir.exists(),
            "--nats-binary must skip download and cache creation"
        );
    }

    #[test]
    fn read_server_name_extracts_or_defaults() {
        let temp = tempfile::tempdir().expect("temp dir");
        let nats_conf = temp.path().join("nats.conf");
        fs::write(
            &nats_conf,
            "server_name: nats-local\nlisten: 0.0.0.0:4222\n",
        )
        .expect("write nats.conf");
        assert_eq!(
            read_server_name(&nats_conf).expect("read server name"),
            "nats-local"
        );

        fs::write(&nats_conf, "listen: 0.0.0.0:4222\n").expect("write nats.conf");
        assert_eq!(
            read_server_name(&nats_conf).expect("default server name"),
            "trellis"
        );
    }
}
