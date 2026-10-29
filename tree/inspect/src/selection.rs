//! The sole metadata-free selected-work model and semantic legality table.
//!
//! Decoding a selection is not an executable binding. Only inspection constructs
//! live preparation; manifest references additionally need an executor-owned bind.

use std::ffi::{OsStr, OsString};
use std::fs::Metadata;
use std::ops::Deref;
use std::path::{Component, Path, PathBuf};

use diff_dirs_tree_observe::{FileKind, NameId, NameTable};

use crate::{Action, Comparison, Container, DirDiff, Presence, Purpose, SemanticId};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Directory,
    File,
    Symlink,
    Other,
}

impl Kind {
    pub fn from_metadata(metadata: &Metadata) -> Self {
        if metadata.is_dir() {
            Self::Directory
        } else if metadata.is_file() {
            Self::File
        } else if metadata.is_symlink() {
            Self::Symlink
        } else {
            Self::Other
        }
    }
    pub fn from_hint(kind: FileKind) -> Option<Self> {
        Some(match kind {
            FileKind::Unknown => return None,
            FileKind::Directory => Self::Directory,
            FileKind::RegularFile => Self::File,
            FileKind::Symlink => Self::Symlink,
            _ => Self::Other,
        })
    }
    pub fn is_leaf(self) -> bool {
        matches!(self, Self::File | Self::Symlink)
    }
}

