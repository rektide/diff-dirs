mod support;

use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, UNIX_EPOCH};

use diff_dirs::{Error, ScanState, walk};
use support::{Fixture, file, gather, set_time, timestamp};

fn names(items: &[&str]) -> Vec<OsString> {
    items.iter().map(OsString::from).collect()
}

fn paths(items: &[&str]) -> Vec<PathBuf> {
    items.iter().map(PathBuf::from).collect()
}

#[test]
fn empty_and_nested_clean_trees_are_silent_and_complete() {
    let f = Fixture::new();
    let (diffs, summary) = gather(&f);
    assert!(diffs.is_empty());
    assert!(summary.is_clean());
    assert_eq!(summary.examined_directories, 1);
    f.both_dir("a/b/c");
    f.both_file("a/b/c/file", b"same");
    let (diffs, summary) = gather(&f);
    assert!(diffs.is_empty());
    assert!(summary.is_clean());
    assert_eq!(summary.examined_directories, 4);
    assert!(summary.changed_dirs.is_empty());
}

#[test]
fn direction_counts_and_one_sided_subtrees() {
    let f = Fixture::new();
    file(&f.original.join("new"), b"new");
    file(&f.target.join("stale"), b"old");
    file(&f.original.join("changed"), b"longer");
    file(&f.target.join("changed"), b"short");
    fs::create_dir_all(f.original.join("new-tree/deep")).unwrap();
    fs::create_dir_all(f.target.join("old-tree/deep")).unwrap();
    let (diffs, summary) = gather(&f);
    assert_eq!(diffs.len(), 1);
    assert_eq!(diffs[0].dir, Path::new(""));
    assert_eq!(diffs[0].files.added, names(&["new"]));
    assert_eq!(diffs[0].files.removed, names(&["stale"]));
    assert_eq!(diffs[0].files.changed, names(&["changed"]));
    assert_eq!(diffs[0].dirs.added, names(&["new-tree"]));
    assert_eq!(diffs[0].dirs.removed, names(&["old-tree"]));
    assert_eq!(summary.counts.files.added, 1);
    assert_eq!(summary.counts.files.removed, 1);
    assert_eq!(summary.counts.files.changed, 1);
    assert_eq!(summary.counts.dirs.added, 1);
    assert_eq!(summary.counts.dirs.removed, 1);
    assert_eq!(summary.examined_directories, 1);
    assert_eq!(summary.topmost_divergent, paths(&[""]));
    assert_eq!(summary.changed_dirs, paths(&[""]));
    assert!(summary.is_complete());
    assert!(!summary.is_clean());
}

#[test]
fn deterministic_bytewise_depth_first_order_and_sorted_categories() {
    let f = Fixture::new();
    for dir in ["z", "a/z", "a/a", "é", "b"] {
        f.both_dir(dir);
        for name in ["z", "é", "a", "Z"] {
            file(&f.original.join(dir).join(name), b"new");
        }
    }
    file(&f.original.join("root-change"), b"");
    let (diffs, summary) = gather(&f);
    assert_eq!(
        diffs.iter().map(|d| d.dir.clone()).collect::<Vec<_>>(),
        paths(&["", "a/a", "a/z", "b", "z", "é"])
    );
    for diff in &diffs[1..] {
        assert_eq!(diff.files.added, names(&["Z", "a", "z", "é"]));
    }
    assert_eq!(summary.topmost_divergent, paths(&[""]));
    assert_eq!(summary.changed_dirs, paths(&["a/a", "a/z", "a", "b", "z", "é", ""]));
    assert_eq!(gather(&f).0, diffs);
}

#[test]
fn topmost_means_directly_divergent_not_root_for_every_change() {
    let f = Fixture::new();
    for dir in ["a/deep", "b/child", "clean"] {
        f.both_dir(dir);
    }
    for dir in ["a/deep", "b", "b/child"] {
        file(&f.original.join(dir).join("new"), b"");
    }
    let (_, summary) = gather(&f);
    assert_eq!(summary.topmost_divergent, paths(&["a/deep", "b"]));
    assert_eq!(summary.changed_dirs, paths(&["a/deep", "a", "b/child", "b", ""]));
    assert_eq!(summary.counts.files.added, 3);
}

#[test]
fn directory_mtimes_never_suppress_nested_divergence() {
    let f = Fixture::new();
    f.both_dir("a/deep");
    f.both_file("a/deep/file", b"old");
    let before = fs::metadata(f.original.join("a/deep")).unwrap().modified().unwrap();
    file(&f.original.join("a/deep/file"), b"new and bigger");
    assert_eq!(fs::metadata(f.original.join("a/deep")).unwrap().modified().unwrap(), before);
    let (diffs, _) = gather(&f);
    assert_eq!(diffs.len(), 1);
    assert_eq!(diffs[0].dir, Path::new("a/deep"));
    assert_eq!(diffs[0].files.changed, names(&["file"]));
}

