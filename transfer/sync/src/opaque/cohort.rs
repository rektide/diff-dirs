//! Runtime-only scheduling of promoted child jobs. There is no saved interior.
use super::*;
use std::collections::VecDeque;

struct Place {
    relative: PathBuf,
    source: Option<PathBuf>,
    target: Option<PathBuf>,
    output: PathBuf,
}
impl Place {
    fn borrowed(&self) -> BatchPlace<'_> {
        BatchPlace {
            relative: &self.relative,
            source_directory: self.source.as_deref(),
            observed_target_directory: self.target.as_deref(),
            output_directory: &self.output,
        }
    }
}
/// A linked lane-local depth stack, not cloned per-child ancestry vectors.
struct Conditional {
    scope: Arc<DirectoryScope>,
    previous: Option<Arc<Self>>,
    parent: Arc<DirectoryScope>,
    created: AtomicBool,
    needed: AtomicBool,
}
impl Conditional {
    fn materialize(&self, call: &EffectCall<'_>) -> Result<(), Failure> {
        self.mark_needed();
        if let Some(previous) = &self.previous {
            previous.materialize(call)?;
        } else {
            self.parent.ensure_writable(call.epoch)?;
        }
        if !self.created.load(Ordering::Acquire) {
            kernel::create_directory(&self.scope, call.epoch)?;
            self.created.store(true, Ordering::Release);
            self.scope.ensure_writable(call.epoch)?;
        }
        Ok(())
    }
    fn mark_needed(&self) {
        self.needed.store(true, Ordering::Relaxed);
        if let Some(previous) = &self.previous {
            previous.mark_needed();
        }
    }
}
struct Input<'a, P: SelectionPolicy, M: Comparator> {
    lane: &'a LaneCall<'a, P, M>,
    batch: PreparedDirectory<P::Context>,
    place: Place,
    scope: Arc<DirectoryScope>,
    owner: RecursionOwner,
    pending: Option<Arc<Conditional>>,
}
struct Staged<'a, P: SelectionPolicy, M: Comparator> {
    lane: &'a LaneCall<'a, P, M>,
    scope: Arc<DirectoryScope>,
    pending: Option<Arc<Conditional>>,
    outcomes: Vec<Result<Progress, Failure>>,
    children: VecDeque<(usize, Child<P::Context>)>,
    kept: bool,
}
type BatchResult = Result<(Progress, bool), Failure>;
type ReadyApplication<'a, P, M> = (&'a Applier<P, M>, PreparedDirectory<<P as SelectionPolicy>::Context>);

pub(crate) fn apply_cohort<P: SelectionPolicy, M: Comparator>(call: &EffectCall<'_>, applications: Vec<ReadyApplication<'_, P, M>>) -> Vec<ApplicationResult> {
    let lanes: Vec<_> = applications.iter().map(|(binding, _)| LaneCall::new(*binding)).collect();
    let mut scopes = Vec::with_capacity(lanes.len());
    let inputs = applications
        .into_iter()
        .zip(&lanes)
        .map(|((binding, batch), lane)| {
            let relative = batch.selection().relative().to_owned();
            let output = binding.target().join(&relative);
            let scope = Arc::new(DirectoryScope::new(&output, batch.selection().entries().map(|e| e.name())));
            scopes.push(scope.clone());
            Input {
                lane,
                batch,
                place: Place {
                    source: Some(binding.original().join(&relative)),
                    target: Some(output.clone()),
                    output,
                    relative,
                },
                scope,
                owner: RecursionOwner::PairedFrontier,
                pending: None,
            }
        })
        .collect();
    let results = batches(call, inputs);
    scopes
        .into_iter()
        .zip(lanes)
        .zip(results)
        .map(|((scope, lane), result)| {
            let complete = result.is_ok() && !lane.stopped.load(Ordering::Acquire);
            let scope = Arc::try_unwrap(scope).ok().expect("joined cohort containing scope");
            lane.finish_accounted(scope.finish(result.map(|(p, _)| p), DirectoryEnd::Retained, complete, call.epoch))
        })
        .collect()
}

