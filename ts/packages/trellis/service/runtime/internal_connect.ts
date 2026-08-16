import type { NatsConnection } from "@nats-io/nats-core";
import { createAuth } from "../../auth.ts";

import type { RuntimeApi } from "../../contract_support/runtime.ts";
import type { ContractKvMetadata } from "../../contract_support/mod.ts";
import {
  DEFAULT_RUNTIME_MAX_RECONNECT_ATTEMPTS,
  DEFAULT_SERVICE_RUNTIME_WAIT_ON_FIRST_CONNECT,
} from "../../runtime_transport.ts";
import type { TrellisServiceRuntimeDeps } from "./runtime.ts";
import {
  createConnectedService,
  type ResourceBindings,
  type Trellis,
  type TrellisServiceInternalConnectArgs,
  type TrellisServiceSession,
} from "./service.ts";

async function closeFailedServiceBootstrapConnection(
  nc: NatsConnection,
): Promise<void> {
  if (nc.isClosed()) {
    return;
  }

  try {
    await nc.drain();
  } catch {
    await nc.closed().catch(() => undefined);
  }
}

export async function connectTrellisServiceInternal<
  TOwnedApi extends RuntimeApi = RuntimeApi,
  TTrellisApi extends RuntimeApi = TOwnedApi,
  TKv extends ContractKvMetadata = {},
>(
  name: string,
  opts: TrellisServiceInternalConnectArgs<TOwnedApi, TTrellisApi, TKv>,
  deps: TrellisServiceRuntimeDeps,
): Promise<TrellisServiceSession<TOwnedApi, TTrellisApi, {}, TKv>> {
  const connectFn = deps.connect;

  const auth = opts.auth ??
    (opts.sessionKeySeed || opts.identity
      ? await createAuth({
        sessionKeySeed: opts.sessionKeySeed ?? opts.identity!.seed,
      })
      : undefined);
  if (!auth) {
    throw new Error(
      "TrellisService.connect requires either opts.auth or opts.sessionKeySeed",
    );
  }
  if (!opts.contractDigest) {
    throw new Error(
      "TrellisService.connect requires opts.contractDigest for NATS runtime auth",
    );
  }

  const authenticator = opts.nats.authenticator;

  const { authenticator: authTokenAuthenticator, inboxPrefix } = await auth
    .natsConnectOptions({
      sessionId: name,
      contextDigest: opts.authorizationContextDigest,
      jwt: "internal-service-test",
    });

  const nc = await connectFn({
    servers: opts.nats.servers,
    maxReconnectAttempts: DEFAULT_RUNTIME_MAX_RECONNECT_ATTEMPTS,
    waitOnFirstConnect: DEFAULT_SERVICE_RUNTIME_WAIT_ON_FIRST_CONNECT,
    inboxPrefix,
    authenticator: [authTokenAuthenticator, authenticator],
    ...(opts.nats.options ?? {}),
  });

  try {
    const bindings: ResourceBindings = { kv: {}, store: {} };
    const contractKv = opts.contractKv ?? ({} as TKv);

    return await createConnectedService<TOwnedApi, TTrellisApi, {}, TKv>({
      name,
      auth,
      nc,
      inboxPrefix,
      contextDigest: opts.authorizationContextDigest,
      contractId: opts.contractId,
      contractDigest: opts.contractDigest,
      contractJobs: {},
      contractKv,
      runtime: opts.runtime,
      bindings,
      healthIdentity: opts.healthIdentity,
    });
  } catch (cause) {
    await closeFailedServiceBootstrapConnection(nc);
    throw cause;
  }
}
