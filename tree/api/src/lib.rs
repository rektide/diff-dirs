//! Deterministic, metadata-based comparison of two existing directory trees.
//!
//! `original` is the source of truth: added entries need copying to `target`,
//! removed entries exist only in `target`. No files are opened for their contents.
//! Files compare by size and exact modification time; symlinks by native target
//! spelling. File modes and same-size edits preserving mtime are not detected.
//!
//! ```no_run
//! # fn main() -> Result<(), diff_dirs_tree_api::Error> {
//! let mut walk = diff_dirs_tree_api::walk("original", "target")?;
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

#[cfg(feature = "parallel-diagnostics")]
pub use diff_dirs_tree_compare::ParallelDiagnostics;
#[cfg(feature = "parallel")]
pub use diff_dirs_tree_compare::{
    ParallelLimits, ParallelPlanWalk, ParallelWalk, walk_in_pool, walk_in_pool_checked, walk_in_pool_scoped, walk_in_pool_visitor,
};

pub use diff_dirs_tree_compare::coordinated;
#[doc(hidden)]
#[cfg(feature = "parallel-bench")]
pub use diff_dirs_tree_compare::walk_per_frame;
pub use diff_dirs_tree_compare::{Audit, AuditFailure, ScanState, Summary, WalkOptions, audit, audit_scoped, audit_with_visitor};
pub use diff_dirs_tree_compare::{
    PlanWalk, Walk, walk, walk_checked, walk_scoped, walk_with_options, walk_with_options_checked, walk_with_options_scoped, walk_with_options_visitor,
    walk_with_visitor,
};
pub use diff_dirs_tree_inspect::{
    Action, AddedRemoved, Admission, ChangedAddedRemoved, Check, CheckError, CheckRef, CheckedScope, Container, DirDiff, DirectoryPlan, EarlyFacts, Entry,
    EntryCounts, Error, Execution, FactOrigin, Facts, FileKind, Name, Names, NoCheck, NoScope, NoVisitor, Operation, Operations, Presence, Purpose, Scope,
    ScopeRef, Side, Visit, Visitor, VisitorRef, inspect_directory, inspect_root, inspect_single_root, scope_includes,
};
pub use diff_dirs_tree_inspect::{
    ApplicationNeed, Assessment, Available, BatchInfo, Choices, Comparator, Comparison, ComparisonInput, ContextSite, Coordinate, Demand, DesiredInput,
    DestinationGrant, DirectoryAdmission, DirectoryCoverage, DirectoryRequest, DirectorySelection, EntryFacts, EntryParts, EntrySelection, EntrySelectionRef,
    ExecutionDemand, FullVisitor, InspectionEpoch, InstallAuthority, InstallIntent, Interior, InteriorInput, InteriorRole, Kind, LeafExpectation, LeafId,
    LeafKind, ManifestId, Mirror, NativePlatform, NoExecutionDemand, NodeId, ObservationBackend, ObservationConfig, ObservationProfile, ObservationResources,
    Policy, PreparedDirectory, PreparedEntries, PreparedIter, PreparedOperation, PreparedOperations, RemovalAuthority, ResourceError, SelectionChoice,
    SelectionDomain, SelectionError, SelectionParts, SemanticId, Shape, ShapeChoice, ShapeChoices, SideDemands, SideFacts, SizeAndLink, SlotId,
    SourceCacheLimits, SourceObservation, SourceSession, StatKey, TargetInspection, inspect_group,
};
