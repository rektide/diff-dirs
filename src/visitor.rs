//! Directory-local, compiled steering over immutable observations.

use std::ffi::{OsStr, OsString};
use std::fs::{self, Metadata};
use std::path::{Path, PathBuf};

use crate::{Check, CheckError, DirDiff, Entry, Error, NoCheck, NoScope, Purpose, Scope, Side, Visit};

/// Presence from immediate directory listings, before entry metadata or readlink.
/// Suppression never changes this fact and therefore cannot manufacture absence.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Presence {
    SourceOnly,
    Both,
    TargetOnly,
}

/// Pair-wide early admission. Preserving a name preserves its entire subtree.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Admission {
    Observe,
    PreserveTree,
}

/// One immutable union name and its mutable admission. Slots cannot be inserted,
/// renamed, reordered, or have their side presence edited by a visitor.
#[derive(Debug)]
pub struct Name {
    name: OsString,
    relative: PathBuf,
    presence: Presence,
    admission: Admission,
}

impl Name {
    pub fn name(&self) -> &OsStr {
        &self.name
    }
    /// Logical final root-relative coordinate; empty for a structural root slot.
    pub fn relative_path(&self) -> &Path {
        &self.relative
    }
    pub fn presence(&self) -> Presence {
        self.presence
    }
    pub fn admission(&self) -> Admission {
        self.admission
    }
    /// Suppress both observations and all descendant admission, not just copying.
    pub fn preserve_tree(&mut self) {
        self.admission = Admission::PreserveTree;
    }
}

/// Sorted union of immediate names. The callback runs after `read_dir`, but
/// before stat/readlink of **any child** in this batch. Listing the containing
/// directory and mandatory structural root/ancestor validation are not skipped.
/// No type, mtime, link text, mount, or repository probes are performed for early
/// context. A preserved child is not statted, readlinked, or entered.
#[derive(Debug)]
pub struct Names {
    dir: PathBuf,
    root: bool,
    entries: Vec<Name>,
}

impl Names {
    pub fn dir(&self) -> &Path {
        &self.dir
    }
    /// A singleton for the actual bound root, not an ordinary directory listing.
    /// Root structural metadata is mandatory even when this slot is preserved.
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

/// Local container work, independent of descending into its children.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Container {
    Keep,
    Create,
    Remove,
}

/// Final work for one named entry. This is not its observed difference.
///
/// | Observed shape | Permitted work (in addition to PreserveTree) |
/// |---|---|
/// | source-only leaf | KeepLeaf, InstallLeaf |
/// | target-only leaf | KeepLeaf, RemoveLeaf |
/// | two leaves | KeepLeaf; InstallLeaf only when different |
/// | source-only directory | Create directory, optionally descend |
/// | target-only directory | Keep/Remove container and descend; Keep may stop |
/// | two directories | Keep container, descend or stop |
/// | source leaf / target directory | ReplaceWithLeaf, or keep target directory |
/// | source directory / target leaf | ReplaceWithDirectory, or KeepLeaf |
///
/// Equal leaves cannot be force-installed. Replacements are **coupled**: no
/// delete-half/create-half authority exists. Removing a directory requires
/// descent; surviving descendants retain necessary ancestors. A source-only
/// directory cannot Keep an absent container while admitting child writes.
/// PreserveTree always stops descent and performs no local work. Keeping a
/// directory container while descending permits selected pruning below it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    PreserveTree,
    KeepLeaf,
    InstallLeaf,
    RemoveLeaf,
    Directory { container: Container, descend: bool },
    ReplaceWithLeaf,
    ReplaceWithDirectory { descend: bool },
}

/// Already-needed no-follow facts; never a certificate valid after writers resume.
/// Literal link text is supplied when comparison/copy needs it. Target-only
/// deletion does not read links merely to populate a callback.
#[derive(Debug)]
pub struct Facts {
    metadata: Metadata,
    link: Option<PathBuf>,
}

impl Facts {
    pub fn metadata(&self) -> &Metadata {
        &self.metadata
    }
    pub fn link_target(&self) -> Option<&Path> {
        self.link.as_deref()
    }
}

