//! One finite inspection engine for contextual policies and legacy full visitors.

use std::ffi::OsString;
use std::fs::Metadata;
use std::path::{Path, PathBuf};
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

pub(crate) use crate::acquire::{Acquire, Physical};
use crate::acquire::{AcquisitionCell, EntryRuntime};
use crate::comparison::structural;
use crate::policy::{BatchInfo, Choice};
use crate::prepared::LiveBinding;
use crate::selection::{baseline, interior_role, mirror_install, normal_relative};
use crate::visitor::{Name, Names, refusal};
use crate::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DirectoryComparison {
    pub(crate) diff: DirDiff,
    pub(crate) coverage: DirectoryCoverage,
    pub(crate) children: Vec<OsString>,
    pub(crate) root: bool,
}
impl DirectoryComparison {
    pub fn diff(&self) -> &DirDiff {
        &self.diff
    }
    pub fn coverage(&self) -> DirectoryCoverage {
        self.coverage
    }
    pub fn paired_children(&self) -> &[OsString] {
        &self.children
    }
    pub fn is_root(&self) -> bool {
        self.root
    }
    pub fn into_parts(self) -> (DirDiff, DirectoryCoverage, Vec<OsString>, bool) {
        (self.diff, self.coverage, self.children, self.root)
    }
}
#[derive(Clone, Copy, Debug)]
pub struct TargetVisitor<'a, V> {
    pub target: usize,
    pub visitor: &'a V,
}
#[derive(Debug)]
pub struct TargetComparison<T = DirectoryComparison> {
    pub target: usize,
    pub outcome: Result<T, Arc<Error>>,
}

#[derive(Debug)]
pub enum SharedBindError {
    NoTargets,
    Source(Error),
    Target { target: usize, error: Error },
}
impl std::fmt::Display for SharedBindError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoTargets => f.write_str("shared comparison requires targets"),
            Self::Source(e) => write!(f, "bind source: {e}"),
            Self::Target { target, error } => write!(f, "bind target {target}: {error}"),
        }
    }
}
impl std::error::Error for SharedBindError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::NoTargets => None,
            Self::Source(e) | Self::Target { error: e, .. } => Some(e),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SharedInputError {
    InvalidDirectory(PathBuf),
    UnknownTarget(usize),
    UnorderedTargets,
    InvalidProfile,
    EmptyRoot,
}
impl std::fmt::Display for SharedInputError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidDirectory(p) => write!(f, "invalid root-relative directory: {p:?}"),
            Self::UnknownTarget(i) => write!(f, "unknown target: {i}"),
            Self::UnorderedTargets => f.write_str("targets must be distinct and ordered"),
            Self::InvalidProfile => f.write_str("LegacyFull requires StatKey comparison"),
            Self::EmptyRoot => f.write_str("root assessment requires at least one actual side"),
        }
    }
}
impl std::error::Error for SharedInputError {}

/// Runtime source-generation binding. It owns no policy or recursive progress.
/// Cells currently share within a joined call; bounded cross-call retention is a
/// separate subsequent implementation, not an unbounded implicit catalog here.
#[derive(Debug)]
pub struct SourceSession {
    root: PathBuf,
    generation: u64,
    resources: diff_dirs_tree_observe::ObservationResources,
}
static GENERATION: AtomicU64 = AtomicU64::new(1);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SourceCacheLimits {
    pub batches: usize,
    pub charged_capacity: usize,
}
impl Default for SourceCacheLimits {
    fn default() -> Self {
        Self {
            batches: 32,
            charged_capacity: 8 * 1024 * 1024,
        }
    }
}
impl SourceSession {
    /// Establish runtime identity, not a lasting route certificate. Caller admits
    /// routes before immediate inspection, as for the existing admitted API.
    pub fn new(root: impl AsRef<Path>) -> Self {
        Self::with_resources(root, diff_dirs_tree_observe::ObservationResources::default())
    }
    pub fn with_resources(root: impl AsRef<Path>, resources: diff_dirs_tree_observe::ObservationResources) -> Self {
        let generation = GENERATION
            .try_update(Ordering::Relaxed, Ordering::Relaxed, |v| v.checked_add(1))
            .expect("source generation space exhausted");
        Self {
            root: root.as_ref().to_owned(),
            generation,
            resources,
        }
    }
    pub fn root(&self) -> &Path {
        &self.root
    }
    pub fn generation(&self) -> u64 {
        self.generation
    }
    pub fn resources(&self) -> &diff_dirs_tree_observe::ObservationResources {
        &self.resources
    }
}
pub use crate::epoch::InspectionEpoch;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DirectoryAdmission {
    RootGated,
    AlreadyAdmitted,
}
#[derive(Clone, Copy, Debug)]
/// Immediate physical input. Omitted routes mean genuine structural absence,
/// never policy filtering. Callers own no-follow route admission and confinement.
pub struct DirectoryRequest<'a> {
    pub relative: &'a Path,
    pub source_directory: Option<&'a Path>,
    pub purpose: Purpose,
    pub admission: DirectoryAdmission,
}
/// Lane-local frozen interpretation and input context. Logical roots remain final
/// coordinates even when the physical directory is a private preparation location.
pub struct TargetInspection<'a, P: Policy, M: Comparator> {
    pub target: usize,
    pub logical_root: &'a Path,
    pub physical_directory: Option<&'a Path>,
    pub context: &'a P::Context,
    pub policy: &'a P,
    pub comparator: &'a M,
}

/// Inspect one joined immediate source coordinate, sharing per-entry acquisition
/// outcomes across independently assessed lanes. No traversal or effects occur.
/// Every selected execution input completes before a prepared lane is returned.
/// Root gating derives the interior input context once; an admitted call never
/// replays that gate. Callers admit physical routes through the call-local epoch.
pub fn inspect_group<P: Policy, M: Comparator, E: ExecutionDemand>(
    source: &SourceSession,
    at: DirectoryRequest<'_>,
    lanes: &[TargetInspection<'_, P, M>],
    execution_demand: &E,
    _epoch: &InspectionEpoch,
    execution: Execution,
) -> Result<Vec<TargetComparison<PreparedDirectory<P::Context>>>, SharedInputError> {
    inspect_group_using(source, at, lanes, execution_demand, execution, &Physical(source.resources()), None)
}

type Listed = Vec<(OsString, Arc<AcquisitionCell>)>;
#[derive(Clone)]
struct PairSlot {
    name: OsString,
    source: Option<Arc<AcquisitionCell>>,
    target: Option<Arc<AcquisitionCell>>,
}
struct Admitted {
    pairs: Vec<PairSlot>,
    excluded: Vec<OsString>,
    root_excluded: bool,
}
struct Ready<C> {
    position: usize,
    context: C,
}
type PreparedOutcome<C> = Option<Result<PreparedDirectory<C>, Arc<Error>>>;

#[allow(clippy::too_many_arguments)]
pub(crate) fn inspect_group_using<P: Policy, M: Comparator, E: ExecutionDemand>(
    source: &SourceSession,
    at: DirectoryRequest<'_>,
    lanes: &[TargetInspection<'_, P, M>],
    demand: &E,
    execution: Execution,
    acquire: &impl Acquire,
    bound: Option<(&Metadata, &[Metadata])>,
) -> Result<Vec<TargetComparison<PreparedDirectory<P::Context>>>, SharedInputError> {
    if !normal_relative(at.relative) {
        return Err(SharedInputError::InvalidDirectory(at.relative.to_owned()));
    }
    if lanes.windows(2).any(|p| p[0].target >= p[1].target) {
        return Err(SharedInputError::UnorderedTargets);
    }
    if lanes
        .iter()
        .any(|l| l.policy.observation_profile() == ObservationProfile::LegacyFull && l.comparator.identity() != StatKey.identity())
    {
        return Err(SharedInputError::InvalidProfile);
    }
    if at.relative.as_os_str().is_empty()
        && at.admission == DirectoryAdmission::RootGated
        && at.source_directory.is_none()
        && lanes.iter().any(|l| l.physical_directory.is_none())
    {
        return Err(SharedInputError::EmptyRoot);
    }
    let mut outcomes: Vec<PreparedOutcome<P::Context>> = lanes.iter().map(|_| None).collect();
    let mut ready = Vec::new();
    for (position, lane) in lanes.iter().enumerate() {
        let site = ContextSite {
            relative: at.relative,
            structural_root: at.relative.as_os_str().is_empty() && at.admission == DirectoryAdmission::RootGated,
            role: None,
        };
        if let Err(cause) = lane.policy.validate_context(site, lane.context) {
            outcomes[position] = Some(Err(Arc::new(refusal(&lane.logical_root.join(at.relative), cause))));
        } else {
            ready.push(Ready {
                position,
                context: lane.context.clone(),
            });
        }
    }
    if at.relative.as_os_str().is_empty() && at.admission == DirectoryAdmission::RootGated && !ready.is_empty() {
        let source_cell = at
            .source_directory
            .map(|path| {
                bound
                    .map_or_else(|| acquire.root_metadata(path), |(m, _)| Ok(m.clone()))
                    .map(|m| Arc::new(AcquisitionCell::bound(path.to_owned(), m)))
            })
            .transpose();
        match source_cell {
            Err(error) => {
                let error = Arc::new(error);
                for r in ready.drain(..) {
                    outcomes[r.position] = Some(Err(error.clone()));
                }
            }
            Ok(source_cell) => {
                let mut admitted_roots = Vec::new();
                for r in ready.drain(..) {
                    let lane = &lanes[r.position];
                    if lane.policy.observation_profile() == ObservationProfile::LegacyFull
                        && let Some(cell) = &source_cell
                        && let Err(error) = cell.full(true, true, acquire)
                    {
                        outcomes[r.position] = Some(Err(error));
                        continue;
                    }
                    let target_cell = lane
                        .physical_directory
                        .map(|path| {
                            bound
                                .map_or_else(|| acquire.root_metadata(path), |(_, ms)| Ok(ms[r.position].clone()))
                                .map(|m| Arc::new(AcquisitionCell::bound(path.to_owned(), m)))
                        })
                        .transpose();
                    let result = target_cell.map_err(Arc::new).and_then(|target| {
                        let name = source_cell
                            .as_ref()
                            .map_or(lane.logical_root, |s| s.path())
                            .file_name()
                            .unwrap_or(source.root().as_os_str())
                            .to_owned();
                        assess(
                            source,
                            at,
                            lane,
                            &r.context,
                            vec![PairSlot {
                                name,
                                source: source_cell.clone(),
                                target,
                            }],
                            true,
                            demand,
                            execution,
                            acquire,
                        )
                    });
                    match result {
                        Ok(root) if root.selected.entry(SlotId(0)).interior.as_ref().is_some_and(|i| i.role == InteriorRole::Paired) => {
                            let context = root.selected.entry(SlotId(0)).interior.as_ref().expect("paired interior").context.clone();
                            admitted_roots.push(Ready { position: r.position, context });
                        }
                        result => outcomes[r.position] = Some(result),
                    }
                }
                ready = admitted_roots;
            }
        }
    }
    if !ready.is_empty() {
        let listed_source = at.source_directory.map(|p| listed(p, acquire)).transpose();
        match listed_source {
            Err(error) => {
                let error = Arc::new(error);
                for r in ready {
                    outcomes[r.position] = Some(Err(error.clone()));
                }
            }
            Ok(source_entries) => {
                // Every lane's names callback precedes all child acquisition.
                let mut admitted = Vec::new();
                for r in ready {
                    let lane = &lanes[r.position];
                    let result = lane
                        .physical_directory
                        .map(|p| listed(p, acquire))
                        .transpose()
                        .map_err(Arc::new)
                        .and_then(|target| {
                            admit(
                                at.relative,
                                false,
                                union(source_entries.as_deref().unwrap_or(&[]), target.unwrap_or_default()),
                                lane,
                                &r.context,
                            )
                        });
                    match result {
                        Ok(entries) => admitted.push((r, entries)),
                        Err(error) => outcomes[r.position] = Some(Err(error)),
                    }
                }
                for (r, entries) in admitted {
                    outcomes[r.position] = Some(assess_admitted(
                        source,
                        at,
                        &lanes[r.position],
                        &r.context,
                        entries,
                        false,
                        demand,
                        execution,
                        acquire,
                    ));
                }
            }
        }
    }
    Ok(lanes
        .iter()
        .zip(outcomes)
        .map(|(lane, outcome)| TargetComparison {
            target: lane.target,
            outcome: outcome.expect("all lanes assessed"),
        })
        .collect())
}

fn union(source: &[(OsString, Arc<AcquisitionCell>)], target: Listed) -> Vec<PairSlot> {
    let mut source = source.iter().peekable();
    let mut target = target.into_iter().peekable();
    let mut entries = Vec::with_capacity(source.len() + target.len());
    while source.peek().is_some() || target.peek().is_some() {
        let order = match (source.peek(), target.peek()) {
            (Some((s, _)), Some((t, _))) => s.as_encoded_bytes().cmp(t.as_encoded_bytes()),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            _ => unreachable!(),
        };
        let s = (order != std::cmp::Ordering::Greater).then(|| source.next().expect("source peeked"));
        let t = (order != std::cmp::Ordering::Less).then(|| target.next().expect("target peeked"));
        let name = s.map_or_else(|| t.as_ref().expect("present target").0.clone(), |(name, _)| name.clone());
        entries.push(PairSlot {
            name,
            source: s.map(|(_, cell)| cell.clone()),
            target: t.map(|(_, cell)| cell),
        });
    }
    entries
}
fn listed(path: &Path, acquire: &impl Acquire) -> Result<Listed, Error> {
    let mut entries: Vec<_> = acquire
        .list(path)?
        .into_iter()
        .map(|e| (e.file_name(), Arc::new(AcquisitionCell::listed(e))))
        .collect();
    entries.sort_unstable_by(|a, b| a.0.as_encoded_bytes().cmp(b.0.as_encoded_bytes()));
    if entries.windows(2).any(|p| p[0].0 == p[1].0) {
        return Err(Error::io(
            path.to_owned(),
            "read directory entry",
            std::io::Error::new(std::io::ErrorKind::InvalidData, "duplicate physical listing name"),
        ));
    }
    Ok(entries)
}
fn admit<P: Policy, M: Comparator>(
    relative: &Path,
    root: bool,
    pairs: Vec<PairSlot>,
    lane: &TargetInspection<'_, P, M>,
    context: &P::Context,
) -> Result<Admitted, Arc<Error>> {
    let mut names = Names {
        dir: relative.to_owned(),
        root,
        entries: pairs
            .iter()
            .map(|p| Name {
                name: p.name.clone(),
                relative: if root { PathBuf::new() } else { relative.join(&p.name) },
                presence: physical_presence(p.source.is_some(), p.target.is_some()),
                source: p.source.as_ref().map(|s| s.hints()),
                target: p.target.as_ref().map(|t| t.hints()),
                admission: Admission::Observe,
            })
            .collect(),
    };
    lane.policy
        .names(context, &mut names)
        .map_err(|e| Arc::new(refusal(&lane.logical_root.join(relative), e)))?;
    if names.entries.iter().zip(&pairs).any(|(n, p)| n.name != p.name) {
        return Err(Arc::new(refusal(
            &lane.logical_root.join(relative),
            "policy reordered immutable union names".into(),
        )));
    }
    let mut admitted = Admitted {
        pairs: Vec::new(),
        excluded: Vec::new(),
        root_excluded: false,
    };
    for (name, pair) in names.entries.into_iter().zip(pairs) {
        if name.admission == Admission::PreserveTree {
            if root {
                admitted.root_excluded = true;
                admitted.pairs.push(pair);
            } else {
                admitted.excluded.push(name.name);
            }
        } else {
            admitted.pairs.push(pair);
        }
    }
    Ok(admitted)
}
fn physical_presence(source: bool, target: bool) -> Presence {
    match (source, target) {
        (true, true) => Presence::Both,
        (true, false) => Presence::SourceOnly,
        (false, true) => Presence::TargetOnly,
        _ => unreachable!("empty physical union slot"),
    }
}

#[allow(clippy::too_many_arguments)]
fn assess<P: Policy, M: Comparator, E: ExecutionDemand>(
    source: &SourceSession,
    at: DirectoryRequest<'_>,
    lane: &TargetInspection<'_, P, M>,
    context: &P::Context,
    pairs: Vec<PairSlot>,
    root: bool,
    demand: &E,
    execution: Execution,
    acquire: &impl Acquire,
) -> Result<PreparedDirectory<P::Context>, Arc<Error>> {
    if root && lane.policy.observation_profile() == ObservationProfile::LegacyFull {
        for pair in &pairs {
            let source = pair.source.as_ref().map(|s| s.full(true, true, acquire)).transpose()?;
            let read_target = source.as_ref().is_some_and(|s| s.kind == Kind::Symlink) || at.purpose == Purpose::Audit;
            if let Some(target) = &pair.target {
                target.full(read_target, false, acquire)?;
            }
        }
    }
    let admitted = admit(at.relative, root, pairs, lane, context)?;
    assess_admitted(source, at, lane, context, admitted, root, demand, execution, acquire)
}
#[allow(clippy::too_many_arguments)]
fn assess_admitted<P: Policy, M: Comparator, E: ExecutionDemand>(
    source: &SourceSession,
    at: DirectoryRequest<'_>,
    lane: &TargetInspection<'_, P, M>,
    context: &P::Context,
    admitted: Admitted,
    root: bool,
    demand: &E,
    execution: Execution,
    acquire: &impl Acquire,
) -> Result<PreparedDirectory<P::Context>, Arc<Error>> {
    let info = BatchInfo {
        source_root: source.root(),
        target_root: lane.logical_root,
        relative: at.relative,
        structural_root: root,
        purpose: at.purpose,
    };
    let target_path = lane.logical_root.join(at.relative);
    let error = |cause| Arc::new(refusal(&target_path, cause));
    lane.policy
        .validate_context(
            ContextSite {
                relative: at.relative,
                structural_root: root,
                role: None,
            },
            context,
        )
        .map_err(error)?;
    let mut choices;
    let mut runtime;
    if lane.policy.observation_profile() == ObservationProfile::LegacyFull {
        // Rank the complete per-name source/target/comparison outcome, not global
        // kind, metadata, link and comparison error lists. Peers only share cells.
        let observed = indexed_map(&admitted.pairs, execution, |pair| {
            #[cfg(all(test, feature = "parallel"))]
            if !root {
                crate::roots::tests::observe_worker();
            }
            let s = pair.source.as_ref().map(|s| s.full(true, true, acquire)).transpose()?;
            let read_target = if root {
                s.as_ref().is_some_and(|s| s.kind == Kind::Symlink) || at.purpose == Purpose::Audit
            } else {
                pair.source.is_some() || at.purpose == Purpose::Audit
            };
            let t = pair.target.as_ref().map(|t| t.full(read_target, false, acquire)).transpose()?;
            let facts = EntryRuntime {
                source: s,
                target: t,
                demand: SideDemands {
                    source: Demand::FULL,
                    target: Demand {
                        attributes: true,
                        literal_link: read_target,
                    },
                },
            };
            let shape = runtime_shape(&facts);
            let compared = compare(pair, shape, facts.view(), source, at, lane, acquire)?;
            Ok((choice(pair, shape, compared), facts))
        })
        .into_iter()
        .collect::<Result<Vec<_>, Arc<Error>>>()?;
        (choices, runtime) = observed.into_iter().unzip::<_, _, Vec<_>, Vec<_>>();
        // Full profile is explicit, but still goes through the same shape/final
        // callbacks and validator. FullVisitor itself has no shape behavior.
        lane.policy.shape(context, &mut ShapeChoices { info, entries: &mut choices }).map_err(error)?;
    } else {
        let shapes = indexed_map(&admitted.pairs, execution, |p| {
            let s = p.source.as_ref().map(|s| s.kind(acquire).map(|(k, _)| k)).transpose()?;
            let t = p.target.as_ref().map(|t| t.kind(acquire).map(|(k, _)| k)).transpose()?;
            Ok(Shape {
                source: s.map_or(SourceObservation::Absent, SourceObservation::Present),
                target: t,
            })
        })
        .into_iter()
        .collect::<Result<Vec<_>, Arc<Error>>>()?;
        choices = admitted.pairs.iter().zip(shapes).map(|(p, s)| choice(p, s, Comparison::NotAssessed)).collect();
        lane.policy.shape(context, &mut ShapeChoices { info, entries: &mut choices }).map_err(error)?;
        runtime = indexed_map(&admitted.pairs.iter().zip(&choices).collect::<Vec<_>>(), execution, |(p, c)| {
            let comparator_demand = if c.assessment == Assessment::Compare && structural(c.shape).is_none() {
                lane.comparator.demand(c.shape)
            } else {
                SideDemands::default()
            };
            acquire_pair(p, c.demand.union(comparator_demand), acquire)
        })
        .into_iter()
        .collect::<Result<Vec<_>, Arc<Error>>>()?;
        let comparisons = indexed_map(
            &admitted.pairs.iter().zip(&choices).zip(&runtime).collect::<Vec<_>>(),
            execution,
            |((p, c), facts)| {
                if c.assessment == Assessment::Omit {
                    Ok(Comparison::NotAssessed)
                } else {
                    compare(p, c.shape, facts.masked(lane.comparator.demand(c.shape)), source, at, lane, acquire)
                }
            },
        )
        .into_iter()
        .collect::<Result<Vec<_>, Arc<Error>>>()?;
        for (choice, compared) in choices.iter_mut().zip(comparisons) {
            choice.comparison = compared;
            choice.baseline = baseline(choice.shape, compared);
        }
    }
    // A selected single-tree target root is a retained structural container.
    if root && choices.len() == 1 && choices[0].shape.source == SourceObservation::Absent && choices[0].shape.target == Some(Kind::Directory) {
        choices[0].action = Some(Action::Directory {
            container: Container::Keep,
            descend: true,
        });
        choices[0].baseline = crate::selection::baseline_at(choices[0].shape, choices[0].comparison, true);
    }
    if admitted.root_excluded {
        choices[0].action = Some(Action::PreserveTree);
    }
    lane.policy
        .select(
            context,
            &mut Choices {
                info,
                entries: &mut choices,
                facts: &runtime,
            },
        )
        .map_err(error)?;
    let mut entries = Vec::with_capacity(choices.len());
    for c in choices {
        let action = if admitted.root_excluded {
            Some(Action::PreserveTree)
        } else {
            c.action.or(c.baseline)
        };
        if !c.shape.supported() && (root || action != Some(Action::PreserveTree)) {
            let path = if c.shape.source.kind() == Some(Kind::Other) {
                source.root()
            } else {
                lane.logical_root
            };
            return Err(Arc::new(Error::UnsupportedEntry {
                path: if root { path.to_owned() } else { path.join(at.relative).join(&c.name) },
            }));
        }
        let action = action.ok_or_else(|| error("entry has no assessed baseline or explicit policy selection".into()))?;
        let install = if admitted.root_excluded {
            None
        } else {
            c.install.or_else(|| (c.action.is_none()).then(|| mirror_install(c.shape, action)).flatten())
        };
        let role = interior_role(c.shape, action);
        let interior = if admitted.root_excluded {
            None
        } else {
            c.interior.or_else(|| {
                role.map(|role| Interior {
                    role,
                    context: context.clone(),
                    input: InteriorInput::PhysicalDirectory,
                })
            })
        };
        if let Some(interior) = &interior {
            let relative = if root { at.relative.to_owned() } else { at.relative.join(&c.name) };
            lane.policy
                .validate_context(
                    ContextSite {
                        relative: &relative,
                        structural_root: root,
                        role: Some(interior.role),
                    },
                    &interior.context,
                )
                .map_err(error)?;
        }
        let removal = matches!(
            action,
            Action::RemoveLeaf
                | Action::Directory {
                    container: Container::Remove,
                    ..
                }
        )
        .then_some(RemovalAuthority::SourceAbsent);
        let entry = EntrySelection {
            source: c.shape.source,
            target: c.shape.target,
            comparison: c.comparison,
            baseline: c.baseline,
            action,
            install,
            removal,
            desired: None,
            interior,
        };
        crate::selection::validate_entry_at(SelectionDomain::PhysicalUnion, &entry, root)
            .map_err(|cause| Arc::new(refusal(&if root { target_path.clone() } else { target_path.join(&c.name) }, Box::new(cause))))?;
        entries.push(EntryParts { name: c.name, entry });
    }
    let selected = DirectorySelection::from_parts(SelectionParts {
        relative: at.relative.to_owned(),
        purpose: at.purpose,
        structural_root: root,
        domain: SelectionDomain::PhysicalUnion,
        input_context: context.clone(),
        policy: lane.policy.identity().clone(),
        context_schema: lane.policy.context_schema().clone(),
        comparator: lane.comparator.identity().clone(),
        scoped: lane.policy.scoped(),
        root_excluded: admitted.root_excluded,
        excluded_names: admitted.excluded,
        entries,
    })
    .map_err(|e| error(Box::new(e)))?;
    if root
        && selected.entry(SlotId(0)).source == SourceObservation::Absent
        && selected.entry(SlotId(0)).target == Some(Kind::Directory)
        && !matches!(
            selected.entry(SlotId(0)).action,
            Action::Directory {
                container: Container::Keep,
                ..
            } | Action::PreserveTree
        )
    {
        return Err(error("selected target root is a retained structural container".into()));
    }
    // All selected execution inputs complete before this immediate batch can be
    // published to any effect owner. Metadata from kind resolution is retained.
    let execution_inputs = indexed_map(&admitted.pairs.iter().enumerate().collect::<Vec<_>>(), execution, |(index, pair)| {
        let extra = demand.for_entry(selected.entry(SlotId(*index)));
        if extra == SideDemands::default() {
            Ok(None)
        } else {
            acquire_pair(pair, runtime[*index].demand.union(extra), acquire).map(Some)
        }
    })
    .into_iter()
    .collect::<Result<Vec<_>, Arc<Error>>>()?;
    for (facts, extra) in runtime.iter_mut().zip(execution_inputs) {
        if let Some(mut extra) = extra {
            // Full legacy projections remain available after execution enrichment.
            if let (Some(old), Some(new)) = (&facts.source, &mut extra.source) {
                new.legacy = old.legacy.clone();
            }
            if let (Some(old), Some(new)) = (&facts.target, &mut extra.target) {
                new.legacy = old.legacy.clone();
            }
            *facts = extra;
        }
    }
    Ok(PreparedDirectory {
        selected: Arc::new(selected),
        facts: runtime,
        binding: LiveBinding {
            source: source.root().to_owned(),
            target: lane.logical_root.to_owned(),
            generation: source.generation(),
        },
    })
}
fn runtime_shape(facts: &EntryRuntime) -> Shape {
    Shape {
        source: facts.source.as_ref().map_or(SourceObservation::Absent, |f| SourceObservation::Present(f.kind)),
        target: facts.target.as_ref().map(|f| f.kind),
    }
}
fn choice<C>(pair: &PairSlot, shape: Shape, compared: Comparison) -> Choice<C> {
    Choice {
        name: pair.name.clone(),
        shape,
        demand: SideDemands::default(),
        assessment: Assessment::Compare,
        comparison: compared,
        baseline: baseline(shape, compared),
        action: None,
        install: None,
        interior: None,
        terminal: false,
    }
}
fn acquire_pair(pair: &PairSlot, demand: SideDemands, acquire: &impl Acquire) -> Result<EntryRuntime, Arc<Error>> {
    let source = pair.source.as_ref().map(|s| s.acquire(demand.source, acquire)).transpose()?;
    let target = pair.target.as_ref().map(|t| t.acquire(demand.target, acquire)).transpose()?;
    Ok(EntryRuntime { source, target, demand })
}
#[allow(clippy::too_many_arguments)]
fn compare<P: Policy, M: Comparator>(
    pair: &PairSlot,
    shape: Shape,
    facts: EntryFacts<'_>,
    source: &SourceSession,
    at: DirectoryRequest<'_>,
    lane: &TargetInspection<'_, P, M>,
    _acquire: &impl Acquire,
) -> Result<Comparison, Arc<Error>> {
    if let Some(compared) = structural(shape) {
        return Ok(compared);
    }
    let relative = if pair.source.as_ref().is_some_and(|s| s.hints().origin() == FactOrigin::RootMetadata) {
        at.relative.to_owned()
    } else {
        at.relative.join(&pair.name)
    };
    #[cfg(test)]
    _acquire.before_compare(&source.root().join(&relative)).map_err(Arc::new)?;
    let compared = lane
        .comparator
        .compare(ComparisonInput {
            shape,
            facts,
            source_path: &source.root().join(&relative),
            target_path: &lane.logical_root.join(&relative),
        })
        .map_err(|cause| {
            // Built-in timestamp failures retain their original structured I/O cause.
            match cause.downcast::<Error>() {
                Ok(e) => Arc::new(*e),
                Err(cause) => Arc::new(refusal(&lane.logical_root.join(&relative), cause)),
            }
        })?;
    if compared == Comparison::NotAssessed {
        return Err(Arc::new(refusal(
            &lane.logical_root.join(&relative),
            "comparator returned NotAssessed for requested comparison".into(),
        )));
    }
    Ok(compared)
}

pub(crate) fn indexed_map<T: Sync, U: Send>(items: &[T], execution: Execution, map: impl Fn(&T) -> U + Sync + Send) -> Vec<U> {
    #[cfg(feature = "parallel")]
    if execution == Execution::Parallel && items.len() >= 64 {
        use rayon::prelude::*;
        return items.par_iter().with_min_len(8).map(map).collect();
    }
    let _ = execution;
    items.iter().map(map).collect()
}

#[derive(Debug)]
pub struct SharedRoots {
    source: SourceSession,
    source_metadata: Metadata,
    targets: Vec<(PathBuf, Metadata)>,
}
enum InspectionAt<'a> {
    RootGated(&'a Path),
    AlreadyAdmitted(&'a Path),
}
impl InspectionAt<'_> {
    fn dir(&self) -> &Path {
        match self {
            Self::RootGated(p) | Self::AlreadyAdmitted(p) => p,
        }
    }
}
impl SharedRoots {
    pub fn new<P: AsRef<Path>>(source: impl AsRef<Path>, targets: impl IntoIterator<Item = P>) -> Result<Self, SharedBindError> {
        let resources = diff_dirs_tree_observe::ObservationResources::default();
        Self::bind_using(
            source.as_ref(),
            targets.into_iter().map(|p| p.as_ref().to_owned()).collect(),
            &Physical(&resources),
        )
    }
    fn bind_using(source: &Path, targets: Vec<PathBuf>, acquire: &impl Acquire) -> Result<Self, SharedBindError> {
        if targets.is_empty() {
            return Err(SharedBindError::NoTargets);
        }
        let bind = |path: &Path| {
            let m = acquire.root_metadata(path)?;
            if !m.is_dir() {
                return Err(Error::NotDirectory { path: path.to_owned() });
            }
            Ok(m)
        };
        let source_metadata = bind(source).map_err(SharedBindError::Source)?;
        let targets = targets
            .into_iter()
            .enumerate()
            .map(|(target, path)| bind(&path).map(|m| (path, m)).map_err(|error| SharedBindError::Target { target, error }))
            .collect::<Result<_, _>>()?;
        Ok(Self {
            source: SourceSession::new(source),
            source_metadata,
            targets,
        })
    }
    pub fn source_root(&self) -> &Path {
        self.source.root()
    }
    pub fn target_roots(&self) -> impl ExactSizeIterator<Item = &Path> {
        self.targets.iter().map(|(p, _)| p.as_path())
    }
    pub fn inspect_visitor<V: Visitor>(
        &self,
        dir: &Path,
        targets: &[TargetVisitor<'_, V>],
        execution: Execution,
    ) -> Result<Vec<TargetComparison>, SharedInputError> {
        self.inspect_using(dir, targets, execution, &Physical(self.source.resources()))
    }
    pub fn inspect_with<V: Visitor, T>(
        &self,
        dir: &Path,
        targets: &[TargetVisitor<'_, V>],
        execution: Execution,
        lower: impl FnMut(usize, DirectoryPlan) -> T,
    ) -> Result<Vec<TargetComparison<T>>, SharedInputError> {
        self.inspect_lowered_using(InspectionAt::RootGated(dir), targets, execution, &Physical(self.source.resources()), lower)
    }
    pub fn inspect_admitted_visitor<V: Visitor>(
        &self,
        dir: &Path,
        targets: &[TargetVisitor<'_, V>],
        execution: Execution,
    ) -> Result<Vec<TargetComparison>, SharedInputError> {
        self.inspect_admitted_with(dir, targets, execution, |_, p| p.into_comparison())
    }
    pub fn inspect_admitted_with<V: Visitor, T>(
        &self,
        dir: &Path,
        targets: &[TargetVisitor<'_, V>],
        execution: Execution,
        lower: impl FnMut(usize, DirectoryPlan) -> T,
    ) -> Result<Vec<TargetComparison<T>>, SharedInputError> {
        self.inspect_lowered_using(
            InspectionAt::AlreadyAdmitted(dir),
            targets,
            execution,
            &Physical(self.source.resources()),
            lower,
        )
    }
    fn inspect_using<V: Visitor>(
        &self,
        dir: &Path,
        targets: &[TargetVisitor<'_, V>],
        execution: Execution,
        acquire: &impl Acquire,
    ) -> Result<Vec<TargetComparison>, SharedInputError> {
        self.inspect_lowered_using(InspectionAt::RootGated(dir), targets, execution, acquire, |_, p| p.into_comparison())
    }
    fn inspect_lowered_using<V: Visitor, T>(
        &self,
        at: InspectionAt<'_>,
        targets: &[TargetVisitor<'_, V>],
        execution: Execution,
        acquire: &impl Acquire,
        mut lower: impl FnMut(usize, DirectoryPlan) -> T,
    ) -> Result<Vec<TargetComparison<T>>, SharedInputError> {
        for t in targets {
            if t.target >= self.targets.len() {
                return Err(SharedInputError::UnknownTarget(t.target));
            }
        }
        let dir = at.dir();
        let policies: Vec<_> = targets.iter().map(|t| FullVisitor(t.visitor.borrowed())).collect();
        let paths: Vec<_> = targets.iter().map(|t| self.targets[t.target].0.join(dir)).collect();
        let lanes: Vec<_> = targets
            .iter()
            .zip(&policies)
            .zip(&paths)
            .map(|((t, p), path)| TargetInspection {
                target: t.target,
                logical_root: &self.targets[t.target].0,
                physical_directory: Some(path.as_path()),
                context: &(),
                policy: p,
                comparator: &StatKey,
            })
            .collect();
        let metadata: Vec<_> = targets.iter().map(|t| self.targets[t.target].1.clone()).collect();
        let request = DirectoryRequest {
            relative: dir,
            source_directory: Some(&self.source.root().join(dir)),
            purpose: Purpose::Compare,
            admission: if V::ENABLED && matches!(at, InspectionAt::RootGated(_)) {
                DirectoryAdmission::RootGated
            } else {
                DirectoryAdmission::AlreadyAdmitted
            },
        };
        Ok(inspect_group_using(
            &self.source,
            request,
            &lanes,
            &NoExecutionDemand,
            execution,
            acquire,
            Some((&self.source_metadata, &metadata)),
        )?
        .into_iter()
        .map(|outcome| TargetComparison {
            target: outcome.target,
            outcome: outcome.outcome.map(|prepared| lower(outcome.target, DirectoryPlan { prepared })),
        })
        .collect())
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn inspect_legacy_directory<V: Visitor>(
    source_root: &Path,
    target_root: &Path,
    dir: &Path,
    source: Option<&Path>,
    target: Option<&Path>,
    purpose: Purpose,
    visitor: &V,
    execution: Execution,
) -> Result<DirectoryPlan, Error> {
    let session = SourceSession::new(source_root);
    let policy = FullVisitor(visitor.borrowed());
    let lane = TargetInspection {
        target: 0,
        logical_root: target_root,
        physical_directory: target,
        context: &(),
        policy: &policy,
        comparator: &StatKey,
    };
    let request = DirectoryRequest {
        relative: dir,
        source_directory: source,
        purpose,
        admission: DirectoryAdmission::AlreadyAdmitted,
    };
    let outcome = inspect_group(&session, request, &[lane], &NoExecutionDemand, &mut InspectionEpoch::new(), execution)
        .map_err(|e| refusal(dir, Box::new(e)))?
        .pop()
        .expect("one lane")
        .outcome;
    outcome.map(|prepared| DirectoryPlan { prepared }).map_err(owned_error)
}
pub(crate) fn inspect_legacy_root<V: Visitor>(
    source_root: &Path,
    target_root: &Path,
    source: Option<Metadata>,
    target: Option<Metadata>,
    purpose: Purpose,
    visitor: &V,
) -> Result<DirectoryPlan, Error> {
    inspect_legacy_root_cells(
        source_root,
        target_root,
        source.map(|m| Arc::new(AcquisitionCell::bound(source_root.to_owned(), m))),
        target.map(|m| Arc::new(AcquisitionCell::bound(target_root.to_owned(), m))),
        purpose,
        visitor,
    )
}
pub(crate) fn inspect_legacy_root_paths<V: Visitor>(source: &Path, target: &Path, purpose: Purpose, visitor: &V) -> Result<DirectoryPlan, Error> {
    let resources = ObservationResources::default();
    let acquire = Physical(&resources);
    let s = Arc::new(AcquisitionCell::bound(source.to_owned(), diff_dirs_tree_observe::symlink_metadata(source)?));
    // A source readlink error precedes even target structural metadata failure.
    s.full(true, true, &acquire).map_err(owned_error)?;
    let t = match diff_dirs_tree_observe::symlink_metadata(target) {
        Ok(m) => Some(Arc::new(AcquisitionCell::bound(target.to_owned(), m))),
        Err(e) if e.io_error().kind() == std::io::ErrorKind::NotFound => None,
        Err(e) => return Err(Error::root_observation(e)),
    };
    inspect_legacy_root_cells(source, target, Some(s), t, purpose, visitor)
}
fn inspect_legacy_root_cells<V: Visitor>(
    source_root: &Path,
    target_root: &Path,
    source: Option<Arc<AcquisitionCell>>,
    target: Option<Arc<AcquisitionCell>>,
    purpose: Purpose,
    visitor: &V,
) -> Result<DirectoryPlan, Error> {
    let session = SourceSession::new(source_root);
    let policy = FullVisitor(visitor.borrowed());
    let lane = TargetInspection {
        target: 0,
        logical_root: target_root,
        physical_directory: target.as_ref().map(|_| target_root),
        context: &(),
        policy: &policy,
        comparator: &StatKey,
    };
    let pair = PairSlot {
        name: source_root.file_name().unwrap_or(source_root.as_os_str()).to_owned(),
        source,
        target,
    };
    assess(
        &session,
        DirectoryRequest {
            relative: Path::new(""),
            source_directory: None,
            purpose,
            admission: DirectoryAdmission::AlreadyAdmitted,
        },
        &lane,
        &(),
        vec![pair],
        true,
        &NoExecutionDemand,
        Execution::Sequential,
        &Physical(session.resources()),
    )
    .map(|prepared| DirectoryPlan { prepared })
    .map_err(owned_error)
}
pub(crate) fn owned_error(error: Arc<Error>) -> Error {
    Arc::try_unwrap(error).unwrap_or_else(|shared| Error::Refused {
        path: shared.path().to_owned(),
        source: Box::new(shared),
    })
}

#[cfg(test)]
#[path = "shared_tests.rs"]
mod tests;
