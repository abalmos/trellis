import { defineAppContract } from "../../packages/trellis/contract_support/mod.ts";
import * as trellisAuth from "@trellis/apis/trellis.auth";

function defineClient(id: string, displayName: string) {
  return defineAppContract(() => ({
    id,
    apiId: id,
    apiVersion: "1.0.0",
    displayName,
    description: "Exercises the default durable browser installation.",
    uses: [trellisAuth.AuthSessionsMe, trellisAuth.AuthSessionsLogout],
  }));
}

export const clientA = defineClient(
  "trellis.test.default-browser-a@v1",
  "Default browser A",
);
export const clientB = defineClient(
  "trellis.test.default-browser-b@v1",
  "Default browser B",
);
export const clientC = defineClient(
  "trellis.test.default-browser-c@v1",
  "Default browser C",
);
