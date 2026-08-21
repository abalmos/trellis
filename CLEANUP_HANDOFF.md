# Trellis 0.12 cleanup handoff

Read this file **after `CLEANUP.md`**. `CLEANUP.md` is the durable overall scope; this file records the exact in-progress packet so a new ChatGPT session can resume without conversation history.

This file is short-lived. Keep it current while a packet is in progress and delete it together with `CLEANUP.md` when the cleanup is complete.

## Repository state

- Repository: `abalmos/trellis`
- Working branch: `rs`
- Frozen/current `rs` baseline for the active packet: `fa723c49bc69f927bc222702756d2124151fa962` (`Track landed typed-error cleanup`)
- Do not write to `Qlever-LLC/trellis`.
- Cleanup work is committed directly to `rs` only after validation. Hidden `agent/*` branches are allowed for Actions validation. Do not create cleanup PRs unless explicitly requested.
- Re-read the current `rs` SHA before every write and never overwrite concurrent work.

## Active packet: generated-SDK / WASM bootstrap decoupling

The current product transform is intentionally **not yet on `rs`**. It is validated from the frozen `fa723c49` baseline on hidden branch infrastructure.

Primary validation branch/workflow:

- branch: `agent/validate-runtime-sdk-decouple-v3`
- workflow: `.github/workflows/agent-runtime-sdk-decouple-v3.yml`
- workflow `BASE_SHA`: `fa723c49bc69f927bc222702756d2124151fa962`
- latest recorded failed run: `32432938671`
- latest failure marker branch: `agent/failed-runtime-sdk-decouple-v3`

### Product changes in the active transform

The transform removes the Rust workspace/runtime bootstrap dependency on generated SDK crates, especially `trellis-sdk-auth`:

- root `rust/Cargo.toml` no longer declares unused generated SDK path dependencies;
- `trellis-runtime` no longer depends on `trellis-sdk-auth` merely to register Auth RPC permission metadata;
- Auth runtime metadata is derived from the canonical checked-in API artifact instead of a generated Rust SDK;
- shared capability-name lookup is moved to the appropriate protocol/contracts surface instead of duplicated in Rust and TS codegen;
- `rust/Cargo.lock` is included so removing generated SDK dependencies cannot leave a stale lockfile.

The validator also currently carries the remaining Rust 1.98 error-size cleanup exposed by the broader `-D warnings` gate (not a suppression workaround):

- shrink additional cold-path `ServerError` diagnostic payloads;
- remove an Event Log error wrapper that unnecessarily carried the full JetStream message in an error tuple;
- keep the already-landed typed-error cleanup on `rs` (`65dae41f`) as the baseline rather than reintroducing those suppressions.

### Gates already proven for this packet

From the frozen `fa723c49` baseline, the current product transform has already proven all of the following in Actions:

- `trellis-protocol-wasm` builds with the entire `generated/` tree removed;
- restoring the tracked generated baseline and regenerating SDKs produces zero generated diff;
- the embedded login portal rebuilds successfully;
- the affected Rust crates pass aggregate `cargo check --all-targets`;
- the Auth runtime no longer requires `trellis-sdk-auth` for runtime metadata registration;
- Cargo lockfile handling is included in the product diff.

The packet remains unlanded because the broader Rust 1.98 Clippy gate exposed remaining pre-existing `result_large_err` / error-envelope debt. The main causes already identified and structurally addressed were oversized `ServerError` cold-path diagnostic variants and Event Log carrying a ~440-byte error tuple. The latest validation still had one remaining Clippy diagnostic being isolated when the prior session stopped.

### Exact next action

