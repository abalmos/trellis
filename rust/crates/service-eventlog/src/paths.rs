use std::path::PathBuf;

const SYSTEM_EVENTLOG_DB_PATH: &str = "/var/lib/trellis/eventlog.sqlite";
const USER_EVENTLOG_DB_RELATIVE_PATH: &str = ".var/lib/trellis/eventlog.sqlite";
const EVENTLOG_DB_PATH_ENV: &str = "TRELLIS_EVENTLOG_DB_PATH";

pub(crate) fn eventlog_db_path_from_env() -> PathBuf {
    std::env::var_os(EVENTLOG_DB_PATH_ENV)
        .map(PathBuf::from)
        .unwrap_or_else(default_eventlog_db_path)
}

fn default_eventlog_db_path() -> PathBuf {
    default_eventlog_db_path_for(
        running_as_root(),
        std::env::var_os("HOME").map(PathBuf::from),
    )
}

fn default_eventlog_db_path_for(is_root: bool, home: Option<PathBuf>) -> PathBuf {
    if is_root {
        return PathBuf::from(SYSTEM_EVENTLOG_DB_PATH);
    }

    home.unwrap_or_else(|| PathBuf::from("."))
        .join(USER_EVENTLOG_DB_RELATIVE_PATH)
}

#[cfg(target_os = "linux")]
fn running_as_root() -> bool {
    std::fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|status| effective_uid_from_proc_status(&status))
        == Some(0)
}

#[cfg(not(target_os = "linux"))]
fn running_as_root() -> bool {
    false
}

#[cfg(target_os = "linux")]
fn effective_uid_from_proc_status(status: &str) -> Option<u32> {
    status.lines().find_map(|line| {
        let uids = line.strip_prefix("Uid:")?;
        uids.split_whitespace().nth(1)?.parse().ok()
    })
}
