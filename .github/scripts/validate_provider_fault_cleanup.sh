#!/usr/bin/env bash
set -euo pipefail

cargo run --manifest-path rust/tools/generate/Cargo.toml -- prepare --no-npm .
deno task -c ts/portals/login/deno.json build:embedded
git diff --exit-code

python /tmp/remove_provider_fault_injection.py
cargo fmt --manifest-path rust/Cargo.toml --all
git diff --check
python -m json.tool integration/rust-runtime-test-matrix.json >/dev/null

expected_files="$(printf '%s\n' \
  integration/rust-runtime-test-matrix.json \
  rust/crates/trellis/src/client/authorization/provider_cache.rs \
  rust/crates/trellis/src/service/local_validator.rs \
  rust/crates/trellis/tests/integration/event_consumers.rs | sort)"
actual_files="$(git diff --name-only | sort)"
test "$actual_files" = "$expected_files"

if grep -R -n --exclude-dir=target \
  -e 'fail_next_context_read' \
  -e 'fail_next_readiness_check' \
  -e 'integration_test_fail_next_context_read' \
  -e 'integration_test_fail_next_readiness_check' \
  -e 'integration_test_take_readiness_failure' \
  -e 'event-consumers.authorization-failures-redeliver-or-term' \
  rust integration; then
  echo 'provider authorization fault injection remains' >&2
  exit 1
fi

cargo check --manifest-path rust/Cargo.toml -p trellis-rs --all-targets
cargo clippy --manifest-path rust/Cargo.toml -p trellis-rs --all-targets -- -D warnings
cargo test --manifest-path rust/Cargo.toml -p trellis-rs --lib
cargo test --manifest-path rust/Cargo.toml -p trellis-rs --test integration --no-run

deno run -A -c ts/deno.json rust/crates/trellis-test/integration_runner.ts \
  event_consumers::event_consumers_invalid_authorization_proof_terms --exact

git config user.name "trellis-validation"
git config user.email "actions@users.noreply.github.com"
git add integration/rust-runtime-test-matrix.json \
  rust/crates/trellis/src/client/authorization/provider_cache.rs \
  rust/crates/trellis/src/service/local_validator.rs \
  rust/crates/trellis/tests/integration/event_consumers.rs
git commit -m "Remove provider Auth fault injection"
git push --force origin HEAD:refs/heads/agent/result-provider-fault-cleanup-v2
