# Trellis Integration

This template covers Trellis integration in a mixed-language project. Add the
project's own policies separately. Trellis does not choose application config,
validation libraries, databases, dependency injection, or general coding rules.

## Documentation

Use Trellis documentation matching the installed CLI and libraries. With a local
checkout, read `docs/static/llms.txt` and the relevant language guide there.
Otherwise use those files from the matching release revision, not automatically
from `main`. Public API documentation is linked from the Trellis docs site's
`/api`; generated source defines this project's exact participant methods.

## Trellis Boundary

- Author `contract.trellis` or `contracts/*.trellis`; declare API dependencies
  in `trellis.toml` and commit the resolved `trellis.lock`.
- Run `trellis update` after dependency changes, `trellis install` to reproduce
  locked dependencies, and `trellis generate` after IDL edits.
- Set `format = 1` and a local package `name` in `trellis.toml`. Configure
  explicit `[generate.rust].output` and `[generate.typescript].output`
  destinations for a mixed-language project. Each language gets one package at
  version `0.0.0`, with `apis` and `participants` namespaces, not per-surface
  packages.
- Commit generated packages when ordinary clones must build without the CLI or
  API cache; regenerate instead of hand-editing them. Runtime dependencies use
  published packages; local substitutions belong in ordinary ecosystem
  overrides.
- Connect through generated participants and supported runtime APIs. Trellis
  owns its proofs, transport metadata, and resolved resource bindings.
- TypeScript services use `TrellisService.connect(...)` and generated flat
  caller/provider methods. Rust services connect through their generated Cargo
  participant facade.
- Trellis operations are caller-visible workflows; Trellis jobs are private
  execution. Authority approval is separate from declaring either surface.
- A direct Trellis publish is not atomic with an application SQL transaction.
  Use the SQL outbox integration when that atomicity is needed.

## Project Details

Fill in the relevant entries; omit those this project does not use.

- Trellis version or local checkout:
- Contract source:
- Generation command:
- Check/test commands:
- Local run command:
- Application-specific policies:
