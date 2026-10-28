//! Directory-local, compiled steering over immutable observations.

use std::ffi::{OsStr, OsString};
use std::fs::Metadata;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use diff_dirs_tree_observe as observe;

use crate::{Check, CheckError, DirDiff, EarlyFacts, Entry, Error, NoCheck, NoScope, Purpose, Scope, Side, Visit};

#[path = "shared.rs"]
pub(crate) mod shared;

/// Presence from immediate directory listings, before entry metadata or readlink.
/// Suppression never changes this fact and therefore cannot manufacture absence.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Presence {
    /// The name appeared only in the source listing.
    SourceOnly,
    /// The name appeared in both listings, independently of its types.
    Both,
    /// The name appeared only in the target listing.
    TargetOnly,
}

/// Pair-wide early admission. Preserving a name preserves its entire subtree.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Admission {
    /// Collect the normal observations and propose baseline work.
    Observe,
    /// Omit both sides and all descendants without child metadata/readlink.
    PreserveTree,
}

/// One immutable union name and its mutable admission. Slots cannot be inserted,
/// renamed, reordered, or have their side presence edited by a visitor.
#[derive(Debug)]
pub struct Name {
    name: OsString,
    relative: PathBuf,
    presence: Presence,
    source: Option<EarlyFacts>,
    target: Option<EarlyFacts>,
    admission: Admission,
}

impl Name {
    /// Lossless native basename. A root singleton uses the source root basename
    /// (or the sole bound tree's basename); `is_root` disambiguates that slot.
    pub fn name(&self) -> &OsStr {
        &self.name
    }
    /// Logical final root-relative coordinate; empty for a structural root slot.
    pub fn relative_path(&self) -> &Path {
        &self.relative
    }
    /// Cheap listing fact, never changed by preservation.
    pub fn presence(&self) -> Presence {
        self.presence
    }
    /// Present source's already-available facts; None means genuine absence.
    /// Some with Unknown kind is present, not absent or implicitly excluded.
    pub fn source(&self) -> Option<&EarlyFacts> {
        self.source.as_ref()
    }
    /// Present target's already-available facts, independently of source kind.
    pub fn target(&self) -> Option<&EarlyFacts> {
        self.target.as_ref()
    }
    /// Current early steering; initially Observe for every union name.
    pub fn admission(&self) -> Admission {
        self.admission
    }
    /// Suppress both observations and all descendant admission, not just copying.
    pub fn preserve_tree(&mut self) {
        self.admission = Admission::PreserveTree;
    }
    /// Refuse this actual union slot without accepting an injected blame path.
    /// Return this error from early: `return Err(name.refuse("protected name"))`.
    /// The library attaches this name to the logical target batch directory.
    /// No metadata/readlink is needed and the whole batch remains unpublished.
    pub fn refuse(&self, cause: impl Into<CheckError>) -> CheckError {
        SlotRefusal::at(&self.relative, &self.name, cause.into())
    }
}

/// Sorted union of immediate names. The callback runs after `read_dir`, but
/// before stat/readlink of **any child** in this batch. Listing the containing
/// directory and mandatory structural root/ancestor validation are not skipped.
/// Already-returned listing hints are available per side without extra child
/// stat/readlink/open. No mtime, mount or repository probes fill missing hints.
/// A preserved child is not statted, readlinked, opened, or entered.
///
/// ```
/// use diff_dirs_tree_inspect::{CheckError, FileKind, Names, Visitor};
/// struct LeavesOnly;
/// impl Visitor for LeavesOnly {
///     fn early(&self, names: &mut Names) -> Result<(), CheckError> {
///         if names.is_root() { return Ok(()); } // keep the structural root
///         for name in names.entries_mut() {
///             let skip = [name.source(), name.target()].into_iter().flatten()
///                 .any(|facts| matches!(facts.kind(), FileKind::Directory | FileKind::Symlink));
///             if name.name() == ".jj" || skip { name.preserve_tree(); }
///         }
///         Ok(())
///     }
/// }
/// ```
/// Unknown is deliberately admitted here; this is not a portable final type
/// policy without a corresponding late decision over completed metadata.
#[derive(Debug)]
pub struct Names {
    dir: PathBuf,
    root: bool,
    entries: Vec<Name>,
}

impl Names {
    /// Logical containing directory, empty at the root. No staging coordinate.
    pub fn dir(&self) -> &Path {
        &self.dir
    }
    /// A singleton for the actual bound root, not an ordinary directory listing.
    /// Root structural metadata is mandatory even when this slot is preserved.
    pub fn is_root(&self) -> bool {
        self.root
    }
    /// Native-byte sorted union slots with immutable identities/presence.
    pub fn entries(&self) -> &[Name] {
        &self.entries
    }
    /// Steer existing slots only. Reordering opaque entries with `mem::swap` is
    /// rejected before child observations, rather than changing error order.
    pub fn entries_mut(&mut self) -> impl Iterator<Item = &mut Name> {
        self.entries.iter_mut()
    }
}

