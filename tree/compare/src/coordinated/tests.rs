use super::*;
use crate::test_support::{Fixture, file};
use crate::{CheckError, Names, Operations};
use std::fs;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

#[test]
fn explicit_settlement_closes_only_ready_returns_without_inspection() {
    let setup = Setup::new(1);
    for root in [&setup.bindings.source, &setup.bindings.targets[0].root] {
        fs::create_dir(root.join("a")).unwrap();
        fs::create_dir(root.join("b")).unwrap();
    }
    let policies = setup.loans();
    let mut run = Coordinated::new(setup.bindings.clone(), Execution::Sequential).unwrap();
    run.bind(&policies).unwrap();
    run.next_group(&policies).unwrap().unwrap();
    run.next_group(&policies).unwrap().unwrap();
    let inspections = setup.policies[0].directories.lock().unwrap().len();
    run.settle();
    assert!(run.current_token().is_none());
    assert_eq!(run.parts().frames.len(), 1);
    assert_eq!(run.parts().frames[0].remaining[0].name, "b");
    assert_eq!(setup.policies[0].directories.lock().unwrap().len(), inspections);
    run.parts().validate().unwrap();
    assert_eq!(run.next_group(&policies).unwrap().unwrap().relative, Path::new("b"));
    run.settle();
    assert_eq!(run.parts().lanes[0].status, LaneStatus::Complete);
}

#[derive(Default)]
struct Policy {
    preserve: Vec<PathBuf>,
    refuse: Option<PathBuf>,
    refuse_root: bool,
    roots: AtomicUsize,
    directories: Mutex<Vec<PathBuf>>,
}

impl Visitor for Policy {
    fn early(&self, names: &mut Names) -> Result<(), CheckError> {
        if names.is_root() {
            self.roots.fetch_add(1, Ordering::Relaxed);
        } else {
            self.directories.lock().unwrap().push(names.dir().to_path_buf());
        }
        for name in names.entries_mut() {
            if self.preserve.iter().any(|p| p == name.relative_path()) {
                name.preserve_tree();
            }
        }
        Ok(())
    }
    fn late(&self, operations: &mut Operations) -> Result<(), CheckError> {
        if operations.is_root() && self.refuse_root {
            return Err("fixture root refusal".into());
        }
        if !operations.is_root() && self.refuse.as_deref() == Some(operations.dir()) {
            return Err("fixture refusal".into());
        }
        Ok(())
    }
}

struct Setup {
    _fixture: Fixture,
    bindings: Bindings,
    policies: Vec<Policy>,
}

impl Setup {
    fn new(n: usize) -> Self {
        let fixture = Fixture::new();
        let mut targets = Vec::new();
        for i in 0..n {
            let root = fixture.root.join(format!("target-{i}"));
            fs::create_dir(&root).unwrap();
            targets.push(TargetBinding {
                id: TargetId(format!("lane-{i}")),
                root,
                policy: format!("fixture-{i}/v1"),
            });
        }
        let bindings = Bindings {
            source: fixture.original.clone(),
            targets,
        };
        Self {
            _fixture: fixture,
            bindings,
            policies: (0..n).map(|_| Policy::default()).collect(),
        }
    }
    fn loans(&self) -> Vec<TargetPolicy<'_, Policy>> {
        self.bindings
            .targets
            .iter()
            .zip(&self.policies)
            .map(|(t, visitor)| TargetPolicy {
                target: &t.id,
                descriptor: &t.policy,
                visitor,
            })
            .collect()
    }
    fn run(&self) -> Coordinated {
        let mut run = Coordinated::new(self.bindings.clone(), Execution::Sequential).unwrap();
        run.bind(&self.loans()).unwrap();
        run
    }
    fn shape(&self) {
        for dir in ["a/nested", "b", "opaque/deep"] {
            fs::create_dir_all(self.bindings.source.join(dir)).unwrap();
        }
        for name in ["root-file", "a/nested/file", "b/file", "opaque/deep/file"] {
            file(&self.bindings.source.join(name), b"source");
        }
        for (i, target) in self.bindings.targets.iter().enumerate() {
            for dir in ["a/nested", "b", "extra/deep"] {
                fs::create_dir_all(target.root.join(dir)).unwrap();
            }
            file(&target.root.join("root-file"), if i == 0 { b"source" } else { b"different" });
            file(&target.root.join("a/nested/file"), b"different");
            file(&target.root.join("b/file"), b"source");
            file(&target.root.join("extra/deep/file"), b"extra");
        }
    }
}

type Projection = (Vec<DirDiff>, Option<PathBuf>, Summary);

fn independent(setup: &Setup, target: usize) -> Projection {
    let mut walk = crate::walk_with_visitor(&setup.bindings.source, &setup.bindings.targets[target].root, setup.policies[target].borrowed()).unwrap();
    let mut diffs = Vec::new();
    let mut error = None;
    for item in walk.by_ref() {
        match item {
            Ok(plan) => diffs.push(plan.diff().clone()),
            Err(e) => error = Some(e.path().to_path_buf()),
        }
    }
    (diffs, error, walk.into_summary())
}

fn collect(setup: &Setup, mut run: Coordinated, restore: bool) -> Vec<Projection> {
    let mut diffs = vec![Vec::new(); setup.bindings.targets.len()];
    let mut errors = vec![None; diffs.len()];
    loop {
        if restore {
            let before = run.parts().clone();
            let old_token = run.current_token();
            let summaries = run.prefix_summaries();
            assert_eq!(run.parts(), &before, "summary projection mutated continuation");
            run = Coordinated::from_parts(run.into_parts(), Execution::Sequential).unwrap();
            assert_eq!(run.parts(), &before);
            if let Some(old) = old_token {
                assert_ne!(run.current_token().as_ref(), Some(&old));
                assert!(run.prune_here(&old, &[]).is_err());
            }
            assert_eq!(run.prefix_summaries(), summaries);
            run.bind(&setup.loans()).unwrap();
        }
        let Some(group) = run.next_group(&setup.loans()).unwrap() else {
            break;
        };
        run.parts().validate().unwrap();
        for outcome in group.outcomes {
            let index = setup.bindings.targets.iter().position(|t| t.id == outcome.target).unwrap();
            match outcome.outcome {
                Ok(compared) if !compared.diff.is_empty() => diffs[index].push(compared.diff),
                Ok(_) => {}
                Err(error) => {
                    assert!(errors[index].is_none());
                    errors[index] = Some(error.path().to_path_buf());
                }
            }
        }
    }
    run.parts().validate().unwrap();
    let summaries = run.prefix_summaries();
    diffs
        .into_iter()
        .zip(errors)
        .zip(summaries)
        .map(|((diff, error), (_, summary))| (diff, error, summary))
        .collect()
}

#[test]
fn every_lane_projects_to_independent_walk_with_and_without_restoration() {
    for n in [1, 2, 4] {
        let mut s = Setup::new(n);
        s.shape();
        if n > 1 {
            s.policies[1].preserve.push("a".into());
        }
        if n > 2 {
            fs::remove_dir_all(s.bindings.targets[2].root.join("a")).unwrap();
            file(&s.bindings.targets[2].root.join("a"), b"collision");
            s.policies[2].preserve.push("extra".into());
        }
        let expected: Vec<_> = (0..n).map(|i| independent(&s, i)).collect();
        assert_eq!(collect(&s, s.run(), false), expected);
        assert_eq!(collect(&s, s.run(), true), expected);
    }
}

#[test]
fn deep_partial_frontier_keeps_another_lanes_ancestor_head_and_does_not_replay_policy() {
    let mut s = Setup::new(2);
    s.shape();
    s.policies[1].preserve.push("a".into());
    let mut run = s.run();
    for expected in ["", "a", "a/nested"] {
        let group = run.next_group(&s.loans()).unwrap().unwrap();
        assert_eq!(group.relative, Path::new(expected));
        let parts = run.into_parts();
        parts.validate().unwrap();
        assert_eq!(parts.lanes[0].status, LaneStatus::Active);
        assert!(parts.lanes[0].summary.changed_dirs.is_empty());
        run = Coordinated::from_parts(parts, Execution::Sequential).unwrap();
        run.bind(&s.loans()).unwrap();
    }
    assert_eq!(run.heads()[1].1, Head::Child { relative: "b".into() });
    assert_eq!(s.policies[0].roots.load(Ordering::Relaxed), 1);
    assert_eq!(s.policies[1].roots.load(Ordering::Relaxed), 1);
    assert_eq!(*s.policies[0].directories.lock().unwrap(), [PathBuf::new(), "a".into(), "a/nested".into()]);
    let before = run.parts().clone();
    let projected = run.prefix_summaries();
    assert_eq!(projected[0].1.changed_dirs, [PathBuf::from("a/nested"), "a".into(), PathBuf::new()]);
    assert_eq!(run.parts(), &before);
    assert_eq!(run.next_group(&s.loans()).unwrap().unwrap().relative, Path::new("b"));
    while run.next_group(&s.loans()).unwrap().is_some() {}
    assert_eq!(run.prefix_summaries()[0].1.changed_dirs, projected[0].1.changed_dirs);
}

#[test]
fn pruning_is_subset_atomic_and_saved_window_reissues_its_token() {
    let s = Setup::new(3);
    s.shape();
    let mut run = s.run();
    let root = run.next_group(&s.loans()).unwrap().unwrap();
    let ids: Vec<_> = s.bindings.targets.iter().map(|t| t.id.clone()).collect();
    for subset in [vec![ids[0].clone(), TargetId::from("unknown")], vec![ids[0].clone(), ids[0].clone()]] {
        let before = run.parts().clone();
        assert!(run.prune_here(&root.token, &subset).is_err());
        assert_eq!(run.parts(), &before);
    }
    let mut foreign = s.run();
    let foreign_token = foreign.next_group(&s.loans()).unwrap().unwrap().token;
    assert!(run.prune_here(&foreign_token, &ids[..1]).is_err());
    run.prune_here(&root.token, &ids[..1]).unwrap();
    let parts = run.into_parts();
    parts.validate().unwrap();
    run = Coordinated::from_parts(parts, Execution::Sequential).unwrap();
    let token = run.current_token().unwrap();
    assert_ne!(token, root.token);
    assert!(run.prune_here(&root.token, &ids[1..2]).is_err());
    let before = run.parts().clone();
    assert!(run.prune_here(&token, &ids[..2]).is_err());
    assert_eq!(run.parts(), &before);
    run.prune_here(&token, &ids[1..2]).unwrap();
    run.bind(&s.loans()).unwrap();
    let a = run.next_group(&s.loans()).unwrap().unwrap();
    assert_eq!(a.relative, Path::new("a"));
    assert_eq!(a.outcomes.len(), 1);
    assert_eq!(a.outcomes[0].target, ids[2]);
    assert!(run.prune_here(&token, &ids[2..]).is_err());
    while run.next_group(&s.loans()).unwrap().is_some() {}
    let summaries = run.prefix_summaries();
    assert_eq!(summaries[0].1.pruned, [PathBuf::new()]);
    assert_eq!(summaries[1].1.pruned, [PathBuf::new()]);
    assert!(summaries[2].1.pruned.is_empty());
    for target in 0..2 {
        let mut independent = crate::walk_with_visitor(&s.bindings.source, &s.bindings.targets[target].root, s.policies[target].borrowed()).unwrap();
        assert!(independent.next().unwrap().is_ok());
        assert!(independent.prune_here());
        assert!(independent.next().is_none());
        assert_eq!(summaries[target].1, independent.into_summary());
    }
    assert_eq!(summaries[2].1, independent(&s, 2).2);
}

#[test]
fn nested_failure_drops_only_that_lanes_siblings_and_folds_its_prefix_once() {
    let mut s = Setup::new(2);
    s.shape();
    s.policies[0].refuse = Some("a/nested".into());
    let expected: Vec<_> = (0..2).map(|i| independent(&s, i)).collect();
    assert_eq!(collect(&s, s.run(), true), expected);
}

#[test]
fn stop_removes_memberships_at_all_depths_and_inactive_missing_roots_do_not_rebind() {
    let s = Setup::new(2);
    s.shape();
    let mut run = s.run();
    for _ in 0..3 {
        run.next_group(&s.loans()).unwrap().unwrap();
    }
    let id = s.bindings.targets[0].id.clone();
    let before = run.parts().clone();
    assert!(run.stop_lanes(&[id.clone(), "unknown".into()]).is_err());
    assert_eq!(run.parts(), &before);
    run.stop_lanes(std::slice::from_ref(&id)).unwrap();
    let stopped = run.prefix_summaries()[0].1.clone();
    let parts = run.into_parts();
    parts.validate().unwrap();
    fs::remove_dir_all(&s.bindings.targets[0].root).unwrap();
    let loans = s.loans();
    run = Coordinated::from_parts(parts, Execution::Sequential).unwrap();
    run.bind(&loans[1..]).unwrap();
    while let Some(group) = run.next_group(&loans[1..]).unwrap() {
        assert!(group.outcomes.iter().all(|o| o.target != id));
    }
    assert_eq!(run.prefix_summaries()[0].1, stopped);
    assert_eq!(run.prefix_summaries()[1].1, independent(&s, 1).2);
}

#[test]
fn all_stopped_at_deep_frontier_has_valid_parts_and_terminal_resume_needs_no_roots() {
    let s = Setup::new(2);
    s.shape();
    let mut run = s.run();
    for _ in 0..3 {
        run.next_group(&s.loans()).unwrap().unwrap();
    }
    let ids: Vec<_> = s.bindings.targets.iter().map(|t| t.id.clone()).collect();
    run.stop_lanes(&ids).unwrap();
    let parts = run.into_parts();
    parts.validate().unwrap();
    fs::remove_dir_all(&s.bindings.source).unwrap();
    for t in &s.bindings.targets {
        fs::remove_dir_all(&t.root).unwrap();
    }
    run = Coordinated::from_parts(parts, Execution::Sequential).unwrap();
    let none: &[TargetPolicy<'_, Policy>] = &[];
    run.bind(none).unwrap();
    assert!(run.next_group(none).unwrap().is_none());
    run.into_parts().validate().unwrap();
}

#[test]
fn root_preservation_never_inspects_interiors_and_empty_clean_groups_are_delivered() {
    let mut s = Setup::new(2);
    s.policies[0].preserve.push(PathBuf::new());
    let mut run = s.run();
    let root = run.next_group(&s.loans()).unwrap().unwrap();
    assert_eq!(root.outcomes.len(), 2);
    assert!(root.outcomes[0].outcome.as_ref().unwrap().structural_root);
    assert!(!root.outcomes[1].outcome.as_ref().unwrap().structural_root);
    assert!(root.outcomes.iter().all(|o| o.outcome.as_ref().unwrap().diff.is_empty()));
    assert!(s.policies[0].directories.lock().unwrap().is_empty());
    assert!(run.next_group(&s.loans()).unwrap().is_none());
    assert_eq!(run.prefix_summaries()[0].1, independent(&s, 0).2);
}

#[test]
fn lowering_can_retain_validated_plans_without_putting_them_in_parts() {
    let s = Setup::new(2);
    s.shape();
    let mut run = s.run();
    let group = run.next_group_with(&s.loans(), |id, plan| (id.clone(), plan)).unwrap().unwrap();
    for lane in group.outcomes {
        let compared = lane.outcome.unwrap();
        assert_eq!(compared.value.0, lane.target);
        assert_eq!(compared.value.1.diff(), &compared.diff);
        assert_eq!(compared.value.1.operations().dir(), group.relative);
    }
    run.into_parts().validate().unwrap();
}

#[test]
fn malformed_parts_and_policy_descriptors_refuse_before_binding_io() {
    let s = Setup::new(2);
    s.shape();
    let mut run = s.run();
    run.next_group(&s.loans()).unwrap().unwrap();
    let parts = run.into_parts();
    let mut bad = Vec::new();
    let mut p = parts.clone();
    p.version = 2;
    bad.push(p);
    let mut p = parts.clone();
    p.frames[0].remaining[0].members.push(100);
    bad.push(p);
    let mut p = parts.clone();
    p.frames[0].remaining[0].name = "../escape".into();
    bad.push(p);
    let mut p = parts.clone();
    p.window.as_mut().unwrap().successful.reverse();
    bad.push(p);
    let mut p = parts.clone();
    p.next_sequence = 1;
    bad.push(p);
    let mut p = parts.clone();
    p.bindings.targets[1].id = p.bindings.targets[0].id.clone();
    bad.push(p);
    let mut p = parts.clone();
    p.lanes[0].summary.counts.files.added = u64::MAX;
    p.lanes[0].summary.counts.files.removed = 1;
    bad.push(p);
    let mut p = parts.clone();
    p.frames[0].lanes[0].subtree_changed = false;
    bad.push(p);
    let mut p = parts.clone();
    p.lanes[0].summary.pruned.push(PathBuf::new());
    bad.push(p);
    let mut p = Parts::fresh(s.bindings.clone()).unwrap();
    p.root_pending = false;
    for lane in &mut p.lanes {
        lane.status = LaneStatus::Complete;
        lane.summary.state = ScanState::Complete;
    }
    bad.push(p);
    fs::remove_dir_all(&s.bindings.source).unwrap();
    for parts in bad {
        assert!(Coordinated::from_parts(parts, Execution::Sequential).is_err());
    }
    let mut run = Coordinated::from_parts(parts, Execution::Sequential).unwrap();
    let mut loans = s.loans();
    loans[0].descriptor = "different-policy/v2";
    assert!(matches!(run.bind(&loans), Err(Error::PolicyMismatch(_))));
    assert!(matches!(run.next_group(&loans), Err(Error::PolicyMismatch(_))));
}

#[test]
fn structural_root_refusal_is_lane_local_and_does_not_replay_after_restore() {
    let mut s = Setup::new(2);
    s.shape();
    s.policies[0].refuse_root = true;
    let expected: Vec<_> = (0..2).map(|i| independent(&s, i)).collect();
    let roots_before: Vec<_> = s.policies.iter().map(|p| p.roots.load(Ordering::Relaxed)).collect();
    assert_eq!(collect(&s, s.run(), true), expected);
    for (i, p) in s.policies.iter().enumerate() {
        assert_eq!(p.roots.load(Ordering::Relaxed), roots_before[i] + 1);
    }
    assert!(s.policies[0].directories.lock().unwrap().is_empty());
}

#[test]
fn stopped_before_root_and_normal_terminal_restore_are_filesystem_free() {
    for stop in [true, false] {
        let s = Setup::new(1);
        let mut run = s.run();
        if stop {
            run.stop_lanes(&[s.bindings.targets[0].id.clone()]).unwrap();
        } else {
            while run.next_group(&s.loans()).unwrap().is_some() {}
        }
        let parts = run.into_parts();
        parts.validate().unwrap();
        fs::remove_dir_all(&s.bindings.source).unwrap();
        fs::remove_dir_all(&s.bindings.targets[0].root).unwrap();
        let mut run = Coordinated::from_parts(parts, Execution::Sequential).unwrap();
        let none: &[TargetPolicy<'_, Policy>] = &[];
        run.bind(none).unwrap();
        assert!(run.next_group(none).unwrap().is_none());
    }
}

#[cfg(unix)]
#[test]
fn native_name_component_dfs_and_read_only_aliases_remain_distinct() {
    use std::os::unix::ffi::OsStringExt;
    let s = Setup::new(2);
    for root in [&s.bindings.source, &s.bindings.targets[0].root] {
        for name in ["a/z", "a-", "b"] {
            fs::create_dir_all(root.join(name)).unwrap();
        }
        fs::create_dir(root.join(std::ffi::OsString::from_vec(vec![0xff]))).unwrap();
    }
    let mut bindings = s.bindings.clone();
    bindings.targets[1].root = bindings.targets[0].root.clone();
    let mut run = Coordinated::new(bindings, Execution::Sequential).unwrap();
    run.bind(&s.loans()).unwrap();
    let mut paths = Vec::new();
    while let Some(group) = run.next_group(&s.loans()).unwrap() {
        assert_eq!(group.outcomes.len(), 2);
        paths.push(group.relative);
    }
    assert_eq!(
        paths,
        [
            PathBuf::new(),
            "a".into(),
            "a/z".into(),
            "a-".into(),
            "b".into(),
            std::ffi::OsString::from_vec(vec![0xff]).into()
        ]
    );
    assert_eq!(run.prefix_summaries()[0].1, run.prefix_summaries()[1].1);
}

#[test]
fn failed_current_edge_does_not_consume_healthy_lanes_later_sibling() {
    let mut s = Setup::new(2);
    s.shape();
    s.policies[1].preserve.push("a".into());
    s.policies[0].refuse = Some("a".into());
    let mut run = s.run();
    run.next_group(&s.loans()).unwrap().unwrap();
    let failure = run.next_group(&s.loans()).unwrap().unwrap();
    assert_eq!(failure.relative, Path::new("a"));
    assert!(failure.outcomes[0].outcome.is_err());
    run.parts().validate().unwrap();
    let b = run.next_group(&s.loans()).unwrap().unwrap();
    assert_eq!(b.relative, Path::new("b"));
    assert_eq!(b.outcomes.len(), 1);
    assert_eq!(b.outcomes[0].target, s.bindings.targets[1].id);
}

#[test]
fn target_route_replacement_is_a_lane_failure_not_followed_or_global_abort() {
    let s = Setup::new(2);
    s.shape();
    let mut run = s.run();
    run.next_group(&s.loans()).unwrap().unwrap();
    fs::remove_dir_all(s.bindings.targets[0].root.join("a")).unwrap();
    file(&s.bindings.targets[0].root.join("a"), b"not a directory");
    let group = run.next_group(&s.loans()).unwrap().unwrap();
    assert!(group.outcomes[0].outcome.is_err());
    assert!(group.outcomes[1].outcome.is_ok());
    run.parts().validate().unwrap();
}

#[cfg(unix)]
#[test]
fn queued_sibling_routes_are_checked_after_resume_before_they_become_current() {
    use std::os::unix::fs::symlink;
    for source in [true, false] {
        let s = Setup::new(2);
        s.shape();
        let mut run = s.run();
        for _ in 0..3 {
            run.next_group(&s.loans()).unwrap().unwrap();
        }
        let parts = run.into_parts();
        let root = if source { &s.bindings.source } else { &s.bindings.targets[0].root };
        fs::rename(root.join("b"), root.join("not-selected-b")).unwrap();
        symlink("not-selected-b", root.join("b")).unwrap();
        let mut run = Coordinated::from_parts(parts, Execution::Sequential).unwrap();
        // Active ancestors are still valid; the future sibling is deliberately
        // validated later, at its own turn, without reopening accepted policy.
        run.bind(&s.loans()).unwrap();
        let b = run.next_group(&s.loans()).unwrap().unwrap();
        assert_eq!(b.relative, Path::new("b"));
        assert!(b.outcomes[0].outcome.is_err());
        if source {
            let a = b.outcomes[0].outcome.as_ref().unwrap_err();
            let b = b.outcomes[1].outcome.as_ref().unwrap_err();
            assert!(Arc::ptr_eq(a, b), "common route failure should retain one physical error");
        } else {
            assert!(b.outcomes[1].outcome.is_ok());
        }
        assert!(!s.policies[0].directories.lock().unwrap().contains(&PathBuf::from("b")));
        run.parts().validate().unwrap();
    }
}

#[test]
fn routing_diagnostics_are_separate_and_share_common_source_checks_within_a_group() {
    let s = Setup::new(2);
    s.shape();
    let mut run = s.run();
    assert_eq!(run.route_metadata_checks(), 0, "root binding is not an extra route check");
    run.next_group(&s.loans()).unwrap().unwrap();
    assert_eq!(run.route_metadata_checks(), 3, "one source root and two targets");
    run.next_group(&s.loans()).unwrap().unwrap();
    assert_eq!(run.route_metadata_checks(), 9, "shared source root/a plus two target routes");
    let mut run = Coordinated::from_parts(run.into_parts(), Execution::Sequential).unwrap();
    assert_eq!(run.route_metadata_checks(), 0);
    run.bind(&s.loans()).unwrap();
    assert_eq!(
        run.route_metadata_checks(),
        3,
        "rebind validates only active a routes, reusing bound root facts"
    );
}

#[cfg(unix)]
#[test]
fn source_special_failure_is_lane_local_and_matches_independent_walks() {
    let mut s = Setup::new(3);
    s.shape();
    let _socket = std::os::unix::net::UnixListener::bind(s.bindings.source.join("secret")).unwrap();
    s.policies[0].preserve.push("secret".into());
    s.policies[2].preserve.push("secret".into());
    let expected: Vec<_> = (0..3).map(|i| independent(&s, i)).collect();
    assert_eq!(collect(&s, s.run(), true), expected);
}

#[test]
fn arithmetic_failure_cannot_partially_accept_a_group() {
    let mut s = Setup::new(2);
    s.shape();
    s.policies[0].preserve.push("a/nested".into());
    let mut run = s.run();
    run.next_group(&s.loans()).unwrap().unwrap();
    let mut parts = run.into_parts();
    parts.lanes[0].summary.excluded = u64::MAX;
    let expected = parts.lanes.clone();
    let mut run = Coordinated::from_parts(parts, Execution::Sequential).unwrap();
    run.bind(&s.loans()).unwrap();
    assert!(matches!(run.next_group(&s.loans()), Err(Error::CounterOverflow)));
    assert_eq!(run.parts().lanes, expected);
    run.parts().validate().unwrap();
}

#[test]
fn failed_lane_missing_root_is_not_required_by_fresh_runtime() {
    let mut s = Setup::new(2);
    s.shape();
    s.policies[0].refuse = Some("a".into());
    let mut run = s.run();
    run.next_group(&s.loans()).unwrap().unwrap();
    run.next_group(&s.loans()).unwrap().unwrap();
    let parts = run.into_parts();
    fs::remove_dir_all(&s.bindings.targets[0].root).unwrap();
    let mut run = Coordinated::from_parts(parts, Execution::Sequential).unwrap();
    run.bind(&s.loans()[1..]).unwrap();
    while let Some(group) = run.next_group(&s.loans()[1..]).unwrap() {
        assert!(group.outcomes.iter().all(|o| o.target == s.bindings.targets[1].id));
    }
    assert_eq!(run.prefix_summaries()[1].1, independent(&s, 1).2);
}

#[test]
fn a_lane_returns_before_its_peers_nested_work_and_its_ancestor_is_folded_once() {
    let mut s = Setup::new(2);
    s.shape();
    file(&s.bindings.source.join("a/direct"), b"source");
    file(&s.bindings.targets[0].root.join("a/direct"), b"different");
    file(&s.bindings.targets[1].root.join("a/direct"), b"source");
    s.policies[0].preserve.push("a/nested".into());
    let expected: Vec<_> = (0..2).map(|i| independent(&s, i).2).collect();
    let mut run = s.run();
    for expected in ["", "a", "a/nested"] {
        assert_eq!(run.next_group(&s.loans()).unwrap().unwrap().relative, Path::new(expected));
    }
    assert_eq!(run.parts().lanes[0].summary.changed_dirs, [PathBuf::from("a")]);
    assert_eq!(run.parts().frames[1].lanes.len(), 1);
    assert_eq!(run.parts().frames[0].remaining[0].members, [1]);
    assert_eq!(run.heads()[0].1, Head::Child { relative: "b".into() });
    let mut run = Coordinated::from_parts(run.into_parts(), Execution::Sequential).unwrap();
    run.bind(&s.loans()).unwrap();
    while run.next_group(&s.loans()).unwrap().is_some() {}
    for (i, (_, summary)) in run.prefix_summaries().into_iter().enumerate() {
        assert_eq!(summary, expected[i]);
        assert_eq!(summary.changed_dirs.iter().filter(|p| p.as_path() == Path::new("a")).count(), 1);
    }
}

#[test]
fn restoration_rejects_lost_already_folded_child_evidence() {
    let s = Setup::new(1);
    for root in [&s.bindings.source, &s.bindings.targets[0].root] {
        for dir in ["a", "b"] {
            fs::create_dir(root.join(dir)).unwrap();
        }
    }
    file(&s.bindings.source.join("a/file"), b"source");
    file(&s.bindings.targets[0].root.join("a/file"), b"different");
    let mut run = s.run();
    for expected in ["", "a", "b"] {
        assert_eq!(run.next_group(&s.loans()).unwrap().unwrap().relative, Path::new(expected));
    }
    let mut parts = run.into_parts();
    parts.validate().unwrap();
    assert_eq!(parts.lanes[0].summary.changed_dirs, [PathBuf::from("a")]);
    assert!(!parts.frames[0].lanes[0].direct_changed);
    assert!(parts.frames[0].lanes[0].subtree_changed);
    parts.frames[0].lanes[0].subtree_changed = false;
    assert!(Coordinated::from_parts(parts, Execution::Sequential).is_err());
}

#[test]
fn restoration_rejects_closed_reductions_inside_unstarted_children() {
    let s = Setup::new(1);
    for root in [&s.bindings.source, &s.bindings.targets[0].root] {
        for dir in ["a", "b", "c/deep"] {
            fs::create_dir_all(root.join(dir)).unwrap();
        }
    }
    file(&s.bindings.source.join("a/file"), b"source");
    file(&s.bindings.targets[0].root.join("a/file"), b"different");
    let mut run = s.run();
    for expected in ["", "a", "b"] {
        assert_eq!(run.next_group(&s.loans()).unwrap().unwrap().relative, Path::new(expected));
    }
    let parts = run.into_parts();
    parts.validate().unwrap();
    assert_eq!(parts.lanes[0].summary.changed_dirs, [PathBuf::from("a")]);
    for unstarted in ["c", "c/deep"] {
        let mut corrupted = parts.clone();
        corrupted.lanes[0].summary.changed_dirs[0] = unstarted.into();
        assert!(Coordinated::from_parts(corrupted, Execution::Sequential).is_err());
    }
}

#[test]
fn partial_active_child_exempts_only_lanes_that_still_owe_its_return() {
    for early_return in [false, true] {
        let mut s = Setup::new(2);
        for root in std::iter::once(&s.bindings.source).chain(s.bindings.targets.iter().map(|t| &t.root)) {
            for dir in ["a/x", "a/y"] {
                fs::create_dir_all(root.join(dir)).unwrap();
            }
        }
        file(&s.bindings.source.join("a/x/file"), b"source");
        for target in &s.bindings.targets {
            file(&target.root.join("a/x/file"), b"different");
        }
        if early_return {
            s.policies[0].preserve.push("a/y".into());
        }
        let expected: Vec<_> = (0..2).map(|i| independent(&s, i).2).collect();
        let mut run = s.run();
        for expected in ["", "a", "a/x", "a/y"] {
            assert_eq!(run.next_group(&s.loans()).unwrap().unwrap().relative, Path::new(expected));
        }
        let parts = run.into_parts();
        parts.validate().unwrap();
        assert_eq!(parts.lanes[1].summary.changed_dirs, [PathBuf::from("a/x")]);
        assert!(!parts.frames[0].lanes[1].subtree_changed, "a has not returned for lane 1");
        assert!(parts.frames[1].lanes.iter().find(|l| l.target == 1).unwrap().subtree_changed);
        if early_return {
            assert_eq!(parts.frames[0].remaining[0].members, [1]);
            assert_eq!(parts.lanes[0].summary.changed_dirs, [PathBuf::from("a/x"), "a".into()]);
            assert!(parts.frames[0].lanes[0].subtree_changed, "a already returned for lane 0");
        } else {
            assert!(!parts.frames[0].lanes[0].subtree_changed);
        }
        let mut run = Coordinated::from_parts(parts, Execution::Sequential).unwrap();
        run.bind(&s.loans()).unwrap();
        while run.next_group(&s.loans()).unwrap().is_some() {}
        for (i, (_, summary)) in run.prefix_summaries().into_iter().enumerate() {
            assert_eq!(summary, expected[i]);
        }
        run.into_parts().validate().unwrap();
    }
}

#[test]
fn compact_live_rebind_keeps_noncontiguous_original_ids_and_binding_order() {
    let mut s = Setup::new(4);
    s.shape();
    s.policies[0].preserve.push(PathBuf::new());
    s.policies[2].preserve.push(PathBuf::new());
    let mut run = s.run();
    run.next_group(&s.loans()).unwrap().unwrap();
    run.next_group(&s.loans()).unwrap().unwrap();
    assert_eq!(run.parts().lanes[0].status, LaneStatus::Complete);
    assert_eq!(run.parts().lanes[2].status, LaneStatus::Complete);
    let parts = run.into_parts();
    for i in [0, 2] {
        fs::remove_dir_all(&s.bindings.targets[i].root).unwrap();
    }
    let mut policies = s.loans();
    policies.retain(|p| p.target != &s.bindings.targets[0].id && p.target != &s.bindings.targets[2].id);
    policies.reverse(); // lending order must not reinterpret binding order
    let mut run = Coordinated::from_parts(parts, Execution::Sequential).unwrap();
    run.bind(&policies).unwrap();
    while let Some(group) = run.next_group(&policies).unwrap() {
        assert_eq!(
            group.outcomes.iter().map(|o| &o.target).collect::<Vec<_>>(),
            [&s.bindings.targets[1].id, &s.bindings.targets[3].id]
        );
    }
    run.parts().validate().unwrap();
}

#[cfg(feature = "parallel")]
#[test]
fn within_group_parallel_execution_uses_the_callers_pool_without_changing_frontier_results() {
    let s = Setup::new(3);
    s.shape();
    for i in 0..80 {
        file(&s.bindings.source.join(format!("file-{i:03}")), b"new");
        for target in &s.bindings.targets {
            file(&target.root.join(format!("file-{i:03}")), b"old contents");
        }
    }
    let expected: Vec<_> = (0..3).map(|i| independent(&s, i)).collect();
    let pool = rayon::ThreadPoolBuilder::new().num_threads(2).build().unwrap();
    let mut run = Coordinated::new(s.bindings.clone(), Execution::Parallel).unwrap();
    run.bind(&s.loans()).unwrap();
    let mut diffs = vec![Vec::new(); 3];
    while let Some(group) = pool.install(|| run.next_group(&s.loans())).unwrap() {
        for outcome in group.outcomes {
            let i = s.bindings.targets.iter().position(|t| t.id == outcome.target).unwrap();
            let compared = outcome.outcome.unwrap();
            if !compared.diff.is_empty() {
                diffs[i].push(compared.diff);
            }
        }
    }
    for (i, (_, summary)) in run.prefix_summaries().into_iter().enumerate() {
        assert_eq!((diffs[i].clone(), None, summary), expected[i]);
    }
}