fn batches<P: SelectionPolicy, M: Comparator>(call: &EffectCall<'_>, inputs: Vec<Input<'_, P, M>>) -> Vec<BatchResult> {
    let mut staged: Vec<_> = inputs
        .into_iter()
        .map(|input| {
            let Input {
                lane,
                batch,
                place,
                scope,
                owner,
                pending,
            } = input;
            let result = (|| {
                lane.count(&batch, owner == RecursionOwner::PairedFrontier);
                lane.gate(call, &batch, &place.borrowed())?;
                let slots: Vec<_> = (0..batch.selection().len()).map(SlotId).collect();
                lane.planned.fetch_add(
                    slots
                        .iter()
                        .filter(|&&slot| {
                            let entry = batch.selection().entry(slot);
                            entry.has_work() && !conditional(&entry)
                        })
                        .count() as u64,
                    Ordering::Relaxed,
                );
                let prepare = |slot: &SlotId| {
                    if lane.stopped.load(Ordering::Acquire) {
                        return Ok((Progress::default(), true, None));
                    }
                    let entry = batch.selection().entry(*slot);
                    let result = (|| {
                        if entry.has_work()
                            && !conditional(&entry)
                            && let Some(pending) = &pending
                        {
                            pending.materialize(call)?;
                        }
                        lane.prepare_entry(call, &batch, *slot, &place.borrowed(), &scope, owner)
                    })();
                    if result.is_err() {
                        lane.stopped.store(true, Ordering::Release);
                    }
                    result
                };
                let parallel = pending.is_none() && (owner == RecursionOwner::PairedFrontier || place.target.is_none());
                let prepared = lane.map(call, &slots, parallel, prepare);
                Ok((batch.selection().coverage().excluded != 0, prepared))
            })();
            // Only promoted children and finish values survive, never this arena.
            drop(batch);
            let mut state = Staged {
                lane,
                scope,
                pending,
                outcomes: Vec::new(),
                children: VecDeque::new(),
                kept: false,
            };
            match result {
                Err(failure) => {
                    lane.stopped.store(true, Ordering::Release);
                    state.outcomes.push(Err(failure));
                }
                Ok((kept, prepared)) => {
                    state.kept = kept;
                    for result in prepared {
                        let index = state.outcomes.len();
                        state.outcomes.push(match result {
                            Ok((progress, kept, child)) => {
                                state.kept |= kept;
                                if let Some(child) = child {
                                    state.children.push_back((index, child));
                                }
                                Ok(progress)
                            }
                            Err(failure) => Err(failure),
                        });
                    }
                }
            }
            state
        })
        .collect();
    // A lane's promoted jobs remain in native-name order. Join compatible next
    // children across lanes BEFORE any of those children's source inspections.
    while let Some(relative) = staged
        .iter()
        .filter_map(|s| s.children.front().map(|(_, c)| &c.relative))
        .min_by(|a, b| a.as_os_str().as_encoded_bytes().cmp(b.as_os_str().as_encoded_bytes()))
        .cloned()
    {
        let mut owners = Vec::new();
        let mut children = Vec::new();
        for (index, stage) in staged.iter_mut().enumerate() {
            if stage.children.front().is_some_and(|(_, c)| c.relative == relative) {
                let (slot, child) = stage.children.pop_front().unwrap();
                owners.push((index, slot));
                children.push(Job {
                    lane: stage.lane,
                    child,
                    parent: stage.scope.clone(),
                    pending: stage.pending.clone(),
                });
            }
        }
        for ((index, slot), result) in owners.into_iter().zip(run_children(call, children)) {
            let stage = &mut staged[index];
            match result {
                Ok((progress, kept)) => {
                    stage.kept |= kept;
                    kernel::add_progress(&mut stage.outcomes[slot], progress);
                }
                Err(mut failure) => {
                    match std::mem::replace(&mut stage.outcomes[slot], Ok(Progress::default())) {
                        Ok(progress) => failure.completed += progress,
                        Err(mut prior) => {
                            prior.append(failure);
                            failure = prior;
                        }
                    }
                    stage.outcomes[slot] = Err(failure);
                }
            }
        }
    }
    staged.into_iter().map(|s| crate::model::merge(s.outcomes).map(|p| (p, s.kept))).collect()
}

