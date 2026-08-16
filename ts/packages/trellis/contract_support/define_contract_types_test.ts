import { Type } from "typebox";

import {
  type ContractSourceRpcMethod,
  defineAgentContract,
  defineAppContract,
  defineDeviceContract,
  defineError,
  defineServiceContract,
  jobs,
  kv,
  type SerializableErrorData,
  state,
  store,
} from "./mod.ts";
import { getContractRuntime } from "./contract_runtime.ts";
import type {
  OperationHandlerErrorOf,
  OperationOutputOf,
  OperationProgressOf,
  OperationRuntimeHandle,
  OperationUpdateOf,
} from "../session.ts";

const EmptySchema = Type.Object({});
const StringSchema = Type.Object({ value: Type.String() });
const ProgressSchema = Type.Object({ step: Type.String() });
const BuilderFailed = defineError({
  type: "BuilderFailed",
  fields: {},
  message: "Builder failed",
});

type Assert<T extends true> = T;
type Not<T extends boolean> = T extends true ? false : true;
type Extends<T, U> = T extends U ? true : false;
type HasKey<T, K extends PropertyKey> = K extends keyof T ? true : false;
type HasMember<T, U> = U extends T ? true : false;
type HasSubject<T, TKey extends PropertyKey> = TKey extends keyof T ? true
  : false;

type BuilderFailedData = Parameters<typeof BuilderFailed.fromSerializable>[0];
type _BuilderFailedDataExtendsSerializableErrorData = Assert<
  Extends<BuilderFailedData, SerializableErrorData>
>;

const authSchemas = {
  Empty: EmptySchema,
  StringValue: StringSchema,
} as const;

function schemaRef<
  TSchemas extends Record<string, unknown>,
  const TName extends keyof TSchemas & string,
>(
  schema: TName,
) {
  return { schema } as const;
}

const auth = defineServiceContract(
  {
    schemas: authSchemas,
  },
  () => ({
    id: "trellis.auth@v1",
    displayName: "Trellis Auth",
    description: "Expose Trellis auth RPCs and events for tests.",
    exports: {
      schemas: ["StringValue"],
    },
    rpc: {
      "Auth.Sessions.Me": {
        version: "v1",
        input: schemaRef<typeof authSchemas, "Empty">("Empty"),
        output: schemaRef<typeof authSchemas, "StringValue">("StringValue"),
      },
      "Auth.Sessions.Logout": {
        version: "v1",
        input: schemaRef<typeof authSchemas, "Empty">("Empty"),
        output: schemaRef<typeof authSchemas, "Empty">("Empty"),
      },
    },
    events: {
      "Auth.Connections.Opened": {
        version: "v1",
        event: schemaRef<typeof authSchemas, "StringValue">("StringValue"),
      },
    },
    feeds: {
      "Auth.ConnectFeed": {
        version: "v1",
        input: schemaRef<typeof authSchemas, "Empty">("Empty"),
        event: schemaRef<typeof authSchemas, "StringValue">("StringValue"),
        capabilities: { subscribe: ["service"] },
      },
    },
  }),
);

const auditSchemas = {
  Empty: EmptySchema,
  StringValue: StringSchema,
} as const;

const audit = defineServiceContract(
  { schemas: auditSchemas },
  () => ({
    id: "trellis.audit@v1",
    displayName: "Audit",
    description: "Expose audit RPCs and subscribe to auth events for tests.",
    uses: [
      auth.AuthSessionsMe,
      auth.AuthConnectionsOpened.subscribe,
      auth.AuthConnectFeed,
    ],
    rpc: {
      "Audit.List": {
        version: "v1",
        input: schemaRef<typeof auditSchemas, "Empty">("Empty"),
        output: schemaRef<typeof auditSchemas, "StringValue">("StringValue"),
      },
    },
    events: {
      "Audit.Recorded": {
        version: "v1",
        event: schemaRef<typeof auditSchemas, "StringValue">("StringValue"),
      },
    },
  }),
);

getContractRuntime(audit).ownedApi.rpc["Audit.List"].subject;
getContractRuntime(audit).usedApi.rpc["Auth.Sessions.Me"].subject;
getContractRuntime(audit).usedApi.events["Auth.Connections.Opened"].subject;
getContractRuntime(audit).usedApi.feeds["Auth.ConnectFeed"].subject;
getContractRuntime(audit).api.rpc["Audit.List"].subject;
getContractRuntime(audit).api.rpc["Auth.Sessions.Me"].subject;
getContractRuntime(audit).api.feeds["Auth.ConnectFeed"].subject;

type _AuthContractDoesNotExposeRawSubjects = Assert<
  Not<HasKey<typeof auth, "subjects">>
>;
type _AuthContractDoesNotExposeCatalog = Assert<
  Not<HasKey<typeof auth, "catalog">>
>;

const dashboard = defineAppContract(() => ({
  id: "trellis.dashboard@v1",
  displayName: "Dashboard",
  description: "Consume audit events in contract typing tests.",
  uses: [audit.AuditRecorded.subscribe],
}));

getContractRuntime(dashboard).usedApi.events["Audit.Recorded"].subject;

const preferencesSchemas = {
  Preferences: Type.Object({ theme: Type.String() }),
  Draft: Type.Object({ title: Type.String() }),
} as const;

const preferencesApp = defineAppContract(
  { schemas: preferencesSchemas },
  (ref) => ({
    id: "trellis.preferences@v1",
    displayName: "Preferences",
    description: "Declare named state stores for client contracts.",
    uses: [state({
      preferences: {
        kind: "value",
        schema: ref.schema("Preferences"),
      },
      drafts: {
        kind: "map",
        schema: ref.schema("Draft"),
      },
    })],
  }),
);

getContractRuntime(preferencesApp).usedApi.rpc["State.Get"].subject;
getContractRuntime(preferencesApp).usedApi.rpc["State.Put"].subject;
getContractRuntime(preferencesApp).usedApi.rpc["State.Delete"].subject;
getContractRuntime(preferencesApp).usedApi.rpc["State.List"].subject;
getContractRuntime(preferencesApp).api.rpc["State.Get"].subject;

if (false) {
  defineAppContract(
    { schemas: preferencesSchemas },
    (ref) => ({
      id: "trellis.invalid-state@v1",
      displayName: "Invalid State",
      description: "Should fail type checking.",
      // @ts-expect-error invalid state feature is not a valid contract selection
      uses: [state({
        // @ts-expect-error state feature declarations require kind
        prefs: {
          schema: ref.schema("Preferences"),
        },
        drafts: {
          // @ts-expect-error state kind is limited to value or map
          kind: "set",
          schema: ref.schema("Draft"),
        },
      })],
    }),
  );
}

const billingSchemas = {
  Empty: EmptySchema,
  Progress: StringSchema,
  Result: StringSchema,
  SelectReason: Type.Object({ reason: Type.String() }),
} as const;

const billingCapabilities = {
  "refund": {
    displayName: "Refund billing",
    description: "Start billing refunds.",
  },
  "read": {
    displayName: "Read billing",
    description: "Read billing operation status.",
  },
  "cancel": {
    displayName: "Cancel billing",
    description: "Cancel billing operations.",
  },
  "control": {
    displayName: "Control billing",
    description: "Control billing operations.",
  },
} as const;

const billing = defineServiceContract(
  { schemas: billingSchemas },
  () => ({
    id: "trellis.billing@v1",
    displayName: "Billing",
    description: "Expose billing operations for contract typing tests.",
    capabilities: billingCapabilities,
    operations: {
      "Billing.Refund": {
        version: "v1",
        input: schemaRef<typeof billingSchemas, "Empty">("Empty"),
        progress: schemaRef<typeof billingSchemas, "Progress">("Progress"),
        output: schemaRef<typeof billingSchemas, "Result">("Result"),
        capabilities: {
          call: ["refund"],
          observe: ["read"],
          cancel: ["cancel"],
          control: ["control"],
        },
        signals: {
          selectReason: {
            input: schemaRef<typeof billingSchemas, "SelectReason">(
              "SelectReason",
            ),
          },
        },
        cancel: true,
      },
    },
  }),
);

