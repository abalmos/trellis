//! Browser-safe Trellis telemetry entrypoint.
//!
//! This module imports only OpenTelemetry API, resources, and browser
//! WebTracerProvider/metrics modules. It never imports Node builtins, async
//! local storage, filesystem, or collector credentials. Built-in and
//! third-party browser apps opt in explicitly and supply a same-origin
//! relative OTLP path; the deployment's restricted relay forwards it to the
//! isolated browser Collector listener.

import { metrics, trace } from "@opentelemetry/api";

/** Built-in application identity accepted by the browser initializer. */
export type BrowserTelemetryApp = "console" | "portal" | string;

/** Explicit opt-in configuration for one browser document owner. */
export interface BrowserTelemetryOptions {
  /** Built-in or third-party application label; never a user or session ID. */
  app: BrowserTelemetryApp;
  /** Same-origin relative OTLP prefix such as `/otel`. */
  endpoint: string;
  /** Optional service name; defaults to `trellis-browser`. */
  serviceName?: string;
  /** Optional service version. */
  serviceVersion?: string;
  /** Trace ratio in `0..1`; defaults to 0.05. */
  traceRatio?: number;
}

/** Process handle for the browser document owner. */
export interface BrowserTelemetryHandle {
  /** Whether browser telemetry was installed. */
  readonly enabled: boolean;
  /** Best-effort flush, used at most once per visibility transition. */
  forceFlush(): Promise<void>;
  /** Flushes and shuts down the document providers. */
  shutdown(): Promise<void>;
}

/** Default browser trace ratio from the observability order. */
const DEFAULT_BROWSER_TRACE_RATIO = 0.05;
/** Browser metrics export interval in milliseconds. */
const BROWSER_METRICS_INTERVAL_MILLIS = 15_000;

let owner: Promise<BrowserTelemetryHandle> | undefined;

/** Reads and sanitizes the same-origin relative OTLP prefix. */
export function sanitizeBrowserEndpoint(
  endpoint: string | undefined,
): string | undefined {
  if (!endpoint) return undefined;
  const trimmed = endpoint.trim();
  if (!trimmed.startsWith("/")) return undefined;
  if (
    trimmed.includes("://") || trimmed.includes("?") || trimmed.includes("#") ||
    trimmed.includes("@")
  ) {
    return undefined;
  }
  return trimmed.replace(/\/+$/, "");
}

/** Validates the opt-in trace ratio. */
function traceRatio(value: number | undefined): number {
  if (
    value === undefined || !Number.isFinite(value) || value < 0 || value > 1
  ) {
    return DEFAULT_BROWSER_TRACE_RATIO;
  }
  return value;
}

/** Dynamic import that keeps browser bundles free of native modules. */
function browserImport<TModule>(specifier: string): Promise<TModule> {
  return import(specifier) as Promise<TModule>;
}

/** Installs one document-owned browser telemetry provider set. */
async function initialize(
  options: BrowserTelemetryOptions,
): Promise<BrowserTelemetryHandle> {
  const base = sanitizeBrowserEndpoint(options.endpoint);
  if (!base) {
    console.warn(
      "trellis telemetry: browser telemetry disabled; endpoint must be a same-origin relative path",
    );
    return {
      enabled: false,
      forceFlush: () => Promise.resolve(),
      shutdown: () => Promise.resolve(),
    };
  }
  try {
    const [web, traceBase, resources, sdkMetrics, traceOtlp, metricsOtlp] =
      await Promise.all([
        browserImport<typeof import("@opentelemetry/sdk-trace-web")>(
          ["@opentelemetry", "sdk-trace-web"].join("/"),
        ),
        browserImport<typeof import("@opentelemetry/sdk-trace-base")>(
          ["@opentelemetry", "sdk-trace-base"].join("/"),
        ),
        browserImport<typeof import("@opentelemetry/resources")>(
          ["@opentelemetry", "resources"].join("/"),
        ),
        browserImport<typeof import("@opentelemetry/sdk-metrics")>(
          ["@opentelemetry", "sdk-metrics"].join("/"),
        ),
        browserImport<
          typeof import("@opentelemetry/exporter-trace-otlp-proto")
        >(
          ["@opentelemetry", "exporter-trace-otlp-proto"].join("/"),
        ),
        browserImport<
          typeof import("@opentelemetry/exporter-metrics-otlp-proto")
        >(["@opentelemetry", "exporter-metrics-otlp-proto"].join("/")),
      ]);
    const attributes = {
      "service.name": options.serviceName ?? "trellis-browser",
      "service.version": options.serviceVersion ?? "0.0.0",
      "service.namespace": "trellis",
      "trellis.role": "browser",
      "trellis.app": options.app,
      // Document-lifetime cumulative-stream identity; never a user identity.
      "service.instance.id": crypto.randomUUID(),
    };
    const tracerProvider = new web.WebTracerProvider({
      resource: resources.resourceFromAttributes(attributes),
      sampler: new traceBase.ParentBasedSampler({
        root: new traceBase.TraceIdRatioBasedSampler(
          traceRatio(options.traceRatio),
        ),
      }),
      spanProcessors: [
        new traceBase.BatchSpanProcessor(
          new traceOtlp.OTLPTraceExporter({ url: `${base}/v1/traces` }),
        ),
      ],
    });
    const meterProvider = new sdkMetrics.MeterProvider({
      resource: resources.resourceFromAttributes(attributes),
      readers: [
        new sdkMetrics.PeriodicExportingMetricReader({
          exporter: new metricsOtlp.OTLPMetricExporter({
            url: `${base}/v1/metrics`,
          }),
          exportIntervalMillis: BROWSER_METRICS_INTERVAL_MILLIS,
        }),
      ],
    });
    trace.setGlobalTracerProvider(tracerProvider);
    metrics.setGlobalMeterProvider(meterProvider);
    return {
      enabled: true,
      forceFlush: async () => {
        await tracerProvider.forceFlush();
        await meterProvider.forceFlush();
      },
      shutdown: async () => {
        try {
          await tracerProvider.shutdown();
          await meterProvider.shutdown();
        } catch (error) {
          console.warn("trellis telemetry: browser shutdown failed", error);
        }
      },
    };
  } catch (error) {
    console.warn("trellis telemetry: browser initialization failed", error);
    return {
      enabled: false,
      forceFlush: () => Promise.resolve(),
      shutdown: () => Promise.resolve(),
    };
  }
}

/**
 * Initializes browser telemetry once per document.
 *
 * Failure never breaks rendering: the promise always resolves to a handle.
 */
export function initBrowserTelemetry(
  options: BrowserTelemetryOptions,
): Promise<BrowserTelemetryHandle> {
  if (owner === undefined) owner = initialize(options);
  return owner;
}
