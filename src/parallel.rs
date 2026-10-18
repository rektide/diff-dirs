use std::iter::FusedIterator;
use std::num::NonZeroUsize;
use std::path::Path;

use rayon::ThreadPool;

use crate::frontier::Frontier;
use crate::pair::Roots;
use crate::{Check, DirDiff, Error, NoCheck, NoScope, Scope, Summary};

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
    check: C,
    scope: S,
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
/// let limits = diff_dirs::ParallelLimits::for_pool(&app_pool);
/// let mut walk = diff_dirs::walk_in_pool("original", "target", &app_pool, limits)?;
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
    Ok(ParallelWalk {
        roots: Roots::new_checked(original, target, &check)?,
        check,
        scope,
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
            check,
            scope,
            pool,
            frontier,
        } = self;
        pool.install(|| frontier.next_with(|dir| roots.inspect_scoped(dir.to_path_buf(), crate::Execution::Sequential, check, scope)))
    }
}

impl<C: Check, S: Scope> FusedIterator for ParallelWalk<'_, C, S> {}