/// Immutable name, presence, facts and comparison, with one mutable action.
/// Public fields/constructors cannot inject paths or fabricate observations.
#[derive(Debug)]
pub struct Operation {
    name: OsString,
    relative: PathBuf,
    source: Option<Facts>,
    target: Option<Facts>,
    different: bool,
    action: Action,
    baseline: Action,
}

impl Operation {
    pub fn name(&self) -> &OsStr {
        &self.name
    }
    pub fn relative_path(&self) -> &Path {
        &self.relative
    }
    pub fn source(&self) -> Option<&Facts> {
        self.source.as_ref()
    }
    pub fn target(&self) -> Option<&Facts> {
        self.target.as_ref()
    }
    pub fn presence(&self) -> Presence {
        presence(self.source.is_some(), self.target.is_some())
    }
    /// Immediate stat-key/literal-link delta, independent of steering or progress.
    /// Matched directories have no immediate difference; their children may differ.
    pub fn is_different(&self) -> bool {
        self.different
    }
    pub fn action(&self) -> Action {
        self.action
    }
    pub fn baseline(&self) -> Action {
        self.baseline
    }
    /// Change work only. The complete batch is validated after late returns,
    /// before application or child publication, even if an invalid action is
    /// later never executed. No partial entry operation is published on error.
    pub fn set_action(&mut self, action: Action) {
        self.action = action;
    }
    /// No local writes or descendant work, including at type collisions.
    pub fn preserve_tree(&mut self) {
        self.action = Action::PreserveTree;
    }
    /// Keep an existing target container but allow descendant work. For an
    /// absent target container this stops the subtree instead of authorizing
    /// impossible child creation. Existing leaves are simply retained.
    pub fn retain(&mut self) {
        self.action = if self.target.as_ref().is_some_and(|f| f.metadata.is_dir()) {
            Action::Directory {
                container: Container::Keep,
                descend: true,
            }
        } else if self.target.is_some() {
            Action::KeepLeaf
        } else {
            Action::PreserveTree
        };
    }
    /// Whether this entry's directory children may be inspected. Replacement
    /// preparation and target removal are handled by the mutation consumer.
    pub fn descends(&self) -> bool {
        matches!(
            self.action,
            Action::Directory { descend: true, .. } | Action::ReplaceWithDirectory { descend: true }
        )
    }
    pub fn has_work(&self) -> bool {
        matches!(
            self.action,
            Action::InstallLeaf
                | Action::RemoveLeaf
                | Action::ReplaceWithLeaf
                | Action::ReplaceWithDirectory { .. }
                | Action::Directory {
                    container: Container::Create | Container::Remove,
                    ..
                }
        )
    }
    fn validate(&self) -> Result<(), CheckError> {
        let s = self.source.as_ref().map(|f| &f.metadata);
        let t = self.target.as_ref().map(|f| &f.metadata);
        let sd = s.is_some_and(Metadata::is_dir);
        let td = t.is_some_and(Metadata::is_dir);
        let leaf = |m: &Metadata| m.is_file() || m.is_symlink();
        let valid = match self.action {
            Action::PreserveTree => true,
            Action::KeepLeaf => t.is_some_and(leaf) || (t.is_none() && s.is_some_and(leaf)),
            Action::InstallLeaf => s.is_some_and(leaf) && !td && self.different,
            Action::RemoveLeaf => s.is_none() && t.is_some_and(leaf),
            Action::ReplaceWithLeaf => s.is_some_and(leaf) && td,
            Action::ReplaceWithDirectory { .. } => sd && t.is_some_and(leaf),
            Action::Directory {
                container: Container::Create, ..
            } => sd && t.is_none(),
            Action::Directory {
                container: Container::Keep, ..
            } => td,
            Action::Directory {
                container: Container::Remove,
                descend,
            } => s.is_none() && td && descend,
        };
        if valid {
            Ok(())
        } else {
            Err("action is incompatible with observed entry or structural dependencies".into())
        }
    }
}

