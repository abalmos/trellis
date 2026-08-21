# Trellis cleanup handoff — 2026-08-20

This branch is a durable continuation point for the cleanup of `abalmos/trellis` branch `rs`.

## Branches and frozen baseline

- Canonical target branch: `rs`
- Frozen baseline for the current packet: `fa723c49bc69f927bc222702756d2124151fa962`
- `rs` was rechecked immediately before this handoff and was still exactly that SHA.
- Materialized current product WIP: `agent/wip-runtime-sdk-decouple-v3`
- This handoff branch: `agent/handoff-trellis-cleanup-2026-08-20` (WIP plus this document)
- Reproducible validator: `agent/validate-runtime-sdk-decouple-v3`
- Latest complete validator run inspected: GitHub Actions run `32432938671`, job `96628233085`.

Do **not** treat the WIP as validated or merge it merely because it is committed. It exists so a new session can directly inspect, diff, edit, and test the exact source state that had previously existed only inside the validator checkout.

## What the current WIP does

The current packet removes the workspace/runtime bootstrap dependency on generated Rust SDKs while preserving the generated SDKs as outputs:

- Removes the generated SDK workspace dependencies from `rust/Cargo.toml`.
- Removes runtime's dependency on `trellis-sdk-auth`.
- Updates `rust/Cargo.lock` so the removed generated packages do not remain as stale lockfile entries.
- Makes Auth RPC capability metadata come from the canonical API/contract metadata rather than from generated Auth Rust types.
- Moves caller-capability lookup to shared contract/API logic used by both Rust and TypeScript code generation, eliminating generator-specific hidden policy as a second source of truth.
- Adds a non-generic Router metadata-registration primitive so runtime code that dispatches outside the typed Router can still use the same exact routing/permission metadata.
- Replaces runtime calls to `trellis_sdk_auth::api::register_rpc_metadata` with canonical Auth metadata registration.

The WIP also contains the current Rust 1.98 structural error-size cleanup:

- Cold diagnostic fields in the largest `ServerError` variants were partially boxed.
- EventLog no longer returns a large `(jetstream::Message, EventVerificationFailure)` error tuple merely to carry the message back to its caller; `process_message` retains the message and calls the inner projector directly.
- The EventLog `result_large_err` expectation associated with that wrapper was removed rather than expanded.

The source-tree WIP was generated from the frozen `fa723c49` baseline and formatted before being committed to `agent/wip-runtime-sdk-decouple-v3`.

## What has already passed

The latest validator run applied the same WIP from the frozen baseline and passed all gates through aggregate `cargo check`:

1. Transform applied successfully on Rust 1.98.0.
2. `cargo fmt` and `git diff --check` passed.
3. The entire tracked `generated/` tree was deleted and `trellis-protocol-wasm` still built successfully for `wasm32-unknown-unknown`. This proves the Rust workspace no longer needs generated SDKs in order to build protocol WASM.
4. The tracked generated baseline was restored.
5. `cargo run --manifest-path rust/tools/generate/Cargo.toml -- prepare --no-npm .` succeeded.
6. `git diff --exit-code -- generated` succeeded: regeneration produced zero generated diff.
7. The embedded login portal rebuilt successfully.
8. Aggregate `cargo check --all-targets` for protocol, contracts, both code generators, `trellis-rs`, and runtime succeeded.

Do not unnecessarily repeat architectural investigation that these gates have already settled. Re-run the gates after the remaining fix, but continue from the WIP.

## Current blocker: one root cause, several Clippy reports

The broader Rust 1.98 Clippy gate is the only current blocker for this packet:

```text
cargo clippy --manifest-path rust/Cargo.toml \
  -p trellis-protocol \
  -p trellis-contracts \
  -p trellis-codegen-rust \
  -p trellis-codegen-ts \
  -p trellis-rs \
  -p trellis-runtime \
  --all-targets -- -D warnings
```

The latest run reports `clippy::result_large_err` at several EventLog and Jobs runtime startup/wait/projector functions. They are not separate design defects: every report says the `ServerError` error variant is still at least **128 bytes**. The call sites include EventLog projector `wait/startup/process` paths and Jobs advisory, janitor, projector, and worker-presence `wait/startup/process` paths.

