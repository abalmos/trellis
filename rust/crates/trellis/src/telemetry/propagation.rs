//! W3C trace-context carrier helpers for Trellis transports.
//!
//! Only `traceparent` and `tracestate` are propagated; baggage is never
//! forwarded. Invalid, duplicate, oversized, or absent telemetry context is
//! ignored and produces a new local root. Trace headers are untrusted
//! diagnostic metadata and never participate in authorization.

use opentelemetry::propagation::{Extractor, Injector, TextMapPropagator as _};
use opentelemetry::trace::TraceContextExt as _;
use opentelemetry::{Context, KeyValue};
use opentelemetry_sdk::propagation::TraceContextPropagator;

/// Maximum accepted `traceparent` length in bytes.
pub const TRACEPARENT_MAX_BYTES: usize = 256;
/// Maximum accepted `tracestate` length in bytes.
pub const TRACESTATE_MAX_BYTES: usize = 512;
/// Maximum accepted `tracestate` members.
pub const TRACESTATE_MAX_MEMBERS: usize = 32;

/// One validated trace carrier retained with durable work.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TraceCarrier {
    /// Validated W3C `traceparent` value.
    pub traceparent: String,
    /// Validated optional W3C `tracestate` value.
    pub tracestate: Option<String>,
}

impl TraceCarrier {
    /// Captures the current OpenTelemetry context when it has a valid span.
    pub fn capture() -> Option<Self> {
        let context = Context::current();
        let span = context.span();
        let span_context = span.span_context();
        if !span_context.is_valid() {
            return None;
        }
        let mut headers = Vec::new();
        inject_context(&context, &mut headers);
        Self::from_pairs(&headers)
    }

    /// Builds a carrier from already-validated wire values.
    pub fn new(traceparent: impl Into<String>, tracestate: Option<String>) -> Option<Self> {
        let traceparent = traceparent.into();
        if !validate_traceparent(&traceparent) {
            return None;
        }
        let tracestate = tracestate.filter(|value| validate_tracestate(value));
        Some(Self {
            traceparent,
            tracestate,
        })
    }

    /// Extracts the carrier from one key/value header set.
    pub fn from_pairs(pairs: &[(String, String)]) -> Option<Self> {
        let mut traceparents = pairs
            .iter()
            .filter(|(key, _)| key.eq_ignore_ascii_case("traceparent"))
            .map(|(_, value)| value.as_str());
        let traceparent = traceparents.next()?;
        // Duplicate trace headers are ignored rather than merged.
        if traceparents.next().is_some() {
            return None;
        }
        let tracestate = pairs
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case("tracestate"))
            .map(|(_, value)| value.clone());
        Self::new(traceparent.to_owned(), tracestate)
    }

    /// Returns the wire pairs to attach to a transport message.
    pub fn to_pairs(&self) -> Vec<(String, String)> {
        let mut pairs = vec![("traceparent".to_owned(), self.traceparent.clone())];
        if let Some(tracestate) = &self.tracestate {
            pairs.push(("tracestate".to_owned(), tracestate.clone()));
        }
        pairs
    }

    /// Rebuilds the OpenTelemetry context for this carrier.
    pub fn context(&self) -> Context {
        let pairs = self.to_pairs();
        extract_context(&pairs)
    }
}

/// Injects only W3C trace fields from a context into header pairs.
pub fn inject_context(context: &Context, into: &mut Vec<(String, String)>) {
    let mut injector = PairInjector { pairs: into };
    TraceContextPropagator::new().inject_context(context, &mut injector);
}

/// Extracts a trace context starting from an empty root context.
///
/// Invalid carriers never reject a request: the returned context is a new
/// local root without a remote parent.
pub fn extract_context(pairs: &[(String, String)]) -> Context {
    let sanitized = sanitize_pairs(pairs);
    let extractor = PairExtractor { pairs: &sanitized };
    TraceContextPropagator::new().extract(&extractor)
}

/// Keeps at most one validated value per whitelisted trace field.
fn sanitize_pairs(pairs: &[(String, String)]) -> Vec<(String, String)> {
    let mut sanitized: Vec<(String, String)> = Vec::new();
    for (key, value) in pairs {
        if key.eq_ignore_ascii_case("traceparent") {
            if sanitized
                .iter()
                .any(|(existing, _)| existing.eq_ignore_ascii_case("traceparent"))
            {
                continue;
            }
            if validate_traceparent(value) {
                sanitized.push(("traceparent".to_owned(), value.clone()));
            }
        } else if key.eq_ignore_ascii_case("tracestate") {
            if sanitized
                .iter()
                .any(|(existing, _)| existing.eq_ignore_ascii_case("tracestate"))
            {
                continue;
            }
            if validate_tracestate(value) {
                sanitized.push(("tracestate".to_owned(), value.clone()));
            }
        }
    }
    sanitized
}