/// Directory-local late context, including equal leaves and matched directory
/// descent. Facts are read-only; only existing action slots are mutable.
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
    pub fn dir(&self) -> &Path {
        &self.dir
    }
    pub fn is_root(&self) -> bool {
        self.root
    }
    pub fn purpose(&self) -> Purpose {
        self.purpose
    }
    pub fn source_root(&self) -> &Path {
        &self.source_root
    }
    pub fn target_root(&self) -> &Path {
        &self.target_root
    }
    pub fn entries(&self) -> &[Operation] {
        &self.entries
    }
    pub fn entries_mut(&mut self) -> impl Iterator<Item = &mut Operation> {
        self.entries.iter_mut()
    }
}

/// Immutable finalized directory batch. Produced only by actual inspection;
/// no public construction or action mutation exists after validation.
#[derive(Debug)]
pub struct DirectoryPlan {
    operations: Operations,
    diff: DirDiff,
    excluded: u64,
    steered: bool,
    first_excluded: Option<PathBuf>,
    preserved: Vec<OsString>,
}

impl DirectoryPlan {
    pub fn operations(&self) -> &Operations {
        &self.operations
    }
    /// Baseline observed delta, not a filtered application report.
    pub fn diff(&self) -> &DirDiff {
        &self.diff
    }
    pub fn excluded(&self) -> u64 {
        self.excluded
    }
    /// One bounded diagnostic example, not an inventory of hidden names.
    pub fn first_excluded(&self) -> Option<&Path> {
        self.first_excluded.as_deref()
    }
    /// Whether this immediate native name was excluded before observation.
    /// Kept directory-local for safe public-record validation, never recursive.
    pub fn preserves(&self, name: &OsStr) -> bool {
        self.preserved.iter().any(|n| n == name)
    }
    pub fn steered(&self) -> bool {
        self.steered
    }
    pub fn retained(&self) -> u64 {
        self.operations.entries.iter().filter(|e| e.action != e.baseline).count() as u64
    }
    /// Immediate planned namespace actions, not recursive or completed work.
    pub fn work(&self) -> u64 {
        self.operations.entries.iter().filter(|e| e.has_work()).count() as u64
    }
    pub(crate) fn children(&self) -> Vec<OsString> {
        self.operations
            .entries
            .iter()
            .filter(|e| e.descends() && e.source.as_ref().is_some_and(|f| f.metadata.is_dir()) && e.target.as_ref().is_some_and(|f| f.metadata.is_dir()))
            .map(|e| e.name.clone())
            .collect()
    }
}

/// Pure compiled early/late steering. `&self` and directory-local mutable
/// contexts permit parallel inspection without a stateful policy scheduler.
/// Calls can be speculative, concurrent, repeated, or never committed: they are
/// not ordered/exactly-once notifications. Decisions must depend only on frozen
/// caller state and supplied facts. Do not perform filesystem I/O/mutation,
/// spawn processes, print, or make decisions depend on invocation count/order.
/// A refusal aborts this batch before its mutation/admission; earlier batches or
/// already-started parallel siblings can have completed. No rollback or bounded
/// cancellation latency is promised for a stalled filesystem call.
///
/// ```
/// use diff_dirs::{Visitor, Names, Operations, Presence, CheckError};
/// struct Seed;
/// impl Visitor for Seed {
///     fn early(&self, names: &mut Names) -> Result<(), CheckError> {
///         for name in names.entries_mut() {
///             if name.name() == ".jj" { name.preserve_tree(); }
///         }
///         Ok(())
///     }
///     fn late(&self, batch: &mut Operations) -> Result<(), CheckError> {
///         for entry in batch.entries_mut() {
///             if entry.presence() != Presence::SourceOnly { entry.retain(); }
///         }
///         Ok(())
///     }
/// }
/// ```
pub trait Visitor: Sync {
    /// False promises both callbacks are unconditional no-ops, with no reliance
    /// on invocation. It permits skipping dispatch and callback error mapping;
    /// observations and plan construction needed by the operation still run.
    /// Leave true for any steering/refusal. No measured speedup is claimed.
    const ENABLED: bool = true;
    /// Whether mere exhaustion must not certify unqualified full-root coverage.
    /// Refusal-only compatibility checks override this; steering visitors keep
    /// the conservative default even if no excluded name happened to occur.
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

/// Unmodified normal mirror actions; stat-equal leaves remain no-ops.
#[derive(Clone, Copy, Debug, Default)]
pub struct NoVisitor;
impl Visitor for NoVisitor {
    const ENABLED: bool = false;
}

#[derive(Clone, Copy, Debug)]
pub struct VisitorRef<'a, V: ?Sized>(pub &'a V);
impl<V: Visitor + ?Sized> Visitor for VisitorRef<'_, V> {
    const ENABLED: bool = V::ENABLED;
    fn early(&self, batch: &mut Names) -> Result<(), CheckError> {
        self.0.early(batch)
    }
    fn late(&self, batch: &mut Operations) -> Result<(), CheckError> {
        self.0.late(batch)
    }
    fn scoped(&self) -> bool {
        self.0.scoped()
    }
}

