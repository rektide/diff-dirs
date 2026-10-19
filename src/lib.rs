#![cfg_attr(all(target_os = "linux", feature = "native-dirent"), feature(dirfd))]

//! Deterministic, metadata-based comparison of two existing directory trees.
//!
//! `original` is the source of truth: added entries need copying to `target`,
//! removed entries exist only in `target`. No files are opened for their contents.
//! Files compare by size and exact modification time; symlinks by native target
//! spelling. File modes and same-size edits preserving mtime are not detected.
//!
//! ```no_run
//! # fn main() -> Result<(), diff_dirs::Error> {
//! let mut walk = diff_dirs::walk("original", "target")?;
//! while let Some(diff) = walk.next() {
//!     let diff = diff?;
//!     println!("{:?}: {:?}", diff.dir, diff.files);
//!     // Immediate children have already been examined; apply policy here.
//!     // walk.prune_here(); // optionally skip this record's paired subdirs
//! }
//! let summary = walk.into_summary();
//! assert!(summary.is_complete());
//! # Ok(())
//! # }
//! ```
//!
//! The walker is not a snapshot or a security boundary. Callers own quiescence,
//! confinement, mutation, and absent/non-directory root dispatch. Symlink leaves
//! are never followed; symlink roots are rejected.

mod audit;
mod check;
mod error;
mod listing;
mod model;
mod pair;
mod scope;
mod visitor;
mod walk;

#[cfg(feature = "parallel")]
mod frontier;
#[cfg(feature = "parallel")]
mod parallel;
#[cfg(feature = "parallel-diagnostics")]
pub use frontier::ParallelDiagnostics;
#[cfg(feature = "parallel")]
pub use parallel::{ParallelLimits, ParallelPlanWalk, ParallelWalk, walk_in_pool, walk_in_pool_checked, walk_in_pool_scoped, walk_in_pool_visitor};

pub use audit::{Audit, AuditFailure, audit, audit_scoped, audit_with_visitor};
pub use check::{Check, CheckError, CheckRef, Entry, NoCheck, Purpose, Side};
pub use error::Error;
pub use model::{AddedRemoved, ChangedAddedRemoved, DirDiff, EntryCounts, Execution, ScanState, Summary, WalkOptions};
pub use scope::{NoScope, Scope, ScopeRef, Visit, scope_includes};
pub use visitor::{
    Action, Admission, CheckedScope, Container, DirectoryPlan, EarlyFacts, FactOrigin, Facts, FileKind, Name, Names, NoVisitor, Operation, Operations,
    Presence, Visitor, VisitorRef, inspect_directory, inspect_root, inspect_single_root,
};
#[doc(hidden)]
#[cfg(feature = "parallel-bench")]
pub use walk::walk_per_frame;
pub use walk::{
    PlanWalk, Walk, walk, walk_checked, walk_scoped, walk_with_options, walk_with_options_checked, walk_with_options_scoped, walk_with_options_visitor,
    walk_with_visitor,
};

#[cfg(test)]
extern crate self as diff_dirs;

#[cfg(test)]
#[path = "../tests/support/mod.rs"]
mod test_support;
