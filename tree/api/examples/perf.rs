//! Reproducible metadata-shape benchmark, not a copy/sync benchmark.
//! See perf/README.md. Fixture setup is never part of a timed walk.

use std::error::Error;
use std::ffi::OsString;
use std::fs::{self, File};
use std::io;
use std::path::Path;
use std::time::{Duration, Instant, UNIX_EPOCH};

use diff_dirs::{DirDiff, EntryCounts, Execution, ScanState, Summary, WalkOptions};
use diff_dirs_tree_api as diff_dirs;

type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;

const HEADER: &str =
    "scenario,iteration,elapsed_ns,records,examined_directories,files_changed,files_added,files_removed,dirs_added,dirs_removed,pruned,first_record_ns";
const HELP: &str = "perf setup ROOT APPROX_ENTRIES\nperf run ROOT ITERATIONS [clean|sparse|one-sided|prune]\nperf once ORIGINAL TARGET\nperf repeat ORIGINAL TARGET ITERATIONS\nperf inspect ROOT";

fn main() -> Result<()> {
    let args = std::env::args_os().skip(1).collect::<Vec<_>>();
    #[cfg(feature = "parallel-bench")]
    if args.first().and_then(|s| s.to_str()) == Some("compare") {
        return compare_backends(&args[1..]);
    }
    #[cfg(feature = "parallel-diagnostics")]
    if args.first().and_then(|s| s.to_str()) == Some("diagnose") {
        return diagnose(&args[1..]);
    }
    #[cfg(feature = "parallel-bench")]
    if args.first().and_then(|s| s.to_str()) == Some("backend-once") {
        return backend_once(&args[1..]);
    }
    #[cfg(feature = "parallel")]
    if args.first().and_then(|s| s.to_str()) == Some("parallel") {
        let threads = number(args.get(1).ok_or_else(|| invalid("parallel requires thread count"))?)?;
        if threads == 0 {
            return Err(invalid("thread count must be positive"));
        }
        // A caller-owned pool, built once outside all timing intervals. No
        // library pool creation or global-pool configuration is involved.
        let pool = rayon::ThreadPoolBuilder::new().num_threads(threads).build()?;
        eprintln!("execution=parallel caller_pool_threads={threads}");
        return pool.install(|| command(&args[2..], Execution::Parallel));
    }
    command(&args, Execution::Sequential)
}

fn command(args: &[OsString], execution: Execution) -> Result<()> {
    match args.first().and_then(|s| s.to_str()) {
        Some("setup") if args.len() == 3 => setup(Path::new(&args[1]), number(&args[2])?),
        Some("shape") if args.len() == 4 => setup_shape(
            Path::new(&args[1]),
            args[2].to_str().ok_or_else(|| invalid("invalid shape"))?,
            number(&args[3])?,
        ),
        Some("run") if args.len() == 3 || args.len() == 4 => {
            let iterations = number(&args[2])?;
            if iterations == 0 {
                return Err(invalid("iterations must be positive"));
            }
            let cases = if let Some(case) = args.get(3) {
                vec![case.to_str().ok_or_else(|| invalid("invalid scenario"))?]
            } else {
                vec!["clean", "sparse", "one-sided", "prune"]
            };
            let root = Path::new(&args[1]);
            eprintln!("fixture={root:?}; one untimed warmup per scenario; iterations={iterations}");
            println!("{HEADER}");
            for case in cases {
                run_case(root, case, iterations, execution)?;
            }
            Ok(())
        }
        Some("once") if args.len() == 3 => {
            println!("{HEADER}");
            let sample = sample(Path::new(&args[1]), Path::new(&args[2]), false, execution)?;
            print_sample("once", 0, &sample);
            Ok(())
        }
        Some("repeat") if args.len() == 4 => {
            let iterations = number(&args[3])?;
            if iterations == 0 {
                return Err(invalid("iterations must be positive"));
            }
            let original = Path::new(&args[1]);
            let target = Path::new(&args[2]);
            eprintln!("read-only original={original:?} target={target:?}; one untimed warmup; iterations={iterations}");
            sample(original, target, false, execution)?;
            println!("{HEADER}");
            for iteration in 0..iterations {
                print_sample("read-only", iteration, &sample(original, target, false, execution)?);
            }
            Ok(())
        }
        Some("inspect") if args.len() == 2 => inspect(Path::new(&args[1])),
        _ => Err(invalid(HELP)),
    }
}

