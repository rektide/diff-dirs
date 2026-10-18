//! Reproducible metadata-shape benchmark, not a copy/sync benchmark.
//! See perf/README.md. Fixture setup is never part of a timed walk.

use std::error::Error;
use std::ffi::OsString;
use std::fs::{self, File};
use std::io;
use std::path::Path;
use std::time::{Duration, Instant, UNIX_EPOCH};

use diff_dirs::{EntryCounts, ScanState, Summary};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

const HEADER: &str = "scenario,iteration,elapsed_ns,records,examined_directories,files_changed,files_added,files_removed,dirs_added,dirs_removed,pruned";
const HELP: &str = "perf setup ROOT APPROX_ENTRIES\nperf run ROOT ITERATIONS [clean|sparse|one-sided|prune]\nperf once ORIGINAL TARGET\nperf repeat ORIGINAL TARGET ITERATIONS\nperf inspect ROOT";

fn main() -> Result<()> {
    let args = std::env::args_os().skip(1).collect::<Vec<_>>();
    match args.first().and_then(|s| s.to_str()) {
        Some("setup") if args.len() == 3 => setup(Path::new(&args[1]), number(&args[2])?),
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
                run_case(root, case, iterations)?;
            }
            Ok(())
        }
        Some("once") if args.len() == 3 => {
            println!("{HEADER}");
            let sample = sample(Path::new(&args[1]), Path::new(&args[2]), false)?;
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
            sample(original, target, false)?;
            println!("{HEADER}");
            for iteration in 0..iterations {
                print_sample("read-only", iteration, &sample(original, target, false)?);
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

fn invalid(message: &str) -> Box<dyn Error> {
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

fn run_case(root: &Path, case: &str, iterations: usize) -> Result<()> {
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

    let warm = sample(&original, &target, case == "prune")?;
    validate(case, &warm)?;
    for iteration in 0..iterations {
        let sample = sample(&original, &target, case == "prune")?;
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
    if !sample(&original, &target, false)?.summary.is_clean() {
        return Err(invalid("fixture did not restore to clean"));
    }
    Ok(())
}

struct Sample {
    elapsed: Duration,
    records: u64,
    summary: Summary,
}

fn sample(original: &Path, target: &Path, prune: bool) -> Result<Sample> {
    let start = Instant::now();
    let mut walk = diff_dirs::walk(original, target)?;
    let mut records = 0;
    while let Some(diff) = walk.next() {
        let diff = diff?;
        records += 1;
        if prune && diff.dir.as_os_str().is_empty() {
            assert!(walk.prune_here());
        }
    }
    let summary = walk.into_summary();
    let elapsed = start.elapsed();
    Ok(Sample { elapsed, records, summary })
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
        "{case},{iteration},{},{},{},{},{},{},{},{},{}",
        sample.elapsed.as_nanos(),
        sample.records,
        sample.summary.examined_directories,
        counts.files.changed,
        counts.files.added,
        counts.files.removed,
        counts.dirs.added,
        counts.dirs.removed,
        sample.summary.pruned.len(),
    );
}

#[cfg(test)]
#[path = "../tests/support/mod.rs"]
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
            run_case(&root, case, 1).unwrap();
        }
        let sample = sample(&root.join("original"), &root.join("target"), false).unwrap();
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
}
