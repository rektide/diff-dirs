//! Tiny production consumers of explicit source-only conditional containers.
use diff_dirs_transfer_sync::{
    Applier, CopyPolicy, Progress,
    coordinated::{SyncCursor, SyncRuntime, SyncTarget},
};
use diff_dirs_tree_compare::{Action, CheckError, Container, Execution, Names, Operations, Visitor};
use std::{
    fs,
    path::{Path, PathBuf},
};

struct Fixture {
    _owned: tempfile::TempDir,
    source: PathBuf,
    target: PathBuf,
}
impl Fixture {
    fn new() -> Self {
        let scratch = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.test-agent/conditional-tests");
        fs::create_dir_all(&scratch).unwrap();
        let owned = tempfile::tempdir_in(scratch.canonicalize().unwrap()).unwrap();
        let source = owned.path().join("source");
        let target = owned.path().join("target");
        fs::create_dir(&source).unwrap();
        fs::create_dir(&target).unwrap();
        Self { _owned: owned, source, target }
    }
    fn bind(&self, rules: Routing) -> Applier<diff_dirs_tree_compare::NoCheck, diff_dirs_tree_compare::NoScope, diff_dirs_transfer_sync::RemoveAll, Routing> {
        Applier::new(&self.source, &self.target, CopyPolicy::Copy).unwrap().visitor(rules)
    }
}

fn file(path: impl AsRef<Path>) {
    let path = path.as_ref();
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, b"selected data").unwrap();
}

#[derive(Default)]
struct Routing {
    selected: Vec<PathBuf>,
    eager: Vec<PathBuf>,
    refuse: Option<PathBuf>,
}
impl Routing {
    fn selecting(paths: &[&str]) -> Self {
        Self {
            selected: paths.iter().map(PathBuf::from).collect(),
            ..Self::default()
        }
    }
}
impl Visitor for Routing {
    fn early(&self, names: &mut Names) -> Result<(), CheckError> {
        if names.is_root() {
            return Ok(());
        }
        for name in names.entries_mut() {
            if name.name() == "skip" {
                name.preserve_tree();
            }
        }
        Ok(())
    }
    fn late(&self, batch: &mut Operations) -> Result<(), CheckError> {
        if batch.is_root() {
            return Ok(());
        }
        for e in batch.entries_mut() {
            let p = e.relative_path();
            assert!(
                !p.as_os_str().to_string_lossy().contains(".diff-dirs-sync-"),
                "policy saw physical staging path"
            );
            if self.refuse.as_deref() == Some(p) {
                return Err(e.refuse("conditional fixture refusal"));
            }
            if self.eager.iter().any(|a| a == p) || self.selected.iter().any(|a| p.starts_with(a)) {
                continue;
            }
            if e.source().is_some_and(|f| f.metadata().is_dir()) {
                e.set_action(Action::Directory {
                    container: if e.target().is_none() { Container::CreateIfNeeded } else { Container::Keep },
                    descend: true,
                });
            } else {
                e.preserve_tree();
            }
        }
        Ok(())
    }
}

#[test]
fn no_selected_descendant_makes_no_container_staging_or_permission_effects() {
    let f = Fixture::new();
    file(f.source.join("route/deep/ordinary"));
    file(f.source.join("route/skip/not-inspected"));
    #[cfg(unix)]
    let fingerprint = {
        use std::os::unix::fs::{MetadataExt, PermissionsExt};
        fs::set_permissions(&f.target, fs::Permissions::from_mode(0o555)).unwrap();
        let metadata = fs::metadata(&f.target).unwrap();
        (metadata.ino(), metadata.mode(), metadata.ctime(), metadata.ctime_nsec())
    };
    let before = fs::metadata(&f.target).unwrap().modified().unwrap();
    let binding = f.bind(Routing::default());
    let preview = binding.preview().unwrap();
    assert_eq!(preview.planned, 0);
    assert_eq!(preview.completed, Progress::default());
    let done = binding.run().unwrap();
    assert_eq!(done.planned, 0);
    assert_eq!(done.completed, Progress::default());
    assert_eq!(fs::read_dir(&f.target).unwrap().count(), 0);
    assert_eq!(fs::metadata(&f.target).unwrap().modified().unwrap(), before);
    #[cfg(unix)]
    {
        use std::os::unix::fs::{MetadataExt, PermissionsExt};
        let metadata = fs::metadata(&f.target).unwrap();
        assert_eq!((metadata.ino(), metadata.mode(), metadata.ctime(), metadata.ctime_nsec()), fingerprint);
        fs::set_permissions(&f.target, fs::Permissions::from_mode(0o755)).unwrap();
    }
}

