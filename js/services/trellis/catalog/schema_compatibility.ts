import type {
  ContractSchemaRef,
  ContractSchemas,
  JsonSchema,
} from "@qlever-llc/trellis/contracts";
import { canonicalizeJson, isJsonValue } from "@qlever-llc/trellis";

type JsonObject = { [key: string]: JsonSchema };

export type SchemaCompatibilityIssue = {
  kind:
    | "schema-required-removed"
    | "schema-property-removed"
    | "schema-property-type-changed"
    | "schema-closed-shape-violation"
    | "digest-incompatible"
    | "unresolved-ref";
  path?: string;
  reason: string;
};

const SUPPORTED_OBJECT_KEYS = new Set([
  "additionalProperties",
  "properties",
  "required",
  "type",
]);

function isJsonObject(value: JsonSchema): value is JsonObject {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function objectValue(value: JsonObject, key: string): JsonSchema | undefined {
  return value[key];
}

function stringArray(value: JsonSchema | undefined): string[] | null {
  if (value === undefined) return [];
  if (!Array.isArray(value)) return null;
  const strings: string[] = [];
  const seen = new Set<string>();
  for (const item of value) {
    if (typeof item !== "string") return null;
    if (seen.has(item)) return null;
    seen.add(item);
    strings.push(item);
  }
  return strings;
}

function propertiesObject(value: JsonSchema | undefined): JsonObject | null {
  if (value === undefined) return {};
  if (!isJsonObject(value)) return null;
  return value;
}

function isOpenObject(schema: JsonObject): boolean {
  return objectValue(schema, "additionalProperties") !== false;
}

function hasSupportedAdditionalProperties(schema: JsonObject): boolean {
  const additionalProperties = objectValue(schema, "additionalProperties");
  return additionalProperties === undefined ||
    typeof additionalProperties === "boolean";
}

function hasOnlySupportedObjectKeys(schema: JsonObject): boolean {
  return Object.keys(schema).every((key) => SUPPORTED_OBJECT_KEYS.has(key));
}

function sameStringSet(left: string[], right: string[]): boolean {
  if (left.length !== right.length) return false;
  const rightValues = new Set(right);
  return left.every((value) => rightValues.has(value));
}

function pointer(path: string, key: string): string {
  return `${path}/${key.replaceAll("~", "~0").replaceAll("/", "~1")}`;
}

function genericSchemaIssue(path: string): SchemaCompatibilityIssue {
  return {
    kind: "digest-incompatible",
    ...(path.length > 0 ? { path } : {}),
    reason: path.length > 0
      ? `Schema value at ${path} changed incompatibly.`
      : "Schema changed incompatibly.",
  };
}

function schemaCompatibilityIssues(
  left: JsonSchema,
  right: JsonSchema,
  path: string,
): SchemaCompatibilityIssue[] {
  if (canonicalizeJson(left) === canonicalizeJson(right)) return [];
  if (!isJsonObject(left) || !isJsonObject(right)) {
    return [genericSchemaIssue(path)];
  }
  const leftType = objectValue(left, "type");
  const rightType = objectValue(right, "type");
  if (leftType !== rightType) {
    return [{
      kind: "schema-property-type-changed",
      path: pointer(path, "type"),
      reason: `Schema type changed from ${String(leftType)} to ${
        String(rightType)
      }.`,
    }];
  }
  if (leftType !== "object") return [genericSchemaIssue(path)];
  if (!hasOnlySupportedObjectKeys(left) || !hasOnlySupportedObjectKeys(right)) {
    return [genericSchemaIssue(path)];
  }
  if (
    !hasSupportedAdditionalProperties(left) ||
    !hasSupportedAdditionalProperties(right)
  ) return [genericSchemaIssue(path)];

  const leftRequired = stringArray(objectValue(left, "required"));
  const rightRequired = stringArray(objectValue(right, "required"));
  if (!leftRequired || !rightRequired) return [genericSchemaIssue(path)];
  const rightRequiredSet = new Set(rightRequired);
  const issues: SchemaCompatibilityIssue[] = [];
  for (const field of leftRequired) {
    if (rightRequiredSet.has(field)) continue;
    issues.push({
      kind: "schema-required-removed",
      path: pointer(pointer(path, "properties"), field),
      reason: `Required field '${field}' was removed from the schema.`,
    });
  }
  if (!sameStringSet(leftRequired, rightRequired) && issues.length === 0) {
    issues.push({
      kind: "schema-closed-shape-violation",
      path: pointer(path, "required"),
      reason: "Schema required fields changed incompatibly.",
    });
  }
  const required = new Set(leftRequired);

  const leftProperties = propertiesObject(objectValue(left, "properties"));
  const rightProperties = propertiesObject(objectValue(right, "properties"));
  if (!leftProperties || !rightProperties) return [genericSchemaIssue(path)];

  if (
    (!isOpenObject(left) || !isOpenObject(right)) &&
    !sameStringSet(Object.keys(leftProperties), Object.keys(rightProperties))
  ) {
    issues.push({
      kind: "schema-closed-shape-violation",
      path: pointer(path, "properties"),
      reason: "Closed object property set changed incompatibly.",
    });
  }

  for (const [name, leftProperty] of Object.entries(leftProperties)) {
    const propertyPath = pointer(pointer(path, "properties"), name);
    const rightProperty = rightProperties[name];
    if (rightProperty === undefined) {
      if (required.has(name) || !isOpenObject(right)) {
        issues.push({
          kind: "schema-property-removed",
          path: propertyPath,
          reason: `Declared property '${name}' was removed from the schema.`,
        });
      }
      continue;
    }
    issues.push(
      ...schemaCompatibilityIssues(leftProperty, rightProperty, propertyPath),
    );
  }

  for (const [name] of Object.entries(rightProperties)) {
    if (leftProperties[name] !== undefined) continue;
    if (required.has(name) || !isOpenObject(left)) {
      issues.push({
        kind: "schema-closed-shape-violation",
        path: pointer(pointer(path, "properties"), name),
        reason: `Declared property '${name}' was added incompatibly.`,
      });
    }
  }

  return issues;
}

function mergeResolvedSchemaCompatible(
  left: JsonSchema,
  right: JsonSchema,
): JsonSchema | null {
  if (canonicalizeJson(left) === canonicalizeJson(right)) return left;
  if (!isJsonObject(left) || !isJsonObject(right)) return null;
  if (objectValue(left, "type") !== "object") return null;
  if (objectValue(right, "type") !== "object") return null;
  if (!hasOnlySupportedObjectKeys(left) || !hasOnlySupportedObjectKeys(right)) {
    return null;
  }
  if (
    !hasSupportedAdditionalProperties(left) ||
    !hasSupportedAdditionalProperties(right)
  ) return null;

  const leftRequired = stringArray(objectValue(left, "required"));
  const rightRequired = stringArray(objectValue(right, "required"));
  if (!leftRequired || !rightRequired) return null;
  if (!sameStringSet(leftRequired, rightRequired)) return null;
  const required = new Set(leftRequired);

  const leftProperties = propertiesObject(objectValue(left, "properties"));
  const rightProperties = propertiesObject(objectValue(right, "properties"));
  if (!leftProperties || !rightProperties) return null;

  if (
    (!isOpenObject(left) || !isOpenObject(right)) &&
    !sameStringSet(Object.keys(leftProperties), Object.keys(rightProperties))
  ) return null;

  const properties: JsonObject = {};
  for (const [name, leftProperty] of Object.entries(leftProperties)) {
    const rightProperty = rightProperties[name];
    if (rightProperty === undefined) {
      if (required.has(name)) return null;
      if (!isOpenObject(right)) return null;
      properties[name] = leftProperty;
      continue;
    }
    const property = mergeResolvedSchemaCompatible(leftProperty, rightProperty);
    if (property === null) return null;
    properties[name] = property;
  }

  for (const [name, rightProperty] of Object.entries(rightProperties)) {
    if (leftProperties[name] !== undefined) continue;
    if (required.has(name)) return null;
    if (!isOpenObject(left)) return null;
    properties[name] = rightProperty;
  }

  return {
    ...left,
    properties,
  };
}

function resolveSchemaRef(
  ref: ContractSchemaRef,
  schemas: ContractSchemas | undefined,
): JsonSchema | null {
  const schema = schemas?.[ref.schema];
  if (schema === undefined) return null;
  if (!isJsonValue(schema) && typeof schema !== "boolean") return null;
  return schema;
}

/**
 * Returns true when two contract schema refs resolve to identical schemas or a
 * conservative same-lineage compatible object evolution.
 */
export function areSchemaRefsCompatible(
  leftRef: ContractSchemaRef,
  leftSchemas: ContractSchemas | undefined,
  rightRef: ContractSchemaRef,
  rightSchemas: ContractSchemas | undefined,
): boolean {
  return mergeCompatibleSchemaRefs(
    leftRef,
    leftSchemas,
    rightRef,
    rightSchemas,
  ) !== null;
}

/**
 * Returns the conservative projected schema when two concrete schemas are
 * active-compatible, or null when Trellis cannot prove compatibility.
 */
export function mergeCompatibleSchemas(
  left: JsonSchema,
  right: JsonSchema,
): JsonSchema | null {
  return mergeResolvedSchemaCompatible(left, right);
}

/** Returns structured reasons why two concrete schemas are not active-compatible. */
export function describeSchemaCompatibility(
  left: JsonSchema,
  right: JsonSchema,
): SchemaCompatibilityIssue[] {
  return schemaCompatibilityIssues(left, right, "");
}

/**
 * Returns the conservative projected schema for compatible schema refs, or null
 * when Trellis cannot prove the resolved schemas are active-compatible.
 */
export function mergeCompatibleSchemaRefs(
  leftRef: ContractSchemaRef,
  leftSchemas: ContractSchemas | undefined,
  rightRef: ContractSchemaRef,
  rightSchemas: ContractSchemas | undefined,
): JsonSchema | null {
  const leftSchema = resolveSchemaRef(leftRef, leftSchemas);
  const rightSchema = resolveSchemaRef(rightRef, rightSchemas);
  if (leftSchema === null || rightSchema === null) return null;
  return mergeResolvedSchemaCompatible(leftSchema, rightSchema);
}

/** Returns structured reasons why two schema refs are not active-compatible. */
export function describeSchemaRefCompatibility(
  leftRef: ContractSchemaRef,
  leftSchemas: ContractSchemas | undefined,
  rightRef: ContractSchemaRef,
  rightSchemas: ContractSchemas | undefined,
): SchemaCompatibilityIssue[] {
  const leftSchema = resolveSchemaRef(leftRef, leftSchemas);
  const rightSchema = resolveSchemaRef(rightRef, rightSchemas);
  if (leftSchema === null || rightSchema === null) {
    return [{
      kind: "unresolved-ref",
      reason: `Schema reference '${
        leftSchema === null ? leftRef.schema : rightRef.schema
      }' could not be resolved.`,
    }];
  }
  return describeSchemaCompatibility(leftSchema, rightSchema);
}
