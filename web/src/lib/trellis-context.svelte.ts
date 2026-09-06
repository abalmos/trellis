import {
  createTrellisApp,
  type TrellisClientFor,
} from "@qlever-llc/trellis-svelte";
import { participants } from "trellis-web-generated";
import { APP_CONFIG } from "./config.ts";

export type TrellisConsoleClient = TrellisClientFor<
  typeof participants.appConsole.participant
>;

let selectedTrellisUrl: string | undefined = APP_CONFIG.authUrl;

/** Sets the Trellis URL selected for the console's provider connection. */
export function setSelectedTrellisUrl(trellisUrl: string | undefined): void {
  selectedTrellisUrl = trellisUrl;
}

export const trellisApp = createTrellisApp({
  participant: participants.appConsole.participant,
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
