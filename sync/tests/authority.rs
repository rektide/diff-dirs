use std::fs::{self, File};
use std::path::{Path, PathBuf};

use diff_dirs::{Check, CheckError, DirDiff, Entry, NoCheck, Scope, Visit};
use diff_dirs_sync::{Absence, Applier, CopyPolicy, NeverRemove, Progress, Report, clear, seed_leaves};

struct Fixture {
    _owned: tempfile::TempDir,
    source: PathBuf,
    target: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let scratch = Path::new(env!("CARGO_MANIFEST_DIR")).join("../.test-agent/sync-tests");
        fs::create_dir_all(&scratch).unwrap();
        let owned = tempfile::tempdir_in(fs::canonicalize(scratch).unwrap()).unwrap();
        let source = owned.path().join("source");
        let target = owned.path().join("target");
        fs::create_dir(&source).unwrap();
        fs::create_dir(&target).unwrap();
        Self { _owned: owned, source, target }
    }

    fn bind(&self) -> Applier {
        Applier::new(&self.source, &self.target, CopyPolicy::Copy).unwrap()
    }
}

fn file(path: impl AsRef<Path>, bytes: &[u8]) {
    let path = path.as_ref();
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, bytes).unwrap();
    File::options()
        .write(true)
        .open(path)
        .unwrap()
        .set_modified(std::time::SystemTime::UNIX_EPOCH)
        .unwrap();
}

fn local(path: &Path) -> Visit {
    if path.file_name().is_some_and(|name| name == "local") {
        Visit::PreserveTree
    } else {
        Visit::Reconcile
    }
}

fn mirrors(path: &Path) -> bool {
    path.components().any(|part| part.as_os_str() == "dist")
}

fn modes() -> &'static [bool] {
    if cfg!(feature = "parallel") { &[false, true] } else { &[false] }
}

fn run<C: Check, S: Scope, A: Absence>(applier: &Applier<C, S, A>, parallel: bool) -> Result<Report, diff_dirs_sync::Failure> {
    #[cfg(feature = "parallel")]
    if parallel {
        let pool = rayon::ThreadPoolBuilder::new().num_threads(2).build().unwrap();
        return applier.run_in_pool(&pool, diff_dirs::ParallelLimits::for_pool(&pool));
    }
    let _ = parallel;
    applier.run()
}

#[test]
fn broad_update_and_nested_absence_authority_keep_real_retained_ancestors() {
    for &parallel in modes() {
        let f = Fixture::new();
        file(f.source.join("new/yes"), b"copied");
        file(f.source.join("new/local/no"), b"excluded source only");
        file(f.source.join("same"), b"new");
        file(f.target.join("same"), b"old longer");
        file(f.target.join("vanished/keep"), b"retained");
        file(f.target.join("vanished/dist/stale"), b"deleted");
        file(f.target.join("vanished/dist/local/secret"), b"preserved");
        file(f.target.join("gone/dist/stale"), b"deleted even below retained parent");
        let applier = f.bind().scope(local).absence(mirrors);
        let report = run(&applier, parallel).unwrap();
        assert!(report.policy.satisfied);
        assert_eq!(report.completed.files_removed, 2);
        assert_eq!(report.completed.files_copied, 2);
        assert!(report.policy.retained > 0);
        assert!(report.policy.excluded > 0);
        assert!(!report.observed.unwrap().is_complete());
        assert_eq!(fs::read(f.target.join("same")).unwrap(), b"new");
        assert_eq!(fs::read(f.target.join("vanished/keep")).unwrap(), b"retained");
        assert_eq!(fs::read(f.target.join("vanished/dist/local/secret")).unwrap(), b"preserved");
        assert!(!f.target.join("new/local").exists());
        assert!(!f.target.join("gone/dist").exists());
        assert!(f.target.join("gone").is_dir()); // its own name is retained
        let before = fs::metadata(&f.target).unwrap().modified().unwrap();
        let repeat = run(&applier, parallel).unwrap();
        assert!(repeat.policy.satisfied);
        assert_eq!(repeat.completed, Progress::default());
        assert!(repeat.observed.unwrap().has_differences()); // retained extras are not equality
        assert_eq!(fs::metadata(&f.target).unwrap().modified().unwrap(), before);
    }
}

