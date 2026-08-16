/**
 * Generates isolated NATS operator/account/JWT/credential bootstraps for
 * Trellis tests.
 *
 * Bootstrap generation runs the pinned `nsc` binary (see `nats_binaries.ts`)
 * through a generated `bootstrap-nsc.sh` script executed with `sh`, with the
 * working directory set to the output directory. This requires a POSIX `sh`
 * on PATH, the same tooling constraint as the rest of the repo's shell-based
 * tooling. No container runtime is required.
 */
import { dirname, join } from "@std/path";
import { ensureNatsBinaries } from "./nats_binaries.ts";

export type LocalNatsBootstrapManifest = {
  accounts: {
    system: { name: string; publicKey: string };
    auth: { name: string; publicKey: string };
    trellis: { name: string; publicKey: string };
  };
  users: {
    system: { name: string; publicKey: string };
    authService: { name: string; publicKey: string };
    trellisService: { name: string; publicKey: string };
  };
  paths: {
    natsConfig: string;
    jwtConfig: string;
    creds: {
      systemService: string;
      authService: string;
      trellisService: string;
    };
    secrets: {
      authIssuerSigning: string;
      authTargetSigning: string;
      authCalloutXKey: string;
    };
  };
};

/** NATS account manifests keyed by isolated integration-test case id. */
export type LocalNatsBootstrapPoolManifest = {
  tenants: Record<string, LocalNatsBootstrapManifest>;
};

/** Localhost TCP ports used by a generated NATS server bootstrap. */
export type NatsBootstrapPorts = {
  readonly nats: number;
  readonly http: number;
  readonly websocket: number;
};

type GeneratedMetadata = {
  systemAccountName: string;
  systemAccountPublicKey: string;
  authAccountName: string;
  authAccountPublicKey: string;
  trellisAccountName: string;
  trellisAccountPublicKey: string;
  systemUserPublicKey: string;
  authUserPublicKey: string;
  trellisUserPublicKey: string;
};

type JsonValue =
  | null
  | boolean
  | number
  | string
  | JsonValue[]
  | { [key: string]: JsonValue };

function shQuote(value: string): string {
  return `'${value.replaceAll("'", "'\\''")}'`;
}

/** Renders the nats-server config for one local bootstrap. */
export function renderNatsConfig(args: {
  serverName: string;
  ports: NatsBootstrapPorts;
  storeDir: string;
}): string {
  return `server_name: ${args.serverName}

listen: 127.0.0.1:${args.ports.nats}
http: 127.0.0.1:${args.ports.http}

authorization {
  timeout: "30s"
}

websocket {
  listen: 127.0.0.1:${args.ports.websocket}
  no_tls: true
}

jetstream {
  store_dir: ${args.storeDir}
}

include ./jwt.conf
`;
}

