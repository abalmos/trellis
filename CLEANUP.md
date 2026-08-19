# Trellis 0.12 simplification

Short-lived tracker for the `rs` cleanup before the first stable release. Delete this file when the work is complete.

- [ ] **Fast feedback** — make live-test concurrency machine-adaptive; simplify `test`, `check`, and `release`; remove redundant live inventory/governance work; record a current timing baseline.
- [ ] **Real tests only** — remove synthetic fault injection and production test hooks; use real bad inputs, real process/NATS failures, or narrow real-SQLite integration tests where failure behavior matters.
- [ ] **First-release protocol** — remove unpublished compatibility/history; make the current wire formats the first public versions; stop carrying `V1`/`V2` in ordinary implementation names when only one version exists.
- [ ] **Clear ownership** — fix durable listener registration/cleanup; give runtime leases single task ownership; simplify authorization cache consistency and Jobs references without introducing a generic actor framework.
- [ ] **Smaller public/internal APIs** — remove invalid test-only runtime states, collapse combinatorial callback APIs and redundant constructor inputs, shorten names through module context, and fix Clippy causes instead of suppressing them.
- [ ] **Final gate** — run one understandable production-semantics live suite plus the small pure checks and release-only packaging/docs checks; update docs and delete this tracker.

## Working rules

- No backwards-compatibility work for unreleased Trellis behavior.
- Prefer deletion and direct code over new frameworks or registries.
- Live integration remains the primary test style for runtime behavior.
- Unit tests stay for small pure invariants or real adapter behavior that does not require the full control plane.
- Test-only code must not change normal production semantics.
