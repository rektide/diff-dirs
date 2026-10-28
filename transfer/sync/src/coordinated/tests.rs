use super::*;
use diff_dirs_tree_inspect::{CheckError, Names, Operations};
use std::fs;
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Default)]
struct Rules {
    roots: Arc<AtomicUsize>,
    preserve: Vec<PathBuf>,
    refuse: Option<PathBuf>,
}
impl Visitor for Rules {
    fn early(&self, names: &mut Names) -> Result<(), CheckError> {
        if names.is_root() {
            self.roots.fetch_add(1, Ordering::Relaxed);
        }
        for entry in names.entries_mut() {
            if self.preserve.iter().any(|p| p == entry.relative_path()) {
                entry.preserve_tree();
            }
        }
        Ok(())
    }
    fn late(&self, ops: &mut Operations) -> Result<(), CheckError> {
        for entry in ops.entries() {
            if self.refuse.as_deref() == Some(entry.relative_path()) {
                return Err(entry.refuse("fixture opaque refusal"));
            }
        }
        Ok(())
    }
}

struct Setup {
    _owned: tempfile::TempDir,
    source: PathBuf,
    targets: Vec<PathBuf>,
}
impl Setup {
    fn new() -> Self {
        let scratch = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.test-agent/sync-tests");
        fs::create_dir_all(&scratch).unwrap();
        let owned = tempfile::tempdir_in(fs::canonicalize(scratch).unwrap()).unwrap();
        let source = owned.path().join("source");
        let targets = vec![owned.path().join("a"), owned.path().join("b")];
        for root in std::iter::once(&source).chain(&targets) {
            fs::create_dir(root).unwrap();
        }
        Self {
            _owned: owned,
            source,
            targets,
        }
    }
    fn runtime(&self, rules: Vec<Rules>) -> SyncRuntime<NoCheck, NoScope, RemoveAll, Rules> {
        SyncRuntime::new(
            rules
                .into_iter()
                .enumerate()
                .map(|(i, v)| SyncTarget {
                    id: TargetId(format!("lane-{i}")),
                    descriptor: format!("fixture-{i}/v1"),
                    applier: Applier::new(&self.source, &self.targets[i], CopyPolicy::Copy).unwrap().visitor(v),
                })
                .collect(),
        )
    }
    fn paired(&self, dir: &str) {
        for root in std::iter::once(&self.source).chain(&self.targets) {
            fs::create_dir_all(root.join(dir)).unwrap();
        }
    }
}
fn file(path: impl AsRef<Path>, bytes: &[u8]) {
    let path = path.as_ref();
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, bytes).unwrap();
}
fn finish<C: Check, S: Scope, A: Absence, V: Visitor>(cursor: &mut SyncCursor, runtime: &SyncRuntime<C, S, A, V>) -> Drive {
    cursor
        .drive(
            runtime,
            Limits {
                groups: 100,
                applications: 100,
            },
        )
        .unwrap()
}

#[test]
fn barrier_saves_a_before_b_without_reapplying_or_replaying_root_policy() {
    let f = Setup::new();
    f.paired("d");
    file(f.source.join("root"), b"root");
    file(f.source.join("d/file"), b"child");
    let roots = Arc::new(AtomicUsize::new(0));
    let runtime = f.runtime(
        (0..2)
            .map(|_| Rules {
                roots: roots.clone(),
                ..Rules::default()
            })
            .collect(),
    );
    let mut run = SyncCursor::new(&runtime, Execution::Sequential).unwrap();
    let group = run.next_group(&runtime).unwrap().unwrap();
    assert_eq!(roots.load(Ordering::Relaxed), 2);
    run.apply(&runtime, &group.token, &"lane-0".into()).unwrap();
    assert!(matches!(run.next_group(&runtime), Err(Error::PendingApplications)));
    assert!(f.targets[0].join("root").exists());
    assert!(!f.targets[1].join("root").exists());
    let parts = run.into_parts();
    parts.validate().unwrap();
    assert_eq!(parts.comparison.frames.len(), 1);
    assert_eq!(parts.current.as_ref().unwrap().applications[0].state, ApplicationState::Accepted);
    assert_eq!(parts.current.as_ref().unwrap().applications[1].state, ApplicationState::Unstarted);
    let mut restored = SyncCursor::from_parts(parts, Execution::Sequential).unwrap();
    restored.bind(&runtime).unwrap();
    assert!(matches!(
        restored.apply(&runtime, &group.token, &"lane-1".into()),
        Err(Error::InvalidControl(_))
    ));
    let token = restored.current_token().unwrap();
    restored.apply(&runtime, &token, &"lane-1".into()).unwrap();
    assert_eq!(roots.load(Ordering::Relaxed), 2);
    assert!(finish(&mut restored, &runtime).failures.is_empty());
    restored.parts().validate().unwrap();
    for r in restored.reports() {
        assert!(r.satisfied);
        assert_eq!(r.totals.completed.files_copied, 2);
        assert_eq!(r.totals.applications, 2);
    }
}

#[test]
fn deep_lane_heads_pure_returns_and_control_windows_survive() {
    let f = Setup::new();
    f.paired("a/deep");
    f.paired("b");
    file(f.source.join("a/deep/leaf"), b"A");
    file(f.source.join("b/leaf"), b"both");
    let runtime = f.runtime(vec![
        Rules::default(),
        Rules {
            preserve: vec!["a".into()],
            ..Rules::default()
        },
    ]);
    let mut run = SyncCursor::new(&runtime, Execution::Sequential).unwrap();
    for expected in ["", "a", "a/deep"] {
        let g = run.next_group(&runtime).unwrap().unwrap();
        assert_eq!(g.relative, Path::new(expected));
        for t in run.pending() {
            run.apply(&runtime, &g.token, &t).unwrap();
        }
    }
    let parts = run.into_parts();
    parts.validate().unwrap();
    assert_eq!(parts.comparison.frames.len(), 3);
    let mut run = SyncCursor::from_parts(parts, Execution::Sequential).unwrap();
    assert!(matches!(run.heads()[1].1, compare::Head::Child { ref relative } if relative == Path::new("b")));
    run.bind(&runtime).unwrap();
    let progress = run.drive(&runtime, Limits::default()).unwrap();
    assert_eq!(progress.groups, 0);
    let parts = run.parts();
    parts.validate().unwrap();
    assert_eq!(parts.comparison.frames.len(), 1);
    assert!(parts.current.is_none());
    assert_eq!(parts.comparison.frames[0].remaining[0].name, "b");
    let g = run.next_group(&runtime).unwrap().unwrap();
    assert_eq!(g.relative, Path::new("b"));
    run.prune_here(&g.token, &["lane-0".into()]).unwrap();
    let parts = run.into_parts();
    parts.validate().unwrap();
    let mut run = SyncCursor::from_parts(parts, Execution::Sequential).unwrap();
    run.bind(&runtime).unwrap();
    for t in run.pending() {
        run.apply(&runtime, &run.current_token().unwrap(), &t).unwrap();
    }
    // Before root closure, the accepted current applications and root return are saved.
    let p = run.parts();
    p.validate().unwrap();
    assert_eq!(p.comparison.frames.len(), 2);
    assert!(finish(&mut run, &runtime).failures.is_empty());
    assert!(!run.reports()[0].satisfied);
    assert!(run.reports()[1].satisfied);
    assert!(!f.targets[1].join("a/deep/leaf").exists());
}

