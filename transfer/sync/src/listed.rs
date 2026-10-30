//! Complete desired/owned manifest execution. Native authority remains with the
//! caller; physical equality never confers occupied-name replacement permission.
mod manifest;
mod runner;
mod selection;
use crate::{EffectBinding, Failure, Report};
use diff_dirs_tree_compare::coordinated::TargetId;
use diff_dirs_tree_inspect::{CheckError, DeclaredCandidate, DesiredInput, EntryFacts, LeafId, NodeId, SideDemands, SourceSession};
pub use diff_dirs_tree_inspect::{LeafExpectation, LeafKind, ManifestId};
pub use manifest::{OwnedLeafSet, PreparedLeafList};
use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DesiredLeaf {
    pub relative: PathBuf,
    pub expected: LeafExpectation,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ListedIntent {
    CreateAbsent,
    RenewOwned,
    /// Retain compatible owned leaves under this explicit metadata heuristic.
    RefreshOwned(diff_dirs_tree_inspect::StatChecks),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ListedMode {
    Declared,
    DesiredPreparation,
    CoupledRemoval,
}
/// Resolved relations describe these supplied indexes, not provider-discovered
/// native ownership. Routes can have no corresponding declaration.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ListedRelation {
    pub desired: Option<DesiredInput>,
    pub owned_leaf: Option<LeafId>,
    pub owned_directory: Option<NodeId>,
}
#[derive(Clone, Copy, Debug)]
pub struct ListedCandidate<'a> {
    pub observed: DeclaredCandidate<'a>,
    pub intent: ListedIntent,
    pub mode: ListedMode,
    pub relation: ListedRelation,
}
/// Pure additional refusal only: cannot drop an obligation or widen ownership.
/// A route may request attributes only on its identified side. An O-only/census
/// entry must not request source facts; this is an integration refusal, not I/O.
pub trait ListedGuard: Sync {
    fn demand(&self, _: ListedCandidate<'_>) -> SideDemands {
        SideDemands::default()
    }
    fn check(&self, candidate: ListedCandidate<'_>, facts: EntryFacts<'_>) -> Result<(), CheckError>;
}
#[derive(Clone, Copy, Debug, Default)]
pub struct NoListedGuard;
impl ListedGuard for NoListedGuard {
    fn check(&self, _: ListedCandidate<'_>, _: EntryFacts<'_>) -> Result<(), CheckError> {
        Ok(())
    }
}
pub struct ListedTarget<'a, G: ListedGuard> {
    pub id: &'a TargetId,
    pub binding: &'a EffectBinding,
    pub owned: &'a OwnedLeafSet,
    pub intent: ListedIntent,
    pub guard: &'a G,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ManifestCoverage {
    pub desired: u64,
    pub installed: u64,
    /// Desired leaves accepted in place; never installation/clone effects.
    pub retained: u64,
    pub owned_obsolete: u64,
    pub owned_retired: u64,
    pub complete: bool,
}
#[derive(Debug)]
pub struct ListedOutcome {
    pub id: TargetId,
    pub coverage: ManifestCoverage,
    pub outcome: Result<Report, Failure>,
}
#[derive(Debug)]
pub struct ListedReport {
    pub outcomes: Vec<ListedOutcome>,
}

/// One coarse call with ordered independent lane outcomes. Every desired leaf is
/// a real obligation, including when neither physical tree contains its name.
/// All started scopes/staging close before return; there is no saved interior.
///
/// Prepare each distinct S/O once, then lend the immutable indexes to receivers.
/// Only `coverage.complete` on a successful lane proves this manifest finished;
/// it does not establish native revision/cache authority or compare tree equality.
/// A changed physical source phase requires a fresh `SourceSession`.
///
/// ```no_run
/// use diff_dirs_transfer_sync::{
///     CopyPolicy, DesiredLeaf, EffectBinding, LeafExpectation, LeafKind,
///     ListedIntent, ListedTarget, NoListedGuard, OwnedLeafSet,
///     PreparedLeafList, reconcile_listed,
/// };
/// use diff_dirs_tree_inspect::SourceSession;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let desired = PreparedLeafList::from_specs([DesiredLeaf {
///     relative: "bin/tool".into(),
///     expected: LeafExpectation::Exact {
///         kind: LeafKind::RegularFile,
///         executable: Some(true), // Unix executable-bit contract
///     },
/// }])?;
/// let owned = OwnedLeafSet::new(["bin/old-tool"])?;
/// let binding = EffectBinding::new("source", "receiver", CopyPolicy::ReflinkRequired)?;
/// let source = SourceSession::new(binding.source());
/// let id = "receiver".into();
/// let report = reconcile_listed(&source, &desired, &[ListedTarget {
///     id: &id, binding: &binding, owned: &owned,
///     intent: ListedIntent::RenewOwned, guard: &NoListedGuard,
/// }])?;
/// let lane = &report.outcomes[0];
/// assert!(lane.outcome.is_ok() && lane.coverage.complete);
/// // Only now may a native caller consider its own state-adoption protocol.
/// # Ok(())
/// # }
/// ```
pub fn reconcile_listed<G: ListedGuard>(
    source: &SourceSession,
    desired: &PreparedLeafList,
    targets: &[ListedTarget<'_, G>],
) -> Result<ListedReport, crate::coordinated::Error> {
    runner::run(source, desired, targets)
}
