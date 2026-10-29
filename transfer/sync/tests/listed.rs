use diff_dirs_transfer_sync::*;
use diff_dirs_tree_compare::coordinated::TargetId;
use diff_dirs_tree_inspect::{CheckError, DeclaredCandidate, EntryFacts, ObservationResources, SourceCacheLimits, SourceSession};
use std::{
    fs,
    path::{Path, PathBuf},
};

struct Fixture {
    _temp: tempfile::TempDir,
    source: PathBuf,
    targets: Vec<PathBuf>,
}
impl Fixture {
    fn new(count: usize) -> Self {
        let scratch = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.test-agent/listed");
        fs::create_dir_all(&scratch).unwrap();
        let temp = tempfile::tempdir_in(fs::canonicalize(scratch).unwrap()).unwrap();
        let source = temp.path().join("source");
        fs::create_dir(&source).unwrap();
        let targets: Vec<_> = (0..count).map(|i| temp.path().join(format!("target-{i}"))).collect();
        for target in &targets {
            fs::create_dir(target).unwrap();
        }
        Self { _temp: temp, source, targets }
    }
    fn bindings(&self, policy: CopyPolicy) -> Vec<EffectBinding> {
        self.targets.iter().map(|t| EffectBinding::new(&self.source, t, policy).unwrap()).collect()
    }
}
fn file(path: impl AsRef<Path>, bytes: &[u8]) {
    fs::create_dir_all(path.as_ref().parent().unwrap()).unwrap();
    fs::write(path, bytes).unwrap();
}
fn run(f: &Fixture, desired: &PreparedLeafList, owned: &OwnedLeafSet, intent: ListedIntent) -> ListedOutcome {
    let bindings = f.bindings(CopyPolicy::Copy);
    let id = TargetId("target".into());
    reconcile_listed(
        &SourceSession::new(&f.source),
        desired,
        &[ListedTarget {
            id: &id,
            binding: &bindings[0],
            owned,
            intent,
            guard: &NoListedGuard,
        }],
    )
    .unwrap()
    .outcomes
    .remove(0)
}
fn clean(path: &Path) {
    assert!(
        fs::read_dir(path)
            .unwrap()
            .all(|e| !e.unwrap().file_name().as_encoded_bytes().starts_with(b".diff-dirs-sync-"))
    );
}

#[test]
fn preparation_is_canonical_filesystem_free_and_rejects_conflicting_obligations() {
    let a = PreparedLeafList::new(["z/a", "p/b"]).unwrap();
    let b = PreparedLeafList::new(["p//b", "z/./a"]).unwrap();
    assert_eq!(a.id(), b.id());
    assert_ne!(a.id(), OwnedLeafSet::new(["p/b", "z/a"]).unwrap().id());
    assert!(PreparedLeafList::new(["a", "a/b"]).is_err());
    assert!(PreparedLeafList::new(["a/b", "a/./b"]).is_err());
    for bad in ["", "/a", "a/../b", "a\0b"] {
        assert!(OwnedLeafSet::new([bad]).is_err());
    }
    assert!(
        PreparedLeafList::from_specs([DesiredLeaf {
            relative: "link".into(),
            expected: LeafExpectation::Exact {
                kind: LeafKind::Symlink,
                executable: Some(true)
            }
        }])
        .is_err()
    );
}

#[test]
fn actual_missing_wrong_kind_and_mode_fail_whole_immediate_obligation_batch() {
    for variant in 0..3 {
        let f = Fixture::new(1);
        file(f.source.join("a"), b"must not install before later failure");
        let expected = match variant {
            0 => LeafExpectation::SupportedLeaf,
            1 => {
                fs::create_dir(f.source.join("z")).unwrap();
                LeafExpectation::Exact {
                    kind: LeafKind::RegularFile,
                    executable: None,
                }
            }
            _ => {
                file(f.source.join("z"), b"non executable");
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    fs::set_permissions(f.source.join("z"), fs::Permissions::from_mode(0o644)).unwrap();
                }
                LeafExpectation::Exact {
                    kind: LeafKind::RegularFile,
                    executable: Some(true),
                }
            }
        };
        let desired = PreparedLeafList::from_specs([
            DesiredLeaf {
                relative: "a".into(),
                expected: LeafExpectation::SupportedLeaf,
            },
            DesiredLeaf {
                relative: "z".into(),
                expected,
            },
        ]);
        #[cfg(not(unix))]
        if variant == 2 {
            assert!(desired.is_err());
            continue;
        }
        let outcome = run(
            &f,
            &desired.unwrap(),
            &OwnedLeafSet::new(Vec::<PathBuf>::new()).unwrap(),
            ListedIntent::CreateAbsent,
        );
        assert!(outcome.outcome.is_err());
        assert!(!outcome.coverage.complete);
        assert_eq!(outcome.coverage.installed, 0);
        assert!(!f.targets[0].join("a").exists());
        clean(&f.targets[0]);
    }
}