/// Thin compatibility projection of old path scope and side-local refusal.
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
    fn early(&self, batch: &mut Names) -> Result<(), CheckError> {
        if S::ENABLED {
            for name in batch.entries_mut() {
                if self.scope.at(name.relative_path()) == Visit::PreserveTree {
                    name.preserve_tree();
                }
            }
        }
        Ok(())
    }
    fn late(&self, batch: &mut Operations) -> Result<(), CheckError> {
        if C::ENABLED && !(batch.is_root() && self.roots_checked) {
            for operation in batch.entries() {
                for (side, root, fact) in [
                    (Side::Original, batch.source_root(), operation.source()),
                    (Side::Target, batch.target_root(), operation.target()),
                ] {
                    if let Some(fact) = fact {
                        let path = root.join(operation.relative_path());
                        self.check
                            .check(Entry {
                                root,
                                path: &path,
                                name: if batch.is_root() {
                                    root.file_name().unwrap_or(root.as_os_str())
                                } else {
                                    operation.name()
                                },
                                side,
                                purpose: batch.purpose(),
                                metadata: fact.metadata(),
                                link_target: fact.link_target(),
                            })
                            .map_err(|source| Box::new(Error::Refused { path, source }) as CheckError)?;
                    }
                }
            }
        }
        Ok(())
    }
}

fn presence(source: bool, target: bool) -> Presence {
    match (source, target) {
        (true, true) => Presence::Both,
        (true, false) => Presence::SourceOnly,
        (false, true) => Presence::TargetOnly,
        _ => unreachable!(),
    }
}

fn facts(path: &Path, read_link: bool) -> Result<Facts, Error> {
    let metadata = fs::symlink_metadata(path).map_err(|e| Error::io(path.to_path_buf(), "inspect entry", e))?;
    let link = if metadata.is_symlink() && read_link {
        Some(fs::read_link(path).map_err(|e| Error::io(path.to_path_buf(), "read symlink", e))?)
    } else {
        None
    };
    Ok(Facts { metadata, link })
}

