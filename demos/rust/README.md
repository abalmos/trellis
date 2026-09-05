# Rust Demo

This directory contains independent Rust Field Ops Trellis projects. The service
and device each own a `contract.trellis`, `trellis.toml`, Cargo crate, and
project-local `.trellis` generated state.

## Generate And Check

From the repository root:

```sh
trellis generate --root demos/rust/service
trellis update --root demos/rust/device
cargo check --manifest-path demos/rust/service/Cargo.toml
cargo check --manifest-path demos/rust/device/Cargo.toml
```

Use `trellis generate --watch --root <project>` while editing IDL. Everything
under `.trellis/` is generated or installed state and must not be edited.

## Service

Print the generated participant identity:

```sh
cargo run --manifest-path demos/rust/service/Cargo.toml -- --contract
```

Run with authenticated service bootstrap after creating and provisioning the
service deployment:

```sh
cargo run --manifest-path demos/rust/service/Cargo.toml -- \
  --trellis-url http://localhost:3000 \
  --deployment-id <deployment-id> \
  --seed <instance-seed>
```

The service mounts generated RPC and operation handlers. Authenticated mode uses
the resolved `siteSummaries` KV bucket and `uploads` object store.

## Device

Run the wizard with offline sample data:

```sh
cargo run --manifest-path demos/rust/device/Cargo.toml
```

Run as a provisioned device:

```sh
cargo run --manifest-path demos/rust/device/Cargo.toml -- \
  --device \
  --trellis-url http://localhost:3000 \
  --device-deployment-id <deployment-id> \
  --device-instance-id <instance-id> \
  --device-root-secret <root-secret>
```

The generated participant facade provides the `fieldOps` RPC, operation, event,
transfer, and state APIs used by the device.
