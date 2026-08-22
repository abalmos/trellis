#!/usr/bin/env bash
set -euo pipefail

TARGET_BRANCH="${TARGET_BRANCH:-agent/wasm-boundary-v3}"
TRANSFORM_BRANCH="${TRANSFORM_BRANCH:-agent/rs-gate-validation}"

# The workflow already persisted .validation-run after configuring commit signing.
git fetch origin "$TRANSFORM_BRANCH"
git show origin/"$TRANSFORM_BRANCH":scripts/agent_wasm_boundary_v3_pre.py > /tmp/wasm-boundary-pre.py
git show origin/"$TRANSFORM_BRANCH":scripts/agent_wasm_boundary_v3.py > /tmp/wasm-boundary.py
git show origin/"$TRANSFORM_BRANCH":scripts/agent_wasm_boundary_v3_participant_fix.py > /tmp/wasm-boundary-participant.py
git show origin/"$TRANSFORM_BRANCH":scripts/agent_wasm_boundary_v3_protocol_writer_fix.py > /tmp/wasm-boundary-protocol-writer.py
git show origin/"$TRANSFORM_BRANCH":scripts/agent_wasm_boundary_v3_ci_fix.py > /tmp/wasm-boundary-ci.py
git show origin/"$TRANSFORM_BRANCH":scripts/agent_wasm_boundary_v3_proof_v1.py > /tmp/proof-v1.py
git show origin/"$TRANSFORM_BRANCH":scripts/agent_wasm_boundary_v3_vector_regen.py > /tmp/vector-regen.py
python3 -m py_compile \
  /tmp/wasm-boundary-pre.py \
  /tmp/wasm-boundary.py \
  /tmp/wasm-boundary-participant.py \
  /tmp/wasm-boundary-protocol-writer.py \
  /tmp/wasm-boundary-ci.py \
  /tmp/proof-v1.py \
  /tmp/vector-regen.py

python3 /tmp/wasm-boundary-pre.py
python3 /tmp/wasm-boundary.py
python3 /tmp/wasm-boundary-participant.py
python3 /tmp/wasm-boundary-protocol-writer.py
python3 /tmp/wasm-boundary-ci.py
git rm \
  ts/packages/trellis/auth/protocol_wasm/trellis_protocol_wasm.js \
  ts/packages/trellis/auth/protocol_wasm/trellis_protocol_wasm_bg.wasm \
  ts/packages/trellis/auth/protocol_wasm/trellis_protocol_wasm_bytes.ts
git rm .validation-run
cargo fmt --manifest-path rust/xtask/Cargo.toml
cargo fmt --manifest-path rust/tools/generate/Cargo.toml
deno fmt -c ts/deno.json \
  ts/deno.json \
  ts/apps/console/deno.json \
  ts/apps/console/src/lib/trellis-context.svelte.ts \
  ts/packages/trellis/deno.json \
  ts/packages/trellis/contract_support/protocol_artifacts.ts \
  ts/packages/trellis/contract_support/protocol_artifacts_test.ts \
  ts/packages/trellis/contract_support/protocol_resolution.ts \
  ts/packages/trellis/contract_support/mod.ts \
  ts/packages/trellis/client_connect.ts \
  ts/packages/trellis/service/runtime/service.ts \
  ts/packages/trellis/contracts/trellis_core.ts \
  ts/packages/trellis/tests/connect_public_typing_test.ts \
  ts/packages/trellis-svelte/src/context.svelte.ts \
  ts/packages/trellis-svelte/src/context.api_check.ts \
  ts/packages/trellis-test/src/runtime.ts \
  ts/packages/trellis-test/src/types.ts \
  ts/packages/trellis-test/src/admin/methods.ts \
  ts/portals/login/contract.ts \
  ts/portals/login/deno.json \
  docs/deno.json
git diff --check
git add -A
git commit -m 'Move protocol WASM to runtime resolution boundary'

python3 /tmp/proof-v1.py
python3 /tmp/vector-regen.py add
cargo test --locked --manifest-path rust/Cargo.toml \
  -p trellis-protocol regenerate_authorization_proof_v1_vector -- --ignored
