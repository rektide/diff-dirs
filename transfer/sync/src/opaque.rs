//! Contextual recursive control over policy-free physical primitives.
mod cohort;
use crate::{
    Applier, Failure, Policy, Progress, Report,
    apply::{PhysicalExecutionDemand, inspection_failure, require_execution_inputs},
    kernel::{self, BoundDestination, DirectoryEnd, DirectoryScope, PreparedLeaf, StageSlot},
};
pub(crate) use cohort::apply_cohort;
use diff_dirs_tree_inspect::{
    Action, Comparator, Container, DirectoryAdmission, DirectoryRequest, EntrySelectionRef, Execution, InspectionEpoch, Interior, InteriorRole, Kind,
    NoExecutionDemand, Policy as SelectionPolicy, PreparedDirectory, Purpose, SelectionDomain, SlotId, SourceSession, SourceWork, TargetInspection,
    inspect_group_with_work,
};
use std::fs::{Metadata, Permissions};
use std::path::{Path, PathBuf};
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicU64, Ordering},
};

pub(crate) struct EffectCall<'a> {
    pub(crate) source: &'a SourceSession,
    pub(crate) epoch: &'a InspectionEpoch,
    pub(crate) execution: Execution,
}
pub(crate) struct LaneCall<'a, P, M> {
    pub(crate) binding: &'a Applier<P, M>,
    stopped: AtomicBool,
    immediate: Counts,
    opaque: Counts,
    necessary: AtomicU64,
    planned: AtomicU64,
}
#[derive(Default)]
struct Counts {
    deviated: AtomicU64,
    retained: AtomicU64,
    excluded: AtomicU64,
}
/// Constant-size returned accounting, never persisted as an operation receipt.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Additions {
    pub(crate) deviated: u64,
    pub(crate) retained: u64,
}
pub(crate) struct ApplicationResult {
    pub(crate) result: Result<Report, Failure>,
    pub(crate) additions: Additions,
}
impl ApplicationResult {
    pub(crate) fn only_additions(mut self) -> Result<Report, Failure> {
        let policy = match &mut self.result {
            Ok(r) => &mut r.policy,
            Err(f) => f.policy.as_mut(),
        };
        policy.deviated = self.additions.deviated;
        policy.retained = self.additions.retained;
        self.result
    }
}
pub(crate) struct BatchPlace<'a> {
    pub(crate) relative: &'a Path,
    pub(crate) source_directory: Option<&'a Path>,
    pub(crate) observed_target_directory: Option<&'a Path>,
    pub(crate) output_directory: &'a Path,
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum RecursionOwner {
    PairedFrontier,
    ThisCall,
}
fn conditional<C>(entry: &EntrySelectionRef<'_, C>) -> bool {
    matches!(
        entry.action,
        Action::Directory {
            container: Container::CreateIfNeeded,
            ..
        }
    )
}
struct PendingDirectory {
    scope: Arc<DirectoryScope>,
    donor: Permissions,
    created: bool,
    needed: bool,
}

/// Promoted recursion/finish frames, not a second selection or operation catalog.
/// Only selected interiors survive the immediate arena. Leaf-only work is done
/// while facts are borrowed; a collapse retains its prepared descriptors instead
/// of retaining source facts through the second removal pass.
struct Child<C> {
    relative: PathBuf,
    from: Option<PathBuf>,
    to: PathBuf,
    interior: Option<Interior<C>>,
    work: ChildWork,
}
enum ChildWork {
    Create {
        donor: Permissions,
        descend: bool,
    },
    Conditional {
        donor: Permissions,
    },
    Reconcile {
        remove: bool,
    },
    Expand {
        slot: SlotId,
        donor: Permissions,
        descend: bool,
        old_kind: Kind,
        old_attributes: Option<Metadata>,
        grant: diff_dirs_tree_inspect::DestinationGrant,
    },
    Collapse {
        slot: StageSlot,
        leaf: PreparedLeaf,
        grant: diff_dirs_tree_inspect::DestinationGrant,
    },
}
type EntryResult<C> = Result<(Progress, bool, Option<Child<C>>), Failure>;

