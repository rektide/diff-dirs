use std::iter::FusedIterator;
use std::path::{Path, PathBuf};

use crate::pair::{Child, Roots};
use crate::{
    Check, CheckedScope, Comparator, DirDiff, DirectoryPlan, Error, ExecutionDemand, FullVisitor, NoCheck, NoExecutionDemand, NoScope, NoVisitor, Policy,
    PreparedDirectory, ScanState, Scope, StatKey, Summary, Visitor, WalkOptions,
};

/// Construct a lazy default comparison of two existing real directory roots.
/// Source-only and target-only directories remain opaque comparison boundaries.
pub fn walk(original: impl AsRef<Path>, target: impl AsRef<Path>) -> Result<Walk, Error> {
    walk_with_options(original, target, WalkOptions::default())
}
/// Default comparison with explicit execution; contents remain lazy.
pub fn walk_with_options(original: impl AsRef<Path>, target: impl AsRef<Path>, options: WalkOptions) -> Result<Walk, Error> {
    walk_with_options_checked(original, target, options, NoCheck)
}
/// Legacy late-fact refusal adapter, including equal entries and structural
/// roots. Checks do not edit work; use [`walk_with_visitor`] for that authority.
pub fn walk_checked<C: Check>(original: impl AsRef<Path>, target: impl AsRef<Path>, check: C) -> Result<Walk<C>, Error> {
    walk_with_options_checked(original, target, WalkOptions::default(), check)
}
/// Checked compatibility comparison with explicit execution.
pub fn walk_with_options_checked<C: Check>(original: impl AsRef<Path>, target: impl AsRef<Path>, options: WalkOptions, check: C) -> Result<Walk<C>, Error> {
    walk_with_options_scoped(original, target, options, check, NoScope)
}
/// Compatibility projection of early scope and late side-local checks. Public
/// differences remain baseline observations, not edited work catalogs.
pub fn walk_scoped<C: Check, S: Scope>(original: impl AsRef<Path>, target: impl AsRef<Path>, check: C, scope: S) -> Result<Walk<C, S>, Error> {
    walk_with_options_scoped(original, target, WalkOptions::default(), check, scope)
}
/// Scoped compatibility comparison with explicit execution, using the same
/// finalized visitor inspection and traversal as ordinary and rich walks.
pub fn walk_with_options_scoped<C: Check, S: Scope>(
    original: impl AsRef<Path>,
    target: impl AsRef<Path>,
    options: WalkOptions,
    check: C,
    scope: S,
) -> Result<Walk<C, S>, Error> {
    let (roots, visitor) = Roots::bind_scoped(original, target, check, scope)?;
    Ok(Walk {
        inner: PlanWalk::new(roots, options, visitor),
    })
}

/// Legacy names/classification projection over the visitor walker. Both roots
/// must be real directories. One-sided directory interiors remain opaque here;
/// synchronization recursively inspects them with the same visitor semantics.
#[derive(Debug)]
pub struct Walk<C: Check = NoCheck, S: Scope = NoScope> {
    pub(crate) inner: PlanWalk<CheckedScope<C, S>>,
}
impl<C: Check, S: Scope> Walk<C, S> {
    /// Stop pending paired descent once immediately after a successful record.
    pub fn prune_here(&mut self) -> bool {
        self.inner.prune_here()
    }
    /// Consume committed-prefix coverage without further I/O or callbacks.
    pub fn into_summary(self) -> Summary {
        self.inner.into_summary()
    }
}
impl<C: Check, S: Scope> Iterator for Walk<C, S> {
    type Item = Result<DirDiff, Error>;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.inner.next()? {
                Ok(plan) if plan.diff().is_empty() => continue,
                result => return Some(result.map(|plan| plan.diff().clone())),
            }
        }
    }
}
impl<C: Check, S: Scope> FusedIterator for Walk<C, S> {}

/// Compare with compiled early/late steering. Every admitted immediate entry,
/// including equal leaves, reaches late before finalized descent is published.
/// The iterator yields finalized batches only at observed divergent directories;
/// clean batches still invoke the visitor and govern child admission. No mutation
/// occurs here. The baseline [`DirDiff`] and summary remain observation, while
/// [`DirectoryPlan::work`] describes finalized work, not completed operations.
pub fn walk_with_visitor<V: Visitor>(original: impl AsRef<Path>, target: impl AsRef<Path>, visitor: V) -> Result<PlanWalk<V>, Error> {
    walk_with_options_visitor(original, target, WalkOptions::default(), visitor)
}
/// Rich visitor walk with explicit execution. Normal callbacks are lazy except
/// mandatory structural binding; each next call joins its admitted epoch.
pub fn walk_with_options_visitor<V: Visitor>(
    original: impl AsRef<Path>,
    target: impl AsRef<Path>,
    options: WalkOptions,
    visitor: V,
) -> Result<PlanWalk<V>, Error> {
    Ok(PlanWalk::new(Roots::new(original, target)?, options, visitor))
}