/// Declared-data digest, not a content hash or native ownership certificate.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ManifestId {
    pub version: u32,
    pub platform: NativePlatform,
    pub digest: [u8; 32],
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NativePlatform {
    Unix,
    Windows,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct LeafId(pub u64);
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NodeId(pub u64);
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SlotId(pub usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SelectionDomain {
    PhysicalUnion,
    Listed { desired: ManifestId, owned: ManifestId },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourceObservation {
    Present(Kind),
    Absent,
    NotRequested,
}
impl SourceObservation {
    pub fn kind(self) -> Option<Kind> {
        if let Self::Present(kind) = self { Some(kind) } else { None }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Shape {
    pub source: SourceObservation,
    pub target: Option<Kind>,
}
impl Shape {
    /// Physical presence only. NotRequested must never project to source absence.
    pub fn presence(self) -> Option<Presence> {
        match (self.source, self.target) {
            (SourceObservation::Present(_), Some(_)) => Some(Presence::Both),
            (SourceObservation::Present(_), None) => Some(Presence::SourceOnly),
            (SourceObservation::Absent, Some(_)) => Some(Presence::TargetOnly),
            _ => None,
        }
    }
    pub fn supported(self) -> bool {
        self.source.kind() != Some(Kind::Other) && self.target != Some(Kind::Other)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InstallIntent {
    ReconcileDifference,
    Renew,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DestinationGrant {
    AbsentOnly,
    ReplaceLeaf,
    ReplaceTree,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InstallAuthority {
    pub intent: InstallIntent,
    pub destination: DestinationGrant,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RemovalAuthority {
    SourceAbsent,
    OwnedLeaf { owned: ManifestId, leaf: LeafId },
    CoupledOwned { owned: ManifestId, subtree: NodeId },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InteriorInput {
    PhysicalDirectory,
    Listed { desired: ManifestId, subtree: NodeId },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InteriorRole {
    Paired,
    SourceOnly,
    TargetOnly,
    PrepareReplacement,
    RemoveForReplacement,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Interior<C> {
    pub role: InteriorRole,
    pub context: C,
    pub input: InteriorInput,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LeafKind {
    RegularFile,
    Symlink,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LeafExpectation {
    SupportedLeaf,
    Exact { kind: LeafKind, executable: Option<bool> },
}
/// A desired obligation at this slot. A missing donor cannot become preservation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DesiredInput {
    Directory { node: NodeId },
    Leaf { leaf: LeafId, expected: LeafExpectation },
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ApplicationNeed {
    #[default]
    None,
    LocalOrOpaque,
}

/// Comparison coverage is independent of selected work and action deviation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DirectoryCoverage {
    pub excluded: u64,
    pub scoped: bool,
    pub planned: u64,
    pub deviated: u64,
    /// Selected preservation boundaries, not surviving filesystem entries.
    pub retained: u64,
    pub equal: u64,
    pub different: u64,
    pub not_assessed: u64,
    pub unsupported: u64,
    pub unresolved: u64,
    pub stopped_paired: u64,
    pub application_need: ApplicationNeed,
}
impl DirectoryCoverage {
    pub fn comparison_complete(self) -> bool {
        !self.scoped && self.excluded == 0 && self.stopped_paired == 0 && self.not_assessed == 0 && self.unsupported == 0 && self.unresolved == 0
    }
    pub fn is_clean(self) -> bool {
        self.comparison_complete() && self.different == 0
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EntrySelection<C = ()> {
    pub source: SourceObservation,
    pub target: Option<Kind>,
    pub comparison: Comparison,
    pub baseline: Option<Action>,
    pub action: Action,
    pub install: Option<InstallAuthority>,
    pub removal: Option<RemovalAuthority>,
    pub desired: Option<DesiredInput>,
    pub interior: Option<Interior<C>>,
}
impl<C> EntrySelection<C> {
    pub fn shape(&self) -> Shape {
        Shape {
            source: self.source,
            target: self.target,
        }
    }
    pub fn has_work(&self) -> bool {
        has_namespace_work(self.action)
    }
    pub fn application_need(&self, domain: SelectionDomain) -> ApplicationNeed {
        if self.has_work()
            || self
                .interior
                .as_ref()
                .is_some_and(|i| i.role != InteriorRole::Paired || matches!(domain, SelectionDomain::Listed { .. }))
        {
            ApplicationNeed::LocalOrOpaque
        } else {
            ApplicationNeed::None
        }
    }
}
fn retained<C>(domain: SelectionDomain, entry: &EntrySelection<C>) -> bool {
    match entry.action {
        Action::PreserveTree => {
            !(matches!(domain, SelectionDomain::Listed { .. })
                && entry.source == SourceObservation::NotRequested
                && entry.target.is_none()
                && entry.desired.is_none()
                && entry.interior.is_none()
                && entry.install.is_none()
                && entry.removal.is_none())
        }
        Action::KeepLeaf => entry.baseline != Some(Action::KeepLeaf),
        Action::Directory {
            container: Container::Keep,
            descend,
        } => !descend || entry.source != SourceObservation::Present(Kind::Directory),
        _ => false,
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EntryParts<C = ()> {
    pub name: OsString,
    pub entry: EntrySelection<C>,
}
#[derive(Debug)]
pub struct EntrySelectionRef<'a, C = ()> {
    name: &'a OsStr,
    entry: &'a EntrySelection<C>,
}
impl<C> Copy for EntrySelectionRef<'_, C> {}
impl<C> Clone for EntrySelectionRef<'_, C> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<'a, C> EntrySelectionRef<'a, C> {
    pub fn name(&self) -> &'a OsStr {
        self.name
    }
}
impl<C> Deref for EntrySelectionRef<'_, C> {
    type Target = EntrySelection<C>;
    fn deref(&self) -> &Self::Target {
        self.entry
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectionParts<C = ()> {
    pub relative: PathBuf,
    pub purpose: Purpose,
    pub structural_root: bool,
    pub domain: SelectionDomain,
    pub input_context: C,
    pub policy: SemanticId,
    pub context_schema: SemanticId,
    pub comparator: SemanticId,
    pub scoped: bool,
    pub root_excluded: bool,
    pub excluded_names: Vec<OsString>,
    pub entries: Vec<EntryParts<C>>,
}

/// Immutable selected semantics. Runtime facts, routes and listing handles are absent.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DirectorySelection<C = ()> {
    relative: PathBuf,
    purpose: Purpose,
    structural_root: bool,
    domain: SelectionDomain,
    input_context: C,
    policy: SemanticId,
    context_schema: SemanticId,
    comparator: SemanticId,
    root_excluded: bool,
    excluded_names: Vec<OsString>,
    names: NameTable,
    entries: Vec<EntrySelection<C>>,
    coverage: DirectoryCoverage,
    diff: DirDiff,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectionError(pub &'static str);
impl std::fmt::Display for SelectionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
    }
}
impl std::error::Error for SelectionError {}
fn require(valid: bool, message: &'static str) -> Result<(), SelectionError> {
    if valid { Ok(()) } else { Err(SelectionError(message)) }
}

impl<C: Clone + Eq> DirectorySelection<C> {
    pub fn from_parts(parts: SelectionParts<C>) -> Result<Self, SelectionError> {
        validate_selection(&parts)?;
        let mut coverage = DirectoryCoverage {
            scoped: parts.scoped,
            excluded: parts.excluded_names.len() as u64 + u64::from(parts.root_excluded),
            ..DirectoryCoverage::default()
        };
        let mut diff = DirDiff {
            dir: parts.relative.clone(),
            ..DirDiff::default()
        };
        for item in &parts.entries {
            let e = &item.entry;
            coverage.planned += u64::from(e.has_work());
            coverage.deviated += u64::from(Some(e.action) != e.baseline);
            coverage.retained += u64::from(retained(parts.domain, e));
            match e.comparison {
                Comparison::Equal => coverage.equal += 1,
                Comparison::Different => coverage.different += 1,
                Comparison::NotAssessed => coverage.not_assessed += 1,
                Comparison::Unsupported { .. } => coverage.unsupported += 1,
                Comparison::Unresolved { .. } => coverage.unresolved += 1,
            }
            coverage.stopped_paired += u64::from(
                e.source == SourceObservation::Present(Kind::Directory)
                    && e.target == Some(Kind::Directory)
                    && !matches!(e.interior.as_ref().map(|i| i.role), Some(InteriorRole::Paired)),
            );
            if e.application_need(parts.domain) == ApplicationNeed::LocalOrOpaque {
                coverage.application_need = ApplicationNeed::LocalOrOpaque;
            }
            if e.comparison == Comparison::Different {
                let s = e.source.kind();
                let t = e.target;
                if s.is_some_and(Kind::is_leaf) && t.is_some_and(Kind::is_leaf) {
                    diff.files.changed.push(item.name.clone());
                } else {
                    if let Some(s) = s {
                        if s == Kind::Directory {
                            diff.dirs.added.push(item.name.clone());
                        } else {
                            diff.files.added.push(item.name.clone());
                        }
                    }
                    if let Some(t) = t {
                        if t == Kind::Directory {
                            diff.dirs.removed.push(item.name.clone());
                        } else {
                            diff.files.removed.push(item.name.clone());
                        }
                    }
                }
            }
        }
        let (names, entries): (Vec<_>, Vec<_>) = parts.entries.into_iter().map(|e| (e.name, e.entry)).unzip();
        Ok(Self {
            relative: parts.relative,
            purpose: parts.purpose,
            structural_root: parts.structural_root,
            domain: parts.domain,
            input_context: parts.input_context,
            policy: parts.policy,
            context_schema: parts.context_schema,
            comparator: parts.comparator,
            root_excluded: parts.root_excluded,
            excluded_names: parts.excluded_names,
            names: NameTable::new(names),
            entries,
            coverage,
            diff,
        })
    }
    pub fn to_parts(&self) -> SelectionParts<C> {
        SelectionParts {
            relative: self.relative.clone(),
            purpose: self.purpose,
            structural_root: self.structural_root,
            domain: self.domain,
            input_context: self.input_context.clone(),
            policy: self.policy.clone(),
            context_schema: self.context_schema.clone(),
            comparator: self.comparator.clone(),
            scoped: self.coverage.scoped,
            root_excluded: self.root_excluded,
            excluded_names: self.excluded_names.clone(),
            entries: self
                .entries()
                .map(|e| EntryParts {
                    name: e.name().to_owned(),
                    entry: e.entry.clone(),
                })
                .collect(),
        }
    }
}
impl<C> DirectorySelection<C> {
    pub fn relative(&self) -> &Path {
        &self.relative
    }
    pub fn purpose(&self) -> Purpose {
        self.purpose
    }
    pub fn is_root(&self) -> bool {
        self.structural_root
    }
    pub fn domain(&self) -> SelectionDomain {
        self.domain
    }
    pub fn input_context(&self) -> &C {
        &self.input_context
    }
    pub fn policy(&self) -> &SemanticId {
        &self.policy
    }
    pub fn context_schema(&self) -> &SemanticId {
        &self.context_schema
    }
    pub fn comparator(&self) -> &SemanticId {
        &self.comparator
    }
    pub fn coverage(&self) -> DirectoryCoverage {
        self.coverage
    }
    pub fn diff(&self) -> &DirDiff {
        &self.diff
    }
    pub fn excluded_names(&self) -> &[OsString] {
        &self.excluded_names
    }
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
    pub fn entry(&self, slot: SlotId) -> EntrySelectionRef<'_, C> {
        EntrySelectionRef {
            name: self.names.get(NameId(slot.0)),
            entry: &self.entries[slot.0],
        }
    }
    pub fn entries(&self) -> impl ExactSizeIterator<Item = EntrySelectionRef<'_, C>> {
        (0..self.len()).map(|i| self.entry(SlotId(i)))
    }
    pub fn paired_children(&self) -> impl Iterator<Item = EntrySelectionRef<'_, C>> {
        self.entries()
            .filter(|e| !self.structural_root && e.interior.as_ref().is_some_and(|i| i.role == InteriorRole::Paired))
    }
}

pub fn baseline(shape: Shape, compared: Comparison) -> Option<Action> {
    if !shape.supported() || !matches!(compared, Comparison::Equal | Comparison::Different) {
        return None;
    }
    let s = shape.source.kind();
    let t = shape.target;
    Some(match (s, t) {
        (Some(Kind::Directory), Some(Kind::Directory)) => Action::Directory {
            container: Container::Keep,
            descend: true,
        },
        (Some(Kind::Directory), Some(_)) => Action::ReplaceWithDirectory { descend: true },
        (Some(_), Some(Kind::Directory)) => Action::ReplaceWithLeaf,
        (Some(Kind::Directory), None) => Action::Directory {
            container: Container::Create,
            descend: true,
        },
        (None, Some(Kind::Directory)) => Action::Directory {
            container: Container::Remove,
            descend: true,
        },
        (None, Some(_)) => Action::RemoveLeaf,
        (Some(_), _) if compared == Comparison::Different => Action::InstallLeaf,
        _ => Action::KeepLeaf,
    })
}
pub(crate) fn baseline_at(shape: Shape, compared: Comparison, structural_root: bool) -> Option<Action> {
    if structural_root
        && shape.source == SourceObservation::Absent
        && shape.target == Some(Kind::Directory)
        && matches!(compared, Comparison::Equal | Comparison::Different)
    {
        Some(Action::Directory {
            container: Container::Keep,
            descend: true,
        })
    } else {
        baseline(shape, compared)
    }
}
pub fn interior_role(shape: Shape, action: Action) -> Option<InteriorRole> {
    match action {
        Action::ReplaceWithLeaf => Some(InteriorRole::RemoveForReplacement),
        Action::ReplaceWithDirectory { .. } => Some(InteriorRole::PrepareReplacement),
        Action::Directory { descend: true, .. } => Some(match (shape.source.kind(), shape.target) {
            (Some(Kind::Directory), Some(Kind::Directory)) => InteriorRole::Paired,
            (Some(Kind::Directory), _) => InteriorRole::SourceOnly,
            _ => InteriorRole::TargetOnly,
        }),
        _ => None,
    }
}
pub fn has_namespace_work(action: Action) -> bool {
    matches!(
        action,
        Action::InstallLeaf
            | Action::RemoveLeaf
            | Action::ReplaceWithLeaf
            | Action::ReplaceWithDirectory { .. }
            | Action::Directory {
                container: Container::Create | Container::CreateIfNeeded | Container::Remove,
                ..
            }
    )
}
pub fn mirror_install(shape: Shape, action: Action) -> Option<InstallAuthority> {
    matches!(action, Action::InstallLeaf | Action::ReplaceWithLeaf | Action::ReplaceWithDirectory { .. }).then_some(InstallAuthority {
        intent: InstallIntent::ReconcileDifference,
        destination: match shape.target {
            None => DestinationGrant::AbsentOnly,
            Some(Kind::Directory) => DestinationGrant::ReplaceTree,
            _ => DestinationGrant::ReplaceLeaf,
        },
    })
}

/// This table is used both before publication and when reconstructing saved values.
pub fn validate_entry<C>(domain: SelectionDomain, e: &EntrySelection<C>) -> Result<(), SelectionError> {
    validate_entry_at(domain, e, false)
}
pub(crate) fn validate_entry_at<C>(domain: SelectionDomain, e: &EntrySelection<C>, structural_root: bool) -> Result<(), SelectionError> {
    let shape = e.shape();
    let s = e.source.kind();
    let t = e.target;
    let sd = s == Some(Kind::Directory);
    let td = t == Some(Kind::Directory);
    let sl = s.is_some_and(Kind::is_leaf);
    let tl = t.is_some_and(Kind::is_leaf);
    if matches!(domain, SelectionDomain::PhysicalUnion) {
        require(
            e.source != SourceObservation::NotRequested && shape.presence().is_some(),
            "physical entry has no actual presence",
        )?;
        require(e.desired.is_none(), "physical union contains desired reference")?;
    } else {
        require(e.comparison == Comparison::NotAssessed, "listed work is not a physical-union comparison")?;
        if let Some(desired) = e.desired {
            require(matches!(e.source, SourceObservation::Present(_)), "desired source obligation is missing")?;
            let expected = match desired {
                DesiredInput::Directory { .. } => {
                    require(
                        matches!(
                            e.action,
                            Action::Directory { descend: true, .. } | Action::ReplaceWithDirectory { descend: true }
                        ),
                        "desired routing input must cover its declared descendants",
                    )?;
                    sd
                }
                DesiredInput::Leaf {
                    expected: LeafExpectation::SupportedLeaf,
                    ..
                } => sl,
                DesiredInput::Leaf {
                    expected: LeafExpectation::Exact { kind, executable },
                    ..
                } => {
                    require(kind == LeafKind::RegularFile || executable.is_none(), "executable expectation on symlink")?;
                    s == Some(match kind {
                        LeafKind::RegularFile => Kind::File,
                        LeafKind::Symlink => Kind::Symlink,
                    })
                }
            };
            require(expected, "desired source kind mismatch")?;
            if matches!(desired, DesiredInput::Leaf { .. }) {
                require(
                    matches!(e.action, Action::InstallLeaf | Action::ReplaceWithLeaf),
                    "desired leaf obligation must select installation",
                )?;
            }
        } else {
            require(
                e.source == SourceObservation::NotRequested,
                "obsolete listed input queried or fabricated source absence",
            )?;
        }
    }
    match e.comparison {
        Comparison::Equal => require(shape.supported() && s.is_some() && s == t, "incompatible shape marked equal")?,
        Comparison::Different => require(shape.supported() && !(sd && td), "invalid structural difference")?,
        Comparison::Unsupported { .. } | Comparison::Unresolved { .. } if shape.supported() => {
            require(sl && tl && s == t, "structural assessment must be resolved by wrapper")?
        }
        _ => {}
    }
    if !shape.supported() {
        require(
            e.action == Action::PreserveTree
                && matches!(
                    e.comparison,
                    Comparison::Unsupported {
                        code: crate::comparison::UNSUPPORTED_PHYSICAL_KIND
                    } | Comparison::NotAssessed
                ),
            "unsupported physical kind requires preservation",
        )?;
    }
    require(e.baseline == baseline_at(shape, e.comparison, structural_root), "invalid comparison baseline")?;
    let valid = match e.action {
        Action::PreserveTree => true,
        Action::KeepLeaf => tl || (t.is_none() && sl),
        Action::InstallLeaf => sl && !td,
        Action::RemoveLeaf => tl && matches!(e.source, SourceObservation::Absent | SourceObservation::NotRequested),
        Action::ConfirmAbsent => {
            matches!(domain, SelectionDomain::Listed { .. })
                && e.source == SourceObservation::NotRequested
                && t.is_none()
                && e.desired.is_none()
                && matches!(e.removal, Some(RemovalAuthority::OwnedLeaf { .. }))
        }
        Action::ReplaceWithLeaf => sl && td,
        Action::ReplaceWithDirectory { .. } => sd && tl,
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
        } => matches!(e.source, SourceObservation::Absent | SourceObservation::NotRequested) && td && descend,
    };
    require(valid, "action is incompatible with observed shape")?;
    let installing = matches!(e.action, Action::InstallLeaf | Action::ReplaceWithLeaf | Action::ReplaceWithDirectory { .. });
    require(installing == e.install.is_some(), "missing or unused installation authority")?;
    if let Some(grant) = e.install {
        require(
            grant.intent != InstallIntent::ReconcileDifference || e.comparison == Comparison::Different,
            "difference installation requires an observed difference",
        )?;
        require(
            match t {
                None => true,
                Some(Kind::File | Kind::Symlink) => grant.destination != DestinationGrant::AbsentOnly,
                Some(Kind::Directory) => grant.destination == DestinationGrant::ReplaceTree,
                _ => false,
            },
            "destination exceeds installation grant",
        )?;
    }
    let deleting = matches!(
        e.action,
        Action::RemoveLeaf
            | Action::ConfirmAbsent
            | Action::Directory {
                container: Container::Remove,
                ..
            }
    );
    match domain {
        SelectionDomain::PhysicalUnion => require(
            if deleting {
                e.removal == Some(RemovalAuthority::SourceAbsent) && e.source == SourceObservation::Absent
            } else {
                e.removal.is_none()
            },
            "invalid physical removal authority",
        )?,
        SelectionDomain::Listed { desired, owned } => {
            let needs_removal = deleting || (installing && t.is_some());
            require(needs_removal == e.removal.is_some(), "missing or unused listed removal authority")?;
            if let Some(authority) = e.removal {
                require(
                    match authority {
                        RemovalAuthority::OwnedLeaf { owned: id, .. } => id == owned && (!td || e.action == Action::ConfirmAbsent),
                        RemovalAuthority::CoupledOwned { owned: id, .. } => {
                            id == owned
                                && td
                                && matches!(
                                    e.action,
                                    Action::ReplaceWithLeaf
                                        | Action::Directory {
                                            container: Container::Remove,
                                            descend: true
                                        }
                                )
                        }
                        RemovalAuthority::SourceAbsent => false,
                    },
                    "listed removal is not scoped to appropriate owned input",
                )?;
            }
            if let Some(interior) = &e.interior {
                require(
                    matches!(interior.input, InteriorInput::Listed { desired: id, .. } if id == desired),
                    "listed interior escaped manifest input",
                )?;
            }
        }
    }
    require(
        e.interior.as_ref().map(|i| i.role) == interior_role(shape, e.action),
        "missing, unused or wrong-role interior context",
    )?;
    if matches!(domain, SelectionDomain::PhysicalUnion) {
        require(
            e.interior.as_ref().is_none_or(|i| i.input == InteriorInput::PhysicalDirectory),
            "physical interior uses declared input",
        )?;
    }
    Ok(())
}

pub fn validate_selection<C>(parts: &SelectionParts<C>) -> Result<(), SelectionError> {
    require(
        parts.policy.valid() && parts.context_schema.valid() && parts.comparator.valid(),
        "invalid semantic identity",
    )?;
    require(normal_relative(&parts.relative), "invalid selection coordinate")?;
    require(
        !matches!(parts.domain, SelectionDomain::Listed { .. }) || parts.excluded_names.is_empty(),
        "listed obligations cannot be silently excluded",
    )?;
    require(
        parts.entries.windows(2).all(|p| p[0].name.as_encoded_bytes() < p[1].name.as_encoded_bytes()),
        "unordered selected names",
    )?;
    require(
        parts.excluded_names.windows(2).all(|p| p[0].as_encoded_bytes() < p[1].as_encoded_bytes()),
        "unordered excluded names",
    )?;
    require(
        parts
            .excluded_names
            .iter()
            .all(|n| native_name(n) && parts.entries.binary_search_by(|e| e.name.as_encoded_bytes().cmp(n.as_encoded_bytes())).is_err()),
        "invalid or overlapping excluded name",
    )?;
    if parts.structural_root {
        require(
            parts.relative.as_os_str().is_empty()
                && parts.entries.len() == 1
                && parts.excluded_names.is_empty()
                && parts.domain == SelectionDomain::PhysicalUnion,
            "invalid structural root selection",
        )?;
        require(
            !parts.root_excluded || parts.entries[0].entry.action == Action::PreserveTree,
            "invalid root exclusion",
        )?;
    } else {
        require(!parts.root_excluded, "non-root exclusion flag")?;
    }
    for item in &parts.entries {
        require(
            !item.name.as_encoded_bytes().contains(&0) && (parts.structural_root || native_name(&item.name)),
            "invalid selection basename",
        )?;
        validate_entry_at(parts.domain, &item.entry, parts.structural_root)?;
        if parts.structural_root {
            require(item.entry.shape().supported(), "unsupported structural root")?;
        }
    }
    Ok(())
}
pub fn normal_relative(path: &Path) -> bool {
    !path.as_os_str().as_encoded_bytes().contains(&0)
        && path.components().all(|c| matches!(c, Component::Normal(_)))
        && path.components().collect::<PathBuf>().as_os_str() == path.as_os_str()
}
pub fn native_name(name: &OsStr) -> bool {
    !name.as_encoded_bytes().contains(&0)
        && matches!(Path::new(name).components().next(), Some(Component::Normal(_)))
        && Path::new(name).components().count() == 1
        && Path::new(name).file_name() == Some(name)
}
