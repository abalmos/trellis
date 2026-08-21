# Trellis cleanup handoff — runtime SDK decoupling and remaining 0.12 cleanup

This file is a durable handoff for a new ChatGPT/Codex session. It records the exact in-progress state that has **not** yet landed on `rs`.

## Repository / protected baseline

- Repository: `abalmos/trellis`
- Product branch: `rs`
- Frozen product baseline for the current packet: `fa723c49bc69f927bc222702756d2124151fa962` (`Track landed typed-error cleanup`)
- `rs` was re-read immediately before this handoff and was still exactly at that SHA.
- Do not overwrite concurrent work: re-read `rs` immediately before every product write. If it has moved, transplant/revalidate the packet against the new head rather than force-pushing old state.
- Cleanup changes land as direct `rs` commits after validation. Hidden `agent/*` branches are for validation/handoff only; do not create cleanup PRs unless explicitly requested.

## Exact in-progress artifacts

The current transform and validation harness live on this branch:

- `scripts/agent_runtime_sdk_decouple_v3.py` — base product transform.
- `.github/workflows/agent-runtime-sdk-decouple-v3.yml` — authoritative current validator. It applies several small corrections to the transform in `/tmp` and applies the current Rust 1.98 error-size cleanup before running the gates. **Read the workflow as well as the script; the script alone is not yet the complete current patch.**
- Latest validation run: `32432938671`
- Latest validation job: `96628233085`
- Failure marker branch: `agent/failed-runtime-sdk-decouple-v3`
- Running marker branch: `agent/running-runtime-sdk-decouple-v3`
- Intended all-green product result branch: `agent/result-runtime-sdk-decouple-v3` (not produced yet because Clippy still fails).

## What the current product transform does

1. Removes generated SDK crates from the root Rust workspace dependency declarations and removes `trellis-sdk-auth` from `trellis-runtime`.
2. Centralizes API capability-to-permission matching in canonical protocol `ApiArtifactV1::capability_names_for_surface(...)` instead of duplicating raw-JSON lookup code in Rust and TypeScript generators.
3. Routes both Rust and TypeScript codegen through that canonical capability lookup. Generated SDK output must remain unchanged.
4. Adds a small non-generated Router primitive (`register_rpc_metadata_parts`) that generated descriptors and built-in runtime code can both reduce to.
5. Makes Auth runtime routing metadata come from `rust/crates/runtime/trellis.api.json`, parsed by `trellis-protocol`, including derived subjects and exact capability names. Runtime no longer imports the generated Auth SDK merely to recover its own route metadata.
6. Updates verifier tests to use the runtime-owned Auth metadata helper rather than `trellis_sdk_auth`.
7. Includes `rust/Cargo.lock` in the product packet so the removed generated SDK package dependency cannot remain stale.
8. Current Rust 1.98 error-size cleanup structurally boxes cold diagnostic strings in remaining large `ServerError` variants and removes EventLog's separate large `(Message, error/event)` result wrapper by processing the borrowed message directly. Do not replace this with broad Clippy suppressions.

## Gates already proven from the frozen baseline

The latest validator has proven all of the following before Clippy:

- With the entire `generated/` directory physically absent, `trellis-protocol-wasm` builds for `wasm32-unknown-unknown` under stable Rust 1.98.
- After that proof, restoring the tracked `generated/` baseline and running `prepare --no-npm` produces **zero diff under `generated/`**.
- The embedded login portal rebuilds successfully.
- Aggregate affected-crate `cargo check --all-targets` succeeds.
- Auth runtime has no `trellis-sdk-auth` dependency/use.
- The root workspace no longer declares generated SDK workspace dependencies.
- `rust/Cargo.lock` no longer contains package `trellis-sdk-auth` after the transform/generation path.

The validator intentionally restores the tracked generated tree after the clean-WASM proof. `prepare --no-npm` does not recreate every tracked participant artifact from an empty `generated/` tree, so deleting it and then comparing immediately produces unrelated deletions. Do not weaken the zero-generated-diff requirement; preserve the restore-then-regenerate sequence.

## Current blocker: remaining Rust 1.98 `result_large_err`

Run `32432938671` reaches Clippy and then fails because `trellis_rs::service::ServerError` remains at least 128 bytes. The visible call sites are symptoms of the same enum-size cause, not independent design problems.

EventLog runtime:

- `rust/crates/eventlog-runtime/src/projector.rs:85` — handle `wait() -> Result<(), ServerError>`
- `rust/crates/eventlog-runtime/src/projector.rs:104` — projector `start(...) -> Result<EventLogProjectorHandle, ServerError>`
- `rust/crates/eventlog-runtime/src/projector.rs:160` — `Result<(), ServerError>`
- `rust/crates/eventlog-runtime/src/projector.rs:173` — `Result<(), ServerError>`

Jobs runtime:

- `rust/crates/jobs-runtime/src/advisory.rs:50` — `wait() -> Result<(), ServerError>`
- `rust/crates/jobs-runtime/src/advisory.rs:107` — `Result<AdvisoryHandle, ServerError>`
- `rust/crates/jobs-runtime/src/janitor.rs:44` — `wait() -> Result<(), ServerError>`
- `rust/crates/jobs-runtime/src/janitor.rs:154` — `Result<JanitorHandle, ServerError>`
- `rust/crates/jobs-runtime/src/projector.rs:47` — `wait() -> Result<(), ServerError>`
- `rust/crates/jobs-runtime/src/projector.rs:65` — `Result<JobsProjectorHandle, ServerError>`
- `rust/crates/jobs-runtime/src/projector.rs:161` — `Result<(), ServerError>`
- `rust/crates/jobs-runtime/src/worker_presence.rs:45` — `wait() -> Result<(), ServerError>`
- `rust/crates/jobs-runtime/src/worker_presence.rs:92` — `Result<WorkerPresenceProjectorHandle, ServerError>`

### Correct next action

Do **not** add `#[allow]/#[expect(clippy::result_large_err)]` to all those functions and do not mechanically box every `Result` error. Inspect the current `ServerError` layout/variants and find the remaining largest cold-path payload(s). Shrink the enum structurally (box genuinely large cold diagnostic payloads or otherwise simplify the variant ownership) until the enum falls below Clippy's threshold. Then rerun the entire frozen gate from the beginning.

The current validator already boxes `OperationMismatch.operation_id`, `BootstrapBindingMismatch.service_name`, and `BootstrapAuthContractMismatch.service_name`, and simplifies EventLog's large tuple wrapper. Re-evaluate enum size after those changes rather than guessing from the old baseline.

## Product files currently expected in this packet

The validator's product-only commit step currently stages:

- `rust/Cargo.toml`
- `rust/Cargo.lock`
- `rust/crates/runtime/Cargo.toml`
- `rust/crates/protocol/src/api.rs`
- `rust/crates/contracts/src/lib.rs`
- `rust/crates/codegen-rust/src/lib.rs`
- `rust/crates/codegen-ts/src/lib.rs`
- `rust/crates/trellis/src/service/router.rs`
- `rust/crates/trellis/src/service/error.rs`
- `rust/crates/trellis/src/service/operations.rs`
- `rust/crates/trellis/src/service/bindings.rs`
- `rust/crates/eventlog-runtime/src/projector.rs`
- `rust/crates/runtime/src/platform/auth/rpc/mod.rs`
- `rust/crates/runtime/src/platform/auth/verifier.rs`

Inspect the final product diff rather than assuming this list remains correct after fixing `ServerError`.

## How to finish and land this packet

1. Re-read `rs` and this handoff/workflow/script.
2. Identify and structurally shrink the remaining large `ServerError` cause(s).
3. Update the hidden validator/transform so the complete patch is reproducible from the current `rs` baseline.
4. Rerun the full gate: format/diff check; protocol-WASM build with `generated/` absent; restore generated baseline; regenerate and require zero generated diff; embedded portal build; aggregate affected-crate check; Clippy `--all-targets -- -D warnings`; affected library tests; dependency/grep assertions; product-only commit with clean worktree.
5. Inspect `agent/result-runtime-sdk-decouple-v3` against the frozen base. Verify no validator/handoff files are in the product diff.
6. Re-read `rs` immediately before landing. If unchanged, fast-forward/apply the validated product commit directly to `rs`; if moved, transplant to the new head and rerun the gate. Never force old product state over a changed `rs`.
7. Update `CLEANUP.md` on `rs` to record the landed packet and correct the obsolete WASM preparation-order assumption: Rust protocol/WASM now builds independently of generated SDKs.

## Entire remaining Trellis 0.12 cleanup after this packet

`CLEANUP.md` on `rs` is the durable source of truth. Continue autonomously through all unchecked items; do not stop after the current packet just because it is green.

### A. First public protocol version + TS/WASM boundary (do together)