const paymentsSchemas = {
  Empty: EmptySchema,
  Result: StringSchema,
} as const;

const payments = defineServiceContract(
  { schemas: paymentsSchemas },
  () => ({
    id: "trellis.payments@v1",
    displayName: "Payments",
    description: "Consume billing operations for contract typing tests.",
    uses: [billing.BillingRefund],
    operations: {
      "Payments.Capture": {
        version: "v1",
        input: schemaRef<typeof paymentsSchemas, "Empty">("Empty"),
        output: schemaRef<typeof paymentsSchemas, "Result">("Result"),
      },
    },
  }),
);

getContractRuntime(payments).ownedApi.operations["Payments.Capture"].subject;
getContractRuntime(payments).usedApi.operations["Billing.Refund"].subject;
getContractRuntime(payments).api.operations["Payments.Capture"].subject;
getContractRuntime(payments).api.operations["Billing.Refund"].subject;
getContractRuntime(payments).usedApi.operations["Billing.Refund"].signals
  ?.selectReason.input;

const paymentsRuntime = getContractRuntime(payments);
type _PaymentsDoesNotExposeBillingWriteoff = Assert<
  Not<HasKey<typeof paymentsRuntime.api.operations, "Billing.Writeoff">>
>;
type _BillingDoesNotExposeWriteoff = Assert<
  Not<HasKey<typeof billing, "BillingWriteoff">>
>;

const inlineSchemaContract = defineServiceContract(
  {
    schemas: {
      Empty: EmptySchema,
      Progress: ProgressSchema,
      Result: StringSchema,
    },
  },
  () => ({
    id: "trellis.inline-schemas@v1",
    displayName: "Inline Schemas",
    description: "Use inline schema refs without a local helper.",
    rpc: {
      "Inline.Run": {
        version: "v1",
        input: { schema: "Empty" },
        output: { schema: "Result" },
      },
    },
    operations: {
      "Inline.Import": {
        version: "v1",
        input: { schema: "Empty" },
        progress: { schema: "Progress" },
        output: { schema: "Result" },
      },
    },
    uses: [jobs({
      import: {
        payload: { schema: "Empty" },
        result: { schema: "Result" },
      },
    })],
  }),
);

getContractRuntime(inlineSchemaContract).ownedApi.rpc["Inline.Run"].subject;
getContractRuntime(inlineSchemaContract).ownedApi.operations["Inline.Import"]
  .subject;

const topLevelJobsContract = defineServiceContract(
  {
    schemas: {
      Empty: EmptySchema,
      Result: StringSchema,
    },
  },
  () => ({
    id: "trellis.top-level-jobs@v1",
    displayName: "Top Level Jobs",
    description: "Ensure jobs are typed as a first-class contract surface.",
    uses: [jobs({
      import: {
        payload: { schema: "Empty" },
        result: { schema: "Result" },
      },
      export: {
        payload: { schema: "Empty" },
      },
    })],
  }),
);

if (false) {
  defineServiceContract(
    {
      schemas: {
        Empty: EmptySchema,
      },
    },
    () => ({
      id: "trellis.invalid-jobs-resource@v1",
      displayName: "Invalid Jobs Resource",
      description: "Should fail type checking.",
      // @ts-expect-error runtime features must be declared in uses
      resources: {
        jobs: {
          queues: {
            import: {
              payload: { schema: "Empty" },
            },
          },
        },
      },
    }),
  );
}

const transferSchemas = {
  UploadInput: Type.Object({
    key: Type.String(),
    contentType: Type.Optional(Type.String()),
  }),
} as const;

const transferContract = defineServiceContract(
  { schemas: transferSchemas },
  (ref) => ({
    id: "trellis.transfer@v1",
    displayName: "Transfer",
    description: "Exercise transfer-capable operation typing.",
    uses: [
      kv({
        uploadsByKey: {
          purpose: "Track upload metadata",
          schema: ref.schema("UploadInput"),
        },
      }),
      store({
        uploads: {
          purpose: "Temporary uploads",
          ttlMs: 60_000,
          maxObjectBytes: 1024,
        },
      }),
    ],
    operations: {
      "Demo.Files.Upload": {
        version: "v1",
        input: { schema: "UploadInput" },
        output: { schema: "UploadInput" },
        transfer: {
          direction: "send",
          store: "uploads",
          key: "/key",
          contentType: "/contentType",
          expiresInMs: 60_000,
        },
      },
    },
  }),
);

