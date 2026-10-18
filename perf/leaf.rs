//! Owned-fixture leaf pipeline probe; build using the scratch manifest recorded
//! in the evidence bundle. No arbitrary-root synchronization interface.
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use diff_dirs_sync::{CopyPolicy, Progress};

fn cpu_ns() -> u128 {
    let mut time = libc::timespec { tv_sec: 0, tv_nsec: 0 };
    // Process CPU includes all caller-pool workers. No filesystem operation.
    assert_eq!(unsafe { libc::clock_gettime(libc::CLOCK_PROCESS_CPUTIME_ID, &mut time) }, 0);
    time.tv_sec as u128 * 1_000_000_000 + time.tv_nsec as u128
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let scratch = fs::canonicalize(Path::new(env!("CARGO_MANIFEST_DIR")))?;
    let root = scratch.join(&args[2]);
    assert_eq!(Path::new(&args[2]).components().count(), 1);
    assert!(args[2].bytes().all(|c| c.is_ascii_alphanumeric() || c == b'-'));
    if args[1] == "setup" {
        fs::create_dir(&root)?;
        fs::write(root.join("OWNED"), b"leaf pipeline v1\n")?;
        let dirs: usize = args[3].parse()?;
        let count: usize = args[4].parse()?;
        let size: usize = args[5].parse()?;
        let payload = vec![b'x'; size];
        for dir in 0..dirs {
            let path = root.join(format!("source/d{dir:04}"));
            fs::create_dir_all(&path)?;
            for file in 0..count {
                fs::write(path.join(format!("f{file:05}")), &payload)?;
            }
            std::os::unix::fs::symlink("f00000", path.join("link"))?;
        }
        return Ok(());
    }
    assert_eq!(fs::read(root.join("OWNED"))?, b"leaf pipeline v1\n");
    let policy = match args[3].as_str() {
        "copy" => CopyPolicy::Copy,
        "strict" => CopyPolicy::ReflinkRequired,
        "fallback" => CopyPolicy::ReflinkOrCopy,
        _ => panic!("policy"),
    };
    let parallel = args[4] == "p4";
    let pool = rayon::ThreadPoolBuilder::new().num_threads(4).build()?;
    let source = root.join("source");
    let owned = tempfile::tempdir_in(&root)?;
    let target = owned.path().join("target");
    let apply = || {
        if parallel {
            diff_dirs_sync::mirror_in_pool(
                &source,
                &target,
                policy,
                &pool,
                diff_dirs::ParallelLimits::new(std::num::NonZeroUsize::new(4).unwrap(), 256),
            )
        } else {
            diff_dirs_sync::mirror(&source, &target, policy)
        }
    };
    let mut files = Vec::new();
    for dir in fs::read_dir(&source)? {
        for file in fs::read_dir(dir?.path())? {
            let file = file?;
            if file.file_type()?.is_file() {
                files.push(file.path().strip_prefix(&source)?.to_owned());
            }
        }
    }
    files.sort();
    for scenario in ["fresh", "dense", "clean", "sparse", "diff-only"] {
        if scenario == "dense" {
            for file in &files {
                fs::write(target.join(file), b"old")?;
            }
        } else if scenario == "sparse" {
            fs::write(target.join(&files[0]), b"old")?;
        }
        let cpu = cpu_ns();
        let start = Instant::now();
        let progress = if scenario == "diff-only" {
            compare(&source, &target, parallel, &pool)?;
            Progress::default()
        } else {
            apply()?.completed
        };
        let elapsed = start.elapsed().as_nanos();
        let cpu = cpu_ns() - cpu;
        if matches!(scenario, "clean" | "diff-only") {
            assert_eq!(progress, Progress::default());
        }
        compare(&source, &target, false, &pool)?;
        println!(
            "{},{},{},{scenario},{elapsed},{cpu},{},{},{},{},{}",
            args[2],
            args[3],
            args[4],
            progress.files_copied,
            progress.files_reflinked,
            progress.bytes_copied,
            progress.bytes_reflinked,
            progress.symlinks_created
        );
    }
    Ok(())
}

fn compare(source: &PathBuf, target: &PathBuf, parallel: bool, pool: &rayon::ThreadPool) -> Result<(), diff_dirs::Error> {
    if parallel {
        let mut walk = diff_dirs::walk_in_pool(
            source,
            target,
            pool,
            diff_dirs::ParallelLimits::new(std::num::NonZeroUsize::new(4).unwrap(), 256),
        )?;
        for item in walk.by_ref() {
            assert!(item?.is_empty());
        }
        assert!(walk.into_summary().is_clean());
    } else {
        let mut walk = diff_dirs::walk(source, target)?;
        for item in walk.by_ref() {
            assert!(item?.is_empty());
        }
        assert!(walk.into_summary().is_clean());
    }
    Ok(())
}
