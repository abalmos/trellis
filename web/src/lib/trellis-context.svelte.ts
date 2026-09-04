import {
  createTrellisApp,
  type TrellisClientFor,
} from "@qlever-llc/trellis-svelte";
import { participant } from "../../contracts/console/.trellis/ts/participants/app-console/mod.ts";
import { APP_CONFIG } from "./config.ts";

export type TrellisConsoleClient = TrellisClientFor<typeof participant>;

let selectedTrellisUrl: string | undefined = APP_CONFIG.authUrl;

/** Sets the Trellis URL selected for the console's provider connection. */
export function setSelectedTrellisUrl(trellisUrl: string | undefined): void {
  selectedTrellisUrl = trellisUrl;
}

export const trellisApp = createTrellisApp({
  participant,
  trellisUrl: () => selectedTrellisUrl,
});

export function getTrellis(): TrellisConsoleClient {
  return trellisApp.getTrellis();
}

export function getAuthenticatedUser(trellis: TrellisConsoleClient) {
  return trellis.authSessionsMe({}).orThrow();
}

export function getConnection() {
  return trellisApp.getConnection();
}
