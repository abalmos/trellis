/** Mutable claims exposed by the live OIDC test provider. */
export type TestOidcClaims = Record<string, unknown>;

/** A minimal live OIDC provider backed by WebCrypto signatures. */
export type TestOidcProvider = {
  readonly issuer: string;
  setClaims(claims: TestOidcClaims, redirectOrigin?: string): void;
  shutdown(): Promise<void>;
};

/** Starts a signed OIDC provider suitable for end-to-end login tests. */
export async function startTestOidcProvider(
  initialClaims: TestOidcClaims,
): Promise<TestOidcProvider> {
  const keyPair = await crypto.subtle.generateKey(
    {
      name: "RSASSA-PKCS1-v1_5",
      modulusLength: 2048,
      publicExponent: new Uint8Array([1, 0, 1]),
      hash: "SHA-256",
    },
    true,
    ["sign", "verify"],
  );
  const publicJwk = await crypto.subtle.exportKey("jwk", keyPair.publicKey);
  let claims = initialClaims;
  const scopedClaims = new Map<string, TestOidcClaims>();
  const codes = new Map<string, { nonce: string; claims: TestOidcClaims }>();
  let issuer = "";
  const server = Deno.serve(
    { hostname: "127.0.0.1", port: 0, onListen() {} },
    async (request) => {
      const url = new URL(request.url);
      if (url.pathname === "/.well-known/openid-configuration") {
        return Response.json({
          issuer,
          authorization_endpoint: `${issuer}/authorize`,
          token_endpoint: `${issuer}/token`,
          jwks_uri: `${issuer}/jwks`,
          response_types_supported: ["code"],
          subject_types_supported: ["public"],
          id_token_signing_alg_values_supported: ["RS256"],
          scopes_supported: ["openid", "profile", "email"],
          token_endpoint_auth_methods_supported: ["client_secret_post"],
          claims_supported: ["sub", "aud", "exp", "iat", "nonce", "roles"],
        });
      }
      if (url.pathname === "/jwks") {
        return Response.json({
          keys: [{ ...publicJwk, kid: "test-key", use: "sig", alg: "RS256" }],
        });
      }
      if (url.pathname === "/authorize") {
        const redirectUri = url.searchParams.get("redirect_uri");
        const state = url.searchParams.get("state");
        const nonce = url.searchParams.get("nonce");
        if (!redirectUri || !state || !nonce) {
          return new Response(null, { status: 400 });
        }
        const code = crypto.randomUUID();
        codes.set(code, {
          nonce,
          claims: scopedClaims.get(new URL(redirectUri).origin) ?? claims,
        });
        const redirect = new URL(redirectUri);
        redirect.searchParams.set("code", code);
        redirect.searchParams.set("state", state);
        return Response.redirect(redirect, 302);
      }
      if (url.pathname === "/token" && request.method === "POST") {
        const form = await request.formData();
        const code = form.get("code");
        const authorization = typeof code === "string"
          ? codes.get(code)
          : undefined;
        if (!authorization || typeof code !== "string") {
          return new Response(null, { status: 400 });
        }
        codes.delete(code);
        const now = Math.floor(Date.now() / 1000);
        const idToken = await signedJwt(
          keyPair.privateKey,
          { alg: "RS256", kid: "test-key", typ: "JWT" },
          {
            iss: issuer,
            sub: "test-oidc-user",
            aud: "trellis-test-client",
            iat: now,
            exp: now + 300,
            nonce: authorization.nonce,
            ...authorization.claims,
          },
        );
        return Response.json({
          access_token: crypto.randomUUID(),
          token_type: "Bearer",
          expires_in: 300,
          id_token: idToken,
        });
      }
      return new Response(null, { status: 404 });
    },
  );
  issuer = `http://127.0.0.1:${server.addr.port}`;
  return {
    issuer,
    setClaims(value, redirectOrigin) {
      if (redirectOrigin === undefined) claims = value;
      else scopedClaims.set(redirectOrigin, value);
    },
    shutdown: () => server.shutdown(),
  };
}

async function signedJwt(
  key: CryptoKey,
  header: Record<string, unknown>,
  payload: Record<string, unknown>,
): Promise<string> {
  const body = `${base64url(JSON.stringify(header))}.${
    base64url(JSON.stringify(payload))
  }`;
  const signature = await crypto.subtle.sign(
    "RSASSA-PKCS1-v1_5",
    key,
    new TextEncoder().encode(body),
  );
  return `${body}.${base64url(new Uint8Array(signature))}`;
}

function base64url(value: string | Uint8Array): string {
  const bytes = typeof value === "string"
    ? new TextEncoder().encode(value)
    : value;
  let binary = "";
  for (const byte of bytes) binary += String.fromCharCode(byte);
  return btoa(binary).replaceAll("+", "-").replaceAll("/", "_").replace(
    /=+$/,
    "",
  );
}
