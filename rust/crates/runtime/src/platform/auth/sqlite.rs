use std::sync::{Arc, Mutex};

use rusqlite::Connection;

mod accounts;
mod authority;
pub(in crate::platform::auth) mod common;
pub(super) mod contexts;
mod deployments;
mod evidence;
pub(super) mod outbox;
mod policy;
mod principals;
mod provisioning;
#[cfg(test)]
mod rollback_tests;
mod sessions;
pub(in crate::platform::auth) mod validation;
use common::SqliteConnectionPool;

/// Owner-scoped SQLite implementation of every authorization repository port.
#[derive(Clone, Debug)]
pub struct SqliteAuthorizationStore {
    writer: Arc<Mutex<Connection>>,
    readers: Option<Arc<SqliteConnectionPool>>,
}

const AUTHORIZATION_CONNECTION_POOL_SIZE: usize = 8;

#[cfg(test)]
mod pool_tests {
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };
    use std::{thread, time::Duration};

    use super::SqliteAuthorizationStore;
    use crate::storage::SqliteStore;
    use crate::{SqliteStorageConfig, SubsystemName};

    #[tokio::test]
    async fn file_store_runs_independent_operations_concurrently() {
        let directory = tempfile::tempdir().expect("create sqlite tempdir");
        let store = SqliteStore::new(
            SubsystemName::Platform,
            SqliteStorageConfig {
                path: directory.path().join("auth.sqlite"),
                journal_mode: Some("wal".to_owned()),
                busy_timeout_ms: Some(2_500),
                single_writer: Some(true),
            },
        );
        let repository = SqliteAuthorizationStore::open(&store).expect("open sqlite pool");
        let active = Arc::new(AtomicUsize::new(0));
        let maximum = Arc::new(AtomicUsize::new(0));
        let operations = (0..4).map(|_| {
            let repository = repository.clone();
            let active = Arc::clone(&active);
            let maximum = Arc::clone(&maximum);
            async move {
                repository
                    .run_read(move |_| {
                        let current = active.fetch_add(1, Ordering::SeqCst) + 1;
                        maximum.fetch_max(current, Ordering::SeqCst);
                        thread::sleep(Duration::from_millis(100));
                        active.fetch_sub(1, Ordering::SeqCst);
                        Ok(())
                    })
                    .await
            }
        });

        let results = futures_util::future::join_all(operations).await;
        assert!(results.into_iter().all(|result| result.is_ok()));
        assert_eq!(maximum.load(Ordering::SeqCst), 4);
    }
}