// Separate read-only metadata census. Never run within a timing interval.
// Unlike comparison, this enumerates one-sided subtrees as well.
fn inspect(root: &Path) -> Result<()> {
    if !fs::symlink_metadata(root)?.is_dir() {
        return Err(invalid("inspect root must be a real directory"));
    }
    let mut pending = vec![root.to_path_buf()];
    let (mut files, mut directories, mut symlinks, mut specials, mut bytes) = (0u64, 0u64, 0u64, 0u64, 0u64);
    while let Some(dir) = pending.pop() {
        let entries = fs::read_dir(&dir).map_err(|source| diff_dirs::Error::Io {
            path: dir.clone(),
            operation: "read census directory",
            source,
        })?;
        for entry in entries {
            let entry = entry.map_err(|source| diff_dirs::Error::Io {
                path: dir.clone(),
                operation: "read census entry",
                source,
            })?;
            let metadata = entry.metadata().map_err(|source| diff_dirs::Error::Io {
                path: entry.path(),
                operation: "inspect census entry",
                source,
            })?;
            if metadata.is_dir() {
                directories += 1;
                pending.push(entry.path());
            } else if metadata.is_file() {
                files += 1;
                bytes += metadata.len();
            } else if metadata.is_symlink() {
                symlinks += 1;
            } else {
                specials += 1;
            }
        }
    }
    println!(
        "root={root:?} files={files} directories={directories} symlinks={symlinks} specials={specials} file_bytes={bytes} entries={}",
        files + directories + symlinks + specials
    );
    Ok(())
}

fn invalid(message: &str) -> Box<dyn Error + Send + Sync> {
    io::Error::new(io::ErrorKind::InvalidInput, message).into()
}

fn number(value: &OsString) -> Result<usize> {
    Ok(value.to_str().ok_or_else(|| invalid("invalid integer"))?.parse()?)
}

fn empty_file(path: &Path) -> io::Result<()> {
    File::create(path)?.set_modified(UNIX_EPOCH + Duration::new(1_700_000_000, 123_456_789))
}

fn setup(root: &Path, budget: usize) -> Result<()> {
    let packages = budget / entries_per_package();
    if packages < 2 {
        return Err(invalid("fixture budget must contain at least two packages"));
    }
    // Reject existing fixture roots rather than resetting or reusing unknown data.
    fs::create_dir(root)?;
    for side in ["original", "target"] {
        let base = root.join(side).join("packages");
        fs::create_dir_all(&base)?;
        for package in 0..packages {
            let dir = base.join(format!("pkg-{package:06}"));
            fs::create_dir(&dir)?;
            for part in ["lib", "dist", "types"] {
                let subdir = dir.join(part);
                fs::create_dir(&subdir)?;
                for file in 0..50 {
                    empty_file(&subdir.join(format!("file-{file:03}.bin")))?;
                }
            }
            #[cfg(unix)]
            std::os::unix::fs::symlink("lib/file-000.bin", dir.join("link"))?;
        }
    }
    println!(
        "root={root:?} requested_entries={budget} packages={packages} files_per_side={} dirs_per_side={} symlinks_per_side={} entries_per_side={} payload_bytes=0",
        packages * 150,
        packages * 4 + 1,
        if cfg!(unix) { packages } else { 0 },
        packages * entries_per_package() + 1,
    );
    Ok(())
}

fn entries_per_package() -> usize {
    154 + usize::from(cfg!(unix))
}

