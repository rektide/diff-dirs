//! Owned, data-only continuation. Indices always refer to the original bindings.

use std::collections::{HashSet, VecDeque};
use std::ffi::OsString;
use std::path::{Component, Path, PathBuf};

use super::{Error, TargetId};
use crate::{Comparator, FullVisitor, NoVisitor, Policy, ScanState, SemanticId, StatKey, Summary};

/// One stable destination identity, coordinate and frozen policy interpretation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TargetBinding<C = ()> {
    pub id: TargetId,
    pub root: PathBuf,
    /// Caller-defined, nonempty identity/version/parameters of the pure visitor.
    /// Equality is a caller contract, not inspection of executable code.
    pub policy: String,
    pub policy_id: SemanticId,
    pub context_schema: SemanticId,
    pub comparator: SemanticId,
    pub initial_context: C,
}

impl<C> TargetBinding<C> {
    /// Freeze interpretation and initial policy input without filesystem access.
    pub fn new<P: Policy<Context = C>, M: Comparator>(id: TargetId, root: PathBuf, descriptor: String, policy: &P, comparator: &M) -> Self {
        Self {
            id,
            root,
            policy: descriptor,
            policy_id: policy.identity().clone(),
            context_schema: policy.context_schema().clone(),
            comparator: comparator.identity().clone(),
            initial_context: policy.initial_context(),
        }
    }
}
impl TargetBinding<()> {
    /// Full-visitor/StatKey compatibility interpretation. The descriptor still
    /// names the caller's actual visitor and behavior-affecting inputs.
    pub fn full(id: TargetId, root: PathBuf, descriptor: String) -> Self {
        Self::new(id, root, descriptor, &FullVisitor(NoVisitor), &StatKey)
    }
}

/// Absolute, lexically normalized native coordinates. No filesystem access.
/// Read-only aliases and nested roots are permitted and remain distinct lanes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Bindings<C = ()> {
    pub source: PathBuf,
    pub targets: Vec<TargetBinding<C>>,
}

impl<C> Bindings<C> {
    pub fn validate(&self) -> Result<(), Error> {
        require(normal_path(&self.source, true), "invalid absolute source root")?;
        require(!self.targets.is_empty(), "no target bindings")?;
        let mut ids = HashSet::new();
        for target in &self.targets {
            require(!target.id.0.is_empty() && ids.insert(&target.id), "empty or duplicate target identity")?;
            require(!target.policy.is_empty(), "empty policy descriptor")?;
            require(
                target.policy_id.valid() && target.context_schema.valid() && target.comparator.valid(),
                "invalid semantic identity",
            )?;
            require(normal_path(&target.root, true), "invalid absolute target root")?;
        }
        Ok(())
    }
}

/// A stopped or failed lane has no remaining traversal obligations. Restart it
/// with a fresh run; this cursor does not reattach terminated destinations.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LaneStatus {
    Active,
    Complete,
    Failed,
    Stopped,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LaneState {
    pub status: LaneStatus,
    /// Only closed postorder reductions live here. Use prefix_summaries() to
    /// project still-active ancestor evidence without changing this state.
    pub summary: Summary,
}

/// Accepted local assessment and not-yet-folded subtree evidence for one lane.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LaneFrame<C = ()> {
    pub target: usize,
    /// Exact input for this directory's listing, including a derived root input.
    pub context: C,
    pub direct_changed: bool,
    /// Includes this directory's own direct divergence, like the pair walker.
    pub direct_ancestor: bool,
    pub subtree_changed: bool,
}

/// One immediate paired child, in native-name order, with independent membership.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChildMember<C = ()> {
    pub target: usize,
    pub context: C,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChildEdge<C = ()> {
    pub name: OsString,
    pub members: Vec<ChildMember<C>>,
}

/// A partial ancestor. The active child's edge stays at the front until return.
/// A lane can wait here while another lane is deeper on the global DFS stack.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GroupFrame<C = ()> {
    pub sequence: u64,
    pub relative: PathBuf,
    pub lanes: Vec<LaneFrame<C>>,
    pub remaining: VecDeque<ChildEdge<C>>,
}

/// Saved control position, not a reusable runtime token. Restoration issues a
/// new opaque token through current_token(); it does not reemit the old group.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ControlWindow {
    pub sequence: u64,
    pub successful: Vec<usize>,
    pub pruned: Vec<usize>,
}

