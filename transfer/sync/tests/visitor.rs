use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use diff_dirs::{Action, CheckError, Container, Names, NoCheck, NoVisitor, Operations, Presence, Visitor};
use diff_dirs_sync::{Applier, CopyPolicy, Progress, Seed};
use diff_dirs_transfer_sync as diff_dirs_sync;
use diff_dirs_tree_compare as diff_dirs;

struct Fixture {
    _owned: tempfile::TempDir,
    source: PathBuf,
    target: PathBuf,
}
impl Fixture {
    fn new() -> Self {
        let scratch = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.test-agent/sync-tests");
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
fn run<V: Visitor>(
    binding: &Applier<NoCheck, diff_dirs::NoScope, diff_dirs_sync::RemoveAll, V>,
    parallel: bool,
) -> Result<diff_dirs_sync::Report, diff_dirs_sync::Failure> {
    #[cfg(feature = "parallel")]
    if parallel {
        let pool = rayon::ThreadPoolBuilder::new().num_threads(2).build().unwrap();
        return binding.run_in_pool(&pool, diff_dirs::ParallelLimits::for_pool(&pool));
    }
    let _ = parallel;
    binding.run()
}

struct ProtectedSeed;
impl Visitor for ProtectedSeed {
    fn early(&self, names: &mut Names) -> Result<(), CheckError> {
        for name in names.entries_mut() {
            if name.name() == ".jj" {
                name.preserve_tree();
            }
        }
        Ok(())
    }
    fn late(&self, operations: &mut Operations) -> Result<(), CheckError> {
        Seed.late(operations)
    }
}

#[test]
fn tree_seed_admits_creates_independently_of_collisions_updates_and_deletion() {
    for &parallel in modes() {
        let f = Fixture::new();
        file(f.source.join("new/sub/file"), b"new");
        file(f.source.join("existing"), b"donor longer");
        file(f.target.join("existing"), b"local");
        file(f.source.join("dir-collision/file"), b"donor");
        file(f.target.join("dir-collision"), b"keep leaf");
        file(f.source.join("leaf-collision"), b"donor");
        file(f.target.join("leaf-collision/keep"), b"keep directory");
        file(f.target.join("extra/deep"), b"retain extras");
        #[cfg(unix)]
        let _special = std::os::unix::net::UnixListener::bind(f.source.join("new/.jj")).unwrap();
        let binding = f.bind().visitor(ProtectedSeed);
        let before = fs::metadata(&f.target).unwrap().modified().unwrap();
        let preview = binding.preview().unwrap();
        assert_eq!(preview.completed, Progress::default());
        assert_eq!(preview.planned, 3);
        assert_eq!(fs::metadata(&f.target).unwrap().modified().unwrap(), before);
        assert!(!f.target.join("new").exists());
        let report = run(&binding, parallel).unwrap();
        assert_eq!(report.planned, preview.planned);
        assert_eq!(report.completed.files_copied, 1);
        assert_eq!(report.completed.directories_created, 2);
        assert_eq!(fs::read(f.target.join("existing")).unwrap(), b"local");
        assert_eq!(fs::read(f.target.join("dir-collision")).unwrap(), b"keep leaf");
        assert_eq!(fs::read(f.target.join("leaf-collision/keep")).unwrap(), b"keep directory");
        assert!(f.target.join("extra/deep").exists());
        assert!(!f.target.join("new/.jj").exists());
        let observed = report.observed.unwrap();
        assert!(observed.has_differences());
        assert!(!observed.is_clean());
        assert_eq!(run(&binding, parallel).unwrap().completed, Progress::default());
    }
}

struct Mixed;
impl Visitor for Mixed {
    fn late(&self, batch: &mut Operations) -> Result<(), CheckError> {
        for entry in batch.entries_mut() {
            if entry.name() == "deny-create" || entry.name() == "whole" {
                entry.preserve_tree();
            }
            if entry.name() == "keep-existing" || entry.name() == "old" {
                entry.retain();
            }
            if entry.name() == "empty-create" {
                entry.set_action(Action::Directory {
                    container: Container::Create,
                    descend: false,
                });
            }
            assert!(!entry.relative_path().starts_with("whole/never"));
        }
        Ok(())
    }
}

#[test]
fn keeping_container_does_not_stop_descendant_work_but_preserve_tree_does() {
    for &parallel in modes() {
        let f = Fixture::new();
        file(f.source.join("deny-create"), b"not installed");
        file(f.source.join("yes-create"), b"installed");
        file(f.source.join("keep-existing"), b"new larger");
        file(f.target.join("keep-existing"), b"old");
        file(f.target.join("old/stale"), b"prune beneath kept container");
        file(f.target.join("whole/never"), b"preserved without child batch");
        file(f.source.join("paired/changed"), b"new larger");
        file(f.target.join("paired/changed"), b"old");
        file(f.source.join("empty-create/omitted"), b"legal empty container");
        let binding = f.bind().visitor(Mixed);
        let preview = binding.preview().unwrap();
        let report = run(&binding, parallel).unwrap();
        assert_eq!(report.planned, preview.planned);
        assert_eq!(report.completed.files_copied, 2);
        assert_eq!(report.completed.files_removed, 1);
        assert_eq!(report.completed.directories_removed, 0);
        assert!(f.target.join("old").is_dir());
        assert!(!f.target.join("old/stale").exists());
        assert!(f.target.join("whole/never").exists());
        assert!(!f.target.join("deny-create").exists());
        assert!(f.target.join("empty-create").is_dir());
        assert!(!f.target.join("empty-create/omitted").exists());
        assert_eq!(fs::read(f.target.join("keep-existing")).unwrap(), b"old");
    }
}

struct InvalidCollision;
impl Visitor for InvalidCollision {
    fn late(&self, batch: &mut Operations) -> Result<(), CheckError> {
        for entry in batch.entries_mut() {
            if entry.name() == "z-collision" {
                entry.set_action(Action::RemoveLeaf);
            }
        }
        Ok(())
    }
}

#[test]
fn coupled_collision_cannot_be_split_and_invalid_batch_makes_zero_writes() {
    for &parallel in modes() {
        let f = Fixture::new();
        file(f.source.join("a-allowed"), b"must not leak from invalid batch");
        file(f.source.join("z-collision/child"), b"donor");
        file(f.target.join("z-collision"), b"old leaf");
        let before = fs::metadata(&f.target).unwrap().modified().unwrap();
        let binding = f.bind().visitor(InvalidCollision);
        assert!(binding.preview().is_err());
        let failure = run(&binding, parallel).unwrap_err();
        assert_eq!(failure.completed, Progress::default());
        assert_eq!(failure.planned, 0);
        assert!(!f.target.join("a-allowed").exists());
        assert_eq!(fs::read(f.target.join("z-collision")).unwrap(), b"old leaf");
        assert_eq!(fs::metadata(&f.target).unwrap().modified().unwrap(), before);
    }
}

#[test]
fn refusing_a_source_only_container_cannot_leave_child_creation_admitted() {
    struct InvalidParent;
    impl Visitor for InvalidParent {
        fn late(&self, batch: &mut Operations) -> Result<(), CheckError> {
            for entry in batch.entries_mut() {
                if entry.name() == "parent" {
                    entry.set_action(Action::Directory {
                        container: Container::Keep,
                        descend: true,
                    });
                }
            }
            Ok(())
        }
    }
    let f = Fixture::new();
    file(f.source.join("parent/child"), b"depends on a real container");
    let failure = f.bind().visitor(InvalidParent).run().unwrap_err();
    assert_eq!(failure.completed, Progress::default());
    assert!(!f.target.join("parent").exists());
}

#[test]
fn pruning_presets_do_not_reinterpret_collisions_as_source_absence() {
    let f = Fixture::new();
    file(f.source.join("collision"), b"source leaf");
    file(f.target.join("collision/keep"), b"not source absence");
    file(f.source.join("new/child"), b"not created by prune");
    file(f.target.join("paired/stale"), b"target only");
    fs::create_dir(f.source.join("paired")).unwrap();
    let report = f.bind().visitor(diff_dirs_sync::Prune).run().unwrap();
    assert_eq!(report.completed.files_removed, 1);
    assert_eq!(report.completed.directories_removed, 0);
    assert_eq!(fs::read(f.target.join("collision/keep")).unwrap(), b"not source absence");
    assert!(!f.target.join("new").exists());
    assert!(f.bind().visitor(diff_dirs_sync::Update).run().is_err());
}

#[cfg(unix)]
#[test]
fn late_retention_keeps_existing_inodes_modes_and_times_without_parent_chmod() {
    use std::os::unix::fs::{MetadataExt, PermissionsExt};
    struct RetainChildren;
    impl Visitor for RetainChildren {
        fn late(&self, batch: &mut Operations) -> Result<(), CheckError> {
            if !batch.is_root() {
                for entry in batch.entries_mut() {
                    entry.preserve_tree();
                }
            }
            Ok(())
        }
    }
    let f = Fixture::new();
    file(f.source.join("leaf"), b"changed donor");
    file(f.target.join("leaf"), b"retained");
    fs::set_permissions(&f.target, fs::Permissions::from_mode(0o555)).unwrap();
    let fingerprint = |p: &Path| {
        let m = fs::symlink_metadata(p).unwrap();
        (m.ino(), m.mode(), m.mtime(), m.mtime_nsec(), m.ctime(), m.ctime_nsec())
    };
    let root = fingerprint(&f.target);
    let leaf = fingerprint(&f.target.join("leaf"));
    let binding = f.bind().visitor(RetainChildren);
    let report = binding.run().unwrap();
    assert_eq!(report.planned, 0);
    assert_eq!(report.completed, Progress::default());
    assert_eq!(fingerprint(&f.target), root);
    assert_eq!(fingerprint(&f.target.join("leaf")), leaf);
    assert_eq!(fs::read_dir(&f.target).unwrap().count(), 1);
    fs::set_permissions(&f.target, fs::Permissions::from_mode(0o755)).unwrap();
}

#[cfg(unix)]
struct PreserveSpecials<'a>(&'a Mutex<Vec<(PathBuf, Presence)>>);
#[cfg(unix)]
impl Visitor for PreserveSpecials<'_> {
    fn late(&self, batch: &mut Operations) -> Result<(), CheckError> {
        for entry in batch.entries_mut() {
            assert!(!entry.relative_path().starts_with("e-special-dir") || entry.relative_path() == Path::new("e-special-dir"));
            if [entry.source(), entry.target()].into_iter().flatten().any(|facts| {
                let m = facts.metadata();
                !m.is_dir() && !m.is_file() && !m.is_symlink()
            }) {
                self.0.lock().unwrap().push((entry.relative_path().to_path_buf(), entry.presence()));
                assert_eq!(entry.comparison(), None);
                assert_eq!(entry.baseline(), None);
                assert_eq!(entry.action(), None);
                entry.preserve_tree();
            }
        }
        Ok(())
    }
}