fn run_case(root: &Path, case: &str, iterations: usize, execution: Execution) -> Result<()> {
    let original = root.join("original");
    let target = root.join("target");
    let changed = target.join("packages/pkg-000000/lib/file-000.bin");
    let added = original.join("packages/pkg-000000/lib/new.bin");
    let removed = target.join("packages/pkg-000001/dist/stale.bin");
    let trigger = original.join("trigger.bin");
    let held_original = root.join("held-original");
    let held_target = root.join("held-target");
    let moved_original = original.join("packages/pkg-000000");
    let moved_target = target.join("packages/pkg-000001");

    // All mutations are outside every timing interval; no copying is measured.
    match case {
        "clean" => {}
        "sparse" => {
            fs::write(&changed, b"x")?;
            empty_file(&added)?;
            empty_file(&removed)?;
        }
        "one-sided" => {
            fs::rename(&moved_original, &held_original)?;
            fs::rename(&moved_target, &held_target)?;
        }
        "prune" => empty_file(&trigger)?,
        _ => return Err(invalid("unknown scenario")),
    }

    let warm = sample(&original, &target, case == "prune", execution)?;
    validate(case, &warm)?;
    for iteration in 0..iterations {
        let sample = sample(&original, &target, case == "prune", execution)?;
        validate(case, &sample)?;
        print_sample(case, iteration, &sample);
    }

    match case {
        "sparse" => {
            empty_file(&changed)?;
            fs::remove_file(added)?;
            fs::remove_file(removed)?;
        }
        "one-sided" => {
            fs::rename(held_original, moved_original)?;
            fs::rename(held_target, moved_target)?;
        }
        "prune" => fs::remove_file(trigger)?,
        _ => {}
    }
    // A final untimed scan checks restoration; failures leave the fixture for
    // inspection, never automatically remove/reconcile it.
    if !sample(&original, &target, false, execution)?.summary.is_clean() {
        return Err(invalid("fixture did not restore to clean"));
    }
    Ok(())
}

struct Sample {
    elapsed: Duration,
    records: u64,
    summary: Summary,
    first_record: Option<Duration>,
    #[cfg(feature = "parallel-diagnostics")]
    diagnostics: Option<diff_dirs::ParallelDiagnostics>,
}

fn sample(original: &Path, target: &Path, prune: bool, execution: Execution) -> Result<Sample> {
    let start = Instant::now();
    consume(diff_dirs::walk_with_options(original, target, WalkOptions { execution })?, start, prune)
}

trait DrivenWalk: Iterator<Item = std::result::Result<DirDiff, diff_dirs::Error>> {
    fn prune(&mut self) -> bool;
    fn finish(self) -> Summary;
    #[cfg(feature = "parallel-diagnostics")]
    fn statistics(&self) -> Option<diff_dirs::ParallelDiagnostics> {
        None
    }
}

impl DrivenWalk for diff_dirs::Walk {
    fn prune(&mut self) -> bool {
        self.prune_here()
    }
    fn finish(self) -> Summary {
        self.into_summary()
    }
}

#[cfg(feature = "parallel")]
impl DrivenWalk for diff_dirs::ParallelWalk<'_> {
    fn prune(&mut self) -> bool {
        self.prune_here()
    }
    fn finish(self) -> Summary {
        self.into_summary()
    }
    #[cfg(feature = "parallel-diagnostics")]
    fn statistics(&self) -> Option<diff_dirs::ParallelDiagnostics> {
        Some(self.diagnostics().clone())
    }
}

fn consume<W: DrivenWalk>(mut walk: W, start: Instant, prune: bool) -> Result<Sample> {
    let mut records = 0;
    let mut first_record = None;
    while let Some(diff) = walk.next() {
        let diff = diff?;
        first_record.get_or_insert_with(|| start.elapsed());
        records += 1;
        if prune && diff.dir.as_os_str().is_empty() {
            assert!(walk.prune());
        }
    }
    #[cfg(feature = "parallel-diagnostics")]
    let diagnostics = walk.statistics();
    let summary = walk.finish();
    let elapsed = start.elapsed();
    Ok(Sample {
        elapsed,
        records,
        summary,
        first_record,
        #[cfg(feature = "parallel-diagnostics")]
        diagnostics,
    })
}

