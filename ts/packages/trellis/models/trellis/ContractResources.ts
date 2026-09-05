import {
  ContractJobQueueSchema,
  ContractJobsSchema,
  ContractKvResourceSchema,
  ContractResourceBindingsSchema,
  ContractResourcesSchema,
  ContractStoreResourceSchema,
  EventConsumerResourceBindingSchema,
  InstalledGeneratedServiceParticipantSchema,
  JobsQueueBindingSchema,
  JobsResourceBindingSchema,
  KvResourceBindingSchema,
  StoreResourceBindingSchema,
} from "../../participant.ts";
import type { Static } from "typebox";

export {
  ContractJobQueueSchema,
  ContractJobsSchema,
  ContractKvResourceSchema,
  ContractResourceBindingsSchema,
  ContractResourcesSchema,
  ContractStoreResourceSchema,
  EventConsumerResourceBindingSchema,
  InstalledGeneratedServiceParticipantSchema,
  JobsQueueBindingSchema,
  JobsResourceBindingSchema,
  KvResourceBindingSchema,
  StoreResourceBindingSchema,
};

export type ContractKvResource = Static<typeof ContractKvResourceSchema>;
export type ContractStoreResource = Static<typeof ContractStoreResourceSchema>;
export type ContractJobQueue = Static<typeof ContractJobQueueSchema>;
export type ContractJobs = Static<typeof ContractJobsSchema>;
export type ContractResources = Static<typeof ContractResourcesSchema>;
export type KvResourceBinding = Static<typeof KvResourceBindingSchema>;
export type StoreResourceBinding = Static<typeof StoreResourceBindingSchema>;
export type EventConsumerResourceBinding = Static<
  typeof EventConsumerResourceBindingSchema
>;
export type JobsQueueBinding = Static<typeof JobsQueueBindingSchema>;
export type JobsResourceBinding = Static<typeof JobsResourceBindingSchema>;
export type ContractResourceBindings = Static<
  typeof ContractResourceBindingsSchema
>;
export type InstalledGeneratedServiceParticipant = Static<
  typeof InstalledGeneratedServiceParticipantSchema
>;
