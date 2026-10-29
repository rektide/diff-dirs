//! Coordinated source-to-many comparison with owned partial-frontier continuation.
//!
//! The observer shares one immediate source acquisition. This module owns only
//! paired traversal, lane-local acceptance and reductions; it never mutates trees.
//!
//! ```no_run
//! use diff_dirs_tree_compare::{Execution, NoVisitor};
//! use diff_dirs_tree_compare::coordinated::{Bindings, Coordinated, TargetBinding, TargetPolicy};
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let bindings = Bindings { source: "/source".into(), targets: vec![TargetBinding::full(
//!     "a".into(), "/target".into(), "mirror/v1".into(),
//! )] };
//! let visitor = NoVisitor;
//! let policies = [TargetPolicy::full(&bindings.targets[0].id,
//!     &bindings.targets[0].policy, &visitor)];
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

use std::collections::{BTreeMap, HashSet};
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::{
    Comparator, ContextSite, DirDiff, DirectoryAdmission, DirectoryCoverage, DirectoryRequest, Execution, ExecutionDemand, FullVisitor, InspectionEpoch,
    InteriorRole, NoExecutionDemand, Policy, PreparedDirectory, Purpose, ScanState, SemanticId, SourceSession, StatKey, Summary, TargetInspection, Visitor,
    inspect_group,
};
use state::add;
pub use state::{Bindings, ChildEdge, ChildMember, ControlWindow, GroupFrame, LaneFrame, LaneState, LaneStatus, Parts, TargetBinding};

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
pub struct TargetPolicy<'a, P, M = StatKey> {
    pub target: &'a TargetId,
    pub descriptor: &'a str,
    pub policy: P,
    pub comparator: M,
}

impl<'a, V: Visitor> TargetPolicy<'a, FullVisitor<crate::VisitorRef<'a, V>>> {
    pub fn full(target: &'a TargetId, descriptor: &'a str, visitor: &'a V) -> Self {
        Self {
            target,
            descriptor,
            policy: FullVisitor(visitor.borrowed()),
            comparator: StatKey,
        }
    }
}

#[derive(Debug)]
pub enum Error {
    InvalidState(&'static str),
    PolicyMismatch(TargetId),
    Context {
        target: TargetId,
        relative: PathBuf,
        cause: crate::CheckError,
    },
    InvalidControl(&'static str),
    NotBound,
    CounterOverflow,
    Source(Arc<crate::Error>),
    Target {
        target: TargetId,
        error: Arc<crate::Error>,
    },
    Inspection(crate::SharedInputError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidState(reason) => write!(f, "invalid coordinated state: {reason}"),
            Self::PolicyMismatch(target) => write!(f, "missing, duplicate or incompatible policy for {:?}", target.0),
            Self::Context { target, relative, cause } => write!(f, "invalid context for {:?} at {relative:?}: {cause}", target.0),
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
            Self::Context { cause, .. } => Some(cause.as_ref()),
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
/// Keeping a PreparedDirectory in value deliberately retains facts outside Parts.
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
    source: Arc<SourceSession>,
}

/// Data and runtime have separate lifetimes. Construction/restoration validate
/// data only; bind() checks lent policy descriptors before tree access.
/// Inter-group traversal is sequential. Execution controls metadata work inside
/// each joined group, with no interpretation as J/K or a width/byte bound.
pub struct Coordinated<C = ()> {
    parts: Parts<C>,
    execution: Execution,
    runtime: Option<Runtime>,
    bound: bool,
    instance: Arc<()>,
    route_metadata_checks: u64,
}

impl<C: Clone + Eq + fmt::Debug + Send + Sync> Coordinated<C> {
    pub fn new(bindings: Bindings<C>, execution: Execution) -> Result<Self, Error> {
        Self::from_parts(Parts::fresh(bindings)?, execution)
    }

    /// Data-only. Only live lanes require executable policies, in any supplied
    /// order; terminated lanes retain their original identities and indices.
    pub fn from_parts(parts: Parts<C>, execution: Execution) -> Result<Self, Error> {
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

    pub fn validate_policies<P: Policy<Context = C>, M: Comparator>(&self, policies: &[TargetPolicy<'_, P, M>]) -> Result<(), Error> {
        let mut seen = HashSet::new();
        for policy in policies {
            let target = self.parts.bindings.targets.iter().position(|t| t.id == *policy.target);
            if target.is_none_or(|i| {
                let bound = &self.parts.bindings.targets[i];
                bound.policy != policy.descriptor
                    || bound.policy_id != *policy.policy.identity()
                    || bound.context_schema != *policy.policy.context_schema()
                    || bound.comparator != *policy.comparator.identity()
                    || bound.initial_context != policy.policy.initial_context()
            }) || !seen.insert(policy.target)
            {
                return Err(Error::PolicyMismatch(policy.target.clone()));
            }
        }
        for (i, target) in self.parts.bindings.targets.iter().enumerate() {
            if self.parts.lanes[i].status == LaneStatus::Active && !seen.contains(&target.id) {
                return Err(Error::PolicyMismatch(target.id.clone()));
            }
        }
        // Pure interpretation checks precede root/route access. No admission or
        // accepted ancestor callbacks are replayed here.
        for policy in policies {
            let target = self
                .parts
                .bindings
                .targets
                .iter()
                .position(|t| t.id == *policy.target)
                .expect("validated target");
            let validate = |relative: &Path, structural_root, role, context: &C| {
                policy
                    .policy
                    .validate_context(
                        ContextSite {
                            relative,
                            structural_root,
                            role,
                        },
                        context,
                    )
                    .map_err(|cause| Error::Context {
                        target: policy.target.clone(),
                        relative: relative.to_owned(),
                        cause,
                    })
            };
            validate(Path::new(""), true, None, &self.parts.bindings.targets[target].initial_context)?;
            for frame in &self.parts.frames {
                if let Some(lane) = frame.lanes.iter().find(|l| l.target == target) {
                    validate(&frame.relative, false, None, &lane.context)?;
                }
                for edge in &frame.remaining {
                    if let Some(member) = edge.members.iter().find(|m| m.target == target) {
                        validate(&frame.relative.join(&edge.name), false, Some(InteriorRole::Paired), &member.context)?;
                    }
                }
            }
        }
        Ok(())
    }

    pub fn parts(&self) -> &Parts<C> {
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
    pub fn into_parts(self) -> Parts<C> {
        self.parts
    }

    /// Structural validation only, with no visitor callbacks or interior listing.
    /// Failed/stopped/completed roots are not rebound. Terminal runs do no I/O.
    pub fn bind<P: Policy<Context = C>, M: Comparator>(&mut self, policies: &[TargetPolicy<'_, P, M>]) -> Result<(), Error> {
        self.bind_with_source(policies, Arc::new(SourceSession::new(&self.parts.bindings.source)))
    }

    /// Use the same runtime source/resources owner as recursive execution. This
    /// handle is not a route certificate or a persisted source generation.
    pub fn bind_with_source<P: Policy<Context = C>, M: Comparator>(
        &mut self,
        policies: &[TargetPolicy<'_, P, M>],
        source: Arc<SourceSession>,
    ) -> Result<(), Error> {
        self.parts.validate()?;
        self.validate_policies(policies)?;
        if source.root() != self.parts.bindings.source {
            return Err(Error::InvalidState("source session root mismatch"));
        }
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
        let mut checked = Routes::default();
        let result = (|| {
            for &target in &original {
                self.validate_route(Path::new(""), target, &mut checked)?;
            }
            // Mandatory binding observations are not extra route checks.
            checked.checks = 0;
            for frame in &self.parts.frames {
                for lane in &frame.lanes {
                    self.validate_route(&frame.relative, lane.target, &mut checked)?;
                }
            }
            Ok(())
        })();
        self.route_metadata_checks = add(self.route_metadata_checks, checked.checks)?;
        result?;
        self.runtime = Some(Runtime { source });
        self.bound = true;
        Ok(())
    }

    pub fn source_session(&self) -> Option<&Arc<SourceSession>> {
        self.runtime.as_ref().map(|r| &r.source)
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
    pub fn next_group<P: Policy<Context = C>, M: Comparator>(&mut self, policies: &[TargetPolicy<'_, P, M>]) -> Result<Option<ComparedGroup>, Error> {
        self.next_group_with(policies, |_, _| ())
    }

    /// Same traversal/acceptance, with access to each validated live preparation.
    /// Lowering is a pure projection, never effect acceptance. It may retain an
    /// ephemeral plan in T for a sync composition; Parts never retain T/facts.
    /// The callback order is unspecified and may precede a later lane error.
    pub fn next_group_with<P: Policy<Context = C>, M: Comparator, T>(
        &mut self,
        policies: &[TargetPolicy<'_, P, M>],
        lower: impl FnMut(&TargetId, PreparedDirectory<C>) -> T,
    ) -> Result<Option<ComparedGroup<T>>, Error> {
        self.next_group_prepared(policies, &NoExecutionDemand, lower)
    }

    /// Finalize and acquire selected execution inputs before initial publication.
    /// No later executor may substitute fallback observations for this boundary.
    /// Lowering remains a pure projection; a preparation is not an external
    /// effect binding or permission to choose arbitrary physical output routes.
    pub fn next_group_prepared<P: Policy<Context = C>, M: Comparator, E: ExecutionDemand, T>(
        &mut self,
        policies: &[TargetPolicy<'_, P, M>],
        demand: &E,
        mut lower: impl FnMut(&TargetId, PreparedDirectory<C>) -> T,
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
                    .filter(|(_, l)| l.status == LaneStatus::Active)
                    .map(|(i, _)| ChildMember {
                        target: i,
                        context: self.parts.bindings.targets[i].initial_context.clone(),
                    })
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
        for member in &members {
            let error = match self.validate_route(&relative, member.target, &mut checked) {
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
            .map(|(member, _)| {
                let id = &self.parts.bindings.targets[member.target].id;
                let policy = policies.iter().find(|p| p.target == id).expect("live policy was validated");
                TargetInspection {
                    target: member.target,
                    logical_root: &self.parts.bindings.targets[member.target].root,
                    physical_directory: None,
                    context: &member.context,
                    policy: &policy.policy,
                    comparator: &policy.comparator,
                }
            })
            .collect();
        let directories: Vec<_> = requests.iter().map(|r| r.logical_root.join(&relative)).collect();
        let requests: Vec<_> = requests
            .into_iter()
            .zip(&directories)
            .map(|(mut r, path)| {
                r.physical_directory = Some(path);
                r
            })
            .collect();
        let source_directory = self.parts.bindings.source.join(&relative);
        let outcomes = inspect_group(
            &runtime.source,
            DirectoryRequest {
                relative: &relative,
                source_directory: Some(&source_directory),
                purpose: Purpose::Compare,
                admission: if self.parts.root_pending {
                    DirectoryAdmission::RootGated
                } else {
                    DirectoryAdmission::AlreadyAdmitted
                },
            },
            &requests,
            demand,
            &checked.epoch,
            self.execution,
        )
        .map_err(Error::Inspection)?;
        let outcomes: Vec<_> = outcomes
            .into_iter()
            .map(|outcome| crate::TargetComparison {
                target: outcome.target,
                outcome: outcome.outcome.map(|plan| {
                    let selected = plan.selection();
                    let local = Local {
                        diff: selected.diff().clone(),
                        coverage: selected.coverage(),
                        comparator: selected.comparator().clone(),
                        context: selected.input_context().clone(),
                        children: selected
                            .paired_children()
                            .map(|entry| (entry.name().to_owned(), entry.interior.as_ref().expect("paired").context.clone()))
                            .collect(),
                        root: selected.is_root(),
                    };
                    (local, lower(&self.parts.bindings.targets[outcome.target].id, plan))
                }),
            })
            .collect();
        let mut inspected = outcomes.into_iter();
        let outcomes: Vec<_> = members
            .iter()
            .zip(route_errors)
            .map(|(member, error)| {
                if let Some(error) = error {
                    crate::TargetComparison {
                        target: member.target,
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
                preflight(&self.parts.lanes[outcome.target].summary, local)?;
            }
        }
        let sequence = self.parts.next_sequence;
        let mut lanes = Vec::new();
        let mut children = BTreeMap::<std::ffi::OsString, Vec<ChildMember<C>>>::new();
        let mut published = Vec::with_capacity(outcomes.len());
        let mut failed = Vec::new();
        for outcome in outcomes {
            let target = outcome.target;
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
                    summary.add_coverage(local.coverage, &local.comparator);
                    if changed && !ancestor {
                        summary.topmost_divergent.push(relative.clone());
                    }
                    lanes.push(LaneFrame {
                        target,
                        context: local.context,
                        direct_changed: changed,
                        direct_ancestor: ancestor || changed,
                        subtree_changed: changed,
                    });
                    for (name, context) in local.children {
                        children.entry(name).or_default().push(ChildMember { target, context });
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
                edge.members.retain(|m| m.target != target);
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
                    } else if let Some(edge) = frame.remaining.iter().find(|e| e.members.iter().any(|m| m.target == target)) {
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
                .filter(|l| frame.remaining.iter().all(|e| e.members.iter().all(|m| m.target != l.target)))
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
            parent.remaining.front_mut().expect("active return edge").members.retain(|m| m.target != target);
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
                edge.members.retain(|m| m.target != target);
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
    epoch: InspectionEpoch,
    seen: HashSet<PathBuf>,
    checks: u64,
}

impl Routes {
    fn check(&mut self, path: &Path) -> Result<Result<(), Arc<crate::Error>>, Error> {
        if self.seen.insert(path.to_owned()) {
            self.checks = add(self.checks, 1)?;
        }
        Ok(self.epoch.directory(path).map(|_| ()))
    }
}

struct Local<C> {
    diff: DirDiff,
    coverage: DirectoryCoverage,
    comparator: SemanticId,
    context: C,
    children: Vec<(std::ffi::OsString, C)>,
    root: bool,
}

fn preflight<C>(summary: &Summary, local: &Local<C>) -> Result<(), Error> {
    add(summary.examined_directories, 1)?;
    add(summary.excluded, local.coverage.excluded)?;
    add(summary.planned, local.coverage.planned)?;
    add(summary.deviated, local.coverage.deviated)?;
    add(summary.retained, local.coverage.retained)?;
    add(summary.equal, local.coverage.equal)?;
    add(summary.different, local.coverage.different)?;
    add(summary.not_assessed, local.coverage.not_assessed)?;
    add(summary.unsupported, local.coverage.unsupported)?;
    add(summary.unresolved, local.coverage.unresolved)?;
    add(summary.stopped_paired, local.coverage.stopped_paired)?;
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
mod context_tests;
#[cfg(test)]
mod tests;
