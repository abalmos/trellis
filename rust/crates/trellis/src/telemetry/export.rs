//! Trellis-owned OTLP HTTP/protobuf provider construction.
//!
//! Only compiled with the non-default `telemetry-otlp` feature. Provider
//! construction, flush, and shutdown never change a business result.

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use opentelemetry::trace::TracerProvider as _;
use opentelemetry::KeyValue;
use opentelemetry_otlp::{MetricExporter, SpanExporter, WithExportConfig as _};
use opentelemetry_sdk::metrics::{PeriodicReader, SdkMeterProvider};
use opentelemetry_sdk::trace::{
    BatchConfigBuilder, BatchSpanProcessor, Sampler, SdkTracerProvider,
};
use opentelemetry_sdk::Resource;

use super::{TelemetryIdentity, INSTRUMENTATION_SCOPE};

/// Default OpenTelemetry export timeout when the environment does not set one.
const DEFAULT_EXPORT_TIMEOUT: Duration = Duration::from_secs(3);
/// Metrics export interval from the observability order.
const METRICS_INTERVAL: Duration = Duration::from_secs(15);
/// Trace batch delay from the observability order.
const TRACE_BATCH_DELAY: Duration = Duration::from_millis(5_000);
/// Trace queue capacity from the observability order.
const TRACE_QUEUE_SIZE: usize = 2_048;
/// Trace batch maximum from the observability order.
const TRACE_BATCH_MAX: usize = 256;
/// Bounded shutdown budget for one process telemetry owner.
const SHUTDOWN_BUDGET: Duration = Duration::from_secs(5);
/// Default parent-based trace ratio when the environment does not set one.
const DEFAULT_TRACE_RATIO: f64 = 0.10;

/// Providers constructed and installed by Trellis.
pub struct OwnedProviders {
    /// Whether managed metrics were installed.
    pub(crate) metrics_enabled: bool,
    /// Whether a managed tracer or an exported trace signal was installed.
    pub(crate) traces_enabled: bool,
    tracer_provider: Option<SdkTracerProvider>,
    meter_provider: Option<SdkMeterProvider>,
}

impl OwnedProviders {
    /// Optional tracer for attaching the `tracing_opentelemetry` layer.
    pub(crate) fn tracer(&self) -> Option<opentelemetry_sdk::trace::Tracer> {
        self.tracer_provider
            .as_ref()
            .map(|provider| provider.tracer(INSTRUMENTATION_SCOPE))
    }
}

/// Resolves one non-empty environment variable.
fn env_value(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

/// Whether Trellis-owned initialization is disabled by the environment.
fn sdk_disabled() -> bool {
    env_value("OTEL_SDK_DISABLED").is_some_and(|value| value.eq_ignore_ascii_case("true"))
}

/// Whether the named exporter variable suppresses its signal.
fn exporter_disabled(get: impl Fn(&str) -> Option<String>, name: &str) -> bool {
    non_empty(get(name)).is_some_and(|value| value.eq_ignore_ascii_case("none"))
}

/// Whether the signal-specific or shared endpoint requests the signal.
fn endpoint_configured(get: impl Fn(&str) -> Option<String>, signal: &str) -> bool {
    non_empty(get(&format!("OTEL_EXPORTER_OTLP_{signal}_ENDPOINT"))).is_some()
        || non_empty(get("OTEL_EXPORTER_OTLP_ENDPOINT")).is_some()
}

/// Resolved signal selection for one process.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct SignalSelection {
    traces: bool,
    metrics: bool,
}

impl SignalSelection {
    /// Resolves which signals explicit, non-empty endpoint variables request.
    fn resolve(get: impl Fn(&str) -> Option<String>) -> Self {
        Self {
            traces: !exporter_disabled(&get, "OTEL_TRACES_EXPORTER")
                && endpoint_configured(&get, "TRACES"),
            metrics: !exporter_disabled(&get, "OTEL_METRICS_EXPORTER")
                && endpoint_configured(&get, "METRICS"),
        }
    }
}

