//! One complete immediate acquisition epoch, with independent target policy.

use std::fmt;

use super::*;
use crate::Execution;

/// Immediate comparison contributions, not recursive or completed work.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DirectoryCoverage {
    pub excluded: u64,
    pub scoped: bool,
    pub planned: u64,
    pub retained: u64,
}

/// Reduced read-only assessment. Full facts and acquisition handles have been
/// released; children are only finalized admitted **paired** directory basenames.
/// One-sided directories and collisions remain opaque comparison boundaries.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DirectoryComparison {
    pub(super) diff: DirDiff,
    pub(super) coverage: DirectoryCoverage,
    pub(super) children: Vec<OsString>,
    pub(super) root: bool,
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
    /// True for a structural root singleton, not an interior listing.
    /// SharedRoots returns this form when root policy stops descent; ordinary
    /// root-plan lowering can also describe initialization or root replacement.
    pub fn is_root(&self) -> bool {
        self.root
    }
    /// Consume the diff, coverage, paired children and structural-root flag.
    pub fn into_parts(self) -> (DirDiff, DirectoryCoverage, Vec<OsString>, bool) {
        (self.diff, self.coverage, self.children, self.root)
    }
}

/// A selected target binding and its frozen, full-fact visitor. Requests must be
/// distinct and in ascending binding-index order. IDs and traversal membership
/// belong to the caller; an index always refers to the original binding order.
#[derive(Clone, Copy, Debug)]
pub struct TargetVisitor<'a, V> {
    pub target: usize,
    pub visitor: &'a V,
}

/// One selected target's entire immediate outcome, in request/binding order.
/// Shared source failures retain the **same original** error and I/O cause in an
/// Arc, rather than reobserving or reconstructing an OS error for each target.
/// The default payload is reduced comparison; [`SharedRoots::inspect_with`]
/// permits a caller-owned projection of the complete validated plan instead.
#[derive(Debug)]
pub struct TargetComparison<T = DirectoryComparison> {
    pub target: usize,
    pub outcome: Result<T, Arc<Error>>,
}

struct Lowering<T, F> {
    outcomes: Vec<Option<Result<T, Arc<Error>>>>,
    lower: F,
}

impl<T, F: FnMut(usize, DirectoryPlan) -> T> Lowering<T, F> {
    fn successful(&mut self, position: usize, target: usize, plan: DirectoryPlan) {
        self.outcomes[position] = Some(Ok((self.lower)(target, plan)));
    }
}

/// Construction fails before any target assessment or interior listing.
#[derive(Debug)]
pub enum SharedBindError {
    NoTargets,
    Source(Error),
    Target { target: usize, error: Error },
}

impl fmt::Display for SharedBindError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoTargets => f.write_str("shared comparison requires at least one target"),
            Self::Source(error) => write!(f, "bind source: {error}"),
            Self::Target { target, error } => write!(f, "bind target {target}: {error}"),
        }
    }
}

impl std::error::Error for SharedBindError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::NoTargets => None,
            Self::Source(error) | Self::Target { error, .. } => Some(error),
        }
    }
}

/// Invalid requests perform no observations or callbacks. Physical/policy errors
/// from valid requests instead appear independently in [`TargetComparison`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SharedInputError {
    InvalidDirectory(PathBuf),
    UnknownTarget(usize),
    UnorderedTargets,
}

impl fmt::Display for SharedInputError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDirectory(dir) => write!(f, "directory must be a normal root-relative path: {dir:?}"),
            Self::UnknownTarget(target) => write!(f, "unknown target binding index: {target}"),
            Self::UnorderedTargets => f.write_str("target requests must be distinct and in binding order"),
        }
    }
}

impl std::error::Error for SharedInputError {}

#[derive(Debug)]
struct BoundTarget {
    path: PathBuf,
    metadata: Metadata,
}

enum InspectionAt<'a> {
    RootGated(&'a Path),
    AlreadyAdmitted(&'a Path),
}

impl InspectionAt<'_> {
    fn dir(&self) -> &Path {
        match self {
            Self::RootGated(dir) | Self::AlreadyAdmitted(dir) => dir,
        }
    }
}