#[test]
fn same_size_and_mtime_content_changes_are_deliberately_trusted() {
    let f = Fixture::new();
    file(&f.original.join("file"), b"aaa");
    file(&f.target.join("file"), b"bbb");
    assert!(gather(&f).1.is_clean());
}

#[test]
fn exact_nanoseconds_and_pre_epoch_times() {
    let f = Fixture::new();
    f.both_file("file", b"same");
    set_time(&f.target.join("file"), timestamp() + Duration::from_nanos(1));
    // This test intentionally verifies nanosecond behavior on the fixture FS.
    assert_ne!(
        fs::metadata(f.original.join("file")).unwrap().modified().unwrap(),
        fs::metadata(f.target.join("file")).unwrap().modified().unwrap()
    );
    assert_eq!(gather(&f).0[0].files.changed, names(&["file"]));
    let pre_epoch = UNIX_EPOCH - Duration::new(123, 456_789);
    set_time(&f.original.join("file"), pre_epoch);
    set_time(&f.target.join("file"), pre_epoch);
    assert_eq!(fs::metadata(f.original.join("file")).unwrap().modified().unwrap(), pre_epoch);
    assert!(gather(&f).1.is_clean());
    set_time(&f.target.join("file"), pre_epoch - Duration::from_nanos(1));
    assert_eq!(gather(&f).0[0].files.changed, names(&["file"]));
}

#[test]
fn file_directory_transitions_have_paired_categories() {
    let f = Fixture::new();
    file(&f.original.join("want-file"), b"");
    fs::create_dir_all(f.target.join("want-file/deep")).unwrap();
    fs::create_dir_all(f.original.join("want-dir/deep")).unwrap();
    file(&f.target.join("want-dir"), b"");
    let (diffs, summary) = gather(&f);
    assert_eq!(diffs.len(), 1);
    assert_eq!(diffs[0].files.added, names(&["want-file"]));
    assert_eq!(diffs[0].files.removed, names(&["want-dir"]));
    assert_eq!(diffs[0].dirs.added, names(&["want-dir"]));
    assert_eq!(diffs[0].dirs.removed, names(&["want-file"]));
    assert!(diffs[0].files.changed.is_empty());
    assert_eq!(summary.examined_directories, 1);
}

#[test]
fn roots_must_exist_and_be_real_directories() {
    let f = Fixture::new();
    let absent = f.root.join("absent");
    let error = walk(&absent, &f.target).unwrap_err();
    assert_eq!(error.path(), absent);
    assert!(matches!(error, Error::Io { .. }));
    file(&f.root.join("file"), b"");
    for (a, b) in [(&f.root.join("file"), &f.target), (&f.original, &f.root.join("file"))] {
        let error = walk(a, b).unwrap_err();
        assert!(matches!(error, Error::NotDirectory { .. }));
        assert_eq!(error.path(), f.root.join("file"));
    }
}

#[test]
fn early_stop_never_finishes_or_claims_clean() {
    let f = Fixture::new();
    let summary = walk(&f.original, &f.target).unwrap().into_summary();
    assert_eq!(summary.state, ScanState::Incomplete);
    assert_eq!(summary.examined_directories, 0);
    assert!(!summary.is_clean());
    f.both_dir("pending");
    file(&f.original.join("new"), b"");
    let mut walk = walk(&f.original, &f.target).unwrap();
    assert!(walk.next().unwrap().is_ok());
    fs::remove_dir(f.original.join("pending")).unwrap();
    let summary = walk.into_summary();
    assert_eq!(summary.state, ScanState::Incomplete);
    assert_eq!(summary.examined_directories, 1);
    assert_eq!(summary.changed_dirs, paths(&[""]));
    assert!(!summary.is_complete());
}

