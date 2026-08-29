# Rust Contract Libraries

## Purpose

Rust authors build the same native API and participant artifacts as TypeScript.
`trellis-contracts` supplies authoring builders while `trellis-protocol` owns
validation, parsing, normalization, digesting, resolution, and grants.

## Builders

`ApiBuilder` constructs a native API authoring source. Finalization calls
`lint_api_authoring` and `parse_api`. `ContractBuilder` constructs or accepts a
native participant, validates it with `lint_participant_authoring` and
`parse_participant`, and resolves it with `resolve_participant` against exact
referenced APIs.

Both authoring constructors require the owned API's independent Semantic Version
release value. The release version is normalized into the API artifact but is
excluded from its semantic digest and participant runtime evidence.

`ContractArtifacts` is intentionally non-serializable. It returns native API and
participant objects, API and participant digests, participant-needs digest, and
required and optional grants. It is not a third wire format.

Built-in Rust APIs and demo contracts use these builders directly. Generated
service and device facades implement `GeneratedServiceContract`; its participant
JSON/digests, owned API JSON/digest, and exact referenced API JSON/digest pairs
are the sole contract-evidence source for bootstrap.

## Generation

Canonical outputs are:

- `generated/protocol/apis/<api-id>.json`
- `generated/protocol/participants/<participant-id>.json`

Generated API crates expose native API constants. Generated participant facades
expose `CONTRACT_ID`, `CONTRACT_DIGEST`, participant JSON,
`PARTICIPANT_NEEDS_DIGEST`, owned API evidence, and referenced API evidence.
Generated service and device connection paths consume that exact evidence
through `GeneratedServiceContract` rather than asking callers to repeat it.
Generated device connection options also do not accept a session seed: each
activation/bootstrap attempt owns a fresh key, and only a ready attempt carries
its successful private seed into connection. Run `cargo xtask prepare` after
source changes; generated files are not authoring inputs.

## Conformance

Rust tests consume the same representative vectors as TypeScript and assert
normalized/canonical API and participant JSON, all three semantic digests, and
required and optional grants. This guards native authoring parity without an
adapter or duplicate protocol DTO.
