use std::ffi::OsString;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use diff_dirs::DirDiff;
use diff_dirs_sync::{Applier, CopyPolicy, Progress, Report, mirror};

struct Fixture {
    temporary: tempfile::TempDir,
    source: PathBuf,
    target: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        // Real nonempty reflinks must run on the checkout's filesystem, not
        // accidentally pass as a fallback/empty-file test on a tmpfs /tmp.
        let scratch = Path::new(env!("CARGO_MANIFEST_DIR")).join("../.test-agent/sync-tests");
        fs::create_dir_all(&scratch).unwrap();
        let temporary = tempfile::tempdir_in(scratch).unwrap();
        let source = temporary.path().join("source");
        let target = temporary.path().join("target");
        fs::create_dir(&source).unwrap();
        fs::create_dir(&target).unwrap();
        Self { temporary, source, target }
    }

    fn clean(&self) {
        let mut walk = diff_dirs::walk(&self.source, &self.target).unwrap();
        assert!(walk.next().is_none());
        assert!(walk.into_summary().is_clean());
    }

    fn reflinks_available(&self) -> bool {
        let from = self.temporary.path().join("owned-reflink-source");
        let to = self.temporary.path().join("owned-reflink-destination");
        fs::write(&from, vec![b'x'; 8192]).unwrap();
        let result = reflink_copy::reflink(&from, &to);
        fs::remove_file(from).unwrap();
        if result.is_ok() {
            fs::remove_file(to).unwrap();
        }
        if let Err(error) = result {
            assert!(
                std::env::var_os("DIFF_DIRS_SYNC_REQUIRE_REFLINK").is_none(),
                "real reflink verification required: {error}"
            );
            eprintln!("real reflink branch unavailable on this test filesystem: {error}");
            return false;
        }
        true
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        #[cfg(unix)]
        fn relax(path: &Path) {
            use std::os::unix::fs::PermissionsExt;
            if fs::symlink_metadata(path).is_ok_and(|metadata| metadata.is_dir()) {
                fs::set_permissions(path, fs::Permissions::from_mode(0o700)).unwrap();
                for entry in fs::read_dir(path).unwrap() {
                    relax(&entry.unwrap().path());
                }
            }
        }
        #[cfg(unix)]
        relax(self.temporary.path());
    }
}

fn modes() -> &'static [bool] {
    #[cfg(feature = "parallel")]
    {
        &[false, true]
    }
    #[cfg(not(feature = "parallel"))]
    {
        &[false]
    }
}

fn sync(f: &Fixture, policy: CopyPolicy, parallel: bool) -> Result<Report, diff_dirs_sync::Failure> {
    #[cfg(feature = "parallel")]
    if parallel {
        let pool = rayon::ThreadPoolBuilder::new().num_threads(4).build().unwrap();
        let limits = diff_dirs::ParallelLimits::for_pool(&pool);
        return diff_dirs_sync::mirror_in_pool(&f.source, &f.target, policy, &pool, limits);
    }
    let _ = parallel;
    mirror(&f.source, &f.target, policy)
}

fn file(path: impl AsRef<Path>, content: &[u8]) {
    let path = path.as_ref();
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}