#[cfg(feature = "parallel-bench")]
#[derive(Clone, Copy)]
enum Backend<'pool> {
    Serial,
    PerFrame(&'pool rayon::ThreadPool),
    Frontier(&'pool rayon::ThreadPool, diff_dirs::ParallelLimits),
}

#[cfg(feature = "parallel-bench")]
impl Backend<'_> {
    fn label(self) -> &'static str {
        match self {
            Self::Serial => "serial",
            Self::PerFrame(_) => "per-frame",
            Self::Frontier(_, _) => "frontier",
        }
    }

    fn sample(self, original: &Path, target: &Path) -> Result<Sample> {
        let start = Instant::now();
        match self {
            Self::Serial => consume(diff_dirs::walk(original, target)?, start, false),
            Self::PerFrame(pool) => pool.install(|| consume(diff_dirs::walk_per_frame(original, target)?, start, false)),
            Self::Frontier(pool, limits) => consume(diff_dirs::walk_in_pool(original, target, pool, limits)?, start, false),
        }
    }

    fn gather(self, original: &Path, target: &Path) -> Result<(Vec<DirDiff>, Summary)> {
        fn gather<W: DrivenWalk>(mut walk: W) -> Result<(Vec<DirDiff>, Summary)> {
            let records = walk.by_ref().collect::<std::result::Result<Vec<_>, _>>()?;
            Ok((records, walk.finish()))
        }
        match self {
            Self::Serial => gather(diff_dirs::walk(original, target)?),
            Self::PerFrame(pool) => pool.install(|| gather(diff_dirs::walk_per_frame(original, target)?)),
            Self::Frontier(pool, limits) => gather(diff_dirs::walk_in_pool(original, target, pool, limits)?),
        }
    }
}

#[cfg(feature = "parallel-bench")]
fn compare_backends(args: &[OsString]) -> Result<()> {
    if args.len() != 6 && args.len() != 7 {
        return Err(invalid("compare ORIGINAL TARGET P ITERATIONS J K [LABEL]"));
    }
    let original = Path::new(&args[0]);
    let target = Path::new(&args[1]);
    let p = number(&args[2])?;
    let iterations = number(&args[3])?;
    if p == 0 || iterations == 0 {
        return Err(invalid("P and ITERATIONS must be positive"));
    }
    let j = std::num::NonZeroUsize::new(number(&args[4])?).ok_or_else(|| invalid("J must be positive"))?;
    let k = number(&args[5])?;
    let label = args.get(6).and_then(|s| s.to_str()).unwrap_or("comparison");
    let pool = rayon::ThreadPoolBuilder::new().num_threads(p).build()?;
    let backends = [
        Backend::Serial,
        Backend::PerFrame(&pool),
        Backend::Frontier(&pool, diff_dirs::ParallelLimits::new(j, k)),
    ];
    eprintln!(
        "read-only comparison original={original:?} target={target:?} P={p} J={j} K={k}; full output parity + warmup outside timing; rotating backend order; iterations={iterations}"
    );
    let expected = backends[0].gather(original, target)?;
    for backend in backends {
        if backend.gather(original, target)? != expected {
            return Err(invalid("full record/summary parity failed (or live tree changed)"));
        }
        backend.sample(original, target)?;
    }
    println!(
        "sample,scenario,backend,threads,j,k,iteration,elapsed_ns,first_record_ns,records,examined_directories,files_changed,files_added,files_removed,dirs_added,dirs_removed,pruned"
    );
    for iteration in 0..iterations {
        for offset in 0..backends.len() {
            let backend = backends[(iteration + offset) % backends.len()];
            let sample = backend.sample(original, target)?;
            if sample.summary != expected.1 {
                return Err(invalid("timed summary changed from parity baseline"));
            }
            let counts = &sample.summary.counts;
            println!(
                "{}-p{p}-j{j}-k{k},{label},{},{p},{j},{k},{iteration},{},{},{},{},{},{},{},{},{},{}",
                backend.label(),
                backend.label(),
                sample.elapsed.as_nanos(),
                sample.first_record.map(|d| d.as_nanos().to_string()).unwrap_or_default(),
                sample.records,
                sample.summary.examined_directories,
                counts.files.changed,
                counts.files.added,
                counts.files.removed,
                counts.dirs.added,
                counts.dirs.removed,
                sample.summary.pruned.len()
            );
        }
    }
    Ok(())
}