#[cfg(unix)]
#[test]
fn strict_equal_occupancy_refuses_but_exact_owned_equal_renewal_uses_private_inode() {
    use std::os::unix::fs::MetadataExt;
    let f = Fixture::new(1);
    file(f.source.join("same"), b"equal payload");
    file(f.targets[0].join("same"), b"equal payload");
    fs::hard_link(f.targets[0].join("same"), f.targets[0].join("old-alias")).unwrap();
    let old = fs::metadata(f.targets[0].join("same")).unwrap().ino();
    let desired = PreparedLeafList::new(["same"]).unwrap();
    let owned = OwnedLeafSet::new(["same"]).unwrap();
    let strict = run(&f, &desired, &owned, ListedIntent::CreateAbsent);
    assert!(strict.outcome.is_err());
    assert_eq!(strict.coverage.owned_obsolete, 0);
    assert_eq!(fs::metadata(f.targets[0].join("same")).unwrap().ino(), old);
    let renewed = run(&f, &desired, &owned, ListedIntent::RenewOwned);
    assert!(renewed.outcome.is_ok());
    assert!(renewed.coverage.complete);
    assert_eq!(renewed.coverage.installed, 1);
    assert_eq!(renewed.coverage.owned_obsolete, 0);
    assert_ne!(fs::metadata(f.targets[0].join("same")).unwrap().ino(), old);
    assert_eq!(fs::metadata(f.targets[0].join("old-alias")).unwrap().ino(), old);
    assert_ne!(
        fs::metadata(f.source.join("same")).unwrap().ino(),
        fs::metadata(f.targets[0].join("same")).unwrap().ino()
    );
    clean(&f.targets[0]);
}

#[test]
fn obsolete_only_never_touches_the_source_and_retains_empty_structural_directories() {
    let f = Fixture::new(1);
    file(f.targets[0].join("keep/old"), b"old");
    let bindings = f.bindings(CopyPolicy::Copy);
    let source = SourceSession::new(&f.source);
    fs::rename(&f.source, f.source.with_file_name("source-not-admitted")).unwrap();
    let desired = PreparedLeafList::new(Vec::<PathBuf>::new()).unwrap();
    let owned = OwnedLeafSet::new(["keep/old", "missing/a", "missing/b", "absent-leaf"]).unwrap();
    let id = TargetId("only-old".into());
    let outcome = reconcile_listed(
        &source,
        &desired,
        &[ListedTarget {
            id: &id,
            binding: &bindings[0],
            owned: &owned,
            intent: ListedIntent::RenewOwned,
            guard: &NoListedGuard,
        }],
    )
    .unwrap()
    .outcomes
    .remove(0);
    assert!(outcome.coverage.complete);
    assert_eq!(outcome.coverage.owned_retired, 4);
    assert_eq!(outcome.outcome.unwrap().completed.files_removed, 1);
    assert!(f.targets[0].join("keep").is_dir());
    assert_eq!(fs::read_dir(f.targets[0].join("keep")).unwrap().count(), 0);
    assert_eq!(source.diagnostics().declared.metadata_acquisitions, 0);
    assert_eq!(source.diagnostics().declared.listings, 0);
}

#[test]
fn missing_owned_prefix_is_settled_once_before_new_desired_materialization() {
    let f = Fixture::new(1);
    file(f.source.join("p/new"), b"new");
    let outcome = run(
        &f,
        &PreparedLeafList::new(["p/new"]).unwrap(),
        &OwnedLeafSet::new(["p/old/a", "p/old/b"]).unwrap(),
        ListedIntent::RenewOwned,
    );
    assert!(outcome.coverage.complete);
    assert_eq!(outcome.coverage.installed, 1);
    assert_eq!(outcome.coverage.owned_retired, 2);
    let report = outcome.outcome.unwrap();
    assert_eq!(report.completed.files_removed, 0);
    assert_eq!(report.completed.directories_created, 1);
    assert!(!f.targets[0].join("p/old").exists());
}

#[cfg(unix)]
#[test]
fn collapse_unknown_file_or_empty_directory_vetoes_before_staging_chmod_or_owned_deletion() {
    use std::os::unix::fs::PermissionsExt;
    for empty_directory in [false, true] {
        let f = Fixture::new(1);
        file(f.source.join("q"), b"replacement");
        file(f.targets[0].join("q/a"), b"owned-a");
        file(f.targets[0].join("q/deep/b"), b"owned-b");
        if empty_directory {
            fs::create_dir(f.targets[0].join("q/deep/local")).unwrap();
        } else {
            file(f.targets[0].join("q/deep/local"), b"unowned");
        }
        fs::set_permissions(f.targets[0].join("q"), fs::Permissions::from_mode(0o555)).unwrap();
        let outcome = run(
            &f,
            &PreparedLeafList::new(["q"]).unwrap(),
            &OwnedLeafSet::new(["q/a", "q/deep/b"]).unwrap(),
            ListedIntent::RenewOwned,
        );
        assert!(!outcome.coverage.complete);
        assert_eq!(outcome.coverage.owned_retired, 0);
        let failure = outcome.outcome.unwrap_err();
        assert_eq!(failure.completed, Progress::default());
        assert_eq!(fs::read(f.targets[0].join("q/a")).unwrap(), b"owned-a");
        assert_eq!(fs::read(f.targets[0].join("q/deep/b")).unwrap(), b"owned-b");
        assert!(fs::symlink_metadata(f.targets[0].join("q/deep/local")).is_ok());
        assert_eq!(fs::metadata(f.targets[0].join("q")).unwrap().permissions().mode() & 0o777, 0o555);
        clean(&f.targets[0]);
        clean(&f.targets[0].join("q"));
    }
}

#[test]
fn expansion_is_manifest_only_and_failed_private_preparation_keeps_old_leaf() {
    for missing in [false, true] {
        let f = Fixture::new(1);
        file(f.targets[0].join("p"), b"old-leaf");
        file(f.source.join("p/a"), b"a");
        file(f.source.join("p/unlisted"), b"must never enter output");
        if !missing {
            file(f.source.join("p/b"), b"b");
        }
        let outcome = run(
            &f,
            &PreparedLeafList::new(["p/a", "p/b"]).unwrap(),
            &OwnedLeafSet::new(["p"]).unwrap(),
            ListedIntent::RenewOwned,
        );
        if missing {
            assert!(outcome.outcome.is_err());
            assert_eq!(outcome.coverage.installed, 0);
            assert_eq!(outcome.coverage.owned_retired, 0);
            assert_eq!(fs::read(f.targets[0].join("p")).unwrap(), b"old-leaf");
        } else {
            assert!(outcome.coverage.complete);
            assert_eq!(outcome.coverage.installed, 2);
            assert_eq!(outcome.coverage.owned_retired, 1);
            let report = outcome.outcome.unwrap();
            assert_eq!(report.completed.files_copied, 2);
            assert_eq!(report.completed.files_removed, 1);
            assert_eq!(report.completed.directories_created, 1);
            assert!(!f.targets[0].join("p/unlisted").exists());
        }
        clean(&f.targets[0]);
    }
}

#[test]
fn owned_leaf_now_directory_refuses_and_successful_collapse_consumes_obsolete_subtree_once() {
    let f = Fixture::new(1);
    file(f.source.join("q"), b"new");
    file(f.targets[0].join("q/a"), b"old-a");
    let refused = run(
        &f,
        &PreparedLeafList::new(["q"]).unwrap(),
        &OwnedLeafSet::new(["q"]).unwrap(),
        ListedIntent::RenewOwned,
    );
    assert!(refused.outcome.is_err());
    assert!(f.targets[0].join("q/a").is_file());
    let outcome = run(
        &f,
        &PreparedLeafList::new(["q"]).unwrap(),
        &OwnedLeafSet::new(["q/a", "q/missing"]).unwrap(),
        ListedIntent::RenewOwned,
    );
    assert!(outcome.coverage.complete);
    assert_eq!(outcome.coverage.owned_retired, 2);
    assert_eq!(outcome.coverage.installed, 1);
    let report = outcome.outcome.unwrap();
    assert_eq!(report.completed.files_removed, 1);
    assert_eq!(report.completed.directories_removed, 1);
    assert_eq!(fs::read(f.targets[0].join("q")).unwrap(), b"new");
}

#[derive(Clone, Copy)]
struct Guard {
    refuse: bool,
}
impl ListedGuard for Guard {
    fn check(&self, at: ListedCandidate<'_>, _: EntryFacts<'_>) -> Result<(), CheckError> {
        if self.refuse && matches!(at.observed,DeclaredCandidate::Entry(e) if e.coordinate.name=="z") {
            return Err("actual later guard refusal".into());
        }
        Ok(())
    }
}
#[test]
fn late_guard_refusal_is_zero_effect_for_its_lane_and_peers_remain_independent() {
    let f = Fixture::new(2);
    file(f.source.join("a"), b"a");
    file(f.source.join("z"), b"z");
    let bindings = f.bindings(CopyPolicy::Copy);
    let ids = [TargetId("bad".into()), TargetId("good".into())];
    let guards = [Guard { refuse: true }, Guard { refuse: false }];
    let owned = OwnedLeafSet::new(Vec::<PathBuf>::new()).unwrap();
    let desired = PreparedLeafList::new(["a", "z"]).unwrap();
    let targets: Vec<_> = (0..2)
        .map(|i| ListedTarget {
            id: &ids[i],
            binding: &bindings[i],
            owned: &owned,
            intent: ListedIntent::CreateAbsent,
            guard: &guards[i],
        })
        .collect();
    let outcomes = reconcile_listed(&SourceSession::new(&f.source), &desired, &targets).unwrap().outcomes;
    assert!(outcomes[0].outcome.is_err());
    assert_eq!(outcomes[0].coverage.installed, 0);
    assert_eq!(fs::read_dir(&f.targets[0]).unwrap().count(), 0);
    assert!(outcomes[1].coverage.complete);
    assert_eq!(outcomes[1].coverage.installed, 2);
}

#[test]
fn absent_retained_and_staged_lanes_share_real_source_lookup_once_with_zero_cache() {
    let f = Fixture::new(3);
    let mut leaves = Vec::new();
    for i in 0..40 {
        let path = format!("q/branch-{i:02}/file");
        file(f.source.join(&path), b"payload");
        leaves.push(path);
    }
    fs::create_dir(f.targets[1].join("q")).unwrap();
    file(f.targets[2].join("q"), b"owned obstruction");
    let policy = if std::env::var_os("DIFF_DIRS_SYNC_REQUIRE_REFLINK").is_some() {
        CopyPolicy::ReflinkRequired
    } else {
        CopyPolicy::Copy
    };
    let bindings = f.bindings(policy);
    let owned = [
        OwnedLeafSet::new(Vec::<PathBuf>::new()).unwrap(),
        OwnedLeafSet::new(Vec::<PathBuf>::new()).unwrap(),
        OwnedLeafSet::new(["q"]).unwrap(),
    ];
    let ids = [TargetId("absent".into()), TargetId("retained".into()), TargetId("staged".into())];
    let targets: Vec<_> = (0..3)
        .map(|i| ListedTarget {
            id: &ids[i],
            binding: &bindings[i],
            owned: &owned[i],
            intent: ListedIntent::RenewOwned,
            guard: &NoListedGuard,
        })
        .collect();
    let source = SourceSession::with_cache_limits(
        &f.source,
        ObservationResources::default(),
        SourceCacheLimits {
            batches: 0,
            charged_capacity: 0,
        },
    );
    let report = reconcile_listed(&source, &PreparedLeafList::new(&leaves).unwrap(), &targets).unwrap();
    for (i, outcome) in report.outcomes.into_iter().enumerate() {
        assert!(outcome.coverage.complete, "lane {i}: {:?}", outcome.outcome);
        assert_eq!(outcome.coverage.installed, 40);
        if policy == CopyPolicy::ReflinkRequired {
            assert_eq!(outcome.outcome.unwrap().completed.files_reflinked, 40);
        }
        clean(&f.targets[i]);
    }
    let diagnostic = source.diagnostics();
    assert_eq!(diagnostic.declared.metadata_acquisitions, 81);
    assert_eq!(diagnostic.declared.listings, 0);
    assert_eq!(diagnostic.active_batches, 0);
    assert_eq!(diagnostic.retained_batches, 0);
}

#[test]
fn absent_structural_proof_has_zero_retention_but_complete_owned_retirement() {
    let f = Fixture::new(1);
    let outcome = run(
        &f,
        &PreparedLeafList::new(Vec::<PathBuf>::new()).unwrap(),
        &OwnedLeafSet::new(["missing/a", "missing/deep/b"]).unwrap(),
        ListedIntent::RenewOwned,
    );
    assert!(outcome.coverage.complete);
    assert_eq!(outcome.coverage.owned_retired, 2);
    let report = outcome.outcome.unwrap();
    assert_eq!(report.policy.retained, 0);
    assert_eq!(report.completed, Progress::default());
}

#[cfg(target_os = "linux")]
#[test]
fn actual_clone_failure_keeps_known_retirement_and_healthy_peer_complete() {
    use std::os::unix::fs::{MetadataExt, PermissionsExt};
    let mut f = Fixture::new(2);
    file(f.source.join("z"), b"actual required reflink");
    let external = tempfile::tempdir_in("/tmp/opencode").unwrap();
    f.targets[0] = external.path().join("receiver");
    fs::create_dir(&f.targets[0]).unwrap();
    assert_ne!(
        fs::metadata(&f.source).unwrap().dev(),
        fs::metadata(&f.targets[0]).unwrap().dev(),
        "real distinct filesystems required"
    );
    for target in &f.targets {
        file(target.join("a"), b"obsolete");
        fs::set_permissions(target, fs::Permissions::from_mode(0o555)).unwrap();
    }
    let bindings = f.bindings(CopyPolicy::ReflinkRequired);
    let ids = [TargetId("bad".into()), TargetId("good".into())];
    let owned = OwnedLeafSet::new(["a", "already-absent"]).unwrap();
    let desired = PreparedLeafList::new(["z"]).unwrap();
    let targets: Vec<_> = (0..2)
        .map(|i| ListedTarget {
            id: &ids[i],
            binding: &bindings[i],
            owned: &owned,
            intent: ListedIntent::RenewOwned,
            guard: &NoListedGuard,
        })
        .collect();
    let report = reconcile_listed(&SourceSession::new(&f.source), &desired, &targets).unwrap();
    let failed = &report.outcomes[0];
    assert!(!failed.coverage.complete);
    assert_eq!(failed.coverage.installed, 0);
    assert_eq!(failed.coverage.owned_retired, 2);
    let failure = failed.outcome.as_ref().unwrap_err();
    assert_eq!(failure.primary.operation, "prepare reflink");
    assert_eq!(failure.completed.files_removed, 1);
    assert!(failure.primary.cause.downcast_ref::<std::io::Error>().unwrap().raw_os_error().is_some());
    assert!(report.outcomes[1].coverage.complete);
    assert_eq!(report.outcomes[1].outcome.as_ref().unwrap().completed.files_reflinked, 1);
    for target in &f.targets {
        assert_eq!(fs::metadata(target).unwrap().permissions().mode() & 0o777, 0o555);
        clean(target);
        fs::set_permissions(target, fs::Permissions::from_mode(0o755)).unwrap();
    }
}