#[test]
fn exact_delta_opaque_subtrees_manual_loop_and_clean_repeat() {
    for &parallel in modes() {
        let f = Fixture::new();
        file(f.source.join("changed"), b"new nonempty");
        file(f.target.join("changed"), b"old");
        file(f.source.join("added"), b"add");
        file(f.target.join("removed"), b"remove");
        file(f.source.join("whole/a/b"), b"whole");
        file(f.target.join("stale/a/b"), b"stale");
        file(f.source.join("paired/nested"), b"nested-new");
        file(f.target.join("paired/nested"), b"old");
        // No repository-specific naming exclusions.
        file(f.source.join(".git/config"), b"ordinary payload");
        file(f.source.join(".jj/store"), b"also payload");
        let report = sync(&f, CopyPolicy::Copy, parallel).unwrap();
        assert_eq!(report.completed.files_copied, 6);
        assert_eq!(report.completed.files_removed, 2);
        assert_eq!(report.completed.directories_removed, 2);
        assert_eq!(report.observed.as_ref().unwrap().counts.dirs.added, 3);
        assert_eq!(report.observed.as_ref().unwrap().counts.files.changed, 2);
        f.clean();
        assert_eq!(sync(&f, CopyPolicy::Copy, parallel).unwrap().completed, Progress::default());

        file(f.source.join("manual"), b"manual caller owned loop");
        let applier = Applier::new(&f.source, &f.target, CopyPolicy::Copy).unwrap();
        let mut walk = diff_dirs::walk(applier.original(), applier.target()).unwrap();
        let mut completed = 0;
        for record in walk.by_ref() {
            completed += applier.apply(&record.unwrap()).unwrap().files_copied;
        }
        assert_eq!(completed, 1);
        assert!(walk.into_summary().is_complete());
        f.clean();
    }
}

#[test]
fn missing_target_initializes_and_rejects_existing_leaf_roots() {
    for &parallel in modes() {
        let f = Fixture::new();
        fs::remove_dir(&f.target).unwrap();
        file(f.source.join("nested/payload"), b"initialize nonempty");
        let report = sync(&f, CopyPolicy::Copy, parallel).unwrap();
        assert!(report.observed.is_none());
        assert_eq!(report.completed.directories_created, 2);
        assert_eq!(report.completed.files_copied, 1);
        f.clean();
        assert!(Applier::new(&f.source, &f.target, CopyPolicy::Copy).unwrap().initialize().is_err());
    }
    let f = Fixture::new();
    file(f.temporary.path().join("leaf"), b"not root");
    assert!(Applier::new(f.temporary.path().join("leaf"), &f.target, CopyPolicy::Copy).is_err());
    assert!(Applier::new(&f.source, f.temporary.path().join("leaf"), CopyPolicy::Copy).is_err());
    assert!(Applier::new(f.temporary.path().join("missing"), &f.target, CopyPolicy::Copy).is_err());
}

#[cfg(unix)]
#[test]
fn all_six_type_transitions_literal_links_and_native_names() {
    use std::os::unix::ffi::OsStringExt;
    use std::os::unix::fs::symlink;
    for &parallel in modes() {
        let f = Fixture::new();
        let external = f.temporary.path().join("external");
        file(external.join("untouched"), b"not synchronized");
        for (source, target, name) in [(0, 1, "fd"), (1, 0, "df"), (0, 2, "fl"), (2, 0, "lf"), (1, 2, "dl"), (2, 1, "ld")] {
            for (root, kind) in [(&f.source, source), (&f.target, target)] {
                let path = root.join(name);
                match kind {
                    0 => file(path, b"file content"),
                    1 => file(path.join("inner"), b"inner content"),
                    2 => symlink(&external, path).unwrap(),
                    _ => unreachable!(),
                }
            }
        }
        let native = OsString::from_vec(b"native-\xff".to_vec());
        file(f.source.join(&native), b"native");
        let literal = OsString::from_vec(b"missing//./../literal-\xff".to_vec());
        symlink(Path::new(&literal), f.source.join("dangling")).unwrap();
        symlink("missing/../different", f.target.join("dangling")).unwrap();
        symlink("fd//./inner", f.source.join("relative")).unwrap();
        symlink(&external, f.target.join("removed-link")).unwrap();
        sync(&f, CopyPolicy::Copy, parallel).unwrap();
        f.clean();
        assert_eq!(fs::read_link(f.target.join("dangling")).unwrap().as_os_str(), literal);
        assert_eq!(fs::read_link(f.target.join("relative")).unwrap().as_os_str(), "fd//./inner");
        assert_eq!(fs::read(f.target.join(native)).unwrap(), b"native");
        assert_eq!(fs::read(external.join("untouched")).unwrap(), b"not synchronized");
        assert_eq!(fs::read_dir(external).unwrap().count(), 1);
        assert_eq!(sync(&f, CopyPolicy::Copy, parallel).unwrap().completed, Progress::default());
    }
}