/** Renders the nsc script that generates accounts, JWTs, and creds locally. */
export function renderNscScript(outDir: string, tenantCount: number): string {
  const tenants = Array.from({ length: tenantCount }, (_, index) => {
    const suffix = tenantCount === 1 ? "" : `_${index}`;
    const fileSuffix = tenantCount === 1 ? "" : `-${index}`;
    return `AUTH_ACCOUNT_NAME='AUTH${suffix}'
TRELLIS_ACCOUNT_NAME='TRELLIS${suffix}'
nsc add account --name "$AUTH_ACCOUNT_NAME"
nsc add account --name "$TRELLIS_ACCOUNT_NAME"
nsc edit account --name "$AUTH_ACCOUNT_NAME" --sk generate
nsc edit account --name "$TRELLIS_ACCOUNT_NAME" --sk generate
nsc edit account --name "$AUTH_ACCOUNT_NAME" --js-mem-storage -1 --js-disk-storage -1 --js-streams -1 --js-consumer 4096
nsc edit account --name "$TRELLIS_ACCOUNT_NAME" --js-mem-storage -1 --js-disk-storage -1 --js-streams -1 --js-consumer 4096
nsc add user --account "$AUTH_ACCOUNT_NAME" --name auth --allow-pubsub ">"
nsc add user --account "$TRELLIS_ACCOUNT_NAME" --name auth --allow-pubsub ">"
AUTH_USER=$(nsc describe user --account "$AUTH_ACCOUNT_NAME" --name auth --field sub | tr -d '"')
TRELLIS_USER=$(nsc describe user --account "$TRELLIS_ACCOUNT_NAME" --name auth --field sub | tr -d '"')
TRELLIS_ACCOUNT=$(nsc describe account --name "$TRELLIS_ACCOUNT_NAME" --field sub | tr -d '"')
nsc edit authcallout --account "$AUTH_ACCOUNT_NAME" --auth-user "$AUTH_USER" --allowed-account "$TRELLIS_ACCOUNT" --curve generate
nsc generate creds --account "$AUTH_ACCOUNT_NAME" --name auth > "$WORK_DIR/creds/auth-auth${fileSuffix}.creds"
nsc generate creds --account "$TRELLIS_ACCOUNT_NAME" --name auth > "$WORK_DIR/creds/trellis-auth${fileSuffix}.creds"
AUTH_ACCOUNT=$(nsc describe account --name "$AUTH_ACCOUNT_NAME" --field sub | tr -d '"')
TRELLIS_ACCOUNT=$(nsc describe account --name "$TRELLIS_ACCOUNT_NAME" --field sub | tr -d '"')
nsc describe account --name "$AUTH_ACCOUNT_NAME" --raw > "$WORK_DIR/data/jwt/\${AUTH_ACCOUNT}.jwt"
nsc describe account --name "$TRELLIS_ACCOUNT_NAME" --raw > "$WORK_DIR/data/jwt/\${TRELLIS_ACCOUNT}.jwt"
nsc list keys --account "$AUTH_ACCOUNT_NAME" --accounts --show-seeds --json > "$WORK_DIR/generated/auth-keys${fileSuffix}.json"
nsc list keys --account "$TRELLIS_ACCOUNT_NAME" --accounts --show-seeds --json > "$WORK_DIR/generated/trellis-keys${fileSuffix}.json"
cat > "$WORK_DIR/generated/metadata${fileSuffix}.json" <<EOF
{
  "systemAccountName": "\${SYSTEM_ACCOUNT_NAME}",
  "systemAccountPublicKey": "\${SYS_ACCOUNT}",
  "authAccountName": "\${AUTH_ACCOUNT_NAME}",
  "authAccountPublicKey": "\${AUTH_ACCOUNT}",
  "trellisAccountName": "\${TRELLIS_ACCOUNT_NAME}",
  "trellisAccountPublicKey": "\${TRELLIS_ACCOUNT}",
  "systemUserPublicKey": "\${SYSTEM_USER}",
  "authUserPublicKey": "\${AUTH_USER}",
  "trellisUserPublicKey": "\${TRELLIS_USER}"
}
EOF`;
  }).join("\n\n");

  return `set -eu
OPERATOR_NAME='Qlever'
SYSTEM_ACCOUNT_NAME='SYS'
WORK_DIR=${shQuote(outDir)}
export NKEYS_PATH="$WORK_DIR/.nkeys"
# Isolate the nsc store under the work dir (nsc 2.x ignores NSC_HOME).
nsc() {
  command nsc -H "$WORK_DIR/.nsc" "$@"
}
mkdir -p "$WORK_DIR/.nsc" "$WORK_DIR/.nkeys" "$WORK_DIR/data/jwt" "$WORK_DIR/creds" "$WORK_DIR/secrets" "$WORK_DIR/generated"

nsc add operator --name "$OPERATOR_NAME" --sys
nsc add user --account "$SYSTEM_ACCOUNT_NAME" --name system --allow-pubsub ">"
nsc generate creds --account "$SYSTEM_ACCOUNT_NAME" --name system > "$WORK_DIR/creds/system.creds"
SYS_ACCOUNT=$(nsc describe account --name "$SYSTEM_ACCOUNT_NAME" --field sub | tr -d '"')
SYSTEM_USER=$(nsc describe user --account "$SYSTEM_ACCOUNT_NAME" --name system --field sub | tr -d '"')
nsc describe account --name "$SYSTEM_ACCOUNT_NAME" --raw > "$WORK_DIR/data/jwt/\${SYS_ACCOUNT}.jwt"

${tenants}

nsc generate config --nats-resolver --config-file "$WORK_DIR/generated/jwt.conf" --force --sys-account "$SYSTEM_ACCOUNT_NAME"
`;
}

