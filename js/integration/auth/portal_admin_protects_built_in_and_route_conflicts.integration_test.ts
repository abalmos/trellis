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
      await admin.authPortalsList({ limit: 10 }).orThrow();
      await assertValidationError(admin.authPortalsPut({
        portalId: builtInPortalId,
        displayName: "Blocked Built-in Portal Update",
        entryUrl,
      }));
      await assertValidationError(admin.authPortalsRemove({
        portalId: builtInPortalId,
      }));

      await admin.authPortalsPut({
        portalId,
        displayName: "Portal Admin Conflicts",
        entryUrl,
      }).orThrow();
      await admin.authPortalsPut({
        portalId: conflictPortalId,
        displayName: "Portal Admin Conflicts Alt",
        entryUrl: conflictEntryUrl,
      }).orThrow();
      await admin.authPortalsRoutesPut({
        portalId,
        contractId: fixture.clientContract.CONTRACT.id,
        origin: appOrigin,
      }).orThrow();

      await assertValidationError(admin.authPortalsRoutesPut({
        portalId: conflictPortalId,
        contractId: fixture.clientContract.CONTRACT.id,
        origin: appOrigin,
      }));
      await assertValidationError(admin.authPortalsRemove({ portalId }));

      const removedRoute = await admin.authPortalsRoutesRemove({
        portalId,
        contractId: fixture.clientContract.CONTRACT.id,
        origin: appOrigin,
      }).orThrow();
      assertEquals(removedRoute.success, true);
      const removedPortal = await admin.authPortalsRemove({ portalId })
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
