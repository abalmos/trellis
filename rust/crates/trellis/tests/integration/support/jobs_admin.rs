use std::path::Path;
use std::process::{Child, Command, Stdio};

/// Child process running the Rust Jobs admin service for integration tests.
pub(crate) struct RustJobsAdminProcess {
    child: Child,
}

impl Drop for RustJobsAdminProcess {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

pub(crate) async fn start_rust_jobs_admin(
    runtime: &trellis_test::TrellisTestRuntime,
    admin: &mut trellis_test::TrellisTestAdmin,
    bootstrap_url: &str,
) -> RustJobsAdminProcess {
    let contract = trellis_test::TrellisTestContract::from_manifest_json(
        trellis_rs::sdk::jobs::contract::CONTRACT_JSON,
    )
    .expect("build Jobs admin service contract");
    let service_key = admin
        .provision_service_instance(bootstrap_url, &contract, Some("trellis-service-jobs"), None)
        .await
        .expect("provision Rust Jobs admin service");
    let rust_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("trellis-rs crate should live under rust/crates/trellis");
    let mut command = if let Some(binary) = std::env::var_os("TRELLIS_TEST_JOBS_SERVICE_BIN") {
        Command::new(binary)
    } else {
        let mut command = Command::new("cargo");
        command.args([
            "run",
            "--manifest-path",
            rust_dir
                .join("Cargo.toml")
                .to_str()
                .expect("UTF-8 Cargo path"),
            "-p",
            "trellis-service-jobs",
        ]);
        command
    };
    let child = command
        .env("TRELLIS_URL", runtime.trellis_url())
        .env("SESSION_KEY_SEED_BASE64URL", service_key.seed)
        .env(
            "TRELLIS_JOBS_DB_PATH",
            runtime.workdir().join("service-jobs.sqlite"),
        )
        .env("TRELLIS_TIMEOUT_MS", "5000")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("start Rust Jobs admin service process");
    RustJobsAdminProcess { child }
}
