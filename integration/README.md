# Integration Matrices

`client-test-matrix.json` is the cross-language client-interoperability
contract. `rust-runtime-test-matrix.json` tracks Rust-owned runtime behavior,
including explicit pending requirements.

## Matrix Rules

- Case IDs are stable semantic IDs in the form `<fixture>.<case>`.
- Implemented mappings name one real live test; ignored, unit, descriptor-only,
  and mock-only tests do not satisfy a row.
- TypeScript mappings identify Deno integration case IDs.
- Rust mappings identify exact compiled `libtest` names.
- Implemented Rust mappings must equal both the compiled executable inventory
  and the executed result stream exactly.
- Pending Rust runtime rows record their current owner or blocker. They are not
  hidden skips.
- Process isolation requires a recorded process-global reason. Every case owns a
  NATS account, Trellis process, and SQLite files regardless of classification.

## Adding A Case

1. Add the requirement to the smallest applicable matrix.
2. Add the live implementation and exact mapping when it is executable.
3. Otherwise leave the Rust runtime requirement pending with its real blocker.
4. Run the language runner and matrix conformance checks.

Removing or renaming a case changes the test contract and must be called out in
review.

## Focused Runs

TypeScript cases are selected from the client matrix:

```sh
deno task -c ts/deno.json test:integration -- --fixture rpc
deno task -c ts/deno.json test:integration -- --case rpc.client-calls-service-success
```

Rust uses exact `libtest` filters through the case-owned runtime runner:

```sh
deno run -A -c ts/deno.json integration/live_runner.ts --rust-only --rust-filter rpc::
```

The complete local live schedule is:

```sh
deno run -A -c ts/deno.json integration/live_runner.ts
```

The normal `Check` workflow is the authoritative full-suite gate. Release
packaging does not rerun live correctness tests.
