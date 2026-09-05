# Trellis Rust Integration

This template covers Trellis integration, not the application's configuration,
database, DI, code organization, logging, or general development policy.

## Documentation

Use `docs/static/llms.txt` and `llms-rust.txt` from the Trellis revision
matching this project's CLI and libraries. With a local checkout, read those
files there. Use the docs site's `/api` for current Rustdoc and generated crate
source for the exact participant-specific methods.

## Trellis Boundary

- Author native IDL and run `trellis generate` before compiling generated
  participant consumers. Use `trellis update` for dependency changes and
  `trellis install` to reproduce `trellis.lock`.
- Commit authored files and lock; regenerate disposable `.trellis/` crates.
- Connect through the generated participant facade. API SDK crates supply
  vocabulary, not independent runtime connections.
- Register owned providers through `handle()` before running the service
  lifecycle. Use generated callers, publishers, event-consumer groups, and
  resource handles for Trellis communication.
- Trellis owns its proofs, transport subjects, credentials, and binding
  resolution. Private Trellis workspace crates are not application dependencies.
- Long-lived clients need durable authorization-context storage. Use the
  supported store types documented by the matching runtime release.
- Direct publication is not atomic with application SQL state; use the outbox
  transaction integration when that atomicity is needed.
- `TrellisTestRuntime` is a Deno test helper that can launch Rust processes, not
  an exported Rust runtime harness.

## Project Details

- Trellis version or local checkout:
- Contract source:
- Generation command:
- Check/test commands:
- Local run command:
- Application-specific policies:
