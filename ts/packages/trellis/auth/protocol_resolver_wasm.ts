import init, {
  initSync,
  resolve_participant,
  type SyncInitInput,
} from "./protocol_wasm/trellis_protocol_resolver_wasm.js";
import type { GrantSet } from "./protocol_wasm.ts";
import type { JsonValue } from "./utils.ts";

type JsonObject = { [key: string]: JsonValue };

/** Native participant resolution returned by the Rust protocol boundary. */
export type ResolvedParticipant = {
  apiArtifacts: Record<string, JsonObject>;
  apiDigests: Record<string, string>;
  participant: JsonObject;
  participantDigest: string;
  participantNeeds: JsonObject;
  participantNeedsDigest: string;
  requiredGrants: GrantSet;
  optionalGrants: GrantSet;
  authorityProposal: JsonObject;
};

let initializedSync = false;
let initialized: Promise<void> | undefined;

function wasmUrl(): URL {
  return new URL(
    "./protocol_wasm/trellis_protocol_resolver_wasm_bg.wasm",
    import.meta.url,
  );
}

async function initializeAsync(): Promise<void> {
  if (initializedSync) return;
  initialized ??= (async () => {
    const runtime = globalThis as Record<string, unknown>;
    const deno = runtime["De" + "no"] as
      | { readFile(path: URL): Promise<Uint8Array> }
      | undefined;
    let bytes: Uint8Array;
    if (deno) {
      bytes = await deno.readFile(wasmUrl());
    } else {
      const process = runtime["pro" + "cess"] as
        | {
          versions?: { node?: string };
          getBuiltinModule?: (name: string) => {
            promises: { readFile(path: URL): Promise<Uint8Array> };
          };
        }
        | undefined;
      if (process?.versions?.node && process.getBuiltinModule) {
        bytes = new Uint8Array(
          await process.getBuiltinModule("fs").promises.readFile(wasmUrl()),
        );
      } else {
        const response = await fetch(wasmUrl());
        if (!response.ok) {
          throw new Error(
            `authorization protocol resolver WASM returned HTTP ${response.status}`,
          );
        }
        bytes = new Uint8Array(await response.arrayBuffer());
      }
    }
    await init({ module_or_path: bytes });
    initializedSync = true;
  })();
  await initialized;
}

function initializeSync(): void {
  if (initializedSync) return;
  const url = wasmUrl();
  const runtime = globalThis as Record<string, unknown>;
  const deno = runtime["De" + "no"] as
    | { readFileSync(path: URL): Uint8Array }
    | undefined;
  if (deno) {
    initSync({ module: deno.readFileSync(url) as SyncInitInput });
  } else {
    const process = runtime["pro" + "cess"] as
      | {
        versions?: { node?: string };
        getBuiltinModule?: (name: string) => {
          readFileSync(path: URL): Uint8Array;
        };
      }
      | undefined;
    if (!process?.versions?.node || !process.getBuiltinModule) {
      throw new Error("browser participant resolution must use the async API");
    }
    initSync({
      module: process.getBuiltinModule("fs").readFileSync(url) as SyncInitInput,
    });
  }
  initializedSync = true;
}

/** Resolve a native participant through the synchronous Rust protocol resolver. */
export function resolveParticipantV1WasmSync(args: {
  participant: unknown;
  apis: Record<string, unknown>;
}): ResolvedParticipant {
  initializeSync();
  return resolve(args);
}

/** Resolve a native participant through the asynchronous Rust protocol resolver. */
export async function resolveParticipantV1Wasm(args: {
  participant: unknown;
  apis: Record<string, unknown>;
}): Promise<ResolvedParticipant> {
  await initializeAsync();
  return resolve(args);
}

function resolve(args: {
  participant: unknown;
  apis: Record<string, unknown>;
}): ResolvedParticipant {
  return JSON.parse(
    resolve_participant(
      JSON.stringify(args.participant),
      JSON.stringify(args.apis),
    ),
  ) as ResolvedParticipant;
}