fn operation(name: OsString, relative: PathBuf, source: Option<Facts>, target: Option<Facts>) -> Result<Operation, Error> {
    let sd = source.as_ref().is_some_and(|f| f.metadata.is_dir());
    let td = target.as_ref().is_some_and(|f| f.metadata.is_dir());
    let different = match (&source, &target) {
        (Some(_), Some(_)) if sd && td => false,
        (Some(s), Some(t)) if s.metadata.is_file() && t.metadata.is_file() => {
            s.metadata.len() != t.metadata.len()
                || s.metadata.modified().map_err(|e| Error::io(relative.clone(), "read modification time", e))?
                    != t.metadata.modified().map_err(|e| Error::io(relative.clone(), "read modification time", e))?
        }
        (Some(s), Some(t)) if s.metadata.is_symlink() && t.metadata.is_symlink() => {
            s.link.as_ref().map(|p| p.as_os_str()) != t.link.as_ref().map(|p| p.as_os_str())
        }
        _ => true,
    };
    let action = match (source.is_some(), target.is_some(), sd, td) {
        (true, true, true, true) => Action::Directory {
            container: Container::Keep,
            descend: true,
        },
        (true, true, true, false) => Action::ReplaceWithDirectory { descend: true },
        (true, true, false, true) => Action::ReplaceWithLeaf,
        (true, _, true, _) => Action::Directory {
            container: Container::Create,
            descend: true,
        },
        (_, true, _, true) => Action::Directory {
            container: Container::Remove,
            descend: true,
        },
        (true, _, _, _) if different => Action::InstallLeaf,
        (false, true, _, _) => Action::RemoveLeaf,
        _ => Action::KeepLeaf,
    };
    Ok(Operation {
        name,
        relative,
        source,
        target,
        different,
        action,
        baseline: action,
    })
}

fn finish<V: Visitor>(mut operations: Operations, excluded: u64, visitor: &V) -> Result<DirectoryPlan, Error> {
    if V::ENABLED {
        visitor
            .late(&mut operations)
            .map_err(|source| refusal(&operations.target_root.join(&operations.dir), source))?;
    }
    if operations
        .entries
        .windows(2)
        .any(|pair| pair[0].name.as_encoded_bytes() >= pair[1].name.as_encoded_bytes())
    {
        return Err(refusal(
            &operations.target_root.join(&operations.dir),
            "visitor reordered immutable operation slots".into(),
        ));
    }
    if operations.root && excluded != 0 {
        operations.entries[0].preserve_tree();
    }
    let mut diff = DirDiff {
        dir: operations.dir.clone(),
        ..DirDiff::default()
    };
    for entry in &operations.entries {
        let path = operations.target_root.join(&entry.relative);
        if entry.action != Action::PreserveTree {
            for (root, facts) in [(&operations.source_root, &entry.source), (&operations.target_root, &entry.target)] {
                if let Some(facts) = facts {
                    if !(facts.metadata.is_dir() || facts.metadata.is_file() || facts.metadata.is_symlink()) {
                        return Err(Error::UnsupportedEntry {
                            path: root.join(&entry.relative),
                        });
                    }
                }
            }
        }
        entry.validate().map_err(|source| refusal(&path, source))?;
        if entry.different {
            let sd = entry.source.as_ref().is_some_and(|f| f.metadata.is_dir());
            let td = entry.target.as_ref().is_some_and(|f| f.metadata.is_dir());
            if entry.source.is_some() && entry.target.is_some() && !sd && !td {
                diff.files.changed.push(entry.name.clone());
            } else {
                if entry.source.is_some() {
                    if sd {
                        diff.dirs.added.push(entry.name.clone());
                    } else {
                        diff.files.added.push(entry.name.clone());
                    }
                }
                if entry.target.is_some() {
                    if td {
                        diff.dirs.removed.push(entry.name.clone());
                    } else {
                        diff.files.removed.push(entry.name.clone());
                    }
                }
            }
        }
    }
    Ok(DirectoryPlan {
        operations,
        diff,
        excluded,
        steered: visitor.scoped(),
        first_excluded: None,
        preserved: Vec::new(),
    })
}

fn refusal(path: &Path, source: CheckError) -> Error {
    // Preserve the precise side-local coordinate carried by compatibility checks.
    match source.downcast::<Error>() {
        Ok(error) => *error,
        Err(source) => Error::Refused {
            path: path.to_path_buf(),
            source,
        },
    }
}

