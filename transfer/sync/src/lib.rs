//! Explicit filesystem synchronization under caller-owned stable trees.
#![doc = include_str!("../README.md")]

mod apply;
mod authority;
mod binding;
pub mod coordinated;
mod kernel;
mod listed;
mod model;
mod opaque;
mod paths;
mod platform;
mod seed;
mod visitor;

pub use apply::{PhysicalExecutionDemand, clear, clear_with_visitor};
pub use authority::{Absence, NeverRemove, RemoveAll};
pub use binding::EffectBinding;
pub use coordinated::{SyncCursor, SyncRuntime, SyncTarget};
pub use listed::{
    DesiredLeaf, LeafExpectation, LeafKind, ListedCandidate, ListedGuard, ListedIntent, ListedMode, ListedOutcome, ListedRelation, ListedReport, ListedTarget,
    ManifestCoverage, ManifestId, NoListedGuard, OwnedLeafSet, PreparedLeafList, reconcile_listed,
};
pub use model::{CopyPolicy, Failure, Issue, Policy, Progress, Report};
pub use seed::seed_leaves;
pub use visitor::{Prune, Seed, Steering, Update};

use diff_dirs_tree_inspect::{
    Check, Comparator, DirDiff, FullVisitor, NoCheck, NoScope, NoVisitor, Policy as SelectionPolicy, Scope, SourceSession, StatKey, Visitor,
};
use std::path::Path;
use std::sync::Arc;

/// Owned compatibility interpretation, in check/scope/absence/late-visitor order.
pub type FullPolicy<C = NoCheck, S = NoScope, A = RemoveAll, V = NoVisitor> = FullVisitor<Steering<C, S, A, V>>;

/// Validated, non-overlapping logical roots plus executable interpretation. Live
/// source resources belong to a run/cursor, never to serialized selections.
#[derive(Debug)]
pub struct Applier<P = FullPolicy, M = StatKey> {
    binding: EffectBinding,
    selection: P,
    comparator: M,
    replace_root: bool,
}

impl<P, M> std::ops::Deref for Applier<P, M> {
    type Target = EffectBinding;
    fn deref(&self) -> &EffectBinding {
        &self.binding
    }
}
impl<P, M> Applier<P, M> {
    pub fn binding(&self) -> &EffectBinding {
        &self.binding
    }
}

impl Applier {
    pub fn new(original: impl AsRef<Path>, target: impl AsRef<Path>, policy: CopyPolicy) -> Result<Self, Failure> {
        Self::new_checked(original, target, policy, NoCheck)
    }
}
impl<C: Check> Applier<FullPolicy<C>> {
    pub fn new_checked(original: impl AsRef<Path>, target: impl AsRef<Path>, policy: CopyPolicy, check: C) -> Result<Self, Failure> {
        Self::new_policy(
            original,
            target,
            policy,
            FullVisitor(Steering {
                check,
                scope: NoScope,
                absence: RemoveAll,
                visitor: NoVisitor,
            }),
            StatKey,
        )
    }
    pub fn new_replacing_checked(original: impl AsRef<Path>, target: impl AsRef<Path>, policy: CopyPolicy, check: C) -> Result<Self, Failure> {
        Self::new_replacing_policy(
            original,
            target,
            policy,
            FullVisitor(Steering {
                check,
                scope: NoScope,
                absence: RemoveAll,
                visitor: NoVisitor,
            }),
            StatKey,
        )
    }
}

impl<P: SelectionPolicy, M: Comparator> Applier<P, M> {
    pub fn new_policy(original: impl AsRef<Path>, target: impl AsRef<Path>, copy: CopyPolicy, policy: P, comparator: M) -> Result<Self, Failure> {
        Self::bind(original.as_ref(), target.as_ref(), copy, policy, comparator, false)
    }
    pub fn new_replacing_policy(original: impl AsRef<Path>, target: impl AsRef<Path>, copy: CopyPolicy, policy: P, comparator: M) -> Result<Self, Failure> {
        Self::bind(original.as_ref(), target.as_ref(), copy, policy, comparator, true)
    }
    fn bind(original: &Path, target: &Path, copy: CopyPolicy, policy: P, comparator: M, replace_root: bool) -> Result<Self, Failure> {
        let binding = EffectBinding::bind(original, target, copy, replace_root)?;
        Ok(Self {
            binding,
            selection: policy,
            comparator,
            replace_root,
        })
    }
    pub fn original(&self) -> &Path {
        &self.original
    }
    pub fn target(&self) -> &Path {
        &self.target
    }
    pub fn selection_policy(&self) -> &P {
        &self.selection
    }
    pub fn comparator(&self) -> &M {
        &self.comparator
    }
    pub fn apply(&self, diff: &DirDiff) -> Result<Progress, Failure> {
        self.apply_report(diff).map(|r| r.completed)
    }
    pub fn initialize(&self) -> Result<Progress, Failure> {
        self.initialize_reported(false).map(|r| r.completed)
    }
    pub fn replace_root(&self) -> Result<Progress, Failure> {
        self.replace_root_reported(false).map(|r| r.completed)
    }
    pub fn mirror(&self) -> Result<Report, Failure> {
        self.run()
    }

