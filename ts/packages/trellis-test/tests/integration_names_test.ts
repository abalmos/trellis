import { assertEquals } from "@std/assert";
import { caseDeploymentId, integrationSlug } from "../src/integration/names.ts";

Deno.test("integrationSlug preserves deterministic repo-local slug behavior", () => {
  assertEquals(
    integrationSlug("billing.invoice-created"),
    "billing-invoice-created",
  );
  assertEquals(
    integrationSlug("Billing Invoice Created"),
    "Billing-Invoice-Created",
  );
  assertEquals(
    integrationSlug("billing/invoice:created"),
    "billing-invoice-created",
  );
  assertEquals(
    integrationSlug("billing_invoice.created"),
    "billing_invoice-created",
  );
});

Deno.test("caseDeploymentId includes the shared runtime run id and case slug", () => {
  assertEquals(
    caseDeploymentId("run-123", "billing.invoice-created"),
    "js-it-run-123-billing-invoice-created",
  );
});