/// Trims one environment value to a non-empty string.
fn non_empty(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

/// Applies the explicit 3-second default only when the environment is silent.
fn export_timeout() -> Option<Duration> {
    let configured = env_value("OTEL_EXPORTER_OTLP_TIMEOUT").is_some()
        || env_value("OTEL_EXPORTER_OTLP_TRACES_TIMEOUT").is_some()
        || env_value("OTEL_EXPORTER_OTLP_METRICS_TIMEOUT").is_some();
    if configured {
        None
    } else {
        Some(DEFAULT_EXPORT_TIMEOUT)
    }
}

/// Resolves the trace sampler from the environment with a 0.10 default.
fn sampler_from_env() -> Sampler {
    let ratio = || {
        env_value("OTEL_TRACES_SAMPLER_ARG")
            .and_then(|value| value.parse::<f64>().ok())
            .filter(|value| (0.0..=1.0).contains(value))
            .unwrap_or(DEFAULT_TRACE_RATIO)
    };
    match env_value("OTEL_TRACES_SAMPLER").as_deref() {
        None => parent_based(DEFAULT_TRACE_RATIO),
        Some("always_on") => Sampler::AlwaysOn,
        Some("always_off") => Sampler::AlwaysOff,
        Some("traceidratio") => Sampler::TraceIdRatioBased(ratio()),
        Some("parentbased_traceidratio") => parent_based(ratio()),
        Some(other) => {
            warn_once(
                "sampler",
                &format!(
                    "unknown OTEL_TRACES_SAMPLER '{other}'; using parentbased_traceidratio {DEFAULT_TRACE_RATIO}"
                ),
            );
            parent_based(DEFAULT_TRACE_RATIO)
        }
    }
}

/// Parent-based wrapper over a trace-id ratio sampler.
fn parent_based(ratio: f64) -> Sampler {
    Sampler::ParentBased(Box::new(Sampler::TraceIdRatioBased(ratio)))
}

/// Emits one bounded warning for a repeated environment problem.
fn warn_once(category: &str, message: &str) {
    use std::sync::OnceLock;
    static SAMPLER: OnceLock<()> = OnceLock::new();
    static OTHER: OnceLock<()> = OnceLock::new();
    let cell = match category {
        "sampler" => &SAMPLER,
        _ => &OTHER,
    };
    if cell.set(()).is_ok() {
        eprintln!("trellis telemetry: {message}");
    }
}

/// Builds the process resource including identity defaults and env overrides.
fn resource(identity: &TelemetryIdentity) -> Resource {
    let mut attributes = vec![
        KeyValue::new("service.name", identity.service_name.clone()),
        KeyValue::new("service.version", identity.service_version.clone()),
        KeyValue::new("service.namespace", "trellis"),
        KeyValue::new(
            "deployment.environment.name",
            env_value("TRELLIS_ENVIRONMENT").unwrap_or_else(|| "development".to_owned()),
        ),
        KeyValue::new(
            "trellis.cluster",
            env_value("TRELLIS_CLUSTER").unwrap_or_else(|| "local".to_owned()),
        ),
        KeyValue::new("trellis.role", identity.role.as_str()),
    ];
    if let Some(name) = env_value("OTEL_SERVICE_NAME") {
        set_resource_attribute(&mut attributes, "service.name", name);
    }
    if let Some(extra) = env_value("OTEL_RESOURCE_ATTRIBUTES") {
        for member in extra.split(',') {
            if let Some((key, value)) = member.split_once('=') {
                let key = key.trim();
                let value = value.trim();
                if !key.is_empty() && !value.is_empty() {
                    set_resource_attribute(&mut attributes, key, value.to_owned());
                }
            }
        }
    }
    // Process identity is generated per process and is never a credential.
    set_resource_attribute(
        &mut attributes,
        "service.instance.id",
        ulid::Ulid::new().to_string(),
    );
    Resource::builder().with_attributes(attributes).build()
}

/// Replaces one resource attribute in place.
fn set_resource_attribute(attributes: &mut Vec<KeyValue>, key: &str, value: String) {
    if let Some(existing) = attributes
        .iter_mut()
        .find(|attribute| attribute.key.as_str() == key)
    {
        existing.value = value.into();
    } else {
        attributes.push(KeyValue::new(key.to_owned(), value));
    }
}

/// Resolves one bounded positive environment override in milliseconds.
fn positive_env_ms(name: &str) -> Option<u64> {
    env_value(name)
        .and_then(|value| value.parse::<u64>().ok())
        .filter(|value| *value > 0)
}

/// Applies the order's trace batch defaults unless the environment overrides.
fn batch_config() -> opentelemetry_sdk::trace::BatchConfig {
    let mut builder = BatchConfigBuilder::default();
    if positive_env_ms("OTEL_BSP_MAX_QUEUE_SIZE").is_none() {
        builder = builder.with_max_queue_size(TRACE_QUEUE_SIZE);
    }
    if positive_env_ms("OTEL_BSP_MAX_EXPORT_BATCH_SIZE").is_none() {
        builder = builder.with_max_export_batch_size(TRACE_BATCH_MAX);
    }
    if positive_env_ms("OTEL_BSP_SCHEDULE_DELAY").is_none() {
        builder = builder.with_scheduled_delay(TRACE_BATCH_DELAY);
    }
    builder.build()
}

/// Resolves the metrics export interval, honoring the standard override.
fn metrics_interval() -> Duration {
    positive_env_ms("OTEL_METRIC_EXPORT_INTERVAL")
        .map(Duration::from_millis)
        .unwrap_or(METRICS_INTERVAL)
}

/// Builds and installs Trellis-owned providers once per process.
pub(crate) fn build_owned(identity: &TelemetryIdentity) -> OwnedProviders {
    if sdk_disabled() {
        return OwnedProviders {
            metrics_enabled: false,
            traces_enabled: false,
            tracer_provider: None,
            meter_provider: None,
        };
    }
    let resource = resource(identity);
    let mut owned = OwnedProviders {
        metrics_enabled: false,
        traces_enabled: false,
        tracer_provider: None,
        meter_provider: None,
    };

    let signals = SignalSelection::resolve(|name| std::env::var(name).ok());

    let traces_requested = signals.traces;
    if traces_requested {
        let mut builder = SpanExporter::builder().with_http();
        if let Some(timeout) = export_timeout() {
            builder = builder.with_timeout(timeout);
        }
        match builder.build() {
            Ok(exporter) => {
                let batch = batch_config();
                let provider = SdkTracerProvider::builder()
                    .with_resource(resource.clone())
                    .with_sampler(sampler_from_env())
                    .with_span_processor(
                        BatchSpanProcessor::builder(exporter)
                            .with_batch_config(batch)
                            .build(),
                    )
                    .build();
                opentelemetry::global::set_tracer_provider(provider.clone());
                owned.tracer_provider = Some(provider);
                owned.traces_enabled = true;
            }
            Err(error) => {
                eprintln!("trellis telemetry: trace export disabled: {error}");
            }
        }
    }

    let metrics_requested = signals.metrics;
    if metrics_requested {
        let mut builder = MetricExporter::builder().with_http();
        if let Some(timeout) = export_timeout() {
            builder = builder.with_timeout(timeout);
        }
        match builder.build() {
            Ok(exporter) => {
                let reader = PeriodicReader::builder(exporter)
                    .with_interval(metrics_interval())
                    .build();
                let provider = SdkMeterProvider::builder()
                    .with_resource(resource)
                    .with_reader(reader)
                    .build();
                opentelemetry::global::set_meter_provider(provider.clone());
                owned.meter_provider = Some(provider);
                owned.metrics_enabled = true;
            }
            Err(error) => {
                eprintln!("trellis telemetry: metric export disabled: {error}");
            }
        }
    }

    owned
}

/// Runs one bounded blocking operation outside the caller's async context.
fn run_bounded(work: impl FnOnce() + Send + 'static, what: &str) {
    static TIMED_OUT: AtomicBool = AtomicBool::new(false);
    let (sender, receiver) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        work();
        let _ = sender.send(());
    });
    if receiver.recv_timeout(SHUTDOWN_BUDGET).is_err() && !TIMED_OUT.swap(true, Ordering::SeqCst) {
        eprintln!("trellis telemetry: {what} exceeded the telemetry shutdown budget");
    }
}

