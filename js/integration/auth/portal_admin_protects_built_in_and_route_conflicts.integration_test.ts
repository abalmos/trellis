import { assert, assertEquals, assertInstanceOf } from "@std/assert";
import { ValidationError } from "@qlever-llc/trellis";
import { isErr } from "@qlever-llc/result";
import { caseScopedName } from "../_support/names.ts";
import { liveTrellisTest, runtimeScopeForCase } from "../_support/runtime.ts";
import { createAuthLocalLoginFixture } from "./_fixture.ts";

const CASE_ID = "auth.portal-admin-protects-built-in-and-route-conflicts";
const fixture = createAuthLocalLoginFixture(CASE_ID);
const builtInPortalId = "trellis.builtin.login";
const portalId = caseScopedName("auth-portal-admin-conflicts", CASE_ID);
const conflictPortalId = caseScopedName(
  "auth-portal-admin-conflicts-alt",
  CASE_ID,
);
const entryUrl = "https://portal-admin-conflicts.example/_trellis/login";
const conflictEntryUrl =
  "https://portal-admin-conflicts-alt.example/_trellis/login";

liveTrellisTest({
  name:
    "auth.portal-admin-protects-built-in-and-route-conflicts rejects protected portal mutations and active selector conflicts",
  scope: runtimeScopeForCase(CASE_ID),
  async fn(runtime) {
    const admin = await fixture.setupSessionAdmin(runtime);
    const appOrigin = new URL(runtime.trellisUrl).origin;
    try {
      await admin.rpc.auth.portalsList({ limit: 10 }).orThrow();
      await assertValidationError(admin.rpc.auth.portalsPut({
        portalId: builtInPortalId,
        displayName: "Blocked Built-in Portal Update",
        entryUrl,
      }));
      await assertValidationError(admin.rpc.auth.portalsRemove({
        portalId: builtInPortalId,
      }));

      await admin.rpc.auth.portalsPut({
        portalId,
        displayName: "Portal Admin Conflicts",
        entryUrl,
      }).orThrow();
      await admin.rpc.auth.portalsPut({
        portalId: conflictPortalId,
        displayName: "Portal Admin Conflicts Alt",
        entryUrl: conflictEntryUrl,
      }).orThrow();
      await admin.rpc.auth.portalsRoutesPut({
        portalId,
        contractId: fixture.clientContract.CONTRACT.id,
        origin: appOrigin,
      }).orThrow();

      await assertValidationError(admin.rpc.auth.portalsRoutesPut({
        portalId: conflictPortalId,
        contractId: fixture.clientContract.CONTRACT.id,
        origin: appOrigin,
      }));
      await assertValidationError(admin.rpc.auth.portalsRemove({ portalId }));

      const removedRoute = await admin.rpc.auth.portalsRoutesRemove({
        portalId,
        contractId: fixture.clientContract.CONTRACT.id,
        origin: appOrigin,
      }).orThrow();
      assertEquals(removedRoute.success, true);
      const removedPortal = await admin.rpc.auth.portalsRemove({ portalId })
        .orThrow();
      assertEquals(removedPortal.success, true);
    } finally {
      await admin.connection.close().catch(() => undefined);
    }
  },
});

async function assertValidationError(
  result: { take(): Promise<unknown> },
): Promise<void> {
  const value = await result.take();
  assert(isErr(value));
  assertInstanceOf(value.error, ValidationError);
}
