//! Production source-to-many reconciliation with a current-group decision barrier.
//! Comparison owns paired traversal; the existing sync kernel owns opaque
//! one-sided/replacement interiors. Save only between returned calls.

mod decision;
mod state;

use std::collections::HashSet;
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use diff_dirs_tree_compare::coordinated::{
    self as compare, Bindings, Compared, ComparedGroup, Coordinated, GroupToken, LaneStatus, TargetBinding, TargetId, TargetOutcome, TargetPolicy,
};
use diff_dirs_tree_inspect::{Check, DirectoryPlan, Execution, NoCheck, NoScope, NoVisitor, Scope, SharedRoots, TargetVisitor, Visitor};

use crate::{Absence, Applier, CopyPolicy, Failure, RemoveAll, Report, paths};
pub use decision::{DirectoryDecision, EntryDecision};
pub use diff_dirs_tree_inspect::Kind;
pub use state::{Application, ApplicationState, Barrier, End, LaneTotals, SyncParts, WorkTotals};

#[derive(Debug)]
pub enum RefreshIssue {
    Changed {
        target: TargetId,
        relative: PathBuf,
    },
    Inspection {
        target: TargetId,
        error: Arc<diff_dirs_tree_inspect::Error>,
    },
    Route {
        target: TargetId,
        failure: Box<Failure>,
    },
}

#[derive(Debug)]
pub enum Error {
    InvalidState(&'static str),
    InvalidControl(&'static str),
    Comparison(compare::Error),
    Binding(Box<Failure>),
    RuntimeMismatch(TargetId),
    PendingApplications,
    NotBound,
    /// All refresh decisions were checked before any refreshed application. Bad
    /// lanes are terminal; other freshly validated lanes remain pending.
    Refresh(Vec<RefreshIssue>),
    Application {
        target: TargetId,
        failure: Box<Failure>,
    },
    Observed {
        target: TargetId,
        error: Arc<diff_dirs_tree_inspect::Error>,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidState(r) => write!(f, "invalid sync state: {r}"),
            Self::InvalidControl(r) => write!(f, "invalid sync control: {r}"),
            Self::Comparison(e) => e.fmt(f),
            Self::Binding(e) => e.fmt(f),
            Self::RuntimeMismatch(t) => write!(f, "missing, duplicate or incompatible sync runtime for {:?}", t.0),
            Self::PendingApplications => f.write_str("dispose current applications before advancing comparison"),
            Self::NotBound => f.write_str("bind sync runtime before filesystem work"),
            Self::Refresh(issues) => write!(f, "resumed decision refresh refused {} lane(s): {issues:?}", issues.len()),
            Self::Application { target, failure } => write!(f, "apply {:?}: {failure}", target.0),
            Self::Observed { target, error } => write!(f, "compare {:?}: {error}", target.0),
        }
    }
}
impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Comparison(e) => Some(e),
            Self::Binding(e) | Self::Application { failure: e, .. } => Some(e.as_ref()),
            Self::Observed { error, .. } => Some(error.as_ref()),
            _ => None,
        }
    }
}

pub(super) fn require(ok: bool, reason: &'static str) -> Result<(), Error> {
    if ok { Ok(()) } else { Err(Error::InvalidState(reason)) }
}

pub(super) fn disjoint(bindings: &Bindings) -> Result<(), Error> {
    let roots: Vec<_> = std::iter::once(&bindings.source).chain(bindings.targets.iter().map(|t| &t.root)).collect();
    for (i, root) in roots.iter().enumerate() {
        for other in &roots[..i] {
            require(
                !root.starts_with(other) && !other.starts_with(root),
                "source/target or cross-target roots overlap",
            )?;
        }
    }
    Ok(())
}

/// A production binding plus its frozen callback identity/version/parameters.
/// The descriptor is a caller promise, not executable policy introspection.
pub struct SyncTarget<C = NoCheck, S = NoScope, A = RemoveAll, V = NoVisitor> {
    pub id: TargetId,
    pub descriptor: String,
    pub applier: Applier<C, S, A, V>,
}

/// Owns executable policies, not traversal. Temporary Steering loans exist only
/// during each call; neither this object nor SyncCursor is self-referential.
pub struct SyncRuntime<C = NoCheck, S = NoScope, A = RemoveAll, V = NoVisitor> {
    targets: Vec<SyncTarget<C, S, A, V>>,
}

impl<C: Check, S: Scope, A: Absence, V: Visitor> SyncRuntime<C, S, A, V> {
    pub fn new(targets: Vec<SyncTarget<C, S, A, V>>) -> Self {
        Self { targets }
    }

