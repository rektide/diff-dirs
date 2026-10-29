use std::fs::{self, File};
use std::path::{Path, PathBuf};

use diff_dirs::{Check, CheckError, Entry, Purpose, Side};
use diff_dirs_sync::{Applier, CopyPolicy, Progress};
use diff_dirs_transfer_sync as diff_dirs_sync;
use diff_dirs_tree_compare as diff_dirs;

struct Fixture {
    _owned: tempfile::TempDir,
    source: PathBuf,
    target: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let scratch = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.test-agent/checked-tests");
        fs::create_dir_all(&scratch).unwrap();
        let owned = tempfile::tempdir_in(scratch).unwrap();
        let root = fs::canonicalize(owned.path()).unwrap();
        let source = root.join("source");
        let target = root.join("target");
        fs::create_dir(&source).unwrap();
        fs::create_dir(&target).unwrap();
        Self { _owned: owned, source, target }
    }

    fn applier(&self) -> Applier<diff_dirs_sync::FullPolicy<impl diff_dirs::Check>> {
        Applier::new_checked(&self.source, &self.target, CopyPolicy::Copy, refuse_admin).unwrap()
    }
}

fn refuse_admin(entry: Entry<'_>) -> Result<(), CheckError> {
    if entry.name == ".git" || entry.name == ".jj" {
        return Err("administrative name".into());
    }
    Ok(())
}

fn file(path: impl AsRef<Path>, content: &[u8]) {
    let path = path.as_ref();
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
    File::options()
        .write(true)
        .open(path)
        .unwrap()
        .set_modified(std::time::UNIX_EPOCH + std::time::Duration::new(12345, 6789))
        .unwrap();
}

fn modes() -> &'static [bool] {
    if cfg!(feature = "parallel") { &[false, true] } else { &[false] }
}

fn mirror<C: diff_dirs::Check>(applier: &Applier<diff_dirs_sync::FullPolicy<C>>, parallel: bool) -> Result<diff_dirs_sync::Report, diff_dirs_sync::Failure> {
    #[cfg(feature = "parallel")]
    if parallel {
        let pool = rayon::ThreadPoolBuilder::new().num_threads(4).build().unwrap();
        return applier.mirror_in_pool(&pool, diff_dirs::ParallelLimits::for_pool(&pool));
    }
    let _ = parallel;
    applier.mirror()
}

#[test]
fn equal_admin_entry_is_refused_even_when_no_diff_would_be_emitted() {
    for parallel in modes() {
        let f = Fixture::new();
        file(f.source.join("equal/.git"), b"same");
        file(f.target.join("equal/.git"), b"same");
        let failure = mirror(&f.applier(), *parallel).unwrap_err();
        assert_eq!(failure.primary.path, f.source.join("equal/.git"));
        assert_eq!(failure.completed, Progress::default());
        assert_eq!(failure.observed.unwrap().state, diff_dirs::ScanState::Failed);
        assert_eq!(fs::read(f.target.join("equal/.git")).unwrap(), b"same");
    }
}

#[test]
fn opaque_copy_and_absent_initialization_check_deep_entries_and_retain_partial_progress() {
    for absent in [false, true] {
        for parallel in modes() {
            let f = Fixture::new();
            file(f.source.join("unit/a-good"), b"installed");
            file(f.source.join("unit/z-deep/nested/.git/untouched"), b"refuse");
            if absent {
                fs::remove_dir(&f.target).unwrap();
            }
            let failure = mirror(&f.applier(), *parallel).unwrap_err();
            assert_eq!(failure.primary.path, f.source.join("unit/z-deep/nested/.git"));
            assert!(!f.target.join("unit/z-deep/nested/.git").exists());
            assert_eq!(fs::read(f.source.join("unit/z-deep/nested/.git/untouched")).unwrap(), b"refuse");
            assert!(failure.completed.directories_created >= 2);
            if !parallel {
                assert_eq!(failure.completed.files_copied, 1);
            }
            if failure.completed.files_copied == 1 {
                assert_eq!(fs::read(f.target.join("unit/a-good")).unwrap(), b"installed");
            }
            assert_eq!(failure.observed.is_none(), absent);
        }
    }
}

