//! Deterministic paired-directory comparison, ordered traversal and coverage.
//!
//! Immediate inspection and immutable validated plans come from
//! [`diff_dirs_tree_inspect`]. This crate owns recursion, summaries, pruning and
//! the optional joined directory frontier, never filesystem mutation.

mod audit;
pub mod coordinated;
mod model;
mod pair;
mod walk;

#[cfg(feature = "parallel")]
mod frontier;
#[cfg(feature = "parallel")]
mod parallel;

// Re-export the same inspection type identities for convenient composition.
pub use audit::{Audit, AuditFailure, audit, audit_scoped, audit_with_visitor};
pub use diff_dirs_tree_inspect::*;
#[cfg(feature = "parallel-diagnostics")]
pub use frontier::ParallelDiagnostics;
pub use model::{ScanState, Summary, WalkOptions};
#[cfg(feature = "parallel")]
pub use parallel::{ParallelLimits, ParallelPlanWalk, ParallelWalk, walk_in_pool, walk_in_pool_checked, walk_in_pool_scoped, walk_in_pool_visitor};
#[doc(hidden)]
#[cfg(feature = "parallel-bench")]
pub use walk::walk_per_frame;
pub use walk::{
    PlanWalk, Walk, walk, walk_checked, walk_scoped, walk_with_options, walk_with_options_checked, walk_with_options_scoped, walk_with_options_visitor,
    walk_with_visitor,
};

#[cfg(test)]
#[path = "../../../tests/support/fixture.rs"]
mod test_support;