#[cfg(feature = "parallel-bench")]
fn backend_once(args: &[OsString]) -> Result<()> {
    if args.len() != 6 {
        return Err(invalid("backend-once serial|per-frame|frontier ORIGINAL TARGET P J K"));
    }
    let p = number(&args[3])?;
    if p == 0 {
        return Err(invalid("P must be positive"));
    }
    let j = std::num::NonZeroUsize::new(number(&args[4])?).ok_or_else(|| invalid("J must be positive"))?;
    let k = number(&args[5])?;
    let pool = rayon::ThreadPoolBuilder::new().num_threads(p).build()?;
    let backend = match args[0].to_str() {
        Some("serial") => Backend::Serial,
        Some("per-frame") => Backend::PerFrame(&pool),
        Some("frontier") => Backend::Frontier(&pool, diff_dirs::ParallelLimits::new(j, k)),
        _ => return Err(invalid("unknown backend")),
    };
    println!("{HEADER}");
    print_sample(backend.label(), 0, &backend.sample(Path::new(&args[1]), Path::new(&args[2]))?);
    Ok(())
}

#[cfg(feature = "parallel-diagnostics")]
fn diagnose(args: &[OsString]) -> Result<()> {
    if args.len() != 5 {
        return Err(invalid("diagnose ORIGINAL TARGET P J K"));
    }
    let p = number(&args[2])?;
    if p == 0 {
        return Err(invalid("P must be positive"));
    }
    let j = std::num::NonZeroUsize::new(number(&args[3])?).ok_or_else(|| invalid("J must be positive"))?;
    let k = number(&args[4])?;
    let pool = rayon::ThreadPoolBuilder::new().num_threads(p).build()?;
    let start = Instant::now();
    let sample = consume(
        diff_dirs::walk_in_pool(&args[0], &args[1], &pool, diff_dirs::ParallelLimits::new(j, k))?,
        start,
        false,
    )?;
    print_sample("instrumented", 0, &sample);
    println!(
        "P={p} J={j} K={k} wall_ns={} first_record_ns={:?} physical={:#?}",
        sample.elapsed.as_nanos(),
        sample.first_record.map(|d| d.as_nanos()),
        sample.diagnostics.unwrap()
    );
    Ok(())
}

fn setup_shape(root: &Path, shape: &str, count: usize) -> Result<()> {
    if !matches!(shape, "small" | "skewed" | "dense" | "wide") || count == 0 {
        return Err(invalid("shape ROOT small|skewed|dense|wide COUNT (positive)"));
    }
    fs::create_dir(root)?;
    for side in ["original", "target"] {
        let base = root.join(side);
        fs::create_dir(&base)?;
        if shape == "wide" {
            for i in 0..count {
                empty_file(&base.join(format!("file-{i:06}")))?;
            }
        } else {
            let parent = base.join(if shape == "skewed" { "a" } else { "packages" });
            fs::create_dir(&parent)?;
            if shape == "skewed" {
                for name in ["b", "c", "z"] {
                    fs::create_dir(base.join(name))?;
                }
            }
            for i in 0..count {
                let group = parent.join(format!("group-{:05}", i / 64));
                if i % 64 == 0 {
                    fs::create_dir(&group)?;
                }
                let dir = group.join(format!("dir-{i:06}"));
                fs::create_dir(&dir)?;
                for name in ["one", "two"] {
                    empty_file(&dir.join(name))?;
                }
                if shape == "dense" && side == "original" {
                    fs::write(dir.join("one"), b"x")?;
                }
            }
        }
    }
    eprintln!("owned fixture root={root:?} shape={shape} count={count}; setup excluded from timing");
    inspect(&root.join("original"))?;
    inspect(&root.join("target"))
}

