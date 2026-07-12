import {
  AuthRequestsValidateResponseSchema,
  AuthRequestsValidateSchema,
} from "./auth/protocol.ts";
import {
  TrellisBindingsGetRequestSchema,
  TrellisBindingsGetResponseSchema,
} from "./models/trellis/rpc/TrellisBindingsGet.ts";
import { actionRuntimeDescriptor } from "./contract_support/descriptors.ts";
import { type RuntimeApi, schema } from "./contract_support/runtime.ts";
import { AuthEventsValidate } from "./sdk/auth.ts";
import { TrellisCatalog } from "./sdk/core.ts";
import type { StaticDecode } from "typebox";

export const AUTH_SESSION_API = {
  rpc: {
    "Auth.Requests.Validate": {
      subject: "rpc.v1.Auth.Requests.Validate",
      input: schema<StaticDecode<typeof AuthRequestsValidateSchema>>(
        AuthRequestsValidateSchema,
      ),
      output: schema<StaticDecode<typeof AuthRequestsValidateResponseSchema>>(
        AuthRequestsValidateResponseSchema,
      ),
      callerCapabilities: ["service"],
      authRequired: false,
      declaredErrorTypes: ["AuthError", "ValidationError", "UnexpectedError"],
    },
    "Auth.Events.Validate": actionRuntimeDescriptor(AuthEventsValidate),
  },
  operations: {},
  events: {},
  feeds: {},
  subjects: {},
} as const satisfies RuntimeApi;

export const CORE_SESSION_API = {
  rpc: {
    "Trellis.Catalog": actionRuntimeDescriptor(TrellisCatalog),
    "Trellis.Bindings.Get": {
      subject: "rpc.v1.Trellis.Bindings.Get",
      input: schema<StaticDecode<typeof TrellisBindingsGetRequestSchema>>(
        TrellisBindingsGetRequestSchema,
      ),
      output: schema<StaticDecode<typeof TrellisBindingsGetResponseSchema>>(
        TrellisBindingsGetResponseSchema,
      ),
      callerCapabilities: ["service"],
      declaredErrorTypes: ["NotFoundError", "UnexpectedError"],
    },
  },
  operations: {},
  events: {},
  feeds: {},
  subjects: {},
} as const satisfies RuntimeApi;