python3 /tmp/vector-regen.py remove
cargo fmt --manifest-path rust/Cargo.toml --all
deno fmt -c ts/deno.json ts/packages/trellis/auth
if rg -n 'Authorization(Request|Event)Proof(Input)?V2|AuthorizationEventPublisherV2|VerifiedAuthorization(Request|Event)V2|AuthorizationProvider(Request|Event)V2|create_(request|event)_proof_v2|verify_event_proof_v2|verify_v2_signature|build_authorization_(request|event)_proof_input_v2|sign_authorization_(request|event)_v2|verify_authorization_(request|event)_v2|VerifyAuthorization(Request|Event)V2|verifyAuthorization(Request|Event)V2Wasm|verify(Request|Event)V2|authorization-(request|event)-proof\.v2' rust ts conformance docs; then
  echo 'obsolete request/event proof v2 surface remains' >&2
  exit 1
fi
rg -n 'trellis\.authorization-request-proof\.v1' \
  rust/crates/protocol/src/authorization.rs ts/packages/trellis/auth/proof.ts
rg -n 'trellis\.authorization-event-proof\.v1' \
  rust/crates/protocol/src/authorization.rs ts/packages/trellis/auth/proof.ts
rg -n 'verify_authorization_request_v1|verify_authorization_event_v1' \
  rust/crates/protocol-wasm/src/lib.rs ts/packages/trellis/auth/protocol_wasm.ts
python3 - <<'PY'
import json
from pathlib import Path
fixture = json.loads(Path('conformance/authorization-context/vectors.json').read_text())
chain = fixture['completeChain']
request = bytes.fromhex(chain['requestProofInputHex'])
event = bytes.fromhex(chain['eventProofInputHex'])
if b'trellis.authorization-request-proof.v1' not in request:
    raise SystemExit('request vector did not regenerate with v1 domain')
if b'trellis.authorization-event-proof.v1' not in event:
    raise SystemExit('event vector did not regenerate with v1 domain')
PY
git diff --check
git add -A
git commit -m 'Publish authorization request and event proofs as v1'

# Contract source evaluation must succeed with the runtime WASM directory absent.
rm -rf ts/packages/trellis/auth/protocol_wasm
deno eval -c ts/deno.json \
  'const m = await import("./ts/packages/trellis/contracts/trellis_core.ts"); if (!m.API_DIGEST || !m.default.CONTRACT_DIGEST || !m.default.PARTICIPANT) throw new Error("core contract did not build intrinsic artifacts")'
deno eval -c ts/deno.json \
  'const m = await import("./ts/portals/login/contract.ts"); if (!m.default.CONTRACT_DIGEST || !m.default.PARTICIPANT) throw new Error("portal contract did not build intrinsic artifacts")'
test ! -e ts/packages/trellis/auth/protocol_wasm/trellis_protocol_wasm.js

# Refresh exactly the dynamic TypeScript participant baselines restored by the
# clean-generation fix. No unrelated generated or lockfile drift is accepted.
cargo run --manifest-path rust/xtask/Cargo.toml -- prepare
cargo fmt --manifest-path rust/Cargo.toml --all
cargo fmt --manifest-path rust/tools/generate/Cargo.toml
cargo fmt --manifest-path rust/xtask/Cargo.toml
python3 - <<'PY'
import subprocess
expected = {
    'generated/protocol/participants/trellis.console@v1.json',
    'generated/protocol/participants/trellis.core@v1.json',
    'generated/protocol/participants/trellis.portal.activation@v1.json',
}
changed = set(filter(None, subprocess.check_output(
    ['git', 'diff', '--name-only'], text=True
).splitlines()))
print('\n'.join(sorted(changed)))
if changed != expected:
    raise SystemExit(f'unexpected baseline drift: {sorted(changed)}')
PY
git diff --check
# These baselines are already tracked under the intentionally ignored generated/
# tree. Stage modifications only; never force-add a new ignored artifact.
git add -u -- \
  generated/protocol/participants/trellis.console@v1.json \
  generated/protocol/participants/trellis.core@v1.json \
  generated/protocol/participants/trellis.portal.activation@v1.json
git commit -m 'Refresh generated participant baselines'