#[cfg(unix)]
#[test]
fn late_special_preservation_leaves_socket_pairs_and_peers_untouched() {
    use std::os::unix::fs::{FileTypeExt, MetadataExt};
    use std::os::unix::net::UnixListener;
    struct NoOp;
    impl Visitor for NoOp {}
    struct RemoveSpecial;
    impl Visitor for RemoveSpecial {
        fn late(&self, batch: &mut Operations) -> Result<(), CheckError> {
            for entry in batch.entries_mut() {
                if entry.comparison().is_none() {
                    entry.set_action(Action::RemoveLeaf);
                }
            }
            Ok(())
        }
    }
    let fingerprint = |p: &Path| {
        let m = fs::symlink_metadata(p).unwrap();
        (m.ino(), m.mode(), m.mtime(), m.mtime_nsec(), m.ctime(), m.ctime_nsec())
    };
    for &parallel in modes() {
        let f = Fixture::new();
        let _sockets = [
            UnixListener::bind(f.source.join("a-source")).unwrap(),
            UnixListener::bind(f.target.join("b-target")).unwrap(),
            UnixListener::bind(f.source.join("c-special-file")).unwrap(),
            UnixListener::bind(f.target.join("d-file-special")).unwrap(),
            UnixListener::bind(f.source.join("e-special-dir")).unwrap(),
        ];
        file(f.target.join("c-special-file"), b"keep peer file");
        file(f.source.join("d-file-special"), b"do not install over socket");
        file(f.target.join("e-special-dir/never"), b"keep peer subtree");
        let paths = [
            &f.source,
            &f.target,
            &f.target.join("b-target"),
            &f.target.join("c-special-file"),
            &f.target.join("d-file-special"),
            &f.target.join("e-special-dir"),
            &f.target.join("e-special-dir/never"),
        ];
        let before: Vec<_> = paths.iter().map(|p| fingerprint(p)).collect();
        let seen = Mutex::new(Vec::new());
        let binding = f.bind().visitor(PreserveSpecials(&seen));
        let preview = binding.preview().unwrap();
        assert_eq!(preview.planned, 0);
        assert_eq!(preview.completed, Progress::default());
        assert_eq!(preview.policy.retained, 5);
        seen.lock().unwrap().clear();
        let report = run(&binding, parallel).unwrap();
        assert_eq!(report.planned, 0);
        assert_eq!(report.completed, Progress::default());
        assert_eq!(report.policy.retained, 5);
        let observed = report.observed.unwrap();
        assert_eq!(observed.counts, diff_dirs::EntryCounts::default());
        assert_eq!(observed.deviated, 5);
        assert_eq!(observed.planned, 0);
        assert!(!observed.is_clean());
        assert!(!observed.is_complete());
        assert_eq!(seen.lock().unwrap().len(), 5);
        assert_eq!(paths.iter().map(|p| fingerprint(p)).collect::<Vec<_>>(), before);
        assert!(fs::symlink_metadata(f.source.join("a-source")).unwrap().file_type().is_socket());
        assert!(fs::symlink_metadata(f.target.join("b-target")).unwrap().file_type().is_socket());
        assert_eq!(fs::read(f.target.join("c-special-file")).unwrap(), b"keep peer file");
        assert_eq!(fs::read(f.target.join("e-special-dir/never")).unwrap(), b"keep peer subtree");
        for failure in [
            f.bind().run().unwrap_err(),
            run(&f.bind().visitor(NoOp), parallel).unwrap_err(),
            run(&f.bind().visitor(RemoveSpecial), parallel).unwrap_err(),
        ] {
            assert_eq!(failure.primary.path, f.source.join("a-source"));
            assert_eq!(failure.completed, Progress::default());
        }
        // A normal yielded diff must not count its special retention once in
        // application and again when merging committed comparison coverage.
        file(f.source.join("z-copy"), b"ordinary admitted sibling");
        let mixed = run(&binding, parallel).unwrap();
        assert_eq!(mixed.completed.files_copied, 1);
        assert_eq!(mixed.planned, 1);
        assert_eq!(mixed.policy.retained, 5);
        assert_eq!(mixed.observed.unwrap().deviated, 5);
        assert!(fs::symlink_metadata(f.target.join("b-target")).unwrap().file_type().is_socket());
    }
}

