//! Live facts are runtime-only and separate from their shared immutable selection.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::acquire::EntryRuntime;
use crate::{Action, DirectoryComparison, DirectorySelection, EntryFacts, EntrySelectionRef, Facts, Presence, SlotId};

#[derive(Debug)]
pub(crate) struct LiveBinding {
    pub source: PathBuf,
    pub target: PathBuf,
    pub generation: u64,
}

/// Only the inspection engine can construct this type. Semantic reconstruction
/// cannot inject historical metadata or manufacture an executable live binding.
#[derive(Debug)]
pub struct PreparedDirectory<C = ()> {
    pub(crate) selected: Arc<DirectorySelection<C>>,
    pub(crate) facts: Vec<EntryRuntime>,
    pub(crate) binding: LiveBinding,
}
impl<C> PreparedDirectory<C> {
    pub fn selection(&self) -> &Arc<DirectorySelection<C>> {
        &self.selected
    }
    pub fn facts(&self, slot: SlotId) -> EntryFacts<'_> {
        self.facts[slot.0].view()
    }
    pub fn source_root(&self) -> &Path {
        &self.binding.source
    }
    pub fn target_root(&self) -> &Path {
        &self.binding.target
    }
    pub fn source_generation(&self) -> u64 {
        self.binding.generation
    }
}

impl<C: Eq> PreparedDirectory<C> {
    /// Reuse an already-published immutable body only when the entire freshly
    /// inspected selection is equal. Facts, roots and generation remain exactly
    /// those of this preparation. This cannot construct preparation from saved
    /// data, supply missing execution facts, or authorize any output mapping.
    pub fn share_selection(mut self, selected: Arc<DirectorySelection<C>>) -> Result<Self, crate::SelectionError> {
        if self.selected != selected {
            return Err(crate::SelectionError("fresh selection differs from published body"));
        }
        self.selected = selected;
        Ok(self)
    }
}

impl PreparedDirectory<()> {
    /// Compatibility projection only. Sparse/custom selections cannot manufacture
    /// full metadata. No observation or policy finalization is performed here.
    pub fn into_full_plan(self) -> Result<DirectoryPlan, crate::SelectionError> {
        use crate::{Comparator, FullVisitor, NoVisitor, Policy, StatKey};
        if self.selected.policy() != FullVisitor(NoVisitor).identity()
            || self.selected.context_schema() != FullVisitor(NoVisitor).context_schema()
            || self.selected.comparator() != StatKey.identity()
            || self
                .facts
                .iter()
                .any(|f| f.source.as_ref().is_some_and(|s| s.legacy.is_none()) || f.target.as_ref().is_some_and(|s| s.legacy.is_none()))
        {
            return Err(crate::SelectionError("not a prepared full-visitor batch"));
        }
        Ok(DirectoryPlan { prepared: self })
    }
}

