#!/usr/bin/env bash
set -euo pipefail

cargo run --manifest-path rust/tools/generate/Cargo.toml -- prepare --no-npm .
deno task -c ts/portals/login/deno.json build:embedded
git diff --exit-code

python /tmp/simplify_listener_teardown.py
cargo fmt --manifest-path rust/Cargo.toml --all
git diff --check

test "$(git diff --name-only)" = "rust/crates/trellis/src/service/runtime_facade.rs"
if grep -n -E 'spawn_service_event_listener(s)?_cleanup|event_listeners\.lock\(\)\.await' rust/crates/trellis/src/service/runtime_facade.rs; then
  echo 'runtime-dependent listener cleanup remains' >&2
  exit 1
fi

cargo check --manifest-path rust/Cargo.toml -p trellis-rs --all-targets
cargo clippy --manifest-path rust/Cargo.toml -p trellis-rs --all-targets -- -D warnings
cargo test --manifest-path rust/Cargo.toml -p trellis-rs --lib
cargo test --manifest-path rust/Cargo.toml -p trellis-rs --test integration --no-run

deno run -A -c ts/deno.json rust/crates/trellis-test/integration_runner.ts \
  'event_consumers::event_consumers_abort_re_register_restarts_delivery' --exact

deno run -A -c ts/deno.json rust/crates/trellis-test/integration_runner.ts \
  'event_consumers::event_consumers_stop_teardown_stops_durable_delivery' --exact

git config user.name "trellis-validation"
git config user.email "actions@users.noreply.github.com"
git add rust/crates/trellis/src/service/runtime_facade.rs
git commit -m "Make durable listener teardown synchronous"
git push --force origin HEAD:refs/heads/agent/result-listener-teardown-v1
