#!/usr/bin/env bash
set -euo pipefail

cargo run --manifest-path rust/tools/generate/Cargo.toml -- prepare --no-npm .
deno task -c ts/portals/login/deno.json build:embedded
git diff --exit-code

python - <<'PY'
from pathlib import Path
path = Path('rust/crates/trellis/src/client/authorization/provider_cache.rs')
source = path.read_text()
old = '''        "provider trust lock poisoned",\n        "provider manifest lock poisoned",\n        "provider context cache lock poisoned",\n        "provider resolution lock poisoned",\n        "provider retention lock poisoned",\n'''
new = '''        "provider state lock poisoned",\n        "provider resolution lock poisoned",\n'''
if source.count(old) != 1:
    raise SystemExit('expected provider error-classification block exactly once')
path.write_text(source.replace(old, new, 1))
PY
python /tmp/consolidate_provider_state.py
cargo fmt --manifest-path rust/Cargo.toml --all
git diff --check

test "$(git diff --name-only)" = "rust/crates/trellis/src/client/authorization/provider_cache.rs"
if grep -n -E 'self\.(root|policy_floor|manifest|verified_contexts|retention_deadlines|revocations|health)' rust/crates/trellis/src/client/authorization/provider_cache.rs; then
  echo 'fragmented provider state access remains' >&2
  exit 1
fi

cargo check --manifest-path rust/Cargo.toml -p trellis-rs --all-targets
cargo clippy --manifest-path rust/Cargo.toml -p trellis-rs --all-targets -- -D warnings
cargo test --manifest-path rust/Cargo.toml -p trellis-rs --lib
cargo test --manifest-path rust/Cargo.toml -p trellis-rs --test integration --no-run

deno run -A -c ts/deno.json rust/crates/trellis-test/integration_runner.ts \
  'event_consumers::event_consumers_invalid_authorization_proof_terms' --exact

git config user.name "trellis-validation"
git config user.email "actions@users.noreply.github.com"
git add rust/crates/trellis/src/client/authorization/provider_cache.rs
git commit -m "Consolidate authorization provider state"
git push --force origin HEAD:refs/heads/agent/result-provider-trust-state-v1
