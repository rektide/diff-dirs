//! Full-fact compatibility callbacks over the contextual inspection engine.

use std::ffi::{OsStr, OsString};
use std::fs::Metadata;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::{Check, CheckError, EarlyFacts, Entry, Error, NoCheck, NoScope, Purpose, Scope, Side, Visit};

#[path = "shared.rs"]
pub(crate) mod shared;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Presence {
    SourceOnly,
    Both,
    TargetOnly,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Admission {
    Observe,
    PreserveTree,
}

/// One immutable physical-union name. Preservation never fabricates absence.
#[derive(Debug)]
pub struct Name {
    pub(crate) name: OsString,
    pub(crate) relative: PathBuf,
    pub(crate) presence: Presence,
    pub(crate) source: Option<EarlyFacts>,
    pub(crate) target: Option<EarlyFacts>,
    pub(crate) admission: Admission,
}
impl Name {
    pub fn name(&self) -> &OsStr {
        &self.name
    }
    pub fn relative_path(&self) -> &Path {
        &self.relative
    }
    pub fn presence(&self) -> Presence {
        self.presence
    }
    pub fn source(&self) -> Option<&EarlyFacts> {
        self.source.as_ref()
    }
    pub fn target(&self) -> Option<&EarlyFacts> {
        self.target.as_ref()
    }
    pub fn admission(&self) -> Admission {
        self.admission
    }
    pub fn preserve_tree(&mut self) {
        self.admission = Admission::PreserveTree;
    }
    pub fn refuse(&self, cause: impl Into<CheckError>) -> CheckError {
        SlotRefusal::at(&self.relative, &self.name, cause.into())
    }
}
/// Native-byte ordered physical union. Hints are the original listing hints,
/// never strengthened by cached acquisition from another lane or earlier stage.
#[derive(Debug)]
pub struct Names {
    pub(crate) dir: PathBuf,
    pub(crate) root: bool,
    pub(crate) entries: Vec<Name>,
}
impl Names {
    pub fn dir(&self) -> &Path {
        &self.dir
    }
    pub fn is_root(&self) -> bool {
        self.root
    }
    pub fn entries(&self) -> &[Name] {
        &self.entries
    }
    pub fn entries_mut(&mut self) -> impl Iterator<Item = &mut Name> {
        self.entries.iter_mut()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Container {
    Keep,
    Create,
    CreateIfNeeded,
    Remove,
}
/// Finite physical work, independent of comparison. New policies must attach
/// explicit installation authority; FullVisitor retains difference-only behavior.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    PreserveTree,
    KeepLeaf,
    InstallLeaf,
    RemoveLeaf,
    /// Settle a declared obsolete owned leaf found absent. Listed domain only.
    ConfirmAbsent,
    Directory {
        container: Container,
        descend: bool,
    },
    ReplaceWithLeaf,
    ReplaceWithDirectory {
        descend: bool,
    },
}

/// Actual full no-follow facts, available only in the compatibility projection.
#[derive(Debug)]
pub struct Facts {
    pub(crate) metadata: Arc<Metadata>,
    pub(crate) link: Option<Arc<PathBuf>>,
}
impl Facts {
    pub fn metadata(&self) -> &Metadata {
        &self.metadata
    }
    pub fn link_target(&self) -> Option<&Path> {
        self.link.as_deref().map(PathBuf::as_path)
    }
}
/// Mutable callback-only operation. It is consumed before immutable publication;
/// no copy of these action slots survives beside DirectorySelection.
#[derive(Debug)]
pub struct Operation {
    name: OsString,
    relative: PathBuf,
    source: Option<Arc<Facts>>,
    target: Option<Arc<Facts>>,
    different: Option<bool>,
    action: Option<Action>,
    baseline: Option<Action>,
}
impl Operation {
    pub fn name(&self) -> &OsStr {
        &self.name
    }
    pub fn relative_path(&self) -> &Path {
        &self.relative
    }
    pub fn source(&self) -> Option<&Facts> {
        self.source.as_deref()
    }
    pub fn target(&self) -> Option<&Facts> {
        self.target.as_deref()
    }
    pub fn presence(&self) -> Presence {
        match (self.source.is_some(), self.target.is_some()) {
            (true, true) => Presence::Both,
            (true, false) => Presence::SourceOnly,
            (false, true) => Presence::TargetOnly,
            _ => unreachable!(),
        }
    }
    pub fn is_different(&self) -> bool {
        self.different == Some(true)
    }
    /// None denotes unsupported physical kinds in this StatKey-only projection.
    pub fn comparison(&self) -> Option<bool> {
        self.different
    }
    pub fn action(&self) -> Option<Action> {
        self.action
    }
    pub fn baseline(&self) -> Option<Action> {
        self.baseline
    }
    pub fn set_action(&mut self, action: Action) {
        self.action = Some(action);
    }
    pub fn preserve_tree(&mut self) {
        self.set_action(Action::PreserveTree);
    }
    pub fn refuse(&self, cause: impl Into<CheckError>) -> CheckError {
        SlotRefusal::at(&self.relative, &self.name, cause.into())
    }
    pub fn retain(&mut self) {
        self.action = Some(if self.different.is_none() {
            Action::PreserveTree
        } else if self.target().is_some_and(|f| f.metadata.is_dir()) {
            Action::Directory {
                container: Container::Keep,
                descend: true,
            }
        } else if self.target.is_some() {
            Action::KeepLeaf
        } else {
            Action::PreserveTree
        });
    }
    pub fn descends(&self) -> bool {
        matches!(
            self.action,
            Some(Action::Directory { descend: true, .. } | Action::ReplaceWithDirectory { descend: true })
        )
    }
    pub fn has_work(&self) -> bool {
        self.action.is_some_and(crate::selection::has_namespace_work)
    }
}
#[derive(Debug)]
pub struct Operations {
    source_root: PathBuf,
    target_root: PathBuf,
    dir: PathBuf,
    root: bool,
    purpose: Purpose,
    entries: Vec<Operation>,
}
impl Operations {
    pub fn source_root(&self) -> &Path {
        &self.source_root
    }
    pub fn target_root(&self) -> &Path {
        &self.target_root
    }
    pub fn dir(&self) -> &Path {
        &self.dir
    }
    pub fn is_root(&self) -> bool {
        self.root
    }
    pub fn purpose(&self) -> Purpose {
        self.purpose
    }
    pub fn entries(&self) -> &[Operation] {
        &self.entries
    }
    pub fn entries_mut(&mut self) -> impl Iterator<Item = &mut Operation> {
        self.entries.iter_mut()
    }
}
pub(crate) fn legacy_operations(info: crate::BatchInfo<'_>, choices: &[crate::policy::Choice<()>], facts: &[crate::acquire::EntryRuntime]) -> Operations {
    Operations {
        source_root: info.source_root.to_owned(),
        target_root: info.target_root.to_owned(),
        dir: info.relative.to_owned(),
        root: info.structural_root,
        purpose: info.purpose,
        entries: choices
            .iter()
            .zip(facts)
            .map(|(c, f)| Operation {
                name: c.name.clone(),
                relative: if info.structural_root { PathBuf::new() } else { info.relative.join(&c.name) },
                source: f.source.as_ref().and_then(|f| f.legacy.clone()),
                target: f.target.as_ref().and_then(|f| f.legacy.clone()),
                different: c.comparison.legacy(),
                action: c.action.or(c.baseline),
                baseline: c.baseline,
            })
            .collect(),
    }
}

/// Pure speculative callbacks. Existing compatibility check/absence/preset
/// composition precedes the caller's late visitor, which may override it.
pub trait Visitor: Sync {
    /// False promises unconditional no-op callbacks; root callback-only work may
    /// also be skipped. Structural binding validation is never skipped.
    const ENABLED: bool = true;
    /// Conservative coverage declaration. Actual exclusions/stopped paired
    /// descent still limit coverage even when this returns false.
    fn scoped(&self) -> bool {
        Self::ENABLED
    }
    fn early(&self, _: &mut Names) -> Result<(), CheckError> {
        Ok(())
    }
    fn late(&self, _: &mut Operations) -> Result<(), CheckError> {
        Ok(())
    }
    fn borrowed(&self) -> VisitorRef<'_, Self>
    where
        Self: Sized,
    {
        VisitorRef(self)
    }
}
#[derive(Clone, Copy, Debug, Default)]
pub struct NoVisitor;
impl Visitor for NoVisitor {
    const ENABLED: bool = false;
}
#[derive(Clone, Copy, Debug)]
pub struct VisitorRef<'a, V: ?Sized>(pub &'a V);
impl<V: Visitor + ?Sized> Visitor for VisitorRef<'_, V> {
    const ENABLED: bool = V::ENABLED;
    fn scoped(&self) -> bool {
        self.0.scoped()
    }
    fn early(&self, b: &mut Names) -> Result<(), CheckError> {
        self.0.early(b)
    }
    fn late(&self, b: &mut Operations) -> Result<(), CheckError> {
        self.0.late(b)
    }
}
#[derive(Debug)]
pub struct CheckedScope<C = NoCheck, S = NoScope> {
    pub check: C,
    pub scope: S,
    roots_checked: bool,
}
impl<C, S> CheckedScope<C, S> {
    pub fn new(check: C, scope: S) -> Self {
        Self {
            check,
            scope,
            roots_checked: false,
        }
    }
    pub(crate) fn roots_checked(mut self) -> Self {
        self.roots_checked = true;
        self
    }
}
impl<C: Check, S: Scope> Visitor for CheckedScope<C, S> {
    const ENABLED: bool = C::ENABLED || S::ENABLED;
    fn scoped(&self) -> bool {
        S::ENABLED
    }
    fn early(&self, b: &mut Names) -> Result<(), CheckError> {
        if S::ENABLED {
            for name in b.entries_mut() {
                if self.scope.at(name.relative_path()) == Visit::PreserveTree {
                    name.preserve_tree();
                }
            }
        }
        Ok(())
    }
    fn late(&self, b: &mut Operations) -> Result<(), CheckError> {
        if C::ENABLED && !(b.is_root() && self.roots_checked) {
            for op in b.entries() {
                for (side, root, facts) in [(Side::Original, b.source_root(), op.source()), (Side::Target, b.target_root(), op.target())] {
                    if let Some(facts) = facts {
                        let path = root.join(op.relative_path());
                        self.check
                            .check(Entry {
                                root,
                                path: &path,
                                name: if b.is_root() {
                                    root.file_name().unwrap_or(root.as_os_str())
                                } else {
                                    op.name()
                                },
                                side,
                                purpose: b.purpose(),
                                metadata: facts.metadata(),
                                link_target: facts.link_target(),
                            })
                            .map_err(|source| Box::new(CheckedRefusal(Error::Refused { path, source })) as CheckError)?;
                    }
                }
            }
        }
        Ok(())
    }
}
#[derive(Debug)]
struct CheckedRefusal(Error);
impl std::fmt::Display for CheckedRefusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
impl std::error::Error for CheckedRefusal {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.0)
    }
}
#[derive(Debug)]
struct SlotRefusal {
    child: Option<OsString>,
    cause: CheckError,
}
impl SlotRefusal {
    fn at(relative: &Path, name: &OsStr, cause: CheckError) -> CheckError {
        Box::new(Self {
            child: (!relative.as_os_str().is_empty()).then(|| name.to_owned()),
            cause,
        })
    }
}
impl std::fmt::Display for SlotRefusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.cause.fmt(f)
    }
}
impl std::error::Error for SlotRefusal {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&*self.cause)
    }
}
pub(crate) fn refusal(path: &Path, source: CheckError) -> Error {
    let source = match source.downcast::<CheckedRefusal>() {
        Ok(e) => return e.0,
        Err(e) => e,
    };
    match source.downcast::<SlotRefusal>() {
        Ok(e) => Error::Refused {
            path: e.child.map_or_else(|| path.to_owned(), |n| path.join(n)),
            source: e.cause,
        },
        Err(source) => Error::Refused { path: path.to_owned(), source },
    }
}