    /// One streaming paired frontier plus opaque selected interiors. Initial
    /// publication acquires execution inputs, including work in comparison-clean
    /// batches. No audit or second driver precedes execution.
    pub fn run(&self) -> Result<Report, Failure> {
        let source = Arc::new(SourceSession::new(&self.original));
        if let Some(report) = self.whole_root(&source, false)? {
            return Ok(report);
        }
        let mut walk = diff_dirs_tree_compare::walk_with_source(
            Arc::clone(&source),
            &self.target,
            Default::default(),
            &self.selection,
            &self.comparator,
            PhysicalExecutionDemand,
        )
        .map_err(Failure::comparison)?;
        let mut accumulated = Report {
            planned: 0,
            completed: Progress::default(),
            policy: Policy::default(),
            observed: None,
        };
        let epoch = diff_dirs_tree_inspect::InspectionEpoch::new();
        while let Some(item) = walk.next() {
            let result = item
                .map_err(Failure::comparison)
                .and_then(|prepared| self.apply_walk_ready(&source, &epoch, prepared, false));
            if let Err(mut failure) = accumulate(&mut accumulated, result) {
                let observed = walk.into_summary();
                failure.policy.deviated += observed.deviated;
                failure.policy.retained += observed.retained;
                failure.observed = Some(Box::new(observed));
                return Err(failure);
            }
        }
        let observed = walk.into_summary();
        accumulated.policy.deviated += observed.deviated;
        accumulated.policy.retained += observed.retained;
        accumulated.policy.satisfied = true;
        accumulated.observed = Some(observed);
        Ok(accumulated)
    }

    #[cfg(feature = "parallel")]
    pub fn apply_in_pool(&self, diff: &DirDiff, pool: &rayon::ThreadPool) -> Result<Progress, Failure> {
        pool.install(|| self.apply_public(diff, true, None).map(|r| r.completed))
    }
    #[cfg(feature = "parallel")]
    pub fn initialize_in_pool(&self, pool: &rayon::ThreadPool) -> Result<Progress, Failure> {
        pool.install(|| self.initialize_reported(true).map(|r| r.completed))
    }
    #[cfg(feature = "parallel")]
    pub fn replace_root_in_pool(&self, pool: &rayon::ThreadPool) -> Result<Progress, Failure> {
        pool.install(|| self.replace_root_reported(true).map(|r| r.completed))
    }
    #[cfg(feature = "parallel")]
    pub fn mirror_in_pool(&self, pool: &rayon::ThreadPool, limits: diff_dirs_tree_compare::ParallelLimits) -> Result<Report, Failure> {
        self.run_in_pool(pool, limits)
    }
    #[cfg(feature = "parallel")]
    pub fn run_in_pool(&self, pool: &rayon::ThreadPool, limits: diff_dirs_tree_compare::ParallelLimits) -> Result<Report, Failure> {
        let source = Arc::new(SourceSession::new(&self.original));
        if let Some(report) = pool.install(|| self.whole_root(&source, true))? {
            return Ok(report);
        }
        let mut walk = diff_dirs_tree_compare::walk_in_pool_with_source(
            Arc::clone(&source),
            &self.target,
            pool,
            limits,
            &self.selection,
            &self.comparator,
            PhysicalExecutionDemand,
        )
        .map_err(Failure::comparison)?;
        // The borrowed-pool walk owns the same session used by every opaque call.
        let source = Arc::clone(walk.source_session());
        let mut accumulated = Report {
            planned: 0,
            completed: Progress::default(),
            policy: Policy::default(),
            observed: None,
        };
        let epoch = diff_dirs_tree_inspect::InspectionEpoch::new();
        while let Some(item) = walk.next() {
            let result = item
                .map_err(Failure::comparison)
                .and_then(|prepared| pool.install(|| self.apply_walk_ready(&source, &epoch, prepared, true)));
            if let Err(mut failure) = accumulate(&mut accumulated, result) {
                let observed = walk.into_summary();
                failure.policy.deviated += observed.deviated;
                failure.policy.retained += observed.retained;
                failure.observed = Some(Box::new(observed));
                return Err(failure);
            }
        }
        let observed = walk.into_summary();
        accumulated.policy.deviated += observed.deviated;
        accumulated.policy.retained += observed.retained;
        accumulated.policy.satisfied = true;
        accumulated.observed = Some(observed);
        Ok(accumulated)
    }
}

fn accumulate(total: &mut Report, result: Result<Report, Failure>) -> Result<(), Failure> {
    match result {
        Ok(r) => {
            total.planned += r.planned;
            total.completed += r.completed;
            total.policy += r.policy;
            Ok(())
        }
        Err(mut f) => {
            f.planned += total.planned;
            f.completed += total.completed.clone();
            *f.policy += total.policy.clone();
            Err(f)
        }
    }
}

