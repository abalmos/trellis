# TypeScript Contract Authoring

## Purpose

TypeScript contract modules provide typed authoring ergonomics and finalize
directly to native `trellis.api.v1` and `trellis.participant.v1` artifacts. See
[Trellis APIs and Participants](./trellis-api-participants.md).

## Defined contracts

Use `defineServiceContract`, `defineAppContract`, `defineDeviceContract`, or the
matching participant-kind helper. The callback returns authoring source:
schemas, owned surfaces, selected dependency actions, resources, jobs, event
consumers, capabilities, and consent wording.

Every source explicitly supplies independent `id`, `apiId`, and `apiVersion`
fields. `apiVersion` is Semantic Version release metadata for the owned API; it
is not encoded into `apiId` and does not affect the semantic API digest.

The result preserves typed action and runtime descriptors and exposes exactly:

- `CONTRACT_ID`
- `CONTRACT_DIGEST`
- `API`
- `API_DIGEST`
- `PARTICIPANT`

`CONTRACT_ID` is the participant id and `CONTRACT_DIGEST` is the semantic
participant digest. TypeScript computes both intrinsic digests without WASM. The
object does not expose a serialized combined artifact or an eager contextual
needs digest.

## Native construction

The authoring source is lowered directly into an API and participant. Protocol
subjects are derived, authoring-only fields are removed, and selected actions
pin exact dependency API artifacts and digests. Runtime descriptors remain the
original typed descriptors; native JSON is never parsed back to reconstruct
them.

Capability selections become `api.capabilities[*].allows`. Every declared
capability exists even when unused. Human wording becomes API consent. Wording
changes preserve `API_DIGEST`; permission changes do not.

Contract authoring is WASM-free. Runtime bootstrap and presentation pass the
participant and exact referenced APIs to the narrow protocol-WASM resolution
boundary. That boundary compares Rust's API and participant digests with the
TypeScript intrinsic identities, then returns the contextual needs digest and
grant derivation.

## Generated packages

Generated API modules expose `API_ID`, `API`, and `API_DIGEST`; `API_JSON` is
emitted only for a consumer that requires static JSON. Generated participant
modules expose contract/participant identity and exact owned and referenced API
evidence. Action sources always carry the exact native API artifact and digest.

Canonical JSON is generated under `.trellis/generated/protocol/apis` and
`.trellis/generated/protocol/participants`. Edit source modules and run
`trellis install`; never edit generated files.

## Runtime presentation

Clients, devices, browser auth starts, and services resolve `PARTICIPANT`, the
owned API, and all exact referenced APIs at runtime before presenting the
resulting contextual needs digest. There is no eager contextual digest on the
authored contract.