#[test]
fn selected_empty_directory_materializes_only_necessary_ancestry() {
    let f = Fixture::new();
    fs::create_dir_all(f.source.join("outer/inner/empty")).unwrap();
    file(f.source.join("outer/unmatched/deep/file"));
    let binding = f.bind(Routing::selecting(&["outer/inner/empty"]));
    assert_eq!(binding.preview().unwrap().planned, 3);
    assert!(!f.target.join("outer").exists());
    let done = binding.run().unwrap();
    assert_eq!(done.planned, 3);
    assert_eq!(done.completed.directories_created, 3);
    assert_eq!(done.completed.files_copied, 0);
    assert!(f.target.join("outer/inner/empty").is_dir());
    assert!(!f.target.join("outer/unmatched").exists());
}

#[cfg(unix)]
#[test]
fn sibling_leaves_reuse_materialized_chain_and_readonly_modes_finalize_postorder() {
    use std::os::unix::fs::PermissionsExt;
    let f = Fixture::new();
    file(f.source.join("route/deep/a"));
    file(f.source.join("route/deep/b"));
    file(f.source.join("route/deep/skip/excluded"));
    for p in [f.source.join("route"), f.source.join("route/deep"), f.target.clone()] {
        fs::set_permissions(p, fs::Permissions::from_mode(0o555)).unwrap();
    }
    let binding = f.bind(Routing::selecting(&["route/deep/a", "route/deep/b"]));
    let done = binding.run().unwrap();
    assert_eq!(done.planned, 4);
    assert_eq!(done.completed.files_copied, 2);
    assert_eq!(done.completed.directories_created, 2);
    assert_eq!(
        done.completed.directory_permissions_set, 4,
        "two routing finalizers plus external parent relax/restore"
    );
    for root in [&f.source, &f.target] {
        for p in ["route", "route/deep"] {
            assert_eq!(fs::metadata(root.join(p)).unwrap().permissions().mode() & 0o777, 0o555);
        }
    }
    assert!(!f.target.join("route/deep/skip").exists());
    assert_eq!(fs::metadata(&f.target).unwrap().permissions().mode() & 0o777, 0o555);
    // Owned fixture cleanup, not production recovery.
    for root in [&f.source, &f.target] {
        for p in ["", "route", "route/deep"] {
            fs::set_permissions(root.join(p), fs::Permissions::from_mode(0o755)).unwrap();
        }
    }
}

#[test]
fn conditional_nested_copy_under_staged_type_transition_uses_final_logical_paths() {
    let f = Fixture::new();
    file(f.source.join("chosen/routes/deep/take"));
    file(f.source.join("chosen/routes/skip/hidden"));
    fs::write(f.target.join("chosen"), b"old leaf").unwrap();
    let mut rules = Routing::selecting(&["chosen/routes/deep/take"]);
    rules.eager.push("chosen".into());
    let binding = f.bind(rules);
    assert_eq!(binding.preview().unwrap().planned, 4);
    let done = binding.run().unwrap();
    assert_eq!(done.completed.files_copied, 1);
    assert_eq!(done.completed.files_removed, 1);
    assert_eq!(done.completed.directories_created, 3);
    assert!(!f.target.join("chosen/routes/skip").exists());
    assert_eq!(fs::read(f.target.join("chosen/routes/deep/take")).unwrap(), b"selected data");
}

#[test]
fn failed_conditional_preparation_keeps_collision_leaf_and_discards_private_progress() {
    let f = Fixture::new();
    file(f.source.join("chosen/routes/a-good/file"));
    file(f.source.join("chosen/routes/z-bad/refused"));
    fs::write(f.target.join("chosen"), b"old leaf").unwrap();
    let mut rules = Routing::selecting(&["chosen/routes/a-good"]);
    rules.eager.push("chosen".into());
    rules.refuse = Some("chosen/routes/z-bad/refused".into());
    let failure = f.bind(rules).run().unwrap_err();
    assert!(failure.to_string().contains("conditional fixture refusal"));
    assert_eq!(failure.completed, Progress::default(), "private preparation is not installed work");
    assert_eq!(fs::read(f.target.join("chosen")).unwrap(), b"old leaf");
    assert_eq!(fs::read_dir(&f.target).unwrap().count(), 1, "owned staging is cleaned");
}

#[cfg(all(unix, feature = "parallel"))]
#[test]
fn parallel_outer_siblings_keep_independent_pending_chains_and_one_parent_lease() {
    use std::os::unix::fs::PermissionsExt;
    let f = Fixture::new();
    file(f.source.join("a/deep/take"));
    file(f.source.join("b/deep/take"));
    fs::set_permissions(&f.target, fs::Permissions::from_mode(0o555)).unwrap();
    let binding = f.bind(Routing::selecting(&["a/deep/take", "b/deep/take"]));
    let pool = rayon::ThreadPoolBuilder::new().num_threads(2).build().unwrap();
    let done = binding.run_in_pool(&pool, diff_dirs_tree_compare::ParallelLimits::for_pool(&pool)).unwrap();
    assert_eq!(done.completed.files_copied, 2);
    assert_eq!(done.completed.directories_created, 4);
    assert_eq!(done.completed.directory_permissions_set, 6);
    assert_eq!(fs::metadata(&f.target).unwrap().permissions().mode() & 0o777, 0o555);
    fs::set_permissions(&f.target, fs::Permissions::from_mode(0o755)).unwrap();
}

