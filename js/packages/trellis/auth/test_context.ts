import { ed25519 } from "@noble/curves/ed25519.js";
import { sha256 as sha256Sync } from "@noble/hashes/sha256";

import type { AuthorizationContextBundle } from "./authorization_context.ts";
import {
  base64urlDecode,
  base64urlEncode,
  canonicalizeJsonValue,
  utf8,
} from "./utils.ts";

const ROOT_SEED = base64urlDecode(
  "AQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQE",
);
const ISSUER_SEED = base64urlDecode(
  "AgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgI",
);
const SESSION_SEED = base64urlDecode(
  "AwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwM",
);
const registryObjects = new Map<string, string>();
const registryFetchWrappers = new WeakSet<typeof globalThis.fetch>();

export function testAuthorizationRegistryResponse(
  input: URL | Request | string,
): Response | undefined {
  const url = new URL(input instanceof Request ? input.url : input.toString());
  const value = registryObjects.get(url.pathname);
  return value === undefined ? undefined : Response.json(JSON.parse(value));
}

function installTestAuthorizationRegistryFetch(): void {
  const current = globalThis.fetch;
  if (registryFetchWrappers.has(current)) return;
  const wrapped: typeof globalThis.fetch = (input, init) => {
    const response = testAuthorizationRegistryResponse(input);
    return response ? Promise.resolve(response) : current(input, init);
  };
  registryFetchWrappers.add(wrapped);
  globalThis.fetch = wrapped;
}

export function testAuthorizationContext(
  now = 1_700_000_002,
): AuthorizationContextBundle {
  const rootPublicKey = ed25519.getPublicKey(ROOT_SEED);
  const issuerPublicKey = ed25519.getPublicKey(ISSUER_SEED);
  const rootKeyId = digest(rootPublicKey);
  const issuerKeyId = digest(issuerPublicKey);
  const root = {
    authority: "trellis-test",
    format: "trellis.authorization-trust-root.v1",
    keyId: rootKeyId,
    publicKey: base64urlEncode(rootPublicKey),
  };
  const certificate = sign({
    authority: "trellis-test",
    critical: [],
    expiresAt: now + 3_600,
    extensions: {},
    format: "trellis.authorization-issuer-certificate.v1",
    issuedAt: now,
    keyId: issuerKeyId,
    notBefore: now - 30,
    publicKey: base64urlEncode(issuerPublicKey),
    rootKeyId,
    serial: "cert_test",
    usages: ["authorizationContext"],
  }, ROOT_SEED);
  const certificateDigest = digest(utf8(canonicalizeJsonValue(certificate)));
  const manifest = sign({
    authority: "trellis-test",
    critical: [],
    expiresAt: now + 3_600,
    extensions: {},
    format: "trellis.authorization-issuer-manifest.v1",
    generation: 1,
    issuedAt: now,
    issuers: [{
      certificateDigest,
      keyId: issuerKeyId,
      status: "active",
    }],
    notBefore: now - 30,
    rootKeyId,
  }, ROOT_SEED);
  const context = sign({
    authority: "trellis-test",
    authorityRef: { id: "usr_test", kind: "identity", version: 1 },
    capabilities: [],
    contextId: "ctx_test",
    critical: [],
    deploymentId: null,
    expiresAt: now + 270,
    extensions: {},
    format: "trellis.authorization-context.v1",
    grantSet: { format: "trellis.grant-set.v1", permissions: [] },
    inboxPrefix: "_INBOX.test",
    instanceId: null,
    issuedAt: now,
    issuerKeyId,
    notBefore: now - 30,
    participant: {
      artifactDigest: digest(utf8("artifact")),
      id: "test-app",
      kind: "app",
      needsDigest: digest(utf8("needs")),
    },
    principal: { id: "usr_test", kind: "user" },
    sessionId: "ses_test",
    sessionKey: base64urlEncode(ed25519.getPublicKey(SESSION_SEED)),
  }, ISSUER_SEED);
  const contextJson = canonicalizeJsonValue(context);
  const contextDigest = digest(utf8(contextJson));
  const manifestJson = canonicalizeJsonValue(manifest);
  const certificateJson = canonicalizeJsonValue(certificate);
  const manifestLocator = "/.well-known/trellis/authorization/trust/manifest.1";
  const certificateLocator =
    `/.well-known/trellis/authorization/trust/certificate.${issuerKeyId}.${certificateDigest}`;
  registryObjects.set(manifestLocator, manifestJson);
  registryObjects.set(certificateLocator, certificateJson);
  installTestAuthorizationRegistryFetch();
  return {
    context: base64urlEncode(utf8(contextJson)),
    contextDigest,
    refreshAt: now + 270 - 60 - contextJitter(contextDigest, 15),
    trust: {
      root,
      issuerManifestGeneration: 1,
      issuerManifestDigest: digest(utf8(manifestJson)),
      issuerManifestLocator: manifestLocator,
      issuerCertificateLocator: certificateLocator,
      policy: {
        allowedClockSkewSeconds: 30,
        maximumContextLifetimeSeconds: 300,
        maximumContextBytes: 16_384,
        maximumPermissions: 4_096,
        maximumCapabilities: 256,
        refreshLeadSeconds: 60,
        refreshJitterSeconds: 15,
      },
    },
  };
}

function contextJitter(contextDigest: string, maximum: number): number {
  const bytes = base64urlDecode(contextDigest);
  let value = 0n;
  for (const byte of bytes.slice(0, 8)) value = (value << 8n) | BigInt(byte);
  return Number(value % BigInt(maximum + 1));
}

function sign<T extends Record<string, unknown>>(
  value: T,
  seed: Uint8Array,
): T & { signature: string } {
  const format = value.format as string;
  const domain = utf8(format);
  const canonical = utf8(canonicalizeJsonValue(value));
  const input = new Uint8Array(8 + domain.length + canonical.length);
  const view = new DataView(input.buffer);
  view.setUint32(0, domain.length);
  input.set(domain, 4);
  view.setUint32(4 + domain.length, canonical.length);
  input.set(canonical, 8 + domain.length);
  return {
    ...value,
    signature: base64urlEncode(ed25519.sign(sha256Sync(input), seed)),
  };
}

function digest(value: Uint8Array): string {
  return base64urlEncode(sha256Sync(value));
}