# Prove clean generation order: source resolution and participant emission occur
# before protocol WASM exists; WASM and portal are explicit later consumers.
rm -rf generated ts/packages/trellis/auth/protocol_wasm rust/crates/runtime/generated/portal
cargo check --locked --manifest-path rust/Cargo.toml -p trellis-protocol-wasm
cargo run --manifest-path rust/tools/generate/Cargo.toml -- prepare --no-npm .
test ! -e ts/packages/trellis/auth/protocol_wasm/trellis_protocol_wasm.js
test -s generated/protocol/participants/trellis.core@v1.json
test -s generated/protocol/participants/trellis.console@v1.json
test -s generated/protocol/participants/trellis.portal.activation@v1.json
cargo run --manifest-path rust/xtask/Cargo.toml -- protocol-wasm
test -s ts/packages/trellis/auth/protocol_wasm/trellis_protocol_wasm.js
test -s ts/packages/trellis/auth/protocol_wasm/trellis_protocol_wasm.d.ts
test -s ts/packages/trellis/auth/protocol_wasm/trellis_protocol_wasm_bg.wasm
test -s ts/packages/trellis/auth/protocol_wasm/trellis_protocol_wasm_bytes.ts
deno task -c ts/portals/login/deno.json build:embedded
git diff --exit-code

# Intrinsic participant identity must match Rust contextual resolution and no
# TypeScript author/runtime surface may resurrect eager contextual identity.
deno test -A -c ts/deno.json \
  ts/packages/trellis/contract_support/protocol_artifacts_test.ts
if rg -n 'PARTICIPANT_NEEDS_DIGEST' ts \
  --glob '!**/npm/**' \
  --glob '!**/node_modules/**'; then
  echo 'eager participant needs digest remains in TypeScript source' >&2
  exit 1
fi
git diff --exit-code

# Actual runtime-resolution consumers must build the focused WASM when starting
# from a clean consumer state.
rm -rf ts/packages/trellis/auth/protocol_wasm
deno task -c ts/deno.json test:contracts
rm -rf ts/packages/trellis/auth/protocol_wasm
deno task -c ts/apps/console/deno.json check
test -s ts/packages/trellis/auth/protocol_wasm/trellis_protocol_wasm.js
git diff --exit-code

# TypeScript and package assembly.
deno fmt -c ts/deno.json --check
deno check -c ts/deno.json \
  ts/packages/trellis/index.ts \
  ts/packages/trellis-svelte/src/index.ts \
  ts/packages/trellis-svelte/src/context.svelte.ts \
  ts/packages/trellis-test/index.ts
deno task -c ts/deno.json check:integration
deno task -c ts/deno.json test:prepared
deno task -c ts/packages/trellis/deno.json build:npm
deno test -A -c ts/deno.json ts/packages/trellis/tests/npm_artifact_smoke_test.ts
git diff --exit-code

# Main Rust workspace is lock-backed.
cargo check --locked --manifest-path rust/Cargo.toml --workspace --all-targets
cargo test --locked --manifest-path rust/Cargo.toml -p trellis-protocol
cargo test --locked --manifest-path rust/Cargo.toml -p trellis-rs --lib
cargo clippy --locked --manifest-path rust/Cargo.toml \
  -p trellis-protocol -p trellis-protocol-wasm -p trellis-rs \
  --all-targets -- -D warnings

# Standalone generator owns the participant freshness regression and must be
# tested explicitly because it is excluded from rust/Cargo.toml.
cargo test --manifest-path rust/tools/generate/Cargo.toml
cargo clippy --manifest-path rust/tools/generate/Cargo.toml --all-targets -- -D warnings

cargo fmt --manifest-path rust/Cargo.toml --all --check
cargo fmt --manifest-path rust/tools/generate/Cargo.toml --check
cargo fmt --manifest-path rust/xtask/Cargo.toml --check
cargo test --manifest-path rust/xtask/Cargo.toml
cargo clippy --manifest-path rust/xtask/Cargo.toml --all-targets -- -D warnings
git diff --exit-code

# Validate every modified production workflow.
go install github.com/rhysd/actionlint/cmd/actionlint@v1.7.7
"$(go env GOPATH)/bin/actionlint" \
  .github/workflows/check.yml \
  .github/workflows/release.yml \
  .github/workflows/pages.yml

git log --oneline -4
git push origin HEAD:"$TARGET_BRANCH"
