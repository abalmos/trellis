import { assertEquals } from "@std/assert";

import { isAuthorizationPending } from "./client_connect_internal.ts";

Deno.test("only authorization_pending requests a browser-bind retry", () => {
  assertEquals(
    isAuthorizationPending({ error: { code: "authorization_pending" } }),
    true,
  );
  assertEquals(
    isAuthorizationPending({ error: { code: "flow_expired" } }),
    false,
  );
  assertEquals(
    isAuthorizationPending({ error: "authorization_pending" }),
    false,
  );
  assertEquals(isAuthorizationPending(null), false);
});