/// Public semantic parts for application-owned transport. Contains no visitors,
/// Metadata, errors with executable causes, pools, handles or speculative jobs.
/// Call from_parts() after decoding; byte validity does not prove past decisions.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Parts<C = ()> {
    pub version: u32,
    pub bindings: Bindings<C>,
    pub lanes: Vec<LaneState>,
    pub root_pending: bool,
    pub frames: Vec<GroupFrame<C>>,
    pub window: Option<ControlWindow>,
    pub next_sequence: u64,
}

impl<C: Clone + Eq> Parts<C> {
    pub fn fresh(bindings: Bindings<C>) -> Result<Self, Error> {
        bindings.validate()?;
        let lanes = bindings
            .targets
            .iter()
            .map(|binding| LaneState {
                status: LaneStatus::Active,
                summary: Summary {
                    comparator: Some(binding.comparator.clone()),
                    ..Summary::default()
                },
            })
            .collect();
        Ok(Self {
            version: 3,
            bindings,
            lanes,
            root_pending: true,
            frames: Vec::new(),
            window: None,
            next_sequence: 1,
        })
    }

    /// Strict structural checks only; no stat, callbacks or root reconstruction.
    pub fn validate(&self) -> Result<(), Error> {
        require(self.version == 3, "unsupported coordinated parts version")?;
        self.bindings.validate()?;
        require(self.lanes.len() == self.bindings.targets.len(), "lane/binding count mismatch")?;
        require(self.next_sequence > 0, "zero group sequence")?;
        if self.root_pending {
            require(
                self.frames.is_empty() && self.window.is_none() && self.next_sequence == 1,
                "invalid pending root",
            )?;
        }
        for (target, lane) in self.lanes.iter().enumerate() {
            let s = &lane.summary;
            require(
                s.comparator.as_ref() == Some(&self.bindings.targets[target].comparator),
                "summary comparator mismatch",
            )?;
            let expected = match lane.status {
                LaneStatus::Active | LaneStatus::Stopped => ScanState::Incomplete,
                LaneStatus::Complete => ScanState::Complete,
                LaneStatus::Failed => ScanState::Failed,
            };
            require(s.state == expected, "lane status/summary state mismatch")?;
            require(s.examined_directories < self.next_sequence, "assessment count exceeds group sequence")?;
            require(
                lane.status != LaneStatus::Complete || s.examined_directories != 0,
                "completed lane has no root assessment",
            )?;
            require(
                lane.status != LaneStatus::Failed || self.next_sequence != 1,
                "failed lane has no attempted group",
            )?;
            let counts = [
                s.counts.files.changed,
                s.counts.files.added,
                s.counts.files.removed,
                s.counts.dirs.added,
                s.counts.dirs.removed,
            ];
            counts.into_iter().try_fold(0u64, add)?;
            let slots = [s.equal, s.different, s.not_assessed, s.unsupported, s.unresolved]
                .into_iter()
                .map(u128::from)
                .sum::<u128>();
            require(u128::from(s.retained) <= slots, "retention exceeds assessed slot boundaries")?;
            if s.examined_directories == 0 {
                require(
                    counts == [0; 5]
                        && !s.scoped
                        && s.excluded == 0
                        && s.planned == 0
                        && s.deviated == 0
                        && s.retained == 0
                        && s.equal == 0
                        && s.different == 0
                        && s.not_assessed == 0
                        && s.unsupported == 0
                        && s.unresolved == 0
                        && s.stopped_paired == 0,
                    "unexamined lane has contributions",
                )?;
            }
            for (paths, postorder) in [(&s.changed_dirs, true), (&s.topmost_divergent, false), (&s.pruned, false)] {
                let mut seen = HashSet::new();
                for path in paths {
                    require(normal_path(path, false) && seen.insert(path), "invalid or repeated summary coordinate")?;
                }
                require(
                    u64::try_from(paths.len()).is_ok_and(|n| n <= s.examined_directories),
                    "too many summary coordinates",
                )?;
                require(
                    paths.windows(2).all(|p| path_order(&p[0], &p[1], postorder).is_lt()),
                    "unordered summary coordinates",
                )?;
            }
            if self.root_pending {
                require(
                    *s == Summary {
                        comparator: Some(self.bindings.targets[target].comparator.clone()),
                        ..Summary::default()
                    },
                    "unstarted lane has observations",
                )?;
                require(
                    matches!(lane.status, LaneStatus::Active | LaneStatus::Stopped),
                    "unstarted lane has terminal scan",
                )?;
            }
            let active = self.frames.iter().filter(|f| f.lanes.iter().any(|l| l.target == target)).count();
            require(
                u64::try_from(active).is_ok_and(|n| n <= s.examined_directories),
                "active frames exceed accepted assessments",
            )?;
            if lane.status == LaneStatus::Active && !self.root_pending {
                require(active != 0, "active lane has no remaining head")?;
            }
        }
        for (depth, frame) in self.frames.iter().enumerate() {
            require(normal_path(&frame.relative, false), "invalid frame coordinate")?;
            require(frame.sequence > 0 && frame.sequence < self.next_sequence, "invalid frame sequence")?;
            if depth == 0 {
                require(frame.relative.as_os_str().is_empty(), "missing root ancestor")?;
            } else {
                let parent = &self.frames[depth - 1];
                require(parent.sequence < frame.sequence, "unordered frame sequences")?;
                let edge = parent.remaining.front().ok_or(Error::InvalidState("active child has no parent edge"))?;
                require(parent.relative.join(&edge.name) == frame.relative, "active child coordinate mismatch")?;
                require(
                    edge.members
                        .iter()
                        .map(|m| (m.target, &m.context))
                        .eq(frame.lanes.iter().map(|l| (l.target, &l.context))),
                    "active child membership mismatch",
                )?;
                for lane in &frame.lanes {
                    let p = parent
                        .lanes
                        .iter()
                        .find(|p| p.target == lane.target)
                        .ok_or(Error::InvalidState("child has no parent lane"))?;
                    require(lane.direct_ancestor == (p.direct_ancestor || lane.direct_changed), "lane ancestry mismatch")?;
                }
            }
            let lane_indices: Vec<_> = frame.lanes.iter().map(|l| l.target).collect();
            ordered(&lane_indices)?;
            for lane in &frame.lanes {
                require(
                    self.lanes.get(lane.target).is_some_and(|l| l.status == LaneStatus::Active),
                    "inactive or unknown frame lane",
                )?;
                require(!lane.direct_changed || (lane.direct_ancestor && lane.subtree_changed), "lost direct divergence")?;
                require(
                    !self.lanes[lane.target].summary.changed_dirs.contains(&frame.relative),
                    "active ancestor already folded",
                )?;
                if self.lanes[lane.target].summary.pruned.contains(&frame.relative) {
                    require(
                        frame.remaining.iter().all(|e| e.members.iter().all(|m| m.target != lane.target)),
                        "pruned ancestor retains child membership",
                    )?;
                }
                if depth == 0 {
                    require(lane.direct_ancestor == lane.direct_changed, "root has a divergent ancestor")?;
                }
            }
            let mut previous: Option<&OsString> = None;
            for (position, edge) in frame.remaining.iter().enumerate() {
                require(native_name(&edge.name), "invalid child basename")?;
                if let Some(previous) = previous {
                    require(previous.as_encoded_bytes() < edge.name.as_encoded_bytes(), "unordered or duplicate child names")?;
                }
                previous = Some(&edge.name);
                ordered(&edge.members.iter().map(|m| m.target).collect::<Vec<_>>())?;
                let empty_active = position == 0 && depth + 1 < self.frames.len();
                require(!edge.members.is_empty() || empty_active, "empty queued membership")?;
                require(
                    edge.members.iter().all(|m| lane_indices.binary_search(&m.target).is_ok()),
                    "child membership outside parent lanes",
                )?;
            }
            require(
                !frame.lanes.is_empty()
                    || frame.remaining.is_empty()
                    || (depth + 1 < self.frames.len() && frame.remaining.len() == 1 && frame.remaining[0].members.is_empty()),
                "empty frame owns future children",
            )?;
        }
        if let Some(window) = &self.window {
            let frame = self.frames.last().ok_or(Error::InvalidState("control window without frame"))?;
            require(
                window.sequence == frame.sequence && window.sequence.checked_add(1) == Some(self.next_sequence),
                "control sequence mismatch",
            )?;
            ordered(&window.successful)?;
            ordered(&window.pruned)?;
            require(
                window.successful.iter().copied().eq(frame.lanes.iter().map(|l| l.target)),
                "control participants mismatch",
            )?;
            require(
                window.pruned.iter().all(|i| window.successful.binary_search(i).is_ok()),
                "invalid pruned participants",
            )?;
            for &target in &window.successful {
                require(
                    self.lanes[target].summary.pruned.contains(&frame.relative) == window.pruned.contains(&target),
                    "control prune flags disagree with coverage",
                )?;
            }
            for &target in &window.pruned {
                require(
                    frame.remaining.iter().all(|e| e.members.iter().all(|m| m.target != target)),
                    "pruned lane retains child membership",
                )?;
                require(self.lanes[target].summary.pruned.contains(&frame.relative), "prune coverage missing")?;
            }
        }
        if !self.root_pending && self.frames.is_empty() {
            require(self.lanes.iter().all(|l| l.status != LaneStatus::Active), "empty frontier has active lanes")?;
        }
        // Check reductions only after the complete frame/membership structure is
        // validated. A closed descendant inside this lane's active child has
        // folded there, but need not have propagated through its pending return.
        for (depth, frame) in self.frames.iter().enumerate() {
            for lane in &frame.lanes {
                let active_child = self.frames.get(depth + 1).filter(|child| child.lanes.iter().any(|l| l.target == lane.target));
                for path in &self.lanes[lane.target].summary.changed_dirs {
                    let Some(Component::Normal(child)) = path.strip_prefix(&frame.relative).ok().and_then(|suffix| suffix.components().next()) else {
                        continue;
                    };
                    if active_child.is_some_and(|active| path.starts_with(&active.relative)) {
                        continue;
                    }
                    if let Ok(index) = frame
                        .remaining
                        .binary_search_by(|edge| edge.name.as_encoded_bytes().cmp(child.as_encoded_bytes()))
                    {
                        require(
                            frame.remaining[index].members.binary_search_by_key(&lane.target, |m| m.target).is_err(),
                            "closed reduction overlaps unstarted child",
                        )?;
                    }
                    require(lane.subtree_changed, "lost returned child divergence")?;
                }
            }
        }
        Ok(())
    }
}