/// Deterministic DFS plan projection. Parallel epochs join before every return;
/// finalized discarded descent cannot create speculative descendant jobs. J/K,
/// demand reservation and observed-divergence seals are shared with plain walks.
#[derive(Debug)]
pub struct PlanWalk<V: Visitor = NoVisitor> {
    pub(crate) inner: PolicyWalk<FullVisitor<V>, StatKey, NoExecutionDemand>,
}

impl<V: Visitor> PlanWalk<V> {
    fn new(roots: Roots, options: WalkOptions, visitor: V) -> Self {
        Self {
            inner: PolicyWalk::new(roots, options, FullVisitor(visitor), StatKey, NoExecutionDemand),
        }
    }
    pub fn prune_here(&mut self) -> bool {
        self.inner.prune_here()
    }
    pub fn into_summary(self) -> Summary {
        self.inner.into_summary()
    }
    #[cfg(feature = "parallel")]
    pub(crate) fn next_in_pool(&mut self, pool: &rayon::ThreadPool) -> Option<Result<DirectoryPlan, Error>> {
        self.inner
            .next_in_pool(pool)
            .map(|r| r.map(|p| p.into_full_plan().expect("full-visitor walker")))
    }
}
impl<V: Visitor> Iterator for PlanWalk<V> {
    type Item = Result<DirectoryPlan, Error>;
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|r| r.map(|p| p.into_full_plan().expect("full-visitor walker")))
    }
}
impl<V: Visitor> FusedIterator for PlanWalk<V> {}

/// Typed contextual plans over the same DFS and joined parallel frontier used by
/// full visitors. Work-only publications retain their prepared facts and seal
/// paired descendants just like divergent publications.
pub fn walk_with_policy<P: Policy, M: Comparator, E: ExecutionDemand>(
    original: impl AsRef<Path>,
    target: impl AsRef<Path>,
    options: WalkOptions,
    policy: P,
    comparator: M,
    demand: E,
) -> Result<PolicyWalk<P, M, E>, Error> {
    Ok(PolicyWalk::new(Roots::new(original, target)?, options, policy, comparator, demand))
}

pub fn walk_with_source<P: Policy, M: Comparator, E: ExecutionDemand>(
    source: std::sync::Arc<crate::SourceSession>,
    target: impl AsRef<Path>,
    options: WalkOptions,
    policy: P,
    comparator: M,
    demand: E,
) -> Result<PolicyWalk<P, M, E>, Error> {
    Ok(PolicyWalk::new(Roots::with_source(source, target)?, options, policy, comparator, demand))
}

#[derive(Debug)]
pub struct PolicyWalk<P: Policy, M = StatKey, E = NoExecutionDemand> {
    roots: Roots,
    policy: P,
    comparator: M,
    demand: E,
    frames: Vec<Frame<P::Context>>,
    start: bool,
    prune_eligible: bool,
    summary: Summary,
    options: WalkOptions,
    #[cfg(feature = "parallel")]
    pub(crate) frontier: Option<Box<crate::frontier::Frontier<P::Context>>>,
    #[cfg(feature = "parallel")]
    use_frontier: bool,
}
#[derive(Debug)]
struct Frame<C> {
    dir: PathBuf,
    children: std::vec::IntoIter<Child<C>>,
    direct_ancestor: bool,
    subtree_changed: bool,
}