/// One real source directory root and ordered real target directory roots.
/// Construction observes the source root once, then targets in binding order.
/// Equal/nested read-only roots are allowed and never silently coalesced.
///
/// This is an immediate inspector, **not** a walker, resumable state, snapshot or
/// write binding. Callers admit routing ancestors, own stable trees/confinement,
/// and maintain target identities and remaining traversal obligations themselves.
#[derive(Debug)]
pub struct SharedRoots {
    source: PathBuf,
    source_facts: Arc<Facts>,
    targets: Vec<BoundTarget>,
}

impl SharedRoots {
    pub fn new<P: AsRef<Path>>(source: impl AsRef<Path>, targets: impl IntoIterator<Item = P>) -> Result<Self, SharedBindError> {
        Self::bind_using(
            source.as_ref(),
            targets.into_iter().map(|path| path.as_ref().to_path_buf()).collect(),
            &Physical,
        )
    }

    fn bind_using(source: &Path, targets: Vec<PathBuf>, acquire: &impl Acquire) -> Result<Self, SharedBindError> {
        if targets.is_empty() {
            return Err(SharedBindError::NoTargets);
        }
        let bind = |path: &Path| {
            let metadata = acquire.root_metadata(path)?;
            if !metadata.is_dir() {
                return Err(Error::NotDirectory { path: path.to_path_buf() });
            }
            Ok(metadata)
        };
        let source_metadata = bind(source).map_err(SharedBindError::Source)?;
        let targets = targets
            .into_iter()
            .enumerate()
            .map(|(target, path)| {
                let metadata = bind(&path).map_err(|error| SharedBindError::Target { target, error })?;
                Ok(BoundTarget { path, metadata })
            })
            .collect::<Result<_, _>>()?;
        Ok(Self {
            source: source.to_path_buf(),
            source_facts: Arc::new(Facts {
                metadata: source_metadata,
                link: None,
            }),
            targets,
        })
    }

    pub fn source_root(&self) -> &Path {
        &self.source
    }

    pub fn target_roots(&self) -> impl ExactSizeIterator<Item = &Path> {
        self.targets.iter().map(|target| target.path.as_path())
    }

