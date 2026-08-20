#!/usr/bin/env bash
set -euo pipefail

cargo run --manifest-path rust/tools/generate/Cargo.toml -- prepare --no-npm .
deno task -c ts/portals/login/deno.json build:embedded
git diff --exit-code

python /tmp/simplify_job_ref.py
cargo fmt --manifest-path rust/Cargo.toml --all
git diff --check

expected_files="$(printf '%s\n' \
  rust/crates/trellis/src/jobs/api.rs \
  rust/crates/trellis/src/service/runtime_facade.rs | sort)"
actual_files="$(git diff --name-only | sort)"
test "$actual_files" = "$expected_files"

if grep -R -n --exclude-dir=target \
  -e 'type JobSnapshotFn' \
  -e 'type TerminalJobFn' \
  -e 'JobRef::new(' \
  -e 'Arc::new(Mutex::new(job.clone()))' \
  rust/crates/trellis/src; then
  echo 'JobRef callback/cache machinery remains' >&2
  exit 1
fi

cargo check --manifest-path rust/Cargo.toml -p trellis-rs --all-targets
cargo clippy --manifest-path rust/Cargo.toml -p trellis-rs --all-targets -- -D warnings
cargo test --manifest-path rust/Cargo.toml -p trellis-rs --lib
cargo test --manifest-path rust/Cargo.toml -p trellis-rs --test integration --no-run

deno run -A -c ts/deno.json rust/crates/trellis-test/integration_runner.ts \
  jobs::jobs_terminal_local_job_edges_and_admin_rpcs --exact

git config user.name "trellis-validation"
git config user.email "actions@users.noreply.github.com"
git add rust/crates/trellis/src/jobs/api.rs \
  rust/crates/trellis/src/service/runtime_facade.rs
git commit -m "Use concrete runtime state for JobRef"
git push --force origin HEAD:refs/heads/agent/result-job-ref-cleanup-v1
