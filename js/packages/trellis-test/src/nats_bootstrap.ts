import { join } from "@std/path";

const NATS_BOX_IMAGE = "docker.io/natsio/nats-box:0.19.7";
const WORK_DIR = "/work";

export type ContainerRuntime = "podman" | "docker";

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
    sentinel: { name: string; publicKey: string };
  };
  paths: {
    natsConfig: string;
    jwtConfig: string;
    creds: {
      systemService: string;
      authService: string;
      trellisService: string;
      sentinel: string;
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
  sentinelUserPublicKey: string;
};

type JsonValue =
  | null
  | boolean
  | number
  | string
  | JsonValue[]
  | { [key: string]: JsonValue };

async function commandSucceeds(
  program: string,
  args: string[],
): Promise<boolean> {
  try {
    const output = await new Deno.Command(program, {
      args,
      stdin: "null",
      stdout: "null",
      stderr: "null",
    }).output();
    return output.success;
  } catch {
    return false;
  }
}

export async function resolveContainerRuntime(): Promise<ContainerRuntime> {
  if (await commandSucceeds("podman", ["--version"])) return "podman";
  if (await commandSucceeds("docker", ["--version"])) return "docker";
  throw new Error("Trellis tests require podman or docker on PATH");
}

function shellQuote(value: string): string {
  return `'${value.replaceAll("'", "'\\''")}'`;
}

function renderNatsConfig(serverName: string): string {
  return `server_name: ${serverName}

listen: 0.0.0.0:4222
http: 0.0.0.0:8222

websocket {
  listen: 0.0.0.0:8080
  no_tls: true
}

jetstream {
  store_dir: /data
}

include ./jwt.conf
`;
}

function renderNscScript(tenantCount: number): string {
  const tenants = Array.from({ length: tenantCount }, (_, index) => {
    const suffix = tenantCount === 1 ? "" : `_${index}`;
    const fileSuffix = tenantCount === 1 ? "" : `-${index}`;
    return `AUTH_ACCOUNT_NAME='AUTH${suffix}'
TRELLIS_ACCOUNT_NAME='TRELLIS${suffix}'
nsc add account --name "$AUTH_ACCOUNT_NAME"
nsc add account --name "$TRELLIS_ACCOUNT_NAME"
nsc edit account --name "$AUTH_ACCOUNT_NAME" --sk generate
nsc edit account --name "$TRELLIS_ACCOUNT_NAME" --sk generate
nsc edit account --name "$AUTH_ACCOUNT_NAME" --js-mem-storage -1 --js-disk-storage -1 --js-streams -1 --js-consumer -1
nsc edit account --name "$TRELLIS_ACCOUNT_NAME" --js-mem-storage -1 --js-disk-storage -1 --js-streams -1 --js-consumer -1
nsc add user --account "$AUTH_ACCOUNT_NAME" --name auth --allow-pubsub ">"
nsc add user --account "$TRELLIS_ACCOUNT_NAME" --name auth --allow-pubsub ">"
nsc add user --account "$AUTH_ACCOUNT_NAME" --name sentinel --deny-pubsub ">"
AUTH_USER=$(nsc describe user --account "$AUTH_ACCOUNT_NAME" --name auth --field sub | tr -d '"')
TRELLIS_USER=$(nsc describe user --account "$TRELLIS_ACCOUNT_NAME" --name auth --field sub | tr -d '"')
SENTINEL_USER=$(nsc describe user --account "$AUTH_ACCOUNT_NAME" --name sentinel --field sub | tr -d '"')
TRELLIS_ACCOUNT=$(nsc describe account --name "$TRELLIS_ACCOUNT_NAME" --field sub | tr -d '"')
nsc edit authcallout --account "$AUTH_ACCOUNT_NAME" --auth-user "$AUTH_USER" --allowed-account "$TRELLIS_ACCOUNT" --curve generate
nsc generate creds --account "$AUTH_ACCOUNT_NAME" --name auth > /work/creds/auth-auth${fileSuffix}.creds
nsc generate creds --account "$TRELLIS_ACCOUNT_NAME" --name auth > /work/creds/trellis-auth${fileSuffix}.creds
nsc generate creds --account "$AUTH_ACCOUNT_NAME" --name sentinel > /work/creds/sentinel${fileSuffix}.creds
AUTH_ACCOUNT=$(nsc describe account --name "$AUTH_ACCOUNT_NAME" --field sub | tr -d '"')
TRELLIS_ACCOUNT=$(nsc describe account --name "$TRELLIS_ACCOUNT_NAME" --field sub | tr -d '"')
nsc describe account --name "$AUTH_ACCOUNT_NAME" --raw > "/work/data/jwt/\${AUTH_ACCOUNT}.jwt"
nsc describe account --name "$TRELLIS_ACCOUNT_NAME" --raw > "/work/data/jwt/\${TRELLIS_ACCOUNT}.jwt"
nsc list keys --account "$AUTH_ACCOUNT_NAME" --accounts --show-seeds --json > /work/generated/auth-keys${fileSuffix}.json
nsc list keys --account "$TRELLIS_ACCOUNT_NAME" --accounts --show-seeds --json > /work/generated/trellis-keys${fileSuffix}.json
cat > /work/generated/metadata${fileSuffix}.json <<EOF
{
  "systemAccountName": "\${SYSTEM_ACCOUNT_NAME}",
  "systemAccountPublicKey": "\${SYS_ACCOUNT}",
  "authAccountName": "\${AUTH_ACCOUNT_NAME}",
  "authAccountPublicKey": "\${AUTH_ACCOUNT}",
  "trellisAccountName": "\${TRELLIS_ACCOUNT_NAME}",
  "trellisAccountPublicKey": "\${TRELLIS_ACCOUNT}",
  "systemUserPublicKey": "\${SYSTEM_USER}",
  "authUserPublicKey": "\${AUTH_USER}",
  "trellisUserPublicKey": "\${TRELLIS_USER}",
  "sentinelUserPublicKey": "\${SENTINEL_USER}"
}
EOF`;
  }).join("\n\n");

  return `set -eu
OPERATOR_NAME='Qlever'
SYSTEM_ACCOUNT_NAME='SYS'
export NSC_HOME=/work/.nsc
export NKEYS_PATH=/work/.nkeys
mkdir -p "$NSC_HOME" "$NKEYS_PATH" /work/data/jwt /work/creds /work/secrets /work/generated

nsc add operator --name "$OPERATOR_NAME" --sys
nsc add user --account "$SYSTEM_ACCOUNT_NAME" --name system --allow-pubsub ">"
nsc generate creds --account "$SYSTEM_ACCOUNT_NAME" --name system > /work/creds/system.creds
SYS_ACCOUNT=$(nsc describe account --name "$SYSTEM_ACCOUNT_NAME" --field sub | tr -d '"')
SYSTEM_USER=$(nsc describe user --account "$SYSTEM_ACCOUNT_NAME" --name system --field sub | tr -d '"')
nsc describe account --name "$SYSTEM_ACCOUNT_NAME" --raw > "/work/data/jwt/\${SYS_ACCOUNT}.jwt"

${tenants}

nsc generate config --nats-resolver --config-file /work/generated/jwt.conf --force --sys-account "$SYSTEM_ACCOUNT_NAME"
`;
}

function containerMount(path: string, runtime: ContainerRuntime): string {
  return `${path}:${WORK_DIR}${runtime === "podman" ? ":Z" : ""}`;
}

async function runChecked(program: string, args: string[]): Promise<void> {
  const result = await new Deno.Command(program, {
    args,
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

function normalizeJwtConfig(config: string): string {
  return config.replaceAll(WORK_DIR, "/data")
    .split("\n")
    .map((line) => {
      const trimmed = line.trimStart();
      if (trimmed.startsWith("dir:") || trimmed.startsWith("dir ")) {
        return `${line.slice(0, line.length - trimmed.length)}dir: /data/jwt`;
      }
      return line;
    })
    .join("\n") + "\n";
}

/** Generates isolated NATS account, credential, and auth-callout files. */
export async function generateLocalNatsBootstrap(args: {
  outDir: string;
  runtime: ContainerRuntime;
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
  runtime: ContainerRuntime;
  tenantIds: readonly string[];
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
    renderNatsConfig("trellis-test"),
  );
  await Deno.writeTextFile(
    join(args.outDir, "bootstrap-nsc.sh"),
    renderNscScript(args.tenantIds.length),
  );

  await runChecked(args.runtime, [
    "run",
    "--rm",
    "-v",
    containerMount(args.outDir, args.runtime),
    NATS_BOX_IMAGE,
    "sh",
    "/work/bootstrap-nsc.sh",
  ]);

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
        sentinel: {
          name: "sentinel",
          publicKey: metadata.sentinelUserPublicKey,
        },
      },
      paths: {
        natsConfig: "nats.conf",
        jwtConfig: "jwt.conf",
        creds: {
          systemService: "creds/system.creds",
          authService: `creds/auth-auth${suffix}.creds`,
          trellisService: `creds/trellis-auth${suffix}.creds`,
          sentinel: `creds/sentinel${suffix}.creds`,
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
    ),
  );

  await Deno.remove(join(args.outDir, "generated"), { recursive: true });
  await Deno.remove(join(args.outDir, "bootstrap-nsc.sh"));

  return { tenants };
}

export function quoteForDisplay(value: string): string {
  return shellQuote(value);
}
