from pathlib import Path

p = Path("ts/packages/trellis/contract_support/protocol_artifacts.ts")
text = p.read_text()
marker = "function compileParticipant(\n"
if text.count(marker) != 1:
    raise RuntimeError("compileParticipant anchor changed")
normalization = r'''function sortedUniqueStrings(
  value: JsonValue | undefined,
  path: string,
): JsonValue[] | undefined {
  if (value === undefined) return undefined;
  if (!Array.isArray(value)) throw new Error(`${path} must be an array`);
  const strings = value.map((item) => {
    if (typeof item !== "string") throw new Error(`${path} must contain only strings`);
    return item;
  });
  return [...new Set(strings)].sort(compareProtocolStrings);
}

function normalizeSelectionList(parent: JsonObject | undefined, key: string, path: string): void {
  if (!parent || !(key in parent)) return;
  const normalized = sortedUniqueStrings(parent[key], path)!;
  if (normalized.length === 0) delete parent[key];
  else parent[key] = normalized;
}

function deleteEmptyObject(parent: JsonObject, key: string): void {
  const value = object(parent[key]);
  if (value && Object.keys(value).length === 0) delete parent[key];
}

function normalizeUsedApi(used: JsonObject, path: string): void {
  const rpc = object(used.rpc);
  normalizeSelectionList(rpc, "call", `${path}.rpc.call`);
  deleteEmptyObject(used, "rpc");
  const operations = object(used.operations);
  normalizeSelectionList(operations, "invoke", `${path}.operations.invoke`);
  normalizeSelectionList(operations, "observe", `${path}.operations.observe`);
  normalizeSelectionList(operations, "cancel", `${path}.operations.cancel`);
  const control = object(operations?.control);
  if (control) {
    for (const [operation, signals] of Object.entries(control)) {
      const normalized = sortedUniqueStrings(signals, `${path}.operations.control.${operation}`)!;
      if (normalized.length === 0) delete control[operation];
      else control[operation] = normalized;
    }
    if (Object.keys(control).length === 0) delete operations!.control;
  }
  deleteEmptyObject(used, "operations");
  for (const [section, keys] of [["events", ["publish", "subscribe"]], ["feeds", ["subscribe"]], ["state", ["read", "write"]]] as const) {
    const selection = object(used[section]);
    for (const key of keys) normalizeSelectionList(selection, key, `${path}.${section}.${key}`);
    deleteEmptyObject(used, section);
  }
}

function normalizeParticipant(participant: JsonObject): JsonObject {
  const uses = object(participant.uses);
  if (uses) {
    for (const requirement of ["required", "optional"] as const) {
      const group = object(uses[requirement]);
      if (!group) continue;
      for (const [alias, value] of Object.entries(group)) {
        const used = object(value);
        if (!used) throw new Error(`uses.${requirement}.${alias} must be an object`);
        normalizeUsedApi(used, `uses.${requirement}.${alias}`);
      }
      if (Object.keys(group).length === 0) delete uses[requirement];
    }
    if (Object.keys(uses).length === 0) delete participant.uses;
  }
  const consumers = object(participant.eventConsumers);
  if (consumers) {
    for (const [name, value] of Object.entries(consumers)) {
      const consumer = object(value);
      if (!consumer) throw new Error(`eventConsumers.${name} must be an object`);
      const events = object(consumer.events);
      if (events) {
        for (const [alias, selected] of Object.entries(events)) {
          events[alias] = sortedUniqueStrings(selected, `eventConsumers.${name}.events.${alias}`)!;
        }
      }
      if (consumer.replay === "new") delete consumer.replay;
      if (consumer.ordering === "strict") delete consumer.ordering;
    }
  }
  const state = object(participant.state);
  if (state) for (const value of Object.values(state)) { const definition = object(value); if (definition) deleteEmptyObject(definition, "acceptedVersions"); }
  const queues = object(participant.jobQueues);
  if (queues) for (const value of Object.values(queues)) { const queue = object(value); if (!queue) continue; for (const key of ["progress", "logs", "dlq"]) if (queue[key] === false) delete queue[key]; }
  const resources = object(participant.resources);
  const kv = object(resources?.kv);
  if (kv) for (const value of Object.values(kv)) { const resource = object(value); if (!resource) continue; if (resource.required === true) delete resource.required; if (resource.history === 1) delete resource.history; if (resource.ttlMs === 0) delete resource.ttlMs; }
  const stores = object(resources?.store);
  if (stores) for (const value of Object.values(stores)) { const resource = object(value); if (!resource) continue; if (resource.required === true) delete resource.required; if (resource.ttlMs === 0) delete resource.ttlMs; }
  if (resources) { deleteEmptyObject(resources, "kv"); deleteEmptyObject(resources, "store"); if (Object.keys(resources).length === 0) delete participant.resources; }
  return participant;
}

'''
text = text.replace(marker, normalization + marker, 1)
anchor = "  return participant;\n}\n\nfunction compareProtocolStrings"
if text.count(anchor) != 1: raise RuntimeError("compileParticipant return anchor changed")
text = text.replace(anchor, "  return normalizeParticipant(participant);\n}\n\nfunction compareProtocolStrings", 1)
p.write_text(text)
