//! Curated public Rust facade for Trellis clients, services, contracts, auth, and jobs.
//!
//! This crate is the normal Rust authoring entrypoint. It re-exports stable,
//! commonly used runtime types without exposing low-level service loops,
//! bootstrap hosts, or generated artifact internals.
//!
//! Generated SDK crates and participant facades include a package-local
//! `TRELLIS.md` for AI-agent use. Participant facades are the connection
//! boundary and expose generated caller methods through `.client()`, service
//! resources through `.service()`, and provider registration through
//! `.handle()`.
//!
//! Prepared event and outbox/inbox support lives under [`client`]:
//! `PreparedTrellisEvent`, `prepare_event::<Descriptor>(...)`,
//! `publish_prepared`, `dispatch_outbox_once`, `OutboxStore`, `InboxStore`,
//! `SqliteOutboxStore`, `SqliteInboxStore`, `PostgresOutboxStore`, and
//! `PostgresInboxStore`.
//!
//! # Authoring model
//!
//! Generated participant facades are the normal connection boundary. Their
//! generated `.client()` methods call typed RPCs, invoke operations and signals,
//! publish or subscribe to events, and access contract state without exposing
//! subjects. Service facades connect with [`service::ServiceConnectOptions`],
//! register generated RPC and operation handlers, publish typed events, process
//! private Jobs queues, and access resolved KV and object-store handles.
//!
//! Connection and request failures retain typed authentication, transport,
//! validation, declared-RPC, and bootstrap errors. Callers should retry only
//! errors documented as transient; service bootstrap already retries the
//! authority-pending state according to its connect options.
//!
//! ```no_run
//! use std::sync::Arc;
//! use trellis_rs::{client::FileAuthorizationContextStore, service::ServiceConnectOptions};
//!
//! let _options = ServiceConnectOptions::new(
//!     "http://localhost:3000",
//!     "documents-worker",
//!     "dep_documents",
//!     "documents-worker@v1",
//!     "participant-digest",
//!     "participant-needs-digest",
//!     r#"{"format":"trellis.participant.v1","id":"documents-worker@v1","kind":"service"}"#,
//!     r#"{"format":"trellis.api.v1","id":"documents-worker@v1"}"#,
//!     "api-digest",
//!     &[],
//!     "base64url-identity-seed",
//!     "base64url-session-seed",
//!     Arc::new(FileAuthorizationContextStore::new("./trellis-context.json")),
//! )
//! .with_timeout_ms(10_000);
//! ```

#[doc(hidden)]
pub mod client;

#[doc(hidden)]
pub mod generated;

/// High-level service runtime and service-authoring support types.
pub mod service;

/// Native API and participant artifact helper types.
pub mod contracts {
    pub use trellis_contracts::{
        canonicalize_json, digest_json, event, schema_ref, sha256_base64url, state, use_contract,
        ApiArtifactV1, ApiBuilder, ContractArtifacts, ContractBuilder, ContractCapabilityMetadata,
        ContractEventConsumerGroup, ContractEventConsumerOrdering, ContractEventConsumerReplay,
        ContractKind, ContractStateKind, ContractsError, PageRequest, PageResponse,
    };
}

#[doc(hidden)]
pub mod auth;

#[doc(hidden)]
pub mod jobs;

/// Public facades for Trellis-owned generated contract SDKs.
pub mod sdk {
    /// Auth contract SDK surface.
    pub mod auth;

    /// Core contract SDK surface.
    pub mod core;

    /// Health contract SDK surface.
    pub mod health;

    /// Event Log contract SDK surface.
    pub mod eventlog;

    /// Jobs contract SDK surface.
    pub mod jobs;

    /// State contract SDK surface.
    pub mod state;
}

#[cfg(test)]
mod tests {
    use std::fs;

    #[test]
    fn exposes_core_facade_modules() {
        let _request = crate::contracts::PageRequest {
            offset: None,
            limit: 25,
        };
        let _options = crate::service::ServiceConnectOptions::new(
            "http://localhost:8080",
            "svc",
            "dep_1",
            "svc@v1",
            "participant-digest",
            "participant-needs-digest",
            "{\"format\":\"trellis.participant.v1\",\"id\":\"svc@v1\"}",
            "{\"format\":\"trellis.api.v1\",\"id\":\"api@v1\"}",
            "api-digest",
            &[],
            "identity-seed",
            "session-seed",
            std::sync::Arc::new(crate::client::MemoryAuthorizationContextStore::default()),
        );
        let _state = crate::jobs::JobState::Pending;
    }

    #[test]
    fn low_level_workspace_crates_are_not_publishable_packages() {
        let crate_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let crates_dir = crate_dir
            .parent()
            .expect("trellis crate should live under rust/crates");
        for manifest in [
            "cli/Cargo.toml",
            "codegen-rust/Cargo.toml",
            "codegen-ts/Cargo.toml",
            "bootstrap/Cargo.toml",
            "generate-runner/Cargo.toml",
            "jobs/Cargo.toml",
            "local-bootstrap/Cargo.toml",
            "protocol-wasm/Cargo.toml",
            "runtime/Cargo.toml",
            "eventlog-runtime/Cargo.toml",
            "jobs-runtime/Cargo.toml",
            "trellis-test/Cargo.toml",
        ] {
            let contents = fs::read_to_string(crates_dir.join(manifest))
                .expect("internal crate manifest should be readable");
            assert!(
                contents.contains("publish = false"),
                "{manifest} must stay non-publishable"
            );
        }
    }

    #[test]
    fn trellis_does_not_depend_on_generated_trellis_owned_sdk_packages() {
        let manifest =
            fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml"))
                .expect("trellis manifest should be readable");
        for package in [
            "trellis-sdk-auth",
            "trellis-sdk-core",
            "trellis-sdk-health",
            "trellis-sdk-jobs",
            "trellis-sdk-state",
        ] {
            assert!(
                !manifest.contains(package),
                "{package} must be embedded as trellis_rs::sdk, not a trellis dependency"
            );
        }
    }

    #[test]
    fn trellis_does_not_depend_on_old_internal_package_identities() {
        let manifest =
            fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml"))
                .expect("trellis manifest should be readable");
        for package in [
            "trellis-auth",
            "trellis-client",
            "trellis-jobs",
            "trellis-service",
            "trellis-service-runtime",
        ] {
            assert!(
                !manifest.contains(package),
                "{package} must be implemented as a trellis module, not a trellis dependency"
            );
        }
    }

    #[test]
    fn trellis_owned_generated_sdk_packages_are_not_publishable_packages() {
        let repo_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(3)
            .expect("trellis crate should live under rust/crates/trellis");
        for manifest in [
            "generated/packages/cargo/auth/Cargo.toml",
            "generated/packages/cargo/health/Cargo.toml",
            "generated/packages/cargo/jobs/Cargo.toml",
        ] {
            let contents = fs::read_to_string(repo_root.join(manifest))
                .expect("generated Trellis-owned SDK manifest should be readable");
            assert!(
                contents.contains("publish = false"),
                "{manifest} must stay non-publishable"
            );
        }
    }
}
