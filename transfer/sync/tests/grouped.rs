//! Exact source-work evidence for actual joined opaque execution.
use diff_dirs_transfer_sync::coordinated::{ApplicationState, Error, Limits, Stop};
use diff_dirs_transfer_sync::{Applier, CopyPolicy, SyncCursor, SyncRuntime, SyncTarget};
use diff_dirs_tree_inspect::{
    Action, Choices, Container, ContextSite, Execution, Names, ObservationResources, Policy, SemanticId, ShapeChoices, SourceCacheLimits, SourceSession,
    StatKey,
};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::{Arc, LazyLock},
};

#[derive(Clone, Default)]
struct Rules {
    context: u8,
    hide: Option<PathBuf>,
    conditional: bool,
    refuse: Option<PathBuf>,
    fact_fault: Option<PathBuf>,
    preserve_link: bool,
}
impl Policy for Rules {
    type Context = u8;
    fn identity(&self) -> &SemanticId {
        static ID: LazyLock<SemanticId> = LazyLock::new(|| SemanticId::new("test.grouped", 1, vec![]));
        &ID
    }
    fn initial_context(&self) -> u8 {
        self.context
    }
    fn validate_context(&self, _: ContextSite<'_>, context: &u8) -> Result<(), diff_dirs_tree_inspect::CheckError> {
        if *context != self.context {
            return Err("lane context leaked".into());
        }
        Ok(())
    }
    fn names(&self, _: &u8, names: &mut Names) -> Result<(), diff_dirs_tree_inspect::CheckError> {
        for e in names.entries_mut() {
            if self.hide.as_deref() == Some(e.relative_path()) {
                e.preserve_tree();
            }
        }
        Ok(())
    }
    fn shape(&self, _: &u8, batch: &mut ShapeChoices<'_, u8>) -> Result<(), diff_dirs_tree_inspect::CheckError> {
        if self.conditional && !batch.info().structural_root {
            for mut e in batch.entries_mut() {
                if e.shape().source.kind() == Some(diff_dirs_tree_inspect::Kind::Directory) && e.shape().target.is_none() && e.name() != "empty" {
                    e.set_action(Action::Directory {
                        container: Container::CreateIfNeeded,
                        descend: true,
                    });
                }
            }
        }
        if self.preserve_link && batch.info().relative == Path::new("q") {
            for mut entry in batch.entries_mut() {
                entry.omit_comparison();
                entry.set_action(Action::PreserveTree);
                if self.fact_fault.is_some() {
                    entry.request(diff_dirs_tree_inspect::SideDemands {
                        source: diff_dirs_tree_inspect::Demand {
                            attributes: false,
                            literal_link: true,
                        },
                        target: diff_dirs_tree_inspect::Demand::NONE,
                    });
                }
            }
            // Test-only loss between kind resolution and an optional stronger
            // fact request. Production policies remain pure.
            if let Some(path) = &self.fact_fault {
                fs::remove_file(path).unwrap();
            }
        }
        Ok(())
    }
    fn select(&self, _: &u8, batch: &mut Choices<'_, u8>) -> Result<(), diff_dirs_tree_inspect::CheckError> {
        if batch
            .entries_mut()
            .any(|e| self.refuse.as_deref() == Some(e.coordinate().to_path_buf().as_path()))
        {
            return Err("original lane refusal".into());
        }
        Ok(())
    }
}

