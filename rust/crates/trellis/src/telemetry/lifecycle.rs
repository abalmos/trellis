//! Exactly-once observation guards for duration and inflight metrics.
//!
//! A guard records one result at its named boundary. Dropping a guard without
//! an explicit finish records the configured cancelled/interrupted outcome, so
//! `?`, panic, or dropped futures cannot silently skip the observation. Guards
//! never change control flow and cannot turn a business failure into a success.

use std::time::{Duration, Instant};

use opentelemetry::KeyValue;

use super::instruments::{add_updown, record_family_duration, DurationFamily, UpDownFamily};

/// Duration observation for one synchronous or asynchronous unit of work.
pub struct Observation {
    started: Instant,
    family: DurationFamily,
    attributes: Vec<KeyValue>,
    drop_outcome: &'static str,
    inflight: Option<InflightGuard>,
    finished: bool,
}

impl Observation {
    /// Starts one un-counted duration observation.
    pub fn start(
        family: DurationFamily,
        attributes: Vec<KeyValue>,
        drop_outcome: &'static str,
    ) -> Self {
        Self {
            started: Instant::now(),
            family,
            attributes,
            drop_outcome,
            inflight: None,
            finished: false,
        }
    }

    /// Starts one duration observation plus an inflight up/down increment.
    pub fn start_counted(
        family: DurationFamily,
        inflight: UpDownFamily,
        attributes: Vec<KeyValue>,
        drop_outcome: &'static str,
    ) -> Self {
        let guard = InflightGuard::acquire(inflight, attributes.clone());
        Self {
            started: Instant::now(),
            family,
            attributes,
            drop_outcome,
            inflight: Some(guard),
            finished: false,
        }
    }

    /// Monotonic elapsed time since the observation started.
    pub fn elapsed(&self) -> Duration {
        self.started.elapsed()
    }

    /// Finishes the observation with one bounded outcome value.
    pub fn finish(self, outcome: &'static str) {
        self.finish_with(outcome, &[]);
    }

    /// Finishes the observation with an outcome and extra attributes.
    pub fn finish_with(mut self, outcome: &'static str, extra: &[KeyValue]) {
        if self.finished {
            return;
        }
        self.finished = true;
        let mut attributes = std::mem::take(&mut self.attributes);
        attributes.push(KeyValue::new("trellis.outcome", outcome));
        attributes.extend_from_slice(extra);
        record_family_duration(self.family, self.started.elapsed(), &attributes);
        if let Some(inflight) = self.inflight.take() {
            drop(inflight);
        }
    }
}

impl Drop for Observation {
    fn drop(&mut self) {
        if self.finished {
            return;
        }
        self.finished = true;
        let mut attributes = std::mem::take(&mut self.attributes);
        attributes.push(KeyValue::new("trellis.outcome", self.drop_outcome));
        record_family_duration(self.family, self.started.elapsed(), &attributes);
        if let Some(inflight) = self.inflight.take() {
            drop(inflight);
        }
    }
}

/// Inflight up/down counter increment released exactly once.
pub struct InflightGuard {
    family: UpDownFamily,
    attributes: Vec<KeyValue>,
    released: bool,
}

impl InflightGuard {
    /// Increments the family and returns the guard that decrements it.
    pub fn acquire(family: UpDownFamily, attributes: Vec<KeyValue>) -> Self {
        add_updown(family, 1, &attributes);
        Self {
            family,
            attributes,
            released: false,
        }
    }

    /// Releases the increment early; dropping has the same effect once.
    pub fn release(mut self) {
        if !self.released {
            self.released = true;
            add_updown(self.family, -1, &self.attributes);
        }
    }
}

impl Drop for InflightGuard {
    fn drop(&mut self) {
        if !self.released {
            self.released = true;
            add_updown(self.family, -1, &self.attributes);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dropped_observations_finish_once() {
        let observation = Observation::start(
            DurationFamily::RpcClient,
            vec![KeyValue::new("trellis.route", "_unknown")],
            "cancelled",
        );
        drop(observation);
    }

    #[test]
    fn explicit_finish_is_idempotent_under_drop() {
        let observation = Observation::start(
            DurationFamily::Cli,
            vec![KeyValue::new("trellis.command", "whoami")],
            "cancelled",
        );
        observation.finish("ok");
    }

    #[test]
    fn inflight_guard_releases_early_once() {
        let guard = InflightGuard::acquire(
            UpDownFamily::RpcServerInflight,
            vec![KeyValue::new("trellis.route", "_unknown")],
        );
        guard.release();
    }
}