#[test]
fn logical_scope_survives_private_directory_staging_and_all_root_dispatches() {
    for &parallel in modes() {
        let f = Fixture::new();
        file(f.source.join("entry/deep/local/secret"), b"excluded");
        file(f.source.join("entry/deep/yes"), b"copied");
        file(f.target.join("entry"), b"old leaf");
        let scope = |path: &Path| {
            if path == Path::new("entry/deep/local") {
                Visit::PreserveTree
            } else {
                Visit::Reconcile
            }
        };
        let report = run(&f.bind().scope(scope).absence(NeverRemove), parallel).unwrap();
        assert_eq!(report.completed.files_copied, 1);
        assert!(!f.target.join("entry/deep/local").exists());
        assert_eq!(fs::read(f.target.join("entry/deep/yes")).unwrap(), b"copied");
        let absent = f.target.with_file_name("absent");
        run(&Applier::new(&f.source, &absent, CopyPolicy::Copy).unwrap().scope(scope), parallel).unwrap();
        assert!(!absent.join("entry/deep/local").exists());
        let root_leaf = f.target.with_file_name("root-leaf");
        file(&root_leaf, b"old root");
        let replacement = Applier::new_replacing_checked(&f.source, &root_leaf, CopyPolicy::Copy, NoCheck)
            .unwrap()
            .scope(scope);
        run(&replacement, parallel).unwrap();
        assert!(!root_leaf.join("entry/deep/local").exists());
        assert_eq!(fs::read(root_leaf.join("entry/deep/yes")).unwrap(), b"copied");
    }
}

#[test]
fn directory_to_leaf_precheck_preserves_entire_old_subtree_and_update_blocks_extras() {
    for &parallel in modes() {
        let f = Fixture::new();
        file(f.source.join("entry"), b"replacement");
        file(f.target.join("entry/a-removable"), b"still intact");
        file(f.target.join("entry/z/local/secret"), b"local");
        let before = fs::metadata(f.target.join("entry")).unwrap().modified().unwrap();
        let failure = run(&f.bind().scope(local), parallel).unwrap_err();
        assert_eq!(failure.primary.path, f.target.join("entry/z/local"));
        assert_eq!(failure.completed, Progress::default());
        assert_eq!(fs::read(f.target.join("entry/a-removable")).unwrap(), b"still intact");
        assert_eq!(fs::metadata(f.target.join("entry")).unwrap().modified().unwrap(), before);
        let failure = run(&f.bind().absence(NeverRemove), parallel).unwrap_err();
        assert_eq!(failure.primary.path, f.target.join("entry/a-removable"));
        assert_eq!(failure.completed, Progress::default());
        run(&f.bind(), parallel).unwrap();
        assert_eq!(fs::read(f.target.join("entry")).unwrap(), b"replacement");
        fs::remove_file(f.target.join("entry")).unwrap();
        fs::create_dir(f.target.join("entry")).unwrap();
        run(&f.bind().absence(NeverRemove), parallel).unwrap(); // empty dir can be replaced
    }
}

#[test]
fn public_records_cannot_reopen_preserved_ancestors_or_basenames() {
    let f = Fixture::new();
    file(f.source.join("local/deep/changed"), b"new");
    file(f.target.join("local/deep/changed"), b"untouched");
    let applier = f.bind().scope(local);
    let record = DirDiff {
        dir: "local/deep".into(),
        files: diff_dirs::ChangedAddedRemoved {
            changed: vec!["changed".into()],
            ..Default::default()
        },
        ..Default::default()
    };
    let report = applier.apply_report(&record).unwrap();
    assert_eq!(report.completed, Progress::default());
    assert_eq!(report.policy.excluded, 1);
    assert_eq!(fs::read(f.target.join("local/deep/changed")).unwrap(), b"untouched");
    // The subtree can differ in type; excluded records require no child stat.
    fs::remove_dir_all(f.source.join("local")).unwrap();
    file(f.source.join("local"), b"different shape");
    let record = DirDiff {
        dirs: diff_dirs::AddedRemoved {
            removed: vec!["local".into()],
            ..Default::default()
        },
        ..Default::default()
    };
    assert_eq!(applier.apply(&record).unwrap(), Progress::default());
    assert!(f.target.join("local/deep/changed").is_file());
    let mut invalid = DirDiff::default();
    invalid.files.added = vec!["local".into(), "missing".into()];
    let failure = applier.apply_report(&invalid).unwrap_err();
    assert_eq!(failure.completed, Progress::default());
    assert_eq!(failure.policy.excluded, 1); // retained even on validation failure
    assert!(!failure.policy.satisfied);
}