/// Local container work, independent of descending into its children.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Container {
    /// Keep the existing directory inode, independently of child work.
    Keep,
    /// Create an absent directory needed as a legal routing ancestor.
    Create,
    /// Source-only routing directory, with descent required. Inspect through the
    /// absent target and create this container only if admitted descendant work
    /// needs it. An explicitly selected empty directory uses eager `Create`.
    /// Pending ancestors are runtime effect state, not new paired traversal.
    CreateIfNeeded,
    /// Remove after child reconciliation only if no descendant survives.
    Remove,
}

/// Final work for one named entry. This is not its observed difference.
///
/// | Observed shape | Permitted work (in addition to PreserveTree) |
/// |---|---|
/// | source-only leaf | KeepLeaf, InstallLeaf |
/// | target-only leaf | KeepLeaf, RemoveLeaf |
/// | two leaves | KeepLeaf; InstallLeaf only when different |
/// | source-only directory | Create, optionally descend; CreateIfNeeded with descent |
/// | target-only directory | Keep/Remove container and descend; Keep may stop |
/// | two directories | Keep container, descend or stop |
/// | source leaf / target directory | ReplaceWithLeaf, or keep target directory |
/// | source directory / target leaf | ReplaceWithDirectory, or KeepLeaf |
/// | unsupported kind on either side | no comparison/proposal; explicit PreserveTree only |
///
/// Equal leaves cannot be force-installed. Replacements are **coupled**: no
/// delete-half/create-half authority exists. Removing a directory requires
/// descent; surviving descendants retain necessary ancestors. A source-only
/// directory cannot Keep an absent container while admitting child writes.
/// PreserveTree always stops descent and performs no local work. Keeping a
/// directory container while descending permits selected pruning below it.
/// Unsupported special types have no comparison or baseline mutation proposal.
/// Late may explicitly preserve the entire pair after inspecting its metadata;
/// any other action, or an unchanged unavailable proposal, is refused. Early
/// preservation additionally avoids that metadata cost. Unsupported structural
/// roots remain errors rather than bypassing the caller's root binding contract.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    /// Retain this name/subtree, with neither namespace work nor descent.
    PreserveTree,
    /// No leaf installation/removal. At an absent target, leave it absent.
    KeepLeaf,
    /// Install a different or absent source leaf using target-side preparation.
    InstallLeaf,
    /// Delete a source-absent target leaf, never a half of a collision.
    RemoveLeaf,
    /// Independent container authority and child admission, subject to the table
    /// above. Removing with descent stopped, or keeping an absent container, is
    /// invalid; the library never silently creates a refused routing ancestor.
    /// `CreateIfNeeded` requires a source-only directory and `descend: true`;
    /// it does not authorize replacement of an existing leaf or directory.
    Directory { container: Container, descend: bool },
    /// Coupled directory-to-leaf replacement. Sync locally checks removability
    /// of this affected target subtree before staging/chmod/dismantling it.
    /// A retained descendant refuses intact; later I/O failure can be partial.
    ReplaceWithLeaf,
    /// Coupled leaf-to-directory replacement. Sync prepares admitted source
    /// contents before removing the old leaf, using final logical visitor paths.
    /// `descend: false` deliberately prepares an empty directory.
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

// Ordinary pair inspection keeps its existing inline ownership. Only the
// source-once epoch shares facts; either representation lends the same full Facts.
#[derive(Debug)]
enum SourceFacts {
    Owned(Facts),
    Shared(Arc<Facts>),
}

impl std::ops::Deref for SourceFacts {
    type Target = Facts;
    fn deref(&self) -> &Facts {
        match self {
            Self::Owned(facts) => facts,
            Self::Shared(facts) => facts,
        }
    }
}

impl From<Facts> for SourceFacts {
    fn from(facts: Facts) -> Self {
        Self::Owned(facts)
    }
}

impl Facts {
    fn supported(&self) -> bool {
        self.metadata.is_dir() || self.metadata.is_file() || self.metadata.is_symlink()
    }
    /// Actual no-follow metadata, borrowed without an additional stat.
    pub fn metadata(&self) -> &Metadata {
        &self.metadata
    }
    /// Literal readlink spelling where already needed. Compare native `OsStr`
    /// spellings, not Path equality (which normalizes repeated separators).
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
    source: Option<SourceFacts>,
    target: Option<Facts>,
    different: Option<bool>,
    action: Option<Action>,
    baseline: Option<Action>,
}

