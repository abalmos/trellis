from pathlib import Path


def replace_once(path: str, old: str, new: str) -> None:
    p = Path(path)
    text = p.read_text()
    count = text.count(old)
    if count != 1:
        raise RuntimeError(f"{path}: expected one occurrence, found {count}: {old[:120]!r}")
    p.write_text(text.replace(old, new, 1))


# Contract authoring must not import the runtime WASM resolver.
p = Path("ts/packages/trellis/contract_support/protocol_artifacts.ts")
text = p.read_text()
text = text.replace(
    'import { resolveParticipantV1WasmSync } from "../auth/protocol_wasm.ts";\n',
    "",
    1,
)
text = text.replace(
    "  participantDigest: string;\n  participantNeedsDigest: string;\n",
    "  participantDigest: string;\n",
    1,
)
text = text.replace(
    "  readonly PARTICIPANT: Readonly<Record<string, unknown>>;\n  readonly PARTICIPANT_NEEDS_DIGEST: string;\n",
    "  readonly PARTICIPANT: Readonly<Record<string, unknown>>;\n",
    1,
)

marker = "function compileParticipant(\n"
insert_at = text.index(marker)
participant_digest = r'''/** Return the semantic digest of a native participant artifact. */
export function participantDigest(
  participant: Readonly<Record<string, unknown>>,
): string {
  const value = checkedObject(participant);
  const projection: JsonObject = {
    format: value.format,
    id: value.id,
    kind: value.kind,
  };
  for (const field of ["schemas", "implements", "uses"]) {
    copy(value, projection, field);
  }
  for (const section of ["state", "jobQueues", "eventConsumers"]) {
    const definitions = object(value[section]);
    if (!definitions || Object.keys(definitions).length === 0) continue;
    const lowered = structuredClone(definitions);
    for (const definition of Object.values(lowered)) {
      const record = object(definition);
      if (record) delete record.docs;
    }
    projection[section] = lowered;
  }
  const resources = object(value.resources);
  if (resources && Object.keys(resources).length > 0) {
    const lowered = structuredClone(resources);
    for (const definitions of Object.values(lowered)) {
      for (const definition of Object.values(object(definitions) ?? {})) {
        const record = object(definition);
        if (!record) continue;
        delete record.purpose;
        delete record.docs;
      }
    }
    projection.resources = lowered;
  }
  return sha256Base64urlSync(canonicalizeJson(projection));
}

'''
text = text[:insert_at] + participant_digest + text[insert_at:]

presentation_start = text.index(
    "/**\n * Returns native artifacts and exact dependency API evidence carried by a\n"
)
build_marker = "/** Build native API and participant artifacts directly from an authoring source. */"
build_start = text.index(build_marker, presentation_start)
pure_presentation = r'''/**
 * Returns intrinsic native artifacts and exact dependency API evidence carried
 * by a defined contract. Contextual participant resolution is deliberately a
 * separate runtime concern.
 */
export function nativeProtocolPresentation(
  contract: NativeProtocolContract,
): NativeProtocolPresentation {
  const api = nativeApi(contract);
  const participant = nativeParticipant(contract);
  const discoveredApis = collectActionSources(
    (contract[CONTRACT_RUNTIME]?.actions ?? []).flatMap((selected) => {
      const source = actionSource(selected.action);
      return source ? [source] : [];
    }),
  );
  const ownedApiId = String(api.id);
  const apis = Object.fromEntries(
    [...discoveredApis].map(([id, source]) => [id, checkedObject(source.api)]),
  );
  if (
    apis[ownedApiId] &&
    canonicalizeJson(apis[ownedApiId]) !== canonicalizeJson(api)
  ) {
    throw new Error(`Conflicting API evidence for owned API '${ownedApiId}'`);
  }
  apis[ownedApiId] = api;
  if (apiDigest(api) !== contract.API_DIGEST) {
    throw new Error("Defined contract API digest does not match its artifact");
  }
  if (participantDigest(participant) !== contract.CONTRACT_DIGEST) {
    throw new Error(
      "Defined contract participant digest does not match its artifact",
    );
  }
  return {
    api,
    participant,
    referencedApis: Object.entries(apis)
      .filter(([id]) => id !== ownedApiId)
      .map(([, referencedApi]) => referencedApi),
  };
}

'''
text = text[:presentation_start] + pure_presentation + text[build_start:]