impl<P: Policy, M: Comparator, E: ExecutionDemand> PolicyWalk<P, M, E> {
    pub fn source_session(&self) -> &std::sync::Arc<crate::SourceSession> {
        self.roots.source_session()
    }
    #[cfg(feature = "parallel")]
    pub(crate) fn initial_context(&self) -> P::Context {
        self.policy.initial_context()
    }
    #[cfg(feature = "parallel")]
    pub(crate) fn comparator_id(&self) -> &crate::SemanticId {
        self.comparator.identity()
    }
    #[cfg(feature = "parallel")]
    pub(crate) fn next_in_pool(&mut self, pool: &rayon::ThreadPool) -> Option<Result<PreparedDirectory<P::Context>, Error>> {
        let roots = &self.roots;
        let policy = &self.policy;
        let comparator = &self.comparator;
        let demand = &self.demand;
        let frontier = self.frontier.as_mut().expect("explicit pool frontier");
        pool.install(|| frontier.next_plan_with(|dir, context| roots.inspect_policy(dir, context, crate::Execution::Sequential, policy, comparator, demand)))
    }
    fn new(roots: Roots, options: WalkOptions, policy: P, comparator: M, demand: E) -> Self {
        let summary = Summary {
            comparator: Some(comparator.identity().clone()),
            ..Summary::default()
        };
        Self {
            roots,
            policy,
            comparator,
            demand,
            frames: Vec::new(),
            start: true,
            prune_eligible: false,
            summary,
            options,
            #[cfg(feature = "parallel")]
            frontier: None,
            #[cfg(feature = "parallel")]
            use_frontier: options.execution == crate::Execution::Parallel,
        }
    }
    /// Suppress remaining paired child descent once immediately after a yield.
    /// There are no outstanding jobs; differing batches seal their children.
    pub fn prune_here(&mut self) -> bool {
        #[cfg(feature = "parallel")]
        if self.use_frontier {
            return self.frontier.as_mut().is_some_and(|f| f.prune_here());
        }
        if !self.prune_eligible {
            return false;
        }
        self.prune_eligible = false;
        let frame = self.frames.last_mut().expect("yielded frame");
        frame.children = Vec::new().into_iter();
        self.summary.pruned.push(frame.dir.clone());
        true
    }
    /// Consume committed-prefix coverage without additional I/O or callbacks.
    pub fn into_summary(mut self) -> Summary {
        #[cfg(feature = "parallel")]
        if self.use_frontier {
            return self.frontier.map_or(self.summary, |f| f.into_summary());
        }
        while !self.frames.is_empty() {
            self.finish_frame();
        }
        self.summary
    }
    fn finish_frame(&mut self) {
        let frame = self.frames.pop().expect("nonempty frames");
        if frame.subtree_changed {
            self.summary.changed_dirs.push(frame.dir);
            if let Some(parent) = self.frames.last_mut() {
                parent.subtree_changed = true;
            }
        }
    }
    fn advance(&mut self) -> Result<Option<PreparedDirectory<P::Context>>, Error> {
        loop {
            let (dir, context, direct_ancestor) = if self.start {
                self.start = false;
                (PathBuf::new(), self.policy.initial_context(), false)
            } else if let Some(frame) = self.frames.last_mut() {
                if let Some(child) = frame.children.next() {
                    (frame.dir.join(child.name), child.context, frame.direct_ancestor)
                } else {
                    self.finish_frame();
                    continue;
                }
            } else {
                self.summary.state = ScanState::Complete;
                return Ok(None);
            };
            let pair = self
                .roots
                .inspect_policy(&dir, &context, self.options.execution, &self.policy, &self.comparator, &self.demand)?;
            let changed = !pair.diff.is_empty();
            self.summary.examined_directories += 1;
            self.summary.add_coverage(pair.coverage, &pair.comparator);
            self.summary.counts.add(&pair.diff);
            if changed && !direct_ancestor {
                self.summary.topmost_divergent.push(pair.diff.dir.clone());
            }
            self.frames.push(Frame {
                dir: pair.diff.dir,
                children: pair.children.into_iter(),
                direct_ancestor: direct_ancestor || changed,
                subtree_changed: changed,
            });
            if let Some(plan) = pair.plan {
                return Ok(Some(plan));
            }
        }
    }
}
impl<P: Policy, M: Comparator, E: ExecutionDemand> Iterator for PolicyWalk<P, M, E> {
    type Item = Result<PreparedDirectory<P::Context>, Error>;
    fn next(&mut self) -> Option<Self::Item> {
        #[cfg(feature = "parallel")]
        if self.use_frontier {
            let frontier = self.frontier.get_or_insert_with(|| {
                Box::new(crate::frontier::Frontier::with_context(
                    crate::ParallelLimits::for_threads(rayon::current_num_threads()),
                    self.policy.initial_context(),
                    self.comparator.identity().clone(),
                ))
            });
            return frontier.next_plan_with(|dir, context| {
                self.roots
                    .inspect_policy(dir, context, crate::Execution::Sequential, &self.policy, &self.comparator, &self.demand)
            });
        }
        self.prune_eligible = false;
        if self.summary.state != ScanState::Incomplete {
            return None;
        }
        match self.advance() {
            Ok(Some(plan)) => {
                self.prune_eligible = true;
                Some(Ok(plan))
            }
            Ok(None) => None,
            Err(error) => {
                self.summary.state = ScanState::Failed;
                Some(Err(error))
            }
        }
    }
}
impl<P: Policy, M: Comparator, E: ExecutionDemand> FusedIterator for PolicyWalk<P, M, E> {}

#[doc(hidden)]
#[cfg(feature = "parallel-bench")]
pub fn walk_per_frame(original: impl AsRef<Path>, target: impl AsRef<Path>) -> Result<Walk, Error> {
    let mut walk = walk_with_options(
        original,
        target,
        WalkOptions {
            execution: crate::Execution::Parallel,
        },
    )?;
    walk.inner.inner.use_frontier = false;
    Ok(walk)
}
