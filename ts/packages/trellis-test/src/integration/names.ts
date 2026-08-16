/**
 * Returns a stable ASCII slug safe for contract IDs, subjects, deployment IDs,
 * and participant or resource names.
 *
 * The slug keeps ASCII letters, digits, `_`, and `-`; replaces `.` with `-`;
 * and replaces all other characters with `-`.
 */
export function integrationSlug(caseId: string): string {
  return caseId.replaceAll(".", "-").replaceAll(/[^a-zA-Z0-9_-]/g, "-");
}

/** Returns a deterministic deployment ID for one case in a shared runtime run. */
export function caseDeploymentId(runId: string, caseId: string): string {
  return `js-it-${runId}-${integrationSlug(caseId)}`;
}

/** Prefixes a deterministic case slug with a local participant or resource prefix. */
export function caseScopedName(prefix: string, caseId: string): string {
  return `${prefix}-${integrationSlug(caseId)}`;
}

/** Creates a NATS subject scoped to one integration test case. */
export function caseScopedSubject(
  prefix: string,
  caseId: string,
  suffix: string,
): string {
  return `${prefix}.${integrationSlug(caseId)}.${suffix}`;
}

/** Creates a contract ID scoped to one integration test case. */
export function caseScopedContractId(base: string, caseId: string): string {
  return `${base}.${integrationSlug(caseId)}@v1`;
}

/** Prefixes action names so protocol-derived subjects are unique to one case. */
export function caseScopedActions<const T extends Record<string, unknown>>(
  caseId: string,
  actions: T,
): T {
  const prefix = caseActionPrefix(caseId);
  return Object.fromEntries(
    Object.entries(actions).map((
      [name, action],
    ) => [`${prefix}.${name}`, action]),
  ) as T;
}

/** Returns the protocol action name corresponding to one case-scoped alias. */
export function caseScopedActionName(caseId: string, name: string): string {
  return `${caseActionPrefix(caseId)}.${name}`;
}

/** Restores the authored alias for a case-scoped protocol action name. */
export function unscopedCaseActionName(
  contract: object,
  name: string,
): string {
  const prefix = Reflect.get(contract, caseActionPrefixSymbol);
  return typeof prefix === "string" && name.startsWith(`${prefix}.`)
    ? name.slice(prefix.length + 1)
    : name;
}

/** Preserves static typed action aliases after case-scoping their authored names. */
export function aliasCaseScopedActions<C extends object>(
  caseId: string,
  contract: C,
): C {
  const prefix = caseActionPrefix(caseId);
  return new Proxy(contract, {
    get(target, property, receiver) {
      if (property === caseActionPrefixSymbol) return prefix;
      if (typeof property !== "string" || property in target) {
        return Reflect.get(target, property, receiver);
      }
      return Reflect.get(target, `${prefix}${property}`, receiver);
    },
  });
}

/** Preserves static generated runtime methods for a case-scoped contract. */
export function aliasCaseScopedRuntime<C extends object, R extends object>(
  contract: C,
  runtime: R,
): R {
  const prefix = Reflect.get(contract, caseActionPrefixSymbol);
  if (typeof prefix !== "string") return runtime;
  return new Proxy(runtime, {
    get(target, property, receiver) {
      if (typeof property !== "string" || property in target) {
        return Reflect.get(target, property, receiver);
      }
      const verb = ["handle", "publish"].find((candidate) =>
        property.startsWith(candidate)
      );
      const scoped = verb === undefined
        ? `${prefix[0].toLowerCase()}${prefix.slice(1)}${
          property[0].toUpperCase()
        }${property.slice(1)}`
        : `${verb}${prefix}${property.slice(verb.length)}`;
      return Reflect.get(target, scoped, receiver);
    },
  });
}

function caseActionPrefix(caseId: string): string {
  return `Case${caseScopeToken(caseId)}`;
}

/** Returns one deterministic lowercase alphanumeric scope token. */
export function caseScopeToken(value: string): string {
  let hash = 0x811c9dc5;
  for (const byte of new TextEncoder().encode(value)) {
    hash = Math.imul(hash ^ byte, 0x01000193) >>> 0;
  }
  return hash.toString(36);
}
const caseActionPrefixSymbol = Symbol("trellis.integration.caseActionPrefix");