    /// Assess one immediate directory for the selected targets. The source is
    /// listed once; each source entry admitted by any otherwise-ready lane is
    /// acquired once, including required literal link text. Every lane still sees
    /// its own full union, full facts, late policy and ordinary action validation.
    ///
    /// An empty selection does no I/O. At the empty relative path, root policy
    /// uses binding facts; preserved/refused roots do not demand interior listing.
    /// Other calls require caller-admitted real-directory routes on both sides.
    ///
    /// Errors obey source-listing, target-listing, early, native-name/source-first
    /// observation, late and validation precedence **per target**. No partial
    /// successful lane result is published. Every return joins any metadata work;
    /// no jobs, child facts or listing handles remain between calls (mandatory
    /// binding-root facts do remain). Parallel
    /// execution only distributes sufficiently wide metadata phases on the current
    /// Rayon pool; it creates no pool and adds no J/K or byte-capacity promise.
    ///
    /// ```no_run
    /// use diff_dirs_tree_inspect::{Execution, NoVisitor, SharedRoots, TargetVisitor};
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let roots = SharedRoots::new("source", ["target-a", "target-b"])?;
    /// let visitor = NoVisitor;
    /// let targets = [TargetVisitor { target: 0, visitor: &visitor },
    ///                TargetVisitor { target: 1, visitor: &visitor }];
    /// for lane in roots.inspect_visitor(std::path::Path::new(""), &targets, Execution::Sequential)? {
    ///     let compared = lane.outcome?;
    ///     println!("{}: {:?}", lane.target, compared.diff());
    /// }
    /// # Ok(()) }
    /// ```
    pub fn inspect_visitor<V: Visitor>(
        &self,
        dir: &Path,
        targets: &[TargetVisitor<'_, V>],
        execution: Execution,
    ) -> Result<Vec<TargetComparison>, SharedInputError> {
        self.inspect_using(dir, targets, execution, &Physical)
    }

    /// Inspect the same source-once epoch, lowering each successful **validated**
    /// full plan into a caller-owned value before it is discarded. This retains
    /// action/baseline choices, exclusion and descent information for consumers
    /// that need more than a DirDiff, without introducing another inspector.
    /// Failed lanes never invoke `lower`; a consumer may return its own Result
    /// as T to distinguish lowering refusal from acquisition/policy failure.
    ///
    /// Lowering runs synchronously outside metadata jobs. Its invocation order is
    /// unspecified (a preserved root can precede a peer's interior assessment).
    /// It must only project supplied facts/decisions, not perform I/O or effects.
    /// It is not a traversal acceptance or effect acknowledgment notification;
    /// another invocation/retry can inspect and lower the same directory again.
    ///
    /// Return owned semantic decisions to release full facts. Deliberately
    /// returning a DirectoryPlan or cloned Metadata instead retains those facts
    /// at the caller's cost. No listing handles enter plans or returned values.
    pub fn inspect_with<V: Visitor, T>(
        &self,
        dir: &Path,
        targets: &[TargetVisitor<'_, V>],
        execution: Execution,
        lower: impl FnMut(usize, DirectoryPlan) -> T,
    ) -> Result<Vec<TargetComparison<T>>, SharedInputError> {
        self.inspect_lowered_using(InspectionAt::RootGated(dir), targets, execution, &Physical, lower)
    }

    /// Reduced inspection of a directory whose admission the caller has already
    /// accepted. Unlike [`Self::inspect_visitor`], an empty relative path does
    /// **not** replay structural-root policy. Real-root binding validation still
    /// applies; ordinary directory-union early/late callbacks run as usual.
    /// See [`Self::inspect_admitted_with`] for the full-plan lowering variant and
    /// the caller's responsibility for admission and routing.
    pub fn inspect_admitted_visitor<V: Visitor>(
        &self,
        dir: &Path,
        targets: &[TargetVisitor<'_, V>],
        execution: Execution,
    ) -> Result<Vec<TargetComparison>, SharedInputError> {
        self.inspect_admitted_with(dir, targets, execution, |_, plan| plan.into_comparison())
    }

    /// Full-plan lowering at an **already admitted** directory. Structural root
    /// hooks are skipped even at the empty relative path, so refreshing an
    /// unstarted directory batch need not replay an accepted root decision.
    /// Directory hooks (`is_root() == false`) still execute for every ready lane.
    ///
    /// The caller must have accepted the root/ancestor descent for these exact
    /// lanes under compatible frozen policy and must validate real-directory
    /// routes before inspection when rebinding resumed state. This method neither
    /// accepts saved decisions nor validates past policy/execution itself. The
    /// same source-once acquisition, full-plan validation, errors and pure-lowering
    /// contract as [`Self::inspect_with`] apply. It grants no mutation authority.
    pub fn inspect_admitted_with<V: Visitor, T>(
        &self,
        dir: &Path,
        targets: &[TargetVisitor<'_, V>],
        execution: Execution,
        lower: impl FnMut(usize, DirectoryPlan) -> T,
    ) -> Result<Vec<TargetComparison<T>>, SharedInputError> {
        self.inspect_lowered_using(InspectionAt::AlreadyAdmitted(dir), targets, execution, &Physical, lower)
    }

    fn inspect_using<V: Visitor>(
        &self,
        dir: &Path,
        targets: &[TargetVisitor<'_, V>],
        execution: Execution,
        acquire: &impl Acquire,
    ) -> Result<Vec<TargetComparison>, SharedInputError> {
        self.inspect_lowered_using(InspectionAt::RootGated(dir), targets, execution, acquire, |_, plan| plan.into_comparison())
    }

    fn inspect_lowered_using<V: Visitor, T>(
        &self,
        at: InspectionAt<'_>,
        targets: &[TargetVisitor<'_, V>],
        execution: Execution,
        acquire: &impl Acquire,
        lower: impl FnMut(usize, DirectoryPlan) -> T,
    ) -> Result<Vec<TargetComparison<T>>, SharedInputError> {
        let dir = at.dir();
        self.validate_input(dir, targets)?;
        let mut output = Lowering {
            outcomes: targets.iter().map(|_| None).collect(),
            lower,
        };
        let mut ready = Vec::new();
        for (position, request) in targets.iter().enumerate() {
            let target = &self.targets[request.target];
            if V::ENABLED && matches!(at, InspectionAt::RootGated(_)) && dir.as_os_str().is_empty() {
                let root = finish_root(
                    &self.source,
                    &target.path,
                    SourceFacts::Shared(Arc::clone(&self.source_facts)),
                    Some(Facts {
                        metadata: target.metadata.clone(),
                        link: None,
                    }),
                    Purpose::Compare,
                    request.visitor,
                );
                match root {
                    Err(error) => {
                        output.outcomes[position] = Some(Err(Arc::new(error)));
                        continue;
                    }
                    Ok(plan) if !plan.operations.entries[0].descends() => {
                        output.successful(position, request.target, plan);
                        continue;
                    }
                    Ok(_) => {}
                }
            }
            ready.push(position);
        }
        if !ready.is_empty() {
            self.inspect_ready(dir, targets, ready, execution, acquire, &mut output);
        }
        Ok(targets
            .iter()
            .zip(output.outcomes)
            .map(|(request, outcome)| TargetComparison {
                target: request.target,
                outcome: outcome.expect("every requested target was assessed"),
            })
            .collect())
    }

    fn validate_input<V>(&self, dir: &Path, targets: &[TargetVisitor<'_, V>]) -> Result<(), SharedInputError> {
        if dir.components().any(|component| !matches!(component, std::path::Component::Normal(_))) {
            return Err(SharedInputError::InvalidDirectory(dir.to_path_buf()));
        }
        for request in targets {
            if request.target >= self.targets.len() {
                return Err(SharedInputError::UnknownTarget(request.target));
            }
        }
        if targets.windows(2).any(|pair| pair[0].target >= pair[1].target) {
            return Err(SharedInputError::UnorderedTargets);
        }
        Ok(())
    }

    fn inspect_ready<V: Visitor, T>(
        &self,
        dir: &Path,
        targets: &[TargetVisitor<'_, V>],
        ready: Vec<usize>,
        execution: Execution,
        acquire: &impl Acquire,
        output: &mut Lowering<T, impl FnMut(usize, DirectoryPlan) -> T>,
    ) {
        let source = match listed(&self.source.join(dir), acquire) {
            Ok(source) => source,
            Err(error) => {
                let error = Arc::new(error);
                for position in ready {
                    output.outcomes[position] = Some(Err(Arc::clone(&error)));
                }
                return;
            }
        };
        let mut demanded = vec![false; source.len()];
        let mut prepared = Vec::with_capacity(ready.len());
        for position in ready {
            let request = &targets[position];
            let target = &self.targets[request.target].path;
            let prepare = || {
                let target_entries = listed(&target.join(dir), acquire)?;
                let (names, pairs) = union(dir, &source, target_entries);
                admit_names(names, pairs, target, request.visitor)
            };
            match prepare() {
                Ok(admitted) => {
                    for (_, pair) in &admitted.entries {
                        if let Some(index) = pair.source {
                            demanded[index] = true;
                        }
                    }
                    prepared.push((position, admitted));
                }
                Err(error) => output.outcomes[position] = Some(Err(Arc::new(error))),
            }
        }
        // Consume every source handle before late assessment. Excluded handles
        // drop without metadata; each demanded handle yields one frozen outcome.
        let source: Vec<_> = indexed_map(source.into_iter().zip(demanded).collect(), execution, |((_, entry), demanded)| {
            demanded.then(|| {
                acquired_facts(entry, true, acquire)
                    .map(|facts| {
                        let facts = Arc::new(facts);
                        #[cfg(test)]
                        acquire.source_acquired(&facts);
                        facts
                    })
                    .map_err(Arc::new)
            })
        });
        for (position, admitted) in prepared {
            let request = &targets[position];
            let target = &self.targets[request.target].path;
            let Admitted { entries, coverage } = admitted;
            let entries = indexed_map(entries, execution, |(name, pair)| {
                let s = pair
                    .source
                    .map(|index| source[index].as_ref().expect("admitted source was demanded").clone().map(SourceFacts::Shared))
                    .transpose()?;
                let t = pair
                    .target
                    .map(|entry| acquired_facts(entry, name.presence == Presence::Both, acquire))
                    .transpose()
                    .map_err(Arc::new)?;
                operation(name.name, name.relative, s, t, &self.source, target).map_err(Arc::new)
            })
            .into_iter()
            .collect::<Result<Vec<_>, _>>();
            let plan = entries.and_then(|entries| {
                finish_admitted(
                    Operations {
                        source_root: self.source.clone(),
                        target_root: target.clone(),
                        dir: dir.to_path_buf(),
                        root: false,
                        purpose: Purpose::Compare,
                        entries,
                    },
                    coverage,
                    request.visitor,
                )
                .map_err(Arc::new)
            });
            match plan {
                Ok(plan) => output.successful(position, request.target, plan),
                Err(error) => output.outcomes[position] = Some(Err(error)),
            }
        }
    }
}

struct PairSlot {
    source: Option<usize>,
    target: Option<observe::Entry>,
}

fn union(dir: &Path, source: &[(OsString, observe::Entry)], target: Vec<(OsString, observe::Entry)>) -> (Names, Vec<PairSlot>) {
    let mut names = Names {
        dir: dir.to_path_buf(),
        root: false,
        entries: Vec::with_capacity(source.len() + target.len()),
    };
    let mut pairs = Vec::with_capacity(source.len() + target.len());
    let mut source = source.iter().enumerate().peekable();
    let mut target = target.into_iter().peekable();
    while source.peek().is_some() || target.peek().is_some() {
        let order = match (source.peek(), target.peek()) {
            (Some((_, (s, _))), Some((t, _))) => s.as_encoded_bytes().cmp(t.as_encoded_bytes()),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            _ => unreachable!(),
        };
        let s = (order != std::cmp::Ordering::Greater).then(|| source.next().expect("source peeked"));
        let t = (order != std::cmp::Ordering::Less).then(|| target.next().expect("target peeked"));
        let name = s
            .as_ref()
            .map_or_else(|| t.as_ref().expect("present target").0.clone(), |(_, (name, _))| name.clone());
        names.entries.push(Name {
            relative: dir.join(&name),
            name,
            presence: presence(s.is_some(), t.is_some()),
            source: s.as_ref().map(|(_, (_, entry))| entry.hints()),
            target: t.as_ref().map(|(_, entry)| entry.hints()),
            admission: Admission::Observe,
        });
        pairs.push(PairSlot {
            source: s.map(|(index, _)| index),
            target: t.map(|(_, entry)| entry),
        });
    }
    (names, pairs)
}

// Private acquisition substitution permits counted/faulted tests of the real
// pipeline without opening a public invented-facts or plan-construction API.
trait Acquire: Sync {
    fn list(&self, path: &Path) -> Result<Vec<observe::Entry>, Error>;
    fn metadata(&self, entry: observe::Entry) -> Result<Metadata, Error>;
    fn root_metadata(&self, path: &Path) -> Result<Metadata, Error>;
    fn link(&self, path: &Path) -> Result<PathBuf, Error>;
    #[cfg(test)]
    fn source_acquired(&self, _: &Arc<Facts>) {}
}

struct Physical;
impl Acquire for Physical {
    fn list(&self, path: &Path) -> Result<Vec<observe::Entry>, Error> {
        observe::read_directory(path).map_err(Error::from)
    }
    fn metadata(&self, entry: observe::Entry) -> Result<Metadata, Error> {
        entry.metadata().map_err(Error::from)
    }
    fn root_metadata(&self, path: &Path) -> Result<Metadata, Error> {
        observe::symlink_metadata(path).map_err(Error::root_observation)
    }
    fn link(&self, path: &Path) -> Result<PathBuf, Error> {
        observe::read_link(path).map_err(Error::from)
    }
}

fn listed(path: &Path, acquire: &impl Acquire) -> Result<Vec<(OsString, observe::Entry)>, Error> {
    let mut entries: Vec<_> = acquire.list(path)?.into_iter().map(|entry| (entry.file_name(), entry)).collect();
    entries.sort_unstable_by(|a, b| a.0.as_encoded_bytes().cmp(b.0.as_encoded_bytes()));
    if entries.windows(2).any(|pair| pair[0].0 == pair[1].0) {
        return Err(Error::io(
            path.to_path_buf(),
            "read directory entry",
            std::io::Error::new(std::io::ErrorKind::InvalidData, "duplicate name in physical listing"),
        ));
    }
    Ok(entries)
}

fn acquired_facts(entry: observe::Entry, read_link: bool, acquire: &impl Acquire) -> Result<Facts, Error> {
    let path = entry.path();
    let metadata = acquire.metadata(entry)?;
    facts_with_metadata_using(&path, metadata, read_link, |path| acquire.link(path))
}

fn indexed_map<T: Send, U: Send>(items: Vec<T>, execution: Execution, map: impl Fn(T) -> U + Sync + Send) -> Vec<U> {
    #[cfg(feature = "parallel")]
    if execution == Execution::Parallel && items.len() >= 64 {
        use rayon::prelude::*;
        return items.into_par_iter().with_min_len(8).map(map).collect();
    }
    let _ = execution;
    items.into_iter().map(map).collect()
}

#[cfg(test)]
#[path = "shared_tests.rs"]
mod tests;