#[cfg(unix)]
#[test]
fn late_special_preservation_blocks_coupled_directory_replacement_intact() {
    use std::os::unix::fs::MetadataExt;
    let f = Fixture::new();
    file(f.source.join("collision"), b"source leaf must not replace retained subtree");
    file(f.target.join("collision/a-deletable"), b"intact sibling");
    let socket = f.target.join("collision/b-socket");
    let _socket = std::os::unix::net::UnixListener::bind(&socket).unwrap();
    let before = fs::metadata(f.target.join("collision")).unwrap().modified().unwrap();
    let inode = fs::symlink_metadata(&socket).unwrap().ino();
    let seen = Mutex::new(Vec::new());
    let failure = f.bind().visitor(PreserveSpecials(&seen)).run().unwrap_err();
    assert_eq!(failure.primary.path, socket);
    assert_eq!(failure.completed, Progress::default());
    assert_eq!(fs::read(f.target.join("collision/a-deletable")).unwrap(), b"intact sibling");
    assert_eq!(fs::symlink_metadata(&socket).unwrap().ino(), inode);
    assert_eq!(fs::metadata(f.target.join("collision")).unwrap().modified().unwrap(), before);
}

#[cfg(target_os = "linux")]
#[test]
fn late_special_preservation_never_opens_source_or_target_fifos() {
    use std::os::unix::fs::{FileTypeExt, MetadataExt};
    let f = Fixture::new();
    let source = f.source.join("source-fifo");
    let target = f.target.join("target-fifo");
    for path in [&source, &target] {
        rustix::fs::mkfifoat(rustix::fs::CWD, path, rustix::fs::Mode::RUSR | rustix::fs::Mode::WUSR).unwrap();
    }
    let before = (fs::symlink_metadata(&source).unwrap().ino(), fs::symlink_metadata(&target).unwrap().ino());
    let seen = Mutex::new(Vec::new());
    let report = f.bind().visitor(PreserveSpecials(&seen)).run().unwrap();
    assert_eq!(report.completed, Progress::default());
    assert_eq!(report.planned, 0);
    assert_eq!(report.policy.retained, 2);
    assert_eq!(seen.lock().unwrap().len(), 2);
    for path in [&source, &target] {
        assert!(fs::symlink_metadata(path).unwrap().file_type().is_fifo());
    }
    assert_eq!(
        (fs::symlink_metadata(&source).unwrap().ino(), fs::symlink_metadata(&target).unwrap().ino()),
        before
    );
    let audited = f.bind().visitor(PreserveSpecials(&seen)).audit().unwrap();
    assert_eq!(audited.original.retained_specials, 1);
    assert_eq!(audited.target.retained_specials, 1);
    let cleared = diff_dirs_sync::clear_with_visitor(&f.target, PreserveSpecials(&seen)).unwrap();
    assert_eq!(cleared.completed, Progress::default());
    assert_eq!(cleared.planned, 0);
    assert_eq!(cleared.policy.retained, 2); // FIFO and the selected container
    assert!(f.bind().run().is_err());
    assert!(fs::symlink_metadata(&target).unwrap().file_type().is_fifo());
}