getContractRuntime(transferContract).ownedApi.operations["Demo.Files.Upload"]
  .transfer?.store;

const builderContract = defineServiceContract(
  {
    schemas: {
      Empty: EmptySchema,
      Result: StringSchema,
      Update: StringSchema,
    },
    errors: {
      BuilderFailed,
    },
  },
  (ref) => ({
    id: "trellis.builder@v1",
    displayName: "Builder Contract",
    description: "Exercise the builder-style contract authoring API.",
    rpc: {
      "Builder.Run": {
        version: "v1",
        input: ref.schema("Empty"),
        output: ref.schema("Result"),
        errors: [ref.error("BuilderFailed"), ref.error("UnexpectedError")],
      },
    },
    operations: {
      "Builder.Process": {
        version: "v1",
        input: ref.schema("Empty"),
        update: ref.schema("Update"),
        output: ref.schema("Result"),
        errors: [ref.error("BuilderFailed")],
        capabilities: { call: [] },
      },
    },
  }),
);

getContractRuntime(builderContract).ownedApi.rpc["Builder.Run"].subject;
getContractRuntime(builderContract).ownedApi.operations["Builder.Process"]
  .subject;

const builderRuntime = getContractRuntime(builderContract);
type BuilderOwnedApi = typeof builderRuntime.ownedApi;
type BuilderProcessHandle = OperationRuntimeHandle<
  OperationProgressOf<BuilderOwnedApi, "Builder.Process">,
  OperationOutputOf<BuilderOwnedApi, "Builder.Process">,
  OperationHandlerErrorOf<BuilderOwnedApi, "Builder.Process">,
  OperationUpdateOf<BuilderOwnedApi, "Builder.Process">
>;
type BuilderFailedError = ReturnType<typeof BuilderFailed.fromSerializable>;
type _BuilderOperationAcceptsDeclaredError = Assert<
  Extends<
    BuilderFailedError,
    OperationHandlerErrorOf<BuilderOwnedApi, "Builder.Process">
  >
>;

function acceptBuilderOperationError(
  handle: BuilderProcessHandle,
  error: BuilderFailedError,
) {
  return handle.fail(error);
}

acceptBuilderOperationError;

const appContract = defineAppContract(() => ({
  id: "trellis.builder-app@v1",
  displayName: "Builder App",
  description: "Exercise the app helper.",
  uses: [auth.AuthSessionsMe],
}));

getContractRuntime(appContract).usedApi.rpc["Auth.Sessions.Me"].subject;

const deviceContract = defineDeviceContract(() => ({
  id: "trellis.builder-device@v1",
  displayName: "Builder Device",
  description: "Exercise the device helper.",
  uses: [auth.AuthSessionsLogout],
}));

getContractRuntime(deviceContract).usedApi.rpc["Auth.Sessions.Logout"].subject;

