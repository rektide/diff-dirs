//! Immediate directory observation, pure visitor steering and validated plans.
//!
//! No recursive traversal or filesystem mutation occurs here. Physical acquisition
//! belongs to observation; this crate decides demands and comparison policy.

mod check;
mod error;
mod model;
mod roots;
mod scope;
mod visitor;

pub use check::{Check, CheckError, CheckRef, Entry, NoCheck, Purpose, Side};
pub use diff_dirs_tree_observe::{EarlyFacts, FactOrigin, FileKind};
pub use error::Error;
pub use model::{AddedRemoved, ChangedAddedRemoved, DirDiff, EntryCounts, Execution};
pub use roots::PairedRoots;
pub use scope::{NoScope, Scope, ScopeRef, Visit, scope_includes};
pub use visitor::shared::{DirectoryComparison, DirectoryCoverage, SharedBindError, SharedInputError, SharedRoots, TargetComparison, TargetVisitor};
pub use visitor::{
    Action, Admission, CheckedScope, Container, DirectoryPlan, Facts, Name, Names, NoVisitor, Operation, Operations, Presence, Visitor, VisitorRef,
    inspect_directory, inspect_root, inspect_single_root,
};

#[cfg(test)]
#[path = "../../../tests/support/fixture.rs"]
mod test_support;