#[cfg(unix)]
#[test]
fn failure_after_visible_effect_retains_counts_and_finalizes_owned_permission_scopes() {
    use std::os::unix::fs::PermissionsExt;
    let f = Fixture::new();
    file(f.source.join("route/a-good/file"));
    file(f.source.join("route/z-bad/refused"));
    fs::set_permissions(f.source.join("route"), fs::Permissions::from_mode(0o555)).unwrap();
    fs::set_permissions(&f.target, fs::Permissions::from_mode(0o555)).unwrap();
    let mut rules = Routing::selecting(&["route/a-good"]);
    rules.refuse = Some("route/z-bad/refused".into());
    let failure = f.bind(rules).run().unwrap_err();
    assert!(failure.to_string().contains("conditional fixture refusal"));
    assert_eq!(failure.completed.files_copied, 1);
    assert_eq!(failure.completed.directories_created, 2);
    assert_eq!(fs::metadata(f.target.join("route")).unwrap().permissions().mode() & 0o777, 0o555);
    assert_eq!(fs::metadata(&f.target).unwrap().permissions().mode() & 0o777, 0o555);
    assert!(!f.target.join("route/z-bad").exists());
    assert_eq!(fs::read_dir(f.target.join("route")).unwrap().count(), 1);
    for p in [&f.target, &f.target.join("route"), &f.source.join("route")] {
        fs::set_permissions(p, fs::Permissions::from_mode(0o755)).unwrap();
    }
}

#[test]
fn illegal_conditional_shapes_and_stopped_descent_refuse_whole_immediate_batch() {
    struct Invalid(bool);
    impl Visitor for Invalid {
        fn late(&self, batch: &mut Operations) -> Result<(), CheckError> {
            for e in batch.entries_mut() {
                if e.name() == "z-bad" {
                    e.set_action(Action::Directory {
                        container: Container::CreateIfNeeded,
                        descend: self.0,
                    });
                }
            }
            Ok(())
        }
    }
    for shape in ["paired", "collision", "leaf", "stopped"] {
        let f = Fixture::new();
        file(f.source.join("a-good"));
        if shape == "leaf" {
            file(f.source.join("z-bad"));
        } else {
            fs::create_dir(f.source.join("z-bad")).unwrap();
        }
        if shape == "paired" {
            fs::create_dir(f.target.join("z-bad")).unwrap();
        }
        if shape == "collision" {
            fs::write(f.target.join("z-bad"), b"local").unwrap();
        }
        let binding = Applier::new(&f.source, &f.target, CopyPolicy::Copy)
            .unwrap()
            .visitor(Invalid(shape != "stopped"));
        assert!(binding.preview().is_err(), "{shape}");
        assert_eq!(binding.run().unwrap_err().completed, Progress::default(), "{shape}");
        assert!(!f.target.join("a-good").exists());
    }
}

#[test]
fn saved_current_group_refresh_validates_conditional_action_without_new_paired_membership() {
    let f = Fixture::new();
    file(f.source.join("route/deep/take"));
    let runtime = SyncRuntime::new(vec![SyncTarget {
        id: "lane".into(),
        descriptor: "conditional-fixture-v1".into(),
        applier: f.bind(Routing::selecting(&["route/deep/take"])),
    }]);
    let mut cursor = SyncCursor::new(&runtime, Execution::Sequential).unwrap();
    let group = cursor.next_group(&runtime).unwrap().unwrap();
    let parts = cursor.into_parts();
    parts.validate().unwrap();
    let decision = &parts.current.as_ref().unwrap().applications[0].decision;
    assert_eq!(
        decision.entries[0].action,
        Action::Directory {
            container: Container::CreateIfNeeded,
            descend: true
        }
    );
    assert_eq!(decision.coverage.planned, 1, "immediate conditional obligation, not recursive actual work");
    let mut invalid = parts.clone();
    invalid.current.as_mut().unwrap().applications[0].decision.entries[0].action = Action::Directory {
        container: Container::CreateIfNeeded,
        descend: false,
    };
    assert!(invalid.validate().is_err());
    let mut cursor = SyncCursor::from_parts(parts, Execution::Sequential).unwrap();
    cursor.bind(&runtime).unwrap();
    assert!(
        cursor.apply(&runtime, &group.token, &"lane".into()).is_err(),
        "old group token is not reusable after restore"
    );
    let token = cursor.current_token().unwrap();
    let done = cursor.apply(&runtime, &token, &"lane".into()).unwrap();
    assert_eq!(done.completed.directories_created, 2);
    assert_eq!(done.completed.files_copied, 1);
    cursor.settle().unwrap();
    assert!(
        cursor.next_group(&runtime).unwrap().is_none(),
        "opaque copied interior must not join paired frontier"
    );
    let report = cursor.reports().pop().unwrap();
    assert!(report.satisfied);
    assert_eq!(report.observed.examined_directories, 1);
}
