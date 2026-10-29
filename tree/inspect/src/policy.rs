//! Pure contextual policy stages. Views cannot edit physical identity or facts.

use std::fmt::Debug;
use std::path::Path;
use std::sync::LazyLock;

use crate::{
    Action, CheckError, Comparison, EntryFacts, EntrySelectionRef, InstallAuthority, Interior, Names, Purpose, SemanticId, Shape, SideDemands, Visitor,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObservationProfile {
    Staged,
    LegacyFull,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Assessment {
    Compare,
    Omit,
}
#[derive(Clone, Copy, Debug)]
pub struct ContextSite<'a> {
    pub relative: &'a Path,
    pub structural_root: bool,
    pub role: Option<crate::InteriorRole>,
}
#[derive(Clone, Copy, Debug)]
pub struct BatchInfo<'a> {
    pub source_root: &'a Path,
    pub target_root: &'a Path,
    pub relative: &'a Path,
    pub structural_root: bool,
    pub purpose: Purpose,
}
/// Borrowed logical coordinate. Only an explicit conversion joins a path.
#[derive(Clone, Copy, Debug)]
pub struct Coordinate<'a> {
    pub parent: &'a Path,
    pub name: &'a std::ffi::OsStr,
    pub structural_root: bool,
}
impl Coordinate<'_> {
    pub fn to_path_buf(self) -> std::path::PathBuf {
        if self.structural_root {
            self.parent.to_owned()
        } else {
            self.parent.join(self.name)
        }
    }
    pub fn components(&self) -> impl Iterator<Item = std::path::Component<'_>> {
        self.parent
            .components()
            .chain((!self.structural_root).then(|| Path::new(self.name).components()).into_iter().flatten())
    }
}

pub trait Policy: Sync {
    type Context: Clone + Eq + Debug + Send + Sync;
    fn identity(&self) -> &SemanticId;
    fn initial_context(&self) -> Self::Context;
    fn scoped(&self) -> bool {
        true
    }
    fn validate_context(&self, _: ContextSite<'_>, _: &Self::Context) -> Result<(), CheckError> {
        Ok(())
    }
    fn observation_profile(&self) -> ObservationProfile {
        ObservationProfile::Staged
    }
    fn names(&self, _: &Self::Context, _: &mut Names) -> Result<(), CheckError> {
        Ok(())
    }
    fn shape(&self, _: &Self::Context, _: &mut ShapeChoices<'_, Self::Context>) -> Result<(), CheckError> {
        Ok(())
    }
    fn select(&self, _: &Self::Context, _: &mut Choices<'_, Self::Context>) -> Result<(), CheckError> {
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Choice<C> {
    pub name: std::ffi::OsString,
    pub shape: Shape,
    pub demand: SideDemands,
    pub assessment: Assessment,
    pub comparison: Comparison,
    pub baseline: Option<Action>,
    pub action: Option<Action>,
    pub install: Option<InstallAuthority>,
    pub interior: Option<Interior<C>>,
    pub terminal: bool,
}
pub struct ShapeChoices<'a, C> {
    pub(crate) info: BatchInfo<'a>,
    pub(crate) entries: &'a mut [Choice<C>],
}
impl<C> ShapeChoices<'_, C> {
    pub fn info(&self) -> BatchInfo<'_> {
        self.info
    }
    pub fn entries_mut(&mut self) -> impl Iterator<Item = ShapeChoice<'_, C>> {
        self.entries.iter_mut().map(|entry| ShapeChoice {
            entry,
            parent: self.info.relative,
            root: self.info.structural_root,
        })
    }
}
pub struct ShapeChoice<'a, C> {
    entry: &'a mut Choice<C>,
    parent: &'a Path,
    root: bool,
}
impl<C> ShapeChoice<'_, C> {
    pub fn name(&self) -> &std::ffi::OsStr {
        &self.entry.name
    }
    pub fn coordinate(&self) -> Coordinate<'_> {
        Coordinate {
            parent: self.parent,
            name: &self.entry.name,
            structural_root: self.root,
        }
    }
    pub fn shape(&self) -> Shape {
        self.entry.shape
    }
    pub fn request(&mut self, demand: SideDemands) {
        self.entry.demand = self.entry.demand.union(demand);
    }
    pub fn omit_comparison(&mut self) {
        self.entry.assessment = Assessment::Omit;
    }
    pub fn set_action(&mut self, action: Action) {
        self.entry.action = Some(action);
    }
    pub fn set_install(&mut self, authority: InstallAuthority) {
        self.entry.install = Some(authority);
    }
    pub fn set_interior(&mut self, interior: Interior<C>) {
        self.entry.interior = Some(interior);
    }
    /// Terminal: final selection cannot reopen this entry or demand its attributes.
    pub fn preserve_tree(&mut self) {
        self.entry.action = Some(Action::PreserveTree);
        self.entry.assessment = Assessment::Omit;
        self.entry.demand = SideDemands::default();
        self.entry.install = None;
        self.entry.interior = None;
        self.entry.terminal = true;
    }
}

pub struct Choices<'a, C> {
    pub(crate) info: BatchInfo<'a>,
    pub(crate) entries: &'a mut [Choice<C>],
    pub(crate) facts: &'a [crate::acquire::EntryRuntime],
}
impl<C> Choices<'_, C> {
    pub fn info(&self) -> BatchInfo<'_> {
        self.info
    }
    pub fn entries_mut(&mut self) -> impl Iterator<Item = SelectionChoice<'_, C>> {
        self.entries.iter_mut().zip(self.facts).map(|(entry, facts)| SelectionChoice {
            entry,
            facts,
            parent: self.info.relative,
            root: self.info.structural_root,
        })
    }
}
pub struct SelectionChoice<'a, C> {
    entry: &'a mut Choice<C>,
    facts: &'a crate::acquire::EntryRuntime,
    parent: &'a Path,
    root: bool,
}
impl<C> SelectionChoice<'_, C> {
    pub fn name(&self) -> &std::ffi::OsStr {
        &self.entry.name
    }
    pub fn coordinate(&self) -> Coordinate<'_> {
        Coordinate {
            parent: self.parent,
            name: &self.entry.name,
            structural_root: self.root,
        }
    }
    pub fn shape(&self) -> Shape {
        self.entry.shape
    }
    /// Only policy-requested facts, never opportunistic comparator/peer facts.
    pub fn facts(&self) -> EntryFacts<'_> {
        self.facts.masked(self.entry.demand)
    }
    pub fn comparison(&self) -> Comparison {
        self.entry.comparison
    }
    pub fn baseline(&self) -> Option<Action> {
        self.entry.baseline
    }
    pub fn action(&self) -> Option<Action> {
        self.entry.action.or(self.entry.baseline)
    }
    pub fn is_terminal(&self) -> bool {
        self.entry.terminal
    }
    pub fn set_action(&mut self, action: Action) {
        if !self.entry.terminal {
            self.entry.action = Some(action);
        }
    }
    pub fn set_install(&mut self, authority: InstallAuthority) {
        if !self.entry.terminal {
            self.entry.install = Some(authority);
        }
    }
    pub fn set_interior(&mut self, interior: Interior<C>) {
        if !self.entry.terminal {
            self.entry.interior = Some(interior);
        }
    }
    pub fn preserve_tree(&mut self) {
        if !self.entry.terminal {
            self.entry.action = Some(Action::PreserveTree);
            self.entry.install = None;
            self.entry.interior = None;
        }
    }
}

/// Pure finite demand table supplied by an execution owner. No I/O is permitted.
pub trait ExecutionDemand: Sync {
    fn for_entry<C>(&self, entry: EntrySelectionRef<'_, C>) -> SideDemands;
}
#[derive(Clone, Copy, Debug, Default)]
pub struct NoExecutionDemand;
impl ExecutionDemand for NoExecutionDemand {
    fn for_entry<C>(&self, _: EntrySelectionRef<'_, C>) -> SideDemands {
        SideDemands::default()
    }
}

static MIRROR_ID: LazyLock<SemanticId> = LazyLock::new(|| SemanticId::new("diff-dirs.mirror", 1, Vec::new()));
/// Baseline mirror; refuses unassessed/unresolved inputs instead of inventing Keep.
#[derive(Clone, Copy, Debug, Default)]
pub struct Mirror;
impl Policy for Mirror {
    type Context = ();
    fn identity(&self) -> &SemanticId {
        &MIRROR_ID
    }
    fn initial_context(&self) {}
    fn scoped(&self) -> bool {
        false
    }
}

/// Full-fact compatibility adapter. Comparator meaning remains explicitly StatKey.
/// Application-owned visitor identity belongs in the enclosing binding descriptor;
/// this ID identifies the adapter contract, not a hash of an arbitrary Rust closure.
#[derive(Clone, Copy, Debug)]
pub struct FullVisitor<V>(pub V);
static FULL_ID: LazyLock<SemanticId> = LazyLock::new(|| SemanticId::new("diff-dirs.full-visitor", 1, Vec::new()));
impl<V: Visitor> Policy for FullVisitor<V> {
    type Context = ();
    fn identity(&self) -> &SemanticId {
        &FULL_ID
    }
    fn initial_context(&self) {}
    fn scoped(&self) -> bool {
        self.0.scoped()
    }
    fn observation_profile(&self) -> ObservationProfile {
        ObservationProfile::LegacyFull
    }
    fn names(&self, _: &(), names: &mut Names) -> Result<(), CheckError> {
        if V::ENABLED { self.0.early(names) } else { Ok(()) }
    }
    fn select(&self, _: &(), batch: &mut Choices<'_, ()>) -> Result<(), CheckError> {
        let mut operations = crate::visitor::legacy_operations(batch.info, batch.entries, batch.facts);
        if V::ENABLED {
            self.0.late(&mut operations)?;
        }
        if operations
            .entries()
            .windows(2)
            .any(|p| p[0].name().as_encoded_bytes() >= p[1].name().as_encoded_bytes())
        {
            return Err("visitor reordered immutable operation slots".into());
        }
        for (choice, operation) in batch.entries.iter_mut().zip(operations.entries()) {
            choice.action = operation.action();
            // Compatibility changes retain their original difference-only grant.
            choice.install = operation.action().and_then(|action| crate::selection::mirror_install(choice.shape, action));
        }
        Ok(())
    }
}

/// A borrowed policy does not alter its context or semantic interpretation.
impl<P: Policy + ?Sized> Policy for &P {
    type Context = P::Context;
    fn identity(&self) -> &SemanticId {
        (**self).identity()
    }
    fn initial_context(&self) -> Self::Context {
        (**self).initial_context()
    }
    fn scoped(&self) -> bool {
        (**self).scoped()
    }
    fn validate_context(&self, at: ContextSite<'_>, value: &Self::Context) -> Result<(), CheckError> {
        (**self).validate_context(at, value)
    }
    fn observation_profile(&self) -> ObservationProfile {
        (**self).observation_profile()
    }
    fn names(&self, context: &Self::Context, names: &mut Names) -> Result<(), CheckError> {
        (**self).names(context, names)
    }
    fn shape(&self, context: &Self::Context, choices: &mut ShapeChoices<'_, Self::Context>) -> Result<(), CheckError> {
        (**self).shape(context, choices)
    }
    fn select(&self, context: &Self::Context, choices: &mut Choices<'_, Self::Context>) -> Result<(), CheckError> {
        (**self).select(context, choices)
    }
}