struct RetainBlocker;
impl Visitor for RetainBlocker {
    fn late(&self, batch: &mut Operations) -> Result<(), CheckError> {
        for entry in batch.entries_mut() {
            if entry.name() == "z-blocker" {
                entry.retain();
            }
        }
        Ok(())
    }
}

#[test]
fn directory_to_leaf_blocker_preserves_the_entire_affected_subtree() {
    let f = Fixture::new();
    file(f.source.join("collision"), b"replacement leaf");
    file(f.target.join("collision/a-deletable"), b"must survive precheck");
    file(f.target.join("collision/z-blocker"), b"retained");
    let before = fs::metadata(f.target.join("collision")).unwrap().modified().unwrap();
    let binding = f.bind().visitor(RetainBlocker);
    assert!(binding.preview().is_err());
    let failure = binding.run().unwrap_err();
    assert_eq!(failure.primary.path, f.target.join("collision/z-blocker"));
    assert_eq!(failure.completed, Progress::default());
    assert_eq!(fs::read(f.target.join("collision/a-deletable")).unwrap(), b"must survive precheck");
    assert_eq!(fs::metadata(f.target.join("collision")).unwrap().modified().unwrap(), before);
}

struct Logical<'a> {
    target: &'a Path,
    seen: &'a Mutex<Vec<PathBuf>>,
    refuse: bool,
}
impl Visitor for Logical<'_> {
    fn early(&self, names: &mut Names) -> Result<(), CheckError> {
        for name in names.entries_mut() {
            if name.name() == ".jj" {
                name.preserve_tree();
            }
        }
        Ok(())
    }
    fn late(&self, batch: &mut Operations) -> Result<(), CheckError> {
        assert_eq!(batch.target_root(), self.target);
        for entry in batch.entries() {
            assert!(!entry.relative_path().to_string_lossy().contains(".diff-dirs-sync-"));
            self.seen.lock().unwrap().push(entry.relative_path().to_path_buf());
            if self.refuse && entry.name() == "deny" {
                return Err("refuse private preparation at logical final path".into());
            }
        }
        Ok(())
    }
}

