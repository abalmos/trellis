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

The result preserves typed action and runtime descriptors and exposes exactly:

- `CONTRACT_ID`
- `CONTRACT_DIGEST`
- `API`
- `API_DIGEST`
- `PARTICIPANT`
- `PARTICIPANT_NEEDS_DIGEST`

`CONTRACT_ID` is the participant id and `CONTRACT_DIGEST` is the semantic
participant digest. The object does not expose a serialized combined artifact.

## Native construction

The authoring source is lowered directly into an API and participant. Protocol
subjects are derived, authoring-only fields are removed, and selected actions
pin exact dependency API artifacts and digests. Runtime descriptors remain the
original typed descriptors; native JSON is never parsed back to reconstruct
them.

Capability selections become `api.capabilities[*].allows`. Every declared
capability exists even when unused. Human wording becomes API consent. Wording
changes preserve `API_DIGEST`; permission changes do not.

Participant resolution and grant derivation use the authoritative
`trellis_protocol` resolver through the lazy protocol WASM bridge. Importing a
descriptor has no WASM initialization side effect; defining a contract performs
synchronous lazy initialization in server and browser environments.

## Generated packages

Generated API modules expose `API_ID`, `API`, and `API_DIGEST`; `API_JSON` is
emitted only for a consumer that requires static JSON. Generated participant
modules expose contract/participant identity and exact owned and referenced API
evidence. Action sources always carry the exact native API artifact and digest.

Canonical JSON is generated under `generated/protocol/apis` and
`generated/protocol/participants`. Edit source modules and run
`deno task
prepare`; never edit generated files.

## Runtime presentation

Clients, devices, browser auth starts, and services present `PARTICIPANT`, the
owned API, all exact referenced APIs, `CONTRACT_DIGEST`, and
`PARTICIPANT_NEEDS_DIGEST`. There is no conversion or alternate bootstrap shape.
