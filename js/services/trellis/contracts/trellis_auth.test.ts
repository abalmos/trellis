import { assert, assertEquals } from "@std/assert";
import { defineAppContract } from "@qlever-llc/trellis";
import { getContractRuntime } from "@qlever-llc/trellis/internal/contract-runtime";

import { trellisAuth } from "./trellis_auth.ts";

const app = defineAppContract(() => ({
  id: "test.portal@v1",
  displayName: "Test Portal",
  description: "Exercise explicit auth selections in contract authoring.",
  uses: [
    trellisAuth.AuthIdentitiesList,
    trellisAuth.AuthSessionsLogout,
    trellisAuth.AuthSessionsMe,
    trellisAuth.AuthUsersResolve,
  ],
}));

Deno.test("auth descriptors record explicit rpc uses", () => {
  assertEquals(
    getContractRuntime(app).actions.map(({ action }) => action.name),
    [
      "Auth.Identities.List",
      "Auth.Sessions.Logout",
      "Auth.Sessions.Me",
      "Auth.Users.Resolve",
    ],
  );
});

Deno.test("trellis auth contract exposes portal-scoped routes only", () => {
  const rpc = getContractRuntime(trellisAuth).ownedApi.rpc;
  assert("Auth.Portals.Get" in rpc);
  assert("Auth.Portals.Routes.Put" in rpc);
  assert("Auth.Portals.Routes.Remove" in rpc);
  assert(!("Auth.Portals.LoginRoutes.List" in rpc));
  assert(!("Auth.Portals.LoginRoutes.Put" in rpc));
  assert(!("Auth.Portals.LoginRoutes.Remove" in rpc));
});

Deno.test("trellis auth resolves users without adding directory search", () => {
  const rpc = getContractRuntime(trellisAuth).ownedApi.rpc;
  assertEquals(rpc["Auth.Users.Resolve"].callerCapabilities, []);
  assertEquals(rpc["Auth.Users.List"].callerCapabilities, ["admin"]);
  assert(!("Auth.Users.Search" in rpc));
});