struct Job<'a, P: SelectionPolicy, M: Comparator> {
    lane: &'a LaneCall<'a, P, M>,
    child: Child<P::Context>,
    parent: Arc<DirectoryScope>,
    pending: Option<Arc<Conditional>>,
}
struct Frame<'a, P: SelectionPolicy, M: Comparator> {
    job: Job<'a, P, M>,
    scope: Arc<DirectoryScope>,
    place: Place,
    conditional: Option<Arc<Conditional>>,
    stage: Option<StageSlot>,
    created: bool,
    managed: bool,
    result: Option<BatchResult>,
}
fn run_children<P: SelectionPolicy, M: Comparator>(call: &EffectCall<'_>, jobs: Vec<Job<'_, P, M>>) -> Vec<BatchResult> {
    let mut frames: Vec<_> = jobs
        .into_iter()
        .map(|job| {
            let mut output = job.child.to.clone();
            let mut slot = None;
            let mut created = false;
            let mut scope = None;
            let result = (|| {
                if job.lane.stopped.load(Ordering::Acquire) {
                    return Ok(Some((Progress::default(), true)));
                }
                let descend = match &job.child.work {
                    ChildWork::Create { descend, .. } => {
                        job.parent.ensure_writable(call.epoch)?;
                        *descend
                    }
                    ChildWork::Expand { slot: index, descend, .. } => {
                        slot = Some(job.parent.stage_slot(*index, &job.child.to, call.epoch)?);
                        output = slot.as_ref().unwrap().path().to_owned();
                        *descend
                    }
                    ChildWork::Conditional { .. } => true,
                    ChildWork::Reconcile { .. } if job.child.from.is_some() => true,
                    // There is no shared source input in removal/collapse. Preserve
                    // the existing serial two-pass recursive owner intact.
                    _ => return job.lane.run_child(call, &job.child, &job.parent, None).map(Some),
                };
                let active = Arc::new(DirectoryScope::new(&output, std::iter::empty()));
                scope = Some(active.clone());
                if matches!(job.child.work, ChildWork::Create { .. } | ChildWork::Expand { .. }) {
                    kernel::create_directory(&active, call.epoch)?;
                    created = true;
                    active.ensure_writable(call.epoch)?;
                }
                Ok((!descend).then_some((Progress::default(), false)))
            })();
            let conditional = if let Some(active) = &scope
                && matches!(job.child.work, ChildWork::Conditional { .. })
            {
                Some(Arc::new(Conditional {
                    scope: active.clone(),
                    previous: job.pending.clone(),
                    parent: job.parent.clone(),
                    created: AtomicBool::new(false),
                    needed: AtomicBool::new(false),
                }))
            } else {
                None
            };
            let target = matches!(job.child.work, ChildWork::Reconcile { .. }).then(|| job.child.to.clone());
            let place = Place {
                relative: job.child.relative.clone(),
                source: job.child.from.clone(),
                target,
                output,
            };
            let managed = scope.is_some();
            Frame {
                job,
                scope: scope.unwrap_or_else(|| Arc::new(DirectoryScope::new(&place.output, std::iter::empty()))),
                place,
                conditional,
                stage: slot,
                created,
                managed,
                result: match result {
                    Ok(None) => None,
                    Ok(Some(p)) => Some(Ok(p)),
                    Err(f) => Some(Err(f)),
                },
            }
        })
        .collect();
    while let Some(first) = frames.iter().position(|f| f.result.is_none()) {
        let group: Vec<_> = (first..frames.len())
            .filter(|&i| {
                let f = &frames[i];
                let a = &frames[first];
                f.result.is_none()
                    && f.place.relative == a.place.relative
                    && f.place.source == a.place.source
                    && f.place.target.is_some() == a.place.target.is_some()
            })
            .collect();
        let mut ready = Vec::new();
        for &i in &group {
            let frame = &mut frames[i];
            let route = (|| {
                if let Some(source) = &frame.place.source {
                    kernel::route(call.epoch, source)?;
                }
                if let Some(target) = &frame.place.target {
                    kernel::route(call.epoch, target)?;
                }
                Ok(())
            })();
            match route {
                Ok(()) => ready.push(i),
                Err(f) => frame.result = Some(Err(f)),
            }
        }
        if ready.is_empty() {
            continue;
        }
        let first = &frames[ready[0]];
        let requests: Vec<_> = ready
            .iter()
            .map(|&i| {
                let frame = &frames[i];
                TargetInspection {
                    target: i,
                    logical_root: frame.job.lane.binding.target(),
                    physical_directory: frame.place.target.as_deref(),
                    context: &frame.job.child.interior.as_ref().unwrap().context,
                    policy: &frame.job.lane.binding.selection,
                    comparator: &frame.job.lane.binding.comparator,
                }
            })
            .collect();
        let outcomes = inspect_group_with_work(
            call.source,
            DirectoryRequest {
                relative: &first.place.relative,
                source_directory: first.place.source.as_deref(),
                purpose: if first.place.target.is_some() { Purpose::Compare } else { Purpose::Copy },
                admission: DirectoryAdmission::AlreadyAdmitted,
            },
            &requests,
            &PhysicalExecutionDemand,
            call.epoch,
            call.execution,
            SourceWork::Opaque,
        )
        .expect("internally ordered physical cohort");
        let mut accepted = Vec::new();
        let mut inputs = Vec::new();
        for outcome in outcomes {
            let index = outcome.target;
            let frame = &frames[index];
            match outcome.outcome.map_err(inspection_failure).and_then(|batch| {
                frame.job.lane.gate(call, &batch, &frame.place.borrowed())?;
                Ok(batch)
            }) {
                Err(f) => frames[index].result = Some(Err(f)),
                Ok(batch) => {
                    frame.scope.reserve_names(batch.selection().entries().map(|e| e.name()));
                    accepted.push(index);
                    inputs.push(Input {
                        lane: frame.job.lane,
                        batch,
                        place: Place {
                            relative: frame.place.relative.clone(),
                            source: frame.place.source.clone(),
                            target: frame.place.target.clone(),
                            output: frame.place.output.clone(),
                        },
                        scope: frame.scope.clone(),
                        owner: RecursionOwner::ThisCall,
                        pending: frame.conditional.clone(),
                    });
                }
            }
        }
        for (index, result) in accepted.into_iter().zip(batches(call, inputs)) {
            frames[index].result = Some(result);
        }
    }
    frames.into_iter().map(|frame| finish_child(call, frame)).collect()
}