#[cfg(unix)]
#[test]
fn a_failed_stronger_source_fact_does_not_poison_the_kind_only_group_peer() {
    let f = Fixture::new();
    fs::create_dir(f.source.join("q")).unwrap();
    std::os::unix::fs::symlink("literal/missing", f.source.join("q/link")).unwrap();
    let runtime = f.runtime([
        Rules {
            context: 1,
            preserve_link: true,
            fact_fault: Some(f.source.join("q/link")),
            ..Default::default()
        },
        Rules {
            context: 2,
            preserve_link: true,
            ..Default::default()
        },
    ]);
    let mut run = f.cursor(
        &runtime,
        SourceCacheLimits {
            batches: 0,
            charged_capacity: 0,
        },
        Execution::Sequential,
    );
    let group = run.next_group(&runtime).unwrap().unwrap();
    let report = run.apply_many(&runtime, &group.token, &run.pending()).unwrap();
    let failure = report.outcomes[0].outcome.as_ref().unwrap_err();
    assert_eq!(failure.primary.path, f.source.join("q/link"));
    assert!(failure.to_string().contains("No such file"));
    let healthy = report.outcomes[1].outcome.as_ref().unwrap();
    assert_eq!(healthy.completed.files_copied, 0);
    assert_eq!(healthy.completed.symlinks_created, 0);
    assert_eq!(healthy.policy.retained, 1);
    let diagnostics = run.source_session().unwrap().diagnostics();
    assert_eq!(diagnostics.opaque.listings, 1);
    assert_eq!(diagnostics.opaque.link_acquisitions, 1);
    assert_eq!(diagnostics.active_batches, 0);
    for target in &f.targets {
        no_staging(target);
    }
    run.parts().validate().unwrap();
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
        for p in std::iter::once(&source).chain(&targets) {
            fs::create_dir(p).unwrap();
        }
        Self {
            _owned: owned,
            source,
            targets,
        }
    }
    fn runtime(&self, rules: [Rules; 2]) -> SyncRuntime<Rules> {
        SyncRuntime::new(
            rules
                .into_iter()
                .enumerate()
                .map(|(i, rule)| SyncTarget {
                    id: format!("lane-{i}").as_str().into(),
                    descriptor: format!("grouped/{i}"),
                    applier: Applier::new_policy(&self.source, &self.targets[i], CopyPolicy::Copy, rule, StatKey).unwrap(),
                })
                .collect(),
        )
    }
    fn cursor(&self, runtime: &SyncRuntime<Rules>, limits: SourceCacheLimits, execution: Execution) -> SyncCursor<u8> {
        let mut run = SyncCursor::new(runtime, execution).unwrap();
        run.bind_with_source(
            runtime,
            Arc::new(SourceSession::with_cache_limits(&self.source, ObservationResources::default(), limits)),
        )
        .unwrap();
        run
    }
}
fn file(path: impl AsRef<Path>) {
    let path = path.as_ref();
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, b"actual donor data").unwrap();
}
fn no_staging(path: &Path) {
    for entry in fs::read_dir(path).unwrap() {
        let e = entry.unwrap();
        assert!(!e.file_name().as_encoded_bytes().starts_with(b".diff-dirs-sync-"));
        if e.file_type().unwrap().is_dir() {
            no_staging(&e.path());
        }
    }
}
fn finish(run: &mut SyncCursor<u8>, runtime: &SyncRuntime<Rules>) {
    let drive = run
        .drive(
            runtime,
            Limits {
                groups: 200,
                applications: 400,
            },
        )
        .unwrap();
    assert_eq!(drive.stop, Stop::Done);
    assert!(drive.failures.is_empty(), "{:?}", drive.failures);
    run.parts().validate().unwrap();
}

#[test]
fn joint_both_missing_and_private_expansion_are_source_once_with_zero_retention() {
    for replacement in [false, true] {
        let f = Fixture::new();
        for i in 0..40 {
            file(f.source.join(format!("q/d{i:02}/nested/leaf")));
        }
        file(f.source.join("q/protected/leaf"));
        if replacement {
            fs::write(f.targets[1].join("q"), b"old receiver leaf").unwrap();
        }
        let runtime = f.runtime([
            Rules {
                context: 3,
                hide: Some("q/protected".into()),
                ..Default::default()
            },
            Rules {
                context: 9,
                ..Default::default()
            },
        ]);
        let mut run = f.cursor(
            &runtime,
            SourceCacheLimits {
                batches: 0,
                charged_capacity: 0,
            },
            Execution::Sequential,
        );
        let group = run.next_group(&runtime).unwrap().unwrap();
        let report = run.apply_many(&runtime, &group.token, &["lane-1".into(), "lane-0".into()]).unwrap();
        assert_eq!(report.outcomes.iter().map(|o| o.target.0.as_str()).collect::<Vec<_>>(), ["lane-0", "lane-1"]);
        assert!(report.outcomes.iter().all(|o| o.outcome.is_ok()), "{:?}", report.outcomes);
        assert_eq!(report.outcomes[0].outcome.as_ref().unwrap().completed.files_copied, 40);
        assert_eq!(report.outcomes[1].outcome.as_ref().unwrap().completed.files_copied, 41);
        assert!(!f.targets[0].join("q/protected").exists());
        assert!(f.targets[1].join("q/protected/leaf").exists());
        let diagnostics = run.source_session().unwrap().diagnostics();
        assert_eq!(diagnostics.paired.listings, 1);
        assert_eq!(diagnostics.paired.metadata_acquisitions, 1);
        assert_eq!(
            diagnostics.opaque.listings, 82,
            "q + 40*(d,nested) + protected; never lane-A's entire tree then B's"
        );
        assert_eq!(
            diagnostics.opaque.metadata_acquisitions, 122,
            "entry attributes acquired once across both lanes, not once per receiver"
        );
        assert_eq!(diagnostics.retained_batches, 0);
        assert_eq!(diagnostics.retained_charged_capacity, 0);
        assert_eq!(diagnostics.active_batches, 0);
        assert_eq!(
            run.current().unwrap().applications.iter().map(|a| a.state).collect::<Vec<_>>(),
            [ApplicationState::Accepted; 2]
        );
        assert!(
            run.parts().comparison.frames.last().unwrap().remaining.is_empty(),
            "creation/expansion cannot add paired edges"
        );
        finish(&mut run, &runtime);
        for target in &f.targets {
            no_staging(target);
        }
    }
}