// Deliberately impure test-only fault injection at the pure guard seam. Production
// guards must not mutate trees; these force genuine cleanup/publication I/O errors
// after known effects, without adding a production fault API or post-failure scan.
struct Fault {
    target: PathBuf,
    cleanup: bool,
    seen: std::sync::atomic::AtomicUsize,
}
impl ListedGuard for Fault {
    fn check(&self, at: ListedCandidate<'_>, _: EntryFacts<'_>) -> Result<(), CheckError> {
        if let DeclaredCandidate::Entry(entry) = at.observed {
            let trigger = if self.cleanup {
                entry.coordinate.parent == Path::new("z")
            } else {
                at.mode == ListedMode::CoupledRemoval
            };
            if trigger {
                let call = self.seen.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                if self.cleanup || call == 1 {
                    let arena = fs::read_dir(&self.target)
                        .unwrap()
                        .map(Result::unwrap)
                        .find(|e| e.file_name().as_encoded_bytes().starts_with(b".diff-dirs-sync-"))
                        .unwrap()
                        .path();
                    if self.cleanup {
                        fs::write(arena.join("unexpected-private-artifact"), b"force cleanup error").unwrap();
                    } else {
                        fs::remove_file(arena.join("0")).unwrap();
                    }
                }
            }
        }
        Ok(())
    }
}
#[test]
fn cleanup_failure_cannot_complete_even_when_all_desired_renames_succeeded() {
    let f = Fixture::new(1);
    file(f.source.join("a"), b"a");
    file(f.source.join("z/b"), b"b");
    let bindings = f.bindings(CopyPolicy::Copy);
    let id = TargetId("cleanup".into());
    let desired = PreparedLeafList::new(["a", "z/b"]).unwrap();
    let owned = OwnedLeafSet::new(Vec::<PathBuf>::new()).unwrap();
    let guard = Fault {
        target: f.targets[0].clone(),
        cleanup: true,
        seen: Default::default(),
    };
    let report = reconcile_listed(
        &SourceSession::new(&f.source),
        &desired,
        &[ListedTarget {
            id: &id,
            binding: &bindings[0],
            owned: &owned,
            intent: ListedIntent::CreateAbsent,
            guard: &guard,
        }],
    )
    .unwrap();
    let outcome = &report.outcomes[0];
    assert_eq!(outcome.coverage.installed, 2);
    assert!(!outcome.coverage.complete);
    let failure = outcome.outcome.as_ref().unwrap_err();
    assert_eq!(failure.primary.operation, "clean private staging directory");
    assert_eq!(failure.completed.files_copied, 2);
    assert_eq!(fs::read(f.targets[0].join("z/b")).unwrap(), b"b");
}
#[test]
fn coupled_old_retirement_survives_later_publication_failure_without_postfailure_census() {
    let f = Fixture::new(1);
    file(f.source.join("q"), b"new");
    file(f.targets[0].join("q/a"), b"old");
    let bindings = f.bindings(CopyPolicy::Copy);
    let id = TargetId("publication".into());
    let desired = PreparedLeafList::new(["q"]).unwrap();
    let owned = OwnedLeafSet::new(["q/a", "q/absent"]).unwrap();
    let guard = Fault {
        target: f.targets[0].clone(),
        cleanup: false,
        seen: Default::default(),
    };
    let report = reconcile_listed(
        &SourceSession::new(&f.source),
        &desired,
        &[ListedTarget {
            id: &id,
            binding: &bindings[0],
            owned: &owned,
            intent: ListedIntent::RenewOwned,
            guard: &guard,
        }],
    )
    .unwrap();
    let outcome = &report.outcomes[0];
    assert!(!outcome.coverage.complete);
    assert_eq!(outcome.coverage.installed, 0);
    assert_eq!(outcome.coverage.owned_retired, 2);
    let failure = outcome.outcome.as_ref().unwrap_err();
    assert_eq!(failure.primary.operation, "install prepared node");
    assert_eq!(failure.completed.files_removed, 1);
    assert_eq!(failure.completed.directories_removed, 1);
    assert_eq!(guard.seen.load(std::sync::atomic::Ordering::Relaxed), 2);
    assert!(!f.targets[0].join("q").exists());
    clean(&f.targets[0]);
}

