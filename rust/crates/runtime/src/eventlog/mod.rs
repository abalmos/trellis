//! Eventlog subsystem scaffold.

use crate::shutdown::StopHandle;
use crate::supervisor::{RuntimeContext, RuntimeError, SubsystemHandle};
use crate::SubsystemName;

pub(crate) fn start(context: &RuntimeContext) -> Result<SubsystemHandle, RuntimeError> {
    let _owner = context.owner(crate::ownership::OwnerGroup::Eventlog)?;
    let stop = StopHandle::new();
    let task_stop = stop.clone();
    let join = tokio::spawn(async move {
        task_stop.stopped().await;
        Ok(())
    });

    Ok(SubsystemHandle {
        name: SubsystemName::Eventlog,
        stop,
        join,
    })
}
