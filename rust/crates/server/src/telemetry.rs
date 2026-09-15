//! Optional OpenTelemetry trace and metric export for trellis-server.
//!
//! Provider construction and export configuration live in the Rust SDK's
//! `telemetry` module. This module owns only the process fmt subscriber
//! assembly, so the CLI and native hosts share one exporter implementation.
//! The existing fmt-based stdout/stderr tracing output is always installed.

use std::io::IsTerminal as _;

use tracing_subscriber::layer::SubscriberExt as _;
use tracing_subscriber::util::SubscriberInitExt as _;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::Layer as _;
use trellis_rs::telemetry::{self, TelemetryGuard, TelemetryIdentity, TelemetryRole};

/// Install the fmt tracing subscriber plus optional OTLP trace/metric export.
///
/// The returned guard owns flush and shutdown; it must be shut down from the
/// process stop path after the async runtime has stopped.
///
/// # Errors
///
/// Returns an error only when the fmt tracing subscriber cannot be installed.
pub(crate) fn init(verbose: bool, check: bool) -> miette::Result<TelemetryGuard> {
    let guard = if check {
        TelemetryGuard::disabled()
    } else {
        telemetry::init_from_env(TelemetryIdentity::new(
            "trellis-server",
            TelemetryRole::Server,
            env!("CARGO_PKG_VERSION"),
        ))
    };

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(if verbose { "debug" } else { "info" }));
    let attached = if check {
        std::io::stderr().is_terminal()
    } else {
        std::io::stdout().is_terminal()
    };
    let writer = if check {
        tracing_subscriber::fmt::writer::BoxMakeWriter::new(std::io::stderr)
    } else {
        tracing_subscriber::fmt::writer::BoxMakeWriter::new(std::io::stdout)
    };
    let otel_layer: Option<
        Box<dyn tracing_subscriber::Layer<tracing_subscriber::Registry> + Send + Sync>,
    > = guard.tracer().map(|tracer| {
        Box::new(tracing_opentelemetry::layer().with_tracer(tracer))
            as Box<dyn tracing_subscriber::Layer<tracing_subscriber::Registry> + Send + Sync>
    });

    // The console filter bounds fmt output only; instrumented Trellis spans
    // reach the OTel layer independently of RUST_LOG console verbosity.
    if attached {
        tracing_subscriber::registry()
            .with(otel_layer)
            .with(
                tracing_subscriber::fmt::layer()
                    .with_target(verbose)
                    .with_ansi(true)
                    .with_writer(writer)
                    .with_filter(filter),
            )
            .try_init()
            .map_err(|error| miette::miette!(error.to_string()))?;
    } else {
        tracing_subscriber::registry()
            .with(otel_layer)
            .with(
                tracing_subscriber::fmt::layer()
                    .with_ansi(false)
                    .json()
                    .with_writer(writer)
                    .with_filter(filter),
            )
            .try_init()
            .map_err(|error| miette::miette!(error.to_string()))?;
    }
    Ok(guard)
}
