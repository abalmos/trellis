# @qlever-llc/trellis

JavaScript Trellis client runtime. Provides generated-participant client helpers
and runtime error types.

For AI-agent context, start with the generated package `TRELLIS.md` files and
the raw docs index:

- https://raw.githubusercontent.com/qlever-llc/trellis/main/docs/static/llms.txt
- https://raw.githubusercontent.com/qlever-llc/trellis/main/docs/static/llms-full.txt

```typescript
import { TrellisClient } from "@qlever-llc/trellis";
import { participants } from "example-trellis";

const client = await TrellisClient.connect({
  trellisUrl: "https://trellis.example.com",
  participant: participants.exampleApp.participant,
}).orThrow();
const meResult = await client.authSessionsMe({});
const me = meResult.orThrow();
```

The local package name comes from `trellis.toml`. Generation emits one ordinary
ESM package with `apis` and `participants` namespaces, executable JavaScript and
declarations, and the matching published runtime dependency. Commit that package
when ordinary builds should not require the Trellis CLI or API cache.

Generated participants expose flat, typed methods for their declared surfaces.
Inspect their declarations for exact caller, provider, event, feed, and
operation names. Do not reconstruct transport subjects or use handwritten
contract metadata.

Prepared events support durable publish flows. `prepare(...)` returns a
`PreparedTrellisEvent`; services can persist prepared events in SQL or NATS KV
outbox repositories and later publish them with `client.publishPrepared(...)`,
dispatch them with `dispatchOutbox`, or run an `OutboxDispatcher` and call
`notify()` after an outbox transaction commits.

Durable service event consumption is contract-declared. Add an event consumer
group to the service contract and call the generated listener with
`{ group: "groupName" }`. Do not pass `durableName`; Trellis provisions the
physical JetStream consumer and grants only the bound consumer subjects to the
service token. Use `{ mode: "ephemeral", replay: "new" }` for live-only
listeners.

Service connection helpers live in `@qlever-llc/trellis/service*` to keep the
root package browser-safe. Browser login and portal-flow helpers live on
`@qlever-llc/trellis/auth` and `@qlever-llc/trellis/auth/browser`.

Service authors should not use the core package to recreate service bootstrap or
fetch resource bindings. Connect with `TrellisService.connect(...)` from
`@qlever-llc/trellis/service/deno` or `@qlever-llc/trellis/service/node` and use
the returned resource handles instead of calling `Trellis.Bindings.Get`,
constructing `TrellisService` or `StoreHandle`, or passing binding/resource data
into `Trellis` constructors.
