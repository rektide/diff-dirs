//! Production contextual effects, not just generic saved data.
use diff_dirs_transfer_sync::coordinated::{ApplicationState, Error, Limits, Stop};
use diff_dirs_transfer_sync::{Applier, CopyPolicy, Progress, SyncCursor, SyncRuntime, SyncTarget};
use diff_dirs_tree_inspect::Execution;
use diff_dirs_tree_inspect::{
    Action, Choices, Container, ContextSite, InstallAuthority, InstallIntent, Interior, InteriorInput, Kind, Names, Policy, SemanticId, ShapeChoices,
    SizeAndLink, SourceObservation,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, LazyLock, Mutex};

#[derive(Clone, Debug, PartialEq, Eq)]
struct Context {
    admitted: bool,
    depth: usize,
    region: bool,
}
#[derive(Clone, Default)]
struct Rules {
    region: Option<PathBuf>,
    conditional: bool,
    renew: bool,
    omit: bool,
    refuse: Option<PathBuf>,
    // Test-only fault injection between shape and execution acquisition.
    remove_donor: Option<PathBuf>,
    events: Arc<Mutex<Vec<(PathBuf, bool, Context)>>>,
    release_witness: Arc<Mutex<Option<std::sync::Weak<diff_dirs_tree_inspect::DirectorySelection<Context>>>>>,
}
static ID: LazyLock<SemanticId> = LazyLock::new(|| SemanticId::new("test.contextual-effects", 1, vec![]));
impl Policy for Rules {
    type Context = Context;
    fn identity(&self) -> &SemanticId {
        &ID
    }
    fn initial_context(&self) -> Context {
        Context {
            admitted: false,
            depth: 0,
            region: false,
        }
    }
    fn scoped(&self) -> bool {
        self.region.is_some()
    }
    fn validate_context(&self, at: ContextSite<'_>, c: &Context) -> Result<(), diff_dirs_tree_inspect::CheckError> {
        let initial = at.structural_root && at.role.is_none();
        if c.admitted == initial || c.depth != if initial { 0 } else { at.relative.components().count() + 1 } {
            return Err("incorrect admitted contextual input".into());
        }
        Ok(())
    }
    fn names(&self, c: &Context, names: &mut Names) -> Result<(), diff_dirs_tree_inspect::CheckError> {
        let root = names.is_root();
        for entry in names.entries_mut() {
            if !root && entry.name() == "protected" {
                entry.preserve_tree();
            }
        }
        let _ = c;
        Ok(())
    }
    fn shape(&self, _: &Context, batch: &mut ShapeChoices<'_, Context>) -> Result<(), diff_dirs_tree_inspect::CheckError> {
        if self.omit {
            for mut entry in batch.entries_mut() {
                if entry.shape().source.kind().is_some_and(Kind::is_leaf) {
                    entry.omit_comparison();
                }
            }
        }
        Ok(())
    }
    fn select(&self, input: &Context, batch: &mut Choices<'_, Context>) -> Result<(), diff_dirs_tree_inspect::CheckError> {
        let root = batch.info().structural_root;
        if !root
            && !batch.info().relative.as_os_str().is_empty()
            && let Some(witness) = self.release_witness.lock().unwrap().as_ref()
        {
            assert_eq!(witness.strong_count(), 2, "recursive frame retained its parent prepared selection/fact arena");
        }
        self.events.lock().unwrap().push((batch.info().relative.to_owned(), root, input.clone()));
        for mut entry in batch.entries_mut() {
            let relative = entry.coordinate().to_path_buf();
            if self.refuse.as_deref() == Some(&relative) {
                return Err("contextual opaque refusal".into());
            }
            let region = input.region || self.region.as_deref().unwrap_or(Path::new("")) == relative;
            if !root && entry.shape().source == SourceObservation::Absent && !region {
                entry.preserve_tree();
                continue;
            }
            if self.conditional
                && entry.shape().source == SourceObservation::Present(Kind::Directory)
                && entry.shape().target.is_none()
                && entry.name() != "empty"
            {
                entry.set_action(Action::Directory {
                    container: Container::CreateIfNeeded,
                    descend: true,
                });
            }
            if (self.renew || self.omit) && entry.shape().source.kind().is_some_and(Kind::is_leaf) && entry.shape().target != Some(Kind::Directory) {
                entry.set_action(Action::InstallLeaf);
                entry.set_install(InstallAuthority {
                    intent: InstallIntent::Renew,
                    destination: if entry.shape().target.is_some() {
                        diff_dirs_tree_inspect::DestinationGrant::ReplaceLeaf
                    } else {
                        diff_dirs_tree_inspect::DestinationGrant::AbsentOnly
                    },
                });
            }
            if let Some(role) = entry.action().and_then(|a| diff_dirs_tree_inspect::selection::interior_role(entry.shape(), a)) {
                entry.set_interior(Interior {
                    role,
                    context: Context {
                        admitted: true,
                        depth: input.depth + 1,
                        region,
                    },
                    input: InteriorInput::PhysicalDirectory,
                });
            }
        }
        if !root && let Some(path) = &self.remove_donor {
            fs::remove_file(path).unwrap();
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
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.test-agent/sync-tests");
        fs::create_dir_all(&root).unwrap();
        let owned = tempfile::tempdir_in(fs::canonicalize(root).unwrap()).unwrap();
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
    fn applier(&self, i: usize, rules: Rules) -> Applier<Rules, SizeAndLink> {
        Applier::new_policy(&self.source, &self.targets[i], CopyPolicy::Copy, rules, SizeAndLink).unwrap()
    }
    fn runtime(&self, rules: &[Rules]) -> SyncRuntime<Rules, SizeAndLink> {
        SyncRuntime::new(
            rules
                .iter()
                .enumerate()
                .map(|(i, r)| SyncTarget {
                    id: format!("lane-{i}").as_str().into(),
                    descriptor: format!("contextual/{i}"),
                    applier: self.applier(i, r.clone()),
                })
                .collect(),
        )
    }
}
fn file(path: impl AsRef<Path>, bytes: &[u8]) {
    let p = path.as_ref();
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(p, bytes).unwrap();
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
fn finish<C: Clone + Eq + std::fmt::Debug + Send + Sync>(
    cursor: &mut SyncCursor<C>,
    runtime: &SyncRuntime<impl Policy<Context = C>, impl diff_dirs_tree_inspect::Comparator>,
) {
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
    assert!(r.failures.is_empty(), "{:?}", r.failures);
}

#[test]
fn inherited_regions_root_derivation_paired_and_missing_lanes_restore_without_readmission() {
    let f = Fixture::new();
    file(f.source.join("region/paired/deep/new"), b"new");
    file(f.source.join("region/protected/no"), b"do not copy");
    fs::create_dir_all(f.targets[0].join("region/paired/deep")).unwrap();
    for target in &f.targets {
        file(target.join("region/obsolete/deep/gone"), b"remove");
        file(target.join("outside/keep"), b"retain");
        file(target.join("region/protected/local"), b"retain");
    }
    let rules = vec![
        Rules {
            region: Some("region".into()),
            ..Default::default()
        };
        2
    ];
    let runtime = f.runtime(&rules);
    let mut run = SyncCursor::new(&runtime, Execution::Sequential).unwrap();
    let source = Arc::clone(run.source_session().unwrap());
    let first = run.next_group(&runtime).unwrap().unwrap();
    let a = first.outcomes[0].outcome.as_ref().unwrap();
    assert!(Arc::ptr_eq(&a.value, &run.current().unwrap().applications[0].decision));
    run.apply(&runtime, &first.token, &"lane-0".into()).unwrap();
    assert!(Arc::ptr_eq(&source, run.source_session().unwrap()));
    let parts = run.into_parts();
    assert_eq!(parts.current.as_ref().unwrap().applications[0].state, ApplicationState::Accepted);
    assert_eq!(parts.current.as_ref().unwrap().applications[1].state, ApplicationState::Unstarted);
    for rule in &rules {
        rule.events.lock().unwrap().clear();
    }
    let mut restored = SyncCursor::from_parts(parts, Execution::Sequential).unwrap();
    restored.bind(&runtime).unwrap();
    let token = restored.current_token().unwrap();
    restored.apply(&runtime, &token, &"lane-1".into()).unwrap();
    finish(&mut restored, &runtime);
    for (i, target) in f.targets.iter().enumerate() {
        assert_eq!(fs::read(target.join("region/paired/deep/new")).unwrap(), b"new");
        assert!(!target.join("region/obsolete").exists());
        assert_eq!(fs::read(target.join("outside/keep")).unwrap(), b"retain");
        assert!(target.join("region/protected/local").is_file());
        assert!(!target.join("region/protected/no").exists());
        no_staging(target);
        assert!(rules[i].events.lock().unwrap().iter().all(|(_, root, _)| !*root));
    }
}

#[cfg(unix)]
#[test]
fn sparse_equal_renewal_is_diff_clean_but_installs_private_inode_and_literal_link() {
    use std::os::unix::fs::{MetadataExt, symlink};
    let f = Fixture::new();
    file(f.source.join("file"), b"equal");
    file(f.targets[0].join("file"), b"equal");
    symlink("literal/../missing", f.source.join("link")).unwrap();
    symlink("literal/../missing", f.targets[0].join("link")).unwrap();
    let old = fs::metadata(f.targets[0].join("file")).unwrap().ino();
    let r = f
        .applier(
            0,
            Rules {
                renew: true,
                ..Default::default()
            },
        )
        .run()
        .unwrap();
    assert_eq!(r.completed.files_copied, 1);
    assert_eq!(r.completed.symlinks_created, 1);
    assert!(r.observed.unwrap().is_clean());
    assert_ne!(fs::metadata(f.targets[0].join("file")).unwrap().ino(), old);
    assert_eq!(fs::read_link(f.targets[0].join("link")).unwrap(), Path::new("literal/../missing"));
    no_staging(&f.targets[0]);
    let r = f
        .applier(
            0,
            Rules {
                omit: true,
                ..Default::default()
            },
        )
        .run()
        .unwrap();
    assert_eq!(r.completed.files_copied, 1);
    assert_eq!(r.observed.unwrap().not_assessed, 2);
}

#[test]
fn conditional_empty_nonempty_and_private_replacement_use_exact_context_and_final_paths() {
    let f = Fixture::new();
    file(f.source.join("unused/protected/no"), b"excluded");
    fs::create_dir_all(f.source.join("route/deep/empty")).unwrap();
    file(f.source.join("route/selected"), b"one");
    file(f.source.join("expand/route/new"), b"two");
    file(f.targets[0].join("expand"), b"old leaf");
    let rules = Rules {
        conditional: true,
        ..Default::default()
    };
    let r = f.applier(0, rules.clone()).run().unwrap();
    assert_eq!(r.completed.files_copied, 2);
    assert!(!f.targets[0].join("unused").exists());
    assert!(f.targets[0].join("route/deep/empty").is_dir());
    assert!(f.targets[0].join("expand/route/new").is_file());
    assert!(
        rules
            .events
            .lock()
            .unwrap()
            .iter()
            .all(|(p, _, _)| !p.as_os_str().as_encoded_bytes().windows(11).any(|w| w == b".diff-dirs-"))
    );
    no_staging(&f.targets[0]);
    // A failed private expansion discards only its own effects, never a visible sibling.
    let g = Fixture::new();
    file(g.source.join("a-visible"), b"visible");
    file(g.source.join("z-expand/good"), b"provisional");
    file(g.source.join("z-expand/nested/bad"), b"refused");
    file(g.targets[0].join("z-expand"), b"old");
    let f = g
        .applier(
            0,
            Rules {
                refuse: Some("z-expand/nested/bad".into()),
                ..Default::default()
            },
        )
        .run()
        .unwrap_err();
    assert_eq!(f.completed.files_copied, 1);
    assert_eq!(f.completed.directories_created, 0);
    assert_eq!(fs::read(g.targets[0].join("z-expand")).unwrap(), b"old");
    assert_eq!(fs::read(g.targets[0].join("a-visible")).unwrap(), b"visible");
    no_staging(&g.targets[0]);
}

#[test]
fn contextual_collapse_veto_is_intact_and_both_passes_inherit_removal_context() {
    let f = Fixture::new();
    file(f.source.join("region/collapse"), b"new leaf");
    file(f.targets[0].join("region/collapse/a-removable"), b"old");
    file(f.targets[0].join("region/collapse/protected/keep"), b"keep");
    let r = Rules {
        region: Some("region".into()),
        ..Default::default()
    };
    let failure = f.applier(0, r.clone()).run().unwrap_err();
    assert_eq!(failure.completed, Progress::default());
    assert!(f.targets[0].join("region/collapse/a-removable").exists());
    no_staging(&f.targets[0]);
    fs::remove_dir_all(f.targets[0].join("region/collapse/protected")).unwrap();
    let report = f.applier(0, r).run().unwrap();
    assert_eq!(report.completed.files_removed, 1);
    assert_eq!(report.completed.directories_removed, 1);
    assert_eq!(fs::read(f.targets[0].join("region/collapse")).unwrap(), b"new leaf");
}

#[cfg(unix)]
#[test]
fn sparse_later_execution_acquisition_failure_has_no_immediate_effect_or_lease() {
    let f = Fixture::new();
    file(f.source.join("a"), b"would install");
    // Unlike attributes retained during unknown-kind resolution, link text is
    // still an unacquired execution input with native-dirent disabled too.
    std::os::unix::fs::symlink("literal-missing", f.source.join("z")).unwrap();
    let rules = Rules {
        omit: true,
        remove_donor: Some(f.source.join("z")),
        ..Default::default()
    };
    // The injector is deliberately not a conforming stable-tree policy. It
    // distinguishes acquire-before-effect from a kernel fallback lookup.
    let failure = f.applier(0, rules).run().unwrap_err();
    assert_eq!(failure.completed, Progress::default());
    assert_eq!(fs::read_dir(&f.targets[0]).unwrap().count(), 0);
}

#[test]
fn all_pending_restored_refresh_precedes_a_selected_healthy_lane_effect() {
    let f = Fixture::new();
    file(f.source.join("new"), b"new");
    let rules = vec![Rules::default(); 2];
    let runtime = f.runtime(&rules);
    let mut run = SyncCursor::new(&runtime, Execution::Sequential).unwrap();
    run.next_group(&runtime).unwrap();
    let parts = run.into_parts();
    file(f.targets[1].join("new"), b"changed target");
    let mut restored = SyncCursor::from_parts(parts, Execution::Sequential).unwrap();
    restored.bind(&runtime).unwrap();
    let token = restored.current_token().unwrap();
    assert!(matches!(restored.apply(&runtime, &token, &"lane-0".into()), Err(Error::Refresh(_))));
    assert!(!f.targets[0].join("new").exists());
    assert_eq!(restored.pending(), vec!["lane-0".into()]);
    restored.apply(&runtime, &token, &"lane-0".into()).unwrap();
}

#[test]
fn opaque_child_frames_release_the_immediate_preparation_before_recursive_inspection() {
    let f = Fixture::new();
    file(f.source.join("opaque/deep/leaf"), b"new");
    let rules = Rules::default();
    let runtime = f.runtime(std::slice::from_ref(&rules));
    let mut cursor = SyncCursor::new(&runtime, Execution::Sequential).unwrap();
    let group = cursor.next_group(&runtime).unwrap().unwrap();
    let decision = &group.outcomes[0].outcome.as_ref().unwrap().value;
    assert_eq!(Arc::strong_count(decision), 3);
    *rules.release_witness.lock().unwrap() = Some(Arc::downgrade(decision));
    cursor.apply(&runtime, &group.token, &"lane-0".into()).unwrap();
    assert_eq!(Arc::strong_count(decision), 2);
    assert!(f.targets[0].join("opaque/deep/leaf").is_file());
}

#[test]
fn contextual_structural_roots_and_preview_use_selected_interior_inputs() {
    let f = Fixture::new();
    file(f.source.join("route/leaf"), b"new");
    file(f.source.join("route/protected/no"), b"excluded");
    fs::remove_dir(&f.targets[0]).unwrap();
    let rules = Rules {
        conditional: true,
        ..Default::default()
    };
    let binding = f.applier(0, rules.clone());
    let preview = binding.preview().unwrap();
    assert_eq!(preview.completed, Progress::default());
    assert!(!f.targets[0].exists());
    let report = binding.initialize().unwrap();
    assert_eq!(report.files_copied, 1);
    assert!(f.targets[0].join("route/leaf").is_file());
    assert!(!f.targets[0].join("route/protected").exists());
    fs::remove_dir(&f.targets[1]).unwrap();
    fs::write(&f.targets[1], b"old root").unwrap();
    let binding = Applier::new_replacing_policy(&f.source, &f.targets[1], CopyPolicy::Copy, rules.clone(), SizeAndLink).unwrap();
    assert_eq!(binding.preview().unwrap().completed, Progress::default());
    assert_eq!(fs::read(&f.targets[1]).unwrap(), b"old root");
    assert_eq!(binding.replace_root().unwrap().files_copied, 1);
    assert!(f.targets[1].join("route/leaf").is_file());
    no_staging(&f.targets[1]);
    assert!(
        rules
            .events
            .lock()
            .unwrap()
            .iter()
            .all(|(p, _, _)| !p.as_os_str().as_encoded_bytes().windows(11).any(|w| w == b".diff-dirs-"))
    );
}