- Treat the current unreleased request/event proof implementation as the first public v1.
- Remove `AuthorizationRequestProofV2`, `AuthorizationEventProofV2`, V2 input/verified type names, and `*_v2` implementation names; use first-public v1/unversioned implementation names where appropriate.
- Change unreleased `.v2` proof domains to `.v1`; remove unpublished aliases/fallback subjects/dual paths/obsolete parsers, schemas, vectors, and never-released migrations.
- Keep actual serialized format-version fields where they truly identify wire versions.
- Keep Rust protocol authoritative for substantial authorization verification.
- Stop committing protocol-WASM binary/bindgen/base64 output as ordinary source; build/transfer it for TS tests, packaging, and release.
- Keep Rust-only check/test independent of WASM generation.
- Replace per-message `root + manifest + context` JSON round trips with an opaque WASM verified-context handle. Verify trust/context once per cached context; request/event hot paths verify proofs and requirements against that handle.
- Preserve TS provider manifest/revocation invalidation and discard/rebuild handles when trust epoch changes.
- Keep lightweight proof-input construction/signing native in TS.
- Validate with shared Rust/TS conformance vectors and a focused repeated-message test proving one context verification serves many request/event verifications.
- Regenerate final conformance/generated artifacts after the public-v1 clean break.

Current TS/provider fact already established: `AuthorizationProviderCache` caches a verified projection in TypeScript, but request/event verification still rebuilds `root + manifest + context` inputs and sends the trust bundle through WASM on every message. That is the boundary to replace.

### B. Real tests / production-test boundary

- Audit remaining `integration-test-hooks`; delete synthetic errors/faults and retain only genuine deterministic scheduling barriers where necessary.
- Replace avoidable fixed sleeps with observable readiness/state boundaries.
- Remove impossible disconnected/test-only `ConnectedServiceRuntime` states; make connected values valid by construction.
- Remove raw integration-test methods from production facades when tests can use normal behavior.
- Remove `runtime-internals` exposure that exists only for tests.
- Remove `integration-test-scoping` behavior that changes production semantics; retain genuine product concepts only.
- Stop building production runtime binaries with test-fault features after hooks are removed.

### C. Runtime composition / Jobs / constructors

- Split oversized runtime startup / `platform::start` into a few explicit phases with clear ownership; no DI container, generic actor framework, or abstract lifecycle framework.
- Split oversized service runtime facade only along real responsibility boundaries where coupling is reduced.
- Collapse remaining Jobs function/wrapper ladder into one implementation plus a small cohesive execution context/hooks if actually needed.
- Remove duplicate contract metadata from `ServiceConnectOptions` / generated service evidence; one source of truth.
- Apply equivalent constructor simplification to device paths.
- Collapse combinatorial operation-registration callback variants to one provider interface with generated adaptation.

### D. Names and remaining lint causes

- Fix remaining `too_many_arguments` through cohesive inputs/ownership rather than suppressions.
- Fix remaining `result_large_err`, `large_enum_variant`, and test-only `dead_code` causes structurally where simplification is possible.
- Shorten long names using module context and module splitting, not abbreviations.

### E. Test/check/release simplification + final gate

- Finish reducing `release.yml` to release-only packaging/docs/container/platform work; normal correctness belongs in `Check`.
- Confirm simple non-overlapping `test` / `check` / `release` command semantics.
- Finish with one understandable production-semantics live suite using real Trellis/NATS/SQLite behavior and only small pure/real-adapter lower-level tests where appropriate.
- Normal `check`: format + lint/type + tests + generated-files-up-to-date.
- Release: package/docs/container/platform/publication validation only.
- Run final format, Clippy, type/check, TS, and full live integration validation on the completed tree.
- Record fresh cold/warm timing baseline and compare old baseline only as historical context.
- Update user/developer docs for the final public protocol/API shape.
- Delete `CLEANUP.md` only when every tracked cleanup item is actually complete.

## Working principles for all remaining packets

- No backwards-compatibility machinery for behavior that has never been publicly released.
- Prefer deletion and direct code over new frameworks/registries/abstraction layers.
- Channels/ownership are useful where they establish one clear owner; do not actorize normal direct cache reads just to avoid locks.
- Live integration is the primary test style for runtime behavior; unit tests are for small pure invariants or real adapters.
- Test-only code must not alter normal production semantics.
- Avoid broad lint suppressions; fix the design cause when practical.
- Keep packets reviewable and independently green. Land a validated direct `rs` commit, update the tracker, then proceed to the next packet.
- Continue through the entire `CLEANUP.md` scope without asking for routine confirmation. Stop only for a real external blocker or an irreversible/product-direction choice that cannot be inferred from the tracker and repository.
