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

    use required inventory "inventory@v1" {
        call rpc "Sites.Get";
        subscribe event "Sites.Changed";
    }

    kv siteSummaries {
        purpose "Latest site summaries.";
        schema SiteSummary;
        history 1;
        ttl_ms 0;
    }

    state selectedSite value {
        schema SiteSummary;
        state_version "selected-site.v1";
        docs {
            summary "Selected site state.";
            markdown "Stores the active site selected by this participant.";
        }
    }
}
```

Participant state supports `value` and `map`; every declaration has a schema,
while `state_version` and documentation are optional. The complete initial
service syntax additionally supports operation progress and cancellation, send
and receive transfers, events, feeds, object stores, job queues, documentation
blocks, enums, string literals, and named unions.

## Project Dependencies

Local API dependencies are declared in `trellis.toml`. Their `path` is relative
to the current project and names another Trellis project root, not a source file
or generated artifact. Version requirements belong in the manifest rather than
the IDL:

```toml
format = 1
name = "inventory-consumer-trellis"

[apis."inventory@v1"]
version = "^1.0"
path = "../inventory"
```

A participant uses a dependency through a local alias and selects concrete
symbolic actions with `use required` or `use optional`. Supported selections are
RPC call; operation invoke, observe, cancel, and signal control; event publish
and subscribe; feed subscribe; and state read and write. The compiler pins the
selected API's exact digest and delegates selection validation and grant
derivation to `trellis-protocol`.

Selections use the exact declared name in the selected API. `Get` and
`Sites.Get` are distinct surfaces; neither declaration order nor a suffix match
can change which is selected. An invented qualifier such as `invented.Sites.Get`
is rejected. API aliases select an API in the `use` declaration; do not prepend
one to an action name unless it is literally part of that action's declared
name. The same exact-name rule applies to resource and implemented-event
selections.

Dependency projects contribute only the requested API compiled from their own
IDL source. Their participants and manifests are not compiled recursively.
Therefore participants in sibling projects may depend on one another without a
compilation order or cycle handling.

Registry dependencies are acquired by the package manager. Tooling loads exact
locked APIs directly from the validated global content-addressed OCI cache and
supplies `ApiArtifact` values to compilation. The compiler does not know cache
locations, installation layouts, credentials, or network policy. Local path
dependencies continue to compile directly from source.

## Generation

`trellis.toml` has `format = 1`. Its language-neutral `name` is required when a
language package is generated; it is not an API or participant identity.

Language detection uses a root `Cargo.toml` for Rust and a root `package.json`,
`deno.json`, or `deno.jsonc` for TypeScript/JavaScript. One detected language
generates one ordinary package at `trellis/`; `[generate].output` can override
that destination. Multiple detected languages require both
`[generate.rust].output` and `[generate.typescript].output`. No
language-selection setting or implicit language subdirectory is added. Projects
with no supported marker generate no language package and remain valid API-only
projects.

Each selected language gets one cohesive package named by `name`, always at
version `0.0.0`. Owned and dependency APIs are modules under `apis`; participant
surfaces are modules under `participants`. Normalization collisions within
either namespace are errors. There are no per-API or per-participant packages or
flattened root surfaces. Generated participants project canonical artifacts,
exact digests, descriptors, resources, state, and transfers; they are runtime
data, not another authoring mode.

Rust output is an ordinary path-dependency crate. TypeScript output is an
ordinary ESM package with executable `.js`, `.d.ts`, and `package.json`, not a
Deno package. The same TypeScript renderer feeds native Oxc JavaScript and
declaration emission. Consumers need no transpilation step. Both packages depend
on the normal published Trellis runtime corresponding to the generator version.
Generation does not modify the consumer's language manifest or infer runtime
paths from repository ancestry; repository development uses ordinary ecosystem
dependency overrides in its harness.

Canonical values stay in memory through resolution, compilation, and rendering.
For file-based tooling, each package contains canonical JSON under
`artifacts/apis/<api-id>.json` and
`artifacts/participants/<participant-id>.json`, including dependency API
evidence. These are not language import surfaces. Publication consumes compiled
owned `ApiArtifact` values directly, not these files. No project-local
dependency installation tree or persistent generation index exists. Commit
generated packages when ordinary clones should build without the CLI or Trellis
API cache, as the Orders example does.

`trellis install` acquires exact locked dependencies and generates packages.
`trellis generate` is offline and uses already available dependencies.

`trellis generate --watch` (or `-w`) performs the same full generation once,
then watches the project and every direct local dependency project recursively.
Changes to `.trellis` source, `trellis.toml`, or `trellis.lock` trigger another
full compilation and generation. Source errors are reported without replacing
the previous successful artifacts, and the watcher remains active so a later
valid edit can regenerate them.

Renderer and formatter failures also retain the previous successful generated
outputs. Output collisions are rejected, unrelated user files are protected, and
publication uses only short-lived staging and backups. Generation does not
modify the global dependency cache.

## Types

Initial scalar types are `string`, `bool`, `int`, `uint`, and `number`. `int`
lowers to a JSON Schema integer, while `uint` adds `minimum: 0`. Types compose
as `list<T>`, typed string-keyed `map<T>`, named references, string literals,
and unions with `|`. Scalar constraints include `minimum`, `maximum`,
`min_length`, `max_length`, `pattern`, and `format`.

Constraints apply to specific types:

| Constraints                | Types                   | Values                                                                     |
| -------------------------- | ----------------------- | -------------------------------------------------------------------------- |
| `minimum`, `maximum`       | `number`, `int`, `uint` | finite JSON numbers, including negative, fractional, and exponent literals |
| `min_length`, `max_length` | `string`                | unsigned integer literals                                                  |
| `pattern`, `format`        | `string`                | string literals                                                            |
| `min_items`, `max_items`   | `list<T>`               | unsigned integer literals                                                  |

For example, `number(minimum = -1, maximum = 2.5)` and
`list<string>(min_items = 1)` are valid. Counts do not accept fractional or
exponent notation. `uint` retains its implicit zero lower bound. Inapplicable
constraints, repeated constraints, and lower bounds exceeding upper bounds are
source-located compilation errors.

`field?: T` means the field may be absent. It does not permit `null`. A nullable
value must explicitly include `null` in its union. Struct models remain open to
unknown future fields. A pure `map<T>` lowers to an object with schema-valued
`additionalProperties`, because map keys are payload rather than future struct
fields.

These are semantic Trellis types. JSON Schema is the current lowering carried by
`trellis.api.v1`, not the definition of the IDL type system. Other protocol
schema representations may exist in the future without becoming raw schema
authoring syntax or changing the Trellis source model.

## Surface Members

Every surface accepts `version`, `docs`, and `capabilities`. Its remaining
members depend on the surface kind:

| Surface   | Members                                                            |
| --------- | ------------------------------------------------------------------ |
| RPC       | `input`, `output`, `errors`, `transfer`                            |
| Operation | `input`, `output`, `progress`, `errors`, `transfer`, `cancellable` |
| Event     | `payload` (or `event`), `params`, `class`                          |
| Feed      | `input`, `event` (or `payload`)                                    |

Other members and repeated members are rejected at their source location,
including using both names of the same payload member. Required members are
still required: for example, an RPC needs its input and output. Event `params`
are JSON pointers into the payload. Generated publishers derive concrete
subjects from those values, and listeners subscribe to the corresponding
wildcard pattern.

## Protocol Boundary

The IDL compiler owns source discovery, parsing, symbol checks, and schema
lowering. It constructs private protocol-shaped JSON and delegates strict
artifact validation, normalization, participant resolution, canonical JSON,
digests, subjects, grants, and compatibility to `trellis-protocol`.

Trellis IDL has no executable expressions, functions, loops, conditionals,
macros, plugins, environment or network access, raw JSON Schema blocks, source
imports, module system, formatter, or language-server extension API.