build_start = text.index(build_marker)
pure_build = r'''/** Build native API and participant artifacts directly from an authoring source. */
export function buildNativeProtocolArtifacts(
  source: Readonly<Record<string, unknown>>,
  referencedApis: Readonly<Record<string, ActionSource>> = {},
): NativeProtocolArtifacts {
  const contract = checkedObject(source);
  const api = compileApi(contract);
  const apiId = String(api.id);
  const apiDigests: Record<string, string> = {};
  const collectedSources = collectActionSources(Object.values(referencedApis));
  const apis: Record<string, JsonObject> = Object.fromEntries(
    [...collectedSources].map(([id, source]) => {
      const artifact = compileReferencedApi(source);
      if (artifact.id !== id) {
        throw new Error(
          `Action source API map key '${id}' does not match artifact id`,
        );
      }
      apiDigests[id] = source.apiDigest;
      return [id, artifact];
    }),
  );
  apis[apiId] = api;
  apiDigests[apiId] = apiDigest(api);
  const contractUses = object(contract.uses);
  for (
    const value of Object.values(object(contractUses?.required) ?? {})
      .concat(Object.values(object(contractUses?.optional) ?? {}))
  ) {
    const reference = object(value);
    const referencedApiId = reference?.contract;
    if (typeof referencedApiId !== "string" || !apis[referencedApiId]) {
      throw new Error(
        `Referenced API artifact '${String(referencedApiId)}' is required`,
      );
    }
  }
  const participant = compileParticipant(contract, api, apis, apiDigests);
  return {
    api,
    participant,
    referencedApis: Object.entries(apis)
      .filter(([id]) => id !== apiId)
      .map(([, value]) => value),
    apiDigest: apiDigests[apiId],
    participantDigest: participantDigest(participant),
  };
}
'''
text = text[:build_start] + pure_build
p.write_text(text)

Path("ts/packages/trellis/contract_support/protocol_resolution.ts").write_text(r'''import { resolveParticipantV1WasmSync } from "../auth/protocol_wasm.ts";
import { canonicalizeJson } from "./canonical.ts";
import {
  type NativeProtocolContract,
  type NativeProtocolPresentation,
  nativeProtocolPresentation,
} from "./protocol_artifacts.ts";

export type ResolvedNativeProtocolPresentation = NativeProtocolPresentation & {
  participantDigest: string;
  participantNeedsDigest: string;
};

/** Resolve and validate one defined contract against its exact API evidence. */
export function resolveNativeProtocolPresentation(
  contract: NativeProtocolContract,
): ResolvedNativeProtocolPresentation {
  const intrinsic = nativeProtocolPresentation(contract);
  const apis = Object.fromEntries(
    [intrinsic.api, ...intrinsic.referencedApis].map((api) => [
      String(api.id),
      api,
    ]),
  );
  const ownedApiId = String(intrinsic.api.id);
  const resolved = resolveParticipantV1WasmSync({
    participant: intrinsic.participant,
    apis,
  });
  const resolvedApi = resolved.apiArtifacts[ownedApiId];
  if (
    !resolvedApi ||
    canonicalizeJson(resolvedApi) !== canonicalizeJson(intrinsic.api)
  ) {
    throw new Error(
      "Resolved owned API does not match the defined contract API",
    );
  }
  if (resolved.apiDigests[ownedApiId] !== contract.API_DIGEST) {
    throw new Error("Defined contract API digest does not match resolution");
  }
  if (resolved.participantDigest !== contract.CONTRACT_DIGEST) {
    throw new Error(
      "Defined contract participant digest does not match resolution",
    );
  }
  return {
    api: resolvedApi,
    participant: resolved.participant,
    referencedApis: Object.entries(resolved.apiArtifacts)
      .filter(([id]) => id !== ownedApiId)
      .map(([, referencedApi]) => referencedApi),
    participantDigest: resolved.participantDigest,
    participantNeedsDigest: resolved.participantNeedsDigest,
  };
}
''')