#[cfg(unix)]
#[test]
fn recursive_visitors_use_final_paths_for_initialization_and_staged_leaf_transitions() {
    use std::os::unix::fs::symlink;
    for existing_leaf in [false, true] {
        for refuse in [false, true] {
            let f = Fixture::new();
            fs::remove_dir(&f.target).unwrap();
            file(f.source.join("deep/deny"), b"donor");
            let _special = std::os::unix::net::UnixListener::bind(f.source.join("deep/.jj")).unwrap();
            let external = f.source.with_file_name("external");
            file(external.join("sentinel"), b"never followed");
            if existing_leaf {
                symlink(&external, &f.target).unwrap();
            }
            let seen = Mutex::new(Vec::new());
            let binding = Applier::new_replacing_checked(&f.source, &f.target, CopyPolicy::Copy, NoCheck)
                .unwrap()
                .visitor(Logical {
                    target: &f.target,
                    seen: &seen,
                    refuse,
                });
            let preview = binding.preview();
            assert_eq!(preview.is_err(), refuse);
            let result = binding.run();
            assert_eq!(result.is_err(), refuse);
            assert!(seen.lock().unwrap().iter().any(|p| p == Path::new("deep/deny")));
            if refuse && existing_leaf {
                assert!(fs::symlink_metadata(&f.target).unwrap().is_symlink());
                assert_eq!(result.unwrap_err().completed, Progress::default());
            } else if !refuse {
                assert_eq!(fs::read(f.target.join("deep/deny")).unwrap(), b"donor");
                assert!(!f.target.join("deep/.jj").exists());
            }
            assert_eq!(fs::read(external.join("sentinel")).unwrap(), b"never followed");
        }
    }
}

