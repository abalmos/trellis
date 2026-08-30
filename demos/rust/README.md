# Rust Demo

This workspace contains Rust Field Ops demo participants that are kept separate
from the TypeScript demo so they resemble out-of-tree Rust consumers.

- `contracts/service.rs`: Rust-authored service contract manifest.
- `contracts/device.rs`: Rust-authored device contract manifest.
- `.trellis/generated/packages/cargo/demo-service`: generated Rust demo service
  SDK.
- `service`: Rust Field Ops service.
- `device`: Rust field-device wizard CLI.

The Rust and TypeScript demo contracts are expected to produce the same
canonical service/device manifests and digests. The parity test lives in
`rust/tools/generate/tests/demo_contract_parity_test.rs`.

## Install

From the repository root:

```sh
cargo xtask install
cargo test --manifest-path rust/Cargo.toml -p trellis-generation --test demo_contract_parity_test
cargo test --manifest-path demos/rust/Cargo.toml --workspace
```

## Service

Print the generated service contract identity:

```sh
cargo run --manifest-path demos/rust/Cargo.toml -p trellis-rust-demo-service -- --contract
```

Run with authenticated Trellis service bootstrap after the service deployment is
created and provisioned. Deployment authority can be updated before startup, or
the service can present its manifest during bootstrap and wait while the
resulting authority update is approved:

```sh
cargo run --manifest-path demos/rust/Cargo.toml -p trellis-rust-demo-service -- \
  --trellis-url http://localhost:3000 \
  --deployment-id <deployment-id> \
  --seed <instance-seed>
```

Enable request, operation, job, and transfer diagnostics with `RUST_LOG`:

```sh
RUST_LOG=trellis_rust_demo_service=debug,trellis_service=debug,trellis_jobs=debug \
  cargo run --manifest-path demos/rust/Cargo.toml -p trellis-rust-demo-service -- \
  --trellis-url http://localhost:3000 \
  --deployment-id <deployment-id> \
  --seed <instance-seed>
```

Use the `deploymentId` and `instanceSeed` fields from
`trellis --format json deploy provision
svc/demo.field-ops` as `<deployment-id>`
and `<instance-seed>`.

Authenticated mode does not need `--nats-url`; Trellis returns the runtime NATS
servers during bootstrap. The authenticated service opens the resolved
`siteSummaries` KV bucket for site summaries and the resolved `uploads` object
store for evidence bytes. Without bootstrap arguments, the service exits after
confirming which run modes are available.

## Device

Run the wizard with offline sample data:

```sh
cargo run --manifest-path demos/rust/Cargo.toml -p trellis-rust-demo-device
```

Run as a preregistered device through the generated device bootstrap path:

```sh
cargo run --manifest-path demos/rust/Cargo.toml -p trellis-rust-demo-device -- \
  --device \
  --trellis-url http://localhost:3000 \
  --device-deployment-id <deployment-id> \
  --device-instance-id <instance-id> \
  --device-root-secret <root-secret>
```

Use the deployment id, instance id, and one-time `rootSecret` returned by
`trellis --format json deploy provision <device-deployment-id>`. The client
derives the provisioned identity and obtains current NATS routing and
authorization material from `POST /bootstrap/device`; it does not persist
sentinel credentials or runtime topology.

The Rust device CLI uses the generated participant facade for online `fieldOps`
RPCs and operations, generated state helpers for `selectedSite` and
`draftInspections`, generated transfer helpers for evidence upload/download, and
generated event-subscription helpers for service events.

## Event Outbox/Inbox Coverage

Prepared outbox dispatch and inbox duplicate suppression are covered by the
integration harness for the Rust runtime. The Field Ops service's normal demo
flows publish events directly; they are not a production persisted-outbox
example.

## Current Gaps

- live authenticated service/device smoke coverage against a running Trellis
  stack
- live verification of worker-host queue consumption for service-private jobs;
  authenticated mode starts a `refreshSiteSummary` worker host when the jobs
  work stream and `siteSummaries` KV binding are available, while raw
  local/tests keep the synchronous inline path
- reusable public `TrellisDevice.connect(...)`-style persistence abstraction;
  the current root-secret persistence is demo-local