#[test]
fn invalid_records_are_rejected_before_any_write() {
    let f = Fixture::new();
    file(f.source.join("valid"), b"new");
    let applier = Applier::new(&f.source, &f.target, CopyPolicy::Copy).unwrap();
    for name in ["", ".", "..", "a/b", "/absolute", "tail/"] {
        let mut record = DirDiff::default();
        record.files.added = vec!["valid".into(), name.into()];
        assert!(applier.apply(&record).is_err(), "{name:?}");
        assert_eq!(fs::read_dir(&f.target).unwrap().count(), 0);
    }
    for dir in ["../escape", "/absolute", "a/../../escape", "."] {
        let mut record = DirDiff {
            dir: dir.into(),
            ..DirDiff::default()
        };
        record.files.added.push("valid".into());
        assert!(applier.apply(&record).is_err());
    }
    // Test all category masks, not only the expected happy transition pairs.
    for mask in 1u8..32 {
        if matches!(mask, 1 | 2 | 4 | 8 | 16 | 12 | 18) {
            continue;
        }
        let mut record = DirDiff::default();
        for (bit, names) in [
            (1, &mut record.files.changed),
            (2, &mut record.files.added),
            (4, &mut record.files.removed),
            (8, &mut record.dirs.added),
            (16, &mut record.dirs.removed),
        ] {
            if mask & bit != 0 {
                names.push("valid".into());
            }
        }
        assert!(applier.apply(&record).is_err(), "mask {mask}");
        assert_eq!(fs::read_dir(&f.target).unwrap().count(), 0);
    }
    let mut duplicate = DirDiff::default();
    duplicate.files.added = vec!["valid".into(), "valid".into()];
    assert!(applier.apply(&duplicate).is_err());
    let mut stale = DirDiff::default();
    stale.files.added = vec!["valid".into(), "missing".into()];
    assert!(applier.apply(&stale).is_err());
    assert_eq!(fs::read_dir(&f.target).unwrap().count(), 0);
}

#[test]
fn overlap_existing_and_absent_descendants_rejected_without_creation() {
    let f = Fixture::new();
    for target in [&f.source, &f.source.join("missing/deep"), f.source.parent().unwrap()] {
        assert!(Applier::new(&f.source, target, CopyPolicy::Copy).is_err());
    }
    assert!(!f.source.join("missing").exists());
}

#[cfg(unix)]
#[test]
fn root_links_and_record_ancestor_links_cannot_escape() {
    use std::os::unix::fs::symlink;
    let f = Fixture::new();
    let outside = f.temporary.path().join("outside");
    file(outside.join("payload"), b"untouched");
    symlink(&f.source, f.temporary.path().join("source-link")).unwrap();
    assert!(Applier::new(f.temporary.path().join("source-link"), &f.target, CopyPolicy::Copy).is_err());
    assert!(Applier::new(&f.source, f.temporary.path().join("source-link/missing"), CopyPolicy::Copy).is_err());
    symlink(&outside, f.target.join("escape")).unwrap();
    file(f.source.join("escape/payload"), b"overwrite?");
    let record = DirDiff {
        dir: "escape".into(),
        files: diff_dirs::ChangedAddedRemoved {
            changed: vec!["payload".into()],
            ..Default::default()
        },
        ..Default::default()
    };
    assert!(Applier::new(&f.source, &f.target, CopyPolicy::Copy).unwrap().apply(&record).is_err());
    assert_eq!(fs::read(outside.join("payload")).unwrap(), b"untouched");
    fs::remove_file(f.target.join("escape")).unwrap();
    fs::remove_dir_all(f.source.join("escape")).unwrap();
    symlink(&outside, f.source.join("escape")).unwrap();
    fs::create_dir(f.target.join("escape")).unwrap();
    assert!(Applier::new(&f.source, &f.target, CopyPolicy::Copy).unwrap().apply(&record).is_err());
}

