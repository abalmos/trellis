# Trellis APIs and Participants

## Status

This document defines the native Trellis contract architecture. A Trellis
contract is authored once and finalized as two protocol artifacts:

- `trellis.api.v1` describes schemas, callable surfaces, machine capability
  allows, and human consent wording.
- `trellis.participant.v1` describes a deployable participant, the exact APIs it
  implements or uses, selected surfaces, resources, jobs, and event consumers.

The two canonical protocol artifacts are `trellis.api.v1` and
`trellis.participant.v1`. There is no combined contract, bundle, catalog, or
runtime-authority artifact.

## Identity

Every normalized API has a stable `lineage@vN` `id`, an independently authored
Semantic Version release `version`, and a semantic `apiDigest`. Release version,
consent wording, and other human metadata are preserved in normalized API JSON
but excluded from the semantic API digest; changing machine permissions changes
the digest. Runtime evidence and permissions pin stable API ID plus semantic
digest, not release version.

Every normalized participant has an `id`, semantic `participantDigest`, and an
authoritatively resolved `participantNeedsDigest`. Participant resolution also
returns required and optional grant sets. Runtime identity and provenance use
the participant digest.

TypeScript defined contracts expose:

- `CONTRACT_ID` — participant id
- `CONTRACT_DIGEST` — semantic participant digest
- `API`, `API_DIGEST`
- `PARTICIPANT`

Rust `ContractArtifacts` exposes the equivalent native objects, digests, and
grant sets without being serializable as a third artifact.

## API artifact

`trellis.api.v1` is normalized and validated by `trellis_protocol`. Subjects are
protocol-derived and are not authoring inputs. Authoring-only selections are
lowered into protocol surfaces and removed from normalized output.

Capabilities are machine policy. Every declared capability is present, even when
its `allows` list is empty. Each allow names an action and a protocol target.
Consent is human-facing wording keyed by capability.

## Participant artifact

`trellis.participant.v1` pins every implemented or used API by exact API id and
digest. Uses select concrete RPC, operation, event, feed, and state surfaces.
Required and optional uses remain distinct. Resources, job queues, and event
consumer declarations belong to the participant.

Resolution requires the participant plus every referenced API artifact. The
authoritative resolver validates pins and selections, normalizes artifacts,
computes participant and needs digests, and derives required and optional
grants.

## Authoring and generation

TypeScript and Rust builders construct native artifacts directly from their
in-memory authoring source. Neither language converts through JSON to recover
runtime descriptors.

Canonical generated JSON lives at:

- `.trellis/generated/protocol/apis/<api-id>.json`
- `.trellis/generated/protocol/participants/<participant-id>.json`

Generated API modules expose API identity and native API evidence. Generated
participant/service modules expose participant identity, native participant
JSON, needs digest, owned API evidence, and exact referenced API evidence.

## Runtime presentation

Clients, devices, and services resolve the normalized participant with its exact
owned and referenced APIs before presentation. The runtime boundary compares the
resolved API and participant digests with TypeScript's intrinsic identities and
supplies the contextual needs digest. Bootstrap validates this evidence
directly. There is no retry state for requesting another artifact shape.

State declarations, writers, migrations, and admin views use the resolved
participant binding's artifact digest. Compatible participant digest changes do
not change the State namespace; namespace identity remains participant id,
scope, owner, store, and state version.

## Conformance

Cross-language conformance covers representative native authoring for minimal
apps, RPC, operations and signals, events and feeds, State, Jobs, KV and store,
required and optional uses, event consumers, capability and consent, transfers,
devices, and agents. Each vector asserts normalized API and participant JSON,
API and participant digests, needs digest, and required and optional grants.

Protocol validation, contextual participant resolution, and grant derivation are
authoritative in `trellis_protocol`. TypeScript computes intrinsic API and
participant digests without WASM, then runtime paths use the narrow
protocol-WASM boundary for contextual resolution.
