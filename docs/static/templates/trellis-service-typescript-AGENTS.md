# Trellis TypeScript Integration

This template covers Trellis, not general application policy. The project owns
configuration parsing, validation of local inputs, database/ORM choice, DI,
layout, testing tools, and coding conventions. No Zod or database library is
required by Trellis.

## Documentation

Use `docs/static/llms.txt` and `llms-typescript.txt` from the Trellis revision
matching this project's CLI and libraries. With a local checkout, read those
files there. Use `llms-svelte.txt` if this project uses the Svelte integration.
Current public APIs are linked from `/api` on the Trellis docs site.

## Trellis Boundary

- Author native IDL and use `trellis update`, `trellis install`, and
  `trellis generate` for dependency resolution, reproduction, and generation.
- Commit IDL, `trellis.toml`, and `trellis.lock`; regenerate `.trellis/` output.
- Services connect with `TrellisService` from `/service/deno` or `/service/node`
  and their generated participant. Generated flat `handle...`, `publish...`,
  `on...`, and caller methods depend on the declared actions.
- Use resolved `service.kv`, `service.store`, and `service.jobs` handles for
  Trellis resources, not handwritten binding or transport payloads.
- Handlers receive Trellis-owned arguments. Application dependencies can be
  captured in closures, including closures constructed by the project's DI.
- Declare Trellis wire models and business errors in IDL. Local models and
  validation libraries are independent application choices.
- Use a durable authorization-context store for a long-lived service. The
  browser client owns its browser auth installation and normal `logout()`.
- SQL outbox integration requires an executor and transaction runner; the
  application owns the database and applies the supplied helper migrations.

## Project Details

- Trellis version or local checkout:
- Contract source:
- Generation command:
- Check/test commands:
- Local run command:
- Application-specific policies:
