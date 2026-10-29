use std::iter::FusedIterator;
use std::num::NonZeroUsize;
use std::path::Path;

use rayon::ThreadPool;

use crate::frontier::Frontier;
use crate::pair::Roots;
use crate::{
    Check, CheckedScope, Comparator, DirDiff, DirectoryPlan, Error, Execution, ExecutionDemand, NoCheck, NoScope, PlanWalk, Policy, PolicyWalk,
    PreparedDirectory, Scope, Summary, Visitor, WalkOptions,
};

/// Per-walk admission budgets, not worker counts or a byte-memory bound.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ParallelLimits {
    /// Queued plus running complete directory inspections (J).
    pub max_directory_jobs: NonZeroUsize,
    /// Uncommitted speculative directories (K), including completed results.
    /// An additional demand slot is always reserved; zero permits progress.
    pub max_speculative_directories: usize,
}

impl ParallelLimits {
    pub const fn new(max_directory_jobs: NonZeroUsize, max_speculative_directories: usize) -> Self {
        Self {
            max_directory_jobs,
            max_speculative_directories,
        }
    }

    /// Starting policy J=P, K=4P. No workers are created or reconfigured.
    pub fn for_pool(pool: &ThreadPool) -> Self {
        Self::for_threads(pool.current_num_threads())
    }

    pub(crate) fn for_threads(threads: usize) -> Self {
        Self::new(NonZeroUsize::new(threads).expect("Rayon pool has workers"), threads.saturating_mul(4))
    }
}

/// A joined-epoch directory frontier borrowing an application-owned Rayon pool.
/// `next()` installs its work in this pool, including when called outside it.
/// No jobs run between calls; drop and `into_summary` do no IO or joining.
#[derive(Debug)]
pub struct ParallelWalk<'pool, C = NoCheck, S = NoScope> {
    roots: Roots,
    visitor: CheckedScope<C, S>,
    pool: &'pool ThreadPool,
    frontier: Frontier,
}

/// Compare real directory roots using a borrowed pool and bounded speculation.
/// Unacknowledged differing directories seal all descent until the caller
/// advances. Summaries include only the committed DFS prefix, not lookahead.
///
/// ```no_run
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let app_pool = rayon::ThreadPoolBuilder::new().num_threads(4).build()?;
/// let limits = diff_dirs_tree_compare::ParallelLimits::for_pool(&app_pool);
/// let mut walk = diff_dirs_tree_compare::walk_in_pool("original", "target", &app_pool, limits)?;
/// while let Some(item) = walk.next() {
///     let diff = item?;
///     println!("{:?}", diff.dir);
///     // Apply classified children here, or prune_here() before replacement.
/// }
/// assert!(walk.into_summary().is_complete());
/// # Ok(())
/// # }
/// ```
pub fn walk_in_pool<'pool>(
    original: impl AsRef<Path>,
    target: impl AsRef<Path>,
    pool: &'pool ThreadPool,
    limits: ParallelLimits,
) -> Result<ParallelWalk<'pool>, Error> {
    walk_in_pool_checked(original, target, pool, limits, NoCheck)
}

/// Checked variant. Checks execute outside the frontier lock and complete
/// before a clean inspection can expose any children. The check must be pure;
/// speculation is not an ordered or exactly-once notification mechanism.
pub fn walk_in_pool_checked<C: Check>(
    original: impl AsRef<Path>,
    target: impl AsRef<Path>,
    pool: &ThreadPool,
    limits: ParallelLimits,
    check: C,
) -> Result<ParallelWalk<'_, C>, Error> {
    walk_in_pool_scoped(original, target, pool, limits, check, NoScope)
}

/// Borrowed-pool variant of [`crate::walk_scoped`]. Scope and check run in the
/// inspection worker outside the frontier mutex, before children are admitted.
/// Joined returns, divergence seals, J/K budgets, and committed-prefix summaries
/// are identical to [`walk_in_pool`]. Scoped exhaustion is not full-root clean
/// coverage; see [`Summary::scoped`]. Callers must keep pending paths stable.
pub fn walk_in_pool_scoped<C: Check, S: Scope>(
    original: impl AsRef<Path>,
    target: impl AsRef<Path>,
    pool: &ThreadPool,
    limits: ParallelLimits,
    check: C,
    scope: S,
) -> Result<ParallelWalk<'_, C, S>, Error> {
    let (roots, visitor) = Roots::bind_scoped(original, target, check, scope)?;
    Ok(ParallelWalk {
        roots,
        visitor,
        pool,
        frontier: Frontier::new(limits),
    })
}

