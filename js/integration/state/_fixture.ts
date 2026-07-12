import { defineAppContract, state } from "@qlever-llc/trellis";
import * as trellisAuth from "@qlever-llc/trellis/sdk/auth";
import * as trellisState from "@qlever-llc/trellis/sdk/state";
import { Type } from "typebox";
import {
  caseScopedContractId,
  caseScopedName,
  integrationSlug,
} from "../_support/names.ts";

export function createStateFixture(caseId: string) {
  const slug = integrationSlug(caseId);
  const stateSchemas = {
    Preferences: Type.Object({
      theme: Type.String(),
      density: Type.String(),
    }),
    Draft: Type.Object({
      title: Type.String(),
      body: Type.String(),
    }),
  } as const;
  const stateSchemasV2 = {
    ...stateSchemas,
    PreferencesV2: Type.Object({
      theme: Type.String(),
      density: Type.String(),
      contrast: Type.String(),
    }),
    DraftV2: Type.Object({
      title: Type.String(),
      body: Type.String(),
      status: Type.String(),
    }),
  } as const;

  const clientContract = defineAppContract(
    { schemas: stateSchemas },
    (ref) => ({
      id: caseScopedContractId("trellis.integration.state-client", caseId),
      displayName: `Trellis Integration State Client (${slug})`,
      description: "Exercises generated contract-owned state store surfaces.",
      uses: [state({
        preferences: {
          kind: "value",
          schema: ref.schema("Preferences"),
          stateVersion: "preferences.v1",
        },
        drafts: {
          kind: "map",
          schema: ref.schema("Draft"),
          stateVersion: "drafts.v1",
        },
      })],
    }),
  );

  const clientContractV2 = defineAppContract(
    { schemas: stateSchemasV2 },
    (ref) => ({
      id: clientContract.CONTRACT_ID,
      displayName: `Trellis Integration State Client v2 (${slug})`,
      description:
        "Exercises generated contract-owned state store migration surfaces.",
      uses: [state({
        preferences: {
          kind: "value",
          schema: ref.schema("PreferencesV2"),
          stateVersion: "preferences.v2",
          acceptedVersions: { "preferences.v1": ref.schema("Preferences") },
        },
        drafts: {
          kind: "map",
          schema: ref.schema("DraftV2"),
          stateVersion: "drafts.v2",
          acceptedVersions: { "drafts.v1": ref.schema("Draft") },
        },
      })],
    }),
  );

  const adminContract = defineAppContract(() => ({
    id: caseScopedContractId("trellis.integration.state-admin", caseId),
    displayName: `Trellis Integration State Admin (${slug})`,
    description:
      "Admin participant for inspecting and deleting state through public generated RPCs.",
    uses: [
      trellisAuth.AuthSessionsList,
      trellisState.StateAdminDelete,
      trellisState.StateAdminGet,
      trellisState.StateAdminList,
    ],
  }));

  return {
    slug,
    adminContract,
    adminName: caseScopedName("state-fixture-admin", caseId),
    clientContract,
    clientContractV2,
    clientName: caseScopedName("state-fixture-client", caseId),
    clientV2Name: caseScopedName("state-fixture-client-v2", caseId),
    draftPrefix: caseScopedName("inspection", caseId),
    draftKey: caseScopedName("state-draft", caseId),
    limitPrefix: caseScopedName("limit-test", caseId),
  };
}
