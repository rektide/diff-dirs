#![cfg(feature = "parallel")]

use diff_dirs_tree_api as diff_dirs;

mod support;

use std::fs;
use std::path::Path;

use diff_dirs::{DirDiff, Execution, ScanState, Summary, WalkOptions, walk_with_options};
use support::{Fixture, file};

fn gather(f: &Fixture, execution: Execution) -> (Vec<DirDiff>, Summary) {
    let mut walk = walk_with_options(&f.original, &f.target, WalkOptions { execution }).unwrap();
    let diffs = walk.by_ref().collect::<Result<Vec<_>, _>>().unwrap();
    (diffs, walk.into_summary())
}

#[test]
fn sequential_and_parallel_records_and_summaries_match() {
    let f = Fixture::new();
    for dir in ["", "a", "a/deep", "z"] {
        if !dir.is_empty() {
            f.both_dir(dir);
        }
        for i in (0..24).rev() {
            let path = Path::new(dir).join(format!("file-{i:03}"));
            f.both_file(&path, b"old");
            if i % 7 == 0 {
                file(&f.original.join(&path), b"changed and larger");
            } else if i % 11 == 0 {
                fs::remove_file(f.original.join(&path)).unwrap();
            } else if i % 13 == 0 {
                fs::remove_file(f.target.join(&path)).unwrap();
            }
        }
    }
    fs::create_dir(f.original.join("source-only")).unwrap();
    fs::create_dir(f.target.join("target-only")).unwrap();
    let sequential = gather(&f, Execution::Sequential);
    let pool = rayon::ThreadPoolBuilder::new().num_threads(4).build().unwrap();
    let parallel = pool.install(|| gather(&f, Execution::Parallel));
    assert_eq!(sequential, parallel);
}

#[test]
fn parallel_work_joins_before_apply_and_prune_boundaries() {
    let f = Fixture::new();
    f.both_dir("pending");
    for i in 0..96 {
        file(&f.original.join(format!("new-{i:03}")), b"new");
    }
    let pool = rayon::ThreadPoolBuilder::new().num_threads(2).build().unwrap();
    pool.install(|| {
        let mut walk = walk_with_options(
            &f.original,
            &f.target,
            WalkOptions {
                execution: Execution::Parallel,
            },
        )
        .unwrap();
        let diff = walk.next().unwrap().unwrap();
        assert_eq!(diff.dir, Path::new(""));
        for name in diff.files.added {
            fs::copy(f.original.join(&name), f.target.join(&name)).unwrap();
            support::set_time(&f.target.join(name), support::timestamp());
        }
        fs::remove_dir(f.target.join("pending")).unwrap();
        assert!(walk.prune_here());
        assert!(walk.next().is_none());
        let summary = walk.into_summary();
        assert_eq!(summary.state, ScanState::Complete);
        assert_eq!(summary.examined_directories, 1);
        assert!(!summary.is_complete());
    });
    fs::create_dir(f.target.join("pending")).unwrap();
    assert!(gather(&f, Execution::Parallel).1.is_clean());
}

#[cfg(unix)]
#[test]
fn deterministic_error_priority_is_name_then_original_side_not_worker_race() {
    use std::os::unix::net::UnixListener;

    let f = Fixture::new();
    for i in 0..64 {
        f.both_file(format!("file-{i:03}"), b"");
    }
    let _later = UnixListener::bind(f.original.join("z-error")).unwrap();
    let _target = UnixListener::bind(f.target.join("a-error")).unwrap();
    let first_path = f.original.join("a-error");
    let first = UnixListener::bind(&first_path).unwrap();
    for expected in [&first_path, &f.target.join("a-error")] {
        for execution in [Execution::Sequential, Execution::Parallel] {
            let mut walk = walk_with_options(&f.original, &f.target, WalkOptions { execution }).unwrap();
            assert_eq!(walk.next().unwrap().unwrap_err().path(), expected);
            assert!(walk.next().is_none());
            assert_eq!(walk.into_summary().state, ScanState::Failed);
        }
        if expected == &first_path {
            fs::remove_file(&first_path).unwrap();
        }
    }
    drop(first);
}