#[test]
fn audit_descends_both_one_sided_interiors_and_never_applies() {
    let f = Fixture::new();
    file(f.source.join("source-only/a-good"), b"new");
    file(f.target.join("target-only/deep/.jj/untouched"), b"old");
    let before = fs::metadata(&f.target).unwrap().modified().unwrap();
    let failure = f.applier().audit().unwrap_err();
    assert_eq!(failure.error.path(), f.target.join("target-only/deep/.jj"));
    assert_eq!(failure.inspected.original.files, 1);
    assert_eq!(failure.inspected.original.directories, 2);
    assert_eq!(failure.inspected.target.directories, 3);
    assert!(!f.target.join("source-only").exists());
    assert_eq!(fs::metadata(&f.target).unwrap().modified().unwrap(), before);
    assert_eq!(fs::read(f.target.join("target-only/deep/.jj/untouched")).unwrap(), b"old");
    fs::remove_dir_all(f.target.join("target-only")).unwrap();
    file(f.source.join("source-only/deep/.git"), b"new blocker");
    assert_eq!(f.applier().audit().unwrap_err().error.path(), f.source.join("source-only/deep/.git"));
}

#[test]
fn checked_manual_loop_uses_the_same_policy_for_recursive_operations() {
    let f = Fixture::new();
    file(f.source.join("new/deep/.git"), b"no");
    let applier = f.applier();
    let mut walk = diff_dirs::walk_checked(applier.original(), applier.target(), applier.check().borrowed()).unwrap();
    let diff = walk.next().unwrap().unwrap();
    assert_eq!(applier.apply(&diff).unwrap_err().primary.path, f.source.join("new/deep/.git"));
    assert!(!f.target.join("new/deep/.git").exists());
}

#[test]
fn guarded_leaf_directory_transitions_preserve_old_leaf_on_preparation_refusal() {
    for parallel in modes() {
        let f = Fixture::new();
        file(f.source.join("entry/deep/.git"), b"blocked");
        file(f.target.join("entry"), b"old leaf survives");
        let failure = mirror(&f.applier(), *parallel).unwrap_err();
        assert_eq!(failure.primary.path, f.source.join("entry/deep/.git"));
        assert_eq!(failure.completed, Progress::default()); // private preparation is discarded
        assert_eq!(fs::read(f.target.join("entry")).unwrap(), b"old leaf survives");
        fs::remove_dir_all(f.source.join("entry")).unwrap();
        fs::remove_file(f.target.join("entry")).unwrap();
        file(f.source.join("entry"), b"new leaf");
        file(f.target.join("entry/a-good"), b"removed");
        file(f.target.join("entry/z-deep/.jj"), b"untouched");
        let failure = mirror(&f.applier(), *parallel).unwrap_err();
        assert_eq!(failure.primary.path, f.target.join("entry/z-deep/.jj"));
        assert!(f.target.join("entry").is_dir());
        assert_eq!(failure.completed.files_removed, 0); // local precheck preserves the entire old subtree
        assert_eq!(fs::read(f.target.join("entry/a-good")).unwrap(), b"removed");
        assert_eq!(failure.completed.files_copied, 0);
        assert_eq!(fs::read(f.target.join("entry/z-deep/.jj")).unwrap(), b"untouched");
        assert!(
            !fs::read_dir(&f.target)
                .unwrap()
                .any(|entry| entry.unwrap().file_name().as_encoded_bytes().starts_with(b".diff-dirs-sync-"))
        );
    }
}

