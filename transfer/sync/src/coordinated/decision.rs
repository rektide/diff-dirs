//! Compatibility names, not a second selected-work body or semantic validator.

use std::sync::Arc;

use diff_dirs_tree_inspect::{
    Comparator, DirectorySelection, FullVisitor, Kind, NoVisitor, Policy, Purpose, SelectionDomain, SlotId, SourceObservation, StatKey,
};

use super::{Error, require};

pub use diff_dirs_tree_inspect::EntrySelection as EntryDecision;
pub type DirectoryDecision = Arc<DirectorySelection<()>>;

pub(super) fn validate_decision(selected: &DirectorySelection<()>) -> Result<(), Error> {
    // Construction already ran inspection's common validator. These are solely
    // coordinated-sync composition restrictions, not a competing action table.
    require(selected.domain() == SelectionDomain::PhysicalUnion, "comparison sync requires a physical union")?;
    require(selected.purpose() == Purpose::Compare, "sync requires comparison selections")?;
    require(
        selected.policy() == FullVisitor(NoVisitor).identity() && selected.comparator() == StatKey.identity(),
        "legacy runtime cannot bind a different policy/comparator interpretation",
    )?;
    if selected.is_root() {
        let root = selected.entry(SlotId(0));
        require(
            root.source == SourceObservation::Present(Kind::Directory) && root.target == Some(Kind::Directory),
            "coordinated sync roots must both be directories",
        )?;
    }
    Ok(())
}