# Defined contracts carry intrinsic identity only. Needs are contextual resolution.
p = Path("ts/packages/trellis/contract_support/mod.ts")
text = p.read_text()
text = text.replace(
    'export { resolveParticipantV1WasmSync } from "../auth/protocol_wasm.ts";\n',
    "",
    1,
)
text = text.replace(
    "  readonly PARTICIPANT: Readonly<Record<string, JsonValue>>;\n  readonly PARTICIPANT_NEEDS_DIGEST: string;\n",
    "  readonly PARTICIPANT: Readonly<Record<string, JsonValue>>;\n",
    1,
)
text = text.replace(
    '      exportName === "PARTICIPANT_NEEDS_DIGEST" ||\n',
    "",
    1,
)
text = text.replace(
    "    PARTICIPANT,\n    PARTICIPANT_NEEDS_DIGEST: native.participantNeedsDigest,\n",
    "    PARTICIPANT,\n",
    1,
)
p.write_text(text)

# Runtime bootstrap resolves contextual needs through WASM only when connecting.
replace_once(
    "ts/packages/trellis/client_connect.ts",
    '''import {
  type NativeProtocolContract,
  nativeProtocolPresentation,
} from "./contract_support/protocol_artifacts.ts";''',
    '''import type { NativeProtocolContract } from "./contract_support/protocol_artifacts.ts";
import { resolveNativeProtocolPresentation } from "./contract_support/protocol_resolution.ts";''',
)
replace_once(
    "ts/packages/trellis/client_connect.ts",
    "const presentation = nativeProtocolPresentation(args.contract);",
    "const presentation = resolveNativeProtocolPresentation(args.contract);",
)
replace_once(
    "ts/packages/trellis/client_connect.ts",
    "args.participant.needsDigest !== args.contract.PARTICIPANT_NEEDS_DIGEST",
    "args.participant.needsDigest !== presentation.participantNeedsDigest",
)

replace_once(
    "ts/packages/trellis/service/runtime/service.ts",
    '''import {
  type NativeProtocolContract,
  nativeProtocolPresentation,
} from "../../contract_support/protocol_artifacts.ts";''',
    '''import type { NativeProtocolContract } from "../../contract_support/protocol_artifacts.ts";
import { resolveNativeProtocolPresentation } from "../../contract_support/protocol_resolution.ts";''',
)
replace_once(
    "ts/packages/trellis/service/runtime/service.ts",
    "const presentation = nativeProtocolPresentation(args.contract);",
    "const presentation = resolveNativeProtocolPresentation(args.contract);",
)
replace_once(
    "ts/packages/trellis/service/runtime/service.ts",
    "args.identity.participantNeedsDigest !==\n      args.contract.PARTICIPANT_NEEDS_DIGEST",
    "args.identity.participantNeedsDigest !==\n      presentation.participantNeedsDigest",
)
replace_once(
    "ts/packages/trellis/service/runtime/service.ts",
    "participantNeedsDigest: args.contract.PARTICIPANT_NEEDS_DIGEST,",
    "participantNeedsDigest: presentation.participantNeedsDigest,",
)

# Keep contract definitions on narrow authoring surfaces.
replace_once(
    "ts/packages/trellis/contracts/trellis_core.ts",
    '''import {
  defineServiceContract,
  TrellisSurfaceStatusRequestSchema,
  TrellisSurfaceStatusResponseSchema,
} from "../index.ts";''',
    '''import { defineServiceContract } from "../contract.ts";
import {
  TrellisSurfaceStatusRequestSchema,
  TrellisSurfaceStatusResponseSchema,
} from "../models/trellis/rpc/TrellisSurfaceStatus.ts";''',
)
replace_once(
    "ts/portals/login/contract.ts",
    'import { defineAppContract } from "@qlever-llc/trellis";',
    'import { defineAppContract } from "@qlever-llc/trellis/contracts";',
)