/// Inspect an actual immediate directory union and finalize one visitor batch.
/// `source`/`target` are physical containing directories or absent sides; logical
/// roots plus `dir` name final destinations even during private preparation.
/// A present side must be a real directory (the caller validates admitted routing
/// ancestors). This function never writes and never follows child symlinks.
/// No whole-tree catalog/preaudit or additional policy-only I/O is performed.
pub fn inspect_directory<V: Visitor>(
    source_root: &Path,
    target_root: &Path,
    dir: &Path,
    source: Option<&Path>,
    target: Option<&Path>,
    purpose: Purpose,
    visitor: &V,
) -> Result<DirectoryPlan, Error> {
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
) -> Result<DirectoryPlan, Error> {
    if dir.components().any(|c| !matches!(c, std::path::Component::Normal(_))) {
        return Err(refusal(dir, "directory must be a normal root-relative path".into()));
    }
    let mut raw = Vec::new();
    for (side, path) in [(Side::Original, source), (Side::Target, target)] {
        if let Some(path) = path {
            for entry in fs::read_dir(path).map_err(|e| Error::io(path.to_path_buf(), "read directory", e))? {
                let entry = entry.map_err(|e| Error::io(path.to_path_buf(), "read directory entry", e))?;
                raw.push((entry.file_name(), side));
            }
        }
    }
    raw.sort_unstable_by(|a, b| a.0.as_encoded_bytes().cmp(b.0.as_encoded_bytes()).then((a.1 as u8).cmp(&(b.1 as u8))));
    let mut names = Names {
        dir: dir.to_path_buf(),
        root: false,
        entries: Vec::with_capacity(raw.len()),
    };
    let mut raw = raw.into_iter().peekable();
    while let Some((name, side)) = raw.next() {
        let both = raw.peek().is_some_and(|(next, _)| *next == name);
        if both {
            raw.next();
        }
        names.entries.push(Name {
            relative: dir.join(&name),
            name,
            presence: if both {
                Presence::Both
            } else if side == Side::Original {
                Presence::SourceOnly
            } else {
                Presence::TargetOnly
            },
            admission: Admission::Observe,
        });
    }
    if V::ENABLED {
        visitor.early(&mut names).map_err(|source| refusal(&target_root.join(dir), source))?;
    }
    if names
        .entries
        .windows(2)
        .any(|pair| pair[0].name.as_encoded_bytes() >= pair[1].name.as_encoded_bytes())
    {
        return Err(refusal(&target_root.join(dir), "visitor reordered immutable union names".into()));
    }
    let mut operations = Operations {
        source_root: source_root.to_path_buf(),
        target_root: target_root.to_path_buf(),
        dir: dir.to_path_buf(),
        root: false,
        purpose,
        entries: Vec::with_capacity(names.entries.len()),
    };
    let mut excluded = 0;
    let mut first_excluded = None;
    let mut preserved = Vec::new();
    let admitted: Vec<_> = names
        .entries
        .into_iter()
        .filter_map(|name| {
            if name.admission == Admission::PreserveTree {
                excluded += 1;
                if first_excluded.is_none() {
                    first_excluded = Some(name.relative);
                }
                preserved.push(name.name);
                None
            } else {
                Some(name)
            }
        })
        .collect();
    let observe = |name: Name| {
        #[cfg(all(test, feature = "parallel"))]
        crate::pair::tests::observe_worker();
        let s = if name.presence != Presence::TargetOnly {
            Some(facts(&source.expect("listed source").join(&name.name), true)?)
        } else {
            None
        };
        let t = if name.presence != Presence::SourceOnly {
            Some(facts(
                &target.expect("listed target").join(&name.name),
                name.presence == Presence::Both || purpose == Purpose::Audit,
            )?)
        } else {
            None
        };
        operation(name.name, name.relative, s, t)
    };
    #[cfg(feature = "parallel")]
    let observed: Result<Vec<_>, _> = if execution == crate::Execution::Parallel && admitted.len() >= 64 {
        use rayon::prelude::*;
        // Indexed results preserve union-name error order; no early-return race
        // selects the failure. Every started metadata operation joins.
        admitted.into_par_iter().with_min_len(8).map(observe).collect::<Vec<_>>().into_iter().collect()
    } else {
        admitted.into_iter().map(observe).collect()
    };
    #[cfg(not(feature = "parallel"))]
    let observed: Result<Vec<_>, _> = {
        let _ = execution;
        admitted.into_iter().map(observe).collect()
    };
    operations.entries = observed?;
    let mut plan = finish(operations, excluded, visitor)?;
    plan.first_excluded = first_excluded;
    plan.preserved = preserved;
    Ok(plan)
}

