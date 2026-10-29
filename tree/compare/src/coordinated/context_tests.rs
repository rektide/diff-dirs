use super::*;
use crate::test_support::{Fixture, set_time, timestamp};
use crate::{Action, Choices, DestinationGrant, InstallAuthority, InstallIntent, Interior, InteriorInput, Kind, SizeAndLink, SourceObservation};
use std::sync::Mutex;

#[derive(Clone, Debug, PartialEq, Eq)]
struct Context {
    depth: usize,
    residual: u32,
}

struct ContextPolicy {
    id: SemanticId,
    schema: SemanticId,
    residual: u32,
    renew: bool,
    omit: bool,
    calls: Mutex<Vec<(PathBuf, Context, bool)>>,
}
impl ContextPolicy {
    fn new(residual: u32) -> Self {
        Self {
            id: SemanticId::new("test.context", 1, vec![]),
            schema: SemanticId::new("test.context-schema", 1, vec![]),
            residual,
            renew: false,
            omit: false,
            calls: Mutex::new(vec![]),
        }
    }
}
impl crate::Policy for ContextPolicy {
    type Context = Context;
    fn identity(&self) -> &SemanticId {
        &self.id
    }
    fn context_schema(&self) -> &SemanticId {
        &self.schema
    }
    fn initial_context(&self) -> Context {
        Context {
            depth: 0,
            residual: self.residual,
        }
    }
    fn scoped(&self) -> bool {
        false
    }
    fn validate_context(&self, at: ContextSite<'_>, context: &Context) -> Result<(), crate::CheckError> {
        let depth = if at.structural_root && at.role.is_none() {
            0
        } else {
            at.relative.components().count() + 1
        };
        if context.residual != self.residual || context.depth != depth {
            return Err("unknown residual or wrong contextual depth".into());
        }
        Ok(())
    }
    fn shape(&self, _: &Context, choices: &mut crate::ShapeChoices<'_, Context>) -> Result<(), crate::CheckError> {
        if self.omit {
            for mut entry in choices.entries_mut() {
                if entry.shape().source == SourceObservation::Present(Kind::File) {
                    entry.omit_comparison();
                }
            }
        }
        Ok(())
    }
    fn select(&self, context: &Context, batch: &mut Choices<'_, Context>) -> Result<(), crate::CheckError> {
        let root = batch.info().structural_root;
        self.calls.lock().unwrap().push((batch.info().relative.to_owned(), context.clone(), root));
        for mut entry in batch.entries_mut() {
            if !root && context.residual == 19 && entry.name() == "a" {
                entry.preserve_tree();
                continue;
            }
            if entry.shape().source == SourceObservation::Present(Kind::Directory) && entry.shape().target == Some(Kind::Directory) {
                entry.set_interior(Interior {
                    role: InteriorRole::Paired,
                    context: Context {
                        depth: context.depth + 1,
                        residual: context.residual,
                    },
                    input: InteriorInput::PhysicalDirectory,
                });
            } else if self.renew && entry.shape().source == SourceObservation::Present(Kind::File) {
                entry.set_action(Action::InstallLeaf);
                entry.set_install(InstallAuthority {
                    intent: InstallIntent::Renew,
                    destination: DestinationGrant::ReplaceLeaf,
                });
            }
        }
        Ok(())
    }
}

#[test]
fn derived_root_and_deep_independent_contextual_heads_survive_without_replay() {
    let fixture = Fixture::new();
    for path in ["a/deep/last", "b"] {
        fixture.both_dir(path);
    }
    let policies = [ContextPolicy::new(7), ContextPolicy::new(19)];
    let bindings = Bindings {
        source: fixture.original.clone(),
        targets: policies
            .iter()
            .enumerate()
            .map(|(i, p)| {
                TargetBinding::new(
                    TargetId(format!("lane-{i}")),
                    fixture.target.clone(),
                    format!("residual-{}", p.residual),
                    p,
                    &StatKey,
                )
            })
            .collect(),
    };
    let loans: Vec<_> = bindings
        .targets
        .iter()
        .zip(&policies)
        .map(|(b, p)| TargetPolicy {
            target: &b.id,
            descriptor: &b.policy,
            policy: p,
            comparator: StatKey,
        })
        .collect();
    let mut run = Coordinated::new(bindings.clone(), Execution::Sequential).unwrap();
    run.bind(&loans).unwrap();
    let root = run.next_group_with(&loans, |_, plan| plan).unwrap().unwrap();
    for outcome in root.outcomes {
        let plan = outcome.outcome.unwrap().value;
        assert_eq!(plan.selection().input_context().depth, 1);
        assert_eq!(plan.selection().paired_children().next().unwrap().interior.as_ref().unwrap().context.depth, 2);
    }
    run.next_group(&loans).unwrap().unwrap();
    run.next_group(&loans).unwrap().unwrap();
    assert_eq!(run.parts().frames.len(), 3);
    assert_eq!(run.parts().frames[2].lanes[0].context.depth, 3);
    assert!(matches!(&run.heads()[1].1, Head::Child { relative } if relative == Path::new("b")));
    let parts = run.into_parts();
    let mut bad = parts.clone();
    bad.frames[2].lanes[0].context.residual += 1;
    assert!(bad.validate().is_err(), "active member/context equality is structural");
    let calls: Vec<_> = policies.iter().map(|p| p.calls.lock().unwrap().len()).collect();
    let mut run = Coordinated::from_parts(parts, Execution::Sequential).unwrap();
    run.bind(&loans).unwrap();
    for (p, count) in policies.iter().zip(calls) {
        assert_eq!(p.calls.lock().unwrap().len(), count);
    }
    while run.next_group(&loans).unwrap().is_some() {}
    assert!(run.parts().lanes[0].summary.is_clean());
    assert!(!run.parts().lanes[1].summary.is_complete());
    assert_eq!(policies[0].calls.lock().unwrap().iter().filter(|(_, _, root)| *root).count(), 1);
}

