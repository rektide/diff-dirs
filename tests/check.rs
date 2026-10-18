mod support;

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use diff_dirs::{CheckError, Entry, Error, Purpose, ScanState, Side, audit, walk_checked};
use support::Fixture;

fn refuse_admin(entry: Entry<'_>) -> Result<(), CheckError> {
    let name = entry.relative.file_name().or_else(|| entry.root.file_name());
    if name.is_some_and(|name| name == ".git" || name == ".jj") {
        return Err("administrative name".into());
    }
    Ok(())
}

#[test]
fn equal_entries_are_checked_original_first_and_failed_frame_is_not_committed() {
    let f = Fixture::new();
    f.both_dir("a");
    f.both_dir("z");
    f.both_file("a/accepted", b"same");
    f.both_file("z/.git", b"same");
    let mut walk = walk_checked(&f.original, &f.target, refuse_admin).unwrap();
    let error = walk.next().unwrap().unwrap_err();
    assert!(matches!(error, Error::Refused { .. }));
    assert_eq!(error.path(), f.original.join("z/.git"));
    assert!(walk.next().is_none());
    let summary = walk.into_summary();
    assert_eq!(summary.state, ScanState::Failed);
    assert_eq!(summary.examined_directories, 2);
    assert!(!summary.has_differences());
}

#[test]
fn roots_and_every_equal_entry_have_borrowed_side_path_and_metadata() {
    let f = Fixture::new();
    f.both_dir("a");
    f.both_file("a/leaf", b"same");
    let seen = Mutex::new(Vec::new());
    let check = |entry: Entry<'_>| -> Result<(), CheckError> {
        assert_eq!(entry.purpose, Purpose::Compare);
        assert_eq!(entry.root, if entry.side == Side::Original { &f.original } else { &f.target });
        seen.lock().unwrap().push((entry.relative.to_path_buf(), entry.side, entry.metadata.is_dir()));
        Ok(())
    };
    let mut walk = walk_checked(&f.original, &f.target, check).unwrap();
    assert!(walk.next().is_none());
    assert!(walk.into_summary().is_clean());
    assert_eq!(
        *seen.lock().unwrap(),
        vec![
            (PathBuf::new(), Side::Original, true),
            (PathBuf::new(), Side::Target, true),
            ("a".into(), Side::Original, true),
            ("a".into(), Side::Target, true),
            ("a/leaf".into(), Side::Original, false),
            ("a/leaf".into(), Side::Target, false),
        ]
    );
}

#[test]
fn root_refusal_precedes_contents_and_audit_covers_opaque_interiors() {
    let f = Fixture::new();
    fs::create_dir_all(f.original.join("new/deep/.git")).unwrap();
    let mut walk = walk_checked(&f.original, &f.target, refuse_admin).unwrap();
    assert_eq!(walk.next().unwrap().unwrap().dirs.added, ["new"]);
    assert!(walk.next().is_none());
    assert!(walk.into_summary().is_complete()); // comparison, NOT guard coverage
    let failure = audit(&f.original, Side::Original, &refuse_admin).unwrap_err();
    assert_eq!(failure.error.path(), f.original.join("new/deep/.git"));
    assert_eq!(failure.inspected.directories, 3);
    let check = |entry: Entry<'_>| -> Result<(), CheckError> {
        assert_eq!(entry.relative, Path::new(""));
        Err("root refused before listing".into())
    };
    assert!(matches!(walk_checked(&f.original, &f.target, check), Err(Error::Refused { .. })));
}

#[cfg(unix)]
#[test]
fn literal_links_are_available_without_following_and_special_facts_reach_check() {
    use std::os::unix::{fs::symlink, net::UnixListener};
    let f = Fixture::new();
    let text = Path::new("/abs/../dangling");
    symlink(text, f.original.join("link")).unwrap();
    symlink(text, f.target.join("link")).unwrap();
    let links = Mutex::new(Vec::new());
    let check = |entry: Entry<'_>| -> Result<(), CheckError> {
        if entry.metadata.is_symlink() {
            assert_eq!(entry.link_target, Some(text));
            links.lock().unwrap().push(entry.side);
        }
        Ok(())
    };
    assert!(walk_checked(&f.original, &f.target, check).unwrap().next().is_none());
    assert_eq!(*links.lock().unwrap(), [Side::Original, Side::Target]);
    let _socket = UnixListener::bind(f.target.join("special")).unwrap();
    let check = |entry: Entry<'_>| -> Result<(), CheckError> {
        if !entry.metadata.is_dir() && !entry.metadata.is_file() && !entry.metadata.is_symlink() {
            return Err("special seen by policy".into());
        }
        Ok(())
    };
    assert!(matches!(
        walk_checked(&f.original, &f.target, check).unwrap().next().unwrap(),
        Err(Error::Refused { .. })
    ));
}

