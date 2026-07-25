use std::time::Duration;

use async_nats::jetstream::{self, stream};

use crate::health::{
    DEFAULT_TRANSPORT_MAX_BYTES, DEFAULT_TRANSPORT_RETENTION_HOURS, HEALTH_STREAM, HEALTH_SUBJECT,
};
use crate::supervisor::RuntimeError;
use crate::{RuntimeConfig, RuntimeMode, SubsystemName};

pub(crate) const EVENT_STREAM: &str = "trellis";
pub(crate) const JOBS_STREAM: &str = "JOBS";
pub(crate) const JOBS_WORK_STREAM: &str = "JOBS_WORK";
pub(crate) const JOBS_ADVISORIES_STREAM: &str = "JOBS_ADVISORIES";

#[derive(Clone, Debug)]
pub(crate) struct ExpectedRuntimeResources {
    subsystems: &'static [SubsystemName],
    streams: Vec<stream::Config>,
}

impl ExpectedRuntimeResources {
    pub(crate) fn for_mode(mode: RuntimeMode, config: &RuntimeConfig) -> Self {
        let subsystems = mode.subsystems();
        let mut streams = Vec::new();
        if subsystems.contains(&SubsystemName::Platform)
            || subsystems.contains(&SubsystemName::Eventlog)
        {
            streams.push(event_stream_config());
        }
        if subsystems.contains(&SubsystemName::Jobs) {
            streams.extend(jobs_stream_configs());
        }
        if subsystems.contains(&SubsystemName::Health) {
            streams.push(health_stream_config(config));
        }
        Self {
            subsystems,
            streams,
        }
    }

    pub(crate) fn requires(&self, subsystem: SubsystemName) -> bool {
        self.subsystems.contains(&subsystem)
    }

    pub(crate) fn streams(&self) -> &[stream::Config] {
        &self.streams
    }

    pub(crate) async fn converge_streams(
        &self,
        client: async_nats::Client,
    ) -> Result<(), RuntimeError> {
        let jetstream = jetstream::new(client);
        for expected in &self.streams {
            match jetstream.get_stream(&expected.name).await {
                Ok(mut stream) => {
                    let info = stream
                        .info()
                        .await
                        .map_err(|error| RuntimeError::Nats(error.to_string()))?;
                    if !stream_is_compatible(&info.config, expected) {
                        jetstream
                            .update_stream(expected.clone())
                            .await
                            .map_err(|error| RuntimeError::Nats(error.to_string()))?;
                    }
                }
                Err(_) => {
                    jetstream
                        .create_stream(expected.clone())
                        .await
                        .map_err(|error| RuntimeError::Nats(error.to_string()))?;
                }
            }
        }
        Ok(())
    }
}

pub(crate) fn stream_is_compatible(actual: &stream::Config, expected: &stream::Config) -> bool {
    if actual.subjects != expected.subjects
        || actual.retention != expected.retention
        || actual.storage != expected.storage
        || actual.discard != expected.discard
    {
        return false;
    }
    match expected.name.as_str() {
        JOBS_STREAM => actual.allow_direct == expected.allow_direct,
        JOBS_WORK_STREAM => {
            actual.allow_direct == expected.allow_direct && actual.sources == expected.sources
        }
        JOBS_ADVISORIES_STREAM => actual.max_age == expected.max_age,
        HEALTH_STREAM => {
            actual.max_age == expected.max_age && actual.max_bytes == expected.max_bytes
        }
        EVENT_STREAM => true,
        _ => false,
    }
}

fn event_stream_config() -> stream::Config {
    stream::Config {
        name: EVENT_STREAM.to_owned(),
        subjects: vec!["events.>".to_owned()],
        retention: stream::RetentionPolicy::Limits,
        storage: stream::StorageType::File,
        discard: stream::DiscardPolicy::Old,
        ..Default::default()
    }
}

