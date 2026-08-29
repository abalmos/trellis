use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

#[derive(Debug, Default)]
struct TargetCount {
    generated: usize,
    skipped: usize,
}

#[derive(Debug)]
struct State {
    started: Instant,
    phases: Vec<(String, Duration)>,
    resolutions: Vec<(String, Duration)>,
    subprocesses: BTreeMap<String, usize>,
    contracts: (usize, usize, usize),
    targets: BTreeMap<String, TargetCount>,
    installed_files: usize,
    installed_bytes: u64,
    cache_hits: usize,
    cache_misses: BTreeMap<String, usize>,
    input_snapshot_time: Duration,
    stat_files: usize,
    hashed_files: usize,
    reused_digests: usize,
}

static STATE: OnceLock<Mutex<Option<State>>> = OnceLock::new();
static ACTIVE: AtomicBool = AtomicBool::new(false);

pub fn run<T>(enabled: bool, operation: impl FnOnce() -> miette::Result<T>) -> miette::Result<T> {
    if !enabled {
        return operation();
    }
    *state().lock().expect("timings mutex poisoned") = Some(State {
        started: Instant::now(),
        phases: Vec::new(),
        resolutions: Vec::new(),
        subprocesses: BTreeMap::new(),
        contracts: (0, 0, 0),
        targets: BTreeMap::new(),
        installed_files: 0,
        installed_bytes: 0,
        cache_hits: 0,
        cache_misses: BTreeMap::new(),
        input_snapshot_time: Duration::ZERO,
        stat_files: 0,
        hashed_files: 0,
        reused_digests: 0,
    });
    ACTIVE.store(true, Ordering::Relaxed);
    let result = operation();
    ACTIVE.store(false, Ordering::Relaxed);
    let collected = state().lock().expect("timings mutex poisoned").take();
    if let Some(collected) = collected {
        print_report(collected);
    }
    result
}

pub fn phase<T>(name: &str, operation: impl FnOnce() -> miette::Result<T>) -> miette::Result<T> {
    if !enabled() {
        return operation();
    }
    let started = Instant::now();
    let result = operation();
    with_state(|state| state.phases.push((name.to_string(), started.elapsed())));
    result
}

pub fn resolve<T>(
    source: &Path,
    operation: impl FnOnce() -> miette::Result<T>,
) -> miette::Result<T> {
    if !enabled() {
        return operation();
    }
    let started = Instant::now();
    let result = operation();
    with_state(|state| {
        state
            .resolutions
            .push((source.display().to_string(), started.elapsed()));
    });
    result
}

pub fn command(program: impl AsRef<OsStr>) -> Command {
    let program = program.as_ref();
    if enabled() {
        let name = Path::new(program)
            .file_name()
            .unwrap_or(program)
            .to_string_lossy()
            .into_owned();
        with_state(|state| *state.subprocesses.entry(name).or_default() += 1);
    }
    Command::new(program)
}

pub fn contracts(generated: usize, verified: usize, skipped: usize) {
    with_state(|state| state.contracts = (generated, verified, skipped));
}

pub fn target(name: &str, selected: bool, generated: bool) {
    if !selected {
        return;
    }
    with_state(|state| {
        let count = state.targets.entry(name.to_string()).or_default();
        if generated {
            count.generated += 1;
        } else {
            count.skipped += 1;
        }
    });
}

pub fn installed(path: &Path) -> miette::Result<()> {
    if !enabled() || !path.exists() {
        return Ok(());
    }
    fn measure(path: &Path) -> std::io::Result<(usize, u64)> {
        if path.is_file() {
            return Ok((1, path.metadata()?.len()));
        }
        let mut total = (0, 0);
        for entry in fs::read_dir(path)? {
            let measured = measure(&entry?.path())?;
            total.0 += measured.0;
            total.1 += measured.1;
        }
        Ok(total)
    }
    let measured = measure(path).map_err(|error| miette::miette!(error))?;
    with_state(|state| {
        state.installed_files += measured.0;
        state.installed_bytes += measured.1;
    });
    Ok(())
}

pub fn resolution_cache_hit() {
    with_state(|state| state.cache_hits += 1);
}

pub fn resolution_cache_miss(reason: &str) {
    with_state(|state| *state.cache_misses.entry(reason.to_string()).or_default() += 1);
}

pub fn input_snapshot(elapsed: Duration, stat_files: usize, hashed_files: usize) {
    with_state(|state| {
        state.input_snapshot_time += elapsed;
        state.stat_files += stat_files;
        state.hashed_files += hashed_files;
        state.reused_digests += stat_files.saturating_sub(hashed_files);
    });
}

fn enabled() -> bool {
    ACTIVE.load(Ordering::Relaxed)
}

fn state() -> &'static Mutex<Option<State>> {
    STATE.get_or_init(|| Mutex::new(None))
}

fn with_state(operation: impl FnOnce(&mut State)) {
    if !enabled() {
        return;
    }
    if let Some(state) = state().lock().expect("timings mutex poisoned").as_mut() {
        operation(state);
    }
}

fn print_report(state: State) {
    println!("\nTimings");
    for (name, elapsed) in state.phases {
        println!("  {name:<24} {:>8.3} s", elapsed.as_secs_f64());
    }
    println!(
        "  {:<24} {:>8.3} s",
        "total",
        state.started.elapsed().as_secs_f64()
    );
    if !state.resolutions.is_empty() {
        let mut totals = BTreeMap::<&str, Duration>::new();
        for (source, elapsed) in &state.resolutions {
            let kind = match Path::new(source).extension().and_then(OsStr::to_str) {
                Some("rs") => "resolve Rust",
                Some("ts" | "js" | "tsx" | "jsx") => "resolve TypeScript",
                _ => "resolve protocol/image",
            };
            *totals.entry(kind).or_default() += *elapsed;
        }
        for (name, elapsed) in totals {
            println!("  {name:<24} {:>8.3} s", elapsed.as_secs_f64());
        }
        println!("  contract resolution");
        for (source, elapsed) in state.resolutions {
            println!("    {:>8.3} s  {source}", elapsed.as_secs_f64());
        }
    }
    if !state.subprocesses.is_empty() {
        println!("  subprocesses");
        for (name, count) in state.subprocesses {
            println!("    {name:<20} {count:>5}");
        }
    }
    if state.cache_hits > 0 || !state.cache_misses.is_empty() {
        println!("  resolution cache");
        println!("    {:<20} {:>5}", "hits", state.cache_hits);
        println!(
            "    {:<20} {:>5}",
            "misses",
            state.cache_misses.values().sum::<usize>()
        );
        for (reason, count) in state.cache_misses {
            println!("    {reason:<20} {count:>5}");
        }
        println!("  input snapshot");
        println!(
            "    {:<20} {:>8.3} s",
            "time",
            state.input_snapshot_time.as_secs_f64()
        );
        println!("    {:<20} {:>8}", "stat files", state.stat_files);
        println!("    {:<20} {:>8}", "hashed files", state.hashed_files);
        println!("    {:<20} {:>8}", "reused digests", state.reused_digests);
    }
    println!(
        "  contracts               generated={} verified={} skipped={}",
        state.contracts.0, state.contracts.1, state.contracts.2
    );
    for (name, count) in state.targets {
        println!(
            "  target {name:<14} generated={} skipped={}",
            count.generated, count.skipped
        );
    }
    println!(
        "  installed               files={} bytes={}",
        state.installed_files, state.installed_bytes
    );
}