#[cfg(unix)]
#[test]
fn checked_recursive_removal_restores_surviving_modes_after_partial_progress() {
    use std::os::unix::fs::{MetadataExt, PermissionsExt};
    for parallel in modes() {
        let f = Fixture::new();
        file(f.target.join("unit/a-good"), b"removed first");
        file(f.target.join("unit/z-deep/nested/.git/untouched"), b"preserved");
        let modes = [(&f.target, 0o555), (&f.target.join("unit"), 0o555), (&f.target.join("unit/z-deep"), 0o500)];
        for (path, mode) in modes {
            fs::set_permissions(path, fs::Permissions::from_mode(mode)).unwrap();
        }
        let failure = mirror(&f.applier(), *parallel).unwrap_err();
        assert_eq!(failure.primary.path, f.target.join("unit/z-deep/nested/.git"));
        assert_eq!(failure.completed.files_removed, 1);
        // Only `unit`, where a child was actually removed, is relaxed/restored.
        assert_eq!(failure.completed.directory_permissions_set, 2);
        assert!(!f.target.join("unit/a-good").exists());
        assert_eq!(fs::read(f.target.join("unit/z-deep/nested/.git/untouched")).unwrap(), b"preserved");
        for (path, mode) in modes {
            assert_eq!(fs::metadata(path).unwrap().mode() & 0o777, mode);
            fs::set_permissions(path, fs::Permissions::from_mode(0o700)).unwrap();
        }
    }
}

#[cfg(unix)]
#[test]
fn root_transition_is_explicit_staged_checked_and_never_follows_destination_link() {
    use std::os::unix::fs::symlink;
    let f = Fixture::new();
    let external = f.source.parent().unwrap().join("external");
    file(external.join("sentinel"), b"untouched");
    fs::remove_dir(&f.target).unwrap();
    symlink(&external, &f.target).unwrap();
    file(f.source.join("deep/.git"), b"refused");
    assert!(Applier::new_checked(&f.source, &f.target, CopyPolicy::Copy, refuse_admin).is_err());
    let applier = Applier::new_replacing_checked(&f.source, &f.target, CopyPolicy::Copy, refuse_admin).unwrap();
    assert_eq!(applier.audit().unwrap_err().error.path(), f.source.join("deep/.git"));
    let failure = applier.mirror().unwrap_err();
    assert_eq!(failure.completed, Progress::default());
    assert_eq!(fs::read_link(&f.target).unwrap(), external);
    fs::remove_file(f.source.join("deep/.git")).unwrap();
    file(f.source.join("deep/good"), b"installed");
    let report = applier.mirror().unwrap();
    assert!(report.observed.is_none());
    assert_eq!(report.completed.symlinks_removed, 1);
    assert_eq!(fs::read(f.target.join("deep/good")).unwrap(), b"installed");
    assert_eq!(fs::read(external.join("sentinel")).unwrap(), b"untouched");
    assert_eq!(applier.mirror().unwrap().completed, Progress::default());
}

#[test]
fn explicit_file_root_transition_and_absent_root_audit_work_with_borrowed_pools() {
    for parallel in modes() {
        let f = Fixture::new();
        file(f.source.join("deep/leaf"), b"prepared directory");
        fs::remove_dir(&f.target).unwrap();
        let absent = f.applier().audit().unwrap();
        assert_eq!(absent.target, diff_dirs::Audit::default());
        assert_eq!(absent.original.files, 1);
        file(&f.target, b"old root file");
        let applier = Applier::new_replacing_checked(&f.source, &f.target, CopyPolicy::Copy, refuse_admin).unwrap();
        let audit = applier.audit().unwrap();
        assert_eq!(audit.target.files, 1);
        assert_eq!(audit.target.directories, 0);
        let report = mirror(&applier, *parallel).unwrap();
        assert_eq!(report.completed.files_removed, 1);
        assert_eq!(report.completed.files_copied, 1);
        assert_eq!(fs::read(f.target.join("deep/leaf")).unwrap(), b"prepared directory");
        assert_eq!(mirror(&applier, *parallel).unwrap().completed, Progress::default());
    }
}

#[cfg(target_os = "linux")]
#[test]
fn checked_real_cross_filesystem_reflink_failure_preserves_old_leaf_without_probe() {
    let f = Fixture::new();
    file(f.source.join("leaf"), b"new actual nonempty source");
    let external = tempfile::tempdir_in("/tmp/opencode").unwrap();
    let target = external.path().join("target");
    fs::create_dir(&target).unwrap();
    file(target.join("leaf"), b"old target survives");
    let applier = Applier::new_checked(&f.source, &target, CopyPolicy::ReflinkRequired, refuse_admin).unwrap();
    let failure = applier.mirror().unwrap_err();
    assert_eq!(failure.primary.operation, "prepare reflink");
    assert_eq!(failure.primary.path, target.join("leaf"));
    assert_eq!(failure.completed, Progress::default());
    assert_eq!(fs::read(target.join("leaf")).unwrap(), b"old target survives");
    assert_eq!(fs::read_dir(&target).unwrap().count(), 1);
}

