//! Deviation, selected retention, necessary containers and returned additions.
use diff_dirs_transfer_sync::coordinated::{ApplicationState, Limits, Stop};
use diff_dirs_transfer_sync::{Applier, CopyPolicy, SyncCursor, SyncRuntime, SyncTarget};
use diff_dirs_tree_inspect::{Action, Choices, Container, DestinationGrant, Execution, InstallAuthority, InstallIntent, Names, Policy, SemanticId, StatKey};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::LazyLock,
};

#[derive(Clone, Default)]
struct Rules {
    renew: bool,
    preserve_root: bool,
    keep_old: bool,
    publication_fault: Option<PathBuf>,
}
impl Policy for Rules {
    type Context = ();
    fn identity(&self) -> &SemanticId {
        static ID: LazyLock<SemanticId> = LazyLock::new(|| SemanticId::new("test.reporting", 1, vec![]));
        &ID
    }
    fn initial_context(&self) {}
    fn scoped(&self) -> bool {
        self.preserve_root || self.keep_old
    }
    fn names(&self, _: &(), names: &mut Names) -> Result<(), diff_dirs_tree_inspect::CheckError> {
        if self.preserve_root && names.is_root() {
            names.entries_mut().next().unwrap().preserve_tree();
        }
        Ok(())
    }
    fn select(&self, _: &(), batch: &mut Choices<'_, ()>) -> Result<(), diff_dirs_tree_inspect::CheckError> {
        let root = batch.info().structural_root;
        for mut e in batch.entries_mut() {
            if e.name() == "protected" {
                e.preserve_tree();
            }
            if self.keep_old && e.name() == "old" {
                e.set_action(Action::Directory {
                    container: Container::Keep,
                    descend: true,
                });
            }
            if self.renew && !root && e.shape().source.kind().is_some_and(|k| k.is_leaf()) {
                e.set_action(Action::InstallLeaf);
                e.set_install(InstallAuthority {
                    intent: InstallIntent::Renew,
                    destination: DestinationGrant::ReplaceLeaf,
                });
            }
        }
        // Deliberate test-only namespace fault AFTER comparison and selection,
        // not an authorized policy behavior or a fallback execution observation.
        if !root && let Some(path) = &self.publication_fault {
            fs::rename(path, path.with_extension("saved")).unwrap();
            fs::create_dir(path).unwrap();
        }
        Ok(())
    }
}
struct Fixture {
    _owned: tempfile::TempDir,
    source: PathBuf,
    targets: Vec<PathBuf>,
}
impl Fixture {
    fn new() -> Self {
        let scratch = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.test-agent/sync-tests");
        fs::create_dir_all(&scratch).unwrap();
        let owned = tempfile::tempdir_in(fs::canonicalize(scratch).unwrap()).unwrap();
        let source = owned.path().join("source");
        let targets = vec![owned.path().join("a"), owned.path().join("b")];
        for path in std::iter::once(&source).chain(&targets) {
            fs::create_dir(path).unwrap();
        }
        Self {
            _owned: owned,
            source,
            targets,
        }
    }
    fn applier(&self, i: usize, rules: Rules) -> Applier<Rules> {
        Applier::new_policy(&self.source, &self.targets[i], CopyPolicy::Copy, rules, StatKey).unwrap()
    }
    fn runtime(&self, rules: Rules) -> SyncRuntime<Rules> {
        SyncRuntime::new(
            (0..2)
                .map(|i| SyncTarget {
                    id: format!("lane-{i}").as_str().into(),
                    descriptor: "reporting/v3".into(),
                    applier: self.applier(i, rules.clone()),
                })
                .collect(),
        )
    }
}
fn file(path: impl AsRef<Path>) {
    let path = path.as_ref();
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, b"actual data").unwrap();
}
fn finish(cursor: &mut SyncCursor, runtime: &SyncRuntime<Rules>) {
    let r = cursor
        .drive(
            runtime,
            Limits {
                groups: 100,
                applications: 100,
            },
        )
        .unwrap();
    assert_eq!(r.stop, Stop::Done);
    assert!(r.failures.is_empty());
    cursor.parts().validate().unwrap();
}

#[test]
fn equal_renewal_success_and_publication_failure_are_deviation_not_retention() {
    for fail in [false, true] {
        let f = Fixture::new();
        file(f.source.join("leaf"));
        file(f.targets[0].join("leaf"));
        fs::File::open(f.targets[0].join("leaf"))
            .unwrap()
            .set_modified(fs::metadata(f.source.join("leaf")).unwrap().modified().unwrap())
            .unwrap();
        let result = f
            .applier(
                0,
                Rules {
                    renew: true,
                    publication_fault: fail.then(|| f.targets[0].join("leaf")),
                    ..Default::default()
                },
            )
            .run();
        if fail {
            let failure = result.unwrap_err();
            assert_eq!(failure.primary.operation, "publish prepared node");
            assert_eq!(failure.policy.deviated, 1);
            assert_eq!(failure.policy.retained, 0);
            assert!(!failure.policy.satisfied);
            assert_eq!(failure.completed.files_copied, 0);
            assert!(failure.observed.as_ref().unwrap().equal == 1);
        } else {
            let report = result.unwrap();
            assert_eq!(report.policy.deviated, 1);
            assert_eq!(report.policy.retained, 0);
            assert_eq!(report.completed.files_copied, 1);
            assert!(report.observed.unwrap().is_clean());
        }
        assert!(
            fs::read_dir(&f.targets[0])
                .unwrap()
                .all(|e| !e.unwrap().file_name().as_encoded_bytes().starts_with(b".diff-dirs-sync-"))
        );
    }
}