impl<'a, P: SelectionPolicy, M: Comparator> LaneCall<'a, P, M> {
    pub(crate) fn new(binding: &'a Applier<P, M>) -> Self {
        Self {
            binding,
            stopped: AtomicBool::new(false),
            immediate: Counts::default(),
            opaque: Counts::default(),
            necessary: AtomicU64::new(0),
            planned: AtomicU64::new(0),
        }
    }
    pub(crate) fn add_excluded(&self, count: u64) {
        self.immediate.excluded.fetch_add(count, Ordering::Relaxed);
    }
    pub(crate) fn finish(&self, result: Result<Progress, Failure>) -> Result<Report, Failure> {
        let policy = Policy {
            satisfied: result.is_ok(),
            deviated: self.immediate.deviated.load(Ordering::Relaxed) + self.opaque.deviated.load(Ordering::Relaxed),
            retained: self.immediate.retained.load(Ordering::Relaxed) + self.opaque.retained.load(Ordering::Relaxed) + self.necessary.load(Ordering::Relaxed),
            excluded: self.immediate.excluded.load(Ordering::Relaxed) + self.opaque.excluded.load(Ordering::Relaxed),
        };
        match result {
            Ok(completed) => Ok(Report {
                planned: self.planned.load(Ordering::Relaxed),
                completed,
                policy,
                observed: None,
            }),
            Err(mut f) => {
                f.planned += self.planned.load(Ordering::Relaxed);
                *f.policy += policy;
                Err(f)
            }
        }
    }
    fn count(&self, batch: &PreparedDirectory<P::Context>, immediate: bool) {
        let c = batch.selection().coverage();
        let counts = if immediate { &self.immediate } else { &self.opaque };
        counts.deviated.fetch_add(c.deviated, Ordering::Relaxed);
        counts.retained.fetch_add(c.retained, Ordering::Relaxed);
        counts.excluded.fetch_add(c.excluded, Ordering::Relaxed);
    }
    pub(crate) fn finish_accounted(&self, result: Result<Progress, Failure>) -> ApplicationResult {
        ApplicationResult {
            result: self.finish(result),
            additions: Additions {
                deviated: self.opaque.deviated.load(Ordering::Relaxed),
                retained: self.opaque.retained.load(Ordering::Relaxed) + self.necessary.load(Ordering::Relaxed),
            },
        }
    }
    pub(crate) fn inspect(
        &self,
        call: &EffectCall<'_>,
        place: &BatchPlace<'_>,
        context: &P::Context,
        root: bool,
        effects: bool,
    ) -> Result<PreparedDirectory<P::Context>, Failure> {
        if let Some(source) = place.source_directory {
            kernel::route(call.epoch, source)?;
        }
        if let Some(target) = place.observed_target_directory {
            kernel::route(call.epoch, if root { target.parent().expect("bound parent") } else { target })?;
        }
        let request = DirectoryRequest {
            relative: place.relative,
            source_directory: place.source_directory,
            purpose: match (place.source_directory.is_some(), place.observed_target_directory.is_some()) {
                (true, false) => Purpose::Copy,
                (false, true) => Purpose::Remove,
                _ => Purpose::Compare,
            },
            admission: if root {
                DirectoryAdmission::RootGated
            } else {
                DirectoryAdmission::AlreadyAdmitted
            },
        };
        let lane = TargetInspection {
            target: 0,
            logical_root: self.binding.target(),
            physical_directory: place.observed_target_directory,
            context,
            policy: &self.binding.selection,
            comparator: &self.binding.comparator,
        };
        let result = if effects {
            inspect_group_with_work(
                call.source,
                request,
                &[lane],
                &PhysicalExecutionDemand,
                call.epoch,
                call.execution,
                SourceWork::Opaque,
            )
        } else {
            inspect_group_with_work(
                call.source,
                request,
                &[lane],
                &NoExecutionDemand,
                call.epoch,
                call.execution,
                SourceWork::Opaque,
            )
        }
        .map_err(|e| {
            Failure::io(
                "inspect physical batch",
                place.relative,
                std::io::Error::new(std::io::ErrorKind::InvalidInput, e),
            )
        })?;
        result.into_iter().next().expect("one lane").outcome.map_err(inspection_failure)
    }
    fn gate(&self, call: &EffectCall<'_>, batch: &PreparedDirectory<P::Context>, place: &BatchPlace<'_>) -> Result<(), Failure> {
        let s = batch.selection();
        if batch.source_root() != self.binding.original()
            || batch.target_root() != self.binding.target()
            || call.source.root() != self.binding.original()
            || batch.source_generation() != call.source.generation()
            || s.relative() != place.relative
            || s.policy() != self.binding.selection.identity()
            || s.context_schema() != self.binding.selection.context_schema()
            || s.comparator() != self.binding.comparator.identity()
            || s.domain() != SelectionDomain::PhysicalUnion
        {
            return Err(Failure::invalid(
                "bind prepared effects",
                place.relative,
                "preparation does not match live physical interpretation",
            ));
        }
        require_execution_inputs(batch)
    }
    pub(crate) fn apply_ready(
        &self,
        call: &EffectCall<'_>,
        batch: PreparedDirectory<P::Context>,
        place: BatchPlace<'_>,
        owner: RecursionOwner,
        selected: Option<&[(std::ffi::OsString, u8)]>,
    ) -> Result<Progress, Failure> {
        self.count(&batch, true);
        self.gate(call, &batch, &place)?;
        let scope = DirectoryScope::new(
            if batch.selection().is_root() {
                place.output_directory.parent().expect("bound parent")
            } else {
                place.output_directory
            },
            batch.selection().entries().map(|e| e.name()),
        );
        let result = self.dispatch(call, batch, &place, &scope, owner, selected, None, false).map(|(p, _)| p);
        let complete = result.is_ok() && !self.stopped.load(Ordering::Acquire);
        scope.finish(result, DirectoryEnd::Retained, complete, call.epoch)
    }
    fn map<T: Sync, U: Send>(&self, call: &EffectCall<'_>, items: &[T], parallel: bool, operation: impl Fn(&T) -> U + Sync) -> Vec<U> {
        #[cfg(feature = "parallel")]
        if parallel && call.execution == Execution::Parallel {
            use rayon::prelude::*;
            return items.par_iter().map(&operation).collect();
        }
        let _ = (call, parallel);
        items.iter().map(operation).collect()
    }
    #[allow(clippy::too_many_arguments)]
    fn dispatch(
        &self,
        call: &EffectCall<'_>,
        batch: PreparedDirectory<P::Context>,
        place: &BatchPlace<'_>,
        scope: &DirectoryScope,
        owner: RecursionOwner,
        selected: Option<&[(std::ffi::OsString, u8)]>,
        mut pending: Option<(&DirectoryScope, &mut Vec<PendingDirectory>)>,
        opaque: bool,
    ) -> Result<(Progress, bool), Failure> {
        self.gate(call, &batch, place)?;
        if opaque {
            self.count(&batch, false);
        }
        let slots: Vec<_> = (0..batch.selection().len())
            .filter(|&i| {
                selected.is_none_or(|names| {
                    names
                        .binary_search_by(|(n, _)| n.as_encoded_bytes().cmp(batch.selection().entry(SlotId(i)).name().as_encoded_bytes()))
                        .is_ok()
                })
            })
            .map(SlotId)
            .collect();
        self.planned.fetch_add(
            slots
                .iter()
                .filter(|&&i| {
                    let e = batch.selection().entry(i);
                    e.has_work() && !conditional(&e)
                })
                .count() as u64,
            Ordering::Relaxed,
        );
        let prepare = |slot: &SlotId| {
            if self.stopped.load(Ordering::Acquire) {
                return Ok((Progress::default(), true, None));
            }
            let result = self.prepare_entry(call, &batch, *slot, place, scope, owner);
            if result.is_err() {
                self.stopped.store(true, Ordering::Release);
            }
            result
        };
        // Preserve the existing execution boundary: paired-frontier siblings and
        // independent copy children may use the pool; recursive reconciliation
        // (including both removal passes) and conditional chains stay serial.
        let parallel = pending.is_none() && (owner == RecursionOwner::PairedFrontier || place.observed_target_directory.is_none());
        let staged = if let Some((parent, chain)) = pending.as_mut() {
            let mut staged = Vec::with_capacity(slots.len());
            for slot in slots {
                let entry = batch.selection().entry(slot);
                let result = if !self.stopped.load(Ordering::Acquire) && entry.has_work() && !conditional(&entry) {
                    self.materialize_pending(call, parent, chain).and_then(|()| prepare(&slot))
                } else {
                    prepare(&slot)
                };
                if result.is_err() {
                    self.stopped.store(true, Ordering::Release);
                }
                staged.push(result);
            }
            staged
        } else {
            self.map(call, &slots, parallel, prepare)
        };
        let mut kept = batch.selection().coverage().excluded != 0;
        // No child/finish frame borrows the selected name table or fact slab.
        drop(batch);
        let mut outcomes = Vec::with_capacity(staged.len());
        let mut children = Vec::new();
        for result in staged {
            let index = outcomes.len();
            outcomes.push(match result {
                Ok((p, k, child)) => {
                    kept |= k;
                    if let Some(child) = child {
                        children.push((index, child));
                    }
                    Ok(p)
                }
                Err(f) => Err(f),
            });
        }
        let results = if let Some((parent, chain)) = pending {
            let mut results = Vec::with_capacity(children.len());
            for (_, child) in &children {
                results.push(self.run_child(call, child, scope, Some((parent, &mut *chain))));
            }
            results
        } else {
            self.map(call, &children, parallel, |(_, child)| self.run_child(call, child, scope, None))
        };
        for ((index, _), result) in children.into_iter().zip(results) {
            match result {
                Ok((p, k)) => {
                    kept |= k;
                    kernel::add_progress(&mut outcomes[index], p);
                }
                Err(mut f) => {
                    let earlier = std::mem::replace(&mut outcomes[index], Ok(Progress::default()));
                    match earlier {
                        Ok(p) => f.completed += p,
                        Err(mut prior) => {
                            prior.append(f);
                            f = prior;
                        }
                    }
                    outcomes[index] = Err(f);
                }
            }
        }
        crate::model::merge(outcomes).map(|p| (p, kept))
    }
    fn prepare_entry(
        &self,
        call: &EffectCall<'_>,
        batch: &PreparedDirectory<P::Context>,
        slot: SlotId,
        place: &BatchPlace<'_>,
        scope: &DirectoryScope,
        owner: RecursionOwner,
    ) -> EntryResult<P::Context> {
        let entry = batch.selection().entry(slot);
        let root = batch.selection().is_root();
        let relative = if root { place.relative.to_owned() } else { place.relative.join(entry.name()) };
        let from = place.source_directory.map(|p| if root { p.to_owned() } else { p.join(entry.name()) });
        let to = if root {
            place.output_directory.to_owned()
        } else {
            place.output_directory.join(entry.name())
        };
        #[cfg(all(test, feature = "parallel"))]
        crate::apply::tests::observe(&to);
        let facts = batch.facts(slot);
        let work = match entry.action {
            Action::ConfirmAbsent => return Err(Failure::invalid("apply physical plan", &relative, "listed obligation in physical execution")),
            Action::PreserveTree | Action::KeepLeaf => return Ok((Progress::default(), entry.target.is_some(), None)),
            Action::InstallLeaf => {
                let staged = scope.stage_slot(slot, &to, call.epoch)?;
                let leaf = kernel::prepare_leaf(from.as_deref().unwrap(), facts.source.unwrap(), &staged, &to, self.binding.policy)?;
                return kernel::publish_leaf(
                    leaf,
                    &staged,
                    &BoundDestination::new(&to, scope, entry.install.unwrap().destination, entry.target)?,
                    call.epoch,
                )
                .map(|p| (p, false, None));
            }
            Action::RemoveLeaf => {
                scope.ensure_writable(call.epoch)?;
                return kernel::remove_leaf(&to, facts.target.unwrap(), call.epoch).map(|p| (p, false, None));
            }
            Action::ReplaceWithLeaf => {
                self.removable(call, &to, &relative, entry.interior.as_ref().unwrap())?;
                let staged = scope.stage_slot(slot, &to, call.epoch)?;
                let leaf = kernel::prepare_leaf(from.as_deref().unwrap(), facts.source.unwrap(), &staged, &to, self.binding.policy)?;
                ChildWork::Collapse {
                    slot: staged,
                    leaf,
                    grant: entry.install.unwrap().destination,
                }
            }
            Action::Directory {
                container: Container::Create,
                descend,
            } => ChildWork::Create {
                donor: facts.source.unwrap().attributes.known().unwrap().permissions(),
                descend,
            },
            Action::Directory {
                container: Container::CreateIfNeeded,
                ..
            } => ChildWork::Conditional {
                donor: facts.source.unwrap().attributes.known().unwrap().permissions(),
            },
            Action::ReplaceWithDirectory { descend } => ChildWork::Expand {
                slot,
                donor: facts.source.unwrap().attributes.known().unwrap().permissions(),
                descend,
                old_kind: entry.target.unwrap(),
                old_attributes: if cfg!(windows) && entry.target == Some(Kind::Symlink) {
                    facts.target.unwrap().attributes.known().cloned()
                } else {
                    None
                },
                grant: entry.install.unwrap().destination,
            },
            Action::Directory { container, descend } => {
                if !descend
                    || entry
                        .interior
                        .as_ref()
                        .is_some_and(|i| i.role == InteriorRole::Paired && owner == RecursionOwner::PairedFrontier)
                {
                    return Ok((Progress::default(), true, None));
                }
                ChildWork::Reconcile {
                    remove: container == Container::Remove,
                }
            }
        };
        let from = if entry
            .interior
            .as_ref()
            .is_some_and(|i| matches!(i.role, InteriorRole::TargetOnly | InteriorRole::RemoveForReplacement))
        {
            None
        } else {
            from
        };
        Ok((
            Progress::default(),
            false,
            Some(Child {
                relative,
                from,
                to,
                interior: entry.interior.clone(),
                work,
            }),
        ))
    }
    fn run_child(
        &self,
        call: &EffectCall<'_>,
        child: &Child<P::Context>,
        parent: &DirectoryScope,
        pending: Option<(&DirectoryScope, &mut Vec<PendingDirectory>)>,
    ) -> Result<(Progress, bool), Failure> {
        if self.stopped.load(Ordering::Acquire) {
            return Ok((Progress::default(), true));
        }
        let result = match &child.work {
            ChildWork::Create { donor, descend } => parent
                .ensure_writable(call.epoch)
                .and_then(|()| self.copy_directory(call, child, child.to.as_path(), donor.clone(), *descend))
                .map(|p| (p, false)),
            ChildWork::Conditional { donor } => if let Some((parent, pending)) = pending {
                self.copy_pending(call, child, donor.clone(), parent, pending)
            } else {
                self.copy_pending(call, child, donor.clone(), parent, &mut Vec::new())
            }
            .map(|p| (p, false)),
            ChildWork::Reconcile { remove } => self.reconcile_directory(call, child, *remove, parent),
            ChildWork::Expand {
                slot,
                donor,
                descend,
                old_kind,
                old_attributes,
                grant,
            } => (|| {
                let staged = parent.stage_slot(*slot, &child.to, call.epoch)?;
                let prepared = self.copy_directory(call, child, staged.path(), donor.clone(), *descend);
                match prepared {
                    Ok(progress) if !self.stopped.load(Ordering::Acquire) => {
                        let removed = kernel::remove_leaf_kind(&child.to, *old_kind, old_attributes.as_ref(), call.epoch)?;
                        let mut destination = BoundDestination::new(&child.to, parent, *grant, Some(*old_kind))?;
                        destination.cleared();
                        let mut result = kernel::publish(&staged, &destination, call.epoch).map(|()| progress);
                        kernel::add_progress(&mut result, removed);
                        result
                    }
                    result => kernel::discard_private(result),
                }
            })()
            .map(|p| (p, false)),
            ChildWork::Collapse { slot, leaf, grant } => (|| {
                let (removed, retained) = self.reconcile_directory(call, child, true, parent)?;
                if retained {
                    let mut f = Failure::invalid("replace directory", &child.to, "target subtree retained by policy");
                    f.completed = removed;
                    return Err(f);
                }
                let mut destination = BoundDestination::new(&child.to, parent, *grant, Some(Kind::Directory))?;
                destination.cleared();
                let mut result = kernel::publish_borrowed_leaf(leaf, slot, &destination, call.epoch);
                kernel::add_progress(&mut result, removed);
                result
            })()
            .map(|p| (p, false)),
        };
        if result.is_err() {
            self.stopped.store(true, Ordering::Release);
        }
        result
    }
    fn copy_directory(&self, call: &EffectCall<'_>, child: &Child<P::Context>, to: &Path, donor: Permissions, descend: bool) -> Result<Progress, Failure> {
        let scope = DirectoryScope::new(to, std::iter::empty());
        kernel::create_directory(&scope, call.epoch)?;
        let result = (|| {
            scope.ensure_writable(call.epoch)?;
            if !descend {
                return Ok(Progress::default());
            }
            let place = BatchPlace {
                relative: &child.relative,
                source_directory: child.from.as_deref(),
                observed_target_directory: None,
                output_directory: to,
            };
            let batch = self.inspect(call, &place, &child.interior.as_ref().unwrap().context, false, true)?;
            self.gate(call, &batch, &place)?;
            scope.reserve_names(batch.selection().entries().map(|e| e.name()));
            scope.ensure_writable(call.epoch)?;
            self.dispatch(call, batch, &place, &scope, RecursionOwner::ThisCall, None, None, true)
                .map(|(p, _)| p)
        })();
        let complete = result.is_ok() && !self.stopped.load(Ordering::Acquire);
        let mut result = scope.finish(result, DirectoryEnd::Created { donor }, complete, call.epoch);
        kernel::add_progress(
            &mut result,
            Progress {
                directories_created: 1,
                ..Progress::default()
            },
        );
        result
    }
    fn copy_pending(
        &self,
        call: &EffectCall<'_>,
        child: &Child<P::Context>,
        donor: Permissions,
        parent: &DirectoryScope,
        pending: &mut Vec<PendingDirectory>,
    ) -> Result<Progress, Failure> {
        let place = BatchPlace {
            relative: &child.relative,
            source_directory: child.from.as_deref(),
            observed_target_directory: None,
            output_directory: &child.to,
        };
        let batch = self.inspect(call, &place, &child.interior.as_ref().unwrap().context, false, true)?;
        self.gate(call, &batch, &place)?;
        let scope = Arc::new(DirectoryScope::new(&child.to, batch.selection().entries().map(|e| e.name())));
        pending.push(PendingDirectory {
            scope: Arc::clone(&scope),
            donor,
            created: false,
            needed: false,
        });
        let result = self
            .dispatch(call, batch, &place, &scope, RecursionOwner::ThisCall, None, Some((parent, pending)), true)
            .map(|(p, _)| p);
        let frame = pending.pop().expect("active conditional directory");
        if frame.needed {
            self.planned.fetch_add(1, Ordering::Relaxed);
        }
        drop(scope);
        let scope = Arc::try_unwrap(frame.scope).ok().expect("joined conditional children");
        let end = if frame.created {
            DirectoryEnd::Created { donor: frame.donor }
        } else {
            DirectoryEnd::Retained
        };
        let complete = result.is_ok() && !self.stopped.load(Ordering::Acquire);
        let mut result = scope.finish(result, end, complete, call.epoch);
        if frame.created {
            kernel::add_progress(
                &mut result,
                Progress {
                    directories_created: 1,
                    ..Progress::default()
                },
            );
        }
        result
    }
    fn materialize_pending(&self, call: &EffectCall<'_>, parent: &DirectoryScope, pending: &mut [PendingDirectory]) -> Result<(), Failure> {
        for frame in pending.iter_mut() {
            frame.needed = true;
        }
        parent.ensure_writable(call.epoch)?;
        for frame in pending {
            if !frame.created {
                kernel::create_directory(&frame.scope, call.epoch)?;
                frame.created = true;
                frame.scope.ensure_writable(call.epoch)?;
            }
        }
        Ok(())
    }
    fn reconcile_directory(
        &self,
        call: &EffectCall<'_>,
        child: &Child<P::Context>,
        remove: bool,
        parent: &DirectoryScope,
    ) -> Result<(Progress, bool), Failure> {
        let place = BatchPlace {
            relative: &child.relative,
            source_directory: child.from.as_deref(),
            observed_target_directory: Some(&child.to),
            output_directory: &child.to,
        };
        let batch = self.inspect(call, &place, &child.interior.as_ref().unwrap().context, false, true)?;
        self.gate(call, &batch, &place)?;
        let scope = DirectoryScope::new(&child.to, batch.selection().entries().map(|e| e.name()));
        let mut kept = !remove;
        let result = self
            .dispatch(call, batch, &place, &scope, RecursionOwner::ThisCall, None, None, true)
            .map(|(p, k)| {
                kept |= k;
                p
            });
        let complete = result.is_ok() && !self.stopped.load(Ordering::Acquire);
        let result = scope.clean(result, complete, call.epoch);
        let result = result.and_then(|p| {
            if kept {
                if remove && !self.stopped.load(Ordering::Acquire) {
                    self.necessary.fetch_add(1, Ordering::Relaxed);
                }
                return Ok(p);
            }
            let mut removed = (|| {
                parent.ensure_writable(call.epoch)?;
                kernel::remove_directory(&child.to, call.epoch)
            })();
            kernel::add_progress(&mut removed, p);
            removed
        });
        let end = if !kept && result.is_ok() {
            DirectoryEnd::Removed
        } else {
            DirectoryEnd::Retained
        };
        scope.finish(result, end, complete, call.epoch).map(|p| (p, kept))
    }
    fn removable(&self, call: &EffectCall<'_>, path: &Path, relative: &Path, interior: &Interior<P::Context>) -> Result<(), Failure> {
        let batch = self.inspect(
            call,
            &BatchPlace {
                relative,
                source_directory: None,
                observed_target_directory: Some(path),
                output_directory: path,
            },
            &interior.context,
            false,
            false,
        )?;
        if let Some(name) = batch.selection().excluded_names().first() {
            return Err(Failure::invalid(
                "replace directory",
                &self.binding.target.join(relative).join(name),
                "target descendant preserved before observation",
            ));
        }
        let children = batch
            .selection()
            .entries()
            .map(|entry| match entry.action {
                Action::RemoveLeaf => Ok(None),
                Action::Directory {
                    container: Container::Remove,
                    descend: true,
                } => Ok(Some((entry.name().to_owned(), entry.interior.as_ref().unwrap().clone()))),
                _ => Err(Failure::invalid(
                    "replace directory",
                    &self.binding.target.join(relative).join(entry.name()),
                    "target descendant retained by policy",
                )),
            })
            .collect::<Vec<_>>();
        drop(batch);
        for child in children {
            if let Some((name, interior)) = child? {
                self.removable(call, &path.join(&name), &relative.join(name), &interior)?;
            }
        }
        Ok(())
    }
    pub(crate) fn preview(&self, call: &EffectCall<'_>, batch: PreparedDirectory<P::Context>) -> Result<bool, Failure> {
        self.preview_inner(call, batch, true).map(|(needed, _)| needed)
    }
    fn preview_inner(&self, call: &EffectCall<'_>, batch: PreparedDirectory<P::Context>, immediate: bool) -> Result<(bool, bool), Failure> {
        self.count(&batch, immediate);
        self.planned.fetch_add(
            batch.selection().entries().filter(|e| e.has_work() && !conditional(e)).count() as u64,
            Ordering::Relaxed,
        );
        let entries = batch
            .selection()
            .entries()
            .map(|entry| {
                (
                    if batch.selection().is_root() {
                        PathBuf::new()
                    } else {
                        batch.selection().relative().join(entry.name())
                    },
                    entry.has_work() && !conditional(&entry),
                    conditional(&entry),
                    entry.interior.clone(),
                    entry.action,
                    entry.target.is_some(),
                )
            })
            .collect::<Vec<_>>();
        drop(batch);
        let mut needed = false;
        let mut keeps_target = false;
        for (relative, mut work, conditional, interior, action, target) in entries {
            let mut kept = target
                && matches!(
                    action,
                    Action::PreserveTree
                        | Action::KeepLeaf
                        | Action::Directory {
                            container: Container::Keep,
                            ..
                        }
                );
            if let Some(interior) = interior {
                let from = self.binding.original.join(&relative);
                let to = self.binding.target.join(&relative);
                if interior.role == InteriorRole::RemoveForReplacement {
                    self.removable(call, &to, &relative, &interior)?;
                }
                let source = matches!(
                    interior.role,
                    InteriorRole::Paired | InteriorRole::SourceOnly | InteriorRole::PrepareReplacement
                )
                .then_some(from.as_path());
                let target = matches!(
                    interior.role,
                    InteriorRole::Paired | InteriorRole::TargetOnly | InteriorRole::RemoveForReplacement
                )
                .then_some(to.as_path());
                let child = self.inspect(
                    call,
                    &BatchPlace {
                        relative: &relative,
                        source_directory: source,
                        observed_target_directory: target,
                        output_directory: &to,
                    },
                    &interior.context,
                    false,
                    false,
                )?;
                let (child_work, child_kept) = self.preview_inner(call, child, false)?;
                work |= child_work;
                if matches!(
                    action,
                    Action::Directory {
                        container: Container::Remove,
                        ..
                    }
                ) && child_kept
                {
                    self.necessary.fetch_add(1, Ordering::Relaxed);
                    kept = true;
                }
            }
            if conditional && work {
                self.planned.fetch_add(1, Ordering::Relaxed);
            }
            needed |= work;
            keeps_target |= kept;
        }
        Ok((needed, keeps_target))
    }
}