#[cfg(unix)]
#[test]
fn hardlink_alias_is_not_written_or_chmodded_and_noop_changes_no_identity_or_metadata() {
    use std::os::unix::fs::{MetadataExt, PermissionsExt};
    for &parallel in modes() {
        for policy in [CopyPolicy::Copy, CopyPolicy::ReflinkRequired, CopyPolicy::ReflinkOrCopy] {
            let f = Fixture::new();
            if policy == CopyPolicy::ReflinkRequired && !f.reflinks_available() {
                continue;
            }
            file(f.source.join("replace"), b"replacement payload");
            file(f.target.join("replace"), b"original alias");
            fs::set_permissions(f.target.join("replace"), fs::Permissions::from_mode(0o444)).unwrap();
            let alias = f.temporary.path().join("alias");
            fs::hard_link(f.target.join("replace"), &alias).unwrap();
            let before = fs::metadata(&alias).unwrap();
            sync(&f, policy, parallel).unwrap();
            assert_eq!(fs::read(&alias).unwrap(), b"original alias");
            let after = fs::metadata(&alias).unwrap();
            assert_eq!(after.mode(), before.mode());
            assert_eq!(after.modified().unwrap(), before.modified().unwrap());
            assert_ne!(after.ino(), fs::metadata(f.target.join("replace")).unwrap().ino());
            // Include root/directory ctime, so even temporary no-op chmod/staging
            // would be detected, not merely a self-reported zero counter.
            let snapshot = || {
                [f.target.clone(), f.target.join("replace")].map(|path| {
                    let m = fs::symlink_metadata(path).unwrap();
                    (m.ino(), m.mode(), m.mtime(), m.mtime_nsec(), m.ctime(), m.ctime_nsec())
                })
            };
            let before = snapshot();
            assert_eq!(sync(&f, policy, parallel).unwrap().completed, Progress::default());
            assert_eq!(snapshot(), before);
        }
    }
}

#[cfg(unix)]
#[test]
fn readonly_directories_are_restored_files_preserve_mode_and_exact_timestamp() {
    use std::os::unix::fs::PermissionsExt;
    for &parallel in modes() {
        for policy in [CopyPolicy::Copy, CopyPolicy::ReflinkRequired] {
            let f = Fixture::new();
            if policy == CopyPolicy::ReflinkRequired && !f.reflinks_available() {
                continue;
            }
            file(f.source.join("paired/file"), b"new readonly");
            file(f.target.join("paired/file"), b"old");
            file(f.source.join("new/nested/file"), b"created readonly");
            file(f.target.join("remove/nested/file"), b"remove readonly");
            for root in [&f.source, &f.target] {
                fs::set_permissions(root.join("paired"), fs::Permissions::from_mode(0o555)).unwrap();
                fs::set_permissions(root, fs::Permissions::from_mode(0o555)).unwrap();
            }
            for path in [
                f.source.join("new"),
                f.source.join("new/nested"),
                f.target.join("remove"),
                f.target.join("remove/nested"),
            ] {
                fs::set_permissions(path, fs::Permissions::from_mode(0o555)).unwrap();
            }
            for name in ["paired/file", "new/nested/file"] {
                let source = f.source.join(name);
                File::options()
                    .write(true)
                    .open(&source)
                    .unwrap()
                    .set_modified(SystemTime::UNIX_EPOCH - Duration::new(12, 123_456_789))
                    .unwrap();
                fs::set_permissions(source, fs::Permissions::from_mode(0o444)).unwrap();
            }
            let report = sync(&f, policy, parallel).unwrap();
            assert!(report.completed.directory_permissions_set > 0);
            f.clean();
            for name in ["paired/file", "new/nested/file"] {
                let source = fs::metadata(f.source.join(name)).unwrap();
                let target = fs::metadata(f.target.join(name)).unwrap();
                assert_eq!(source.modified().unwrap(), target.modified().unwrap());
                assert_eq!(target.permissions().mode() & 0o7777, 0o444);
            }
            for name in ["", "paired", "new", "new/nested"] {
                assert_eq!(fs::metadata(f.target.join(name)).unwrap().permissions().mode() & 0o7777, 0o555);
                assert_eq!(fs::metadata(f.source.join(name)).unwrap().permissions().mode() & 0o7777, 0o555);
            }
            // Directory modes and mode-only file drift are intentionally not
            // repaired by an extra metadata traversal.
            fs::set_permissions(f.target.join("paired/file"), fs::Permissions::from_mode(0o400)).unwrap();
            assert_eq!(sync(&f, policy, parallel).unwrap().completed, Progress::default());
            assert_eq!(fs::metadata(f.target.join("paired/file")).unwrap().permissions().mode() & 0o7777, 0o400);
        }
    }
}