#[test]
fn interpretation_and_context_refuse_before_missing_root_io() {
    let fixture = Fixture::new();
    fixture.both_dir("a");
    let policy = ContextPolicy::new(11);
    let binding = TargetBinding::new("lane".into(), fixture.target.clone(), "test/v1".into(), &policy, &StatKey);
    let bindings = Bindings {
        source: fixture.original.clone(),
        targets: vec![binding],
    };
    let loans = [TargetPolicy {
        target: &bindings.targets[0].id,
        descriptor: &bindings.targets[0].policy,
        policy: &policy,
        comparator: StatKey,
    }];
    let mut run = Coordinated::new(bindings.clone(), Execution::Sequential).unwrap();
    run.bind(&loans).unwrap();
    run.next_group(&loans).unwrap().unwrap();
    let parts = run.into_parts();
    std::fs::remove_dir_all(&fixture.original).unwrap();
    for which in 0..4 {
        let mut altered = parts.clone();
        match which {
            0 => altered.bindings.targets[0].policy_id.version += 1,
            1 => altered.bindings.targets[0].context_schema.version += 1,
            2 => {
                altered.bindings.targets[0].comparator.parameters.push(1);
                altered.lanes[0].summary.comparator = Some(altered.bindings.targets[0].comparator.clone());
            }
            _ => {
                altered.frames[0].remaining[0].members[0].context.residual += 1;
            }
        }
        let mut restored = Coordinated::from_parts(altered, Execution::Sequential).unwrap();
        assert!(matches!(restored.bind(&loans), Err(Error::PolicyMismatch(_) | Error::Context { .. })));
        assert_eq!(restored.route_metadata_checks(), 0);
    }
}

#[test]
fn real_comparators_and_equal_renewal_keep_comparison_coverage_separate() {
    let fixture = Fixture::new();
    fixture.both_file("equal-size", b"data");
    set_time(&fixture.target.join("equal-size"), timestamp() + std::time::Duration::from_secs(1));
    let mut policy = ContextPolicy::new(1);
    policy.renew = true;
    for size_only in [false, true] {
        // A closed enum is a supported heterogeneous comparator, not a registry.
        enum Rule {
            Stat,
            Size,
        }
        impl Comparator for Rule {
            fn identity(&self) -> &SemanticId {
                match self {
                    Self::Stat => StatKey.identity(),
                    Self::Size => SizeAndLink.identity(),
                }
            }
            fn demand(&self, shape: crate::Shape) -> crate::SideDemands {
                match self {
                    Self::Stat => StatKey.demand(shape),
                    Self::Size => SizeAndLink.demand(shape),
                }
            }
            fn compare(&self, input: crate::ComparisonInput<'_>) -> Result<crate::Comparison, crate::CheckError> {
                match self {
                    Self::Stat => StatKey.compare(input),
                    Self::Size => SizeAndLink.compare(input),
                }
            }
        }
        let rule = if size_only { Rule::Size } else { Rule::Stat };
        let binding = TargetBinding::new("lane".into(), fixture.target.clone(), "renew/v1".into(), &policy, &rule);
        let bindings = Bindings {
            source: fixture.original.clone(),
            targets: vec![binding],
        };
        let loans = [TargetPolicy {
            target: &bindings.targets[0].id,
            descriptor: &bindings.targets[0].policy,
            policy: &policy,
            comparator: &rule,
        }];
        let mut run = Coordinated::new(bindings.clone(), Execution::Sequential).unwrap();
        run.bind(&loans).unwrap();
        let group = run.next_group_with(&loans, |_, p| p).unwrap().unwrap();
        let selected = group.outcomes[0].outcome.as_ref().unwrap().value.selection();
        assert_eq!(selected.diff().is_empty(), size_only);
        assert_eq!(selected.coverage().application_need, crate::ApplicationNeed::LocalOrOpaque);
        assert!(run.next_group(&loans).unwrap().is_none());
        let summary = &run.parts().lanes[0].summary;
        assert_eq!(summary.is_clean(), size_only);
        assert_eq!(summary.deviated, u64::from(size_only));
        assert_eq!(summary.planned, 1);
        assert_eq!(summary.comparator.as_ref(), Some(rule.identity()));
    }
}