struct KeepAll;
impl Visitor for KeepAll {
    fn late(&self, batch: &mut Operations) -> Result<(), CheckError> {
        for entry in batch.entries_mut() {
            entry.preserve_tree();
        }
        Ok(())
    }
}

#[test]
fn retained_absent_or_leaf_root_has_no_staging_namespace_or_permission_writes() {
    let f = Fixture::new();
    file(f.source.join("payload"), b"donor");
    fs::remove_dir(&f.target).unwrap();
    let parent = f.target.parent().unwrap();
    let before = fs::metadata(parent).unwrap().modified().unwrap();
    let binding = f.bind().visitor(KeepAll);
    let report = binding.run().unwrap();
    assert_eq!(report.completed, Progress::default());
    assert_eq!(report.planned, 0);
    assert!(!f.target.exists());
    assert_eq!(fs::metadata(parent).unwrap().modified().unwrap(), before);
    file(&f.target, b"selected root leaf");
    let before = fs::metadata(&f.target).unwrap().modified().unwrap();
    let binding = Applier::new_replacing_checked(&f.source, &f.target, CopyPolicy::Copy, NoCheck)
        .unwrap()
        .visitor(Seed);
    assert_eq!(binding.run().unwrap().completed, Progress::default());
    assert_eq!(fs::read(&f.target).unwrap(), b"selected root leaf");
    assert_eq!(fs::metadata(&f.target).unwrap().modified().unwrap(), before);
}

