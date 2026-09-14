# WO-04 through WO-06 candidate evidence

This is the implementation and acceptance handoff for WO-04 through WO-06. It
records the exact pre-commit candidate tree and does not claim release
acceptance.

## Candidate

- Accepted WO-03 base: `63f5ebd1ec83b6a858ea9d3f11d9fd39d14cdd1d`.
- Final implementation binary-diff SHA-256, excluding this evidence file:
  `7c92ca540988a8b4285876395592010243bc043bc5754765c5cb7bb0929ad9ed`.
- `cargo xtask install` produced that identical diff on two consecutive runs.
- Publishable Trellis, Result, and Svelte npm trees were rebuilt before package
  consumer validation.
- `workorders/` is protected review input and is not part of the candidate.
- The exact candidate commit and matching `origin/rs` SHA are reported in the
  external handoff because a commit cannot contain its own SHA.

## Coverage

- **WO-04:** authority-bound State/KV/Store resources, resource lifecycle and
  physical validation, companion consent/delegation/session fencing, shared
  companion State, and refresh/restart behavior.
- **WO-05:** generated Jobs and Operation surfaces, durable operation lifecycle,
  cancellation/signals/uploads/replica recovery, keyed Jobs coordination, Events
  projection/query/watch/Consumer/DLQ/replay, exact signed event descriptor
  authorization, and broker-evidence validation.
- **WO-06:** native source-package cutover, generated Rust and TypeScript
  consumers, cursor pagination, CLI and Console surfaces, browser hosting,
  documentation, demos, and removal of superseded Event Log/runtime and
  handwritten contract paths.

## Verification

The following commands passed against the final candidate on 2026-09-13:

| Boundary                         | Command and result                                                                                                                                                             |
| -------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Rust lint                        | `cargo clippy --manifest-path rust/Cargo.toml --workspace --all-targets --all-features -- -D warnings` passed.                                                                 |
| Rust workspace                   | `cargo test --manifest-path rust/Cargo.toml --workspace --all-targets --all-features` passed every target.                                                                     |
| Rust formatting                  | `cargo fmt --manifest-path rust/Cargo.toml --all --check` passed.                                                                                                              |
| TypeScript packages and UI tools | `deno task test` in `ts/` passed, including 196 Trellis tests with 70 nested steps.                                                                                            |
| Public TypeScript surfaces       | Public entrypoint and integration `deno check` commands passed.                                                                                                                |
| TypeScript live integration      | `deno test -A --no-check integration/*_test.ts` passed 20 tests with 15 nested steps in 14m47s.                                                                                |
| Rust live integration            | `cargo test --manifest-path rust/Cargo.toml -p trellis-rs --features live-integration --test integration -- --nocapture` passed 2/2.                                           |
| Console                          | Svelte check reported zero errors and warnings; unit tests and the production build passed.                                                                                    |
| Documentation                    | Svelte check, generated TypeScript API docs, and the production build passed.                                                                                                  |
| Protocol/browser                 | Protocol WASM and embedded web assets built; embedded, configured-directory, and reverse-proxy hosting passed 3/3.                                                             |
| Package consumers                | Isolated staged Node, Deno, and Svelte consumers passed; Rust service and device demos compiled with a command-line crates.io patch to the unpublished candidate `trellis-rs`. |
| Orders example                   | `deno run -A ts/tools/package_build/orders_example.ts` built and passed its real service/caller test without repository imports.                                               |
| Repository integrity             | `git diff --check` passed and no merge conflicts were present.                                                                                                                 |

Focused production-bound suites also passed for companion authority and expiry,
resource transactions, operation cancellation/lease/revision behavior, Jobs
horizons, Events projection and recovery, Consumer replay, stable cursor
windows, outbox conflicts, retained authorization, State CAS, and Store/KV
physical validation.

## Review Status

The candidate incorporates eight correction gates from independent WO-04, WO-05,
WO-06, and adversarial reviews. Per the final handoff direction, further
independent review cycling stops here so the primary designer can perform the
whole-release review.

**READY FOR WHOLE-RELEASE REVIEW - NOT ACCEPTED**

## Whole-release correction candidate

The follow-up to candidate `4997f94a5ffa9f80d271421ddc7eb2b90c8ff5d5` implements
the bounded C1-C7 correction round requested by the primary whole-release
review. It does not replace or reinterpret the verification record above.

- C1-C6 and the C7 CI protocol-WASM dependency correction have focused
  production-bound evidence.