/// Finalize a singleton actual root operation. Mandatory structural root facts
/// precede early steering; preserving the root prevents listing its contents.
/// This also represents absent-target initialization and explicitly bound root
/// leaf replacement, without a fictitious parent listing or fabricated metadata.
pub fn inspect_root<V: Visitor>(source: &Path, target: &Path, purpose: Purpose, visitor: &V) -> Result<DirectoryPlan, Error> {
    let s = facts(source, true)?;
    let t = match fs::symlink_metadata(target) {
        Ok(_) => Some(facts(target, true)?),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
        Err(e) => return Err(Error::io(target.to_path_buf(), "inspect root", e)),
    };
    let mut names = Names {
        dir: PathBuf::new(),
        root: true,
        entries: vec![Name {
            name: source.file_name().unwrap_or(source.as_os_str()).to_os_string(),
            relative: PathBuf::new(),
            presence: presence(true, t.is_some()),
            admission: Admission::Observe,
        }],
    };
    if V::ENABLED {
        visitor.early(&mut names).map_err(|source| refusal(target, source))?;
    }
    let excluded = u64::from(names.entries[0].admission == Admission::PreserveTree);
    let mut entry = operation(names.entries.remove(0).name, PathBuf::new(), Some(s), t)?;
    if excluded != 0 {
        entry.preserve_tree();
    }
    // Structural root checks remain visible even when early preserves contents.
    let plan = finish(
        Operations {
            source_root: source.to_path_buf(),
            target_root: target.to_path_buf(),
            dir: PathBuf::new(),
            root: true,
            purpose,
            entries: vec![entry],
        },
        excluded,
        visitor,
    )?;
    Ok(plan)
}

/// Inspect one actual bound tree root for explicit audit or container clearing.
/// The target directory root is a structurally retained container, not a proposal
/// to delete the selected root. Its children use ordinary TargetOnly actions.
/// Root metadata still precedes early steering; a preserved root is not listed.
pub fn inspect_single_root<V: Visitor>(root: &Path, side: Side, purpose: Purpose, visitor: &V) -> Result<DirectoryPlan, Error> {
    let observed = facts(root, purpose == Purpose::Audit || side == Side::Original)?;
    let directory = observed.metadata.is_dir();
    let mut names = Names {
        dir: PathBuf::new(),
        root: true,
        entries: vec![Name {
            name: root.file_name().unwrap_or(root.as_os_str()).to_os_string(),
            relative: PathBuf::new(),
            presence: if side == Side::Original { Presence::SourceOnly } else { Presence::TargetOnly },
            admission: Admission::Observe,
        }],
    };
    if V::ENABLED {
        visitor.early(&mut names).map_err(|source| refusal(root, source))?;
    }
    let excluded = u64::from(names.entries[0].admission == Admission::PreserveTree);
    let (source, target) = if side == Side::Original {
        (Some(observed), None)
    } else {
        (None, Some(observed))
    };
    let mut entry = operation(names.entries.remove(0).name, PathBuf::new(), source, target)?;
    if directory && side == Side::Target {
        entry.action = Action::Directory {
            container: Container::Keep,
            descend: true,
        };
        entry.baseline = entry.action;
    }
    if excluded != 0 {
        entry.preserve_tree();
    }
    let plan = finish(
        Operations {
            source_root: root.to_path_buf(),
            target_root: root.to_path_buf(),
            dir: PathBuf::new(),
            root: true,
            purpose,
            entries: vec![entry],
        },
        excluded,
        visitor,
    )?;
    if directory
        && side == Side::Target
        && !matches!(
            plan.operations.entries[0].action,
            Action::Directory {
                container: Container::Keep,
                ..
            } | Action::PreserveTree
        )
    {
        return Err(refusal(root, "selected target root is a retained structural container".into()));
    }
    Ok(plan)
}
