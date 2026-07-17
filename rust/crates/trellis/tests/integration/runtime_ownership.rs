use std::fs::File;
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::time::Duration;

use async_nats::jetstream::{self, kv};
use async_nats::ConnectOptions;
use bytes::Bytes;

const TEST_NAME: &str = "runtime_ownership::runtime_singleton_ownership_lifecycle";
const INCOMPATIBLE_BUCKET_TEST_NAME: &str =
    "runtime_ownership::runtime_incompatible_lease_bucket_fails_before_storage_open";
const LEASE_BUCKET: &str = "trellis_runtime_leases";

struct RuntimeProcess {
    child: Child,
    url: String,
    stderr_path: PathBuf,
}

impl RuntimeProcess {
    fn start(
        runtime: &trellis_test::TrellisTestRuntime,
        mode: &str,
        config_path: &Path,
        label: &str,
    ) -> Self {
        let port = reserve_port();
        write_runtime_config(runtime, mode, config_path, label, port);
        let stdout = File::create(runtime.workdir().join(format!("{label}.stdout.log")))
            .expect("create runtime stdout log");
        let stderr_path = runtime.workdir().join(format!("{label}.stderr.log"));
        let stderr = File::create(&stderr_path).expect("create runtime stderr log");
        let child = runtime_command(mode, config_path)
            .stdout(Stdio::from(stdout))
            .stderr(Stdio::from(stderr))
            .spawn()
            .expect("start Rust runtime process");
        Self {
            child,
            url: format!("http://127.0.0.1:{port}"),
            stderr_path,
        }
    }