#[cfg(unix)]
#[test]
fn kinds_and_excluded_names_are_checked_even_with_unchanged_legacy_diff() {
    use std::os::unix::fs::symlink;
    for change_excluded in [false, true] {
        let f = Setup::new();
        file(f.source.join("x"), b"new payload");
        file(f.targets[0].join("x"), b"old");
        file(f.source.join("hidden-old"), b"excluded");
        let runtime = f.runtime(vec![
            Rules {
                preserve: vec!["hidden-old".into(), "hidden-new".into()],
                ..Rules::default()
            },
            Rules::default(),
        ]);
        let mut run = SyncCursor::new(&runtime, Execution::Sequential).unwrap();
        let g = run.next_group(&runtime).unwrap().unwrap();
        let diff = g.outcomes[0].outcome.as_ref().unwrap().diff.clone();
        let parts = run.into_parts();
        if change_excluded {
            fs::rename(f.source.join("hidden-old"), f.source.join("hidden-new")).unwrap();
        } else {
            fs::remove_file(f.targets[0].join("x")).unwrap();
            symlink("old", f.targets[0].join("x")).unwrap();
        }
        let mut run = SyncCursor::from_parts(parts, Execution::Sequential).unwrap();
        run.bind(&runtime).unwrap();
        let fresh = diff_dirs_tree_inspect::inspect_directory(
            &f.source,
            &f.targets[0],
            Path::new(""),
            Some(&f.source),
            Some(&f.targets[0]),
            diff_dirs_tree_inspect::Purpose::Compare,
            &runtime.targets[0].applier.steering(),
        )
        .unwrap();
        assert_eq!(*fresh.diff(), diff);
        let result = run.apply(&runtime, &run.current_token().unwrap(), &"lane-0".into());
        assert!(matches!(result, Err(Error::Refresh(_))));
        assert_eq!(run.reports()[0].totals.applications, 0);
        assert!(!f.targets[1].join("x").exists()); // all refresh validation preceded any effects
        run.parts().validate().unwrap();
        if change_excluded {
            assert_eq!(fs::read(f.targets[0].join("x")).unwrap(), b"old");
        } else {
            assert!(fs::symlink_metadata(f.targets[0].join("x")).unwrap().is_symlink());
        }
    }
}

#[test]
fn opaque_failure_is_terminal_and_healthy_lane_finishes_without_retrying() {
    let f = Setup::new();
    file(f.source.join("opaque/bad"), b"refused interior");
    f.paired("paired");
    file(f.source.join("paired/leaf"), b"healthy");
    let runtime = f.runtime(vec![
        Rules {
            refuse: Some("opaque/bad".into()),
            ..Rules::default()
        },
        Rules::default(),
    ]);
    let mut run = SyncCursor::new(&runtime, Execution::Sequential).unwrap();
    let progress = finish(&mut run, &runtime);
    assert_eq!(progress.failures.len(), 1);
    match &progress.failures[0] {
        Error::Application { failure, .. } => {
            assert_eq!(failure.completed.directories_created, 1);
            assert_eq!(failure.primary.path, f.targets[0].join("opaque/bad"));
        }
        e => panic!("{e}"),
    }
    assert!(f.targets[0].join("opaque").is_dir());
    assert!(!f.targets[0].join("opaque/bad").exists());
    assert!(f.targets[1].join("opaque/bad").is_file());
    assert!(f.targets[1].join("paired/leaf").is_file());
    let reports = run.reports();
    assert_eq!(reports[0].totals.end, Some(End::ApplicationFailed));
    assert_eq!(reports[0].totals.completed.directories_created, 1);
    assert!(!reports[0].satisfied);
    assert!(reports[1].satisfied);
    run.parts().validate().unwrap();
    let parts = run.into_parts();
    for p in std::iter::once(&f.source).chain(&f.targets) {
        fs::remove_dir_all(p).unwrap();
    }
    let mut restored = SyncCursor::from_parts(parts.clone(), Execution::Sequential).unwrap();
    let empty: SyncRuntime = SyncRuntime::new(Vec::new());
    restored.bind(&empty).unwrap();
    assert_eq!(restored.drive(&empty, Limits::default()).unwrap().stop, Stop::Done);
    assert_eq!(restored.parts(), parts);
}

#[test]
fn zero_work_count_still_reconciles_opaque_kept_target_container() {
    struct KeepContainer;
    impl Visitor for KeepContainer {
        fn late(&self, ops: &mut Operations) -> Result<(), CheckError> {
            for e in ops.entries_mut() {
                if e.relative_path() == Path::new("old") {
                    e.retain();
                }
            }
            Ok(())
        }
    }
    let f = Setup::new();
    file(f.targets[0].join("old/stale"), b"remove");
    let runtime = SyncRuntime::new(vec![SyncTarget {
        id: "a".into(),
        descriptor: "keep-container/v1".into(),
        applier: Applier::new(&f.source, &f.targets[0], CopyPolicy::Copy).unwrap().visitor(KeepContainer),
    }]);
    let mut run = SyncCursor::new(&runtime, Execution::Sequential).unwrap();
    let g = run.next_group(&runtime).unwrap().unwrap();
    assert_eq!(g.outcomes[0].outcome.as_ref().unwrap().coverage.planned, 0);
    run.apply(&runtime, &g.token, &"a".into()).unwrap();
    finish(&mut run, &runtime);
    let r = run.reports().remove(0);
    assert!(r.satisfied);
    assert_eq!(r.totals.completed.files_removed, 1);
    assert_eq!(r.retained, 2);
    assert!(f.targets[0].join("old").is_dir());
    assert!(!f.targets[0].join("old/stale").exists());
    run.parts().validate().unwrap();
}