#[cfg(unix)]
#[test]
fn retained_only_readonly_regions_do_zero_writes_including_chmod() {
    use std::os::unix::fs::{MetadataExt, PermissionsExt};
    let f = Fixture::new();
    file(f.target.join("extra/local/secret"), b"keep");
    for path in [&f.target, &f.target.join("extra")] {
        fs::set_permissions(path, fs::Permissions::from_mode(0o555)).unwrap();
    }
    let before = [&f.target, &f.target.join("extra")].map(|path| {
        let m = fs::metadata(path).unwrap();
        (m.ino(), m.mtime(), m.mtime_nsec(), m.ctime(), m.ctime_nsec(), m.mode())
    });
    for &parallel in modes() {
        let report = run(&f.bind().scope(local).absence(|_: &Path| false), parallel).unwrap();
        assert_eq!(report.completed, Progress::default());
        assert!(report.policy.satisfied);
        assert_eq!(run(&f.bind().absence(NeverRemove), parallel).unwrap().completed, Progress::default());
    }
    for (path, expected) in [&f.target, &f.target.join("extra")].into_iter().zip(before) {
        let m = fs::metadata(path).unwrap();
        assert_eq!((m.ino(), m.mtime(), m.mtime_nsec(), m.ctime(), m.ctime_nsec(), m.mode()), expected);
        fs::set_permissions(path, fs::Permissions::from_mode(0o700)).unwrap();
    }
}

#[test]
fn clear_preserves_container_and_local_boundaries_using_recursive_remover() {
    let f = Fixture::new();
    file(f.target.join("local/root-secret"), b"keep");
    file(f.target.join("gone/local/deep-secret"), b"keep nested");
    file(f.target.join("gone/delete"), b"remove");
    file(f.target.join("delete"), b"remove");
    #[cfg(unix)]
    let root_inode = {
        use std::os::unix::fs::MetadataExt;
        fs::metadata(&f.target).unwrap().ino()
    };
    let report = clear(&f.target, local, NoCheck).unwrap();
    assert_eq!(report.completed.files_removed, 2);
    assert!(report.policy.satisfied);
    assert!(f.target.is_dir());
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        assert_eq!(fs::metadata(&f.target).unwrap().ino(), root_inode);
    }
    assert_eq!(fs::read(f.target.join("gone/local/deep-secret")).unwrap(), b"keep nested");
    assert_eq!(clear(&f.target, local, NoCheck).unwrap().completed, Progress::default());
}

#[test]
fn scoped_audit_checks_equal_and_one_sided_interiors_without_mutation() {
    let f = Fixture::new();
    file(f.source.join("source-only/local/secret"), b"excluded");
    file(f.source.join("source-only/yes"), b"checked");
    file(f.target.join("target-only/local/secret"), b"excluded");
    file(f.target.join("target-only/yes"), b"checked");
    let check = |entry: Entry<'_>| -> Result<(), CheckError> {
        assert!(!entry.relative_path().components().any(|part| part.as_os_str() == "local"));
        Ok(())
    };
    let applier = Applier::new_checked(&f.source, &f.target, CopyPolicy::Copy, check)
        .unwrap()
        .scope(local)
        .absence(mirrors);
    let report = applier.audit().unwrap();
    assert_eq!(report.original.files, 1);
    assert_eq!(report.target.files, 1);
    assert_eq!(report.original.excluded, 1);
    assert_eq!(report.target.excluded, 1);
    assert!(!f.target.join("source-only").exists());
    assert!(f.target.join("target-only/yes").exists());
}

#[test]
fn missing_source_is_never_an_empty_authoritative_tree_and_root_scope_is_respected() {
    let f = Fixture::new();
    file(f.target.join("sentinel"), b"keep");
    let applier = f.bind().scope(local);
    fs::remove_dir(&f.source).unwrap();
    assert!(applier.run().is_err());
    assert_eq!(fs::read(f.target.join("sentinel")).unwrap(), b"keep");
    fs::create_dir(&f.source).unwrap();
    file(f.source.join("new"), b"new");
    let absent = f.target.with_file_name("absent");
    let preserve_root = |_: &Path| Visit::PreserveTree;
    assert_eq!(
        Applier::new(&f.source, &absent, CopyPolicy::Copy)
            .unwrap()
            .scope(preserve_root)
            .run()
            .unwrap()
            .completed,
        Progress::default()
    );
    assert!(!absent.exists());
    assert_eq!(f.bind().scope(preserve_root).run().unwrap().completed, Progress::default());
    assert!(!f.target.join("new").exists());
    let leaf = f.target.with_file_name("preserved-leaf");
    file(&leaf, b"leave root leaf intact");
    let replacement = Applier::new_replacing_checked(&f.source, &leaf, CopyPolicy::Copy, NoCheck)
        .unwrap()
        .scope(preserve_root);
    assert_eq!(replacement.run().unwrap().completed, Progress::default());
    assert_eq!(fs::read(leaf).unwrap(), b"leave root leaf intact");
}