fn validate(case: &str, sample: &Sample) -> Result<()> {
    let mut expected = EntryCounts::default();
    match case {
        "sparse" => {
            expected.files.changed = 1;
            expected.files.added = 1;
            expected.files.removed = 1;
        }
        "one-sided" => {
            expected.dirs.added = 1;
            expected.dirs.removed = 1;
        }
        "prune" => expected.files.added = 1,
        _ => {}
    }
    if sample.summary.counts != expected || sample.summary.state != ScanState::Complete || sample.summary.pruned.len() != usize::from(case == "prune") {
        return Err(invalid(&format!("unexpected fixture result for {case}: {:?}", sample.summary)));
    }
    Ok(())
}

fn print_sample(case: &str, iteration: usize, sample: &Sample) {
    let counts = &sample.summary.counts;
    println!(
        "{case},{iteration},{},{},{},{},{},{},{},{},{},{}",
        sample.elapsed.as_nanos(),
        sample.records,
        sample.summary.examined_directories,
        counts.files.changed,
        counts.files.added,
        counts.files.removed,
        counts.dirs.added,
        counts.dirs.removed,
        sample.summary.pruned.len(),
        sample.first_record.map(|d| d.as_nanos().to_string()).unwrap_or_default(),
    );
}

#[cfg(test)]
#[path = "../../../tests/support/fixture.rs"]
mod support;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixture_and_all_scenarios_restore_clean() {
        let fixture = support::Fixture::new();
        let root = fixture.root.join("benchmark");
        setup(&root, entries_per_package() * 2).unwrap();
        for case in ["clean", "sparse", "one-sided", "prune"] {
            run_case(&root, case, 1, Execution::Sequential).unwrap();
        }
        let sample = sample(&root.join("original"), &root.join("target"), false, Execution::Sequential).unwrap();
        assert!(sample.summary.is_clean());
        assert_eq!(sample.summary.examined_directories, 10);
    }

    #[test]
    fn setup_never_reuses_an_existing_root() {
        let fixture = support::Fixture::new();
        assert!(setup(&fixture.root, entries_per_package() * 2).is_err());
        assert!(fixture.original.is_dir());
        assert!(fixture.target.is_dir());
    }

    #[test]
    fn topology_fixtures_have_the_expected_scope_and_dense_records() {
        let fixture = support::Fixture::new();
        for (shape, directories, records) in [("small", 7, 0), ("skewed", 10, 0), ("dense", 7, 4), ("wide", 1, 0)] {
            let root = fixture.root.join(shape);
            setup_shape(&root, shape, 4).unwrap();
            let sample = sample(&root.join("original"), &root.join("target"), false, Execution::Sequential).unwrap();
            assert_eq!(sample.summary.examined_directories, directories);
            assert_eq!(sample.records, records);
            assert_eq!(sample.summary.counts.files.changed, records);
            assert_eq!(sample.summary.is_clean(), records == 0);
        }
    }

    #[cfg(feature = "parallel-bench")]
    #[test]
    fn same_build_controls_compare_full_records_and_demand_only_frontier() {
        let fixture = support::Fixture::new();
        let root = fixture.root.join("small");
        setup_shape(&root, "dense", 8).unwrap();
        compare_backends(&[
            root.join("original").into_os_string(),
            root.join("target").into_os_string(),
            "2".into(),
            "1".into(),
            "2".into(),
            "0".into(),
            "smoke".into(),
        ])
        .unwrap();
        backend_once(&[
            "frontier".into(),
            root.join("original").into_os_string(),
            root.join("target").into_os_string(),
            "2".into(),
            "2".into(),
            "0".into(),
        ])
        .unwrap();
    }
}