#[test]
fn necessary_remove_ancestors_count_once_and_preview_matches_actual_policy_counts() {
    for keep_old in [false, true] {
        let f = Fixture::new();
        file(f.targets[0].join("old/deep/protected"));
        file(f.targets[0].join("old/deep/stale"));
        let applier = f.applier(
            0,
            Rules {
                keep_old,
                ..Default::default()
            },
        );
        let preview = applier.preview().unwrap();
        let actual = applier.run().unwrap();
        assert_eq!(preview.policy.retained, 3, "one preserved leaf and each required ancestor exactly once");
        assert_eq!(actual.policy.retained, 3);
        assert_eq!(actual.policy.deviated, if keep_old { 2 } else { 1 });
        assert_eq!(preview.policy.deviated, actual.policy.deviated);
        assert_eq!(actual.completed.files_removed, 1);
        assert_eq!(actual.completed.directories_removed, 0);
        assert!(f.targets[0].join("old/deep/protected").exists());
    }
}

#[test]
fn accepted_a_pending_b_restores_only_returned_additions_and_rejects_ambiguous_history() {
    let f = Fixture::new();
    for target in &f.targets {
        file(target.join("old/deep/protected"));
        file(target.join("old/deep/stale"));
    }
    let runtime = f.runtime(Rules::default());
    let mut cursor = SyncCursor::new(&runtime, Execution::Sequential).unwrap();
    let group = cursor.next_group(&runtime).unwrap().unwrap();
    cursor.apply(&runtime, &group.token, &"lane-0".into()).unwrap();
    let parts = cursor.into_parts();
    assert_eq!(parts.version, 3);
    assert_eq!(parts.comparison.version, 3);
    assert_eq!(parts.lanes[0].additional_retained, 3);
    assert_eq!(parts.lanes[0].opaque_deviated, 1);
    assert_eq!(parts.lanes[1].additional_retained, 0);
    assert_eq!(parts.current.as_ref().unwrap().applications[1].state, ApplicationState::Unstarted);
    let mut old = parts.clone();
    old.version = 2;
    assert!(SyncCursor::from_parts(old, Execution::Sequential).is_err());
    let mut old = parts.clone();
    old.comparison.version = 2;
    assert!(SyncCursor::from_parts(old, Execution::Sequential).is_err());
    let mut cursor = SyncCursor::from_parts(parts, Execution::Sequential).unwrap();
    cursor.bind(&runtime).unwrap();
    let before = cursor.reports();
    cursor.refresh(&runtime).unwrap();
    assert_eq!(before, cursor.reports());
    let token = cursor.current_token().unwrap();
    cursor.apply(&runtime, &token, &"lane-1".into()).unwrap();
    finish(&mut cursor, &runtime);
    for lane in cursor.reports() {
        assert_eq!(lane.retained, 3);
        assert_eq!(lane.deviated, 1);
        assert_eq!(lane.totals.completed.files_removed, 1);
        assert_eq!(lane.totals.applications, 1);
    }
}

#[test]
fn clean_and_preserved_root_autoaccept_without_applications_and_keep_control_windows() {
    for preserve_root in [false, true] {
        let f = Fixture::new();
        fs::create_dir(f.source.join("paired")).unwrap();
        for target in &f.targets {
            fs::create_dir(target.join("paired")).unwrap();
        }
        let runtime = f.runtime(Rules {
            preserve_root,
            ..Default::default()
        });
        let mut cursor = SyncCursor::new(&runtime, Execution::Sequential).unwrap();
        cursor.next_group(&runtime).unwrap().unwrap();
        assert!(cursor.pending().is_empty());
        assert!(cursor.current_token().is_some());
        assert!(cursor.current().unwrap().applications.iter().all(|a| a.state == ApplicationState::Accepted));
        let parts = cursor.into_parts();
        assert_eq!(parts.lanes[0].applications, 0);
        let mut bad = parts.clone();
        bad.lanes[0].additional_retained = 1;
        assert!(bad.validate().is_err());
        let mut bad = parts.clone();
        bad.current.as_mut().unwrap().applications[0].state = ApplicationState::Unstarted;
        assert!(bad.validate().is_err());
        let mut bad = parts.clone();
        bad.comparison.lanes[0].summary.retained = u64::MAX;
        assert!(bad.validate().is_err());
        let mut cursor = SyncCursor::from_parts(parts, Execution::Sequential).unwrap();
        cursor.bind(&runtime).unwrap();
        finish(&mut cursor, &runtime);
        for lane in cursor.reports() {
            assert_eq!(lane.totals.applications, 0);
            assert_eq!(lane.retained, u128::from(preserve_root));
            assert_eq!(lane.deviated, u128::from(preserve_root));
            assert!(lane.satisfied);
        }
    }
}
