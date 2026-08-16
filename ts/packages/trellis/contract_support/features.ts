/** Runtime facilities selected by a participant contract. */
export type RuntimeFeatureKind = "state" | "kv" | "store" | "jobs";

/** A portable declaration of one Trellis runtime facility. */
export type RuntimeFeatureDescriptor<
  TKind extends RuntimeFeatureKind = RuntimeFeatureKind,
  TConfig = unknown,
> = Readonly<{
  feature: TKind;
  config: TConfig;
}>;

function runtimeFeature<const TKind extends RuntimeFeatureKind, const TConfig>(
  feature: TKind,
  config: TConfig,
): RuntimeFeatureDescriptor<TKind, TConfig> {
  return Object.freeze({ feature, config });
}

/** Declares named participant state stores and enables their typed facade. */
export function state<const TConfig extends ContractSourceState>(
  config: TConfig,
): RuntimeFeatureDescriptor<"state", TConfig> {
  return runtimeFeature("state", config);
}

/** Declares named key-value resources and enables their typed handles. */
export function kv<
  const TConfig extends Record<string, ContractSourceKvResource>,
>(
  config: TConfig,
): RuntimeFeatureDescriptor<"kv", TConfig> {
  return runtimeFeature("kv", config);
}

/** Declares named object-store resources and enables their typed handles. */
export function store<
  const TConfig extends Record<string, ContractSourceStoreResource>,
>(
  config: TConfig,
): RuntimeFeatureDescriptor<"store", TConfig> {
  return runtimeFeature("store", config);
}

/** Declares service-private job types and enables their typed handles. */
export function jobs<const TConfig extends ContractSourceJobs>(
  config: TConfig,
): RuntimeFeatureDescriptor<"jobs", TConfig> {
  return runtimeFeature("jobs", config);
}
import type {
  ContractSourceJobs,
  ContractSourceKvResource,
  ContractSourceState,
  ContractSourceStoreResource,
} from "./mod.ts";