/// Narrow full-visitor adapter used by existing traversal/effect APIs. It shares
/// the selected body, rather than preserving a second action/validation model.
#[derive(Debug)]
pub struct DirectoryPlan {
    pub(crate) prepared: PreparedDirectory<()>,
}
impl DirectoryPlan {
    pub fn selection(&self) -> &Arc<DirectorySelection<()>> {
        self.prepared.selection()
    }
    pub fn into_prepared(self) -> PreparedDirectory<()> {
        self.prepared
    }
    pub fn operations(&self) -> PreparedOperations<'_> {
        PreparedOperations { prepared: &self.prepared }
    }
    pub fn diff(&self) -> &crate::DirDiff {
        self.selection().diff()
    }
    pub fn excluded(&self) -> u64 {
        self.selection().coverage().excluded
    }
    pub fn first_excluded(&self) -> Option<PathBuf> {
        self.selection().excluded_names().first().map(|n| self.selection().relative().join(n))
    }
    pub fn preserves(&self, name: &OsStr) -> bool {
        self.selection().excluded_names().iter().any(|n| n == name)
    }
    pub fn excluded_names(&self) -> &[std::ffi::OsString] {
        self.selection().excluded_names()
    }
    pub fn steered(&self) -> bool {
        self.selection().coverage().scoped
    }
    /// Selected preservation boundaries, not an inventory of surviving entries.
    pub fn retained(&self) -> u64 {
        self.selection().coverage().retained
    }
    pub fn deviated(&self) -> u64 {
        self.selection().coverage().deviated
    }
    pub fn work(&self) -> u64 {
        self.selection().coverage().planned
    }
    pub fn paired_children(&self) -> impl Iterator<Item = &OsStr> {
        self.selection().paired_children().map(|e| e.name())
    }
    pub fn into_comparison(self) -> DirectoryComparison {
        DirectoryComparison {
            diff: self.selection().diff().clone(),
            coverage: self.selection().coverage(),
            children: self.paired_children().map(OsStr::to_os_string).collect(),
            root: self.selection().is_root(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PreparedOperations<'a> {
    prepared: &'a PreparedDirectory<()>,
}
impl<'a> PreparedOperations<'a> {
    pub fn source_root(&self) -> &'a Path {
        self.prepared.source_root()
    }
    pub fn target_root(&self) -> &'a Path {
        self.prepared.target_root()
    }
    pub fn dir(&self) -> &'a Path {
        self.prepared.selected.relative()
    }
    pub fn purpose(&self) -> crate::Purpose {
        self.prepared.selected.purpose()
    }
    pub fn is_root(&self) -> bool {
        self.prepared.selected.is_root()
    }
    pub fn entries(&self) -> PreparedEntries<'a> {
        PreparedEntries { prepared: self.prepared }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct PreparedEntries<'a> {
    prepared: &'a PreparedDirectory<()>,
}
impl<'a> PreparedEntries<'a> {
    pub fn len(&self) -> usize {
        self.prepared.selected.len()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn get(&self, index: usize) -> Option<PreparedOperation<'a>> {
        (index < self.len()).then(|| PreparedOperation {
            selected: self.prepared.selected.entry(SlotId(index)),
            relative: if self.prepared.selected.is_root() {
                PathBuf::new()
            } else {
                self.prepared.selected.relative().join(self.prepared.selected.entry(SlotId(index)).name())
            },
            facts: &self.prepared.facts[index],
        })
    }
    pub fn iter(&self) -> PreparedIter<'a> {
        PreparedIter { entries: *self, next: 0 }
    }
    pub fn binary_search_by(&self, mut compare: impl FnMut(&PreparedOperation<'a>) -> std::cmp::Ordering) -> Result<usize, usize> {
        let (mut low, mut high) = (0, self.len());
        while low < high {
            let middle = low + (high - low) / 2;
            match compare(&self.get(middle).expect("entry in range")) {
                std::cmp::Ordering::Less => low = middle + 1,
                std::cmp::Ordering::Greater => high = middle,
                std::cmp::Ordering::Equal => return Ok(middle),
            }
        }
        Err(low)
    }
}
impl<'a> IntoIterator for PreparedEntries<'a> {
    type Item = PreparedOperation<'a>;
    type IntoIter = PreparedIter<'a>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
pub struct PreparedIter<'a> {
    entries: PreparedEntries<'a>,
    next: usize,
}
impl<'a> Iterator for PreparedIter<'a> {
    type Item = PreparedOperation<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        let entry = self.entries.get(self.next)?;
        self.next += 1;
        Some(entry)
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let n = self.entries.len() - self.next;
        (n, Some(n))
    }
}
impl ExactSizeIterator for PreparedIter<'_> {}
#[derive(Debug)]
pub struct PreparedOperation<'a> {
    selected: EntrySelectionRef<'a>,
    relative: PathBuf,
    facts: &'a EntryRuntime,
}
impl<'a> PreparedOperation<'a> {
    pub fn name(&self) -> &'a OsStr {
        self.selected.name()
    }
    pub fn relative_path(&self) -> &Path {
        &self.relative
    }
    pub fn source(&self) -> Option<&'a Facts> {
        self.facts.source.as_ref().and_then(|f| f.legacy.as_deref())
    }
    pub fn target(&self) -> Option<&'a Facts> {
        self.facts.target.as_ref().and_then(|f| f.legacy.as_deref())
    }
    pub fn presence(&self) -> Presence {
        self.selected.shape().presence().expect("physical full visitor presence")
    }
    pub fn is_different(&self) -> bool {
        self.selected.comparison == crate::Comparison::Different
    }
    pub fn comparison(&self) -> Option<bool> {
        self.selected.comparison.legacy()
    }
    pub fn action(&self) -> Option<Action> {
        Some(self.selected.action)
    }
    pub fn baseline(&self) -> Option<Action> {
        self.selected.baseline
    }
    pub fn descends(&self) -> bool {
        matches!(
            self.selected.action,
            Action::Directory { descend: true, .. } | Action::ReplaceWithDirectory { descend: true }
        )
    }
    pub fn has_work(&self) -> bool {
        self.selected.has_work()
    }
}
