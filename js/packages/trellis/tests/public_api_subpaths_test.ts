import { assertEquals } from "@std/assert";
import { Type } from "typebox";
import { getContractRuntime } from "../contract_support/contract_runtime.ts";

import { defineServiceContract } from "@qlever-llc/trellis";
import * as authSdk from "@qlever-llc/trellis/sdk/auth";
import * as authSurface from "@qlever-llc/trellis/auth";
import * as authBrowserSurface from "@qlever-llc/trellis/auth/browser";
import * as healthSdk from "@qlever-llc/trellis/sdk/health";
import * as contracts from "@qlever-llc/trellis/contracts";
import * as coreSdk from "@qlever-llc/trellis/sdk/core";
import * as stateSdk from "@qlever-llc/trellis/sdk/state";
import * as deviceDeno from "@qlever-llc/trellis/device/deno";
import * as serviceSurface from "@qlever-llc/trellis/service";
import type { TrellisService as TrellisServiceType } from "@qlever-llc/trellis/service";
import { TrellisService as DenoTrellisService } from "@qlever-llc/trellis/service/deno";
import { TrellisService as NodeTrellisService } from "@qlever-llc/trellis/service/node";

// @ts-expect-error Service runtime internals must not be public fields.
type ServiceServerField = TrellisServiceType["server"];
// @ts-expect-error Service runtime internals must not be public fields.
type ServiceOperationsField = TrellisServiceType["operations"];
// @ts-expect-error Raw NATS handles must not be public service fields.
type ServiceNatsField = TrellisServiceType["nc"];

Deno.test("service and SDK subpaths expose the canonical wrapper API", () => {
  assertEquals("TrellisServer" in serviceSurface, false);
  assertEquals(typeof serviceSurface.TrellisService, "function");
  assertEquals(typeof serviceSurface.OutboxDispatcher, "function");
  assertEquals(typeof DenoTrellisService, "function");
  assertEquals(typeof NodeTrellisService, "function");
  assertEquals("connectInternal" in DenoTrellisService, false);
  assertEquals("connectInternal" in NodeTrellisService, false);
  assertEquals(authSdk.AuthSessionsMe.kind, "rpc");
  assertEquals(coreSdk.TrellisCatalog.kind, "rpc");
  assertEquals(healthSdk.HealthQuery.kind, "rpc");
  assertEquals(stateSdk.StateGet.kind, "rpc");
});

Deno.test("auth and device runtime subpaths retain depended-on helpers", () => {
  assertEquals(typeof authSurface.signSessionProofV1, "function");
  assertEquals(typeof authBrowserSurface.completeSessionLogout, "function");
  assertEquals(typeof authBrowserSurface.fetchPortalFlowState, "function");
  assertEquals(typeof deviceDeno.checkDeviceActivation, "function");
  assertEquals("openDeviceActivationStateStore" in deviceDeno, false);
  assertEquals("resolveDeviceActivationStatePath" in deviceDeno, false);
});

Deno.test("contracts subpath exposes only kind-specific contract helpers", () => {
  assertEquals("defineContract" in contracts, false);
  assertEquals(typeof contracts.defineAppContract, "function");
  assertEquals(typeof contracts.defineAgentContract, "function");
  assertEquals(typeof contracts.defineDeviceContract, "function");
  assertEquals(typeof contracts.defineServiceContract, "function");
  assertEquals(typeof contracts.CursorQuerySchema, "object");
  assertEquals(typeof contracts.CursorPageSchema, "function");
  assertEquals(typeof contracts.normalizeCursorQuery, "function");
  assertEquals(typeof contracts.normalizePageQuery, "function");

  const contract = contracts.defineServiceContract(
    {
      schemas: {
        Ping: Type.Object({ ok: Type.Literal(true) }),
      },
    },
    (ref) => ({
      id: "example.device@v1",
      displayName: "Example Device",
      description: "Example device contract.",
      rpc: {
        "Example.Ping": {
          version: "v1",
          input: ref.schema("Ping"),
          output: ref.schema("Ping"),
          errors: [],
        },
      },
    }),
  );

  assertEquals(typeof contract.CONTRACT_ID, "string");
  assertEquals(
    typeof getContractRuntime(contract).api.rpc["Example.Ping"].subject,
    "string",
  );
});
