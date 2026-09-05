---
title: Type System Patterns
description: Trellis wire models, validation, generated types, and error semantics.
order: 30
---

# Design: Type System Patterns

## Scope

This document describes Trellis-owned wire semantics and generated runtime
types. It does not prescribe validation libraries, identifier schemes, database
schemas, or typing conventions for application-local code. Contributor rules for
the Trellis repository live in its `AGENTS.md`.

## API Schema

Native IDL is the authoring source. The compiler emits canonical
`trellis.api.v1` and `trellis.participant.v1` artifacts, and code generators
produce language-specific types and runtime validation schemas.

```trellis
api "acme.users@v1" {
  version "1.0.0";
  display_name "Users";
  description "Example user lookup.";
  model FindUser { id: string; }
  model User { id: string; }
  error UserNotFound;
  rpc "Users.Find" {
    version "v1";
    input FindUser;
    output User;
    errors [UserNotFound];
  }
}
```

Public runtime types follow those declarations. Editing generated TypeBox
schemas, Rust structs, or canonical JSON is not an authoring workflow.
Dependency selections belong to participant `use` blocks. See
[native IDL](../contracts/trellis-idl.md) for compilation and dependency
semantics.

## Schema Organization

The compiler discovers `contract.trellis` or direct-child `contracts/*.trellis`
files. It resolves models within the owning API before generating SDKs. The
application decides how to organize its other source files and local models.
Trellis-owned schemas live in this repository; domain APIs live with their
owner.

Event payload models contain application data. Event IDs, timestamps, proofs,
and correlation metadata are supplied separately by the runtime. Duplicate
fields in the body do not replace authenticated runtime metadata.

## List Pagination Schemas

Trellis-owned list endpoints and resource APIs document their page shapes and
bounds. These are not a requirement for all application-owned list RPCs. An
application can use the optional pagination helpers or declare another shape.

### Offset Pagination

The standard Trellis offset response contains `entries`, `count`, `offset`,
`limit`, and optional `nextOffset`. The request requires `limit`; `offset`
defaults to zero. `count` describes matching rows before the page bound.
Concurrent mutations can change later pages: this is not a snapshot.

TypeScript exposes `PageRequestSchema`, `PageResponseSchema`,
`normalizePageQuery`, and `buildPageResponse`. Endpoint-specific limits still
apply when calling a Trellis endpoint.

### Cursor Pagination

Cursor responses contain `items` and `page.nextCursor` when more data exists.
The request accepts `cursor` and `limit`; a supplied cursor must be nonempty.
The generic helper defaults to 100 items with a maximum of 500, but individual
APIs can document different limits. Callers treat cursors as opaque positions.

TypeScript exposes `CursorQuerySchema`, `CursorPageInfoSchema`,
`CursorPageSchema`, `normalizeCursorQuery`, and `buildCursorPage`.

## Schema Validation

Trellis validates declared wire payloads before handing them to application
handlers. TypeBox is an implementation detail of the generated TypeScript
validation surface, not a service configuration requirement. Applications may
validate environment variables, files, forms, and local domain objects with any
library or ordinary language code.

Trellis-owned public wire objects tolerate additive unknown fields while still
validating known members. Signed proof formats and authored source/configuration
formats have stricter rules where ignoring unknown data would be unsafe. A
proof-bearing request binds the complete payload, including unknown members,
before any projection to known fields. These protocol guarantees are distinct
from application database schema or migration policy.

### Annotated Validation Metadata

The canonical schema representation supports `x-trellis-validation` metadata for
field-level validation feedback. It is descriptive, not additional authority or
a change to validation acceptance. Fully annotated failures can produce
`SchemaValidationError`; structural or unannotated failures use
`ValidationError`. The removed TypeScript contracts package and its
`withTrellisValidation` helper are not current authoring APIs. Consult native
IDL support before relying on a metadata feature in authored source.

## Storage Identity

Trellis runtime IDs, digests, session keys, and revision tokens have semantics
defined by their owning APIs. Applications must preserve those meanings when
calling Trellis. Trellis does not require application tables to use ULIDs,
separate surrogate primary keys, or a particular repository abstraction.

## Result Type

Generated RPCs and many runtime operations return Result-style values so callers
can distinguish declared business failures from validation and transport
failures. This does not mean every public method returns a Result: lifecycle
methods and registration methods have their own documented signatures.

TypeScript supports `.orThrow()` where exception-based control flow is useful;
Rust uses ordinary typed `Result`. Neither library dictates how an application
models errors outside the Trellis boundary.

## TypeScript Typing Policy

Generated participant types describe the selected actions. Inline registration
infers a handler type; extracted handlers can use `RpcHandler` parameterized by
the generated participant and action name. Runtime validation protects the wire
boundary independently of TypeScript compilation. Changing or asserting an
application type does not grant extra Trellis actions.

## Error Handling

Declare expected RPC and operation errors in IDL. Generation creates the wire
representation and the corresponding TypeScript error classes or Rust variants.
Callers narrow those generated types rather than reconstructing unknown wire
payloads as if they were declared errors.

- `TransportError` reports runtime/transport failures, with a stable code and
  actionable message or hint where available.
- `ValidationError` reports malformed, structural, or unannotated invalid input.
- `SchemaValidationError` reports fully annotated validation issues.
- Unknown or undeclared remote errors remain distinguishable from generated
  declared errors; their diagnostic data is not discarded.

The runtime's extensible error envelope is not permission to bypass a declared
surface's error contract. Application-local error types remain
application-owned.

### SchemaValidationError shape

```json
{
  "type": "SchemaValidationError",
  "message": "Schema validation failed.",
  "issues": [
    {
      "path": "/title",
      "schemaPath": "#/properties/title",
      "keyword": "minLength",
      "code": "documents.title.empty",
      "message": "Enter a title.",
      "label": "Title",
      "params": { "limit": 1 }
    }
  ]
}
```

Consumers can display issue paths and messages without treating UX metadata as
authentication or authority. Public exact signatures belong in generated API
documentation; protocol/schema semantics belong in this design documentation.