#[test]
fn conditional_chains_are_independent_while_child_inspection_is_joined() {
    let f = Fixture::new();
    file(f.source.join("q/only-hidden/x"));
    file(f.source.join("q/needed/deep/value"));
    fs::create_dir_all(f.source.join("q/explicit/empty")).unwrap();
    let runtime = f.runtime([
        Rules {
            context: 4,
            conditional: true,
            hide: Some("q/only-hidden/x".into()),
            ..Default::default()
        },
        Rules {
            context: 5,
            conditional: true,
            hide: Some("q/needed/deep/value".into()),
            ..Default::default()
        },
    ]);
    let mut run = f.cursor(
        &runtime,
        SourceCacheLimits {
            batches: 0,
            charged_capacity: 0,
        },
        Execution::Sequential,
    );
    let group = run.next_group(&runtime).unwrap().unwrap();
    let report = run.apply_many(&runtime, &group.token, &run.pending()).unwrap();
    assert!(report.outcomes.iter().all(|o| o.outcome.is_ok()), "{:?}", report.outcomes);
    assert!(f.targets[0].join("q/needed/deep/value").exists());
    assert!(!f.targets[0].join("q/only-hidden").exists());
    assert!(f.targets[1].join("q/only-hidden/x").exists());
    assert!(!f.targets[1].join("q/needed").exists());
    for target in &f.targets {
        assert!(target.join("q/explicit/empty").is_dir());
        no_staging(target);
    }
    assert_eq!(run.source_session().unwrap().diagnostics().opaque.listings, 6);
    finish(&mut run, &runtime);
}

#[test]
fn mixed_opaque_paired_reuses_only_retained_observations_and_eviction_preserves_results() {
    for batches in [32, 1, 0] {
        let f = Fixture::new();
        file(f.source.join("q/a/deep/value"));
        file(f.source.join("q/b/value"));
        fs::create_dir_all(f.targets[1].join("q/a/deep")).unwrap();
        fs::create_dir_all(f.targets[1].join("q/b")).unwrap();
        let runtime = f.runtime([
            Rules {
                context: 7,
                ..Default::default()
            },
            Rules {
                context: 8,
                ..Default::default()
            },
        ]);
        let mut run = f.cursor(
            &runtime,
            SourceCacheLimits {
                batches,
                charged_capacity: 8 * 1024 * 1024,
            },
            Execution::Sequential,
        );
        let group = run.next_group(&runtime).unwrap().unwrap();
        run.apply_many(&runtime, &group.token, &run.pending()).unwrap();
        let before = run.source_session().unwrap().diagnostics();
        assert_eq!(before.opaque.listings, 4);
        finish(&mut run, &runtime);
        let after = run.source_session().unwrap().diagnostics();
        assert_eq!(after.paired.listings, if batches == 32 { 1 } else { 5 });
        if batches == 32 {
            assert_eq!(after.paired.cache_hits, 4);
        } else {
            assert_eq!(after.paired.cache_hits, 0);
        }
        for target in &f.targets {
            assert_eq!(fs::read(target.join("q/a/deep/value")).unwrap(), b"actual donor data");
            assert!(target.join("q/b/value").exists());
        }
    }
}

#[test]
fn subset_validation_is_atomic_and_failed_lane_does_not_stop_healthy_peer() {
    let f = Fixture::new();
    file(f.source.join("q/a"));
    file(f.source.join("q/nested/value"));
    let runtime = f.runtime([
        Rules {
            context: 1,
            refuse: Some("q/nested/value".into()),
            ..Default::default()
        },
        Rules {
            context: 2,
            ..Default::default()
        },
    ]);
    let mut run = f.cursor(&runtime, SourceCacheLimits::default(), Execution::Sequential);
    let group = run.next_group(&runtime).unwrap().unwrap();
    let saved = run.parts();
    let before = run.source_session().unwrap().diagnostics();
    assert!(matches!(
        run.apply_many(&runtime, &group.token, &["lane-0".into(), "lane-0".into()]),
        Err(Error::InvalidControl(_))
    ));
    assert!(matches!(
        run.apply_many(&runtime, &group.token, &["lane-0".into(), "unknown".into()]),
        Err(Error::InvalidControl(_))
    ));
    assert_eq!(saved, run.parts());
    assert_eq!(before, run.source_session().unwrap().diagnostics());
    assert!(!f.targets[0].join("q").exists());
    let result = run.apply_many(&runtime, &group.token, &run.pending()).unwrap();
    let failure = result.outcomes[0].outcome.as_ref().unwrap_err();
    assert_eq!(failure.primary.path, f.targets[0].join("q/nested"));
    assert!(failure.to_string().contains("original lane refusal"));
    assert_eq!(failure.completed.files_copied, 1);
    assert_eq!(result.outcomes[1].outcome.as_ref().unwrap().completed.files_copied, 2);
    assert_eq!(
        run.current().unwrap().applications.iter().map(|a| a.state).collect::<Vec<_>>(),
        [ApplicationState::Failed, ApplicationState::Accepted]
    );
    run.parts().validate().unwrap();
    assert_eq!(run.source_session().unwrap().diagnostics().active_batches, 0);
    for target in &f.targets {
        no_staging(target);
    }
    finish(&mut run, &runtime);
}