impl<C: Check, S: Scope, A: Absence, V: Visitor> Applier<FullPolicy<C, S, A, V>> {
    pub fn scope<T: Scope>(self, scope: T) -> Applier<FullPolicy<C, T, A, V>> {
        let Steering { check, absence, visitor, .. } = self.selection.0;
        Applier {
            binding: self.binding,
            selection: FullVisitor(Steering {
                check,
                scope,
                absence,
                visitor,
            }),
            comparator: StatKey,
            replace_root: self.replace_root,
        }
    }
    pub fn absence<B: Absence>(self, absence: B) -> Applier<FullPolicy<C, S, B, V>> {
        let Steering { check, scope, visitor, .. } = self.selection.0;
        Applier {
            binding: self.binding,
            selection: FullVisitor(Steering {
                check,
                scope,
                absence,
                visitor,
            }),
            comparator: StatKey,
            replace_root: self.replace_root,
        }
    }
    pub fn visitor<W: Visitor>(self, visitor: W) -> Applier<FullPolicy<C, S, A, W>> {
        let Steering { check, scope, absence, .. } = self.selection.0;
        Applier {
            binding: self.binding,
            selection: FullVisitor(Steering {
                check,
                scope,
                absence,
                visitor,
            }),
            comparator: StatKey,
            replace_root: self.replace_root,
        }
    }
    pub fn check(&self) -> &C {
        &self.selection.0.check
    }
    pub fn scope_policy(&self) -> &S {
        &self.selection.0.scope
    }
    pub fn audit(&self) -> Result<AuditReport, AuditFailure> {
        let mut report = AuditReport::default();
        for (path, side) in [
            (&self.original, diff_dirs_tree_inspect::Side::Original),
            (&self.target, diff_dirs_tree_inspect::Side::Target),
        ] {
            if side == diff_dirs_tree_inspect::Side::Target
                && !paths::exists(path).map_err(|f| AuditFailure {
                    error: diff_dirs_tree_inspect::Error::Io {
                        path: path.clone(),
                        operation: "inspect audit target",
                        source: std::io::Error::other(f),
                    },
                    inspected: Box::new(report.clone()),
                })?
            {
                break;
            }
            match diff_dirs_tree_compare::audit_with_visitor(path, side, &self.selection.0) {
                Ok(a) => {
                    if side == diff_dirs_tree_inspect::Side::Original {
                        report.original = a
                    } else {
                        report.target = a
                    }
                }
                Err(f) => {
                    if side == diff_dirs_tree_inspect::Side::Original {
                        report.original = f.inspected
                    } else {
                        report.target = f.inspected
                    };
                    return Err(AuditFailure {
                        error: f.error,
                        inspected: Box::new(report),
                    });
                }
            }
        }
        Ok(report)
    }
    pub fn apply_plan(&self, plan: &diff_dirs_tree_inspect::DirectoryPlan) -> Result<Report, Failure> {
        self.apply_prepared(plan)
    }
    fn apply_prepared(&self, plan: &diff_dirs_tree_inspect::DirectoryPlan) -> Result<Report, Failure> {
        if plan.operations().source_root() != self.original || plan.operations().target_root() != self.target || plan.selection().is_root() {
            return Err(Failure::invalid(
                "apply plan",
                plan.selection().relative(),
                "plan does not belong to these directory roots",
            ));
        }
        self.apply_public(plan.diff(), false, Some(plan.selection()))
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AuditReport {
    pub original: diff_dirs_tree_compare::Audit,
    pub target: diff_dirs_tree_compare::Audit,
}
#[derive(Debug)]
pub struct AuditFailure {
    pub error: diff_dirs_tree_inspect::Error,
    pub inspected: Box<AuditReport>,
}
impl std::fmt::Display for AuditFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.error.fmt(f)
    }
}
impl std::error::Error for AuditFailure {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.error)
    }
}

pub fn mirror(original: impl AsRef<Path>, target: impl AsRef<Path>, policy: CopyPolicy) -> Result<Report, Failure> {
    Applier::new(original, target, policy)?.run()
}
pub fn mirror_checked<C: Check>(original: impl AsRef<Path>, target: impl AsRef<Path>, policy: CopyPolicy, check: C) -> Result<Report, Failure> {
    Applier::new_checked(original, target, policy, check)?.run()
}
pub fn update(original: impl AsRef<Path>, target: impl AsRef<Path>, policy: CopyPolicy) -> Result<Report, Failure> {
    Applier::new(original, target, policy)?.visitor(Update).run()
}
#[cfg(feature = "parallel")]
pub fn mirror_in_pool(
    original: impl AsRef<Path>,
    target: impl AsRef<Path>,
    policy: CopyPolicy,
    pool: &rayon::ThreadPool,
    limits: diff_dirs_tree_compare::ParallelLimits,
) -> Result<Report, Failure> {
    Applier::new(original, target, policy)?.run_in_pool(pool, limits)
}
