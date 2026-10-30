use super::selection::{Context, Mode, Scope, Selector, bind_listed_batch};
use super::*;
use crate::apply::{PhysicalExecutionDemand, inspection_failure};
use crate::kernel::{self, BoundDestination, DirectoryEnd, DirectoryScope, StageSlot};
use crate::{Progress, model::merge};
use diff_dirs_tree_inspect::*;
use std::collections::HashSet;
use std::fs::{Metadata, Permissions};
use std::path::{Path, PathBuf};

struct Lane<'a, G: ListedGuard> {
    selector: Selector<'a, G>,
    retired: HashSet<LeafId>,
    planned: u64,
    policy: crate::Policy,
}
impl<G: ListedGuard> Lane<'_, G> {
    fn retire_leaf(&mut self, leaf: LeafId) {
        let path = &self.selector.target.owned.index.leaves[leaf.0 as usize].relative;
        if self.selector.desired.index.exact_leaf(path).is_none() {
            self.retired.insert(leaf);
        }
    }
    fn retire_node(&mut self, node: NodeId) {
        let range = self.selector.target.owned.index.node(node).unwrap().leaves.clone();
        for leaf in range {
            self.retire_leaf(LeafId(leaf as u64));
        }
    }
    fn absent(&mut self, scope: &Scope, batch: &PreparedDirectory<Context>) {
        // The exact coordinate and whole immediate coverage were bound before
        // these credits. ApplicationNeed::None is deliberately not a skip here.
        for entry in batch.selection().entries().filter(|e| e.target.is_none()) {
            let relation = self.selector.relation(&scope.context, &scope.relative.join(entry.name()));
            if let Some(leaf) = relation.owned_leaf {
                self.retire_leaf(leaf);
            }
            if let Some(node) = relation.owned_directory {
                self.retire_node(node);
            }
        }
    }
    fn count(&mut self, batch: &PreparedDirectory<Context>) {
        let coverage = batch.selection().coverage();
        self.planned += coverage.planned;
        self.policy.deviated += coverage.deviated;
        self.policy.retained += coverage.retained;
    }
}
struct Work {
    result: Result<Progress, Failure>,
    installed: u64,
    retained: u64,
}
impl Default for Work {
    fn default() -> Self {
        Self {
            result: Ok(Progress::default()),
            installed: 0,
            retained: 0,
        }
    }
}
impl Work {
    fn good(&self) -> bool {
        self.result.is_ok()
    }
    fn take(&mut self) -> Result<Progress, Failure> {
        std::mem::replace(&mut self.result, Ok(Progress::default()))
    }
    fn add(&mut self, result: Result<Progress, Failure>) {
        let old = self.take();
        self.result = merge([old, result]);
    }
    fn join(&mut self, mut other: Work) {
        self.installed += other.installed;
        self.retained += other.retained;
        self.add(other.take());
    }
}
struct Job {
    lane: usize,
    place: Scope,
    directory: Option<DirectoryScope>,
    donor: Option<Permissions>,
}
struct Frame {
    job: Job,
    scope: DirectoryScope,
    work: Work,
    // Only root construction needs a separate external parent lease.
    root_parent: Option<DirectoryScope>,
    root_donor: Option<Permissions>,
}
enum ChildEnd {
    Keep,
    Created,
    Expand {
        slot: StageSlot,
        old_kind: Kind,
        old_attributes: Option<Box<Metadata>>,
        old_leaf: LeafId,
    },
}
struct Child {
    parent: usize,
    job: Job,
    end: ChildEnd,
}

