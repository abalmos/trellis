from pathlib import Path


def replace_once(source: str, old: str, new: str, label: str) -> str:
    count = source.count(old)
    if count != 1:
        raise RuntimeError(f"{label}: expected one match, found {count}")
    return source.replace(old, new, 1)


api_path = Path("rust/crates/trellis/src/jobs/api.rs")
api = api_path.read_text()
api = replace_once(
    api,
    "use std::sync::Arc;\n\nuse futures_util::future::BoxFuture;\n",
    "",
    "remove JobRef callback imports",
)
api = replace_once(
    api,
    "use crate::jobs::manager::TrellisJobMetaSource;\nuse crate::jobs::types::{Job, JobContext, JobLogEntry, JobProgress, JobState};\nuse crate::jobs::TrellisJobEventPublisher;\n\npub(super) type RuntimeJob = RuntimeActiveJob<TrellisJobEventPublisher, TrellisJobMetaSource>;\n",
    "use crate::jobs::manager::{JobManager, TrellisJobMetaSource};\nuse crate::jobs::projection::is_terminal;\nuse crate::jobs::runtime_ref::NatsJobWaiter;\nuse crate::jobs::types::{Job, JobContext, JobLogEntry, JobProgress, JobState};\nuse crate::jobs::TrellisJobEventPublisher;\n\npub(super) type RuntimeJob = RuntimeActiveJob<TrellisJobEventPublisher, TrellisJobMetaSource>;\ntype RuntimeJobManager = JobManager<TrellisJobEventPublisher, TrellisJobMetaSource>;\n",
    "add concrete JobRef runtime types",
)
start = api.index("/// Handle for a created job.\n")
end = api.index("/// Typed snapshot of one job.\n", start)
replacement = '''/// Handle for a created job.\npub struct JobRef<TPayload, TResult> {\n    identity: JobIdentity,\n    seed: Job,\n    waiter: NatsJobWaiter,\n    manager: RuntimeJobManager,\n    _types: PhantomData<fn() -> (TPayload, TResult)>,\n}\n\nimpl<TPayload, TResult> std::fmt::Debug for JobRef<TPayload, TResult> {\n    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {\n        formatter\n            .debug_struct("JobRef")\n            .field("identity", &self.identity)\n            .finish_non_exhaustive()\n    }\n}\n\nimpl<TPayload, TResult> Clone for JobRef<TPayload, TResult> {\n    fn clone(&self) -> Self {\n        Self {\n            identity: self.identity.clone(),\n            seed: self.seed.clone(),\n            waiter: self.waiter.clone(),\n            manager: self.manager.clone(),\n            _types: PhantomData,\n        }\n    }\n}\n\nimpl<TPayload, TResult> JobRef<TPayload, TResult>\nwhere\n    TPayload: DeserializeOwned + Clone + Send + Sync + 'static,\n    TResult: DeserializeOwned + Clone + Send + Sync + 'static,\n{\n    pub(crate) fn from_runtime(\n        seed: Job,\n        waiter: NatsJobWaiter,\n        manager: RuntimeJobManager,\n    ) -> Self {\n        Self {\n            identity: JobIdentity::from(&seed),\n            seed,\n            waiter,\n            manager,\n            _types: PhantomData,\n        }\n    }\n\n    #[doc = concat!("Trellis API operation `", stringify!(identity), "`.")]\n    pub fn identity(&self) -> &JobIdentity {\n        &self.identity\n    }\n\n    #[doc = concat!("Asynchronous Trellis API operation `", stringify!(get), "`.")]\n    pub async fn get(&self) -> Result<JobSnapshot<TPayload, TResult>, JobsError> {\n        JobSnapshot::try_from(self.waiter.get(self.seed.clone()).await?)\n    }\n\n    #[doc = concat!("Asynchronous Trellis API operation `", stringify!(wait), "`.")]\n    pub async fn wait(&self) -> Result<TerminalJob<TPayload, TResult>, JobsError> {\n        JobSnapshot::try_from(self.waiter.wait_for_terminal(self.seed.clone()).await?)\n    }\n\n    #[doc = concat!("Asynchronous Trellis API operation `", stringify!(cancel), "`.")]\n    pub async fn cancel(&self) -> Result<JobSnapshot<TPayload, TResult>, JobsError> {\n        let current = self.waiter.get(self.seed.clone()).await?;\n        if is_terminal(current.state) {\n            return JobSnapshot::try_from(current);\n        }\n        self.manager.cancel(&current).await.map_err(jobs_message)?;\n        JobSnapshot::try_from(self.waiter.get(current).await?)\n    }\n}\n\n'''
api_path.write_text(api[:start] + replacement + api[end:])

facade_path = Path("rust/crates/trellis/src/service/runtime_facade.rs")
facade = facade_path.read_text()
facade = replace_once(
    facade,
    "    start_worker_host_from_client, JobDescriptor, JobIdentity, JobManager, JobProcessError, JobRef,\n    JobSnapshot, JobsError, TrellisJobEventPublisher, TrellisJobMetaSource, WorkerHostHandle,\n",
    "    start_worker_host_from_client, JobDescriptor, JobManager, JobProcessError, JobRef, JobsError,\n    TrellisJobEventPublisher, TrellisJobMetaSource, WorkerHostHandle,\n",
    "remove callback-only facade imports",
)
old = '''        let state = Arc::new(Mutex::new(job.clone()));\n        let identity = JobIdentity::from(&job);\n\n        let get_state = Arc::clone(&state);\n        let get_waiter = waiter.clone();\n        let wait_state = Arc::clone(&state);\n        let wait_waiter = waiter.clone();\n        let cancel_state = Arc::clone(&state);\n        let cancel_waiter = waiter;\n        let cancel_manager = manager;\n        Ok(JobRef::new(\n            identity,\n            move || {\n                let state = Arc::clone(&get_state);\n                let waiter = get_waiter.clone();\n                Box::pin(async move {\n                    let current = state.lock().await.clone();\n                    let current = waiter.get(current).await?;\n                    *state.lock().await = current.clone();\n                    JobSnapshot::try_from(current)\n                })\n            },\n            move || {\n                let state = Arc::clone(&wait_state);\n                let waiter = wait_waiter.clone();\n                Box::pin(async move {\n                    let current = state.lock().await.clone();\n                    let current = waiter.wait_for_terminal(current).await?;\n                    *state.lock().await = current.clone();\n                    JobSnapshot::try_from(current)\n                })\n            },\n            move || {\n                let state = Arc::clone(&cancel_state);\n                let waiter = cancel_waiter.clone();\n                let manager = cancel_manager.clone();\n                Box::pin(async move {\n                    let current = state.lock().await.clone();\n                    if crate::jobs::projection::is_terminal(current.state) {\n                        return JobSnapshot::try_from(current);\n                    }\n                    manager\n                        .cancel(&current)\n                        .await\n                        .map_err(|error| JobsError::Message {\n                            message: error.to_string(),\n                        })?;\n                    let current = waiter.get(current).await?;\n                    *state.lock().await = current.clone();\n                    JobSnapshot::try_from(current)\n                })\n            },\n        ))\n'''
facade = replace_once(
    facade,
    old,
    "        Ok(JobRef::from_runtime(job, waiter, manager))\n",
    "replace generated JobRef callback/cache construction",
)
facade_path.write_text(facade)