#[cfg(unix)]
#[test]
fn recursive_copy_removal_and_clear_skip_excluded_specials_before_metadata() {
    let f = Fixture::new();
    fs::create_dir(f.source.join("tree")).unwrap();
    fs::create_dir(f.target.join("old")).unwrap();
    let _source_socket = std::os::unix::net::UnixListener::bind(f.source.join("tree/local")).unwrap();
    let _target_socket = std::os::unix::net::UnixListener::bind(f.target.join("old/local")).unwrap();
    file(f.source.join("tree/yes"), b"copy");
    file(f.target.join("old/gone"), b"remove");
    let guard = |entry: Entry<'_>| -> Result<(), CheckError> {
        assert_ne!(entry.name, "local");
        Ok(())
    };
    let applier = Applier::new_checked(&f.source, &f.target, CopyPolicy::Copy, guard).unwrap().scope(local);
    let report = applier.run().unwrap();
    assert_eq!(report.completed.files_copied, 1);
    assert_eq!(report.completed.files_removed, 1);
    assert!(!f.target.join("tree/local").exists());
    assert!(fs::symlink_metadata(f.target.join("old/local")).is_ok());
    clear(&f.target, local, guard).unwrap();
    assert!(!f.target.join("tree").exists());
    assert!(fs::symlink_metadata(f.target.join("old/local")).is_ok());
}

#[cfg(unix)]
#[test]
fn seed_installs_only_listed_absent_leaves_with_fresh_metadata_literal_links_and_guards() {
    use std::os::unix::fs::{MetadataExt, PermissionsExt, symlink};
    let f = Fixture::new();
    file(f.source.join("listed/deep/file"), b"reflink payload");
    fs::set_permissions(f.source.join("listed/deep/file"), fs::Permissions::from_mode(0o751)).unwrap();
    symlink("../literal/./missing", f.source.join("listed/link")).unwrap();
    let _special = std::os::unix::net::UnixListener::bind(f.source.join("unlisted-special")).unwrap();
    let policy = if std::env::var_os("DIFF_DIRS_SYNC_REQUIRE_REFLINK").is_some() {
        CopyPolicy::ReflinkRequired
    } else {
        CopyPolicy::Copy
    };
    let progress = seed_leaves(&f.source, &f.target, ["listed/deep/file", "listed/link"], policy, NoCheck).unwrap();
    assert_eq!(progress.files_reflinked + progress.files_copied, 1);
    assert_eq!(progress.symlinks_created, 1);
    assert_eq!(progress.directories_created, 2);
    assert!(!f.target.join("unlisted-special").exists());
    assert_eq!(fs::read_link(f.target.join("listed/link")).unwrap(), Path::new("../literal/./missing"));
    let source = fs::metadata(f.source.join("listed/deep/file")).unwrap();
    let target = fs::metadata(f.target.join("listed/deep/file")).unwrap();
    assert_eq!(source.modified().unwrap(), target.modified().unwrap());
    assert_eq!(source.mode() & 0o777, target.mode() & 0o777);
    assert_ne!(source.ino(), target.ino());
    let failure = seed_leaves(&f.source, &f.target, ["listed/deep/file"], policy, NoCheck).unwrap_err();
    assert_eq!(failure.primary.operation, "seed destination exists");
    let failure = seed_leaves(&f.source, &f.target, ["missing"], policy, NoCheck).unwrap_err();
    assert_eq!(failure.primary.operation, "seed source missing");
    let failure = seed_leaves(&f.source, &f.target, ["listed"], policy, NoCheck).unwrap_err();
    assert_eq!(failure.primary.operation, "seed source type");
    let guard = |entry: Entry<'_>| -> Result<(), CheckError> { if entry.name == "link" { Err("refuse listed link".into()) } else { Ok(()) } };
    fs::remove_file(f.target.join("listed/link")).unwrap();
    assert!(seed_leaves(&f.source, &f.target, ["listed/link"], policy, guard).is_err());
    assert!(!f.target.join("listed/link").exists());
    let external = f.target.with_file_name("external");
    fs::create_dir(&external).unwrap();
    symlink(&external, f.target.join("escape")).unwrap();
    file(f.source.join("escape/file"), b"must not escape");
    assert!(seed_leaves(&f.source, &f.target, ["escape/file"], policy, NoCheck).is_err());
    assert!(!external.join("file").exists());
    assert!(seed_leaves(&f.source, &f.target, ["../escape"], policy, NoCheck).is_err());
}

