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

mod error;
mod model;
mod pair;
mod walk;

pub use error::Error;
pub use model::{AddedRemoved, ChangedAddedRemoved, DirDiff, EntryCounts, Execution, ScanState, Summary, WalkOptions};
pub use walk::{Walk, walk, walk_with_options};

#[cfg(all(test, feature = "parallel"))]
extern crate self as diff_dirs;

#[cfg(all(test, feature = "parallel"))]
#[path = "../tests/support/mod.rs"]
mod test_support;