- The generated Runtime workflow passed all eight steps, including exact
  Operation identity and epoch fencing, same-deployment replica updates,
  authored Job policy, and the bounded native-admin revocation scenario.
- Historical Check run #310 remains failed in Install tooling and Live
  integration; it is not evidence for this follow-up tree.
- Complete live acceptance is not green: the focused
  `generated Rust resources use live NATS` case reproducibly leaves stale NATS
  ACLs and resource handles usable after the server has durably revoked the
  service context and completed its post-commit actions.
- A speculative client reconnect-generation gate was removed after it delayed
  unrelated authority and deployment workflows. The unresolved C7b boundary
  requires root-cause review rather than another mitigation.
- `workorders/` remains protected review input and is excluded from the
  follow-up commit.

**REVIEW-BLOCKED CORRECTION CANDIDATE - NOT ACCEPTED**

## Whole-release correction follow-up

This follow-up to correction candidate
`ce88e73259ae471547444e421fa9d64da6ad0cd9` addresses the independent review's
remaining R1-R4 findings and the C7b live failure. It does not replace the
historical evidence above.

- Historical Check #310 remains failed in Install tooling and Live integration.
- Exact-candidate Check #311 (`34759048838`) remains failed because
  `trellis-test-generated` did not match its semantic lock. Dependent lanes were
  skipped and are not treated as green evidence.
- The affected semantic lock was refreshed through normal `trellis update` and
  generated artifacts were rebuilt. Two consecutive `cargo xtask install` passes
  produced the identical tracked binary-diff SHA-256
  `4d3f66faa9a98226fa079c05e6ca641d71b899136ece246c0857874ac3cf4172` before this
  evidence section was appended.
- R1 now derives presentation and companion approval from one current,
  server-owned portal-policy ceiling and carries its expiry and provenance into
  the atomic decision. Repeated approval submits only the current delta and
  retains only server-confirmed eligible authority.
- R2 now fences admission after acknowledged physical presence and makes SDK
  authorization installation candidate-then-promote. Revocation suspends
  resource usability before refresh. A correlated live trace is retained at
  `target/device-companion-auth-flow-trace.log` for follow-up flow analysis.
- R3 requires the immutable fence acquired by a successful local Operation claim
  through executor-held controls; durable rereads cannot mint ownership.
- R4 uses one signed cross-language transient-update envelope with operation,
  API, action, deployment, process executor, signed connection, owner epoch,
  sequence, timestamp, and generated update identity.

The following local verification passed on 2026-09-14:

| Boundary                    | Result                                                                                                                                                                                                    |
| --------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Rust static/workspace       | Formatting passed; warning-denied workspace/all-target Clippy passed; the complete Rust workspace passed.                                                                                                 |
| TypeScript static/packages  | Repository formatting, public package checks, integration checks, and all package/UI-tool tests passed.                                                                                                   |
| TypeScript live integration | Complete matrix passed 20 tests with 14 nested steps in 15m23s. Device.Companion separately passed its two-device policy, State, restart, logout, and sibling-survival case with retained trace evidence. |
| Rust live integration       | Passed 2/2 with prebuilt repository server and CLI binaries.                                                                                                                                              |
| Runtime and Events          | Complete Runtime passed 7/7 with 12 nested steps; complete Events passed 6/6.                                                                                                                             |
| Protocol and browser        | Protocol WASM and embedded applications built; embedded, configured-directory, and reverse-proxy hosting passed 3/3.                                                                                      |
| Package consumers           | Staged Node, Deno, and Svelte consumers passed; Rust service and device demos compiled against the unpublished local `trellis-rs`; packaged Orders passed its real service/caller test.                   |
| Console and docs            | Both Svelte checks reported zero errors and warnings; embedded Console and documentation production builds passed, including generated TypeScript API docs.                                               |
| Repository integrity        | Consecutive generation digests matched; `git diff --check` passed; no merge-conflict markers or temporary print diagnostics remain.                                                                       |

One initial local Rust-live attempt overlapped an embedded-asset replacement and
compiled a stale generated `include_bytes!` list; one browser-hosting attempt
also omitted its required prebuilt-server environment and waited behind that
Cargo lock. Neither is counted as product evidence. After the asset build
completed, the server and CLI were rebuilt once and both exact lanes passed as
recorded above.

`workorders/` remains protected review input and is excluded from the follow-up
commit. This evidence records local execution only; it does not claim an
exact-candidate CI result or release acceptance.

**READY FOR INDEPENDENT WHOLE-RELEASE REVIEW - NOT ACCEPTED**
