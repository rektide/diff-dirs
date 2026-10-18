#![cfg(feature = "parallel")]

mod support;

#[cfg(unix)]
use std::fs;
use std::num::NonZeroUsize;
use std::path::Path;

use diff_dirs::{DirDiff, ParallelLimits, Summary, walk, walk_in_pool};
use support::{Fixture, file};

fn limits(j: usize, k: usize) -> ParallelLimits {
    ParallelLimits::new(NonZeroUsize::new(j).unwrap(), k)
}

fn gather(f: &Fixture, pool: &rayon::ThreadPool, k: usize) -> (Vec<DirDiff>, Summary) {
    let mut walk = walk_in_pool(&f.original, &f.target, pool, limits(pool.current_num_threads(), k)).unwrap();
    let diffs = walk.by_ref().collect::<Result<Vec<_>, _>>().unwrap();
    (diffs, walk.into_summary())
}

#[test]
fn explicit_pool_outside_inside_nested_and_concurrent_walks_progress() {
    let f = Fixture::new();
    for dir in ["a/deep", "b", "c/deep"] {
        f.both_dir(dir);
        f.both_file(Path::new(dir).join("same"), b"same");
        file(&f.original.join(dir).join("new"), b"new");
    }
    let mut serial = walk(&f.original, &f.target).unwrap();
    let expected = (serial.by_ref().collect::<Result<Vec<_>, _>>().unwrap(), serial.into_summary());
    for p in [1, 2, 4] {
        let pool = rayon::ThreadPoolBuilder::new().num_threads(p).build().unwrap();
        for k in [0, 1, 4 * p] {
            assert_eq!(gather(&f, &pool, k), expected);
            assert_eq!(pool.install(|| gather(&f, &pool, k)), expected);
            let (a, b) = pool.install(|| rayon::join(|| gather(&f, &pool, k), || gather(&f, &pool, k)));
            assert_eq!(a, expected);
            assert_eq!(b, expected);
        }
        let outer = rayon::ThreadPoolBuilder::new().num_threads(1).build().unwrap();
        assert_eq!(outer.install(|| gather(&f, &pool, 4 * p)), expected);
    }
}

#[test]
fn summaries_after_each_early_stop_equal_the_serial_prefix_not_lookahead() {
    let f = Fixture::new();
    for dir in ["a", "a/deep", "b", "c"] {
        f.both_dir(dir);
        file(&f.original.join(dir).join("new"), b"");
    }
    let pool = rayon::ThreadPoolBuilder::new().num_threads(4).build().unwrap();
    for stop_after in 0..=4 {
        let mut serial = walk(&f.original, &f.target).unwrap();
        let mut parallel = walk_in_pool(&f.original, &f.target, &pool, limits(4, 16)).unwrap();
        for _ in 0..stop_after {
            assert_eq!(serial.next().unwrap().unwrap(), parallel.next().unwrap().unwrap());
        }
        assert_eq!(serial.into_summary(), parallel.into_summary());
    }
}

#[cfg(unix)]
#[test]
fn earlier_dfs_error_wins_and_later_speculative_errors_do_not_leak_into_prefix() {
    use std::os::unix::net::UnixListener;
    let f = Fixture::new();
    for dir in ["a/deep", "z"] {
        f.both_dir(dir);
    }
    let _first = UnixListener::bind(f.original.join("a/deep/socket")).unwrap();
    let _later = UnixListener::bind(f.target.join("z/socket")).unwrap();
    let pool = rayon::ThreadPoolBuilder::new().num_threads(4).build().unwrap();
    let mut serial = walk(&f.original, &f.target).unwrap();
    let mut parallel = walk_in_pool(&f.original, &f.target, &pool, limits(4, 16)).unwrap();
    let a = serial.next().unwrap().unwrap_err();
    let b = parallel.next().unwrap().unwrap_err();
    assert_eq!(a.path(), b.path());
    assert_eq!(b.path(), f.original.join("a/deep/socket"));
    assert!(parallel.next().is_none());
    assert!(!parallel.prune_here());
    assert_eq!(serial.into_summary(), parallel.into_summary());
}

#[cfg(unix)]
fn copy_entry(original: &Path, target: &Path) {
    let metadata = fs::symlink_metadata(original).unwrap();
    if metadata.is_symlink() {
        std::os::unix::fs::symlink(fs::read_link(original).unwrap(), target).unwrap();
    } else if metadata.is_dir() {
        fs::create_dir(target).unwrap();
        for entry in fs::read_dir(original).unwrap() {
            let entry = entry.unwrap();
            copy_entry(&entry.path(), &target.join(entry.file_name()));
        }
    } else {
        fs::copy(original, target).unwrap();
        support::set_time(target, metadata.modified().unwrap());
    }
}

#[cfg(unix)]
#[test]
fn test_side_apply_converges_with_native_names_links_and_all_type_transitions() {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;
    use std::os::unix::fs::symlink;

    for p in [1, 2, 4] {
        let f = Fixture::new();
        f.both_dir("paired/deep");
        f.both_file("paired/deep/changed", b"old");
        file(&f.original.join("paired/deep/changed"), b"new and longer");
        file(&f.original.join(OsString::from_vec(vec![b'n', 0xff])), b"native");
        file(&f.target.join("stale"), b"");
        for (name, a, b) in [
            ("file-dir", 0, 2),
            ("dir-file", 2, 0),
            ("file-link", 0, 1),
            ("link-file", 1, 0),
            ("dir-link", 2, 1),
            ("link-dir", 1, 2),
        ] {
            for (root, kind) in [(&f.original, a), (&f.target, b)] {
                match kind {
                    0 => file(&root.join(name), b"file"),
                    1 => symlink("./missing//literal", root.join(name)).unwrap(),
                    _ => fs::create_dir(root.join(name)).unwrap(),
                }
            }
        }
        let pool = rayon::ThreadPoolBuilder::new().num_threads(p).build().unwrap();
        let mut walk = walk_in_pool(&f.original, &f.target, &pool, ParallelLimits::for_pool(&pool)).unwrap();
        for item in walk.by_ref() {
            let diff = item.unwrap();
            let original = f.original.join(&diff.dir);
            let target = f.target.join(&diff.dir);
            for name in &diff.dirs.removed {
                fs::remove_dir_all(target.join(name)).unwrap();
            }
            for name in diff.files.removed.iter().chain(&diff.files.changed) {
                fs::remove_file(target.join(name)).unwrap();
            }
            for name in diff.files.added.iter().chain(&diff.files.changed).chain(&diff.dirs.added) {
                copy_entry(&original.join(name), &target.join(name));
            }
        }
        assert!(walk.into_summary().is_complete());
        assert!(gather(&f, &pool, 4 * p).1.is_clean());
    }
}