impl Operation {
    /// Immutable native name from the union (or structural root singleton).
    pub fn name(&self) -> &OsStr {
        &self.name
    }
    /// Final root-relative coordinate; never a private physical staging path.
    pub fn relative_path(&self) -> &Path {
        &self.relative
    }
    /// Actual admitted source facts; None is genuine source absence, not filtering.
    pub fn source(&self) -> Option<&Facts> {
        self.source.as_deref()
    }
    /// Actual admitted target facts; source-only preparation has no target facts.
    pub fn target(&self) -> Option<&Facts> {
        self.target.as_ref()
    }
    /// Immutable listing/observation presence. Type collisions remain Both.
    pub fn presence(&self) -> Presence {
        presence(self.source.is_some(), self.target.is_some())
    }
    /// True only for a known supported stat-key/literal-link difference. False
    /// alone does not prove equality: unsupported kinds have no comparison.
    /// Use [`Self::comparison`] when that distinction matters.
    pub fn is_different(&self) -> bool {
        self.different == Some(true)
    }
    /// Immediate supported comparison, independent of steering/progress. None
    /// means an unsupported kind was observed on at least one side: no normal
    /// file comparison is claimed. Matched directories are Some(false), but their
    /// children may differ. One-sided supported entries are Some(true).
    pub fn comparison(&self) -> Option<bool> {
        self.different
    }
    /// Current proposal, changed only through typed steering methods. Initially
    /// None for unsupported kinds; late must explicitly select PreserveTree or
    /// the batch refuses. Every successfully finalized entry has Some(action).
    pub fn action(&self) -> Option<Action> {
        self.action
    }
    /// Original mirror proposal, never rewritten by steering. None means the
    /// observed kind has no supported baseline work, not a fictitious leaf op.
    pub fn baseline(&self) -> Option<Action> {
        self.baseline
    }
    /// Change work only. The complete batch is validated after late returns,
    /// before application or child publication, even if an invalid action is
    /// later never executed. No partial entry operation is published on error.
    pub fn set_action(&mut self, action: Action) {
        self.action = Some(action);
    }
    /// No local writes or descendant work, including at type collisions.
    pub fn preserve_tree(&mut self) {
        self.action = Some(Action::PreserveTree);
    }
    /// Refuse this observed slot at its logical final target coordinate. Only
    /// the cause is caller-supplied; names/paths remain those actually observed.
    /// Return the result from late to refuse before this batch changes anything.
    pub fn refuse(&self, cause: impl Into<CheckError>) -> CheckError {
        SlotRefusal::at(&self.relative, &self.name, cause.into())
    }
    /// Keep an existing target container but allow descendant work. For an
    /// absent target container this stops the subtree instead of authorizing
    /// impossible child creation. Existing leaves are simply retained. If either
    /// side is unsupported, preserve the whole pair without descendant work.
    pub fn retain(&mut self) {
        self.action = Some(if self.different.is_none() {
            Action::PreserveTree
        } else if self.target.as_ref().is_some_and(|f| f.metadata.is_dir()) {
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
    /// Whether this entry's directory children may be inspected. Replacement
    /// preparation and target removal are handled by the mutation consumer.
    pub fn descends(&self) -> bool {
        matches!(
            self.action,
            Some(Action::Directory { descend: true, .. } | Action::ReplaceWithDirectory { descend: true })
        )
    }
    /// Whether a namespace action is proposed for this name. Directory removal
    /// and conditional creation depend on descendants; this is not a claim of completed work.
    pub fn has_work(&self) -> bool {
        matches!(
            self.action,
            Some(
                Action::InstallLeaf
                    | Action::RemoveLeaf
                    | Action::ReplaceWithLeaf
                    | Action::ReplaceWithDirectory { .. }
                    | Action::Directory {
                        container: Container::Create | Container::CreateIfNeeded | Container::Remove,
                        ..
                    }
            )
        )
    }
    fn validate(&self) -> Result<(), CheckError> {
        let Some(action) = self.action else {
            return Err("entry has no supported operation proposal".into());
        };
        let s = self.source.as_ref().map(|f| &f.metadata);
        let t = self.target.as_ref().map(|f| &f.metadata);
        let sd = s.is_some_and(Metadata::is_dir);
        let td = t.is_some_and(Metadata::is_dir);
        let leaf = |m: &Metadata| m.is_file() || m.is_symlink();
        let valid = match action {
            Action::PreserveTree => true,
            Action::KeepLeaf => t.is_some_and(leaf) || (t.is_none() && s.is_some_and(leaf)),
            Action::InstallLeaf => s.is_some_and(leaf) && !td && self.is_different(),
            Action::RemoveLeaf => s.is_none() && t.is_some_and(leaf),
            Action::ReplaceWithLeaf => s.is_some_and(leaf) && td,
            Action::ReplaceWithDirectory { .. } => sd && t.is_some_and(leaf),
            Action::Directory {
                container: Container::Create, ..
            } => sd && t.is_none(),
            Action::Directory {
                container: Container::CreateIfNeeded,
                descend,
            } => sd && t.is_none() && descend,
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
    /// Final root-relative containing directory; empty for the root pair.
    pub fn dir(&self) -> &Path {
        &self.dir
    }
    /// Whether these are mandatory structural root facts, not child entries.
    pub fn is_root(&self) -> bool {
        self.root
    }
    /// Comparison, source-only preparation, target-only pruning, or explicit audit.
    pub fn purpose(&self) -> Purpose {
        self.purpose
    }
    /// Logical source binding, including while target output is privately staged.
    pub fn source_root(&self) -> &Path {
        &self.source_root
    }
    /// Logical final target binding; never the private output directory.
    pub fn target_root(&self) -> &Path {
        &self.target_root
    }
    /// All admitted immediate observations/proposals, including unchanged leaves
    /// and matched directory descent. Excluded early names have no facts here.
    pub fn entries(&self) -> &[Operation] {
        &self.entries
    }
    /// Edit only actions of existing entries. Names, metadata, presence and
    /// baseline comparison cannot be fabricated or edited; swapping slots is
    /// rejected at finalization. Invalid combinations refuse this entire batch.
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
    planned: u64,
    retained: u64,
}

impl DirectoryPlan {
    /// Finalized immutable observations and actions; no mutable projection exists.
    pub fn operations(&self) -> &Operations {
        &self.operations
    }
    /// Baseline supported delta, not a filtered application report. A special
    /// retained late has no normal file comparison and is omitted from these
    /// categories; its retained slot still prevents a full-root clean claim.
    pub fn diff(&self) -> &DirDiff {
        &self.diff
    }
    /// Count of premetadata-excluded immediate boundaries, not hidden entries.
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
    /// Sorted immediate basenames excluded before child observation. Late
    /// preservation remains visible as an operation's PreserveTree action.
    /// Structural roots use their singleton operation and excluded count instead
    /// of a fictitious parent-listing name. These names have no observations in
    /// this plan; shared peers may still have demanded their source facts.
    pub fn excluded_names(&self) -> &[OsString] {
        &self.preserved
    }
    /// Declared scoped coverage, not proof that some particular action changed.
    pub fn steered(&self) -> bool {
        self.steered
    }
    /// Immediate slots whose action differs from the mirror baseline, including
    /// stopping otherwise-clean descent. Not unique-path or recursive counts.
    pub fn retained(&self) -> u64 {
        self.retained
    }
    /// Immediate planned namespace actions, not recursive or completed work.
    pub fn work(&self) -> u64 {
        self.planned
    }
    /// Finalized admitted paired-directory basenames in native-byte order.
    /// One-sided directories and type collisions remain opaque to comparison;
    /// other drivers can inspect their operations under their own recursion rules.
    /// Structural-root singletons contain no child listing and yield no names.
    /// This projection does not publish mutable actions or plan constructors.
    pub fn paired_children(&self) -> impl Iterator<Item = &OsStr> {
        self.operations
            .entries
            .iter()
            .filter(|e| {
                !self.operations.root
                    && e.descends()
                    && e.source.as_ref().is_some_and(|f| f.metadata.is_dir())
                    && e.target.as_ref().is_some_and(|f| f.metadata.is_dir())
            })
            .map(|e| e.name.as_os_str())
    }

    /// Lower this finalized batch to read-only comparison, coverage and owned
    /// paired-child names. This plan releases its full-fact ownership, including
    /// divergent entries; shared peers/root bindings can retain their own facts.
    /// The result grants no filesystem mutation authority.
    pub fn into_comparison(self) -> shared::DirectoryComparison {
        let children = self.paired_children().map(OsStr::to_os_string).collect();
        shared::DirectoryComparison {
            diff: self.diff,
            coverage: shared::DirectoryCoverage {
                excluded: self.excluded,
                scoped: self.steered,
                planned: self.planned,
                retained: self.retained,
            },
            children,
            root: self.operations.root,
        }
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
/// use diff_dirs_tree_inspect::{Visitor, Names, Operations, Presence, CheckError};
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
    /// The paired-root callback-only observation/context is also skipped in the
    /// disabled walk; mandatory root binding/type validation still runs. This
    /// does not eliminate the ordinary per-directory observation/action batch.
    /// Leave true for any steering/refusal. No measured speedup is claimed.
    const ENABLED: bool = true;
    /// Whether mere exhaustion must not certify unqualified full-root coverage.
    /// Refusal-only compatibility checks override this; steering visitors keep
    /// the conservative default even if no excluded name happened to occur.
    /// Returning false promises that this visitor never suppresses coverage or
    /// changes work. Actual exclusion/retention still invalidates completeness.
    fn scoped(&self) -> bool {
        Self::ENABLED
    }
    /// Steer the sorted union using already-available per-side facts, before any
    /// additional child stat/readlink. Return an error to
    /// refuse the batch without publishing its children. The directory listing
    /// and structural root metadata exceptions are specified on [`Names`].
    fn early(&self, _: &mut Names) -> Result<(), CheckError> {
        Ok(())
    }
    /// Steer actual proposals using already-needed immutable facts. This runs
    /// outside frontier locks, before even a clean result exposes child jobs.
    /// A normal return is followed by whole-immediate-batch action validation.
    fn late(&self, _: &mut Operations) -> Result<(), CheckError> {
        Ok(())
    }
    /// Lend the same frozen visitor without losing static no-op specialization.
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

/// Borrowed compiled visitor, forwarding both callbacks and ENABLED's promise.
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
    /// Side-local refusal projection, run on immutable late facts.
    pub check: C,
    /// Pair-wide early preservation projection.
    pub scope: S,
    roots_checked: bool,
}
impl<C, S> CheckedScope<C, S> {
    /// Compose the legacy interfaces as one compiled visitor, with no separate
    /// policy engine or filesystem observations solely for these projections.
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
                            .map_err(|source| Box::new(CheckedRefusal(Error::Refused { path, source })) as CheckError)?;
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
    let metadata = observe::symlink_metadata(path)?;
    facts_with_metadata(path, metadata, read_link)
}

fn listed_facts(entry: observe::Entry, read_link: bool) -> Result<Facts, Error> {
    let path = entry.path();
    // Observation owns deferred no-follow acquisition and its backend resources.
    // Inspection consumes only admitted handles; none reach late or a plan.
    let metadata = entry.metadata()?;
    facts_with_metadata(&path, metadata, read_link)
}

struct ListedPair {
    source: Option<observe::Entry>,
    target: Option<observe::Entry>,
}

fn facts_with_metadata(path: &Path, metadata: Metadata, read_link: bool) -> Result<Facts, Error> {
    facts_with_metadata_using(path, metadata, read_link, |path| observe::read_link(path).map_err(Error::from))
}

fn facts_with_metadata_using(
    path: &Path,
    metadata: Metadata,
    read_link: bool,
    acquire_link: impl FnOnce(&Path) -> Result<PathBuf, Error>,
) -> Result<Facts, Error> {
    let link = if metadata.is_symlink() && read_link {
        Some(acquire_link(path)?)
    } else {
        None
    };
    Ok(Facts { metadata, link })
}

fn operation(
    name: OsString,
    relative: PathBuf,
    source: Option<SourceFacts>,
    target: Option<Facts>,
    source_root: &Path,
    target_root: &Path,
) -> Result<Operation, Error> {
    let sd = source.as_ref().is_some_and(|f| f.metadata.is_dir());
    let td = target.as_ref().is_some_and(|f| f.metadata.is_dir());
    let comparable = source.as_deref().into_iter().chain(target.as_ref()).all(Facts::supported);
    let different = if comparable {
        Some(match (&source, &target) {
            (Some(_), Some(_)) if sd && td => false,
            (Some(s), Some(t)) if s.metadata.is_file() && t.metadata.is_file() => {
                s.metadata.len() != t.metadata.len()
                    || s.metadata
                        .modified()
                        .map_err(|e| Error::io(source_root.join(&relative), "read modification time", e))?
                        != t.metadata
                            .modified()
                            .map_err(|e| Error::io(target_root.join(&relative), "read modification time", e))?
            }
            (Some(s), Some(t)) if s.metadata.is_symlink() && t.metadata.is_symlink() => {
                s.link.as_ref().map(|p| p.as_os_str()) != t.link.as_ref().map(|p| p.as_os_str())
            }
            _ => true,
        })
    } else {
        None
    };
    let action = comparable.then(|| match (source.is_some(), target.is_some(), sd, td) {
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
        (true, _, _, _) if different == Some(true) => Action::InstallLeaf,
        (false, true, _, _) => Action::RemoveLeaf,
        _ => Action::KeepLeaf,
    });
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
    let mut planned = 0;
    let mut retained = 0;
    for entry in &operations.entries {
        let path = operations.target_root.join(&entry.relative);
        for (root, facts) in [(&operations.source_root, entry.source()), (&operations.target_root, entry.target())] {
            if let Some(facts) = facts
                && !facts.supported()
                && (operations.root || entry.action != Some(Action::PreserveTree))
            {
                return Err(Error::UnsupportedEntry {
                    path: root.join(&entry.relative),
                });
            }
        }
        entry.validate().map_err(|source| refusal(&path, source))?;
        planned += u64::from(entry.has_work());
        retained += u64::from(entry.action != entry.baseline);
        if entry.is_different() {
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
        planned,
        retained,
    })
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
            child: (!relative.as_os_str().is_empty()).then(|| name.to_os_string()),
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

fn refusal(path: &Path, source: CheckError) -> Error {
    // Only private wrappers may select a narrower observed coordinate. A public
    // Error supplied as a visitor's cause must not inject a different blame path.
    let source = match source.downcast::<CheckedRefusal>() {
        Ok(error) => return error.0,
        Err(source) => source,
    };
    match source.downcast::<SlotRefusal>() {
        Ok(named) => Error::Refused {
            path: named.child.map_or_else(|| path.to_path_buf(), |name| path.join(name)),
            source: named.cause,
        },
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
/// Listed handles remain private through early admission, then supply
/// admitted no-follow metadata and are dropped before late/publication. Excluded
/// handles are dropped without metadata/readlink; none become plan/catalog data.
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
            for entry in observe::read_directory(path)? {
                raw.push((entry.file_name(), side, entry));
            }
        }
    }
    inspect_listed(source_root, target_root, dir, purpose, visitor, execution, raw)
}

#[allow(clippy::too_many_arguments)]
fn inspect_listed<V: Visitor>(
    source_root: &Path,
    target_root: &Path,
    dir: &Path,
    purpose: Purpose,
    visitor: &V,
    execution: crate::Execution,
    mut raw: Vec<(OsString, Side, observe::Entry)>,
) -> Result<DirectoryPlan, Error> {
    raw.sort_unstable_by(|a, b| a.0.as_encoded_bytes().cmp(b.0.as_encoded_bytes()).then((a.1 as u8).cmp(&(b.1 as u8))));
    let mut names = Names {
        dir: dir.to_path_buf(),
        root: false,
        entries: Vec::with_capacity(raw.len()),
    };
    let mut raw = raw.into_iter().peekable();
    let mut listed = Vec::with_capacity(raw.len());
    while let Some((name, side, entry)) = raw.next() {
        let mut pair = match side {
            Side::Original => ListedPair {
                source: Some(entry),
                target: None,
            },
            Side::Target => ListedPair {
                source: None,
                target: Some(entry),
            },
        };
        let both = raw.peek().is_some_and(|(next, _, _)| *next == name);
        if both {
            let (_, other, entry) = raw.next().expect("peeked listed entry");
            match (side, other) {
                (Side::Original, Side::Target) => pair.target = Some(entry),
                (Side::Target, Side::Original) => pair.source = Some(entry),
                _ => return Err(refusal(&target_root.join(dir).join(name), "duplicate name on one listing side".into())),
            }
        }
        names.entries.push(Name {
            relative: dir.join(&name),
            name,
            presence: presence(pair.source.is_some(), pair.target.is_some()),
            source: pair.source.as_ref().map(observe::Entry::hints),
            target: pair.target.as_ref().map(observe::Entry::hints),
            admission: Admission::Observe,
        });
        listed.push(pair);
    }
    let Admitted { entries: admitted, coverage } = admit_names(names, listed, target_root, visitor)?;
    let mut operations = Operations {
        source_root: source_root.to_path_buf(),
        target_root: target_root.to_path_buf(),
        dir: dir.to_path_buf(),
        root: false,
        purpose,
        entries: Vec::with_capacity(admitted.len()),
    };
    let observe = |(name, listed): (Name, ListedPair)| {
        #[cfg(all(test, feature = "parallel"))]
        crate::roots::tests::observe_worker();
        // Original before target, and indexed union order across parallel jobs.
        let s = listed.source.map(|entry| listed_facts(entry, true).map(SourceFacts::from)).transpose()?;
        let t = listed
            .target
            .map(|entry| listed_facts(entry, name.presence == Presence::Both || purpose == Purpose::Audit))
            .transpose()?;
        operation(name.name, name.relative, s, t, source_root, target_root)
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
    finish_admitted(operations, coverage, visitor)
}

struct AdmissionCoverage {
    excluded: u64,
    first_excluded: Option<PathBuf>,
    preserved: Vec<OsString>,
}

struct Admitted<T> {
    entries: Vec<(Name, T)>,
    coverage: AdmissionCoverage,
}

// Shared by independent pairs and source-once lanes. Physical presence and slot
// identity remain fixed; only admission is projected into the acquisition stage.
fn admit_names<T, V: Visitor>(mut names: Names, listed: Vec<T>, target_root: &Path, visitor: &V) -> Result<Admitted<T>, Error> {
    if V::ENABLED {
        visitor.early(&mut names).map_err(|source| refusal(&target_root.join(&names.dir), source))?;
    }
    if names
        .entries
        .windows(2)
        .any(|pair| pair[0].name.as_encoded_bytes() >= pair[1].name.as_encoded_bytes())
    {
        return Err(refusal(&target_root.join(&names.dir), "visitor reordered immutable union names".into()));
    }
    let mut coverage = AdmissionCoverage {
        excluded: 0,
        first_excluded: None,
        preserved: Vec::new(),
    };
    let entries = names
        .entries
        .into_iter()
        .zip(listed)
        .filter_map(|(name, listed)| {
            if name.admission == Admission::PreserveTree {
                coverage.excluded += 1;
                if coverage.first_excluded.is_none() {
                    coverage.first_excluded = Some(name.relative);
                }
                coverage.preserved.push(name.name);
                None
            } else {
                Some((name, listed))
            }
        })
        .collect();
    Ok(Admitted { entries, coverage })
}

fn finish_admitted<V: Visitor>(operations: Operations, coverage: AdmissionCoverage, visitor: &V) -> Result<DirectoryPlan, Error> {
    let mut plan = finish(operations, coverage.excluded, visitor)?;
    plan.first_excluded = coverage.first_excluded;
    plan.preserved = coverage.preserved;
    Ok(plan)
}

/// Finalize a singleton actual root operation. Mandatory structural root facts
/// precede early steering; preserving the root prevents listing its contents.
/// This also represents absent-target initialization and explicitly bound root
/// leaf replacement, without a fictitious parent listing or fabricated metadata.
pub fn inspect_root<V: Visitor>(source: &Path, target: &Path, purpose: Purpose, visitor: &V) -> Result<DirectoryPlan, Error> {
    let s = facts(source, true)?;
    let t = match observe::symlink_metadata(target) {
        Ok(metadata) => Some(facts_with_metadata(target, metadata, s.metadata.is_symlink() || purpose == Purpose::Audit)?),
        Err(e) if e.io_error().kind() == std::io::ErrorKind::NotFound => None,
        Err(e) => return Err(Error::root_observation(e)),
    };
    finish_root(source, target, s.into(), t, purpose, visitor)
}

pub(crate) fn inspect_bound_root<V: Visitor>(source: &Path, target: &Path, s: &Metadata, t: &Metadata, visitor: &V) -> Result<DirectoryPlan, Error> {
    // These actual real-directory facts were already needed at construction.
    // Reuse them under the stable-root contract rather than restatting for hooks.
    finish_root(
        source,
        target,
        Facts {
            metadata: s.clone(),
            link: None,
        }
        .into(),
        Some(Facts {
            metadata: t.clone(),
            link: None,
        }),
        Purpose::Compare,
        visitor,
    )
}

fn finish_root<V: Visitor>(source: &Path, target: &Path, s: SourceFacts, t: Option<Facts>, purpose: Purpose, visitor: &V) -> Result<DirectoryPlan, Error> {
    let mut names = Names {
        dir: PathBuf::new(),
        root: true,
        entries: vec![Name {
            name: source.file_name().unwrap_or(source.as_os_str()).to_os_string(),
            relative: PathBuf::new(),
            presence: presence(true, t.is_some()),
            source: Some(EarlyFacts::from_root_metadata(&s.metadata)),
            target: t.as_ref().map(|facts| EarlyFacts::from_root_metadata(&facts.metadata)),
            admission: Admission::Observe,
        }],
    };
    if V::ENABLED {
        visitor.early(&mut names).map_err(|source| refusal(target, source))?;
    }
    let excluded = u64::from(names.entries[0].admission == Admission::PreserveTree);
    let mut entry = operation(names.entries.remove(0).name, PathBuf::new(), Some(s), t, source, target)?;
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
            source: (side == Side::Original).then(|| EarlyFacts::from_root_metadata(&observed.metadata)),
            target: (side == Side::Target).then(|| EarlyFacts::from_root_metadata(&observed.metadata)),
            admission: Admission::Observe,
        }],
    };
    if V::ENABLED {
        visitor.early(&mut names).map_err(|source| refusal(root, source))?;
    }
    let excluded = u64::from(names.entries[0].admission == Admission::PreserveTree);
    let (source, target) = if side == Side::Original {
        (Some(observed.into()), None)
    } else {
        (None, Some(observed))
    };
    let mut entry = operation(names.entries.remove(0).name, PathBuf::new(), source, target, root, root)?;
    if directory && side == Side::Target {
        entry.action = Some(Action::Directory {
            container: Container::Keep,
            descend: true,
        });
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
            Some(
                Action::Directory {
                    container: Container::Keep,
                    ..
                } | Action::PreserveTree
            )
        )
    {
        return Err(refusal(root, "selected target root is a retained structural container".into()));
    }
    Ok(plan)
}

#[cfg(all(test, target_os = "linux"))]
mod listing_tests {
    use super::*;
    use crate::{FactOrigin, FileKind};
    use std::fs;

    #[test]
    fn early_preservation_skips_unavailable_children_and_admitted_hints_complete() {
        struct Filter;
        impl Visitor for Filter {
            fn early(&self, batch: &mut Names) -> Result<(), CheckError> {
                for name in batch.entries_mut() {
                    assert_eq!(name.source().unwrap().origin(), FactOrigin::Listing);
                    assert!(name.source().unwrap().inode().is_some());
                    let kind = if name.name() == ".jj" || name.name() == "dir" {
                        FileKind::Directory
                    } else {
                        FileKind::Symlink
                    };
                    let expected = if cfg!(feature = "native-dirent") { kind } else { FileKind::Unknown };
                    assert_eq!(name.source().unwrap().kind(), expected);
                    if name.name() == "admitted-link" {
                        assert!(name.target().is_none()); // not present-unknown
                    } else {
                        if name.name() == ".jj" {
                            assert_eq!(name.target().unwrap().kind(), expected);
                        }
                        name.preserve_tree();
                    }
                }
                Ok(())
            }
            fn late(&self, batch: &mut Operations) -> Result<(), CheckError> {
                assert_eq!(batch.entries().len(), 1);
                let facts = batch.entries()[0].source().unwrap();
                assert!(facts.metadata().is_symlink());
                assert_eq!(facts.link_target().unwrap().as_os_str(), "missing//./literal");
                Ok(())
            }
        }
        let f = crate::test_support::Fixture::new();
        f.both_dir(".jj/never");
        f.both_file(".jj/never/child", b"not entered");
        fs::create_dir(f.original.join("dir")).unwrap();
        std::os::unix::fs::symlink("never-follow", f.original.join("link")).unwrap();
        std::os::unix::fs::symlink("missing//./literal", f.original.join("admitted-link")).unwrap();
        let mut raw = Vec::new();
        for (side, root) in [(Side::Original, &f.original), (Side::Target, &f.target)] {
            for entry in observe::read_directory(root).unwrap() {
                let name = entry.file_name();
                raw.push((name, side, entry));
            }
        }
        // Change only this owned fixture, BEFORE invoking a visitor. Any metadata
        // or readlink of a preserved entry would now fail. Default Unix exercises
        // unknown hints; native enumeration exercises the corresponding known
        // hints, without injecting counters or mutable facts into observation.
        fs::remove_dir_all(f.original.join(".jj")).unwrap();
        fs::remove_dir_all(f.target.join(".jj")).unwrap();
        fs::remove_dir(f.original.join("dir")).unwrap();
        fs::remove_file(f.original.join("link")).unwrap();
        let plan = inspect_listed(
            &f.original,
            &f.target,
            Path::new(""),
            Purpose::Compare,
            &Filter,
            crate::Execution::Sequential,
            raw,
        )
        .unwrap();
        assert_eq!(plan.excluded(), 3);
        assert_eq!(plan.diff().files.added, ["admitted-link"]);
    }

    #[test]
    fn root_early_reuses_mandatory_binding_facts_without_a_second_lookup() {
        struct RootFacts;
        impl Visitor for RootFacts {
            fn early(&self, names: &mut Names) -> Result<(), CheckError> {
                assert!(names.is_root());
                let name = &mut names.entries_mut().next().unwrap();
                for facts in [name.source().unwrap(), name.target().unwrap()] {
                    assert_eq!(facts.origin(), FactOrigin::RootMetadata);
                    assert_eq!(facts.kind(), FileKind::Directory);
                    assert!(facts.inode().is_some());
                }
                name.preserve_tree();
                Ok(())
            }
        }
        let f = crate::test_support::Fixture::new();
        let source = fs::symlink_metadata(&f.original).unwrap();
        let target = fs::symlink_metadata(&f.target).unwrap();
        // Owned fixture only, before inspection: if root finalization restats
        // either bound path, it fails. This does not promise traversal under writers.
        fs::rename(&f.original, f.original.with_file_name("moved-source")).unwrap();
        fs::rename(&f.target, f.target.with_file_name("moved-target")).unwrap();
        let plan = inspect_bound_root(&f.original, &f.target, &source, &target, &RootFacts).unwrap();
        assert_eq!(plan.excluded(), 1);
        assert!(!plan.operations().entries()[0].descends());
    }

    #[cfg(feature = "native-dirent")]
    #[test]
    fn real_cached_type_filter_performs_only_the_containing_directory_listing() {
        struct Filter;
        impl Visitor for Filter {
            fn early(&self, names: &mut Names) -> Result<(), CheckError> {
                for name in names.entries_mut() {
                    let facts = name.source().unwrap();
                    assert_eq!(facts.origin(), FactOrigin::Listing);
                    assert!(facts.inode().is_some());
                    assert_eq!(facts.kind(), if name.name() == "dir" { FileKind::Directory } else { FileKind::Symlink });
                    assert!(name.target().is_none());
                    name.preserve_tree();
                }
                Ok(())
            }
            fn late(&self, operations: &mut Operations) -> Result<(), CheckError> {
                assert!(operations.entries().is_empty());
                Ok(())
            }
        }
        let scratch = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.test-agent/native-dirent");
        fs::create_dir_all(&scratch).unwrap();
        let f = crate::test_support::Fixture::new_in(&scratch);
        fs::create_dir(f.original.join("dir")).unwrap();
        std::os::unix::fs::symlink("missing", f.original.join("link")).unwrap();
        // Markers are outside callbacks. The narrowly owned syscall proof can
        // distinguish observation from fixture setup and cleanup without timings.
        eprintln!("NATIVE_DIRENT_BEGIN");
        let plan = inspect_directory(&f.original, &f.target, Path::new(""), Some(&f.original), None, Purpose::Copy, &Filter).unwrap();
        eprintln!("NATIVE_DIRENT_END");
        assert_eq!(plan.excluded(), 2);
    }
}