pub(super) fn run<G: ListedGuard>(
    source: &SourceSession,
    desired: &PreparedLeafList,
    targets: &[ListedTarget<'_, G>],
) -> Result<ListedReport, crate::coordinated::Error> {
    use crate::coordinated::Error;
    // Structural request errors refuse atomically, before any route/guard/I/O.
    for (i, target) in targets.iter().enumerate() {
        if target.binding.source() != source.root() {
            return Err(Error::InvalidState("listed source binding differs from source generation"));
        }
        if target.id.0.is_empty() || targets[..i].iter().any(|t| t.id == target.id) {
            return Err(Error::InvalidState("listed target IDs must be nonempty and distinct"));
        }
        for prior in &targets[..i] {
            if prior.binding.target().starts_with(target.binding.target()) || target.binding.target().starts_with(prior.binding.target()) {
                return Err(Error::InvalidState("listed target roots overlap"));
            }
        }
    }
    let mut lanes: Vec<_> = targets
        .iter()
        .map(|target| Lane {
            selector: Selector::new(desired, target),
            retired: HashSet::new(),
            planned: 0,
            policy: crate::Policy::default(),
        })
        .collect();
    let jobs = lanes
        .iter()
        .enumerate()
        .map(|(lane, state)| Job {
            lane,
            directory: None,
            donor: None,
            place: Scope {
                relative: PathBuf::new(),
                context: state.selector.root(),
                observed: Some(state.selector.target.binding.target().to_owned()),
                output: state.selector.target.binding.target().to_owned(),
            },
        })
        .collect();
    let epoch = InspectionEpoch::new();
    let results = group(source, &epoch, &mut lanes, jobs);
    let outcomes = lanes
        .into_iter()
        .zip(results)
        .map(|(lane, mut work)| {
            let mut result = work.take();
            let owned_obsolete = if lane.selector.target.intent == ListedIntent::CreateAbsent {
                0
            } else {
                lane.selector.target.owned.paths().filter(|p| desired.index.exact_leaf(p).is_none()).count() as u64
            };
            let mut coverage = ManifestCoverage {
                desired: desired.len() as u64,
                installed: work.installed,
                retained: work.retained,
                owned_obsolete,
                owned_retired: lane.retired.len() as u64,
                complete: false,
            };
            if result.is_ok() && (coverage.installed + coverage.retained != coverage.desired || coverage.owned_retired != coverage.owned_obsolete) {
                let missing = Failure::invalid(
                    "finish listed coverage",
                    lane.selector.target.binding.target(),
                    "manifest obligations did not all complete",
                );
                result = merge([result, Err(missing)]);
            }
            coverage.complete =
                result.is_ok() && coverage.installed + coverage.retained == coverage.desired && coverage.owned_retired == coverage.owned_obsolete;
            let mut policy = lane.policy;
            policy.satisfied = coverage.complete;
            let outcome = match result {
                Ok(completed) => Ok(Report {
                    planned: lane.planned,
                    completed,
                    policy,
                    observed: None,
                }),
                Err(mut failure) => {
                    failure.planned += lane.planned;
                    *failure.policy += policy;
                    Err(failure)
                }
            };
            ListedOutcome {
                id: lane.selector.target.id.clone(),
                coverage,
                outcome,
            }
        })
        .collect();
    Ok(ListedReport { outcomes })
}

fn inspect<G: ListedGuard>(
    source: &SourceSession,
    epoch: &InspectionEpoch,
    lanes: &[Lane<'_, G>],
    jobs: &[Job],
    effects: bool,
) -> Vec<Result<PreparedDirectory<Context>, Failure>> {
    if jobs.is_empty() {
        return Vec::new();
    }
    let declarations: Vec<_> = jobs
        .iter()
        .map(|job| lanes[job.lane].selector.declarations(&job.place.relative, &job.place.context))
        .collect();
    let names: Vec<Vec<_>> = declarations
        .iter()
        .map(|names| names.iter().map(|(name, desired)| DeclaredName { name, desired: *desired }).collect())
        .collect();
    let requests: Vec<_> = jobs
        .iter()
        .zip(&names)
        .map(|(job, names)| {
            let state = &lanes[job.lane];
            DeclaredInspection {
                target: job.lane,
                logical_root: state.selector.target.binding.target(),
                physical_directory: job.place.observed.as_deref(),
                desired: state.selector.desired.id(),
                owned: state.selector.target.owned.id(),
                context: &job.place.context,
                names: if matches!(job.place.context.mode, Mode::Coupled { .. }) {
                    DeclaredNames::TargetCensus
                } else {
                    DeclaredNames::Exact(names)
                },
                selection: &state.selector,
            }
        })
        .collect();
    let at = DeclaredRequest {
        relative: &jobs[0].place.relative,
        purpose: if declarations.iter().any(|names| names.iter().any(|(_, desired)| desired.is_some())) {
            Purpose::Copy
        } else {
            Purpose::Remove
        },
    };
    let result = if effects {
        inspect_declared_group(source, at, &requests, &PhysicalExecutionDemand, epoch, Execution::Sequential)
    } else {
        inspect_declared_group(source, at, &requests, &NoExecutionDemand, epoch, Execution::Sequential)
    };
    match result {
        Ok(results) => results.into_iter().map(|r| r.outcome.map_err(inspection_failure)).collect(),
        Err(error) => jobs
            .iter()
            .map(|j| {
                Err(Failure::io(
                    "inspect declared batch",
                    &j.place.relative,
                    std::io::Error::new(std::io::ErrorKind::InvalidInput, error.clone()),
                ))
            })
            .collect(),
    }
}

/// Compatible source-owning children are joined before their next inspection,
/// independent of target shape and cache size. Only promoted child/finish values
/// survive an immediate batch; no physical-source enumeration or second copier.
fn group<G: ListedGuard>(source: &SourceSession, epoch: &InspectionEpoch, lanes: &mut [Lane<'_, G>], jobs: Vec<Job>) -> Vec<Work> {
    let preparations = inspect(source, epoch, lanes, &jobs, true);
    let mut frames = Vec::with_capacity(jobs.len());
    let mut children = Vec::new();
    for (mut job, preparation) in jobs.into_iter().zip(preparations) {
        let parent = frames.len();
        let scope = job
            .directory
            .take()
            .unwrap_or_else(|| DirectoryScope::new(&job.place.output, std::iter::empty()));
        let mut frame = Frame {
            job,
            scope,
            work: Work::default(),
            root_parent: None,
            root_donor: None,
        };
        let result = preparation.and_then(|batch| {
            let state = &mut lanes[frame.job.lane];
            let bound = bind_listed_batch(&batch, source, &state.selector, &frame.job.place, true)?;
            frame.scope.reserve_names(bound.batch.selection().entries().map(|e| e.name()));
            state.count(&batch);
            state.absent(&frame.job.place, &batch);
            // Root absence is an actual admitted-prefix observation. Build its
            // container only after this complete immediate preparation/bind gate.
            if frame.job.place.relative.as_os_str().is_empty() && !state.selector.desired.is_empty() {
                match epoch.directory(&frame.job.place.output) {
                    Ok(_) => {}
                    Err(error) if matches!(error.as_ref(), diff_dirs_tree_inspect::Error::Io { source, .. } if source.kind() == std::io::ErrorKind::NotFound) =>
                    {
                        let donor = epoch.directory(source.root()).map_err(inspection_failure)?.permissions();
                        let parent_scope = DirectoryScope::new(frame.job.place.output.parent().unwrap(), std::iter::empty());
                        parent_scope.ensure_writable(epoch)?;
                        frame.root_parent = Some(parent_scope);
                        kernel::create_directory(&frame.scope, epoch)?;
                        frame.root_donor = Some(donor);
                        frame.work.add(Ok(Progress {
                            directories_created: 1,
                            ..Progress::default()
                        }));
                        frame.scope.ensure_writable(epoch)?;
                    }
                    Err(error) => return Err(inspection_failure(error)),
                }
            }
            for slot in 0..batch.selection().len() {
                let state = &mut lanes[frame.job.lane];
                let slot = SlotId(slot);
                let entry = batch.selection().entry(slot);
                let facts = batch.facts(slot);
                let relative = frame.job.place.relative.join(entry.name());
                let output = frame.job.place.output.join(entry.name());
                let from = source.root().join(&relative);
                let interior = entry.interior.as_ref();
                let child_place = || Scope {
                    relative: relative.clone(),
                    context: interior.unwrap().context.clone(),
                    output: output.clone(),
                    observed: if matches!(interior.unwrap().context.mode, Mode::Prepare) {
                        None
                    } else {
                        Some(state.selector.target.binding.target().join(&relative))
                    },
                };
                match entry.action {
                    Action::KeepLeaf => {
                        frame.work.retained += 1;
                    }
                    Action::ConfirmAbsent | Action::PreserveTree => {}
                    Action::InstallLeaf => {
                        let staged = frame.scope.stage_slot(slot, &output, epoch)?;
                        let leaf = kernel::prepare_leaf(
                            &from,
                            facts.source.unwrap(),
                            &staged,
                            &output,
                            state.selector.target.binding.copy_policy(),
                            state.selector.target.binding.postcheck(),
                        )?;
                        frame.work.add(kernel::publish_leaf(
                            leaf,
                            &staged,
                            &BoundDestination::new(&output, &frame.scope, entry.install.unwrap().destination, entry.target)?,
                            epoch,
                        ));
                        if !frame.work.good() {
                            return Ok(());
                        }
                        frame.work.installed += 1;
                    }
                    Action::RemoveLeaf => {
                        frame.scope.ensure_writable(epoch)?;
                        frame.work.add(kernel::remove_leaf(&output, facts.target.unwrap(), epoch));
                        if !frame.work.good() {
                            return Ok(());
                        }
                        if let Some(RemovalAuthority::OwnedLeaf { leaf, .. }) = entry.removal {
                            state.retire_leaf(leaf);
                        }
                    }
                    Action::Directory {
                        container: Container::Keep, ..
                    } => children.push(Child {
                        parent,
                        job: Job {
                            lane: frame.job.lane,
                            place: child_place(),
                            directory: None,
                            donor: None,
                        },
                        end: ChildEnd::Keep,
                    }),
                    Action::Directory {
                        container: Container::Create, ..
                    } => {
                        let donor = facts.source.unwrap().attributes.known().unwrap().permissions();
                        frame.scope.ensure_writable(epoch)?;
                        let scope = DirectoryScope::new(&output, std::iter::empty());
                        kernel::create_directory(&scope, epoch)?;
                        frame.work.add(Ok(Progress {
                            directories_created: 1,
                            ..Progress::default()
                        }));
                        if let Err(failure) = scope.ensure_writable(epoch) {
                            frame.work.add(scope.finish(Err(failure), DirectoryEnd::Created { donor }, false, epoch));
                            return Ok(());
                        }
                        children.push(Child {
                            parent,
                            job: Job {
                                lane: frame.job.lane,
                                place: child_place(),
                                directory: Some(scope),
                                donor: Some(donor),
                            },
                            end: ChildEnd::Created,
                        });
                    }
                    Action::ReplaceWithDirectory { .. } => {
                        let donor = facts.source.unwrap().attributes.known().unwrap().permissions();
                        let staged = frame.scope.stage_slot(slot, &output, epoch)?;
                        let scope = DirectoryScope::new(staged.path(), std::iter::empty());
                        kernel::create_directory(&scope, epoch)?;
                        if let Err(failure) = scope.ensure_writable(epoch) {
                            frame.work.add(kernel::discard_private(scope.finish(
                                Err(failure),
                                DirectoryEnd::Created { donor },
                                false,
                                epoch,
                            )));
                            return Ok(());
                        }
                        let mut place = child_place();
                        place.output = staged.path().to_owned();
                        place.observed = None;
                        let Some(RemovalAuthority::OwnedLeaf { leaf, .. }) = entry.removal else {
                            unreachable!("bound expansion authority")
                        };
                        children.push(Child {
                            parent,
                            job: Job {
                                lane: frame.job.lane,
                                place,
                                directory: Some(scope),
                                donor: Some(donor),
                            },
                            end: ChildEnd::Expand {
                                slot: staged,
                                old_kind: entry.target.unwrap(),
                                old_attributes: facts.target.and_then(|f| f.attributes.known()).cloned().map(Box::new),
                                old_leaf: leaf,
                            },
                        });
                    }
                    Action::ReplaceWithLeaf => {
                        let removal = child_place();
                        preflight(source, epoch, lanes, frame.job.lane, &removal)?;
                        let state = &mut lanes[frame.job.lane];
                        let staged = frame.scope.stage_slot(slot, &output, epoch)?;
                        let leaf = kernel::prepare_leaf(
                            &from,
                            facts.source.unwrap(),
                            &staged,
                            &output,
                            state.selector.target.binding.copy_policy(),
                            state.selector.target.binding.postcheck(),
                        )?;
                        let removed = remove_tree(source, epoch, lanes, frame.job.lane, removal, &frame.scope);
                        frame.work.join(removed);
                        if !frame.work.good() {
                            return Ok(());
                        }
                        let mut destination = BoundDestination::new(&output, &frame.scope, entry.install.unwrap().destination, entry.target)?;
                        destination.cleared();
                        frame.work.add(kernel::publish_leaf(leaf, &staged, &destination, epoch));
                        if !frame.work.good() {
                            return Ok(());
                        }
                        frame.work.installed += 1;
                    }
                    _ => return Err(Failure::invalid("dispatch listed batch", &relative, "unreachable action after listed binding")),
                }
            }
            Ok(())
        });
        if let Err(failure) = result {
            frame.work.add(Err(failure));
        }
        frames.push(frame);
    }
    // Group by final source coordinate, not receiver or physical/staging path.
    children.sort_by(|a, b| path_order(&a.job.place.relative, &b.job.place.relative).then(a.job.lane.cmp(&b.job.lane)));
    while !children.is_empty() {
        let relative = children[0].job.place.relative.clone();
        let count = children.iter().take_while(|c| c.job.place.relative == relative).count();
        let peers: Vec<_> = children.drain(..count).collect();
        let mut active = Vec::new();
        let mut finishes = Vec::new();
        for mut child in peers {
            if frames[child.parent].work.good() {
                finishes.push((child.parent, child.end));
                active.push(child.job);
            } else {
                // Started containers still finish even though no child work starts
                // after this lane's failure. Private trees remain provisional.
                let scope = child
                    .job
                    .directory
                    .take()
                    .unwrap_or_else(|| DirectoryScope::new(&child.job.place.output, std::iter::empty()));
                let private = matches!(child.end, ChildEnd::Expand { .. });
                let end = child.job.donor.map_or(DirectoryEnd::Retained, |donor| DirectoryEnd::Created { donor });
                let result = scope.finish(Ok(Progress::default()), end, false, epoch);
                frames[child.parent].work.add(if private { kernel::discard_private(result) } else { result });
            }
        }
        let results = group(source, epoch, lanes, active);
        for ((parent, end), mut result) in finishes.into_iter().zip(results) {
            match end {
                ChildEnd::Keep | ChildEnd::Created => frames[parent].work.join(result),
                ChildEnd::Expand {
                    slot,
                    old_kind,
                    old_attributes,
                    old_leaf,
                    ..
                } => {
                    // The staged root itself is provisional too. All child finish
                    // obligations must succeed before removing the old leaf.
                    if result.good() {
                        let frame = &mut frames[parent];
                        let to = lanes[frame.job.lane].selector.target.binding.target().join(&relative);
                        let publication = (|| {
                            frame.scope.ensure_writable(epoch)?;
                            let removed = kernel::remove_leaf_kind(&to, old_kind, old_attributes.as_deref(), epoch)?;
                            frame.work.add(Ok(removed));
                            lanes[frame.job.lane].retire_leaf(old_leaf);
                            let mut destination = BoundDestination::new(&to, &frame.scope, DestinationGrant::ReplaceLeaf, Some(old_kind))?;
                            destination.cleared();
                            kernel::publish(&slot, &destination, epoch)?;
                            Ok(())
                        })();
                        match publication {
                            Ok(()) => {
                                result.add(Ok(Progress {
                                    directories_created: 1,
                                    ..Progress::default()
                                }));
                                frame.work.join(result);
                            }
                            Err(failure) => {
                                frame.work.add(Err(failure));
                                frame.work.add(kernel::discard_private(result.take()));
                            }
                        }
                    } else {
                        frames[parent].work.add(kernel::discard_private(result.take()));
                    }
                }
            }
        }
    }
    frames
        .into_iter()
        .map(|mut frame| {
            let complete = frame.work.good();
            let progress = frame.work.take();
            let end = frame
                .root_donor
                .or(frame.job.donor)
                .map_or(DirectoryEnd::Retained, |donor| DirectoryEnd::Created { donor });
            let mut result = frame.scope.finish(progress, end, complete, epoch);
            if let Some(parent) = frame.root_parent {
                let complete = result.is_ok();
                result = parent.finish(result, DirectoryEnd::Retained, complete, epoch);
            }
            frame.work.result = result;
            frame.work
        })
        .collect()
}

fn path_order(a: &Path, b: &Path) -> std::cmp::Ordering {
    a.components()
        .map(|c| c.as_os_str().as_encoded_bytes())
        .cmp(b.components().map(|c| c.as_os_str().as_encoded_bytes()))
}

fn preflight<G: ListedGuard>(source: &SourceSession, epoch: &InspectionEpoch, lanes: &[Lane<'_, G>], lane: usize, scope: &Scope) -> Result<(), Failure> {
    let job = Job {
        lane,
        directory: None,
        donor: None,
        place: Scope {
            relative: scope.relative.clone(),
            context: scope.context.clone(),
            observed: scope.observed.clone(),
            output: scope.output.clone(),
        },
    };
    let batch = inspect(source, epoch, lanes, std::slice::from_ref(&job), false).pop().unwrap()?;
    bind_listed_batch(&batch, source, &lanes[lane].selector, scope, false)?;
    let children: Vec<_> = batch
        .selection()
        .entries()
        .filter_map(|e| {
            e.interior.as_ref().map(|interior| Scope {
                relative: scope.relative.join(e.name()),
                context: interior.context.clone(),
                observed: Some(scope.output.join(e.name())),
                output: scope.output.join(e.name()),
            })
        })
        .collect();
    drop(batch);
    for child in children {
        preflight(source, epoch, lanes, lane, &child)?;
    }
    Ok(())
}

fn remove_tree<G: ListedGuard>(
    source: &SourceSession,
    epoch: &InspectionEpoch,
    lanes: &mut [Lane<'_, G>],
    lane: usize,
    place: Scope,
    parent: &DirectoryScope,
) -> Work {
    let job = Job {
        lane,
        place,
        directory: None,
        donor: None,
    };
    let prepared = inspect(source, epoch, lanes, std::slice::from_ref(&job), true).pop().unwrap();
    let scope = DirectoryScope::new(&job.place.output, std::iter::empty());
    let mut work = Work::default();
    let result = (|| {
        let batch = prepared?;
        bind_listed_batch(&batch, source, &lanes[lane].selector, &job.place, true)?;
        lanes[lane].count(&batch);
        enum Removal {
            Leaf {
                kind: Kind,
                attributes: Option<Box<Metadata>>,
                owned: LeafId,
            },
            Directory(Context),
        }
        // Runtime child continuations retain only the actual unlink inputs or
        // selected interior. No ancestor fact/name arena survives recursion.
        let children: Vec<_> = (0..batch.selection().len())
            .map(|slot| {
                let slot = SlotId(slot);
                let entry = batch.selection().entry(slot);
                let removal = if entry.action == Action::RemoveLeaf {
                    let facts = batch.facts(slot).target.unwrap();
                    let Some(RemovalAuthority::OwnedLeaf { leaf, .. }) = entry.removal else {
                        unreachable!("bound leaf removal")
                    };
                    Removal::Leaf {
                        kind: facts.kind,
                        attributes: facts.attributes.known().cloned().map(Box::new),
                        owned: leaf,
                    }
                } else {
                    Removal::Directory(entry.interior.as_ref().unwrap().context.clone())
                };
                (entry.name().to_owned(), removal)
            })
            .collect();
        drop(batch);
        for (name, removal) in children {
            let path = job.place.output.join(&name);
            match removal {
                Removal::Leaf { kind, attributes, owned } => {
                    scope.ensure_writable(epoch)?;
                    work.add(kernel::remove_leaf_kind(&path, kind, attributes.as_deref(), epoch));
                    if !work.good() {
                        return Ok(false);
                    }
                    lanes[lane].retire_leaf(owned);
                }
                Removal::Directory(context) => {
                    work.join(remove_tree(
                        source,
                        epoch,
                        lanes,
                        lane,
                        Scope {
                            relative: job.place.relative.join(&name),
                            context,
                            observed: Some(path.clone()),
                            output: path,
                        },
                        &scope,
                    ));
                    if !work.good() {
                        return Ok(false);
                    }
                }
            }
        }
        parent.ensure_writable(epoch)?;
        work.add(kernel::remove_directory(&job.place.output, epoch));
        if !work.good() {
            return Ok(false);
        }
        lanes[lane].retire_node(job.place.context.owned_node.unwrap());
        Ok(true)
    })();
    let removed = match result {
        Ok(removed) => removed,
        Err(failure) => {
            work.add(Err(failure));
            false
        }
    };
    let complete = work.good();
    let progress = work.take();
    work.result = scope.finish(progress, if removed { DirectoryEnd::Removed } else { DirectoryEnd::Retained }, complete, epoch);
    work
}
