//! Immediate directory observation, pure visitor steering and validated plans.
//!
//! No recursive traversal or filesystem mutation occurs here. Physical acquisition
//! belongs to observation; this crate decides demands and comparison policy.

mod acquire;
mod check;
mod comparison;
mod declared;
mod epoch;
mod error;
mod model;
mod policy;
mod prepared;
mod roots;
mod scope;
pub mod selection;
mod source;
mod visitor;

pub use acquire::{Available, Demand, EntryFacts, SideDemands, SideFacts};
pub use check::{Check, CheckError, CheckRef, Entry, NoCheck, Purpose, Side};
pub use comparison::{Comparator, Comparison, ComparisonInput, SemanticId, SizeAndLink, StatChecks, StatComparator, StatKey};
pub use declared::{
    DeclaredCandidate, DeclaredChoice, DeclaredEntryCandidate, DeclaredInspection, DeclaredName, DeclaredNames, DeclaredRequest, DeclaredRouteCandidate,
    DeclaredSelection, inspect_declared_group,
};
pub use diff_dirs_tree_observe::{EarlyFacts, FactOrigin, FileKind};
pub use diff_dirs_tree_observe::{ObservationBackend, ObservationConfig, ObservationResources, ResourceError};
pub use error::Error;
pub use model::{AddedRemoved, ChangedAddedRemoved, DirDiff, EntryCounts, Execution};
pub use policy::{
    Assessment, BatchInfo, Choices, ContextSite, Coordinate, ExecutionDemand, FullVisitor, Mirror, NoExecutionDemand, ObservationProfile, Policy,
    SelectionChoice, ShapeChoice, ShapeChoices,
};
pub use prepared::{DirectoryPlan, PreparedDirectory, PreparedEntries, PreparedIter, PreparedOperation, PreparedOperations};
pub use roots::PairedRoots;
pub use scope::{NoScope, Scope, ScopeRef, Visit, scope_includes};
pub use selection::{
    ApplicationNeed, DesiredInput, DestinationGrant, DirectoryCoverage, DirectorySelection, EntryParts, EntrySelection, EntrySelectionRef, InstallAuthority,
    InstallIntent, Interior, InteriorInput, InteriorRole, Kind, LeafExpectation, LeafId, LeafKind, ManifestId, NativePlatform, NodeId, RemovalAuthority,
    SelectionDomain, SelectionError, SelectionParts, Shape, SlotId, SourceObservation,
};
pub use source::{SourceCacheLimits, SourceDiagnostics, SourceSession, SourceWork, SourceWorkDiagnostics};
pub use visitor::shared::{
    DirectoryAdmission, DirectoryComparison, DirectoryRequest, InspectionEpoch, SharedBindError, SharedInputError, SharedRoots, TargetComparison,
    TargetInspection, TargetVisitor, inspect_group, inspect_group_with_work,
};
pub use visitor::{
    Action, Admission, CheckedScope, Container, Facts, Name, Names, NoVisitor, Operation, Operations, Presence, Visitor, VisitorRef, inspect_directory,
    inspect_root, inspect_single_root,
};

#[cfg(test)]
#[path = "../../../tests/support/fixture.rs"]
mod test_support;

#[cfg(test)]
mod selection_tests;
