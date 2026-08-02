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
  const manifest = sign({
    authority: "trellis-test",
    critical: [],
    expiresAt: now + 3_600,
    extensions: {},
    format: "trellis.authorization-issuer-manifest.v1",
    generation: 1,
    issuedAt: now,
    issuers: [{
      keyId: issuerKeyId,
      publicKey: base64urlEncode(issuerPublicKey),
    }],
    notBefore: now - 30,
    rootKeyId,
  }, ROOT_SEED);
  const context = sign({
    authority: "trellis-test",
    authorityRef: { id: "usr_test", kind: "identity", version: 1 },
    capabilities: [],
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
    issuerManifestGeneration: 1,
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
  const manifestJson = canonicalizeJsonValue(manifest);
  return {
    context: JSON.parse(contextJson),
    trust: {
      root,
      manifest: JSON.parse(manifestJson),
      authorizationRegistry: {
        trustBucket: "trust",
        contextBucket: "contexts",
      },
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