#[test]
fn prune_is_immediate_single_use_and_does_not_hide_incomplete_coverage() {
    let f = Fixture::new();
    f.both_dir("a/deep");
    f.both_dir("b");
    file(&f.original.join("a/new"), b"");
    file(&f.original.join("a/deep/skipped"), b"");
    file(&f.original.join("b/new"), b"");
    let mut walk = walk(&f.original, &f.target).unwrap();
    assert!(!walk.prune_here());
    assert_eq!(walk.next().unwrap().unwrap().dir, Path::new("a"));
    assert!(walk.prune_here());
    assert!(!walk.prune_here());
    assert_eq!(walk.next().unwrap().unwrap().dir, Path::new("b"));
    assert!(walk.next().is_none());
    assert!(!walk.prune_here());
    assert!(walk.next().is_none());
    let summary = walk.into_summary();
    assert_eq!(summary.state, ScanState::Complete);
    assert_eq!(summary.pruned, paths(&["a"]));
    assert_eq!(summary.counts.files.added, 2);
    assert_eq!(summary.examined_directories, 3);
    assert!(!summary.is_complete());
    assert!(!summary.is_clean());
}

#[test]
fn prune_of_leaf_is_also_an_explicit_recorded_decision() {
    let f = Fixture::new();
    file(&f.original.join("new"), b"");
    let mut walk = walk(&f.original, &f.target).unwrap();
    walk.next().unwrap().unwrap();
    assert!(walk.prune_here());
    assert!(walk.next().is_none());
    assert_eq!(walk.into_summary().pruned, paths(&[""]));
}

#[test]
fn mid_walk_io_error_is_path_specific_terminal_and_not_counted_as_examined() {
    let f = Fixture::new();
    f.both_dir("pending");
    file(&f.original.join("new"), b"");
    let mut walk = walk(&f.original, &f.target).unwrap();
    walk.next().unwrap().unwrap();
    fs::remove_dir(f.target.join("pending")).unwrap();
    let error = walk.next().unwrap().unwrap_err();
    assert_eq!(error.path(), f.target.join("pending"));
    assert!(error.to_string().contains("pending"));
    assert!(!walk.prune_here());
    assert!(walk.next().is_none());
    assert!(walk.next().is_none());
    let summary = walk.into_summary();
    assert_eq!(summary.state, ScanState::Failed);
    assert_eq!(summary.examined_directories, 1);
    assert_eq!(summary.counts.files.added, 1);
    assert_eq!(summary.changed_dirs, paths(&[""]));
    assert!(!summary.is_complete());
}

#[test]
fn exhaustive_last_record_still_requires_next_none_for_completion() {
    let f = Fixture::new();
    file(&f.original.join("new"), b"");
    let mut walk = walk(&f.original, &f.target).unwrap();
    walk.next().unwrap().unwrap();
    assert_eq!(walk.into_summary().state, ScanState::Incomplete);
}

#[cfg(unix)]
mod unix {
    use super::*;
    use std::os::unix::ffi::OsStringExt;
    use std::os::unix::fs::{PermissionsExt, symlink};
    use std::os::unix::net::UnixListener;

    #[test]
    fn symlink_roots_are_rejected_even_when_they_resolve_to_directories() {
        let f = Fixture::new();
        let link = f.root.join("link");
        symlink(&f.original, &link).unwrap();
        for (a, b) in [(&link, &f.target), (&f.original, &link)] {
            assert!(matches!(walk(a, b), Err(Error::NotDirectory { .. })));
        }
    }

    #[test]
    fn links_are_leaves_including_dangling_cycles_and_literal_spelling() {
        let f = Fixture::new();
        f.both_file("file", b"same");
        for (name, dest) in [("dangling", "missing"), ("cycle", "cycle"), ("parent", ".."), ("dir", ".")] {
            symlink(dest, f.original.join(name)).unwrap();
            symlink(dest, f.target.join(name)).unwrap();
        }
        symlink("./file", f.original.join("spelling")).unwrap();
        symlink("file", f.target.join("spelling")).unwrap();
        symlink("a//b", f.original.join("separators")).unwrap();
        symlink("a/b", f.target.join("separators")).unwrap();
        let (diffs, summary) = gather(&f);
        assert_eq!(diffs[0].files.changed, names(&["separators", "spelling"]));
        assert_eq!(summary.examined_directories, 1);
    }

    #[test]
    fn symlink_mtime_is_not_a_comparison_key() {
        let f = Fixture::new();
        symlink("missing", f.original.join("link")).unwrap();
        std::thread::sleep(Duration::from_millis(2));
        symlink("missing", f.target.join("link")).unwrap();
        assert!(gather(&f).1.is_clean());
    }