#[test]
fn real_nonempty_reflink_is_required_and_cow_edits_are_independent() {
    for &parallel in modes() {
        let f = Fixture::new();
        if !f.reflinks_available() {
            continue;
        }
        let payload = vec![b'x'; 16384];
        file(f.source.join("payload"), &payload);
        let report = sync(&f, CopyPolicy::ReflinkRequired, parallel).unwrap();
        assert_eq!(report.completed.files_reflinked, 1);
        assert_eq!(report.completed.bytes_reflinked, payload.len() as u64);
        assert_eq!(report.completed.files_copied, 0);
        f.clean();
        fs::write(f.target.join("payload"), b"target independent").unwrap();
        assert_eq!(fs::read(f.source.join("payload")).unwrap(), payload);
        fs::write(f.source.join("payload"), b"source independent").unwrap();
        assert_eq!(fs::read(f.target.join("payload")).unwrap(), b"target independent");
    }
}

#[cfg(unix)]
#[test]
fn sibling_batches_preserve_collision_names_modes_timestamps_and_source() {
    use std::os::unix::fs::{MetadataExt, PermissionsExt, symlink};
    for &parallel in modes() {
        // Keep the complete source/target/staging pair below 256 entries.
        for count in [1, 8, 64] {
            for policy in [CopyPolicy::Copy, CopyPolicy::ReflinkRequired, CopyPolicy::ReflinkOrCopy] {
                let f = Fixture::new();
                if policy != CopyPolicy::Copy && !f.reflinks_available() {
                    continue;
                }
                // Two nested levels exercise recursive copy, then changed
                // siblings exercise public-record application on the next run.
                let mut snapshot = Vec::new();
                for n in 0..count {
                    let name = format!("opaque/nested/.diff-dirs-sync-{n}-payload");
                    let path = f.source.join(&name);
                    let payload = vec![b'x'; n % 4097]; // empty and unaligned too
                    file(&path, &payload);
                    File::options()
                        .write(true)
                        .open(&path)
                        .unwrap()
                        .set_modified(SystemTime::UNIX_EPOCH - Duration::new(3, 987_654_321))
                        .unwrap();
                    fs::set_permissions(&path, fs::Permissions::from_mode(0o444)).unwrap();
                    let metadata = fs::metadata(&path).unwrap();
                    snapshot.push((name, payload, metadata.ino(), metadata.mode(), metadata.modified().unwrap()));
                }
                symlink("missing//./literal", f.source.join("opaque/nested/.diff-dirs-sync-1001-link")).unwrap();
                file(f.source.join("opaque/nested/.diff-dirs-sync-1002-directory/child"), b"nested directory");
                let first = sync(&f, policy, parallel).unwrap();
                assert_eq!(first.completed.files_copied + first.completed.files_reflinked, count as u64 + 1);
                for (name, _, _, _, _) in &snapshot {
                    fs::remove_file(f.target.join(name)).unwrap();
                    fs::write(f.target.join(name), b"old target inode").unwrap();
                }
                fs::set_permissions(f.target.join("opaque/nested"), fs::Permissions::from_mode(0o555)).unwrap();
                let second = sync(&f, policy, parallel).unwrap();
                assert_eq!(second.completed.files_copied + second.completed.files_reflinked, count as u64);
                assert_eq!(fs::metadata(f.target.join("opaque/nested")).unwrap().mode() & 0o777, 0o555);
                f.clean();
                for (name, payload, inode, mode, modified) in snapshot {
                    let source = fs::metadata(f.source.join(&name)).unwrap();
                    let target = fs::metadata(f.target.join(&name)).unwrap();
                    assert_eq!((source.ino(), source.mode(), source.modified().unwrap()), (inode, mode, modified));
                    assert_eq!((target.mode(), target.modified().unwrap()), (mode, modified));
                    assert_eq!(fs::read(f.source.join(&name)).unwrap(), payload);
                    assert_eq!(fs::read(f.target.join(&name)).unwrap(), payload);
                    assert_ne!(source.ino(), target.ino());
                }
                assert_eq!(sync(&f, policy, parallel).unwrap().completed, Progress::default());
            }
        }
    }
}

