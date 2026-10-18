//! Sync benchmark restricted to freshly generated, marked owned fixtures.
//! No arbitrary-root synchronization command is provided.
use std::fs;
use std::path::Path;
use std::time::Instant;

use diff_dirs_sync::{CopyPolicy, Report};

const MARKER: &str = ".diff-dirs-sync-benchmark";
const SIGNATURE: &str = "owned diff-dirs-sync benchmark fixture v1\n";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args_os().collect();
    match args.get(1).and_then(|arg| arg.to_str()) {
        Some("setup") if args.len() == 5 => {
            let root = Path::new(&args[2]);
            let packages: usize = args[3].to_str().unwrap().parse()?;
            let files: usize = args[4].to_str().unwrap().parse()?;
            fs::create_dir(root)?; // Never reuse/clear an existing root.
            fs::write(root.join(MARKER), SIGNATURE)?;
            let payload = vec![b'x'; 4096];
            for package in 0..packages {
                let dir = root.join(format!("source/package-{package:04}"));
                fs::create_dir_all(&dir)?;
                for file in 0..files {
                    fs::write(dir.join(format!("file-{file:04}")), &payload)?;
                }
                #[cfg(unix)]
                std::os::unix::fs::symlink("file-0000", dir.join("link"))?;
            }
            eprintln!(
                "owned source: {packages} directories, {} files, {} logical bytes, {packages} Unix links",
                packages * files,
                packages * files * payload.len()
            );
        }
        Some("run") if args.len() == 4 => {
            let root = fs::canonicalize(&args[2])?;
            assert_eq!(fs::read_to_string(root.join(MARKER))?, SIGNATURE, "not an owned fixture");
            let n: usize = args[3].to_str().unwrap().parse()?;
            let source = root.join("source");
            #[cfg(feature = "parallel")]
            let pool = rayon::ThreadPoolBuilder::new().num_threads(4).build()?;
            println!(
                "iteration,execution,policy,scenario,ns,observed_files_changed,observed_files_added,observed_files_removed,observed_dirs_added,observed_dirs_removed,files_copied,files_reflinked,bytes_copied,bytes_reflinked,symlinks_created,dirs_created,files_removed,symlinks_removed,dirs_removed,dir_permission_sets"
            );
            // One warmup per mode/policy, then rotate order across repetitions.
            for iteration in 0..=n {
                let mut configs = vec![(false, CopyPolicy::Copy), (false, CopyPolicy::ReflinkRequired)];
                #[cfg(feature = "parallel")]
                configs.extend([(true, CopyPolicy::Copy), (true, CopyPolicy::ReflinkRequired)]);
                let len = configs.len();
                configs.rotate_left(iteration % len);
                for (parallel, policy) in configs {
                    let owned = tempfile::tempdir_in(&root)?;
                    let target = owned.path().join("target");
                    let apply = || {
                        #[cfg(feature = "parallel")]
                        if parallel {
                            return diff_dirs_sync::mirror_in_pool(
                                &source,
                                &target,
                                policy,
                                &pool,
                                diff_dirs::ParallelLimits::new(std::num::NonZeroUsize::new(4).unwrap(), 256),
                            );
                        }
                        diff_dirs_sync::mirror(&source, &target, policy)
                    };
                    for scenario in ["fresh", "clean", "sparse", "subtree"] {
                        match scenario {
                            "sparse" => {
                                fs::remove_file(target.join("package-0000/file-0000"))?;
                                fs::write(target.join("package-0001/file-0001"), b"changed target size")?;
                                fs::write(target.join("stale"), b"remove this")?;
                            }
                            "subtree" => {
                                fs::remove_dir_all(target.join("package-0000"))?;
                                fs::create_dir(target.join("obsolete"))?;
                                for file in 0..16 {
                                    fs::write(target.join(format!("obsolete/{file}")), b"remove subtree")?;
                                }
                            }
                            _ => {}
                        }
                        let start = Instant::now();
                        let report = apply()?;
                        let elapsed = start.elapsed().as_nanos();
                        if iteration > 0 {
                            print_report(iteration, parallel, policy, scenario, elapsed, &report);
                        }
                        let mut verify = diff_dirs::walk(&source, &target)?;
                        assert!(verify.next().is_none(), "post-sync comparison differs");
                        assert!(verify.into_summary().is_clean());
                        if scenario == "clean" {
                            assert_eq!(report.completed, Default::default());
                            let start = Instant::now();
                            #[cfg(feature = "parallel")]
                            let summary = if parallel {
                                let mut walk = diff_dirs::walk_in_pool(
                                    &source,
                                    &target,
                                    &pool,
                                    diff_dirs::ParallelLimits::new(std::num::NonZeroUsize::new(4).unwrap(), 256),
                                )?;
                                for item in walk.by_ref() {
                                    assert!(item?.is_empty());
                                }
                                walk.into_summary()
                            } else {
                                compare(&source, &target)?
                            };
                            #[cfg(not(feature = "parallel"))]
                            let summary = compare(&source, &target)?;
                            let elapsed = start.elapsed().as_nanos();
                            assert!(summary.is_clean());
                            if iteration > 0 {
                                print_report(
                                    iteration,
                                    parallel,
                                    policy,
                                    "diff-only",
                                    elapsed,
                                    &Report {
                                        policy: Default::default(),
                                        completed: Default::default(),
                                        observed: Some(summary),
                                    },
                                );
                            }
                        }
                    }
                    // TempDir owns this generated target, never the input source.
                }
            }
        }
        _ => return Err("usage: sync_perf setup NEW_OWNED_ROOT PACKAGES FILES | run OWNED_ROOT REPETITIONS".into()),
    }
    Ok(())
}

fn compare(source: &Path, target: &Path) -> Result<diff_dirs::Summary, diff_dirs::Error> {
    let mut walk = diff_dirs::walk(source, target)?;
    for item in walk.by_ref() {
        assert!(item?.is_empty());
    }
    Ok(walk.into_summary())
}

fn print_report(iteration: usize, parallel: bool, policy: CopyPolicy, scenario: &str, elapsed: u128, report: &Report) {
    let empty = diff_dirs::EntryCounts::default();
    let counts = report.observed.as_ref().map_or(&empty, |summary| &summary.counts);
    let p = &report.completed;
    println!(
        "{iteration},{},{policy:?},{scenario},{elapsed},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
        if parallel { "pool4-j4-k256" } else { "serial" },
        counts.files.changed,
        counts.files.added,
        counts.files.removed,
        counts.dirs.added,
        counts.dirs.removed,
        p.files_copied,
        p.files_reflinked,
        p.bytes_copied,
        p.bytes_reflinked,
        p.symlinks_created,
        p.directories_created,
        p.files_removed,
        p.symlinks_removed,
        p.directories_removed,
        p.directory_permissions_set
    );
}
