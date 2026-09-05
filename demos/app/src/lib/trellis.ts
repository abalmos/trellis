import { env } from "$env/dynamic/public";
import {
  createTrellisApp,
  type TrellisClientFor,
} from "@qlever-llc/trellis-svelte";
import { participant } from "../../.trellis/ts/participants/demo-app/mod.ts";

export type TrellisDemoAppClient = TrellisClientFor<typeof participant>;

const defaultTrellisUrl = "http://localhost:3000";

export const trellisUrl = new URL(
  env.PUBLIC_TRELLIS_URL?.trim() || defaultTrellisUrl,
)
  .toString()
  .replace(/\/$/, "");

export { participant };

export const trellisApp = createTrellisApp({
  participant,
  trellisUrl,
});

export function getTrellis(): TrellisDemoAppClient {
  return trellisApp.getTrellis();
}

export function getConnection() {
  return trellisApp.getConnection();
}
