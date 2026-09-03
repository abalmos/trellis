# Trellis IDL

## Purpose

Trellis IDL is the declarative source language for Trellis APIs and deployable
participants. The compiler lowers `.trellis` source into canonical
`trellis.api.v1` and `trellis.participant.v1` artifacts. Rust and TypeScript are
generated targets, not contract authoring languages.

A project uses exactly one of these source layouts:

```text
project/contract.trellis
```

```text
project/contracts/*.trellis
```

Files directly under `contracts/` are sorted by path and parsed in one project
scope. Source directories are not recursive. A project cannot use both layouts.

## APIs And Participants

An `api` declares wire schemas, exported models, errors, and public RPC,
operation, event, and feed surfaces. A `participant` declares deployable
identity, implemented APIs, resources, jobs, and provider-side transfer
bindings. API artifacts are compiled before participants so every implementation
is pinned to the canonical API digest and resolved by `trellis-protocol`.

```trellis
api "demo.service@v1" {
    version "1.0.0";
    display_name "Field Ops Demo Service";
    description "Field operations APIs.";

    type NonEmptyString = string(min_length = 1);
    model SitesListRequest {
        limit: uint(maximum = 500);
        offset?: uint;
    }
    model SiteSummary {
        siteId: NonEmptyString;
        labels: map<string>;
    }

    export SiteSummary;
    error UnexpectedError;

    rpc "Sites.List" {
        version "v1";
        input SitesListRequest;
        output SiteSummary;
        errors [UnexpectedError];
    }
}

participant "demo.service@v1" service {
    implements "demo.service@v1";

    kv siteSummaries {
        purpose "Latest site summaries.";
        schema SiteSummary;
        history 1;
        ttl_ms 0;
    }
}
```

The complete initial service syntax additionally supports operation progress and
cancellation, send and receive transfers, events, feeds, object stores, job
queues, documentation blocks, enums, string literals, and named unions.

## Types

Initial scalar types are `string`, `bool`, `int`, `uint`, and `number`. `int`
lowers to a JSON Schema integer, while `uint` adds `minimum: 0`. Types compose
as `list<T>`, typed string-keyed `map<T>`, named references, string literals,
and unions with `|`. Scalar constraints include `minimum`, `maximum`,
`min_length`, `max_length`, `pattern`, and `format`.

`field?: T` means the field may be absent. It does not permit `null`. A nullable
value must explicitly include `null` in its union. Struct models remain open to
unknown future fields. A pure `map<T>` lowers to an object with schema-valued
`additionalProperties`, because map keys are payload rather than future struct
fields.

## Protocol Boundary

The IDL compiler owns source discovery, parsing, symbol checks, and schema
lowering. It constructs private protocol-shaped JSON and delegates strict
artifact validation, normalization, participant resolution, canonical JSON,
digests, subjects, grants, and compatibility to `trellis-protocol`.

Trellis IDL has no executable expressions, functions, loops, conditionals,
macros, plugins, environment or network access, raw JSON Schema blocks, source
imports, module system, formatter, or language-server extension API.