if (false) {
  const invalidRpcSchemas = {
    Empty: EmptySchema,
    Result: StringSchema,
  } as const;

  const invalidRpcMethod: ContractSourceRpcMethod<
    keyof typeof invalidRpcSchemas & string
  > = {
    version: "v1",
    // @ts-expect-error rpc schema refs must use local schema keys
    input: { schema: "Missing" },
    output: { schema: "Result" },
  };

  invalidRpcMethod;

  defineServiceContract(
    {
      schemas: {
        Empty: EmptySchema,
      },
    },
    () => ({
      id: "trellis.invalid-job-schema@v1",
      displayName: "Invalid Job Schema",
      description: "Should fail type checking.",
      // @ts-expect-error job queue schema refs must use local schema keys
      uses: [jobs({
        import: {
          payload: { schema: "Missing" },
        },
      })],
    }),
  );

  defineServiceContract(
    {
      schemas: {
        Empty: EmptySchema,
      },
    },
    (ref) => ({
      id: "trellis.invalid-kv-schema@v1",
      displayName: "Invalid KV Schema",
      description: "Should fail type checking.",
      uses: [kv({
        cache: {
          purpose: "Broken KV schema ref",
          // @ts-expect-error kv resource schema refs must use local schema keys
          schema: ref.schema("Missing"),
        },
      })],
    }),
  );

  defineServiceContract(
    {
      schemas: {
        Empty: EmptySchema,
      },
      errors: {
        BuilderFailed,
      },
    },
    (ref) => ({
      id: "trellis.invalid-builder@v1",
      displayName: "Invalid Builder",
      description: "Should fail type checking.",
      rpc: {
        "Builder.Run": {
          version: "v1",
          // @ts-expect-error builder schema refs must use local schema keys
          input: ref.schema("Missing"),
          output: ref.schema("Empty"),
          errors: [
            ref.error("BuilderFailed"),
            // @ts-expect-error builder error refs must use local or builtin error names
            ref.error("MissingError"),
          ],
        },
      },
    }),
  );
  defineServiceContract(
    {
      schemas: {
        Empty: EmptySchema,
      },
      errors: {
        BuilderFailed,
      },
    },
    (ref) => ({
      id: "trellis.invalid-operation-error@v1",
      displayName: "Invalid Operation Error",
      description: "Should fail type checking.",
      operations: {
        "Example.Run": {
          version: "v1",
          input: ref.schema("Empty"),
          output: ref.schema("Empty"),
          errors: [
            ref.error("BuilderFailed"),
            // @ts-expect-error operation error refs must use local or builtin error names
            ref.error("MissingError"),
          ],
          capabilities: { call: [] },
        },
      },
    }),
  );

  defineAppContract(() => ({
    id: "trellis.invalid-app@v1",
    displayName: "Invalid App",
    description: "Should fail type checking.",
    // @ts-expect-error app contracts may not declare local schemas
    schemas: { Empty: EmptySchema },
  }));

  defineServiceContract(
    {
      // @ts-expect-error contract exports must be declared in the callback body
      exports: { schemas: ["Empty"] },
    },
    () => ({
      id: "trellis.invalid-service-exports@v1",
      displayName: "Invalid Service Exports",
      description: "Should fail type checking.",
    }),
  );

  defineAppContract(
    {
      // @ts-expect-error contract exports must be declared in the callback body
      exports: { schemas: ["Empty"] },
    },
    () => ({
      id: "trellis.invalid-app-exports@v1",
      displayName: "Invalid App Exports",
      description: "Should fail type checking.",
    }),
  );

  defineAgentContract(
    {
      // @ts-expect-error contract exports must be declared in the callback body
      exports: { schemas: ["Empty"] },
    },
    () => ({
      id: "trellis.invalid-agent-exports@v1",
      displayName: "Invalid Agent Exports",
      description: "Should fail type checking.",
    }),
  );

  defineDeviceContract(
    {
      // @ts-expect-error contract exports must be declared in the callback body
      exports: { schemas: ["Empty"] },
    },
    () => ({
      id: "trellis.invalid-device-exports@v1",
      displayName: "Invalid Device Exports",
      description: "Should fail type checking.",
    }),
  );

  defineServiceContract(
    {
      schemas: {
        Empty: EmptySchema,
      },
    },
    () => ({
      id: "trellis.invalid-raw-subjects@v1",
      displayName: "Invalid Raw Subjects",
      description: "Should fail type checking.",
      // @ts-expect-error raw subject ownership is not contract authoring API
      subjects: {
        Audit: { subject: "audit.raw" },
      },
    }),
  );

  defineDeviceContract(() => ({
    id: "trellis.invalid-device@v1",
    displayName: "Invalid Device",
    description: "Should fail type checking.",
    // @ts-expect-error device contracts may not declare resources
    resources: {
      store: {
        uploads: {
          purpose: "not allowed",
        },
      },
    },
  }));
}

Deno.test("kind-specific contract helper type coverage compiles", () => {});