# Update protocol artifact tests to use explicit runtime resolution where intended.
p = Path("ts/packages/trellis/contract_support/protocol_artifacts_test.ts")
text = p.read_text()
anchor = '''import {
  apiDigest,
  collectActionSources,
  nativeProtocolPresentation,
} from "./protocol_artifacts.ts";'''
replacement = anchor + '''
import { resolveNativeProtocolPresentation } from "./protocol_resolution.ts";'''
if text.count(anchor) != 1:
    raise RuntimeError("protocol artifact test import anchor changed")
text = text.replace(anchor, replacement, 1)
start = text.index('Deno.test("native presentation verifies all participant identities and evidence"')
end = text.index('\nDeno.test("action sources reject conflicting and forged API revisions"', start)
old_test = text[start:end]
new_test = r'''Deno.test("resolved native presentation verifies participant identity and evidence", () => {
  const provider = defineServiceContract(
    { schemas: { Empty: Type.Object({}) } },
    (ref) => ({
      id: "trellis.test.presentation-provider@v1",
      displayName: "Presentation provider",
      description: "Provides exact API evidence.",
      rpc: {
        Call: {
          version: "v1",
          input: ref.schema("Empty"),
          output: ref.schema("Empty"),
          errors: [],
        },
      },
    }),
  );
  const consumer = defineAppContract(() => ({
    id: "trellis.test.presentation-consumer@v1",
    displayName: "Presentation consumer",
    description: "Consumes exact API evidence.",
    uses: [provider.Call],
  }));
  const forged = (changes: Partial<typeof consumer>) => ({
    ...consumer,
    [CONTRACT_RUNTIME]: consumer[CONTRACT_RUNTIME],
    ...changes,
  });

  const resolved = resolveNativeProtocolPresentation(consumer);
  assertEquals(resolved.participantDigest, consumer.CONTRACT_DIGEST);
  assertEquals(
    resolved.participantNeedsDigest,
    resolveParticipantV1WasmSync({
      participant: consumer.PARTICIPANT,
      apis: Object.fromEntries(
        [resolved.api, ...resolved.referencedApis].map((api) => [
          String(api.id),
          api,
        ]),
      ),
    }).participantNeedsDigest,
  );
  assertThrows(
    () => resolveNativeProtocolPresentation(forged({ CONTRACT_DIGEST: "forged" })),
    Error,
    "participant digest",
  );
  const staleSelf = structuredClone(provider.PARTICIPANT);
  (staleSelf.implements as Record<string, Record<string, unknown>>).self
    .apiDigest = "forged";
  assertThrows(
    () => resolveNativeProtocolPresentation({ ...provider, PARTICIPANT: staleSelf }),
    Error,
    "digest",
  );
  const runtime = consumer[CONTRACT_RUNTIME];
  assertThrows(
    () =>
      resolveNativeProtocolPresentation(
        forged({ [CONTRACT_RUNTIME]: { ...runtime, actions: [] } }),
      ),
    Error,
    "required",
  );
});
'''
text = text[:start] + new_test + text[end:]
p.write_text(text)

# Protocol WASM is generated build state, not repository source.
replace_once(
    ".gitignore",
    "# Generated build artifacts\ngenerated/",
    "# Generated build artifacts\ngenerated/\nts/packages/trellis/auth/protocol_wasm/",
)