#[cfg(unix)]
#[test]
fn seed_parent_batches_share_verified_prefixes_for_files_links_and_nested_groups() {
    use std::collections::BTreeMap;
    use std::os::unix::fs::symlink;
    use std::sync::Mutex;
    let f = Fixture::new();
    file(f.source.join("p/a-file"), b"first");
    file(f.source.join("p/c-file"), b"third");
    symlink("literal/./missing", f.source.join("p/b-link")).unwrap();
    file(f.source.join("p/deep/nested"), b"nested");
    file(f.source.join("p/other/nested"), b"other nested");
    fs::create_dir_all(f.target.join("p/deep")).unwrap();
    let observed = Mutex::new(BTreeMap::new());
    let guard = |entry: Entry<'_>| -> Result<(), CheckError> {
        if entry.metadata.is_dir() {
            *observed
                .lock()
                .unwrap()
                .entry((entry.side as u8, entry.relative_path().to_path_buf()))
                .or_insert(0) += 1;
        }
        Ok(())
    };
    let policy = if std::env::var_os("DIFF_DIRS_SYNC_REQUIRE_REFLINK").is_some() {
        CopyPolicy::ReflinkRequired
    } else {
        CopyPolicy::Copy
    };
    let progress = seed_leaves(
        &f.source,
        &f.target,
        ["p/other/nested", "p/c-file", "p/deep/nested", "p/b-link", "p/a-file"],
        policy,
        guard,
    )
    .unwrap();
    assert_eq!(progress.files_copied + progress.files_reflinked, 4);
    assert_eq!(progress.symlinks_created, 1);
    assert_eq!(progress.directories_created, 1);
    assert_eq!(fs::read_link(f.target.join("p/b-link")).unwrap(), Path::new("literal/./missing"));
    assert_eq!(fs::read(f.target.join("p/c-file")).unwrap(), b"third");
    assert_eq!(fs::read(f.target.join("p/deep/nested")).unwrap(), b"nested");
    let observed = observed.into_inner().unwrap();
    for (key, count) in &observed {
        assert_eq!(*count, 1, "routing directory re-observed: {key:?}");
    }
    for side in [diff_dirs::Side::Original, diff_dirs::Side::Target] {
        assert_eq!(observed.get(&(side as u8, PathBuf::new())), Some(&1));
        assert_eq!(observed.get(&(side as u8, PathBuf::from("p"))), Some(&1));
    }
    assert!(
        fs::read_dir(f.target.join("p"))
            .unwrap()
            .all(|entry| !entry.unwrap().file_name().as_encoded_bytes().starts_with(b".diff-dirs-sync-"))
    );
}

#[test]
fn seed_sorted_batch_failures_retain_only_actual_prefix_work_and_reject_duplicate_names_upfront() {
    for existing in [false, true] {
        let f = Fixture::new();
        file(f.source.join("root-leaf"), b"earlier parent batch");
        file(f.source.join("p/a"), b"first installed");
        if existing {
            file(f.source.join("p/z"), b"source");
            file(f.target.join("p/z"), b"existing sentinel");
        }
        let failure = seed_leaves(&f.source, &f.target, ["p/z", "root-leaf", "p/a"], CopyPolicy::Copy, NoCheck).unwrap_err();
        assert_eq!(
            failure.primary.operation,
            if existing { "seed destination exists" } else { "seed source missing" }
        );
        assert_eq!(failure.completed.files_copied, 2); // earlier group plus current prefix
        assert_eq!(failure.completed.directories_created, u64::from(!existing));
        assert_eq!(fs::read(f.target.join("p/a")).unwrap(), b"first installed");
        assert_eq!(fs::read(f.target.join("root-leaf")).unwrap(), b"earlier parent batch");
        if existing {
            assert_eq!(fs::read(f.target.join("p/z")).unwrap(), b"existing sentinel");
        } else {
            assert!(!f.target.join("p/z").exists());
        }
        assert_eq!(fs::read_dir(f.target.join("p")).unwrap().count(), if existing { 2 } else { 1 });
    }
    let f = Fixture::new();
    file(f.source.join("p/a"), b"not installed");
    for paths in [["p/a", "p/a"], ["p/a", "p/./a"]] {
        let failure = seed_leaves(&f.source, &f.target, paths, CopyPolicy::Copy, NoCheck).unwrap_err();
        assert_eq!(failure.primary.operation, "seed duplicate path");
        assert_eq!(failure.completed, Progress::default());
        assert!(!f.target.join("p").exists());
    }
    let failure = seed_leaves(&f.source, &f.target, ["p/a", "../invalid"], CopyPolicy::Copy, NoCheck).unwrap_err();
    assert_eq!(failure.primary.operation, "seed relative path");
    assert_eq!(failure.completed, Progress::default());
    assert!(!f.target.join("p").exists());
}