    fn target(&self, id: &TargetId) -> Result<&SyncTarget<C, S, A, V>, Error> {
        self.targets.iter().find(|t| t.id == *id).ok_or_else(|| Error::RuntimeMismatch(id.clone()))
    }

    fn bindings(&self) -> Result<Bindings, Error> {
        let first = self.targets.first().ok_or(Error::InvalidState("fresh sync needs targets"))?;
        require(
            self.targets.iter().all(|t| t.applier.original() == first.applier.original()),
            "runtime sources differ",
        )?;
        let bindings = Bindings {
            source: first.applier.original().to_owned(),
            targets: self
                .targets
                .iter()
                .map(|t| TargetBinding {
                    id: t.id.clone(),
                    root: t.applier.target().to_owned(),
                    policy: t.descriptor.clone(),
                })
                .collect(),
        };
        bindings.validate().map_err(Error::Comparison)?;
        disjoint(&bindings)?;
        Ok(bindings)
    }

    fn with_policies<T>(&self, call: impl FnOnce(&[TargetPolicy<'_, crate::visitor::Steering<'_, C, S, A, V>>]) -> T) -> T {
        let steering: Vec<_> = self.targets.iter().map(|t| t.applier.steering()).collect();
        let policies: Vec<_> = self
            .targets
            .iter()
            .zip(&steering)
            .map(|(t, visitor)| TargetPolicy {
                target: &t.id,
                descriptor: &t.descriptor,
                visitor,
            })
            .collect();
        call(&policies)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LaneReport {
    pub target: TargetId,
    pub totals: LaneTotals,
    pub observed: diff_dirs_tree_compare::Summary,
    pub retained: u128,
    pub satisfied: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Limits {
    pub groups: u64,
    pub applications: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stop {
    Budget,
    Done,
}

#[derive(Debug)]
pub struct Drive {
    pub groups: u64,
    /// Application requests, including a refused restored refresh. One request
    /// can recursively copy/remove a whole one-sided subtree.
    pub applications: u64,
    pub stop: Stop,
    pub failures: Vec<Error>,
}

/// Owned traversal and decisions. Full plans are an expendable runtime cache;
/// parts()/into_parts() never contain them. Do not resume forked old parts while
/// also advancing this lineage: effects/publication are not exactly-once.
pub struct SyncCursor {
    comparison: Coordinated,
    copy_policies: Vec<CopyPolicy>,
    lanes: Vec<LaneTotals>,
    current: Option<Barrier>,
    plans: Vec<Option<DirectoryPlan>>,
    execution: Execution,
    bound: bool,
}

impl SyncCursor {
    /// All roots must already be real directories. No whole-root initialization
    /// or replacement is introduced by this coordinated composition.
    pub fn new<C: Check, S: Scope, A: Absence, V: Visitor>(runtime: &SyncRuntime<C, S, A, V>, execution: Execution) -> Result<Self, Error> {
        let bindings = runtime.bindings()?;
        let n = bindings.targets.len();
        let parts = SyncParts {
            version: 2,
            comparison: compare::Parts::fresh(bindings).map_err(Error::Comparison)?,
            copy_policies: runtime.targets.iter().map(|t| t.applier.policy).collect(),
            lanes: vec![LaneTotals::default(); n],
            current: None,
        };
        let mut cursor = Self::from_parts(parts, execution)?;
        cursor.bind(runtime)?;
        Ok(cursor)
    }

    /// Strictly data-only, including terminal states whose roots no longer exist.
    pub fn from_parts(parts: SyncParts, execution: Execution) -> Result<Self, Error> {
        parts.validate()?;
        let n = parts.lanes.len();
        Ok(Self {
            comparison: Coordinated::from_parts(parts.comparison, execution).map_err(Error::Comparison)?,
            copy_policies: parts.copy_policies,
            lanes: parts.lanes,
            current: parts.current,
            plans: (0..n).map(|_| None).collect(),
            execution,
            bound: false,
        })
    }

    pub fn parts(&self) -> SyncParts {
        SyncParts {
            version: 2,
            comparison: self.comparison.parts().clone(),
            copy_policies: self.copy_policies.clone(),
            lanes: self.lanes.clone(),
            current: self.current.clone(),
        }
    }
    pub fn into_parts(self) -> SyncParts {
        SyncParts {
            version: 2,
            comparison: self.comparison.into_parts(),
            copy_policies: self.copy_policies,
            lanes: self.lanes,
            current: self.current,
        }
    }
    pub fn current(&self) -> Option<&Barrier> {
        self.current.as_ref()
    }
    pub fn current_token(&self) -> Option<GroupToken> {
        self.comparison.current_token()
    }
    pub fn heads(&self) -> Vec<(TargetId, compare::Head)> {
        self.comparison.heads()
    }
    pub fn is_terminal(&self) -> bool {
        self.comparison.parts().lanes.iter().all(|l| l.status != LaneStatus::Active)
    }

    /// Pure traversal returns after every current application is disposed. This
    /// invalidates the old control token but never starts another inspection.
    pub fn settle(&mut self) -> Result<(), Error> {
        if !self.pending().is_empty() {
            return Err(Error::PendingApplications);
        }
        self.comparison.settle();
        self.current = None;
        self.plans.iter_mut().for_each(|p| *p = None);
        Ok(())
    }

    fn validate_runtime<C: Check, S: Scope, A: Absence, V: Visitor>(&self, runtime: &SyncRuntime<C, S, A, V>) -> Result<(), Error> {
        let bindings = &self.comparison.parts().bindings;
        let mut seen = HashSet::new();
        for t in &runtime.targets {
            let i = bindings
                .targets
                .iter()
                .position(|b| b.id == t.id)
                .ok_or_else(|| Error::RuntimeMismatch(t.id.clone()))?;
            let b = &bindings.targets[i];
            if !seen.insert(&t.id)
                || b.policy != t.descriptor
                || t.applier.original() != bindings.source
                || t.applier.target() != b.root
                || t.applier.policy != self.copy_policies[i]
            {
                return Err(Error::RuntimeMismatch(t.id.clone()));
            }
        }
        for (i, lane) in self.comparison.parts().lanes.iter().enumerate() {
            if lane.status == LaneStatus::Active && !seen.contains(&bindings.targets[i].id) {
                return Err(Error::RuntimeMismatch(bindings.targets[i].id.clone()));
            }
        }
        Ok(())
    }

    /// Reconstruct execution separately. Descriptor checks precede tree access;
    /// existing ancestors must be real, all resolved roots remain disjoint, and
    /// no root/ancestor visitor decision is replayed.
    pub fn bind<C: Check, S: Scope, A: Absence, V: Visitor>(&mut self, runtime: &SyncRuntime<C, S, A, V>) -> Result<(), Error> {
        self.validate_runtime(runtime)?;
        self.bound = false;
        self.plans.iter_mut().for_each(|p| *p = None);
        if !self.is_terminal() {
            let b = &self.comparison.parts().bindings;
            let source = paths::root(&b.source, false).map_err(|f| Error::Binding(Box::new(f)))?;
            require(source == b.source, "source root no longer has its frozen resolved coordinate")?;
            let mut resolved = Bindings { source, targets: Vec::new() };
            for (i, lane) in self.comparison.parts().lanes.iter().enumerate() {
                if lane.status != LaneStatus::Active {
                    continue;
                }
                let t = &b.targets[i];
                let root = paths::root(&t.root, false).map_err(|f| Error::Binding(Box::new(f)))?;
                require(root == t.root, "target root no longer has its frozen resolved coordinate")?;
                resolved.targets.push(TargetBinding { root, ..t.clone() });
            }
            disjoint(&resolved)?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::MetadataExt;
                let mut identities = HashSet::new();
                for root in std::iter::once(&resolved.source).chain(resolved.targets.iter().map(|t| &t.root)) {
                    let m = paths::directory(root).map_err(|f| Error::Binding(Box::new(f)))?;
                    require(identities.insert((m.dev(), m.ino())), "physical source or cross-target root alias")?;
                }
            }
        }
        runtime.with_policies(|p| self.comparison.bind(p)).map_err(Error::Comparison)?;
        self.bound = true;
        Ok(())
    }

    pub fn reports(&self) -> Vec<LaneReport> {
        self.comparison
            .prefix_summaries()
            .into_iter()
            .enumerate()
            .map(|(i, (target, observed))| LaneReport {
                target,
                retained: self.lanes[i].recursive_retained + u128::from(observed.retained),
                satisfied: self.comparison.parts().lanes[i].status == LaneStatus::Complete && self.lanes[i].end.is_none() && !self.lanes[i].pruned,
                totals: self.lanes[i].clone(),
                observed,
            })
            .collect()
    }

    pub fn pending(&self) -> Vec<TargetId> {
        self.current
            .iter()
            .flat_map(|b| &b.applications)
            .filter(|a| a.state == ApplicationState::Unstarted)
            .map(|a| self.comparison.parts().bindings.targets[a.target].id.clone())
            .collect()
    }

    pub fn next_group<C: Check, S: Scope, A: Absence, V: Visitor>(
        &mut self,
        runtime: &SyncRuntime<C, S, A, V>,
    ) -> Result<Option<ComparedGroup<DirectoryDecision>>, Error> {
        self.validate_runtime(runtime)?;
        if !self.pending().is_empty() {
            return Err(Error::PendingApplications);
        }
        if !self.bound && !self.is_terminal() {
            return Err(Error::NotBound);
        }
        self.current = None;
        self.plans.iter_mut().for_each(|p| *p = None);
        let Some(group) = runtime
            .with_policies(|p| self.comparison.next_group_with(p, |_, plan| plan))
            .map_err(Error::Comparison)?
        else {
            return Ok(None);
        };
        let mut applications = Vec::new();
        let mut outcomes = Vec::new();
        for lane in group.outcomes {
            let i = self
                .comparison
                .parts()
                .bindings
                .targets
                .iter()
                .position(|t| t.id == lane.target)
                .expect("published target");
            let outcome = match lane.outcome {
                Err(error) => {
                    self.lanes[i].end = Some(End::ComparisonFailed);
                    Err(error)
                }
                Ok(compared) => {
                    let decision = Arc::clone(compared.value.selection());
                    let state = if compared.structural_root {
                        ApplicationState::Accepted
                    } else {
                        self.plans[i] = Some(compared.value);
                        ApplicationState::Unstarted
                    };
                    applications.push(Application {
                        target: i,
                        decision: decision.clone(),
                        state,
                    });
                    Ok(Compared {
                        diff: compared.diff,
                        coverage: compared.coverage,
                        structural_root: compared.structural_root,
                        value: decision,
                    })
                }
            };
            outcomes.push(TargetOutcome { target: lane.target, outcome });
        }
        self.current = Some(Barrier {
            sequence: self.comparison.parts().window.as_ref().expect("new group window").sequence,
            applications,
        });
        Ok(Some(ComparedGroup {
            token: group.token,
            relative: group.relative,
            outcomes,
        }))
    }

    pub fn prune_here(&mut self, token: &GroupToken, targets: &[TargetId]) -> Result<(), Error> {
        self.comparison.prune_here(token, targets).map_err(Error::Comparison)?;
        for id in targets {
            let i = self.index(id)?;
            self.lanes[i].pruned = true;
        }
        Ok(())
    }
    pub fn stop_lanes(&mut self, targets: &[TargetId]) -> Result<(), Error> {
        self.comparison.stop_lanes(targets).map_err(Error::Comparison)?;
        for id in targets {
            let i = self.index(id)?;
            self.lanes[i].end = Some(End::Abandoned);
            if let Some(app) = self.current.as_mut().and_then(|b| b.applications.iter_mut().find(|a| a.target == i))
                && app.state == ApplicationState::Unstarted
            {
                app.state = ApplicationState::Abandoned;
            }
            self.plans[i] = None;
        }
        Ok(())
    }
    fn index(&self, id: &TargetId) -> Result<usize, Error> {
        self.comparison
            .parts()
            .bindings
            .targets
            .iter()
            .position(|b| b.id == *id)
            .ok_or(Error::InvalidControl("unknown target"))
    }

    fn fail(&mut self, index: usize, end: End) {
        let id = self.comparison.parts().bindings.targets[index].id.clone();
        self.comparison.stop_lanes(&[id]).expect("failing active application lane");
        self.lanes[index].end = Some(end);
        self.current
            .as_mut()
            .expect("application barrier")
            .applications
            .iter_mut()
            .find(|a| a.target == index)
            .expect("current application")
            .state = ApplicationState::Failed;
        self.plans[index] = None;
    }

    /// Restore only unstarted decisions, in a single source-sharing epoch. Check
    /// every refreshed decision before permitting any refreshed effect. Failed
    /// lanes become terminal; valid peers retain their fresh plans for dispatch.
    pub fn refresh<C: Check, S: Scope, A: Absence, V: Visitor>(&mut self, runtime: &SyncRuntime<C, S, A, V>) -> Result<(), Error> {
        self.validate_runtime(runtime)?;
        if !self.bound && !self.is_terminal() {
            return Err(Error::NotBound);
        }
        let pending: Vec<_> = self
            .current
            .iter()
            .flat_map(|b| &b.applications)
            .filter(|a| a.state == ApplicationState::Unstarted)
            .map(|a| a.target)
            .collect();
        if pending.is_empty() || pending.iter().all(|&i| self.plans[i].is_some()) {
            return Ok(());
        }
        // Never mix a saved plan cache with freshly reconstructed peers.
        for &i in &pending {
            self.plans[i] = None;
        }
        let relative = self
            .current
            .as_ref()
            .expect("pending barrier")
            .applications
            .iter()
            .find(|a| a.state == ApplicationState::Unstarted)
            .expect("pending application")
            .decision
            .relative()
            .to_owned();
        let mut issues = Vec::new();
        let mut ready = Vec::new();
        for i in pending {
            match self.route(i, &relative) {
                Ok(()) => ready.push(i),
                Err(failure) => {
                    issues.push(RefreshIssue::Route {
                        target: self.comparison.parts().bindings.targets[i].id.clone(),
                        failure: Box::new(failure),
                    });
                    self.fail(i, End::RefreshFailed);
                }
            }
        }
        if !ready.is_empty() {
            let b = &self.comparison.parts().bindings;
            let roots = SharedRoots::new(&b.source, ready.iter().map(|&i| &b.targets[i].root)).map_err(|e| {
                Error::Comparison(match e {
                    diff_dirs_tree_inspect::SharedBindError::NoTargets => compare::Error::InvalidState("empty refresh"),
                    diff_dirs_tree_inspect::SharedBindError::Source(e) => compare::Error::Source(Arc::new(e)),
                    diff_dirs_tree_inspect::SharedBindError::Target { target, error } => compare::Error::Target {
                        target: b.targets[ready[target]].id.clone(),
                        error: Arc::new(error),
                    },
                })
            })?;
            let steering: Vec<_> = ready
                .iter()
                .map(|&i| runtime.target(&b.targets[i].id).map(|t| t.applier.steering()))
                .collect::<Result<_, _>>()?;
            let requests: Vec<_> = steering.iter().enumerate().map(|(target, visitor)| TargetVisitor { target, visitor }).collect();
            let outcomes = roots
                .inspect_admitted_with(&relative, &requests, self.execution, |_, plan| plan)
                .map_err(|e| Error::Comparison(compare::Error::Inspection(e)))?;
            for outcome in outcomes {
                let i = ready[outcome.target];
                match outcome.outcome {
                    Err(error) => {
                        issues.push(RefreshIssue::Inspection {
                            target: self.comparison.parts().bindings.targets[i].id.clone(),
                            error,
                        });
                        self.fail(i, End::RefreshFailed);
                    }
                    Ok(plan) => {
                        let expected = &self
                            .current
                            .as_ref()
                            .expect("barrier")
                            .applications
                            .iter()
                            .find(|a| a.target == i)
                            .expect("pending application")
                            .decision;
                        if plan.selection() != expected {
                            issues.push(RefreshIssue::Changed {
                                target: self.comparison.parts().bindings.targets[i].id.clone(),
                                relative: relative.clone(),
                            });
                            self.fail(i, End::DecisionChanged);
                        } else {
                            self.plans[i] = Some(plan);
                        }
                    }
                }
            }
        }
        if issues.is_empty() { Ok(()) } else { Err(Error::Refresh(issues)) }
    }

    fn route(&self, target: usize, relative: &Path) -> Result<(), Failure> {
        let b = &self.comparison.parts().bindings;
        for root in [&b.source, &b.targets[target].root] {
            // Also check absolute ancestors of the frozen root, not just the
            // relative suffix; read_dir alone would follow an intermediate link.
            let resolved = paths::root(root, false)?;
            if resolved != *root {
                return Err(Failure::invalid("validate sync route", root, "resolved root changed"));
            }
            let mut path = root.clone();
            for part in relative.components() {
                path.push(part);
                paths::directory(&path)?;
            }
        }
        Ok(())
    }

    /// Consume the cursor's own pending plan, never a caller-supplied diff. A
    /// returned kernel failure is accepted once and terminates this lane. No
    /// operation-internal progress is retained for retry.
    pub fn apply<C: Check, S: Scope, A: Absence, V: Visitor>(
        &mut self,
        runtime: &SyncRuntime<C, S, A, V>,
        token: &GroupToken,
        target: &TargetId,
    ) -> Result<Report, Error> {
        if self.current_token().as_ref() != Some(token) {
            return Err(Error::InvalidControl("stale or foreign group token"));
        }
        let i = self.index(target)?;
        let app = self
            .current
            .as_ref()
            .and_then(|b| b.applications.iter().find(|a| a.target == i))
            .ok_or(Error::InvalidControl("target not in current group"))?;
        if app.state != ApplicationState::Unstarted {
            return Err(Error::InvalidControl("application already disposed"));
        }
        self.refresh(runtime)?;
        let app = self
            .current
            .as_ref()
            .expect("barrier")
            .applications
            .iter()
            .find(|a| a.target == i)
            .expect("pending target");
        let retained = app.decision.coverage().deviated;
        let relative = app.decision.relative().to_owned();
        if let Err(failure) = self.route(i, &relative) {
            self.fail(i, End::ApplicationFailed);
            return Err(Error::Application {
                target: target.clone(),
                failure: Box::new(failure),
            });
        }
        // The saved bound on totals plus this pre-effect call-count increment
        // guarantees u128 cumulative sums can hold every returned u64 counter.
        self.lanes[i].applications = self.lanes[i]
            .applications
            .checked_add(1)
            .ok_or(Error::InvalidState("application count overflow"))?;
        let plan = self.plans[i].take().expect("fresh validated plan");
        #[cfg(feature = "parallel")]
        let parallel = self.execution == Execution::Parallel;
        #[cfg(not(feature = "parallel"))]
        let parallel = false;
        let result = runtime.target(target)?.applier.apply_plan_with(&plan, parallel);
        let totals = &mut self.lanes[i];
        let (planned, completed, policy) = match &result {
            Ok(r) => (r.planned, &r.completed, &r.policy),
            Err(f) => (f.planned, &f.completed, f.policy.as_ref()),
        };
        totals.planned += u128::from(planned);
        totals.completed.add(completed);
        totals.excluded += u128::from(policy.excluded);
        totals.recursive_retained += u128::from(
            policy
                .retained
                .checked_sub(retained)
                .expect("kernel counts immediate retention before execution"),
        );
        match result {
            Ok(report) => {
                self.current
                    .as_mut()
                    .expect("barrier")
                    .applications
                    .iter_mut()
                    .find(|a| a.target == i)
                    .expect("application")
                    .state = ApplicationState::Accepted;
                Ok(report)
            }
            Err(failure) => {
                self.fail(i, End::ApplicationFailed);
                Err(Error::Application {
                    target: target.clone(),
                    failure: Box::new(failure),
                })
            }
        }
    }

    fn future_group(&self) -> bool {
        let p = self.comparison.parts();
        (p.root_pending && p.lanes.iter().any(|l| l.status == LaneStatus::Active))
            || p.frames.iter().enumerate().any(|(depth, f)| {
                f.remaining
                    .iter()
                    .enumerate()
                    .any(|(position, e)| !(position == 0 && depth + 1 < p.frames.len()) && !e.members.is_empty())
            })
    }

    /// Budgets count joined groups and application requests, not bytes, syscalls,
    /// memory or latency. Pure returns can drain with a zero group budget.
    /// Lane failures are returned as events while healthy lanes keep advancing.
    pub fn drive<C: Check, S: Scope, A: Absence, V: Visitor>(&mut self, runtime: &SyncRuntime<C, S, A, V>, limits: Limits) -> Result<Drive, Error> {
        let mut progress = Drive {
            groups: 0,
            applications: 0,
            stop: Stop::Budget,
            failures: Vec::new(),
        };
        loop {
            if self.is_terminal() {
                progress.stop = Stop::Done;
                return Ok(progress);
            }
            if let Some(target) = self.pending().first() {
                if progress.applications == limits.applications {
                    return Ok(progress);
                }
                let token = self.current_token().expect("pending window");
                progress.applications += 1;
                match self.apply(runtime, &token, target) {
                    Ok(_) => {}
                    Err(e @ (Error::Application { .. } | Error::Refresh(_))) => progress.failures.push(e),
                    Err(e) => return Err(e),
                }
            } else {
                self.settle()?;
                if self.is_terminal() {
                    progress.stop = Stop::Done;
                    return Ok(progress);
                }
                if progress.groups == limits.groups && self.future_group() {
                    return Ok(progress);
                }
                match self.next_group(runtime)? {
                    None => {
                        progress.stop = Stop::Done;
                        return Ok(progress);
                    }
                    Some(group) => {
                        progress.groups += 1;
                        for lane in group.outcomes {
                            if let Err(error) = lane.outcome {
                                progress.failures.push(Error::Observed { target: lane.target, error });
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests;
