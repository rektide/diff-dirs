//! Selected contextual recursion. Physical primitives deliberately have no policy.

use crate::{
    Applier, Failure, Policy, Progress, Report,
    apply::{PhysicalExecutionDemand, inspection_failure, require_execution_inputs},
    kernel::{self, BoundDestination, DirectoryEnd, DirectoryScope},
};
use diff_dirs_tree_inspect::{
    Action, Comparator, Container, DirectoryAdmission, DirectoryRequest, EntrySelectionRef, Execution, InspectionEpoch, Interior, InteriorRole,
    NoExecutionDemand, Policy as SelectionPolicy, PreparedDirectory, Purpose, SelectionDomain, SlotId, SourceSession, TargetInspection, inspect_group,
};
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

/// Lane-local cancellation/counters; no healthy future cohort shares these.
pub(crate) struct LaneCall<'a, P, M> {
    pub(crate) binding: &'a Applier<P, M>,
    stopped: AtomicBool,
    retained: AtomicU64,
    excluded: AtomicU64,
    planned: AtomicU64,
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
    donor: std::fs::Permissions,
    created: bool,
    needed: bool,
}

impl<'a, P: SelectionPolicy, M: Comparator> LaneCall<'a, P, M> {
    pub(crate) fn add_excluded(&self, count: u64) {
        self.excluded.fetch_add(count, Ordering::Relaxed);
    }
    pub(crate) fn new(binding: &'a Applier<P, M>) -> Self {
        Self {
            binding,
            stopped: AtomicBool::new(false),
            retained: AtomicU64::new(0),
            excluded: AtomicU64::new(0),
            planned: AtomicU64::new(0),
        }
    }
    pub(crate) fn finish(&self, result: Result<Progress, Failure>) -> Result<Report, Failure> {
        let policy = Policy {
            satisfied: result.is_ok(),
            retained: self.retained.load(Ordering::Relaxed),
            excluded: self.excluded.load(Ordering::Relaxed),
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
    fn count(&self, prepared: &PreparedDirectory<P::Context>) {
        let selected = prepared.selection();
        self.retained.fetch_add(selected.coverage().deviated, Ordering::Relaxed);
        self.excluded.fetch_add(selected.coverage().excluded, Ordering::Relaxed);
    }
    fn planned(&self, prepared: &PreparedDirectory<P::Context>) {
        self.planned.fetch_add(
            prepared.selection().entries().filter(|e| e.has_work() && !conditional(e)).count() as u64,
            Ordering::Relaxed,
        );
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
            kernel::route(call.epoch, if root { target.parent().expect("bound target has parent") } else { target })?;
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
            inspect_group(call.source, request, &[lane], &PhysicalExecutionDemand, call.epoch, call.execution)
        } else {
            inspect_group(call.source, request, &[lane], &NoExecutionDemand, call.epoch, call.execution)
        }
        .map_err(|e| {
            Failure::io(
                "inspect physical batch",
                place.relative,
                std::io::Error::new(std::io::ErrorKind::InvalidInput, e),
            )
        })?;
        result.into_iter().next().expect("one requested lane").outcome.map_err(inspection_failure)
    }
    fn gate(&self, call: &EffectCall<'_>, prepared: &PreparedDirectory<P::Context>, place: &BatchPlace<'_>) -> Result<(), Failure> {
        let s = prepared.selection();
        if prepared.source_root() != self.binding.original()
            || prepared.target_root() != self.binding.target()
            || call.source.root() != self.binding.original()
            || prepared.source_generation() != call.source.generation()
            || s.relative() != place.relative
            || s.policy() != self.binding.selection.identity()
            || s.context_schema() != self.binding.selection.context_schema()
            || s.comparator() != self.binding.comparator.identity()
            || s.domain() != SelectionDomain::PhysicalUnion
        {
            return Err(Failure::invalid(
                "bind prepared effects",
                place.relative,
                "preparation does not match this live physical interpretation",
            ));
        }
        require_execution_inputs(prepared)
    }

    pub(crate) fn apply_ready(
        &self,
        call: &EffectCall<'_>,
        prepared: &PreparedDirectory<P::Context>,
        place: BatchPlace<'_>,
        owner: RecursionOwner,
        selected: Option<&[(std::ffi::OsString, u8)]>,
    ) -> Result<Progress, Failure> {
        self.gate(call, prepared, &place)?; // Pure whole-batch gate BEFORE scope/I/O.
        self.count(prepared);
        let entries: Vec<_> = (0..prepared.selection().len())
            .filter(|&index| {
                let entry = prepared.selection().entry(SlotId(index));
                selected.is_none_or(|names| {
                    names
                        .binary_search_by(|(n, _)| n.as_encoded_bytes().cmp(entry.name().as_encoded_bytes()))
                        .is_ok()
                })
            })
            .collect();
        self.planned.fetch_add(
            entries
                .iter()
                .filter(|&&i| {
                    let e = prepared.selection().entry(SlotId(i));
                    e.has_work() && !conditional(&e)
                })
                .count() as u64,
            Ordering::Relaxed,
        );
        let scope = DirectoryScope::new(
            if prepared.selection().is_root() {
                place.output_directory.parent().expect("bound root has parent")
            } else {
                place.output_directory
            },
            prepared.selection().entries().map(|e| e.name()),
        );
        let result = self.batch(call, &entries, |&index| {
            self.execute_entry(call, prepared, SlotId(index), &place, &scope, owner).map(|(p, _)| p)
        });
        let complete = result.is_ok() && !self.stopped.load(Ordering::Acquire);
        scope.finish(result, DirectoryEnd::Retained, complete, call.epoch)
    }

    fn batch<T: Sync>(&self, call: &EffectCall<'_>, entries: &[T], operation: impl Fn(&T) -> Result<Progress, Failure> + Sync) -> Result<Progress, Failure> {
        let run = |entry: &T| {
            if self.stopped.load(Ordering::Acquire) {
                return Ok(Progress::default());
            }
            let result = operation(entry);
            if result.is_err() {
                self.stopped.store(true, Ordering::Release);
            }
            result
        };
        #[cfg(feature = "parallel")]
        if call.execution == Execution::Parallel {
            use rayon::prelude::*;
            let results: Vec<_> = entries.par_iter().map(run).collect();
            return crate::model::merge(results);
        }
        let _ = call;
        crate::model::merge(entries.iter().map(run))
    }

    fn execute_entry(
        &self,
        call: &EffectCall<'_>,
        prepared: &PreparedDirectory<P::Context>,
        slot: SlotId,
        place: &BatchPlace<'_>,
        scope: &DirectoryScope,
        owner: RecursionOwner,
    ) -> Result<(Progress, bool), Failure> {
        let entry = prepared.selection().entry(slot);
        let relative = if prepared.selection().is_root() {
            place.relative.to_owned()
        } else {
            place.relative.join(entry.name())
        };
        let from = if prepared.selection().is_root() {
            place.source_directory.map(Path::to_owned)
        } else {
            place.source_directory.map(|p| p.join(entry.name()))
        };
        let to = if prepared.selection().is_root() {
            place.output_directory.to_owned()
        } else {
            place.output_directory.join(entry.name())
        };
        #[cfg(all(test, feature = "parallel"))]
        crate::apply::tests::observe(&to);
        let facts = prepared.facts(slot);
        match entry.action {
            Action::ConfirmAbsent => Err(Failure::invalid(
                "apply physical plan",
                &relative,
                "listed absence obligation in physical execution",
            )),
            Action::PreserveTree | Action::KeepLeaf => Ok((Progress::default(), entry.target.is_some())),
            Action::InstallLeaf | Action::ReplaceWithLeaf => {
                if entry.action == Action::ReplaceWithLeaf {
                    self.removable(call, &to, &relative, entry.interior.as_ref().expect("selected removal interior"))?;
                }
                let staged = scope.stage_slot(slot, &to, call.epoch)?;
                let mut destination = BoundDestination::new(&to, scope, entry.install.expect("selected install authority").destination, entry.target)?;
                let leaf = kernel::prepare_leaf(
                    from.as_deref().expect("selected source leaf"),
                    facts.source.expect("source leaf"),
                    &staged,
                    &to,
                    self.binding.policy,
                )?;
                let removed = if entry.action == Action::ReplaceWithLeaf {
                    let (removed, retained) = self.reconcile_directory(call, None, &to, &relative, entry.interior.as_ref().unwrap(), true, scope)?;
                    if retained {
                        let mut f = Failure::invalid("replace directory", &to, "target subtree retained by policy");
                        f.completed = removed;
                        return Err(f);
                    }
                    destination.cleared();
                    removed
                } else {
                    Progress::default()
                };
                let mut result = kernel::publish_leaf(leaf, &staged, &destination, call.epoch);
                kernel::add_progress(&mut result, removed);
                result.map(|p| (p, false))
            }
            Action::RemoveLeaf => {
                scope.ensure_writable(call.epoch)?;
                kernel::remove_leaf(&to, facts.target.expect("target leaf"), call.epoch).map(|p| (p, false))
            }
            Action::Directory {
                container: Container::Create,
                descend,
            } => {
                scope.ensure_writable(call.epoch)?;
                self.copy_directory(
                    call,
                    from.as_deref().unwrap(),
                    &to,
                    &relative,
                    facts.source.unwrap().attributes.known().unwrap().permissions(),
                    entry.interior.as_ref(),
                    descend,
                )
                .map(|p| (p, false))
            }
            Action::Directory {
                container: Container::CreateIfNeeded,
                ..
            } => self
                .copy_pending(
                    call,
                    from.as_deref().unwrap(),
                    &to,
                    &relative,
                    facts.source.unwrap().attributes.known().unwrap().permissions(),
                    entry.interior.as_ref().unwrap(),
                    scope,
                    &mut Vec::new(),
                )
                .map(|p| (p, false)),
            Action::ReplaceWithDirectory { descend } => {
                let staged = scope.stage_slot(slot, &to, call.epoch)?;
                let result = self.copy_directory(
                    call,
                    from.as_deref().unwrap(),
                    staged.path(),
                    &relative,
                    facts.source.unwrap().attributes.known().unwrap().permissions(),
                    entry.interior.as_ref(),
                    descend,
                );
                match result {
                    Ok(progress) if !self.stopped.load(Ordering::Acquire) => {
                        let removed = kernel::remove_leaf(&to, facts.target.expect("old leaf"), call.epoch)?;
                        let mut destination = BoundDestination::new(&to, scope, entry.install.unwrap().destination, entry.target)?;
                        destination.cleared();
                        let mut result = kernel::publish(&staged, &destination, call.epoch).map(|()| progress);
                        kernel::add_progress(&mut result, removed);
                        result.map(|p| (p, false))
                    }
                    result => kernel::discard_private(result).map(|p| (p, false)),
                }
            }
            Action::Directory { container, descend } => {
                if !descend {
                    return Ok((Progress::default(), true));
                }
                let interior = entry.interior.as_ref().expect("selected interior");
                if interior.role == InteriorRole::Paired && owner == RecursionOwner::PairedFrontier {
                    return Ok((Progress::default(), true));
                }
                self.reconcile_directory(
                    call,
                    if interior.role == InteriorRole::Paired { from.as_deref() } else { None },
                    &to,
                    &relative,
                    interior,
                    container == Container::Remove,
                    scope,
                )
            }
        }
    }

    fn copy_directory(
        &self,
        call: &EffectCall<'_>,
        from: &Path,
        to: &Path,
        relative: &Path,
        donor: std::fs::Permissions,
        interior: Option<&Interior<P::Context>>,
        descend: bool,
    ) -> Result<Progress, Failure> {
        let scope = DirectoryScope::new(to, std::iter::empty());
        kernel::create_directory(&scope, call.epoch)?;
        // Creation is owned immediately. Every subsequent failure finalizes and
        // reports it, including post-umask metadata/lease setup failures.
        let result = (|| {
            scope.ensure_writable(call.epoch)?;
            if !descend {
                return Ok(Progress::default());
            }
            let place = BatchPlace {
                relative,
                source_directory: Some(from),
                observed_target_directory: None,
                output_directory: to,
            };
            let batch = self.inspect(call, &place, &interior.expect("selected source context").context, false, true)?;
            self.execute_in_scope(call, &batch, &place, &scope)
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
    fn execute_in_scope(
        &self,
        call: &EffectCall<'_>,
        batch: &PreparedDirectory<P::Context>,
        place: &BatchPlace<'_>,
        scope: &DirectoryScope,
    ) -> Result<Progress, Failure> {
        self.gate(call, batch, place)?;
        self.count(batch);
        self.planned(batch);
        self.batch(call, &(0..batch.selection().len()).collect::<Vec<_>>(), |&i| {
            self.execute_entry(call, batch, SlotId(i), place, scope, RecursionOwner::ThisCall)
                .map(|(p, _)| p)
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn copy_pending(
        &self,
        call: &EffectCall<'_>,
        from: &Path,
        to: &Path,
        relative: &Path,
        donor: std::fs::Permissions,
        interior: &Interior<P::Context>,
        parent: &DirectoryScope,
        pending: &mut Vec<PendingDirectory>,
    ) -> Result<Progress, Failure> {
        if self.stopped.load(Ordering::Acquire) {
            return Ok(Progress::default());
        }
        let place = BatchPlace {
            relative,
            source_directory: Some(from),
            observed_target_directory: None,
            output_directory: to,
        };
        let batch = self.inspect(call, &place, &interior.context, false, true)?;
        self.gate(call, &batch, &place)?;
        self.count(&batch);
        self.planned(&batch);
        let scope = Arc::new(DirectoryScope::new(to, batch.selection().entries().map(|e| e.name())));
        pending.push(PendingDirectory {
            scope: Arc::clone(&scope),
            donor,
            created: false,
            needed: false,
        });
        let result = crate::model::merge((0..batch.selection().len()).map(|i| {
            if self.stopped.load(Ordering::Acquire) {
                return Ok(Progress::default());
            }
            let entry = batch.selection().entry(SlotId(i));
            let result = if conditional(&entry) {
                self.copy_pending(
                    call,
                    &from.join(entry.name()),
                    &to.join(entry.name()),
                    &relative.join(entry.name()),
                    batch.facts(SlotId(i)).source.unwrap().attributes.known().unwrap().permissions(),
                    entry.interior.as_ref().unwrap(),
                    parent,
                    pending,
                )
            } else {
                (|| {
                    if entry.has_work() {
                        self.materialize_pending(call, parent, pending)?;
                    }
                    self.execute_entry(call, &batch, SlotId(i), &place, &scope, RecursionOwner::ThisCall)
                        .map(|(p, _)| p)
                })()
            };
            if result.is_err() {
                self.stopped.store(true, Ordering::Release);
            }
            result
        }));
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

    #[allow(clippy::too_many_arguments)]
    fn reconcile_directory(
        &self,
        call: &EffectCall<'_>,
        source: Option<&Path>,
        path: &Path,
        relative: &Path,
        interior: &Interior<P::Context>,
        remove: bool,
        parent: &DirectoryScope,
    ) -> Result<(Progress, bool), Failure> {
        let place = BatchPlace {
            relative,
            source_directory: source,
            observed_target_directory: Some(path),
            output_directory: path,
        };
        let batch = self.inspect(call, &place, &interior.context, false, true)?;
        self.gate(call, &batch, &place)?;
        self.count(&batch);
        self.planned(&batch);
        let scope = DirectoryScope::new(path, batch.selection().entries().map(|e| e.name()));
        let mut kept = !remove || batch.selection().coverage().excluded != 0;
        let mut progress = Progress::default();
        let result = (|| {
            for i in 0..batch.selection().len() {
                match self.execute_entry(call, &batch, SlotId(i), &place, &scope, RecursionOwner::ThisCall) {
                    Ok((done, retained)) => {
                        progress += done;
                        kept |= retained;
                    }
                    Err(mut f) => {
                        f.completed += progress;
                        return Err(f);
                    }
                }
            }
            Ok(progress)
        })();
        let complete = result.is_ok() && !self.stopped.load(Ordering::Acquire);
        let result = scope.clean(result, complete, call.epoch);
        let result = result.and_then(|p| {
            if kept {
                self.retained.fetch_add(1, Ordering::Relaxed);
                return Ok(p);
            }
            let mut removed = (|| {
                parent.ensure_writable(call.epoch)?;
                kernel::remove_directory(path, call.epoch)
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
        for entry in batch.selection().entries() {
            match entry.action {
                Action::RemoveLeaf => {}
                Action::Directory {
                    container: Container::Remove,
                    descend: true,
                } => self.removable(call, &path.join(entry.name()), &relative.join(entry.name()), entry.interior.as_ref().unwrap())?,
                _ => {
                    return Err(Failure::invalid(
                        "replace directory",
                        &self.binding.target.join(relative).join(entry.name()),
                        "target descendant retained by policy",
                    ));
                }
            }
        }
        Ok(())
    }

    pub(crate) fn preview(&self, call: &EffectCall<'_>, batch: &PreparedDirectory<P::Context>) -> Result<bool, Failure> {
        self.count(batch);
        self.planned(batch);
        let mut needed = false;
        for entry in batch.selection().entries() {
            let relative = if batch.selection().is_root() {
                PathBuf::new()
            } else {
                batch.selection().relative().join(entry.name())
            };
            let from = self.binding.original.join(&relative);
            let to = self.binding.target.join(&relative);
            let mut work = entry.has_work() && !conditional(&entry);
            if let Some(interior) = &entry.interior {
                if interior.role == InteriorRole::RemoveForReplacement {
                    self.removable(call, &to, &relative, interior)?;
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
                work |= self.preview(call, &child)?;
            }
            if conditional(&entry) && work {
                self.planned.fetch_add(1, Ordering::Relaxed);
            }
            needed |= work;
        }
        Ok(needed)
    }
}