async function runChecked(
  program: string,
  args: string[],
  options: { cwd: string; env: Record<string, string> },
): Promise<void> {
  const result = await new Deno.Command(program, {
    args,
    cwd: options.cwd,
    env: options.env,
    stdin: "null",
    stdout: "piped",
    stderr: "piped",
  }).output();
  if (result.success) return;
  const stderr = new TextDecoder().decode(result.stderr).trim();
  const stdout = new TextDecoder().decode(result.stdout).trim();
  throw new Error(
    `${program} ${args.join(" ")} failed with status ${result.code}: ${
      stderr || stdout
    }`,
  );
}

function isRecord(value: JsonValue): value is { [key: string]: JsonValue } {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function collectSeeds(
  value: JsonValue,
  prefix: string,
  signing: boolean,
  curve: boolean,
  out: string[],
): void {
  if (Array.isArray(value)) {
    for (const item of value) collectSeeds(item, prefix, signing, curve, out);
    return;
  }
  if (!isRecord(value)) return;
  if (
    typeof value.seed === "string" && value.seed.startsWith(prefix) &&
    value.signing === signing && value.curve === curve
  ) {
    out.push(value.seed);
  }
  for (const item of Object.values(value)) {
    collectSeeds(item, prefix, signing, curve, out);
  }
}

async function firstSeedMatching(
  path: string,
  prefix: string,
  signing: boolean,
  curve: boolean,
  label: string,
): Promise<string> {
  const value = JSON.parse(await Deno.readTextFile(path)) as JsonValue;
  const seeds: string[] = [];
  collectSeeds(value, prefix, signing, curve, seeds);
  const seed = seeds[0];
  if (!seed) throw new Error(`missing generated ${label}`);
  return seed;
}

/** Rewrites the resolver JWT store directory to the local bootstrap output. */
export function normalizeJwtConfig(config: string, outDir: string): string {
  return config.replace(/\n+$/, "")
    .split("\n")
    .map((line) => {
      const trimmed = line.trimStart();
      if (trimmed.startsWith("dir:") || trimmed.startsWith("dir ")) {
        return `${line.slice(0, line.length - trimmed.length)}dir: ${
          join(outDir, "data", "jwt")
        }`;
      }
      return line;
    })
    .join("\n") + "\n";
}

/** Generates isolated NATS account, credential, and auth-callout files. */
export async function generateLocalNatsBootstrap(args: {
  outDir: string;
  ports: NatsBootstrapPorts;
}): Promise<LocalNatsBootstrapManifest> {
  const pool = await generateLocalNatsBootstrapPool({
    ...args,
    tenantIds: ["default"],
  });
  return pool.tenants.default;
}

/** Generates one NATS server bootstrap with isolated account pairs per tenant. */
export async function generateLocalNatsBootstrapPool(args: {
  outDir: string;
  tenantIds: readonly string[];
  ports: NatsBootstrapPorts;
}): Promise<LocalNatsBootstrapPoolManifest> {
  if (
    args.tenantIds.length === 0 ||
    new Set(args.tenantIds).size !== args.tenantIds.length
  ) {
    throw new Error("NATS bootstrap tenant ids must be non-empty and unique");
  }
  await Deno.mkdir(join(args.outDir, "data", "jwt"), { recursive: true });
  await Deno.mkdir(join(args.outDir, "creds"), { recursive: true });
  await Deno.mkdir(join(args.outDir, "secrets"), { recursive: true });
  await Deno.mkdir(join(args.outDir, "generated"), { recursive: true });
  await Deno.writeTextFile(
    join(args.outDir, "nats.conf"),
    renderNatsConfig({
      serverName: "trellis-test",
      ports: args.ports,
      storeDir: join(args.outDir, "data"),
    }),
  );
  await Deno.writeTextFile(
    join(args.outDir, "bootstrap-nsc.sh"),
    renderNscScript(args.outDir, args.tenantIds.length),
  );

  const binaries = await ensureNatsBinaries();
  await runChecked("sh", [join(args.outDir, "bootstrap-nsc.sh")], {
    cwd: args.outDir,
    env: {
      PATH: `${dirname(binaries.nsc)}:${Deno.env.get("PATH") ?? ""}`,
    },
  });

  const tenants: Record<string, LocalNatsBootstrapManifest> = {};
  for (const [index, tenantId] of args.tenantIds.entries()) {
    const suffix = args.tenantIds.length === 1 ? "" : `-${index}`;
    const issuerPath = `secrets/auth-issuer-signing${suffix}.seed`;
    const targetPath = `secrets/auth-target-signing${suffix}.seed`;
    const xkeyPath = `secrets/auth-sx${suffix}.seed`;
    await Deno.writeTextFile(
      join(args.outDir, issuerPath),
      await firstSeedMatching(
        join(args.outDir, "generated", `auth-keys${suffix}.json`),
        "SA",
        true,
        false,
        "auth issuer signing seed",
      ),
    );
    await Deno.writeTextFile(
      join(args.outDir, targetPath),
      await firstSeedMatching(
        join(args.outDir, "generated", `trellis-keys${suffix}.json`),
        "SA",
        true,
        false,
        "auth target signing seed",
      ),
    );
    await Deno.writeTextFile(
      join(args.outDir, xkeyPath),
      await firstSeedMatching(
        join(args.outDir, "generated", `auth-keys${suffix}.json`),
        "SX",
        false,
        true,
        "auth callout xkey seed",
      ),
    );
    const metadata = JSON.parse(
      await Deno.readTextFile(
        join(args.outDir, "generated", `metadata${suffix}.json`),
      ),
    ) as GeneratedMetadata;
    tenants[tenantId] = {
      accounts: {
        system: {
          name: metadata.systemAccountName,
          publicKey: metadata.systemAccountPublicKey,
        },
        auth: {
          name: metadata.authAccountName,
          publicKey: metadata.authAccountPublicKey,
        },
        trellis: {
          name: metadata.trellisAccountName,
          publicKey: metadata.trellisAccountPublicKey,
        },
      },
      users: {
        system: { name: "system", publicKey: metadata.systemUserPublicKey },
        authService: { name: "auth", publicKey: metadata.authUserPublicKey },
        trellisService: {
          name: "auth",
          publicKey: metadata.trellisUserPublicKey,
        },
      },
      paths: {
        natsConfig: "nats.conf",
        jwtConfig: "jwt.conf",
        creds: {
          systemService: "creds/system.creds",
          authService: `creds/auth-auth${suffix}.creds`,
          trellisService: `creds/trellis-auth${suffix}.creds`,
        },
        secrets: {
          authIssuerSigning: issuerPath,
          authTargetSigning: targetPath,
          authCalloutXKey: xkeyPath,
        },
      },
    };
  }
  await Deno.writeTextFile(
    join(args.outDir, "jwt.conf"),
    normalizeJwtConfig(
      await Deno.readTextFile(join(args.outDir, "generated", "jwt.conf")),
      args.outDir,
    ),
  );

  await Deno.remove(join(args.outDir, "generated"), { recursive: true });
  await Deno.remove(join(args.outDir, "bootstrap-nsc.sh"));

  return { tenants };
}