#[cfg(unix)]
#[test]
fn special_entries_reach_same_check_in_copy_removal_and_audit() {
    use std::os::unix::net::UnixListener;
    for removal in [false, true] {
        let f = Fixture::new();
        let base = if removal { &f.target } else { &f.source };
        fs::create_dir_all(base.join("unit/deep")).unwrap();
        let _socket = UnixListener::bind(base.join("unit/deep/special")).unwrap();
        let check = |entry: Entry<'_>| -> Result<(), CheckError> {
            if !entry.metadata.is_file() && !entry.metadata.is_dir() && !entry.metadata.is_symlink() {
                assert_eq!(entry.side, if removal { Side::Target } else { Side::Original });
                return Err("special refused by caller".into());
            }
            Ok(())
        };
        let applier = Applier::new_checked(&f.source, &f.target, CopyPolicy::Copy, check).unwrap();
        let audit = applier.audit().unwrap_err();
        assert!(matches!(audit.error, diff_dirs::Error::Refused { .. }));
        let failure = applier.mirror().unwrap_err();
        assert_eq!(failure.primary.operation, "check");
        assert_eq!(failure.primary.path, base.join("unit/deep/special"));
        assert!(fs::symlink_metadata(base.join("unit/deep/special")).is_ok());
    }
}

#[cfg(unix)]
#[test]
fn clean_repeat_has_zero_progress_stable_inodes_and_stat_key_limits_are_unchanged() {
    use std::os::unix::fs::{MetadataExt, PermissionsExt, symlink};
    for parallel in modes() {
        let f = Fixture::new();
        file(f.source.join("unit/leaf"), b"original");
        symlink("/absolute/../dangling", f.source.join("unit/link")).unwrap();
        let check = |entry: Entry<'_>| -> Result<(), CheckError> {
            refuse_admin(entry)?;
            if entry.side == Side::Original && entry.metadata.is_symlink() {
                assert_eq!(entry.link_target, Some(Path::new("/absolute/../dangling")));
                assert!(matches!(entry.purpose, Purpose::Compare | Purpose::Copy | Purpose::Audit));
            }
            Ok(())
        };
        let applier = Applier::new_checked(&f.source, &f.target, CopyPolicy::Copy, check).unwrap();
        mirror(&applier, *parallel).unwrap();
        let leaf = f.target.join("unit/leaf");
        let before = fs::metadata(&leaf).unwrap();
        let directory = fs::metadata(f.target.join("unit")).unwrap();
        let report = mirror(&applier, *parallel).unwrap();
        assert_eq!(report.completed, Progress::default());
        assert!(report.observed.unwrap().is_clean());
        let after = fs::metadata(&leaf).unwrap();
        assert_eq!(
            (after.ino(), after.ctime(), after.ctime_nsec()),
            (before.ino(), before.ctime(), before.ctime_nsec())
        );
        assert_eq!(fs::metadata(f.target.join("unit")).unwrap().modified().unwrap(), directory.modified().unwrap());
        fs::write(&leaf, b"modified").unwrap(); // same length, restore comparison mtime
        fs::set_permissions(&leaf, fs::Permissions::from_mode(0o600)).unwrap();
        File::options()
            .write(true)
            .open(&leaf)
            .unwrap()
            .set_modified(before.modified().unwrap())
            .unwrap();
        let report = mirror(&applier, *parallel).unwrap();
        assert_eq!(report.completed, Progress::default());
        assert_eq!(fs::read(&leaf).unwrap(), b"modified");
        assert_eq!(fs::metadata(&leaf).unwrap().mode() & 0o777, 0o600);
    }
}