fn jobs_stream_configs() -> [stream::Config; 3] {
    [
        stream::Config {
            name: JOBS_STREAM.to_owned(),
            subjects: vec!["trellis.jobs.>".to_owned()],
            retention: stream::RetentionPolicy::Limits,
            storage: stream::StorageType::File,
            discard: stream::DiscardPolicy::Old,
            allow_direct: true,
            ..Default::default()
        },
        stream::Config {
            name: JOBS_WORK_STREAM.to_owned(),
            subjects: vec!["trellis.work.>".to_owned()],
            retention: stream::RetentionPolicy::WorkQueue,
            storage: stream::StorageType::File,
            discard: stream::DiscardPolicy::Old,
            allow_direct: true,
            sources: Some(vec![stream::Source {
                name: JOBS_STREAM.to_owned(),
                subject_transforms: vec![
                    stream::SubjectTransform {
                        source: "trellis.jobs.*.*.*.created".to_owned(),
                        destination: "trellis.work.$1.$2".to_owned(),
                    },
                    stream::SubjectTransform {
                        source: "trellis.jobs.*.*.*.retried".to_owned(),
                        destination: "trellis.work.$1.$2".to_owned(),
                    },
                ],
                ..Default::default()
            }]),
            ..Default::default()
        },
        stream::Config {
            name: JOBS_ADVISORIES_STREAM.to_owned(),
            subjects: vec!["$JS.EVENT.ADVISORY.CONSUMER.MAX_DELIVERIES.JOBS_WORK.>".to_owned()],
            retention: stream::RetentionPolicy::Limits,
            storage: stream::StorageType::File,
            discard: stream::DiscardPolicy::Old,
            max_age: Duration::from_secs(7 * 24 * 60 * 60),
            ..Default::default()
        },
    ]
}

fn health_stream_config(config: &RuntimeConfig) -> stream::Config {
    let health = config.health.as_ref();
    let max_age = Duration::from_secs(
        health
            .and_then(|health| health.transport_retention_hours)
            .map(u64::from)
            .unwrap_or(DEFAULT_TRANSPORT_RETENTION_HOURS)
            * 60
            * 60,
    );
    let max_bytes = health
        .and_then(|health| health.transport_max_bytes)
        .map(|bytes| i64::try_from(bytes).unwrap_or(i64::MAX))
        .unwrap_or(DEFAULT_TRANSPORT_MAX_BYTES);
    stream::Config {
        name: HEALTH_STREAM.to_owned(),
        subjects: vec![HEALTH_SUBJECT.to_owned()],
        retention: stream::RetentionPolicy::Limits,
        storage: stream::StorageType::File,
        discard: stream::DiscardPolicy::Old,
        max_age,
        max_bytes,
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> RuntimeConfig {
        RuntimeConfig::from_toml_str("").expect("empty optional config")
    }

    fn names(mode: RuntimeMode) -> Vec<String> {
        ExpectedRuntimeResources::for_mode(mode, &config())
            .streams()
            .iter()
            .map(|stream| stream.name.clone())
            .collect()
    }

    #[test]
    fn runtime_mode_derives_only_owned_streams() {
        assert_eq!(names(RuntimeMode::Platform), [EVENT_STREAM]);
        assert_eq!(
            names(RuntimeMode::Jobs),
            [JOBS_STREAM, JOBS_WORK_STREAM, JOBS_ADVISORIES_STREAM]
        );
        assert_eq!(names(RuntimeMode::Health), [HEALTH_STREAM]);
        assert_eq!(names(RuntimeMode::Eventlog), [EVENT_STREAM]);
        assert_eq!(
            names(RuntimeMode::All),
            [
                EVENT_STREAM,
                JOBS_STREAM,
                JOBS_WORK_STREAM,
                JOBS_ADVISORIES_STREAM,
                HEALTH_STREAM,
            ]
        );
    }

    #[test]
    fn all_mode_is_the_union_without_duplicate_event_streams() {
        let expected = ExpectedRuntimeResources::for_mode(RuntimeMode::All, &config());
        assert_eq!(
            expected
                .streams()
                .iter()
                .filter(|stream| stream.name == EVENT_STREAM)
                .count(),
            1
        );
        for subsystem in [
            SubsystemName::Platform,
            SubsystemName::Jobs,
            SubsystemName::Health,
            SubsystemName::Eventlog,
        ] {
            assert!(expected.requires(subsystem));
        }
    }

    #[test]
    fn stream_compatibility_rejects_non_durable_policies() {
        let expected = event_stream_config();
        let mut actual = expected.clone();
        actual.storage = stream::StorageType::Memory;
        assert!(!stream_is_compatible(&actual, &expected));

        actual = expected.clone();
        actual.discard = stream::DiscardPolicy::New;
        assert!(!stream_is_compatible(&actual, &expected));
    }
}
