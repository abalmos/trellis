use std::ffi::OsStr;
use std::path::Path;
use std::process::Command;
use std::time::Duration;

pub fn resolve<T>(
    _source: &Path,
    operation: impl FnOnce() -> miette::Result<T>,
) -> miette::Result<T> {
    operation()
}

pub fn command(program: impl AsRef<OsStr>) -> Command {
    Command::new(program)
}

pub fn target(_name: &str, _selected: bool, _generated: bool) {}

pub fn installed(_path: &Path) -> miette::Result<()> {
    Ok(())
}

pub fn resolution_cache_hit() {}

pub fn resolution_cache_miss(_reason: &str) {}

pub fn input_snapshot(_elapsed: Duration, _stat_files: usize, _hashed_files: usize) {}
