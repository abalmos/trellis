import {
  AuthEventsValidateResponseSchema,
  AuthEventsValidateSchema,
  AuthRequestsValidateResponseSchema,
  AuthRequestsValidateSchema,
} from "./auth/protocol.ts";
import { type RuntimeApi, schema } from "./contract_support/runtime.ts";
import type { StaticDecode } from "typebox";

// Transitional central validation is removed with local proof-v2 validation in Milestone 10.
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
    "Auth.Events.Validate": {
      subject: "rpc.v1.Auth.Events.Validate",
      input: schema<StaticDecode<typeof AuthEventsValidateSchema>>(
        AuthEventsValidateSchema,
      ),
      output: schema<StaticDecode<typeof AuthEventsValidateResponseSchema>>(
        AuthEventsValidateResponseSchema,
      ),
      callerCapabilities: ["service"],
      authRequired: false,
      declaredErrorTypes: ["AuthError", "ValidationError", "UnexpectedError"],
    },
  },
  operations: {},
  events: {},
  feeds: {},
  subjects: {},
} as const satisfies RuntimeApi;
