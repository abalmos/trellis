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