#[cfg(target_os = "linux")]
#[test]
fn strict_cross_filesystem_failure_preserves_old_leaf_and_fallback_reports_copy() {
    for &parallel in modes() {
        let f = Fixture::new();
        let other = tempfile::tempdir_in("/tmp/opencode").unwrap();
        file(other.path().join("payload"), b"nonempty cross filesystem source");
        file(f.target.join("payload"), b"old survives");
        let before = fs::read_dir(&f.target).unwrap().count();
        let failure = mirror(other.path(), &f.target, CopyPolicy::ReflinkRequired).unwrap_err();
        assert_eq!(failure.primary.operation, "prepare reflink");
        assert!(failure.primary.path.ends_with("payload"));
        assert_eq!(failure.completed.files_copied, 0);
        assert_eq!(failure.completed.files_reflinked, 0);
        assert_eq!(fs::read(f.target.join("payload")).unwrap(), b"old survives");
        assert_eq!(fs::read_dir(&f.target).unwrap().count(), before);
        let fallback = mirror(other.path(), &f.target, CopyPolicy::ReflinkOrCopy).unwrap();
        assert_eq!(fallback.completed.files_copied, 1);
        assert_eq!(fallback.completed.files_reflinked, 0);
        // Same-filesystem fallback policy truthfully reports a real clone.
        file(f.source.join("local"), b"nonempty local");
        let report = sync(&f, CopyPolicy::ReflinkOrCopy, parallel).unwrap();
        if f.reflinks_available() {
            assert_eq!(report.completed.files_reflinked, 1);
            assert_eq!(report.completed.files_copied, 0);
        } else {
            assert_eq!(report.completed.files_copied, 1);
        }
    }
}

#[cfg(unix)]
#[test]
fn unreadable_source_never_falls_back_or_disturbs_old_hardlinked_leaf() {
    use std::os::unix::fs::{MetadataExt, PermissionsExt};
    for &parallel in modes() {
        for policy in [CopyPolicy::Copy, CopyPolicy::ReflinkRequired, CopyPolicy::ReflinkOrCopy] {
            let f = Fixture::new();
            file(f.source.join("payload"), b"new nonempty data");
            file(f.target.join("payload"), b"old hardlinked leaf");
            let alias = f.temporary.path().join("alias");
            fs::hard_link(f.target.join("payload"), &alias).unwrap();
            let before = fs::metadata(&alias).unwrap();
            fs::set_permissions(f.source.join("payload"), fs::Permissions::from_mode(0o000)).unwrap();
            if File::open(f.source.join("payload")).is_ok() {
                // Root/capability-bearing runs exercise readable mode-000
                // through the real public clone/copy path too.
                sync(&f, policy, parallel).unwrap();
                assert_eq!(fs::metadata(f.target.join("payload")).unwrap().mode() & 0o777, 0);
            } else {
                let failure = sync(&f, policy, parallel).unwrap_err();
                assert_eq!(
                    failure.primary.cause.downcast_ref::<std::io::Error>().unwrap().kind(),
                    std::io::ErrorKind::PermissionDenied
                );
                assert_eq!(failure.completed, Progress::default());
                assert_eq!(fs::read(f.target.join("payload")).unwrap(), b"old hardlinked leaf");
            }
            assert_eq!(fs::read_dir(&f.target).unwrap().count(), 1);
            let after = fs::metadata(&alias).unwrap();
            assert_eq!(
                (after.ino(), after.mode(), after.modified().unwrap()),
                (before.ino(), before.mode(), before.modified().unwrap())
            );
            assert_eq!(fs::read(alias).unwrap(), b"old hardlinked leaf");
            fs::set_permissions(f.source.join("payload"), fs::Permissions::from_mode(0o444)).unwrap();
        }
    }
}