impl<C: Check, S: Scope> ParallelWalk<'_, C, S> {
    pub fn prune_here(&mut self) -> bool {
        self.frontier.prune_here()
    }

    pub fn into_summary(self) -> Summary {
        self.frontier.into_summary()
    }

    /// Instrumented builds only; these counters measure private physical work,
    /// separately from the public committed-prefix summary.
    #[cfg(feature = "parallel-diagnostics")]
    pub fn diagnostics(&self) -> &crate::frontier::ParallelDiagnostics {
        self.frontier.diagnostics()
    }
}

impl<C: Check, S: Scope> Iterator for ParallelWalk<'_, C, S> {
    type Item = Result<DirDiff, Error>;
    fn next(&mut self) -> Option<Self::Item> {
        let Self {
            roots,
            visitor,
            pool,
            frontier,
        } = self;
        pool.install(|| frontier.next_with(|dir| roots.inspect_visitor(dir.to_path_buf(), crate::Execution::Sequential, visitor)))
    }
}

impl<C: Check, S: Scope> FusedIterator for ParallelWalk<'_, C, S> {}

/// Finalized visitor batches borrowing the same bounded, joined frontier and
/// caller-owned pool as the comparison projection. No callbacks run under its
/// lock, and only finalized descent can expose children of a clean result.
#[derive(Debug)]
pub struct ParallelPlanWalk<'pool, V: Visitor> {
    walk: PlanWalk<V>,
    pool: &'pool ThreadPool,
}

/// Borrow a caller pool and its J/K limits for finalized visitor planning.
/// No pool is created or reconfigured; callbacks complete outside frontier locks
/// before clean results can publish children. No `Send` bound is imposed on the
/// owned visitor: worker closures share `&V` under its `Sync` contract.
pub fn walk_in_pool_visitor<V: Visitor>(
    original: impl AsRef<Path>,
    target: impl AsRef<Path>,
    pool: &ThreadPool,
    limits: ParallelLimits,
    visitor: V,
) -> Result<ParallelPlanWalk<'_, V>, Error> {
    let mut walk = crate::walk_with_options_visitor(
        original,
        target,
        WalkOptions {
            execution: Execution::Parallel,
        },
        visitor,
    )?;
    walk.inner.frontier = Some(Box::new(Frontier::new(limits)));
    Ok(ParallelPlanWalk { walk, pool })
}
impl<V: Visitor> ParallelPlanWalk<'_, V> {
    /// Stop pending paired descent once after a yielded divergent plan.
    pub fn prune_here(&mut self) -> bool {
        self.walk.prune_here()
    }
    /// Consume only committed DFS coverage; does no I/O and has no running jobs.
    pub fn into_summary(self) -> Summary {
        self.walk.into_summary()
    }
}
impl<V: Visitor> Iterator for ParallelPlanWalk<'_, V> {
    type Item = Result<DirectoryPlan, Error>;
    fn next(&mut self) -> Option<Self::Item> {
        self.walk.next_in_pool(self.pool)
    }
}
impl<V: Visitor> FusedIterator for ParallelPlanWalk<'_, V> {}

/// Contextual planning on the existing joined frontier and a caller-owned pool.
/// The demand provider participates in initial publication, not a later repair.
pub struct ParallelPolicyWalk<'pool, P: Policy, M, E> {
    walk: PolicyWalk<P, M, E>,
    pool: &'pool ThreadPool,
}

pub fn walk_in_pool_policy<P: Policy, M: Comparator, E: ExecutionDemand>(
    original: impl AsRef<Path>,
    target: impl AsRef<Path>,
    pool: &ThreadPool,
    limits: ParallelLimits,
    policy: P,
    comparator: M,
    demand: E,
) -> Result<ParallelPolicyWalk<'_, P, M, E>, Error> {
    let mut walk = crate::walk_with_policy(
        original,
        target,
        WalkOptions {
            execution: Execution::Parallel,
        },
        policy,
        comparator,
        demand,
    )?;
    walk.frontier = Some(Box::new(Frontier::with_context(limits, walk.initial_context(), walk.comparator_id().clone())));
    Ok(ParallelPolicyWalk { walk, pool })
}
impl<P: Policy, M: Comparator, E: ExecutionDemand> ParallelPolicyWalk<'_, P, M, E> {
    pub fn prune_here(&mut self) -> bool {
        self.walk.prune_here()
    }
    pub fn into_summary(self) -> Summary {
        self.walk.into_summary()
    }
}
impl<P: Policy, M: Comparator, E: ExecutionDemand> Iterator for ParallelPolicyWalk<'_, P, M, E> {
    type Item = Result<PreparedDirectory<P::Context>, Error>;
    fn next(&mut self) -> Option<Self::Item> {
        self.walk.next_in_pool(self.pool)
    }
}
impl<P: Policy, M: Comparator, E: ExecutionDemand> FusedIterator for ParallelPolicyWalk<'_, P, M, E> {}