#[cfg(unix)]
#[test]
fn restrictive_umask_preserves_one_created_scope_through_inspection_and_postorder_failure() {
    use std::os::unix::fs::PermissionsExt;
    const CHILD: &str = "DIFF_DIRS_LISTED_UMASK_CHILD";
    let scratch = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.test-agent/listed-umask");
    if std::env::var_os(CHILD).is_none() {
        fs::create_dir_all(&scratch).unwrap();
        let output = std::process::Command::new("sh")
            .args([
                "-c",
                "umask 0777; exec \"$1\" --exact restrictive_umask_preserves_one_created_scope_through_inspection_and_postorder_failure --nocapture",
                "listed-umask-test",
            ])
            .arg(std::env::current_exe().unwrap())
            .env(CHILD, "1")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        return;
    }
    let temp = tempfile::tempdir_in(fs::canonicalize(scratch).unwrap()).unwrap();
    fs::set_permissions(temp.path(), fs::Permissions::from_mode(0o700)).unwrap();
    let source_root = temp.path().join("source");
    fs::create_dir(&source_root).unwrap();
    fs::set_permissions(&source_root, fs::Permissions::from_mode(0o700)).unwrap();
    for path in ["p", "p/deep"] {
        fs::create_dir(source_root.join(path)).unwrap();
        fs::set_permissions(source_root.join(path), fs::Permissions::from_mode(0o700)).unwrap();
    }
    fs::write(source_root.join("p/deep/value"), b"post-umask physical copy").unwrap();
    fs::set_permissions(source_root.join("p/deep/value"), fs::Permissions::from_mode(0o444)).unwrap();
    fs::set_permissions(source_root.join("p"), fs::Permissions::from_mode(0o751)).unwrap();
    fs::set_permissions(source_root.join("p/deep"), fs::Permissions::from_mode(0o750)).unwrap();
    let paths: Vec<_> = (0..3).map(|i| temp.path().join(format!("target-{i}"))).collect();
    for path in &paths[1..] {
        fs::create_dir(path).unwrap();
        fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
    }
    fs::write(paths[2].join("p"), b"old owned leaf").unwrap();
    fs::set_permissions(paths[2].join("p"), fs::Permissions::from_mode(0o444)).unwrap();
    let policy = if std::env::var_os("DIFF_DIRS_SYNC_REQUIRE_REFLINK").is_some() {
        CopyPolicy::ReflinkRequired
    } else {
        CopyPolicy::Copy
    };
    let bindings: Vec<_> = paths.iter().map(|p| EffectBinding::new(&source_root, p, policy).unwrap()).collect();
    let desired = PreparedLeafList::new(["p/deep/value"]).unwrap();
    let owned = [
        OwnedLeafSet::new(Vec::<PathBuf>::new()).unwrap(),
        OwnedLeafSet::new(Vec::<PathBuf>::new()).unwrap(),
        OwnedLeafSet::new(["p"]).unwrap(),
    ];
    let ids = [TargetId("new-root".into()), TargetId("retained".into()), TargetId("staged".into())];
    struct Refuse(bool);
    impl ListedGuard for Refuse {
        fn check(&self, at: ListedCandidate<'_>, _: EntryFacts<'_>) -> Result<(), CheckError> {
            if self.0 && matches!(at.observed,DeclaredCandidate::Entry(e) if e.coordinate.name=="value") {
                Err("deep batch refusal".into())
            } else {
                Ok(())
            }
        }
    }
    for refuse in [true, false] {
        let guard = Refuse(refuse);
        let targets: Vec<_> = (0..3)
            .map(|i| ListedTarget {
                id: &ids[i],
                binding: &bindings[i],
                owned: &owned[i],
                intent: ListedIntent::RenewOwned,
                guard: &guard,
            })
            .collect();
        let source = SourceSession::new(&source_root);
        let report = reconcile_listed(&source, &desired, &targets).unwrap();
        assert_eq!(source.diagnostics().declared.metadata_acquisitions, 3);
        for (i, outcome) in report.outcomes.into_iter().enumerate() {
            if refuse {
                let failure = outcome.outcome.unwrap_err();
                assert_eq!(outcome.coverage.installed, 0);
                assert!(!outcome.coverage.complete);
                assert_eq!(failure.completed.directories_created, [3, 2, 0][i]);
                assert_eq!(failure.completed.directory_permissions_set, [6, 4, 0][i]);
                if i == 2 {
                    assert_eq!(fs::read(paths[i].join("p")).unwrap(), b"old owned leaf");
                    clean(&paths[i]);
                    continue;
                }
            } else {
                assert!(outcome.coverage.complete, "lane {i}: {:?}", outcome.outcome);
                assert_eq!(outcome.coverage.installed, 1);
                assert_eq!(fs::read(paths[i].join("p/deep/value")).unwrap(), b"post-umask physical copy");
            }
            assert_eq!(fs::metadata(paths[i].join("p")).unwrap().permissions().mode() & 0o777, 0o751);
            assert_eq!(fs::metadata(paths[i].join("p/deep")).unwrap().permissions().mode() & 0o777, 0o750);
            clean(&paths[i]);
        }
    }
}
