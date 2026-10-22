#![cfg_attr(all(target_os = "linux", feature = "native-dirent"), feature(dirfd))]

//! Immediate directory observation, pure visitor steering and validated plans.
//!
//! No recursive traversal or filesystem mutation occurs here. Listing backends
//! remain private pending their separate physical-observation extraction.

mod check;
mod error;
mod listing;
mod model;
mod roots;
mod scope;
mod visitor;

pub use check::{Check, CheckError, CheckRef, Entry, NoCheck, Purpose, Side};
pub use error::Error;
pub use model::{AddedRemoved, ChangedAddedRemoved, DirDiff, EntryCounts, Execution};
pub use roots::PairedRoots;
pub use scope::{NoScope, Scope, ScopeRef, Visit, scope_includes};
pub use visitor::{
    Action, Admission, CheckedScope, Container, DirectoryPlan, EarlyFacts, FactOrigin, Facts, FileKind, Name, Names, NoVisitor, Operation, Operations,
    Presence, Visitor, VisitorRef, inspect_directory, inspect_root, inspect_single_root,
};

#[cfg(test)]
#[path = "../../../tests/support/fixture.rs"]
mod test_support;