#[test]
fn stopped_lane_need_not_be_rebound_and_root_preservation_is_a_noop() {
    let f = Setup::new();
    file(f.source.join("file"), b"data");
    let runtime = f.runtime(vec![
        Rules::default(),
        Rules {
            preserve: vec!["".into()],
            ..Rules::default()
        },
    ]);
    let mut run = SyncCursor::new(&runtime, Execution::Sequential).unwrap();
    run.next_group(&runtime).unwrap().unwrap();
    assert_eq!(run.pending(), vec![TargetId::from("lane-0")]);
    run.stop_lanes(&["lane-0".into()]).unwrap();
    let p = run.into_parts();
    p.validate().unwrap();
    fs::remove_dir(&f.targets[0]).unwrap();
    let mut run = SyncCursor::from_parts(p, Execution::Sequential).unwrap();
    let runtime = SyncRuntime::new(runtime.targets.into_iter().filter(|t| t.id == TargetId::from("lane-1")).collect());
    run.bind(&runtime).unwrap();
    finish(&mut run, &runtime);
    run.parts().validate().unwrap();
    assert!(run.reports()[1].satisfied);
    assert_eq!(run.reports()[1].retained, 1);
    assert_eq!(run.reports()[1].totals.applications, 0);
}

#[test]
fn stopped_root_descent_is_also_a_structural_noop_decision() {
    struct StopRoot;
    impl Visitor for StopRoot {
        fn late(&self, ops: &mut Operations) -> Result<(), CheckError> {
            if ops.is_root() {
                for entry in ops.entries_mut() {
                    entry.set_action(diff_dirs_tree_inspect::Action::Directory {
                        container: diff_dirs_tree_inspect::Container::Keep,
                        descend: false,
                    });
                }
            }
            Ok(())
        }
    }
    let f = Setup::new();
    file(f.source.join("file"), b"not traversed");
    let runtime = SyncRuntime::new(vec![SyncTarget {
        id: "A".into(),
        descriptor: "stop-root/v1".into(),
        applier: Applier::new(&f.source, &f.targets[0], CopyPolicy::Copy).unwrap().visitor(StopRoot),
    }]);
    let mut run = SyncCursor::new(&runtime, Execution::Sequential).unwrap();
    run.next_group(&runtime).unwrap().unwrap();
    run.parts().validate().unwrap();
    assert!(run.pending().is_empty());
    run.settle().unwrap();
    assert!(run.reports()[0].satisfied);
    assert_eq!(run.reports()[0].totals.applications, 0);
    assert!(!f.targets[0].join("file").exists());
}