pub(super) fn add(a: u64, b: u64) -> Result<u64, Error> {
    a.checked_add(b).ok_or(Error::CounterOverflow)
}

pub(super) fn require(condition: bool, reason: &'static str) -> Result<(), Error> {
    if condition { Ok(()) } else { Err(Error::InvalidState(reason)) }
}

fn ordered(indices: &[usize]) -> Result<(), Error> {
    require(indices.windows(2).all(|p| p[0] < p[1]), "unordered or duplicate lane indices")
}

pub(super) fn native_name(name: &std::ffi::OsStr) -> bool {
    let path = Path::new(name);
    !name.as_encoded_bytes().contains(&0)
        && matches!(path.components().next(), Some(Component::Normal(_)))
        && path.components().count() == 1
        && path.file_name() == Some(name)
}

fn normal_path(path: &Path, absolute: bool) -> bool {
    !path.as_os_str().as_encoded_bytes().contains(&0)
        && path.is_absolute() == absolute
        && path
            .components()
            .all(|c| matches!(c, Component::Normal(_)) || (absolute && matches!(c, Component::RootDir | Component::Prefix(_))))
        && path.components().collect::<PathBuf>().as_os_str() == path.as_os_str()
}

fn path_order(left: &Path, right: &Path, postorder: bool) -> std::cmp::Ordering {
    let mut left = left.components();
    let mut right = right.components();
    loop {
        match (left.next(), right.next()) {
            (Some(a), Some(b)) => {
                let order = a.as_os_str().as_encoded_bytes().cmp(b.as_os_str().as_encoded_bytes());
                if !order.is_eq() {
                    return order;
                }
            }
            (None, None) => return std::cmp::Ordering::Equal,
            (None, Some(_)) => return if postorder { std::cmp::Ordering::Greater } else { std::cmp::Ordering::Less },
            (Some(_), None) => return if postorder { std::cmp::Ordering::Less } else { std::cmp::Ordering::Greater },
        }
    }
}
