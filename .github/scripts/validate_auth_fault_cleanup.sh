#!/usr/bin/env bash
set -euo pipefail

cargo run --manifest-path rust/tools/generate/Cargo.toml -- prepare --no-npm .
deno task -c ts/portals/login/deno.json build:embedded
git diff --exit-code

python /tmp/remove_auth_fault_injection.py
cargo fmt --manifest-path rust/Cargo.toml --all
git diff --check
python -m json.tool integration/rust-runtime-test-matrix.json >/dev/null

expected_files="$(printf '%s\n' \
  integration/rust-runtime-test-matrix.json \
  rust/crates/runtime/src/platform/auth/sqlite/common.rs \
  rust/crates/runtime/src/platform/auth_post_commit.rs \
  rust/crates/trellis-test/src/lib.rs \
  rust/crates/trellis/tests/integration/auth.rs | sort)"
actual_files="$(git diff --name-only | sort)"
test "$actual_files" = "$expected_files"

if grep -R -n --exclude-dir=target \
  -e 'fail_user_update_transaction' \
  -e 'clear_user_update_transaction_failure' \
  -e 'fail_next_context_revocation_dispatch' \
  -e 'consume_test_post_commit_failure' \
  -e 'auth.transaction-failure-rolls-back-state-idempotency-and-actions' \
  -e 'auth.post-commit-failure-retries-committed-context-revocation-once' \
  rust integration; then
  echo 'synthetic Auth failure injection remains' >&2
  exit 1
fi

cargo test --manifest-path rust/Cargo.toml -p trellis-runtime rollback_tests:: --lib
cargo check --manifest-path rust/Cargo.toml -p trellis-runtime -p trellis-test -p trellis-rs --all-targets
cargo clippy --manifest-path rust/Cargo.toml -p trellis-runtime -p trellis-test -p trellis-rs --all-targets -- -D warnings
cargo test --manifest-path rust/Cargo.toml -p trellis-rs --test integration --no-run

git config user.name "trellis-validation"
git config user.email "actions@users.noreply.github.com"
git add integration/rust-runtime-test-matrix.json \
  rust/crates/runtime/src/platform/auth/sqlite/common.rs \
  rust/crates/runtime/src/platform/auth_post_commit.rs \
  rust/crates/trellis-test/src/lib.rs \
  rust/crates/trellis/tests/integration/auth.rs
git commit -m "Remove synthetic Auth failure injection"
git push --force origin HEAD:refs/heads/agent/result-auth-fault-cleanup-v5
