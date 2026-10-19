use std::ffi::OsString;
use std::iter::FusedIterator;
use std::path::{Path, PathBuf};

use crate::pair::Roots;
use crate::{Check, CheckedScope, DirDiff, DirectoryPlan, Error, NoCheck, NoScope, NoVisitor, ScanState, Scope, Summary, Visitor, WalkOptions};

pub fn walk(original: impl AsRef<Path>, target: impl AsRef<Path>) -> Result<Walk, Error> {
    walk_with_options(original, target, WalkOptions::default())
}
pub fn walk_with_options(original: impl AsRef<Path>, target: impl AsRef<Path>, options: WalkOptions) -> Result<Walk, Error> {
    walk_with_options_checked(original, target, options, NoCheck)
}
pub fn walk_checked<C: Check>(original: impl AsRef<Path>, target: impl AsRef<Path>, check: C) -> Result<Walk<C>, Error> {
    walk_with_options_checked(original, target, WalkOptions::default(), check)
}
pub fn walk_with_options_checked<C: Check>(original: impl AsRef<Path>, target: impl AsRef<Path>, options: WalkOptions, check: C) -> Result<Walk<C>, Error> {
    walk_with_options_scoped(original, target, options, check, NoScope)
}
/// Compatibility projection of early scope and late side-local checks. Public
/// differences remain baseline observations, not edited work catalogs.
pub fn walk_scoped<C: Check, S: Scope>(original: impl AsRef<Path>, target: impl AsRef<Path>, check: C, scope: S) -> Result<Walk<C, S>, Error> {
    walk_with_options_scoped(original, target, WalkOptions::default(), check, scope)
}
pub fn walk_with_options_scoped<C: Check, S: Scope>(
    original: impl AsRef<Path>,
    target: impl AsRef<Path>,
    options: WalkOptions,
    check: C,
    scope: S,
) -> Result<Walk<C, S>, Error> {
    let roots = Roots::new_checked(original, target, &check)?;
    Ok(Walk {
        inner: PlanWalk::new(roots, options, CheckedScope::new(check, scope).roots_checked()),
    })
}

/// Legacy names/classification projection over the visitor walker. Both roots
/// must be real directories. One-sided directory interiors remain opaque here;
/// synchronization recursively inspects them with the same visitor semantics.
#[derive(Debug)]
pub struct Walk<C = NoCheck, S = NoScope> {
    pub(crate) inner: PlanWalk<CheckedScope<C, S>>,
}
impl<C: Check, S: Scope> Walk<C, S> {
    pub fn prune_here(&mut self) -> bool {
        self.inner.prune_here()
    }
    pub fn into_summary(self) -> Summary {
        self.inner.into_summary()
    }
}
impl<C: Check, S: Scope> Iterator for Walk<C, S> {
    type Item = Result<DirDiff, Error>;
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|result| result.map(|plan| plan.diff().clone()))
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
pub fn walk_with_options_visitor<V: Visitor>(
    original: impl AsRef<Path>,
    target: impl AsRef<Path>,
    options: WalkOptions,
    visitor: V,
) -> Result<PlanWalk<V>, Error> {
    Ok(PlanWalk::new(Roots::new_checked(original, target, &NoCheck)?, options, visitor))
}

/// Deterministic DFS plan projection. Parallel epochs join before every return;
/// finalized discarded descent cannot create speculative descendant jobs. J/K,
/// demand reservation and observed-divergence seals are shared with plain walks.
#[derive(Debug)]
pub struct PlanWalk<V = NoVisitor> {
    roots: Roots,
    visitor: V,
    frames: Vec<Frame>,
    start: bool,
    prune_eligible: bool,
    summary: Summary,
    options: WalkOptions,
    #[cfg(feature = "parallel")]
    pub(crate) frontier: Option<Box<crate::frontier::Frontier>>,
    #[cfg(feature = "parallel")]
    use_frontier: bool,
}
#[derive(Debug)]
struct Frame {
    dir: PathBuf,
    children: std::vec::IntoIter<OsString>,
    direct_ancestor: bool,
    subtree_changed: bool,
}

impl<V: Visitor> PlanWalk<V> {
    #[cfg(feature = "parallel")]
    pub(crate) fn next_in_pool(&mut self, pool: &rayon::ThreadPool) -> Option<Result<DirectoryPlan, Error>> {
        let roots = &self.roots;
        let visitor = &self.visitor;
        let frontier = self.frontier.as_mut().expect("explicit pool frontier");
        pool.install(|| frontier.next_plan_with(|dir| roots.inspect_visitor(dir.to_path_buf(), crate::Execution::Sequential, visitor)))
    }
    fn new(roots: Roots, options: WalkOptions, visitor: V) -> Self {
        Self {
            roots,
            visitor,
            frames: Vec::new(),
            start: true,
            prune_eligible: false,
            summary: Summary::default(),
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
            return self.frontier.map_or_else(Summary::default, |f| f.into_summary());
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
    fn advance(&mut self) -> Result<Option<DirectoryPlan>, Error> {
        loop {
            let (dir, direct_ancestor) = if self.start {
                self.start = false;
                (PathBuf::new(), false)
            } else if let Some(frame) = self.frames.last_mut() {
                if let Some(name) = frame.children.next() {
                    (frame.dir.join(name), frame.direct_ancestor)
                } else {
                    self.finish_frame();
                    continue;
                }
            } else {
                self.summary.state = ScanState::Complete;
                return Ok(None);
            };
            let pair = self.roots.inspect_visitor(dir, self.options.execution, &self.visitor)?;
            let changed = !pair.diff.is_empty();
            let plan = pair.plan.expect("visitor plan");
            self.summary.examined_directories += 1;
            self.summary.scoped |= pair.scoped;
            self.summary.excluded += pair.excluded;
            self.summary.counts.add(&pair.diff);
            self.summary.planned += plan.work();
            self.summary.retained += plan.retained();
            if changed && !direct_ancestor {
                self.summary.topmost_divergent.push(pair.diff.dir.clone());
            }
            self.frames.push(Frame {
                dir: pair.diff.dir,
                children: pair.children.into_iter(),
                direct_ancestor: direct_ancestor || changed,
                subtree_changed: changed,
            });
            if changed {
                return Ok(Some(plan));
            }
        }
    }
}
impl<V: Visitor> Iterator for PlanWalk<V> {
    type Item = Result<DirectoryPlan, Error>;
    fn next(&mut self) -> Option<Self::Item> {
        #[cfg(feature = "parallel")]
        if self.use_frontier {
            let frontier = self
                .frontier
                .get_or_insert_with(|| Box::new(crate::frontier::Frontier::new(crate::ParallelLimits::for_threads(rayon::current_num_threads()))));
            return frontier.next_plan_with(|dir| self.roots.inspect_visitor(dir.to_path_buf(), crate::Execution::Sequential, &self.visitor));
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
impl<V: Visitor> FusedIterator for PlanWalk<V> {}

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
    walk.inner.use_frontier = false;
    Ok(walk)
}
