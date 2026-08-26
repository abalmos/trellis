import { assertEquals, assertStringIncludes } from "@std/assert";
import { join } from "@std/path";
import {
  buildControlPlaneConfig,
  reserveHostTestSlot,
  reserveLocalPort,
  writeTrellisConfig,
} from "../src/control_plane_config.ts";
import type { LocalNatsBootstrapManifest } from "../src/nats_bootstrap.ts";

function testManifest(): LocalNatsBootstrapManifest {
  return {
    accounts: {
      system: { name: "SYS", publicKey: "SYS_PUBLIC" },
      auth: { name: "AUTH", publicKey: "AUTH_PUBLIC" },
      trellis: { name: "TRELLIS", publicKey: "TRELLIS_PUBLIC" },
    },
    users: {
      system: { name: "system", publicKey: "SYSTEM_USER_PUBLIC" },
      authService: { name: "auth", publicKey: "AUTH_USER_PUBLIC" },
      trellisService: { name: "auth", publicKey: "TRELLIS_USER_PUBLIC" },
    },
    paths: {
      natsConfig: "nats.conf",
      jwtConfig: "jwt.conf",
      creds: {
        systemService: "creds/system.creds",
        authService: "creds/auth-auth.creds",
        trellisService: "creds/trellis-auth.creds",
      },
      secrets: {
        authIssuerSigning: "secrets/auth-issuer-signing.seed",
        authTargetSigning: "secrets/auth-target-signing.seed",
        authCalloutXKey: "secrets/auth-sx.seed",
      },
    },
  };
}

Deno.test("reserveLocalPort records a process-wide host lease", async () => {
  const port = reserveLocalPort();
  const lockRoot = Deno.env.get("TRELLIS_TEST_PORT_LOCK_DIR") ??
    (Deno.build.os === "windows" ? Deno.env.get("TEMP") : "/tmp");
  if (lockRoot === undefined) {
    throw new Error("no temporary directory is configured");
  }
  const lockPath = `${lockRoot}/trellis-test-port-${port}.lock`;

  assertEquals((await Deno.readTextFile(lockPath)).trim(), String(Deno.pid));
});

Deno.test("reserveHostTestSlot enforces the configured host bound", async () => {
  const lockRoot = await Deno.makeTempDir({ prefix: "trellis-host-slots-" });
  const previousJobs = Deno.env.get("TRELLIS_TEST_HOST_JOBS");
  const previousRoot = Deno.env.get("TRELLIS_TEST_HOST_LOCK_DIR");
  Deno.env.set("TRELLIS_TEST_HOST_JOBS", "2");
  Deno.env.set("TRELLIS_TEST_HOST_LOCK_DIR", lockRoot);
  try {
    const legacyLock = join(lockRoot, "trellis-test-host-slots", "0.lock");
    Deno.mkdirSync(legacyLock, { recursive: true });
    const migrated = await reserveHostTestSlot();
    assertEquals(
      Array.from(Deno.readDirSync(join(lockRoot, "trellis-test-host-slots")))
        .filter((entry) => entry.isFile).length,
      1,
    );
    migrated?.release();
    Deno.removeSync(legacyLock);

    const first = await reserveHostTestSlot();
    const second = await reserveHostTestSlot();
    assertEquals(
      Array.from(Deno.readDirSync(join(lockRoot, "trellis-test-host-slots")))
        .length,
      2,
    );
    first?.release();
    second?.release();
  } finally {
    if (previousJobs === undefined) Deno.env.delete("TRELLIS_TEST_HOST_JOBS");
    else Deno.env.set("TRELLIS_TEST_HOST_JOBS", previousJobs);
    if (previousRoot === undefined) {
      Deno.env.delete("TRELLIS_TEST_HOST_LOCK_DIR");
    } else {
      Deno.env.set("TRELLIS_TEST_HOST_LOCK_DIR", previousRoot);
    }
    await Deno.remove(lockRoot, { recursive: true });
  }
});

Deno.test("writeTrellisConfig writes file-backed test control-plane config", async () => {
  const workdir = await Deno.makeTempDir({ prefix: "trellis-config-test-" });
  try {
    const natsDir = join(workdir, "nats");
    await Deno.mkdir(join(natsDir, "secrets"), { recursive: true });
    await Deno.writeTextFile(
      join(natsDir, "secrets", "auth-issuer-signing.seed"),
      "issuer-seed\n",
    );
    await Deno.writeTextFile(
      join(natsDir, "secrets", "auth-target-signing.seed"),
      "target-seed\n",
    );
    await Deno.writeTextFile(
      join(natsDir, "secrets", "auth-sx.seed"),
      "sx-seed\n",
    );

    const config = buildControlPlaneConfig({
      workdir,
      natsUrl: "nats://127.0.0.1:4222",
      websocketUrl: "ws://127.0.0.1:8080",
      manifest: testManifest(),
      port: 3000,
      oauthProviders: {
        oidc_test: {
          type: "oidc",
          issuer: "https://idp.example",
          clientId: "test-client",
          clientSecret: "test-secret",
          logout: {
            enabled: true,
            endpoint: "https://idp.example/logout",
          },
        },
      },
    });
    const configPath = await writeTrellisConfig({ workdir, config });
    const text = await Deno.readTextFile(configPath);

    assertEquals(configPath, join(workdir, "trellis", "config.toml"));
    assertEquals(config.logLevel, "info");
    for (const section of ["platform", "jobs", "health", "eventlog"]) {
      assertStringIncludes(
        text,
        `path = "${join(workdir, "trellis", `trellis.sqlite.${section}`)}"`,
      );
    }
    assertStringIncludes(
      text,
      `system_creds_path = "${join(workdir, "nats", "creds/system.creds")}"`,
    );
    assertStringIncludes(text, `[oauth.providers."oidc_test"]`);
    assertEquals(
      await Deno.readTextFile(
        join(workdir, "trellis", "auth-issuer-signing.seed"),
      ),
      "issuer-seed\n",
    );
  } finally {
    await Deno.remove(workdir, { recursive: true }).catch(() => undefined);
  }
});