# Canonical preparation: SDKs first, then WASM, then embedded portal; no npm work.
p = Path("rust/xtask/src/main.rs")
text = p.read_text()
text = text.replace(
    '''    #[command(name = "prepare-watch")]
    PrepareWatch,
    #[command(name = "build", disable_help_flag = true)]''',
    '''    #[command(name = "prepare-watch")]
    PrepareWatch,
    #[command(name = "protocol-wasm")]
    ProtocolWasm,
    #[command(name = "build", disable_help_flag = true)]''',
    1,
)
text = text.replace(
    '''        XtaskCommand::Prepare => run_prepare(),
        XtaskCommand::PrepareWatch => run_prepare_watch(),
        XtaskCommand::Build { args } => run_build(&args),''',
    '''        XtaskCommand::Prepare => run_prepare(),
        XtaskCommand::PrepareWatch => run_prepare_watch(),
        XtaskCommand::ProtocolWasm => generate_protocol_wasm(),
        XtaskCommand::Build { args } => run_build(&args),''',
    1,
)
text = text.replace(
    '''fn run_prepare() -> Result<()> {
    generate_protocol_wasm()?;
    run_generate_prepare(&[])?;
    build_embedded_login_portal()?;''',
    '''fn run_prepare() -> Result<()> {
    run_generate_prepare(&["--no-npm"])?;
    generate_protocol_wasm()?;
    build_embedded_login_portal()?;''',
    1,
)
text = text.replace(
    '"// Generated by cargo xtask prepare.\\nexport const PROTOCOL_WASM_BASE64 = \\"{}\\";\\n",',
    '"// Generated by cargo xtask protocol-wasm.\\nexport const PROTOCOL_WASM_BASE64 = \\"{}\\";\\n",',
    1,
)
test_anchor = '''    #[test]
    fn parse_prepare_watch_command() {'''
text = text.replace(
    test_anchor,
    '''    #[test]
    fn parse_protocol_wasm_command() {
        let command = parse_command(["protocol-wasm".to_string()].into_iter())
            .expect("parse protocol-wasm")
            .expect("protocol-wasm command");
        assert_eq!(command, XtaskCommand::ProtocolWasm);
    }

    #[test]
    fn parse_prepare_watch_command() {''',
    1,
)
reject_anchor = '''    #[test]
    fn prepare_watch_rejects_extra_args() {'''
text = text.replace(
    reject_anchor,
    '''    #[test]
    fn protocol_wasm_rejects_extra_args() {
        let error = parse_command(
            ["protocol-wasm", "--workspace"]
                .into_iter()
                .map(str::to_string),
        )
        .expect_err("protocol-wasm should reject extra args");
        assert!(error.to_string().contains("unexpected argument"));
    }

    #[test]
    fn prepare_watch_rejects_extra_args() {''',
    1,
)
p.write_text(text)

# Local TS/package paths request WASM explicitly when needed.
replace_once(
    "ts/deno.json",
    '    "prepare:watch": "deno run -A @qlever-llc/trellis/generate prepare --watch ..",',
    '    "prepare:watch": "deno run -A @qlever-llc/trellis/generate prepare --watch ..",\n    "protocol:wasm": "cargo run --manifest-path ../rust/xtask/Cargo.toml -- protocol-wasm",',
)
replace_once(
    "ts/deno.json",
    '    "check": "deno task prepare && deno check packages/trellis/index.ts packages/trellis-svelte/src/index.ts packages/trellis-svelte/src/context.svelte.ts packages/trellis-test/index.ts",',
    '    "check": "deno task prepare && deno task protocol:wasm && deno check packages/trellis/index.ts packages/trellis-svelte/src/index.ts packages/trellis-svelte/src/context.svelte.ts packages/trellis-test/index.ts",',
)
replace_once(
    "ts/deno.json",
    '    "test:auth": "deno task prepare && deno test -A packages/trellis/auth/conformance_test.ts packages/trellis/auth/session_auth_test.ts",',
    '    "test:auth": "deno task prepare && deno task protocol:wasm && deno test -A packages/trellis/auth/conformance_test.ts packages/trellis/auth/session_auth_test.ts",',
)
replace_once(
    "ts/portals/login/deno.json",
    '    "prepare": "deno task -c ../../deno.json prepare",',
    '    "prepare": "deno task -c ../../deno.json prepare && deno task -c ../../deno.json protocol:wasm",',
)
replace_once(
    "ts/packages/trellis/deno.json",
    '    "build:npm": "deno run -A ./scripts/build_npm.ts",',
    '    "build:npm": "deno task -c ../../deno.json protocol:wasm && deno run -A ./scripts/build_npm.ts",',
)