    async fn wait_ready(&mut self) {
        let deadline = tokio::time::Instant::now() + Duration::from_secs(15);
        loop {
            if reqwest::get(format!("{}/readyz", self.url))
                .await
                .is_ok_and(|response| response.status().is_success())
            {
                return;
            }
            if let Some(status) = self.child.try_wait().expect("inspect runtime process") {
                panic!(
                    "runtime exited before readiness with {status}: {}",
                    self.stderr()
                );
            }
            assert!(
                tokio::time::Instant::now() < deadline,
                "runtime did not become ready: {}",
                self.stderr()
            );
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    }

    async fn wait_exit(&mut self) -> ExitStatus {
        let deadline = tokio::time::Instant::now() + Duration::from_secs(15);
        loop {
            if let Some(status) = self.child.try_wait().expect("inspect runtime process") {
                return status;
            }
            assert!(
                tokio::time::Instant::now() < deadline,
                "runtime did not exit: {}",
                self.stderr()
            );
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    }

    async fn terminate(&mut self) -> ExitStatus {
        let started = tokio::time::Instant::now();
        let pid = self.child.id().to_string();
        let signal = Command::new("kill")
            .args(["-TERM", &pid])
            .status()
            .expect("send SIGTERM to runtime");
        assert!(signal.success(), "send SIGTERM to runtime process");
        let status = self.wait_exit().await;
        assert!(
            started.elapsed() < Duration::from_secs(10),
            "SIGTERM shutdown exceeded its bound: {}",
            self.stderr()
        );
        status
    }

    async fn is_ready(&mut self) -> bool {
        self.child
            .try_wait()
            .expect("inspect runtime process")
            .is_none()
            && reqwest::get(format!("{}/readyz", self.url))
                .await
                .is_ok_and(|response| response.status().is_success())
    }

    fn stderr(&self) -> String {
        std::fs::read_to_string(&self.stderr_path).unwrap_or_default()
    }
}

impl Drop for RuntimeProcess {
    fn drop(&mut self) {
        if self.child.try_wait().ok().flatten().is_none() {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }
}

#[tokio::test]
async fn runtime_singleton_ownership_lifecycle() {
    trellis_test::set_current_test_tenant(TEST_NAME);
    let runtime =
        trellis_test::TrellisTestRuntime::start(trellis_test::TrellisTestRuntimeOptions::default())
            .await
            .expect("start live Trellis test runtime");
    let client = ConnectOptions::new()
        .credentials_file(runtime.workdir().join("nats/creds/trellis-auth.creds"))
        .await
        .expect("load Trellis NATS credentials")
        .connect(runtime.nats_url())
        .await
        .expect("connect authenticated lease test client");

    let first_config = runtime.workdir().join("jobs-first.toml");
    let mut first = RuntimeProcess::start(&runtime, "jobs", &first_config, "jobs-first");
    first.wait_ready().await;

    let blocked_storage = runtime.workdir().join("blocked-jobs.sqlite");
    std::fs::create_dir(&blocked_storage).expect("create path that SQLite cannot open as a file");
    let duplicate_config = runtime.workdir().join("jobs-duplicate.toml");
    let mut duplicate =
        RuntimeProcess::start(&runtime, "jobs", &duplicate_config, "jobs-duplicate");
    let duplicate_status = duplicate.wait_exit().await;
    let duplicate_error = duplicate.stderr();
    assert!(!duplicate_status.success());
    assert!(duplicate_error.contains("jobs.owner"), "{duplicate_error}");
    assert!(duplicate_error.contains("OwnerHeld"), "{duplicate_error}");
    assert!(!duplicate_error.contains("runtime storage failed"));
    assert!(blocked_storage.is_dir());
    assert!(first.is_ready().await, "first Jobs owner lost readiness");

    assert!(first.terminate().await.success());
    let successor_config = runtime.workdir().join("jobs-successor.toml");
    let mut successor =
        RuntimeProcess::start(&runtime, "jobs", &successor_config, "jobs-successor");
    successor.wait_ready().await;

    let leases = jetstream::new(client.clone())
        .get_key_value(LEASE_BUCKET)
        .await
        .expect("open runtime lease bucket");
    leases
        .delete("jobs.owner")
        .await
        .expect("force Jobs owner lease loss");
    let loss_status = successor.wait_exit().await;
    assert!(!loss_status.success(), "lease loss must fail the process");
    assert!(!successor.is_ready().await, "lost owner remained ready");
    let loss_error = successor.stderr();
    assert!(loss_error.contains("jobs.owner"), "{loss_error}");
    assert!(loss_error.contains("OwnerRenewal"), "{loss_error}");
    assert!(matches!(
        leases
            .entry("jobs.owner")
            .await
            .expect("inspect lost Jobs lease")
            .map(|entry| entry.operation),
        None | Some(kv::Operation::Delete | kv::Operation::Purge)
    ));
    acquire_and_release(&leases, "jobs.owner").await;

    let held_health_revision = leases
        .create("health.owner", Bytes::from_static(b"fixture-holder"))
        .await
        .expect("pre-hold Health owner lease");
    let all_config = runtime.workdir().join("all-partial.toml");
    let mut all = RuntimeProcess::start(&runtime, "all", &all_config, "all-partial");
    let all_status = all.wait_exit().await;
    let all_error = all.stderr();
    assert!(!all_status.success());
    assert!(all_error.contains("health.owner"), "{all_error}");
    assert!(all_error.contains("OwnerHeld"), "{all_error}");
    acquire_and_release(&leases, "platform.owner").await;
    acquire_and_release(&leases, "jobs.owner").await;
    leases
        .delete_expect_revision("health.owner", Some(held_health_revision))
        .await
        .expect("release fixture-held Health lease");

    let all_runtime_config = runtime.workdir().join("all-runtime.toml");
    let mut all_runtime =
        RuntimeProcess::start(&runtime, "all", &all_runtime_config, "all-runtime");
    all_runtime.wait_ready().await;
    let jobs_entry = leases
        .entry("jobs.owner")
        .await
        .expect("inspect all-mode Jobs lease")
        .expect("all-mode Jobs lease exists");
    let manipulated_revision = leases
        .update(
            "jobs.owner",
            Bytes::from_static(b"fixture-takeover"),
            jobs_entry.revision,
        )
        .await
        .expect("make all-mode Jobs lease guard stale");

    let mut fixture_owned = vec![("jobs.owner", manipulated_revision)];
    for key in ["platform.owner", "health.owner", "eventlog.owner"] {
        fixture_owned.push((key, wait_to_acquire(&leases, key).await));
    }

    let all_loss_status = all_runtime.wait_exit().await;
    assert!(
        !all_loss_status.success(),
        "one all-mode lease loss must fail the process"
    );
    assert!(
        !all_runtime.is_ready().await,
        "all-mode runtime retained readiness after one owner loss"
    );
    let all_loss_error = all_runtime.stderr();
    assert!(all_loss_error.contains("jobs.owner"), "{all_loss_error}");
    assert!(all_loss_error.contains("OwnerRenewal"), "{all_loss_error}");
    let manipulated = leases
        .entry("jobs.owner")
        .await
        .expect("inspect manipulated Jobs lease")
        .expect("fixture-owned Jobs lease remains");
    assert_eq!(manipulated.revision, manipulated_revision);
    assert_eq!(manipulated.value, Bytes::from_static(b"fixture-takeover"));
    for (key, revision) in fixture_owned {
        leases
            .delete_expect_revision(key, Some(revision))
            .await
            .unwrap_or_else(|error| panic!("release fixture-owned lease {key}: {error}"));
    }
}

#[tokio::test]
async fn runtime_incompatible_lease_bucket_fails_before_storage_open() {
    trellis_test::set_current_test_tenant(INCOMPATIBLE_BUCKET_TEST_NAME);
    let runtime =
        trellis_test::TrellisTestRuntime::start(trellis_test::TrellisTestRuntimeOptions::default())
            .await
            .expect("start live Trellis test runtime");
    let client = ConnectOptions::new()
        .credentials_file(runtime.workdir().join("nats/creds/trellis-auth.creds"))
        .await
        .expect("load Trellis NATS credentials")
        .connect(runtime.nats_url())
        .await
        .expect("connect authenticated lease test client");
    jetstream::new(client)
        .create_key_value(kv::Config {
            bucket: LEASE_BUCKET.to_owned(),
            history: 1,
            max_age: Duration::from_secs(1),
            num_replicas: 1,
            ..Default::default()
        })
        .await
        .expect("precreate incompatible lease bucket");

    let blocked_storage = runtime.workdir().join("incompatible-jobs.sqlite");
    std::fs::create_dir(&blocked_storage).expect("create path that SQLite cannot open as a file");
    let config = runtime.workdir().join("incompatible.toml");
    let mut process = RuntimeProcess::start(&runtime, "jobs", &config, "incompatible");
    let status = process.wait_exit().await;
    let error = process.stderr();

    assert!(!status.success());
    assert!(error.contains("InfrastructureMismatch"), "{error}");
    assert!(error.contains("max_age"), "{error}");
    assert!(!error.contains("runtime storage failed"), "{error}");
    assert!(blocked_storage.is_dir());
}

async fn acquire_and_release(leases: &kv::Store, key: &str) {
    let revision = leases
        .create(key, Bytes::from_static(b"fixture-probe"))
        .await
        .unwrap_or_else(|error| panic!("acquire released lease {key}: {error}"));
    leases
        .delete_expect_revision(key, Some(revision))
        .await
        .unwrap_or_else(|error| panic!("release probed lease {key}: {error}"));
}

async fn wait_to_acquire(leases: &kv::Store, key: &str) -> u64 {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
    loop {
        match leases
            .create(key, Bytes::from_static(b"fixture-takeover"))
            .await
        {
            Ok(revision) => return revision,
            Err(error) => {
                assert!(
                    tokio::time::Instant::now() < deadline,
                    "could not acquire owner lease {key} after forced ownership loss: {error}"
                );
                tokio::time::sleep(Duration::from_millis(25)).await;
            }
        }
    }
}

fn write_runtime_config(
    runtime: &trellis_test::TrellisTestRuntime,
    mode: &str,
    path: &Path,
    label: &str,
    port: u16,
) {
    let nats_dir = runtime.workdir().join("nats/creds");
    let session_seed = runtime
        .workdir()
        .join(&runtime.manifest().paths.session_seed);
    let jobs_path = if label == "jobs-duplicate" {
        runtime.workdir().join("blocked-jobs.sqlite")
    } else {
        runtime.workdir().join(format!("{label}-jobs.sqlite"))
    };
    let mut config = format!(
        r#"instance_name = "{label}"
event_session_seed_file = "{}"

[http]
port = {port}

[nats]
servers = "{}"

[nats.runtime]
auth_creds_path = "{}"
trellis_creds_path = "{}"
system_creds_path = "{}"
sentinel_creds_path = "{}"

[jobs.storage]
kind = "sqlite"
path = "{}"

[leases]
bucket = "{LEASE_BUCKET}"
replicas = 1
ttl_ms = 3000
renew_ms = 500
"#,
        toml_path(&session_seed),
        runtime.nats_url(),
        toml_path(&nats_dir.join("auth-auth.creds")),
        toml_path(&nats_dir.join("trellis-auth.creds")),
        toml_path(&nats_dir.join("system.creds")),
        toml_path(&nats_dir.join("sentinel.creds")),
        toml_path(&jobs_path),
    );
    if mode == "all" {
        config.push_str(&format!(
            r#"
[nats.auth_callout]
issuer_signing_seed_file = "unused-issuer.seed"
target_signing_seed_file = "unused-target.seed"
xkey_seed_file = "unused-xkey.seed"

[platform.storage]
kind = "sqlite"
path = "{}"

[health.storage]
kind = "sqlite"
path = "{}"

[eventlog.storage]
kind = "sqlite"
path = "{}"
"#,
            toml_path(&runtime.workdir().join("all-platform.sqlite")),
            toml_path(&runtime.workdir().join("all-health.sqlite")),
            toml_path(&runtime.workdir().join("all-eventlog.sqlite")),
        ));
    }
    std::fs::write(path, config).expect("write Rust runtime config");
}

fn runtime_command(mode: &str, config_path: &Path) -> Command {
    let rust_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("trellis-rs crate should live under rust/crates/trellis");
    let mut command = if let Some(binary) = std::env::var_os("TRELLIS_TEST_SERVER_BIN") {
        Command::new(binary)
    } else {
        let mut command = Command::new("cargo");
        command.args([
            "run",
            "--quiet",
            "--manifest-path",
            rust_dir
                .join("Cargo.toml")
                .to_str()
                .expect("UTF-8 Cargo path"),
            "-p",
            "trellis-runtime",
            "--bin",
            "trellis-server",
            "--",
        ]);
        command
    };
    command
        .args([
            mode,
            "--config",
            config_path.to_str().expect("UTF-8 runtime config path"),
        ])
        .current_dir(rust_dir)
        .stdin(Stdio::null());
    command
}

fn reserve_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .expect("reserve runtime port")
        .local_addr()
        .expect("read reserved runtime port")
        .port()
}

fn toml_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "\\\\")
}