#[derive(Clone, Copy)]
struct InstallInputs;
impl ExecutionDemand for InstallInputs {
    fn for_entry<C>(&self, entry: crate::EntrySelectionRef<'_, C>) -> crate::SideDemands {
        crate::SideDemands {
            source: crate::Demand {
                attributes: entry.action == Action::InstallLeaf,
                literal_link: false,
            },
            target: crate::Demand::NONE,
        }
    }
}

#[test]
fn initial_sparse_publication_completes_execution_demand_without_comparator_facts() {
    let fixture = Fixture::new();
    fixture.both_file("work", b"data");
    let mut policy = ContextPolicy::new(3);
    policy.renew = true;
    policy.omit = true;
    let bindings = Bindings {
        source: fixture.original.clone(),
        targets: vec![TargetBinding::new(
            "lane".into(),
            fixture.target.clone(),
            "omitted-renew/v1".into(),
            &policy,
            &SizeAndLink,
        )],
    };
    let loans = [TargetPolicy {
        target: &bindings.targets[0].id,
        descriptor: &bindings.targets[0].policy,
        policy: &policy,
        comparator: SizeAndLink,
    }];
    for requested in [false, true] {
        let mut run = Coordinated::new(bindings.clone(), Execution::Sequential).unwrap();
        run.bind(&loans).unwrap();
        let group = if requested {
            run.next_group_prepared(&loans, &InstallInputs, |_, p| p)
        } else {
            run.next_group_with(&loans, |_, p| p)
        }
        .unwrap()
        .unwrap();
        let prepared = &group.outcomes[0].outcome.as_ref().unwrap().value;
        let facts = prepared.facts(crate::SlotId(0));
        assert_eq!(matches!(facts.source.unwrap().attributes, crate::Available::Known(_)), requested);
        assert!(matches!(facts.target.unwrap().attributes, crate::Available::NotRequested));
        assert_eq!(prepared.selection().coverage().not_assessed, 1);
        assert!(prepared.selection().diff().is_empty());
        assert_eq!(prepared.selection().coverage().application_need, crate::ApplicationNeed::LocalOrOpaque);
    }
}

#[test]
fn typed_plan_walk_publishes_clean_work_with_inputs_and_exact_child_context() {
    let fixture = Fixture::new();
    fixture.both_dir("a/deep");
    fixture.both_file("work", b"data");
    fixture.both_file("a/deep/work", b"data");
    let mut policy = ContextPolicy::new(5);
    policy.renew = true;
    let source = Arc::new(SourceSession::new(&fixture.original));
    let mut walk = crate::walk_with_source(
        source.clone(),
        &fixture.target,
        crate::WalkOptions::default(),
        &policy,
        SizeAndLink,
        InstallInputs,
    )
    .unwrap();
    assert!(Arc::ptr_eq(walk.source_session(), &source));
    let plan = walk.next().unwrap().unwrap();
    assert!(plan.selection().diff().is_empty());
    assert_eq!(plan.selection().input_context().depth, 1);
    assert_eq!(plan.source_generation(), source.generation());
    let source_facts = plan.facts(crate::SlotId(1)).source.unwrap();
    assert!(matches!(source_facts.attributes, crate::Available::Known(_)));
    assert!(policy.calls.lock().unwrap().iter().all(|(path, _, _)| path.as_os_str().is_empty()));
    let deep = walk.next().unwrap().unwrap();
    assert_eq!(deep.selection().relative(), Path::new("a/deep"));
    assert_eq!(deep.selection().input_context().depth, 3);
    assert_eq!(deep.source_generation(), source.generation());
    assert!(walk.next().is_none());
    let summary = walk.into_summary();
    assert!(summary.is_clean());
    assert_eq!(summary.planned, 2);
    assert_eq!(summary.deviated, 2);
}

#[cfg(feature = "parallel")]
#[test]
fn clean_work_seals_parallel_children_until_the_next_joined_call() {
    let fixture = Fixture::new();
    fixture.both_dir("a/deep");
    fixture.both_file("work", b"data");
    fixture.both_file("a/deep/work", b"data");
    let mut policy = ContextPolicy::new(5);
    policy.renew = true;
    let pool = rayon::ThreadPoolBuilder::new().num_threads(3).build().unwrap();
    let mut walk = crate::walk_in_pool_policy(
        &fixture.original,
        &fixture.target,
        &pool,
        crate::ParallelLimits::for_pool(&pool),
        &policy,
        SizeAndLink,
        InstallInputs,
    )
    .unwrap();
    let root = walk.next().unwrap().unwrap();
    assert!(root.selection().diff().is_empty());
    assert!(
        policy.calls.lock().unwrap().iter().all(|(path, _, _)| path.as_os_str().is_empty()),
        "clean selected work must seal descendants before speculation"
    );
    assert!(walk.prune_here());
    assert!(walk.next().is_none());
    assert!(!walk.into_summary().is_complete());
}