#[cfg(feature = "parallel")]
#[test]
fn parallel_refusal_prevents_child_admission_and_retains_ordered_prefix_at_k_zero_and_positive() {
    let f = Fixture::new();
    for dir in ["a/deep", "b/denied/never-open", "z/deep"] {
        f.both_dir(dir);
    }
    f.both_file("a/deep/leaf", b"same");
    f.both_file("z/deep/leaf", b"same");
    let pool = rayon::ThreadPoolBuilder::new().num_threads(4).build().unwrap();
    for k in [0, 1, 32] {
        let calls = Mutex::new(Vec::new());
        let check = |entry: Entry<'_>| -> Result<(), CheckError> {
            assert!(!entry.relative.starts_with("b/denied/never-open"));
            calls.lock().unwrap().push(entry.relative.to_path_buf());
            if entry.relative == Path::new("b/denied") {
                return Err("refuse before descent".into());
            }
            if entry.relative == Path::new("z/deep/leaf") {
                return Err("later speculative error".into());
            }
            Ok(())
        };
        let limits = diff_dirs::ParallelLimits::new(std::num::NonZeroUsize::new(4).unwrap(), k);
        let mut walk = diff_dirs::walk_in_pool_checked(&f.original, &f.target, &pool, limits, check).unwrap();
        assert_eq!(walk.next().unwrap().unwrap_err().path(), f.original.join("b/denied"));
        let count = calls.lock().unwrap().len();
        assert!(walk.next().is_none());
        assert_eq!(calls.lock().unwrap().len(), count); // all work already joined
        let summary = walk.into_summary();
        assert_eq!(summary.state, ScanState::Failed);
        assert_eq!(summary.examined_directories, 3); // root,a,a/deep, not b or z
    }
}

#[cfg(feature = "parallel")]
#[test]
fn real_parallel_checks_run_outside_frontier_lock_and_divergence_seals_stay_closed() {
    use std::sync::Condvar;
    use std::time::Duration;
    let f = Fixture::new();
    f.both_dir("a");
    f.both_dir("b");
    f.both_file("a/leaf", b"same");
    f.both_file("b/leaf", b"same");
    let pool = rayon::ThreadPoolBuilder::new().num_threads(2).build().unwrap();
    let entered = Mutex::new(0);
    let changed = Condvar::new();
    // Instrumentation is deliberately stateful to detect the placement, not a
    // recommended policy. A lock-held checker would prevent the second arrival.
    let check = |entry: Entry<'_>| -> Result<(), CheckError> {
        if entry.side == Side::Original && entry.relative.file_name().is_some_and(|name| name == "leaf") {
            let mut arrivals = entered.lock().unwrap();
            *arrivals += 1;
            changed.notify_all();
            let (arrivals, timeout) = changed.wait_timeout_while(arrivals, Duration::from_secs(5), |arrivals| *arrivals < 2).unwrap();
            assert!(!timeout.timed_out());
            assert_eq!(*arrivals, 2);
        }
        Ok(())
    };
    let limits = diff_dirs::ParallelLimits::for_pool(&pool);
    let mut walk = diff_dirs::walk_in_pool_checked(&f.original, &f.target, &pool, limits, check).unwrap();
    assert!(walk.next().is_none());
    assert!(walk.into_summary().is_clean());

    support::file(&f.original.join("new"), b"root divergence");
    let check = |entry: Entry<'_>| -> Result<(), CheckError> {
        assert!(!entry.relative.ends_with("leaf"), "descendant inspected through a closed divergence seal");
        Ok(())
    };
    let mut walk = diff_dirs::walk_in_pool_checked(&f.original, &f.target, &pool, limits, check).unwrap();
    assert_eq!(walk.next().unwrap().unwrap().dir, Path::new(""));
    assert!(walk.prune_here());
    assert!(walk.next().is_none());
    assert!(!walk.into_summary().is_complete());
}
