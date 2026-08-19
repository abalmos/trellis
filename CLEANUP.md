# Trellis 0.12 simplification

Short-lived tracker for the `rs` cleanup before the first stable release. Delete this file when the work is complete.

- [ ] **Fast feedback** — make live-test concurrency machine-adaptive; simplify `test`, `check`, and `release`; remove redundant live inventory/governance work; record a current timing baseline.
- [ ] **Real tests only** — remove synthetic fault injection and production test hooks; use real bad inputs, real process/NATS failures, or narrow real-SQLite integration tests where failure behavior matters.
- [ ] **First-release protocol** — remove unpublished compatibility/history; make the current wire formats the first public versions; stop carrying `V1`/`V2` in ordinary implementation names when only one version exists.
- [ ] **Clear ownership** — fix durable listener registration/cleanup; keep runtime leases single-owner; simplify authorization cache consistency and Jobs references without introducing a generic actor framework.
- [ ] **Runtime composition** — split oversized startup/facade responsibilities where one module currently owns unrelated lifecycle, routing, resource, and bootstrap work; do not add a DI/framework layer.
- [ ] **Production/test boundary** — remove invalid test-only runtime states, `runtime-internals` leakage, integration-test scoping that changes normal semantics, and raw test methods on production facades.
- [ ] **Jobs and constructors** — collapse the Jobs wrapper ladder and callback machinery; remove redundant generated-contract inputs from long service/device constructors.
- [ ] **Names and lints** — shorten names through module context; fix repeated `too_many_arguments`, `result_large_err`, `module_inception`, and test-only `dead_code` causes instead of expanding Clippy suppressions.
- [ ] **Final gate** — run one understandable production-semantics live suite plus the small pure checks and release-only packaging/docs checks; update docs and delete this tracker.

## Working rules

- No backwards-compatibility work for unreleased Trellis behavior.
- Prefer deletion and direct code over new frameworks or registries.
- Live integration remains the primary test style for runtime behavior.
- Unit tests stay for small pure invariants or real adapter behavior that does not require the full control plane.
- Test-only code must not change normal production semantics.