/// Validates one `traceparent` value without parsing it manually.
pub fn validate_traceparent(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= TRACEPARENT_MAX_BYTES
        && value.is_ascii()
        && !value.contains(['\r', '\n'])
}

/// Validates one `tracestate` value: bounded size, members, and ASCII.
pub fn validate_tracestate(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= TRACESTATE_MAX_BYTES
        && value.is_ascii()
        && !value.contains(['\r', '\n'])
        && value.split(',').count() <= TRACESTATE_MAX_MEMBERS
}

/// Returns the trace-only attribute pairs for one validated carrier.
///
/// The returned value is used for correlation attributes only and never for a
/// metric label.
pub fn trace_attributes(carrier: &TraceCarrier) -> Vec<KeyValue> {
    let context = carrier.context();
    let span_context = context.span().span_context().clone();
    if !span_context.is_valid() {
        return Vec::new();
    }
    vec![
        KeyValue::new("trace.id", span_context.trace_id().to_string()),
        KeyValue::new("span.id", span_context.span_id().to_string()),
    ]
}

struct PairInjector<'a> {
    pairs: &'a mut Vec<(String, String)>,
}

impl Injector for PairInjector<'_> {
    fn set(&mut self, key: &str, value: String) {
        if key.eq_ignore_ascii_case("traceparent") {
            if validate_traceparent(&value) {
                self.pairs.push(("traceparent".to_owned(), value));
            }
        } else if key.eq_ignore_ascii_case("tracestate") && validate_tracestate(&value) {
            self.pairs.push(("tracestate".to_owned(), value));
        }
    }
}

struct PairExtractor<'a> {
    pairs: &'a [(String, String)],
}

impl Extractor for PairExtractor<'_> {
    fn get(&self, key: &str) -> Option<&str> {
        self.pairs
            .iter()
            .find(|(name, _)| name.eq_ignore_ascii_case(key))
            .map(|(_, value)| value.as_str())
    }

    fn keys(&self) -> Vec<&str> {
        self.pairs.iter().map(|(key, _)| key.as_str()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use opentelemetry::trace::Tracer as _;
    use opentelemetry::trace::TracerProvider as _;
    use opentelemetry_sdk::trace::SdkTracerProvider;

    #[test]
    fn invalid_carriers_are_ignored_and_produce_a_local_root() {
        let context = extract_context(&[
            ("traceparent".to_owned(), "not-a-traceparent".to_owned()),
            ("traceparent".to_owned(), "also-bad".to_owned()),
            (
                "tracestate".to_owned(),
                "x".repeat(TRACESTATE_MAX_BYTES + 1),
            ),
            ("baggage".to_owned(), "user=1".to_owned()),
        ]);
        assert!(!context.span().span_context().is_valid());
    }

    #[test]
    fn valid_carriers_round_trip_and_never_forward_baggage() {
        let provider = SdkTracerProvider::builder().build();
        let tracer = provider.tracer("propagation-test");
        let span = tracer.start("parent");
        let context = Context::current_with_span(span);
        let mut pairs = Vec::new();
        inject_context(&context, &mut pairs);
        assert_eq!(pairs.len(), 1);
        assert_eq!(pairs[0].0, "traceparent");
        let extracted = extract_context(&pairs);
        assert!(extracted.span().span_context().is_valid());
        assert_eq!(
            extracted.span().span_context().trace_id(),
            context.span().span_context().trace_id()
        );
        let carrier = TraceCarrier::from_pairs(&pairs).expect("carrier");
        assert_eq!(carrier.to_pairs().len(), 1);
    }

    #[test]
    fn duplicate_traceparents_are_rejected() {
        let provider = SdkTracerProvider::builder().build();
        let tracer = provider.tracer("propagation-test");
        let mut pairs = Vec::new();
        inject_context(
            &Context::current_with_span(tracer.start("parent")),
            &mut pairs,
        );
        pairs.push(pairs[0].clone());
        assert!(TraceCarrier::from_pairs(&pairs).is_none());
    }
}
