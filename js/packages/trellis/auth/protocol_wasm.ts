import init, {
  verify_authorization_context_token,
} from "./protocol_wasm/trellis_protocol_wasm.js";

let initialized: Promise<void> | undefined;

async function wasmBytes(): Promise<Uint8Array> {
  const url = new URL(
    "./protocol_wasm/trellis_protocol_wasm_bg.wasm",
    import.meta.url,
  );
  const runtime = globalThis as Record<string, unknown>;
  const deno = runtime["De" + "no"] as
    | { readFile(path: URL): Promise<Uint8Array> }
    | undefined;
  if (deno) return await deno.readFile(url);
  const process = runtime["pro" + "cess"] as
    | {
      versions?: { node?: string };
      getBuiltinModule?: (name: string) => {
        promises: { readFile(path: URL): Promise<Uint8Array> };
      };
    }
    | undefined;
  if (process?.versions?.node && process.getBuiltinModule) {
    return new Uint8Array(
      await process.getBuiltinModule("fs").promises.readFile(url),
    );
  }
  const response = await fetch(url);
  if (!response.ok) {
    throw new Error(
      `authorization protocol WASM returned HTTP ${response.status}`,
    );
  }
  return new Uint8Array(await response.arrayBuffer());
}

async function initialize(): Promise<void> {
  initialized ??= (async () => {
    await init({ module_or_path: await wasmBytes() });
  })();
  await initialized;
}

export async function verifyAuthorizationContextTokenWasm(args: {
  root: unknown;
  manifest: unknown;
  certificate: unknown;
  contextToken: string;
  policy: unknown;
}): Promise<unknown> {
  await initialize();
  return JSON.parse(
    verify_authorization_context_token(
      JSON.stringify(args.root),
      JSON.stringify(args.manifest),
      JSON.stringify(args.certificate),
      args.contextToken,
      JSON.stringify(args.policy),
    ),
  );
}