#[test]
fn rebound_source_is_fresh_and_apply_one_still_leaves_an_exact_pending_peer() {
    let f = Fixture::new();
    file(f.source.join("q/nested/value"));
    let runtime = f.runtime(std::array::from_fn(|_| Rules {
        context: 6,
        ..Default::default()
    }));
    let mut run = f.cursor(&runtime, SourceCacheLimits::default(), Execution::Sequential);
    let group = run.next_group(&runtime).unwrap().unwrap();
    run.apply(&runtime, &group.token, &"lane-0".into()).unwrap();
    let source = run.source_session().unwrap().clone();
    assert!(source.diagnostics().retained_batches > 0);
    let parts = run.into_parts();
    let mut run = SyncCursor::from_parts(parts, Execution::Sequential).unwrap();
    run.bind(&runtime).unwrap();
    assert_ne!(source.generation(), run.source_session().unwrap().generation());
    assert_eq!(run.source_session().unwrap().diagnostics().retained_batches, 0);
    assert_eq!(run.pending(), ["lane-1".into()]);
    assert!(!f.targets[1].join("q").exists());
    let token = run.current_token().unwrap();
    run.apply_many(&runtime, &token, &["lane-1".into()]).unwrap();
    finish(&mut run, &runtime);
}

#[cfg(target_os = "linux")]
#[test]
fn actual_target_clone_failure_closes_its_leases_and_healthy_peer_finishes() {
    use std::os::unix::fs::{MetadataExt, PermissionsExt};
    let mut f = Fixture::new();
    file(f.source.join("q/deep/value"));
    let external = tempfile::tempdir_in("/tmp/opencode").unwrap();
    let failed = external.path().join("receiver");
    fs::create_dir(&failed).unwrap();
    f.targets[0] = failed;
    assert_ne!(
        fs::metadata(&f.source).unwrap().dev(),
        fs::metadata(&f.targets[0]).unwrap().dev(),
        "this discriminator needs real distinct filesystems"
    );
    for target in &f.targets {
        fs::set_permissions(target, fs::Permissions::from_mode(0o555)).unwrap();
    }
    let runtime = SyncRuntime::new(
        (0..2)
            .map(|i| SyncTarget {
                id: format!("lane-{i}").as_str().into(),
                descriptor: format!("clone/{i}"),
                applier: Applier::new_policy(
                    &f.source,
                    &f.targets[i],
                    CopyPolicy::ReflinkRequired,
                    Rules {
                        context: i as u8,
                        ..Default::default()
                    },
                    StatKey,
                )
                .unwrap(),
            })
            .collect(),
    );
    let mut run = SyncCursor::new(&runtime, Execution::Sequential).unwrap();
    let group = run.next_group(&runtime).unwrap().unwrap();
    let report = run.apply_many(&runtime, &group.token, &run.pending()).unwrap();
    let failure = report.outcomes[0].outcome.as_ref().unwrap_err();
    assert_eq!(failure.primary.operation, "prepare reflink");
    assert_eq!(failure.primary.path, f.targets[0].join("q/deep/value"));
    assert!(failure.primary.cause.downcast_ref::<std::io::Error>().unwrap().raw_os_error().is_some());
    assert_eq!(failure.completed.files_reflinked, 0);
    assert_eq!(failure.completed.directories_created, 2);
    assert_eq!(report.outcomes[1].outcome.as_ref().unwrap().completed.files_reflinked, 1);
    assert!(f.targets[1].join("q/deep/value").exists());
    assert_eq!(run.source_session().unwrap().diagnostics().opaque.listings, 2);
    for target in &f.targets {
        assert_eq!(fs::metadata(target).unwrap().permissions().mode() & 0o777, 0o555);
        no_staging(target);
        fs::set_permissions(target, fs::Permissions::from_mode(0o755)).unwrap();
    }
    run.parts().validate().unwrap();
}