fn finish_child<P: SelectionPolicy, M: Comparator>(call: &EffectCall<'_>, frame: Frame<'_, P, M>) -> BatchResult {
    let Frame {
        job,
        scope,
        conditional,
        stage,
        created,
        managed,
        mut result,
        ..
    } = frame;
    let lane = job.lane;
    let result = result.take().expect("joined child");
    // Lane-local stop is set BEFORE publication and before any later sibling.
    if result.is_err() {
        lane.stopped.store(true, Ordering::Release);
    }
    if !managed {
        return result;
    }
    let complete = result.is_ok() && !lane.stopped.load(Ordering::Acquire);
    let (end, credit) = match &job.child.work {
        ChildWork::Create { donor, .. } | ChildWork::Expand { donor, .. } if created => (DirectoryEnd::Created { donor: donor.clone() }, true),
        ChildWork::Conditional { donor } => {
            let frame = conditional.as_ref().expect("conditional frame");
            if frame.needed.load(Ordering::Relaxed) {
                lane.planned.fetch_add(1, Ordering::Relaxed);
            }
            if frame.created.load(Ordering::Acquire) {
                (DirectoryEnd::Created { donor: donor.clone() }, true)
            } else {
                (DirectoryEnd::Retained, false)
            }
        }
        _ => (DirectoryEnd::Retained, false),
    };
    drop(conditional);
    let scope = Arc::try_unwrap(scope).ok().expect("joined child scope");
    let mut result = scope.finish(result.map(|(p, _)| p), end, complete, call.epoch);
    if credit {
        kernel::add_progress(
            &mut result,
            Progress {
                directories_created: 1,
                ..Progress::default()
            },
        );
    }
    if let ChildWork::Expand {
        old_kind,
        old_attributes,
        grant,
        ..
    } = &job.child.work
    {
        result = match result {
            Ok(progress) if !lane.stopped.load(Ordering::Acquire) => (|| {
                let removed = kernel::remove_leaf_kind(&job.child.to, *old_kind, old_attributes.as_ref(), call.epoch)?;
                let mut destination = BoundDestination::new(&job.child.to, &job.parent, *grant, Some(*old_kind))?;
                destination.cleared();
                let mut installed = kernel::publish(stage.as_ref().unwrap(), &destination, call.epoch).map(|()| progress);
                kernel::add_progress(&mut installed, removed);
                installed
            })(),
            failed => kernel::discard_private(failed),
        };
    }
    if result.is_err() {
        lane.stopped.store(true, Ordering::Release);
    }
    result.map(|p| (p, matches!(job.child.work, ChildWork::Reconcile { .. })))
}