    #[test]
    fn every_regular_symlink_directory_transition() {
        let f = Fixture::new();
        for (name, a, b) in [("file-link", 0, 1), ("link-file", 1, 0), ("dir-link", 2, 1), ("link-dir", 1, 2)] {
            for (root, kind) in [(&f.original, a), (&f.target, b)] {
                match kind {
                    0 => file(&root.join(name), b""),
                    1 => symlink("missing", root.join(name)).unwrap(),
                    _ => fs::create_dir(root.join(name)).unwrap(),
                }
            }
        }
        let (diffs, _) = gather(&f);
        assert_eq!(diffs[0].files.changed, names(&["file-link", "link-file"]));
        assert_eq!(diffs[0].files.added, names(&["link-dir"]));
        assert_eq!(diffs[0].files.removed, names(&["dir-link"]));
        assert_eq!(diffs[0].dirs.added, names(&["dir-link"]));
        assert_eq!(diffs[0].dirs.removed, names(&["link-dir"]));
    }

    #[test]
    fn native_names_and_link_targets_stay_lossless_and_byte_sorted() {
        let f = Fixture::new();
        let a = OsString::from_vec(vec![b'x', 0x80]);
        let b = OsString::from_vec(vec![b'x', 0xff]);
        for name in [&b, &a] {
            file(&f.original.join(name), b"");
            f.both_dir(Path::new("nested").join(name));
            file(&f.original.join("nested").join(name).join("new"), b"");
        }
        symlink(Path::new(&a), f.original.join("link")).unwrap();
        symlink(Path::new(&b), f.target.join("link")).unwrap();
        let (diffs, _) = gather(&f);
        assert_eq!(diffs[0].files.added, vec![a.clone(), b.clone()]);
        assert_eq!(diffs[0].files.changed, names(&["link"]));
        assert_eq!(diffs[1].dir, Path::new("nested").join(a));
        assert_eq!(diffs[2].dir, Path::new("nested").join(b));
    }

    #[test]
    fn unsupported_special_files_fail_closed_on_either_side() {
        let f = Fixture::new();
        for root in [&f.original, &f.target] {
            let path = root.join("socket");
            let socket = UnixListener::bind(&path).unwrap();
            let mut walk = walk(&f.original, &f.target).unwrap();
            let error = walk.next().unwrap().unwrap_err();
            assert!(matches!(error, Error::UnsupportedEntry { .. }));
            assert_eq!(error.path(), path);
            assert!(walk.next().is_none());
            let summary = walk.into_summary();
            assert_eq!(summary.state, ScanState::Failed);
            assert_eq!(summary.examined_directories, 0);
            drop(socket);
            fs::remove_file(path).unwrap();
        }
    }

    #[test]
    fn modes_are_outside_the_comparison_contract() {
        let f = Fixture::new();
        f.both_file("file", b"same");
        fs::set_permissions(f.original.join("file"), fs::Permissions::from_mode(0o755)).unwrap();
        fs::set_permissions(f.target.join("file"), fs::Permissions::from_mode(0o644)).unwrap();
        assert!(gather(&f).1.is_clean());
    }

    // This is deliberately a test-side applier, not a library reconciler.
    fn copy_entry(original: &Path, target: &Path) {
        let metadata = fs::symlink_metadata(original).unwrap();
        if metadata.is_symlink() {
            symlink(fs::read_link(original).unwrap(), target).unwrap();
        } else if metadata.is_dir() {
            fs::create_dir(target).unwrap();
            for entry in fs::read_dir(original).unwrap() {
                let entry = entry.unwrap();
                copy_entry(&entry.path(), &target.join(entry.file_name()));
            }
        } else {
            fs::copy(original, target).unwrap();
            set_time(target, metadata.modified().unwrap());
        }
    }

    #[test]
    fn apply_immediate_records_in_loop_then_next_walk_is_clean() {
        let f = Fixture::new();
        f.both_dir("paired/deep");
        f.both_file("paired/deep/changed", b"old");
        file(&f.original.join("paired/deep/changed"), b"new and longer");
        file(&f.original.join("added-file"), b"new");
        file(&f.target.join("removed-file"), b"old");
        fs::create_dir_all(f.original.join("added-dir/sub")).unwrap();
        file(&f.original.join("added-dir/sub/file"), b"subtree");
        fs::create_dir_all(f.target.join("removed-dir/sub")).unwrap();
        file(&f.original.join("want-file"), b"file");
        fs::create_dir(f.target.join("want-file")).unwrap();
        fs::create_dir(f.original.join("want-dir")).unwrap();
        file(&f.target.join("want-dir"), b"file");
        symlink("missing", f.original.join("want-link")).unwrap();
        file(&f.target.join("want-link"), b"file");
        file(&f.original.join("want-regular"), b"file");
        symlink("missing", f.target.join("want-regular")).unwrap();
        let mut walk = walk(&f.original, &f.target).unwrap();
        for diff in walk.by_ref() {
            let diff = diff.unwrap();
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
        assert!(gather(&f).1.is_clean());
    }
}