Treat this as an enum-layout/root-cause problem. Do **not** add `#[allow]`, `#[expect(clippy::result_large_err)]`, or blanket `Box<ServerError>` wrappers at each reported function merely to silence the call sites.

Next action:

1. Work from `agent/wip-runtime-sdk-decouple-v3` (or this handoff branch, removing `CLEANUP_HANDOFF.md` from the eventual product commit).
2. Measure/inspect the remaining largest `ServerError` variant(s), including nested transparent variants such as `SubjectError`; do not guess from the Clippy call-site list.
3. `git grep` every candidate variant/constructor. If a diagnostic variant is truly impossible/unused (for example, the latest validator's grep showed no constructor for `BootstrapAuthContractMismatch` beyond its definition), delete it if semantics permit rather than preserving dead error surface.
4. Otherwise shrink cold diagnostic payloads structurally (boxed diagnostic records/strings or a compact cold-path detail object) while keeping useful typed diagnostics. Prefer one coherent representation over field-by-field boxing if the latter becomes awkward.
5. Keep public/common success-path API ergonomics intact and do not introduce a general error framework.

## Acceptance gate for this packet

After fixing the remaining `ServerError` layout, re-run the **complete frozen gate**, not just Clippy:

1. Format and `git diff --check`.
2. Remove `generated/`; build `trellis-protocol-wasm` for `wasm32-unknown-unknown` from the root Rust workspace.
3. Restore `generated/` from the frozen baseline.
4. Regenerate with `trellis-generate prepare --no-npm`.
5. Require zero diff under `generated/`.
6. Rebuild the embedded login portal.
7. Aggregate `cargo check --all-targets` for the affected crates.
8. Aggregate Clippy `--all-targets -- -D warnings`, including the EventLog/Jobs path crates reached by the selected packages.
9. Run the affected Rust library tests.
10. Recheck that no generated SDK dependency or `trellis_sdk_auth` runtime use remains, and that Rust/TS codegen use the shared capability lookup.
11. Inspect the final product-only diff against `fa723c49`; exclude validator/materializer/handoff files.
12. Re-read `rs`. Only if it is still at the expected head and the complete gate is green should the product commit be fast-forwarded/pushed to `rs`.
13. Update `CLEANUP.md` in the same landed sequence so the durable tracker reflects what actually landed.

## Continue the entire cleanup after this packet lands

`CLEANUP.md` on `rs` is the source of truth. Do not stop after landing the SDK-decoupling prerequisite. Continue through every unchecked item in coherent, reviewable packets. The intended order is:

### A. Protocol WASM + first-public V1 clean break (do together)

- Rust remains the authoritative protocol verifier.
- Stop committing built protocol-WASM outputs; make clean prep/build order explicit and let CI/release build the artifact.
- Keep Rust-only checks/tests independent of prebuilt WASM/generated outputs.
- Replace repeated stateless TS→WASM verification with an opaque verified-context handle: verify/normalize the context once, then perform repeated proof/message verification against that state.
- Explicitly invalidate/discard/rebuild that state when manifest, participant, authority, or revocation inputs change.
- Keep simple signing/input construction native TypeScript where WASM provides no semantic value.
- Add conformance coverage and a repeated-message test proving the context is not rebuilt for every message.
- Because this is the first public protocol V1, remove unreleased V2 proof names/domains/types/aliases/fallbacks/migration shims rather than preserving compatibility with development history.
- Use clean V1 names at public boundaries and unversion internal implementation names where there is no coexistence requirement. Do not leave `V1`/`V2` suffixes as historical sediment.
- Regenerate all protocol/conformance/generated artifacts and require identity/up-to-date checks.

### B. Production/test boundary + runtime composition

- Audit every remaining `integration-test-hooks` path. Delete synthetic production fault/bypass behavior; retain only genuine deterministic scheduling barriers that cannot change product semantics.
- Remove impossible disconnected/test-only runtime states and raw test-only production methods/runtime-internals exposure.
- Ensure production runtime binaries are not built with test-fault features.
- Replace avoidable fixed sleeps with event/barrier/state-based synchronization.
- Split oversized startup/runtime facades along real ownership/lifecycle boundaries. Prefer explicit owned tasks/channels where they remove shared mutable state, but do not introduce an actor framework, DI container, service locator, or abstraction layer for its own sake.
- Continue reducing unnecessary `Arc<RwLock<_>>`/`Arc<Mutex<_>>` sharing by giving mutable state a clear owner and communicating through narrow channels where that is the simpler model.

### C. Jobs, constructors, names, and remaining lint debt

- Collapse the remaining Jobs wrapper ladder and redundant constructor/helper layers.
- Remove duplicated contract metadata from `ServiceConnectOptions` where an authoritative descriptor already exists.
- Simplify device paths and combinatorial operation-registration callback variants.
- Resolve remaining `too_many_arguments`, `result_large_err`, `large_enum_variant`, and test `dead_code` findings through design changes rather than suppressions.
- Shorten long type/function names by relying on module boundaries and better concepts, not abbreviations that reduce clarity.

### D. Test strategy, fast feedback, and release separation

- Finish the separation between normal development validation and release-only packaging/platform/docs checks.
- Reduce `release.yml` to genuinely release-specific work; normal pushes/PRs should not repeatedly perform packaging/release work.
- Make the normal check path the fast authoritative format + lint/type + tests + generated-up-to-date gate.
- Ensure tests default to meaningful machine parallelism (at least roughly half the available logical/hyper-thread count unless a test's resource model requires otherwise).
- Audit the live integration suite so it tests real Trellis behavior rather than implementation-specific scaffolding.
- Record fresh cold and warm timing baselines for normal checks, targeted tests, the live suite, and release validation. Use the timings to remove redundant recompilation/generation and avoidable serialization.

### E. Final cleanup/release-readiness pass

- Run the full final format, Clippy `-D warnings`, type/check, unit/contract/runtime, generated-identity, and complete live-integration gates.
- Verify the release-only workflow independently.
- Re-read source for over-engineering, duplicated policy, compatibility leftovers, giant shared-state objects, hidden test bypasses, and unjustified lint suppressions.
- Update documentation to match the final architecture and workflow.
- Record final cold/warm timing results and test parallelism policy.
- Remove obsolete helper/validation branches or files if they would confuse maintainers.
- Delete `CLEANUP.md` only when every tracked item is either completed or intentionally rejected with a documented reason. The final tree should not ship a stale cleanup ledger.

## Working rules

- This branch is pre-stable/first-public-V1 work: do not preserve compatibility with unreleased internal history unless there is a concrete externally consumed behavior that requires it.
- Prefer deletion and direct code over compatibility shims, wrappers, feature-flagged bypass paths, or speculative abstraction.
- Do not silence Clippy to make the gate green; suppressions require a genuinely unavoidable reason and should become rarer, not broader.
- Tests should exercise production semantics. A test-only scheduling mechanism is acceptable only when it controls timing/order without creating a behavior that production can never take.
- Generated artifacts are outputs, not bootstrap prerequisites or hidden sources of truth.
- Keep runtime ownership explicit. Channels are useful when they create one clear state owner; they are not a goal by themselves.
- After each green packet, land it on `rs`, update the tracker, then immediately continue to the next unchecked packet in the same session unless there is a true external blocker.

## Useful durable references

- `agent/wip-runtime-sdk-decouple-v3`: exact materialized source WIP.
- `agent/validate-runtime-sdk-decouple-v3`: reproducible strong validator and original transform.
- `agent/materialize-runtime-sdk-decouple-v3`: one-shot materializer plus preserved error-size transform.
- Actions run `32432938671`, job `96628233085`: latest inspected validator evidence; all gates through `cargo check` passed and Clippy exposed the remaining `ServerError >= 128 bytes` root cause.
- `CLEANUP.md`: authoritative full cleanup ledger and working rules.