/// Flushes owned providers within the bounded process budget.
pub(crate) fn flush_bounded(providers: &OwnedProviders) {
    let tracer = providers.tracer_provider.clone();
    let meter = providers.meter_provider.clone();
    if tracer.is_none() && meter.is_none() {
        return;
    }
    run_bounded(
        move || {
            if let Some(provider) = tracer {
                let _ = provider.force_flush();
            }
            if let Some(provider) = meter {
                let _ = provider.force_flush();
            }
        },
        "flush",
    );
}

/// Flushes and shuts down owned providers within the bounded process budget.
pub(crate) fn shutdown_bounded(providers: &OwnedProviders) {
    let tracer = providers.tracer_provider.clone();
    let meter = providers.meter_provider.clone();
    if tracer.is_none() && meter.is_none() {
        return;
    }
    run_bounded(
        move || {
            if let Some(provider) = tracer {
                let _ = provider.shutdown();
            }
            if let Some(provider) = meter {
                let _ = provider.shutdown();
            }
        },
        "shutdown",
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn environment_overrides_replace_defaults() {
        let mut attributes = vec![KeyValue::new("service.name", "trellis-server")];
        set_resource_attribute(&mut attributes, "service.name", "orders-api".to_owned());
        assert_eq!(attributes.len(), 1);
        assert_eq!(attributes[0].value, "orders-api".into());
        set_resource_attribute(&mut attributes, "trellis.role", "server".to_owned());
        assert_eq!(attributes.len(), 2);
    }

    #[test]
    fn sampler_defaults_are_parent_based() {
        assert!(matches!(
            parent_based(DEFAULT_TRACE_RATIO),
            Sampler::ParentBased(_)
        ));
    }

    #[test]
    fn explicit_endpoint_variables_enable_each_signal_independently() {
        let resolve = |variables: &[(&str, &str)]| {
            let values: std::collections::HashMap<&str, &str> = variables.iter().copied().collect();
            SignalSelection::resolve(|name| values.get(name).map(|value| (*value).to_owned()))
        };
        assert_eq!(
            resolve(&[]),
            SignalSelection {
                traces: false,
                metrics: false,
            }
        );
        assert_eq!(
            resolve(&[("OTEL_EXPORTER_OTLP_ENDPOINT", "http://collector:4318")]),
            SignalSelection {
                traces: true,
                metrics: true,
            }
        );
        assert_eq!(
            resolve(&[(
                "OTEL_EXPORTER_OTLP_TRACES_ENDPOINT",
                "http://collector:4318"
            )]),
            SignalSelection {
                traces: true,
                metrics: false,
            }
        );
        assert_eq!(
            resolve(&[(
                "OTEL_EXPORTER_OTLP_METRICS_ENDPOINT",
                "http://collector:4318"
            )]),
            SignalSelection {
                traces: false,
                metrics: true,
            }
        );
        assert_eq!(
            resolve(&[("OTEL_EXPORTER_OTLP_ENDPOINT", "   ")]),
            SignalSelection {
                traces: false,
                metrics: false,
            }
        );
        assert_eq!(
            resolve(&[
                ("OTEL_EXPORTER_OTLP_ENDPOINT", "http://collector:4318"),
                ("OTEL_TRACES_EXPORTER", "none"),
            ]),
            SignalSelection {
                traces: false,
                metrics: true,
            }
        );
    }
}
