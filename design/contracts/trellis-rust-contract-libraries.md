# Generated Rust Libraries

## Purpose

`trellis-protocol` owns canonical artifact parsing, normalization, validation,
semantic digests, participant resolution, and grant derivation. Generated Rust
crates use private render projections and do not create a second contract model.

Trellis IDL is the only authoring input. Rust code generation consumes validated
canonical API and participant values and projects them into modules in one
crate.

## Generation

The configured Rust output is one ordinary path-dependency crate, named by
`trellis.toml` and always version `0.0.0`. It contains `apis::<api>` and
`participants::<participant>` modules, not independently versioned SDK or facade
crates. It declares the published Trellis runtime dependency corresponding to
the generator version; consumer-side Cargo overrides own any local development
substitution. No CLI, API cache, or generation step is needed for an ordinary
Cargo build of committed output.

Tooling JSON inside that crate is:

- `artifacts/apis/<api-id>.json`
- `artifacts/participants/<participant-id>.json`

Generated API modules expose canonical API constants. Generated participant
facades expose participant identity and digest, `PARTICIPANT_NEEDS_DIGEST`,
canonical participant data, owned API evidence, and referenced API evidence.
Generated service and device connection paths consume that exact evidence
through `GeneratedServiceParticipant` rather than asking callers to repeat it.
Generated device connection options also do not accept a session seed: each
activation/bootstrap attempt owns a fresh key, and only a ready attempt carries
its successful private seed into connection. Run `cargo xtask install` after
source changes; generated files are not authoring inputs.