#[test]
fn rejects_overlap_forged_barrier_and_incompatible_runtime_before_effects() {
    let f = Setup::new();
    file(f.source.join("file"), b"data");
    let runtime = f.runtime(vec![Rules::default(), Rules::default()]);
    let mut run = SyncCursor::new(&runtime, Execution::Sequential).unwrap();
    run.next_group(&runtime).unwrap().unwrap();
    let parts = run.parts();
    let mut bad = parts.clone();
    bad.current.as_mut().unwrap().applications.pop();
    assert!(SyncCursor::from_parts(bad, Execution::Sequential).is_err());
    let mut bad = parts.clone();
    bad.version = 99;
    assert!(SyncCursor::from_parts(bad, Execution::Sequential).is_err());
    let mut bad = parts.clone();
    bad.comparison.bindings.targets[1].root = bad.comparison.bindings.targets[0].root.clone();
    assert!(SyncCursor::from_parts(bad, Execution::Sequential).is_err());
    let mut bad = parts.clone();
    bad.current.as_mut().unwrap().applications[0].decision.entries[0].source = Some(Kind::Directory);
    assert!(SyncCursor::from_parts(bad, Execution::Sequential).is_err());
    let mut runtime = runtime;
    runtime.targets[0].descriptor = "changed/v2".into();
    let mut run = SyncCursor::from_parts(parts, Execution::Sequential).unwrap();
    assert!(matches!(run.bind(&runtime), Err(Error::RuntimeMismatch(_))));
    assert!(!f.targets[0].join("file").exists());
    assert!(!f.targets[1].join("file").exists());
    let nested = f.targets[0].join("nested");
    fs::create_dir(&nested).unwrap();
    let runtime = SyncRuntime::new(vec![
        SyncTarget {
            id: "a".into(),
            descriptor: "m/v1".into(),
            applier: Applier::new(&f.source, &f.targets[0], CopyPolicy::Copy).unwrap(),
        },
        SyncTarget {
            id: "b".into(),
            descriptor: "m/v1".into(),
            applier: Applier::new(&f.source, nested, CopyPolicy::Copy).unwrap(),
        },
    ]);
    assert!(SyncCursor::new(&runtime, Execution::Sequential).is_err());
}

#[cfg(unix)]
#[test]
fn nofollow_routes_and_returned_permission_and_staging_scopes() {
    use std::os::unix::fs::{PermissionsExt, symlink};
    let f = Setup::new();
    f.paired("d");
    file(f.source.join("d/file"), b"data");
    fs::set_permissions(&f.targets[0], fs::Permissions::from_mode(0o500)).unwrap();
    file(f.source.join("root-file"), b"root");
    let runtime = f.runtime(vec![Rules::default(), Rules::default()]);
    let mut run = SyncCursor::new(&runtime, Execution::Sequential).unwrap();
    let g = run.next_group(&runtime).unwrap().unwrap();
    for t in run.pending() {
        run.apply(&runtime, &g.token, &t).unwrap();
    }
    assert_eq!(fs::metadata(&f.targets[0]).unwrap().permissions().mode() & 0o777, 0o500);
    assert!(
        fs::read_dir(&f.targets[0])
            .unwrap()
            .all(|e| !e.unwrap().file_name().as_encoded_bytes().starts_with(b".diff-dirs-sync-"))
    );
    let p = run.into_parts();
    fs::set_permissions(&f.targets[0], fs::Permissions::from_mode(0o700)).unwrap();
    let moved = f.targets[0].join("old-d");
    fs::rename(f.targets[0].join("d"), &moved).unwrap();
    symlink(&moved, f.targets[0].join("d")).unwrap();
    let mut run = SyncCursor::from_parts(p, Execution::Sequential).unwrap();
    run.bind(&runtime).unwrap();
    let progress = finish(&mut run, &runtime);
    assert_eq!(progress.failures.len(), 1);
    assert!(!moved.join("file").exists());
    assert!(f.targets[1].join("d/file").exists());
    fs::set_permissions(&f.targets[0], fs::Permissions::from_mode(0o700)).unwrap();
}

#[cfg(feature = "parallel")]
#[test]
fn caller_pool_execution_uses_same_saved_barrier_and_production_kernel() {
    let f = Setup::new();
    file(f.source.join("opaque/file"), b"parallel");
    let runtime = f.runtime(vec![Rules::default(), Rules::default()]);
    let pool = rayon::ThreadPoolBuilder::new().num_threads(2).build().unwrap();
    pool.install(|| {
        let mut run = SyncCursor::new(&runtime, Execution::Parallel).unwrap();
        let first = run.drive(&runtime, Limits { groups: 1, applications: 1 }).unwrap();
        assert_eq!(first.stop, Stop::Budget);
        let mut run = SyncCursor::from_parts(run.into_parts(), Execution::Parallel).unwrap();
        run.bind(&runtime).unwrap();
        assert!(finish(&mut run, &runtime).failures.is_empty());
        for report in run.reports() {
            assert!(report.satisfied);
            assert_eq!(report.totals.completed.files_copied, 1);
        }
    });
}