#[test]
fn public_plan_cannot_override_binding_actions_or_install_supplied_metadata() {
    let f = Fixture::new();
    file(f.source.join("new"), b"real bound source");
    let plan = diff_dirs::inspect_directory(
        &f.source,
        &f.target,
        Path::new(""),
        Some(&f.source),
        Some(&f.target),
        diff_dirs::Purpose::Compare,
        &NoVisitor,
    )
    .unwrap();
    let binding = f.bind().visitor(KeepAll);
    assert!(binding.apply_plan(&plan).is_err());
    assert!(!f.target.join("new").exists());
    let fake_source = f.source.with_file_name("other-source");
    file(fake_source.join("new"), b"not trusted");
    let fabricated = diff_dirs::inspect_directory(
        &f.source,
        &f.target,
        Path::new(""),
        Some(&fake_source),
        Some(&f.target),
        diff_dirs::Purpose::Compare,
        &NoVisitor,
    )
    .unwrap();
    assert_ne!(
        fabricated.operations().entries().get(0).unwrap().source().unwrap().metadata().len(),
        fs::metadata(f.source.join("new")).unwrap().len()
    );
    f.bind().apply_plan(&fabricated).unwrap();
    assert_eq!(fs::read(f.target.join("new")).unwrap(), b"real bound source");
    assert!(
        diff_dirs::inspect_directory(
            &f.source,
            &f.target,
            Path::new("../escape"),
            Some(&f.source),
            Some(&f.target),
            diff_dirs::Purpose::Compare,
            &NoVisitor
        )
        .is_err()
    );
}

#[cfg(unix)]
#[test]
fn public_plan_ancestor_links_are_not_admission_certificates() {
    use std::os::unix::fs::symlink;
    let f = Fixture::new();
    file(f.source.join("escape/leaf"), b"donor");
    let outside = f.source.with_file_name("outside");
    file(outside.join("leaf"), b"must survive");
    symlink(&outside, f.target.join("escape")).unwrap();
    let plan = diff_dirs::inspect_directory(
        &f.source,
        &f.target,
        Path::new("escape"),
        Some(&f.source.join("escape")),
        Some(&outside),
        diff_dirs::Purpose::Compare,
        &NoVisitor,
    )
    .unwrap();
    assert!(f.bind().apply_plan(&plan).is_err());
    assert_eq!(fs::read(outside.join("leaf")).unwrap(), b"must survive");
}

#[test]
fn clear_and_audit_use_the_same_target_only_steering_and_retained_ancestors() {
    struct Clear;
    impl Visitor for Clear {
        fn early(&self, batch: &mut Names) -> Result<(), CheckError> {
            for name in batch.entries_mut() {
                if name.name() == ".jj" {
                    name.preserve_tree();
                }
            }
            Ok(())
        }
        fn late(&self, batch: &mut Operations) -> Result<(), CheckError> {
            for entry in batch.entries_mut() {
                assert_eq!(entry.presence(), Presence::TargetOnly);
                if entry.name() == "kept-container" {
                    entry.retain();
                }
            }
            Ok(())
        }
    }
    let f = Fixture::new();
    file(f.target.join("kept-container/gone"), b"delete");
    file(f.target.join(".jj/hidden"), b"retain");
    let audited = diff_dirs::audit_with_visitor(&f.target, diff_dirs::Side::Target, &Clear).unwrap();
    assert_eq!(audited.files, 1);
    assert_eq!(audited.excluded, 1);
    let report = diff_dirs_sync::clear_with_visitor(&f.target, Clear).unwrap();
    assert_eq!(report.completed.files_removed, 1);
    assert_eq!(report.completed.directories_removed, 0);
    assert!(f.target.join("kept-container").is_dir());
    assert!(f.target.join(".jj/hidden").exists());
}