#[allow(clippy::too_many_arguments)]
pub fn inspect_directory<V: Visitor>(
    source_root: &Path,
    target_root: &Path,
    dir: &Path,
    source: Option<&Path>,
    target: Option<&Path>,
    purpose: Purpose,
    visitor: &V,
) -> Result<crate::DirectoryPlan, Error> {
    inspect_directory_execution(source_root, target_root, dir, source, target, purpose, visitor, crate::Execution::Sequential)
}
#[allow(clippy::too_many_arguments)]
pub(crate) fn inspect_directory_execution<V: Visitor>(
    source_root: &Path,
    target_root: &Path,
    dir: &Path,
    source: Option<&Path>,
    target: Option<&Path>,
    purpose: Purpose,
    visitor: &V,
    execution: crate::Execution,
) -> Result<crate::DirectoryPlan, Error> {
    shared::inspect_legacy_directory(source_root, target_root, dir, source, target, purpose, visitor, execution)
}
pub fn inspect_root<V: Visitor>(source: &Path, target: &Path, purpose: Purpose, visitor: &V) -> Result<crate::DirectoryPlan, Error> {
    shared::inspect_legacy_root_paths(source, target, purpose, visitor)
}
pub(crate) fn inspect_bound_root<V: Visitor>(source: &Path, target: &Path, s: &Metadata, t: &Metadata, visitor: &V) -> Result<crate::DirectoryPlan, Error> {
    shared::inspect_legacy_root(source, target, Some(s.clone()), Some(t.clone()), Purpose::Compare, visitor)
}
pub fn inspect_single_root<V: Visitor>(root: &Path, side: Side, purpose: Purpose, visitor: &V) -> Result<crate::DirectoryPlan, Error> {
    let m = diff_dirs_tree_observe::symlink_metadata(root)?;
    let (s, t) = if side == Side::Original { (Some(m), None) } else { (None, Some(m)) };
    shared::inspect_legacy_root(root, root, s, t, purpose, visitor)
}
