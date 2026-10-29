//! Inspect-owned direct declarations and complete target-only censuses. Pure
//! selectors lend authority; they cannot construct facts or executable batches.

use std::collections::HashMap;
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::sync::{Arc, LazyLock};

use crate::acquire::{Acquire, AcquisitionCell, EntryRuntime, Physical};
use crate::prepared::LiveBinding;
use crate::selection::{native_name, normal_relative};
use crate::visitor::refusal;
use crate::*;

#[derive(Clone, Copy, Debug)]
pub struct DeclaredName<'a> {
    pub name: &'a OsStr,
    pub desired: Option<DesiredInput>,
}
#[derive(Clone, Copy, Debug)]
pub enum DeclaredNames<'a> {
    Exact(&'a [DeclaredName<'a>]),
    /// Every actual immediate target child, never a source query. A missing or
    /// non-directory containing route fails rather than becoming an empty census.
    TargetCensus,
}
#[derive(Clone, Copy, Debug)]
pub struct DeclaredRequest<'a> {
    pub relative: &'a Path,
    pub purpose: Purpose,
}
pub struct DeclaredInspection<'a, L: DeclaredSelection> {
    pub target: usize,
    pub logical_root: &'a Path,
    pub physical_directory: Option<&'a Path>,
    pub desired: ManifestId,
    pub owned: ManifestId,
    pub context: &'a L::Context,
    pub names: DeclaredNames<'a>,
    pub selection: &'a L,
}
#[derive(Clone, Copy, Debug)]
pub struct DeclaredEntryCandidate<'a> {
    pub info: BatchInfo<'a>,
    pub domain: SelectionDomain,
    pub coordinate: Coordinate<'a>,
    pub desired: Option<DesiredInput>,
    pub shape: Shape,
}
#[derive(Clone, Copy, Debug)]
pub struct DeclaredRouteCandidate<'a> {
    pub info: BatchInfo<'a>,
    pub side: Side,
    pub relative: &'a Path,
    pub kind: Kind,
}
#[derive(Clone, Copy, Debug)]
pub enum DeclaredCandidate<'a> {
    Route(DeclaredRouteCandidate<'a>),
    Entry(DeclaredEntryCandidate<'a>),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeclaredChoice<C> {
    pub action: Action,
    pub install: Option<InstallAuthority>,
    pub removal: Option<RemovalAuthority>,
    pub interior: Option<Interior<C>>,
}
pub trait DeclaredSelection: Sync {
    type Context: Clone + Eq + std::fmt::Debug + Send + Sync;
    fn identity(&self) -> &SemanticId;
    fn context_schema(&self) -> &SemanticId {
        self.identity()
    }
    fn validate_context(&self, at: ContextSite<'_>, context: &Self::Context) -> Result<(), CheckError>;
    fn demand(&self, context: &Self::Context, at: DeclaredCandidate<'_>) -> SideDemands;
    fn check(&self, context: &Self::Context, at: DeclaredCandidate<'_>, facts: EntryFacts<'_>) -> Result<(), CheckError>;
    fn select(&self, context: &Self::Context, at: DeclaredEntryCandidate<'_>) -> Result<DeclaredChoice<Self::Context>, CheckError>;
}
static OMITTED: LazyLock<SemanticId> = LazyLock::new(|| SemanticId::new("diff-dirs.listed.not-assessed", 1, Vec::new()));

/// Finite route/lookup, guard/expectation, selection and execution-input stages.
/// All lanes return before any caller can dispatch an effect. Source positive
/// cells and original causes share within the group even with retention disabled.
pub fn inspect_declared_group<L: DeclaredSelection, E: ExecutionDemand>(
    source: &SourceSession,
    at: DeclaredRequest<'_>,
    lanes: &[DeclaredInspection<'_, L>],
    demand: &E,
    epoch: &InspectionEpoch,
    execution: Execution,
) -> Result<Vec<TargetComparison<PreparedDirectory<L::Context>>>, SharedInputError> {
    inspect_using(source, at, lanes, demand, epoch, execution, &Physical(source.resources()))
}

struct Slot {
    name: OsString,
    desired: Option<DesiredInput>,
    source: Option<Arc<AcquisitionCell>>,
    target: Option<Arc<AcquisitionCell>>,
    shape: Shape,
}
impl Slot {
    fn candidate<'a>(&'a self, info: BatchInfo<'a>, domain: SelectionDomain) -> DeclaredEntryCandidate<'a> {
        DeclaredEntryCandidate {
            info,
            domain,
            coordinate: Coordinate {
                parent: info.relative,
                name: &self.name,
                structural_root: false,
            },
            desired: self.desired,
            shape: self.shape,
        }
    }
    fn acquire(&self, demand: SideDemands, acquire: &impl Acquire) -> Result<EntryRuntime, Arc<Error>> {
        if self.desired.is_none() && demand.source != Demand::NONE {
            return Err(Arc::new(refusal(Path::new(&self.name), "source demand for NotRequested declared input".into())));
        }
        Ok(EntryRuntime {
            source: self.source.as_ref().map(|s| s.acquire(demand.source, acquire)).transpose()?,
            target: self.target.as_ref().map(|s| s.acquire(demand.target, acquire)).transpose()?,
            demand,
        })
    }
}
type Lookup = Result<Option<Arc<AcquisitionCell>>, Arc<Error>>;

#[allow(clippy::too_many_arguments)]
fn inspect_using<L: DeclaredSelection, E: ExecutionDemand>(
    source: &SourceSession,
    at: DeclaredRequest<'_>,
    lanes: &[DeclaredInspection<'_, L>],
    demand: &E,
    epoch: &InspectionEpoch,
    execution: Execution,
    acquire: &impl Acquire,
) -> Result<Vec<TargetComparison<PreparedDirectory<L::Context>>>, SharedInputError> {
    if !normal_relative(at.relative) {
        return Err(SharedInputError::InvalidDirectory(at.relative.to_owned()));
    }
    if lanes.windows(2).any(|w| w[0].target >= w[1].target) {
        return Err(SharedInputError::UnorderedTargets);
    }
    for lane in lanes {
        if !manifest_valid(lane.desired) || !manifest_valid(lane.owned) {
            return Err(SharedInputError::InvalidDeclared("unsupported manifest version or platform"));
        }
        if let DeclaredNames::Exact(names) = lane.names
            && (names.iter().any(|n| !native_name(n.name)) || names.windows(2).any(|w| w[0].name.as_encoded_bytes() >= w[1].name.as_encoded_bytes()))
        {
            return Err(SharedInputError::InvalidDeclared("declared names must be ordered distinct native basenames"));
        }
    }
    // Lazy checkout: an O-only/census group does not even enter the source cache.
    let needs_source = lanes
        .iter()
        .any(|l| matches!(l.names, DeclaredNames::Exact(n) if n.iter().any(|n| n.desired.is_some())));
    let lease = needs_source.then(|| source.checkout(at.relative, SourceWork::Declared));
    let mut shared: HashMap<OsString, Lookup> = HashMap::new();
    let mut staged = Vec::with_capacity(lanes.len());
    for lane in lanes {
        let info = BatchInfo {
            source_root: source.root(),
            target_root: lane.logical_root,
            relative: at.relative,
            structural_root: false,
            purpose: at.purpose,
        };
        let error = |cause| Arc::new(refusal(&lane.logical_root.join(at.relative), cause));
        let slots = (|| {
            lane.selection
                .validate_context(
                    ContextSite {
                        relative: at.relative,
                        structural_root: false,
                        role: None,
                    },
                    lane.context,
                )
                .map_err(error)?;
            let requested = matches!(lane.names, DeclaredNames::Exact(n) if n.iter().any(|n| n.desired.is_some()));
            if requested {
                guard_routes(source.root(), &source.root().join(at.relative), Side::Original, false, info, lane, epoch)?;
            }
            let target_present = if let Some(path) = lane.physical_directory {
                guard_routes(
                    lane.logical_root,
                    path,
                    Side::Target,
                    matches!(lane.names, DeclaredNames::Exact(_)),
                    info,
                    lane,
                    epoch,
                )?
            } else {
                if matches!(lane.names, DeclaredNames::TargetCensus) {
                    return Err(error("census requires a real target route".into()));
                }
                false
            };
            let target_parent = lane.physical_directory.filter(|_| target_present);
            let mut slots = Vec::new();
            match lane.names {
                DeclaredNames::Exact(names) => {
                    for name in names {
                        let s = if name.desired.is_some() {
                            shared
                                .entry(name.name.to_owned())
                                .or_insert_with(|| lease.as_ref().unwrap().lookup(&source.root().join(at.relative), name.name, acquire, epoch))
                                .clone()?
                        } else {
                            None
                        };
                        let sk = s.as_ref().map(|s| s.kind(acquire).map(|(k, _)| k)).transpose()?;
                        let t = target_parent
                            .map(|parent| lookup(&parent.join(name.name), acquire, epoch))
                            .transpose()?
                            .flatten();
                        let tk = t.as_ref().map(|s| s.kind(acquire).map(|(k, _)| k)).transpose()?;
                        slots.push(Slot {
                            name: name.name.to_owned(),
                            desired: name.desired,
                            source: s,
                            target: t,
                            shape: Shape {
                                source: if name.desired.is_some() {
                                    sk.map_or(SourceObservation::Absent, SourceObservation::Present)
                                } else {
                                    SourceObservation::NotRequested
                                },
                                target: tk,
                            },
                        });
                    }
                }
                DeclaredNames::TargetCensus => {
                    let parent = target_parent.expect("census route admitted");
                    let mut entries = acquire.list(parent).map_err(Arc::new)?;
                    entries.sort_unstable_by(|a, b| a.name().as_encoded_bytes().cmp(b.name().as_encoded_bytes()));
                    if entries.windows(2).any(|w| w[0].name() == w[1].name()) {
                        return Err(error("duplicate target census name".into()));
                    }
                    for entry in entries {
                        let name = entry.file_name();
                        let target = Arc::new(AcquisitionCell::listed(entry));
                        let kind = target.kind(acquire)?.0;
                        slots.push(Slot {
                            name,
                            desired: None,
                            source: None,
                            target: Some(target),
                            shape: Shape {
                                source: SourceObservation::NotRequested,
                                target: Some(kind),
                            },
                        });
                    }
                }
            }
            Ok(slots)
        })();
        staged.push(slots);
    }
    // No per-lane combined FULL acquisition: optional peer failures must remain
    // independently masked. Shared cells intern each individual fact outcome.
    let mut selected = Vec::with_capacity(lanes.len());
    for (lane, slots) in lanes.iter().zip(staged) {
        let info = BatchInfo {
            source_root: source.root(),
            target_root: lane.logical_root,
            relative: at.relative,
            structural_root: false,
            purpose: at.purpose,
        };
        let domain = SelectionDomain::Listed {
            desired: lane.desired,
            owned: lane.owned,
        };
        let error = |cause| Arc::new(refusal(&lane.logical_root.join(at.relative), cause));
        let result = slots.and_then(|slots| {
            let masks: Vec<_> = slots
                .iter()
                .map(|s| lane.selection.demand(lane.context, DeclaredCandidate::Entry(s.candidate(info, domain))))
                .collect();
            let facts = crate::visitor::shared::indexed_map(&slots.iter().zip(&masks).collect::<Vec<_>>(), execution, |(s, mask)| {
                s.acquire(mask.union(expectation_demand(s.desired)), acquire)
            })
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;
            let mut entries = Vec::with_capacity(slots.len());
            for ((slot, mask), facts) in slots.iter().zip(&masks).zip(&facts) {
                let candidate = slot.candidate(info, domain);
                let entry_error = |cause: CheckError| match cause.downcast::<Error>() {
                    Ok(error) => Arc::new(*error),
                    Err(cause) => Arc::new(refusal(&lane.logical_root.join(at.relative).join(&slot.name), cause)),
                };
                check_expectation(slot.desired, facts, &source.root().join(at.relative).join(&slot.name))?;
                lane.selection
                    .check(lane.context, DeclaredCandidate::Entry(candidate), facts.masked(*mask))
                    .map_err(entry_error)?;
                let choice = lane.selection.select(lane.context, candidate).map_err(entry_error)?;
                if let Some(interior) = &choice.interior {
                    lane.selection
                        .validate_context(
                            ContextSite {
                                relative: &at.relative.join(&slot.name),
                                structural_root: false,
                                role: Some(interior.role),
                            },
                            &interior.context,
                        )
                        .map_err(entry_error)?;
                }
                entries.push(EntryParts {
                    name: slot.name.clone(),
                    entry: EntrySelection {
                        source: slot.shape.source,
                        target: slot.shape.target,
                        comparison: Comparison::NotAssessed,
                        baseline: None,
                        desired: slot.desired,
                        action: choice.action,
                        install: choice.install,
                        removal: choice.removal,
                        interior: choice.interior,
                    },
                });
            }
            let selection = DirectorySelection::from_parts(SelectionParts {
                relative: at.relative.to_owned(),
                purpose: at.purpose,
                structural_root: false,
                domain,
                input_context: lane.context.clone(),
                policy: lane.selection.identity().clone(),
                context_schema: lane.selection.context_schema().clone(),
                comparator: OMITTED.clone(),
                scoped: true,
                root_excluded: false,
                excluded_names: Vec::new(),
                entries,
            })
            .map_err(|e| error(Box::new(e)))?;
            Ok((slots, facts, Arc::new(selection)))
        });
        selected.push(result);
    }
    Ok(lanes
        .iter()
        .zip(selected)
        .map(|(lane, result)| TargetComparison {
            target: lane.target,
            outcome: result.and_then(|(slots, mut facts, selected)| {
                for (i, slot) in slots.iter().enumerate() {
                    let extra = demand.for_entry(selected.entry(SlotId(i)));
                    facts[i] = slot.acquire(facts[i].demand.union(extra), acquire)?;
                    // Current lookup/listing observations may seed target routes. A
                    // retained source fact cannot certify a later call's route.
                    if let (Some(parent), Some(target)) = (lane.physical_directory, &facts[i].target)
                        && let Some(metadata) = &target.attributes
                    {
                        epoch.acquired_directory(&parent.join(&slot.name), metadata)?;
                    }
                }
                Ok(PreparedDirectory {
                    selected,
                    facts,
                    binding: LiveBinding {
                        source: source.root().to_owned(),
                        target: lane.logical_root.to_owned(),
                        generation: source.generation(),
                    },
                })
            }),
        })
        .collect())
}

pub(crate) fn manifest_valid(id: ManifestId) -> bool {
    id.version == 1 && id.platform == if cfg!(windows) { NativePlatform::Windows } else { NativePlatform::Unix }
}
fn lookup(path: &Path, acquire: &impl Acquire, epoch: &InspectionEpoch) -> Lookup {
    match acquire.path_metadata(path) {
        Ok(metadata) => {
            epoch.acquired_directory(path, &metadata)?;
            Ok(Some(Arc::new(AcquisitionCell::lookup(path.to_owned(), Arc::new(metadata)))))
        }
        Err(Error::Io { source, .. }) if source.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(Arc::new(error)),
    }
}

#[allow(clippy::too_many_arguments)]
fn guard_routes<L: DeclaredSelection>(
    root: &Path,
    path: &Path,
    side: Side,
    missing: bool,
    info: BatchInfo<'_>,
    lane: &DeclaredInspection<'_, L>,
    epoch: &InspectionEpoch,
) -> Result<bool, Arc<Error>> {
    let error = |cause| Arc::new(refusal(path, cause));
    // Logical and private routes differ during staging. A private empty target
    // has no observed route; Some always names a real bound-root descendant.
    let relative = path.strip_prefix(root).map_err(|_| error("declared route is outside logical root".into()))?;
    let mut prefix = root.to_owned();
    let mut rel = PathBuf::new();
    for component in std::iter::once(None).chain(relative.components().map(Some)) {
        if let Some(component) = component {
            prefix.push(component);
            rel.push(component);
        }
        let metadata = match epoch.directory(&prefix) {
            Ok(metadata) => metadata,
            Err(e) if missing && matches!(e.as_ref(), Error::Io { source, .. } if source.kind() == std::io::ErrorKind::NotFound) => return Ok(false),
            Err(e) => return Err(e),
        };
        let candidate = DeclaredCandidate::Route(DeclaredRouteCandidate {
            info,
            side,
            relative: &rel,
            kind: Kind::Directory,
        });
        let masks = lane.selection.demand(lane.context, candidate);
        let (own, other) = if side == Side::Original {
            (masks.source, masks.target)
        } else {
            (masks.target, masks.source)
        };
        if other != Demand::NONE || own.literal_link {
            return Err(error("route demand must name only its directory side attributes".into()));
        }
        let facts = SideFacts {
            kind: Kind::Directory,
            origin: FactOrigin::AcquiredMetadata,
            attributes: if own.attributes {
                Available::Known(metadata.as_ref())
            } else {
                Available::NotRequested
            },
            literal_link: Available::NotRequested,
        };
        lane.selection
            .check(
                lane.context,
                candidate,
                EntryFacts {
                    source: (side == Side::Original).then_some(facts),
                    target: (side == Side::Target).then_some(facts),
                },
            )
            .map_err(error)?;
    }
    Ok(true)
}
fn expectation_demand(desired: Option<DesiredInput>) -> SideDemands {
    SideDemands {
        source: Demand {
            attributes: matches!(
                desired,
                Some(DesiredInput::Leaf {
                    expected: LeafExpectation::Exact { executable: Some(_), .. },
                    ..
                })
            ),
            literal_link: false,
        },
        target: Demand::NONE,
    }
}
fn check_expectation(desired: Option<DesiredInput>, facts: &EntryRuntime, path: &Path) -> Result<(), Arc<Error>> {
    let Some(desired) = desired else {
        return Ok(());
    };
    let failure = |message: &'static str| Arc::new(refusal(path, message.into()));
    let Some(source) = &facts.source else {
        return Err(failure("declared source obligation is missing"));
    };
    let valid = match desired {
        DesiredInput::Directory { .. } => source.kind == Kind::Directory,
        DesiredInput::Leaf {
            expected: LeafExpectation::SupportedLeaf,
            ..
        } => source.kind.is_leaf(),
        DesiredInput::Leaf {
            expected: LeafExpectation::Exact { kind, executable },
            ..
        } => {
            if executable.is_some() && (kind != LeafKind::RegularFile || !cfg!(unix)) {
                return Err(failure("unsupported executable expectation"));
            }
            let valid = source.kind
                == match kind {
                    LeafKind::RegularFile => Kind::File,
                    LeafKind::Symlink => Kind::Symlink,
                };
            #[cfg(unix)]
            if valid && let Some(expected) = executable {
                use std::os::unix::fs::PermissionsExt;
                if (source.attributes.as_ref().unwrap().permissions().mode() & 0o111 != 0) != expected {
                    return Err(failure("declared source executable expectation mismatch"));
                }
            }
            valid
        }
    };
    if valid {
        Ok(())
    } else {
        Err(failure("declared source kind expectation mismatch"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{Fixture, file};
    fn id(value: u8) -> ManifestId {
        ManifestId {
            version: 1,
            platform: if cfg!(windows) { NativePlatform::Windows } else { NativePlatform::Unix },
            digest: [value; 32],
        }
    }
    struct Select {
        link: bool,
    }
    impl DeclaredSelection for Select {
        type Context = ();
        fn identity(&self) -> &SemanticId {
            &OMITTED
        }
        fn validate_context(&self, _: ContextSite<'_>, _: &()) -> Result<(), CheckError> {
            Ok(())
        }
        fn demand(&self, _: &(), at: DeclaredCandidate<'_>) -> SideDemands {
            let mut demand = SideDemands::default();
            if matches!(at, DeclaredCandidate::Entry(e) if e.desired.is_some()) {
                demand.source.literal_link = self.link;
            }
            demand
        }
        fn check(&self, _: &(), at: DeclaredCandidate<'_>, facts: EntryFacts<'_>) -> Result<(), CheckError> {
            if let DeclaredCandidate::Entry(entry) = at {
                if entry.desired.is_none() {
                    assert!(facts.source.is_none());
                }
                if !self.link {
                    assert!(facts.source.is_none_or(|s| s.literal_link == Available::NotRequested));
                }
            }
            Ok(())
        }
        fn select(&self, _: &(), at: DeclaredEntryCandidate<'_>) -> Result<DeclaredChoice<()>, CheckError> {
            let removal = (at.desired.is_none()).then_some(RemovalAuthority::OwnedLeaf { owned: id(2), leaf: LeafId(0) });
            Ok(DeclaredChoice {
                action: if at.desired.is_some() {
                    Action::InstallLeaf
                } else if at.shape.target.is_some() {
                    Action::RemoveLeaf
                } else {
                    Action::ConfirmAbsent
                },
                install: at.desired.map(|_| InstallAuthority {
                    intent: InstallIntent::Renew,
                    destination: DestinationGrant::AbsentOnly,
                }),
                removal,
                interior: None,
            })
        }
    }
    fn lane<'a>(target: usize, root: &'a Path, path: Option<&'a Path>, names: DeclaredNames<'a>, selector: &'a Select) -> DeclaredInspection<'a, Select> {
        DeclaredInspection {
            target,
            logical_root: root,
            physical_directory: path,
            desired: id(1),
            owned: id(2),
            context: &(),
            names,
            selection: selector,
        }
    }
    fn desired(expected: LeafExpectation) -> Option<DesiredInput> {
        Some(DesiredInput::Leaf { leaf: LeafId(0), expected })
    }
    #[test]
    fn missing_both_wrong_kind_and_executable_are_real_obligation_failures() {
        let f = Fixture::new();
        let source = SourceSession::new(&f.original);
        let selector = Select { link: false };
        let at = DeclaredRequest {
            relative: Path::new(""),
            purpose: Purpose::Copy,
        };
        for expected in [
            LeafExpectation::SupportedLeaf,
            LeafExpectation::Exact {
                kind: LeafKind::Symlink,
                executable: None,
            },
        ] {
            let names = [DeclaredName {
                name: OsStr::new("required"),
                desired: desired(expected),
            }];
            assert!(
                inspect_declared_group(
                    &source,
                    at,
                    &[lane(0, &f.target, Some(&f.target), DeclaredNames::Exact(&names), &selector)],
                    &NoExecutionDemand,
                    &InspectionEpoch::new(),
                    Execution::Sequential
                )
                .unwrap()[0]
                    .outcome
                    .is_err()
            );
        }
        file(&f.original.join("required"), b"real donor");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(f.original.join("required"), std::fs::Permissions::from_mode(0o644)).unwrap();
            let names = [DeclaredName {
                name: OsStr::new("required"),
                desired: desired(LeafExpectation::Exact {
                    kind: LeafKind::RegularFile,
                    executable: Some(true),
                }),
            }];
            assert!(
                inspect_declared_group(
                    &source,
                    at,
                    &[lane(0, &f.target, Some(&f.target), DeclaredNames::Exact(&names), &selector)],
                    &NoExecutionDemand,
                    &InspectionEpoch::new(),
                    Execution::Sequential
                )
                .unwrap()[0]
                    .outcome
                    .is_err()
            );
        }
        assert_eq!(source.diagnostics().declared.listings, 0);
    }
    #[test]
    fn o_only_and_census_do_not_admit_source_even_with_desired_peer() {
        let f = Fixture::new();
        file(&f.target.join("entry"), b"owned");
        let source = SourceSession::new(f.original.join("inaccessible-unrelated-root"));
        let selector = Select { link: false };
        let old = [DeclaredName {
            name: OsStr::new("entry"),
            desired: None,
        }];
        let new = [DeclaredName {
            name: OsStr::new("entry"),
            desired: desired(LeafExpectation::SupportedLeaf),
        }];
        let outcomes = inspect_declared_group(
            &source,
            DeclaredRequest {
                relative: Path::new(""),
                purpose: Purpose::Remove,
            },
            &[
                lane(0, &f.target, Some(&f.target), DeclaredNames::Exact(&old), &selector),
                lane(1, &f.target, Some(&f.target), DeclaredNames::Exact(&new), &selector),
                lane(2, &f.target, Some(&f.target), DeclaredNames::TargetCensus, &selector),
            ],
            &NoExecutionDemand,
            &InspectionEpoch::new(),
            Execution::Sequential,
        )
        .unwrap();
        assert!(outcomes[0].outcome.is_ok());
        assert!(outcomes[1].outcome.is_err());
        assert!(outcomes[2].outcome.is_ok());
        assert_eq!(source.diagnostics().declared.metadata_acquisitions, 0);
        assert_eq!(source.diagnostics().declared.listings, 0);
    }
    #[test]
    fn grouped_lookup_once_sparse_cache_then_complete_physical_union() {
        let f = Fixture::new();
        file(&f.original.join("entry"), b"required");
        file(&f.original.join("unlisted"), b"not declared");
        let source = SourceSession::new(&f.original);
        let selector = Select { link: false };
        let names = [DeclaredName {
            name: OsStr::new("entry"),
            desired: desired(LeafExpectation::SupportedLeaf),
        }];
        let at = DeclaredRequest {
            relative: Path::new(""),
            purpose: Purpose::Copy,
        };
        let lanes = [
            lane(0, &f.target, Some(&f.target), DeclaredNames::Exact(&names), &selector),
            lane(1, &f.target, None, DeclaredNames::Exact(&names), &selector),
        ];
        assert!(
            inspect_declared_group(&source, at, &lanes, &NoExecutionDemand, &InspectionEpoch::new(), Execution::Sequential)
                .unwrap()
                .iter()
                .all(|o| o.outcome.is_ok())
        );
        assert_eq!(source.diagnostics().declared.metadata_acquisitions, 1);
        assert_eq!(source.diagnostics().declared.listings, 0);
        let mut outcomes = inspect_group(
            &source,
            DirectoryRequest {
                relative: Path::new(""),
                source_directory: Some(&f.original),
                purpose: Purpose::Compare,
                admission: DirectoryAdmission::AlreadyAdmitted,
            },
            &[TargetInspection {
                target: 0,
                logical_root: &f.target,
                physical_directory: Some(&f.target),
                context: &(),
                policy: &Mirror,
                comparator: &StatKey,
            }],
            &NoExecutionDemand,
            &InspectionEpoch::new(),
            Execution::Sequential,
        )
        .unwrap();
        let prepared = outcomes.pop().unwrap().outcome.unwrap();
        assert_eq!(
            prepared.selection().entries().map(|e| e.name().to_str().unwrap()).collect::<Vec<_>>(),
            ["entry", "unlisted"]
        );
        assert_eq!(source.diagnostics().paired.listings, 1);
        assert_eq!(source.diagnostics().paired.cache_hits, 0);
    }
    #[cfg(unix)]
    #[test]
    fn genuine_missing_target_prefix_is_not_a_symlink_or_enotdir() {
        use std::os::unix::fs::symlink;
        let f = Fixture::new();
        let source = SourceSession::new(f.original.join("not-needed"));
        let selector = Select { link: false };
        let names = [DeclaredName {
            name: OsStr::new("old"),
            desired: None,
        }];
        for (prefix, valid) in [("missing/deep", true), ("file/deep", false), ("link/deep", false)] {
            if prefix.starts_with("file") {
                file(&f.target.join("file"), b"obstruction");
            }
            if prefix.starts_with("link") {
                symlink(&f.original, f.target.join("link")).unwrap();
            }
            let path = f.target.join(prefix);
            let results = inspect_declared_group(
                &source,
                DeclaredRequest {
                    relative: Path::new(prefix),
                    purpose: Purpose::Remove,
                },
                &[lane(0, &f.target, Some(&path), DeclaredNames::Exact(&names), &selector)],
                &NoExecutionDemand,
                &InspectionEpoch::new(),
                Execution::Sequential,
            )
            .unwrap();
            assert_eq!(results[0].outcome.is_ok(), valid);
        }
    }
}
