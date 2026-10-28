//! Coordinated source-to-many comparison with owned partial-frontier continuation.
//!
//! The observer shares one immediate source acquisition. This module owns only
//! paired traversal, lane-local acceptance and reductions; it never mutates trees.
//!
//! ```no_run
//! use diff_dirs_tree_compare::{Execution, NoVisitor};
//! use diff_dirs_tree_compare::coordinated::{Bindings, Coordinated, TargetBinding, TargetPolicy};
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let bindings = Bindings { source: "/source".into(), targets: vec![TargetBinding {
//!     id: "a".into(), root: "/target".into(), policy: "mirror/v1".into(),
//! }] };
//! let visitor = NoVisitor;
//! let policies = [TargetPolicy { target: &bindings.targets[0].id,
//!     descriptor: &bindings.targets[0].policy, visitor: &visitor }];
//! let mut run = Coordinated::new(bindings.clone(), Execution::Sequential)?;
//! run.bind(&policies)?;
//! let group = run.next_group(&policies)?;
//! let mut restored = Coordinated::from_parts(run.into_parts(), Execution::Sequential)?;
//! let control = restored.current_token(); // new token; no repeated group
//! restored.bind(&policies)?;
//! # let _ = (group, control);
//! # Ok(()) }
//! ```

mod state;

use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::{DirDiff, DirectoryCoverage, DirectoryPlan, Execution, ScanState, SharedBindError, SharedRoots, Summary, TargetVisitor, Visitor};
use state::add;
pub use state::{Bindings, ChildEdge, ControlWindow, GroupFrame, LaneFrame, LaneState, LaneStatus, Parts, TargetBinding};

/// Stable caller identity. Binding order, not the lexical ID, orders publication.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TargetId(pub String);

impl From<&str> for TargetId {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

/// Supplied executable policy and its frozen interpretation. The caller promises
/// the descriptor names all behavior-affecting inputs. Visitors remain pure and
/// may be repeated during inspection; descriptors cannot verify arbitrary code.
pub struct TargetPolicy<'a, V> {
    pub target: &'a TargetId,
    pub descriptor: &'a str,
    pub visitor: &'a V,
}

#[derive(Debug)]
pub enum Error {
    InvalidState(&'static str),
    PolicyMismatch(TargetId),
    InvalidControl(&'static str),
    NotBound,
    CounterOverflow,
    Source(Arc<crate::Error>),
    Target { target: TargetId, error: Arc<crate::Error> },
    Inspection(crate::SharedInputError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidState(reason) => write!(f, "invalid coordinated state: {reason}"),
            Self::PolicyMismatch(target) => write!(f, "missing, duplicate or incompatible policy for {:?}", target.0),
            Self::InvalidControl(reason) => write!(f, "invalid coordinated control: {reason}"),
            Self::NotBound => f.write_str("bind coordinated runtime before inspection"),
            Self::CounterOverflow => f.write_str("coordinated comparison counter overflow"),
            Self::Source(error) => write!(f, "bind source route: {error}"),
            Self::Target { target, error } => write!(f, "bind target {:?} route: {error}", target.0),
            Self::Inspection(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Source(error) | Self::Target { error, .. } => Some(error.as_ref()),
            Self::Inspection(error) => Some(error),
            _ => None,
        }
    }
}

/// Runtime-local control capability. Private identity prevents foreign tokens.
/// from_parts() creates a fresh identity; obtain its reissued current_token().
/// Tokens are deliberately not part of the application transport schema.
#[derive(Clone, Debug)]
pub struct GroupToken {
    instance: Arc<()>,
    sequence: u64,
}

impl PartialEq for GroupToken {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.instance, &other.instance) && self.sequence == other.sequence
    }
}
impl Eq for GroupToken {}

/// One successful immediate assessment plus optional caller-owned lowering.
/// Keeping a DirectoryPlan in value deliberately retains facts outside Parts.
#[derive(Debug)]
pub struct Compared<T = ()> {
    pub diff: DirDiff,
    pub coverage: DirectoryCoverage,
    pub structural_root: bool,
    pub value: T,
}

#[derive(Debug)]
pub struct TargetOutcome<T = ()> {
    pub target: TargetId,
    pub outcome: Result<Compared<T>, Arc<crate::Error>>,
}

/// Complete joined directory group, including clean outcomes and lane failures.
/// An absent target did not participate; it is not implicitly clean or absent.
#[derive(Debug)]
pub struct ComparedGroup<T = ()> {
    pub token: GroupToken,
    pub relative: PathBuf,
    pub outcomes: Vec<TargetOutcome<T>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Head {
    Root,
    Control { relative: PathBuf },
    Child { relative: PathBuf },
    Return { relative: PathBuf },
    Terminal(LaneStatus),
}

struct Runtime {
    roots: SharedRoots,
    /// Compact inspector index -> original binding index. Never saved/reordered.
    original: Vec<usize>,
}

/// Data and runtime have separate lifetimes. Construction/restoration validate
/// data only; bind() checks lent policy descriptors before tree access.
/// Inter-group traversal is sequential. Execution controls metadata work inside
/// each joined group, with no interpretation as J/K or a width/byte bound.
pub struct Coordinated {
    parts: Parts,
    execution: Execution,
    runtime: Option<Runtime>,
    bound: bool,
    instance: Arc<()>,
    route_metadata_checks: u64,
}

impl Coordinated {
    pub fn new(bindings: Bindings, execution: Execution) -> Result<Self, Error> {
        Self::from_parts(Parts::fresh(bindings)?, execution)
    }

    /// Data-only. Only live lanes require executable policies, in any supplied
    /// order; terminated lanes retain their original identities and indices.
    pub fn from_parts(parts: Parts, execution: Execution) -> Result<Self, Error> {
        parts.validate()?;
        Ok(Self {
            parts,
            execution,
            runtime: None,
            bound: false,
            instance: Arc::new(()),
            route_metadata_checks: 0,
        })
    }

    fn validate_policies<V: Visitor>(&self, policies: &[TargetPolicy<'_, V>]) -> Result<(), Error> {
        let mut seen = HashSet::new();
        for policy in policies {
            let target = self.parts.bindings.targets.iter().position(|t| t.id == *policy.target);
            if target.is_none_or(|i| self.parts.bindings.targets[i].policy != policy.descriptor) || !seen.insert(policy.target) {
                return Err(Error::PolicyMismatch(policy.target.clone()));
            }
        }
        for (i, target) in self.parts.bindings.targets.iter().enumerate() {
            if self.parts.lanes[i].status == LaneStatus::Active && !seen.contains(&target.id) {
                return Err(Error::PolicyMismatch(target.id.clone()));
            }
        }
        Ok(())
    }

    pub fn parts(&self) -> &Parts {
        &self.parts
    }

    /// Invocation-local extra no-follow routing checks, not listing/entry fact
    /// acquisitions or the inspector's mandatory binding-root observations.
    /// Restoration starts a new counter; this diagnostic is never saved.
    pub fn route_metadata_checks(&self) -> u64 {
        self.route_metadata_checks
    }

    /// Preserve every partial ancestor, suffix, membership and open window.
    /// Never projects/finalizes active ancestor reductions or performs I/O.
    pub fn into_parts(self) -> Parts {
        self.parts
    }

    /// Structural validation only, with no visitor callbacks or interior listing.
    /// Failed/stopped/completed roots are not rebound. Terminal runs do no I/O.
    pub fn bind<V: Visitor>(&mut self, policies: &[TargetPolicy<'_, V>]) -> Result<(), Error> {
        self.parts.validate()?;
        self.validate_policies(policies)?;
        self.runtime = None;
        self.bound = false;
        let original: Vec<_> = self
            .parts
            .lanes
            .iter()
            .enumerate()
            .filter_map(|(i, l)| (l.status == LaneStatus::Active).then_some(i))
            .collect();
        if original.is_empty() {
            self.runtime = None;
            self.bound = true;
            return Ok(());
        }
        let targets = original.iter().map(|&i| &self.parts.bindings.targets[i].root);
        let roots = SharedRoots::new(&self.parts.bindings.source, targets).map_err(|error| match error {
            SharedBindError::NoTargets => Error::InvalidState("no live target bindings"),
            SharedBindError::Source(error) => Error::Source(Arc::new(error)),
            SharedBindError::Target { target, error } => Error::Target {
                target: self.parts.bindings.targets[original[target]].id.clone(),
                error: Arc::new(error),
            },
        })?;
        let mut checked = Routes::default();
        checked.results.insert(self.parts.bindings.source.clone(), Ok(()));
        for &target in &original {
            checked.results.insert(self.parts.bindings.targets[target].root.clone(), Ok(()));
        }
        let result = (|| {
            for frame in &self.parts.frames {
                for lane in &frame.lanes {
                    self.validate_route(&frame.relative, lane.target, &mut checked)?;
                }
            }
            Ok(())
        })();
        self.route_metadata_checks = add(self.route_metadata_checks, checked.checks)?;
        result?;
        self.runtime = Some(Runtime { roots, original });
        self.bound = true;
        Ok(())
    }

    /// The restored open window receives a new runtime-local token. No previously
    /// accepted group is delivered again. Old/foreign tokens always refuse.
    pub fn current_token(&self) -> Option<GroupToken> {
        self.parts.window.as_ref().map(|window| GroupToken {
            instance: Arc::clone(&self.instance),
            sequence: window.sequence,
        })
    }

    /// Close the current control window and fold only already-ready returns.
    /// Performs no observation, policy callback or filesystem access. Effectful
    /// compositions call this only after disposing their current applications;
    /// it permits a checkpoint after child return but before a queued sibling's
    /// next inspection, without duplicating this frontier's reduction owner.
    pub fn settle(&mut self) {
        self.parts.window = None;
        self.settle_returns();
    }

    /// Observe exactly one next directory group, including a clean/empty group.
    /// None drains only remaining pure returns and marks eligible lanes complete.
    pub fn next_group<V: Visitor>(&mut self, policies: &[TargetPolicy<'_, V>]) -> Result<Option<ComparedGroup>, Error> {
        self.next_group_with(policies, |_, _| ())
    }

    /// Same traversal/acceptance, with access to each complete validated plan.
    /// Lowering is a pure projection, never effect acceptance. It may retain an
    /// ephemeral plan in T for a sync composition; Parts never retain T/facts.
    /// The callback order is unspecified and may precede a later lane error.
    pub fn next_group_with<V: Visitor, T>(
        &mut self,
        policies: &[TargetPolicy<'_, V>],
        mut lower: impl FnMut(&TargetId, DirectoryPlan) -> T,
    ) -> Result<Option<ComparedGroup<T>>, Error> {
        self.validate_policies(policies)?;
        if self.parts.lanes.iter().any(|l| l.status == LaneStatus::Active) && !self.bound {
            return Err(Error::NotBound);
        }
        self.parts.window = None;
        self.settle_returns();
        let (relative, members) = if self.parts.root_pending {
            (
                PathBuf::new(),
                self.parts
                    .lanes
                    .iter()
                    .enumerate()
                    .filter_map(|(i, l)| (l.status == LaneStatus::Active).then_some(i))
                    .collect::<Vec<_>>(),
            )
        } else if let Some(frame) = self.parts.frames.last() {
            let edge = frame.remaining.front().expect("settled frontier has an edge");
            (frame.relative.join(&edge.name), edge.members.clone())
        } else {
            return Ok(None);
        };
        if members.is_empty() {
            self.parts.root_pending = false;
            return Ok(None);
        }
        let next_sequence = add(self.parts.next_sequence, 1)?;
        // A route failure is a lane outcome, not a global abort. In particular a
        // replaced target ancestor must not redirect inspection through a link.
        let mut route_errors = Vec::new();
        let mut checked = Routes::default();
        for &target in &members {
            let error = match self.validate_route(&relative, target, &mut checked) {
                Ok(()) => None,
                Err(Error::Source(error) | Error::Target { error, .. }) => Some(error),
                Err(error) => return Err(error),
            };
            route_errors.push(error);
        }
        self.route_metadata_checks = add(self.route_metadata_checks, checked.checks)?;
        let runtime = self.runtime.as_ref().ok_or(Error::NotBound)?;
        let requests: Vec<_> = members
            .iter()
            .zip(&route_errors)
            .filter(|(_, error)| error.is_none())
            .map(|(target, _)| {
                let index = runtime.original.binary_search(target).expect("live lane was bound");
                let id = &self.parts.bindings.targets[*target].id;
                let policy = policies.iter().find(|p| p.target == id).expect("live policy was validated");
                TargetVisitor {
                    target: index,
                    visitor: policy.visitor,
                }
            })
            .collect();
        let outcomes = runtime
            .roots
            .inspect_with(&relative, &requests, self.execution, |index, plan| {
                let target = runtime.original[index];
                let local = Local {
                    diff: plan.diff().clone(),
                    coverage: DirectoryCoverage {
                        excluded: plan.excluded(),
                        scoped: plan.steered(),
                        planned: plan.work(),
                        retained: plan.retained(),
                    },
                    children: plan.paired_children().map(std::ffi::OsStr::to_os_string).collect(),
                    root: plan.operations().is_root(),
                };
                (local, lower(&self.parts.bindings.targets[target].id, plan))
            })
            .map_err(Error::Inspection)?;
        let mut inspected = outcomes.into_iter();
        let outcomes: Vec<_> = members
            .iter()
            .zip(route_errors)
            .map(|(&target, error)| {
                if let Some(error) = error {
                    crate::TargetComparison {
                        target: runtime.original.binary_search(&target).expect("bound lane"),
                        outcome: Err(error),
                    }
                } else {
                    inspected.next().expect("requested lane outcome")
                }
            })
            .collect();
        // All fallible arithmetic precedes accepting any lane's group outcome.
        for outcome in &outcomes {
            if let Ok((local, _)) = &outcome.outcome {
                preflight(&self.parts.lanes[runtime.original[outcome.target]].summary, local)?;
            }
        }
        let sequence = self.parts.next_sequence;
        let mut lanes = Vec::new();
        let mut children = BTreeMap::<std::ffi::OsString, Vec<usize>>::new();
        let mut published = Vec::with_capacity(outcomes.len());
        let mut failed = Vec::new();
        for outcome in outcomes {
            let target = runtime.original[outcome.target];
            let id = self.parts.bindings.targets[target].id.clone();
            let outcome = match outcome.outcome {
                Err(error) => {
                    failed.push(target);
                    Err(error)
                }
                Ok((local, value)) => {
                    let changed = !local.diff.is_empty();
                    let ancestor = self
                        .parts
                        .frames
                        .last()
                        .and_then(|f| f.lanes.iter().find(|l| l.target == target))
                        .is_some_and(|l| l.direct_ancestor);
                    let summary = &mut self.parts.lanes[target].summary;
                    summary.examined_directories += 1;
                    summary.counts.add(&local.diff);
                    summary.excluded += local.coverage.excluded;
                    summary.scoped |= local.coverage.scoped;
                    summary.planned += local.coverage.planned;
                    summary.retained += local.coverage.retained;
                    if changed && !ancestor {
                        summary.topmost_divergent.push(relative.clone());
                    }
                    lanes.push(LaneFrame {
                        target,
                        direct_changed: changed,
                        direct_ancestor: ancestor || changed,
                        subtree_changed: changed,
                    });
                    for name in local.children {
                        children.entry(name).or_default().push(target);
                    }
                    Ok(Compared {
                        diff: local.diff,
                        coverage: local.coverage,
                        structural_root: local.root,
                        value,
                    })
                }
            };
            published.push(TargetOutcome { target: id, outcome });
        }
        self.parts.root_pending = false;
        self.parts.next_sequence = next_sequence;
        let successful = lanes.iter().map(|l| l.target).collect();
        let mut remaining: Vec<_> = children.into_iter().map(|(name, members)| ChildEdge { name, members }).collect();
        remaining.sort_unstable_by(|a, b| a.name.as_encoded_bytes().cmp(b.name.as_encoded_bytes()));
        self.parts.frames.push(GroupFrame {
            sequence,
            relative: relative.clone(),
            lanes,
            remaining: remaining.into(),
        });
        self.parts.window = Some(ControlWindow {
            sequence,
            successful,
            pruned: Vec::new(),
        });
        // Preserve the current incoming edge until this group's return, even if
        // every participant failed. Remove failed lanes from all future suffixes.
        for target in failed {
            self.terminate(target, LaneStatus::Failed);
        }
        Ok(Some(ComparedGroup {
            token: self.current_token().expect("published window"),
            relative,
            outcomes: published,
        }))
    }

    /// Prune only selected successful participants of the current group. Validate
    /// the entire subset before changing memberships/coverage; repeated members
    /// and stale/foreign/failed targets refuse without any partial effect.
    pub fn prune_here(&mut self, token: &GroupToken, targets: &[TargetId]) -> Result<(), Error> {
        if self.current_token().as_ref() != Some(token) {
            return Err(Error::InvalidControl("stale or foreign group token"));
        }
        let indices = self.control_targets(targets)?;
        let window = self.parts.window.as_ref().expect("token checked");
        if indices
            .iter()
            .any(|i| window.successful.binary_search(i).is_err() || window.pruned.binary_search(i).is_ok())
        {
            return Err(Error::InvalidControl("target failed, stopped or already pruned"));
        }
        let frame = self.parts.frames.last_mut().expect("open window frame");
        for &target in &indices {
            for edge in &mut frame.remaining {
                edge.members.retain(|&i| i != target);
            }
            self.parts.lanes[target].summary.pruned.push(frame.relative.clone());
        }
        frame.remaining.retain(|e| !e.members.is_empty());
        let window = self.parts.window.as_mut().expect("open window");
        window.pruned.extend(indices);
        window.pruned.sort_unstable();
        Ok(())
    }

    /// Deliberately abandon live lanes, removing all future memberships. Their
    /// already accepted prefix is retained as incomplete; no callback or I/O runs.
    pub fn stop_lanes(&mut self, targets: &[TargetId]) -> Result<(), Error> {
        let indices = self.control_targets(targets)?;
        if indices.iter().any(|&i| self.parts.lanes[i].status != LaneStatus::Active) {
            return Err(Error::InvalidControl("target is not active"));
        }
        for target in indices {
            self.terminate(target, LaneStatus::Stopped);
        }
        Ok(())
    }

    /// Nonmutating prefix projection, including known changes in partial active
    /// ancestors. These projected closures are never saved back into Parts.
    pub fn prefix_summaries(&self) -> Vec<(TargetId, Summary)> {
        self.parts
            .lanes
            .iter()
            .enumerate()
            .map(|(target, lane)| {
                let mut summary = lane.summary.clone();
                let mut changed = false;
                for frame in self.parts.frames.iter().rev() {
                    if let Some(lane) = frame.lanes.iter().find(|l| l.target == target) {
                        changed |= lane.subtree_changed;
                        if changed {
                            summary.changed_dirs.push(frame.relative.clone());
                        }
                    }
                }
                (self.parts.bindings.targets[target].id.clone(), summary)
            })
            .collect()
    }

    /// Diagnostic view of every destination head, including lanes waiting above
    /// the global stack top. No second authoritative cursor is maintained.
    pub fn heads(&self) -> Vec<(TargetId, Head)> {
        self.parts
            .lanes
            .iter()
            .enumerate()
            .map(|(target, lane)| {
                let head = if lane.status != LaneStatus::Active {
                    Head::Terminal(lane.status)
                } else if self.parts.root_pending {
                    Head::Root
                } else {
                    let frame = self
                        .parts
                        .frames
                        .iter()
                        .rev()
                        .find(|f| f.lanes.iter().any(|l| l.target == target))
                        .expect("active lane has a head");
                    if self.parts.window.as_ref().is_some_and(|w| w.sequence == frame.sequence) {
                        Head::Control {
                            relative: frame.relative.clone(),
                        }
                    } else if let Some(edge) = frame.remaining.iter().find(|e| e.members.contains(&target)) {
                        Head::Child {
                            relative: frame.relative.join(&edge.name),
                        }
                    } else {
                        Head::Return {
                            relative: frame.relative.clone(),
                        }
                    }
                };
                (self.parts.bindings.targets[target].id.clone(), head)
            })
            .collect()
    }

    fn control_targets(&self, targets: &[TargetId]) -> Result<Vec<usize>, Error> {
        let mut indices = Vec::with_capacity(targets.len());
        for id in targets {
            let index = self
                .parts
                .bindings
                .targets
                .iter()
                .position(|t| t.id == *id)
                .ok_or(Error::InvalidControl("unknown target"))?;
            if indices.contains(&index) {
                return Err(Error::InvalidControl("duplicate target in subset"));
            }
            indices.push(index);
        }
        Ok(indices)
    }

    fn settle_returns(&mut self) {
        loop {
            let Some(frame) = self.parts.frames.last() else {
                return;
            };
            let ready: Vec<_> = frame
                .lanes
                .iter()
                .filter(|l| frame.remaining.iter().all(|e| !e.members.contains(&l.target)))
                .map(|l| l.target)
                .collect();
            for target in ready {
                self.close_lane(target);
            }
            if self.parts.frames.last().is_some_and(|f| f.lanes.is_empty()) {
                let closed = self.parts.frames.pop().expect("empty frame");
                if let Some(parent) = self.parts.frames.last_mut() {
                    let edge = parent.remaining.pop_front().expect("return edge");
                    debug_assert_eq!(parent.relative.join(edge.name), closed.relative);
                    debug_assert!(edge.members.is_empty());
                }
            } else {
                return;
            }
        }
    }

    fn close_lane(&mut self, target: usize) {
        let depth = self.parts.frames.len() - 1;
        let frame = &mut self.parts.frames[depth];
        let position = frame.lanes.iter().position(|l| l.target == target).expect("closing live lane");
        let lane = frame.lanes.remove(position);
        if lane.subtree_changed {
            self.parts.lanes[target].summary.changed_dirs.push(frame.relative.clone());
        }
        if depth == 0 {
            self.parts.lanes[target].status = LaneStatus::Complete;
            self.parts.lanes[target].summary.state = ScanState::Complete;
        } else {
            let parent = &mut self.parts.frames[depth - 1];
            if lane.subtree_changed {
                parent.lanes.iter_mut().find(|l| l.target == target).expect("parent lane").subtree_changed = true;
            }
            parent.remaining.front_mut().expect("active return edge").members.retain(|&i| i != target);
        }
    }

    fn terminate(&mut self, target: usize, status: LaneStatus) {
        let mut changed = false;
        for depth in (0..self.parts.frames.len()).rev() {
            let active_child = depth + 1 < self.parts.frames.len();
            let frame = &mut self.parts.frames[depth];
            if let Some(position) = frame.lanes.iter().position(|l| l.target == target) {
                changed |= frame.lanes.remove(position).subtree_changed;
                if changed {
                    self.parts.lanes[target].summary.changed_dirs.push(frame.relative.clone());
                }
            }
            for edge in &mut frame.remaining {
                edge.members.retain(|&i| i != target);
            }
            let mut position = 0;
            frame.remaining.retain(|e| {
                let keep = !e.members.is_empty() || (position == 0 && active_child);
                position += 1;
                keep
            });
        }
        if let Some(window) = &mut self.parts.window {
            window.successful.retain(|&i| i != target);
            window.pruned.retain(|&i| i != target);
        }
        self.parts.lanes[target].status = status;
        self.parts.lanes[target].summary.state = if status == LaneStatus::Failed {
            ScanState::Failed
        } else {
            ScanState::Incomplete
        };
        if self.parts.lanes.iter().all(|l| l.status != LaneStatus::Active) && self.parts.root_pending {
            self.parts.root_pending = false;
        }
    }

    fn validate_route(&self, relative: &Path, target: usize, checked: &mut Routes) -> Result<(), Error> {
        for (root, source) in [(&self.parts.bindings.source, true), (&self.parts.bindings.targets[target].root, false)] {
            let mut path = root.clone();
            for suffix in std::iter::once(None).chain(relative.components().map(Some)) {
                if let Some(component) = suffix {
                    path.push(component);
                }
                if let Err(error) = checked.check(&path)? {
                    return Err(if source {
                        Error::Source(error)
                    } else {
                        Error::Target {
                            target: self.parts.bindings.targets[target].id.clone(),
                            error,
                        }
                    });
                }
            }
        }
        Ok(())
    }
}

#[derive(Default)]
struct Routes {
    results: HashMap<PathBuf, Result<(), Arc<crate::Error>>>,
    checks: u64,
}

impl Routes {
    fn check(&mut self, path: &Path) -> Result<Result<(), Arc<crate::Error>>, Error> {
        if let Some(result) = self.results.get(path) {
            return Ok(result.clone());
        }
        self.checks = add(self.checks, 1)?;
        let result = std::fs::symlink_metadata(path)
            .map_err(|source| crate::Error::Io {
                path: path.to_path_buf(),
                operation: "validate coordinated route",
                source,
            })
            .and_then(|m| {
                if m.is_dir() {
                    Ok(())
                } else {
                    Err(crate::Error::NotDirectory { path: path.to_path_buf() })
                }
            })
            .map_err(Arc::new);
        self.results.insert(path.to_path_buf(), result.clone());
        Ok(result)
    }
}

struct Local {
    diff: DirDiff,
    coverage: DirectoryCoverage,
    children: Vec<std::ffi::OsString>,
    root: bool,
}

fn preflight(summary: &Summary, local: &Local) -> Result<(), Error> {
    add(summary.examined_directories, 1)?;
    add(summary.excluded, local.coverage.excluded)?;
    add(summary.planned, local.coverage.planned)?;
    add(summary.retained, local.coverage.retained)?;
    let mut total = 0;
    for (before, names) in [
        (summary.counts.files.changed, &local.diff.files.changed),
        (summary.counts.files.added, &local.diff.files.added),
        (summary.counts.files.removed, &local.diff.files.removed),
        (summary.counts.dirs.added, &local.diff.dirs.added),
        (summary.counts.dirs.removed, &local.diff.dirs.removed),
    ] {
        total = add(total, add(before, u64::try_from(names.len()).map_err(|_| Error::CounterOverflow)?)?)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests;