1. Re-read `rs` and require the intended base to still be `fa723c49`; if `rs` moved, rebase/reconstruct the product transform on the new head before doing anything else.
2. Inspect Actions run `32432938671` / the latest `agent/failed-runtime-sdk-decouple-v3` state and isolate the **last remaining Clippy diagnostic**. Do not add an `allow`/`expect` merely to silence it; fix the cause if the design can be made smaller.
3. Keep validation-product changes separate from hidden validation workflow/scripts. The final product commit must contain only real repository source/config/lockfile changes.
4. Rerun the complete frozen gate:
   - apply product transform;
   - `cargo fmt` / `git diff --check`;
   - delete `generated/` and prove `trellis-protocol-wasm` builds before generated SDKs exist;
   - restore/regenerate SDKs and require zero generated diff;
   - build embedded login portal;
   - aggregate affected-crate `cargo check --all-targets`;
   - affected-crate Clippy with `-D warnings`;
   - affected Rust library tests;
   - assertions that generated SDK path/runtime dependencies are actually gone.
5. Inspect the resulting **product-only diff** carefully. Ensure no `.validation-*`, `agent_*` helper script, hidden workflow, generated build output, or unrelated source is included.
6. Fast-forward/graft the validated product commit onto `rs` **only if `rs` still has the expected parent/head**. Never force-overwrite concurrent work.
7. Immediately update `CLEANUP.md` and this handoff with the landed commit SHA and validation run.

## WASM/protocol work immediately after this packet

Do not lose the broader reason for the SDK-decoupling prerequisite. The agreed direction is:

- Rust protocol remains the single authoritative implementation for substantial protocol/security verification; do not reimplement the trust/context verifier independently in TS.
- WASM becomes a generated TS/package artifact instead of ordinary committed source state.
- Rust-only check/test paths should not build protocol WASM.
- Contract/SDK preparation and protocol-WASM generation must work from a clean checkout without a circular dependency on generated Rust SDKs.
- The TS authorization hot path must stop sending root + manifest + full context through WASM for every message.
- Verify the trust root/manifest/context once and retain an opaque verified-context handle in WASM; request/event verification then uses that verified state.
- Invalidate/free handles when context expiry, revocation, manifest/trust-floor changes, or provider cache lifecycle invalidates the underlying context.
- Pass payload bytes across the JS/WASM boundary directly (`Uint8Array`/`&[u8]`), not JSON arrays of numbers.
- Keep simple proof-input construction/signing native in TS.
- Remove committed `.wasm`, wasm-bindgen JS/d.ts, and base64 binary copies when package/build ownership is proven; package tests must explicitly require the generated WASM payload to be present in ESM/CJS publication output.
- Do not duplicate WASM build work between generic preparation, Rust validation, TS checks, and release packaging. Build it only where the TS/package/runtime boundary requires it.
- Complete the first-public proof cleanup in the same final boundary transition: current unreleased request/event proof V2 becomes first public wire `v1`, implementation APIs become unversioned, deterministic proof vectors are recomputed, and zero unreleased V2 proof history remains.

Important discovery from the WASM investigation: TS contract authoring/generation currently synchronously uses Rust/WASM participant resolution. Do not casually replace that with an independent TS resolver. Preserve one authoritative Rust protocol implementation while removing the bootstrap/build coupling.

## Entire remaining cleanup scope

After the active SDK/WASM/proof packet, continue every unchecked item in `CLEANUP.md` until the file can be deleted. In particular:

1. audit remaining `integration-test-hooks`, delete semantic overrides/synthetic failures, retain only genuine deterministic race barriers;
2. remove `integration-test-scoping`, raw production-facade test bypasses, `runtime-internals`, and impossible disconnected runtime states;
3. simplify Jobs wrapper ladders, service/device connection constructors, duplicate generated contract evidence, and operation-registration callback variants;
4. split oversized runtime startup/service facade responsibilities only where there are real ownership boundaries; no DI/actor/lifecycle framework;
5. finish remaining `too_many_arguments`, `result_large_err`, `large_enum_variant`, and test-only `dead_code` causes without broad lint suppression;
6. reduce `release.yml` to release-only work and establish the final simple `test` / `check` / `release` split;
7. run the complete production-semantics live suite, final format/Clippy/type/package/docs gates, and record fresh cold/warm timing baselines;
8. update public/developer documentation for the final first-release API/protocol shape;
9. delete `CLEANUP.md` and this handoff only when every cleanup item is complete.
