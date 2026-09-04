# Generated Rust Libraries

## Purpose

`trellis-protocol` owns canonical artifact parsing, normalization, validation,
semantic digests, participant resolution, and grant derivation. Generated Rust
crates use private render projections and do not create a second contract model.

Trellis IDL is the only authoring input. Rust code generation parses validated
canonical API and participant artifacts through `trellis-protocol` and projects
them into typed SDK and participant crates.

## Generation

Canonical outputs are:

- `.trellis/artifacts/apis/<api-id>.json`
- `.trellis/artifacts/participants/<participant-id>.json`

Generated API crates expose canonical API constants. Generated participant
facades expose participant identity and digest, `PARTICIPANT_NEEDS_DIGEST`,
canonical participant data, owned API evidence, and referenced API evidence.
Generated service and device connection paths consume that exact evidence
through `GeneratedServiceParticipant` rather than asking callers to repeat it.
Generated device connection options also do not accept a session seed: each
activation/bootstrap attempt owns a fresh key, and only a ready attempt carries
its successful private seed into connection. Run `cargo xtask install` after
source changes; generated files are not authoring inputs.