#[cfg(target_os = "linux")]
#[test]
fn failed_directory_transition_preparation_does_not_destroy_old_leaf() {
    let f = Fixture::new();
    let other = tempfile::tempdir_in("/tmp/opencode").unwrap();
    file(other.path().join("transition/nested/payload"), b"cross device");
    file(f.target.join("transition"), b"old leaf");
    let failure = mirror(other.path(), &f.target, CopyPolicy::ReflinkRequired).unwrap_err();
    assert_eq!(failure.completed, Progress::default());
    assert_eq!(fs::read(f.target.join("transition")).unwrap(), b"old leaf");
    assert_eq!(fs::read_dir(&f.target).unwrap().count(), 1);
}

#[cfg(unix)]
#[test]
fn special_in_opaque_subtree_fails_in_place_and_retry_converges() {
    use std::os::unix::net::UnixListener;
    for &parallel in modes() {
        let f = Fixture::new();
        file(f.source.join("opaque/a-good"), b"good");
        file(f.source.join("opaque/z-after"), b"after");
        let socket = UnixListener::bind(f.source.join("opaque/m-socket")).unwrap();
        let failure = sync(&f, CopyPolicy::Copy, parallel).unwrap_err();
        assert!(failure.primary.path.ends_with("opaque/m-socket"));
        assert!(failure.primary.to_string().contains("unsupported special"));
        assert!(failure.completed.directories_created >= 1);
        // The diff saw one opaque added directory, not its copied children.
        assert_eq!(failure.observed.unwrap().counts.dirs.added, 1);
        assert!(
            fs::read_dir(f.target.join("opaque"))
                .unwrap()
                .all(|entry| !entry.unwrap().file_name().as_encoded_bytes().starts_with(b".diff-dirs-sync-"))
        );
        drop(socket);
        fs::remove_file(f.source.join("opaque/m-socket")).unwrap();
        sync(&f, CopyPolicy::Copy, parallel).unwrap();
        f.clean();
        assert_eq!(sync(&f, CopyPolicy::Copy, parallel).unwrap().completed, Progress::default());
    }
}

#[cfg(feature = "parallel")]
#[test]
fn pool_sizes_limits_and_nested_callers_converge() {
    for p in [1, 2, 4] {
        for k in [0, 1, 16] {
            let f = Fixture::new();
            for n in 0..24 {
                file(f.source.join(format!("branch-{n}/nested/payload")), b"payload");
            }
            let pool = rayon::ThreadPoolBuilder::new().num_threads(p).build().unwrap();
            let limits = diff_dirs::ParallelLimits::new(std::num::NonZeroUsize::new(p).unwrap(), k);
            let report = pool
                .install(|| diff_dirs_sync::mirror_in_pool(&f.source, &f.target, CopyPolicy::Copy, &pool, limits))
                .unwrap();
            assert_eq!(report.completed.files_copied, 24);
            f.clean();
        }
    }
}
