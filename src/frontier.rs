//! Pure ordered reducer plus completion-driven, scoped dispatch.
//!
//! All admission/commitment happens under one short-held mutex. Inspection and
//! scope operations happen outside it. Workers publish, reserve successors,
//! unlock, then spawn; no worker receives from a channel or waits for a permit.

use std::collections::VecDeque;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use crate::pair::ComparedPair;
use crate::parallel::ParallelLimits;
use crate::{DirDiff, Error, ScanState, Summary};

#[cfg(any(test, feature = "parallel-diagnostics"))]
#[derive(Clone, Debug, Default)]
pub struct ParallelDiagnostics {
    pub epochs: u64,
    pub admitted: u64,
    pub completed: u64,
    pub committed: u64,
    pub peak_outstanding: usize,
    pub peak_active: usize,
    pub peak_speculative: usize,
    pub peak_uncommitted: usize,
    pub peak_retained_name_bytes: usize,
    pub demand_reserve_uses: u64,
    /// Admission observations with a free J slot and an eligible child, but K
    /// full. Counts opportunities, not distinct intervals or stalled time.
    pub speculative_budget_stalls: u64,
    pub active_job_nanoseconds: u128,
    pub drain_nanoseconds: u128,
    pub max_drain_nanoseconds: u128,
    pub worker_ids: std::collections::BTreeSet<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct Id {
    slot: usize,
    generation: u64,
}

#[derive(Debug)]
struct Slot {
    generation: u64,
    node: Option<Node>,
}

#[derive(Debug, Default)]
struct Arena {
    slots: Vec<Slot>,
    free: Vec<usize>,
    len: usize,
}

impl Arena {
    fn insert(&mut self, node: Node) -> Id {
        let slot = if let Some(slot) = self.free.pop() {
            self.slots[slot].generation = self.slots[slot].generation.checked_add(1).expect("arena generation overflow");
            self.slots[slot].node = Some(node);
            slot
        } else {
            self.slots.push(Slot {
                generation: 0,
                node: Some(node),
            });
            self.slots.len() - 1
        };
        self.len += 1;
        Id {
            slot,
            generation: self.slots[slot].generation,
        }
    }

    fn get(&self, id: Id) -> &Node {
        let slot = &self.slots[id.slot];
        assert_eq!(slot.generation, id.generation, "stale frontier handle");
        slot.node.as_ref().expect("live frontier handle")
    }

    fn get_mut(&mut self, id: Id) -> &mut Node {
        let slot = &mut self.slots[id.slot];
        assert_eq!(slot.generation, id.generation, "stale frontier handle");
        slot.node.as_mut().expect("live frontier handle")
    }

    fn remove(&mut self, id: Id) -> Node {
        assert_eq!(self.slots[id.slot].generation, id.generation, "stale frontier handle");
        self.len -= 1;
        self.free.push(id.slot);
        self.slots[id.slot].node.take().expect("live frontier handle")
    }

    fn clear(&mut self) {
        self.slots.clear();
        self.free.clear();
        self.len = 0;
    }
}

#[derive(Debug)]
struct Node {
    dir: PathBuf,
    speculative: bool,
    state: NodeState,
}

#[derive(Debug)]
// Only K+1 reusable arena slots exist. Inline results avoid an allocation on
// every directory completion; queued slots do not justify a per-result Box.
#[allow(clippy::large_enum_variant)]
enum NodeState {
    Queued,
    Running,
    Ready(Result<ReadyPair, Error>),
}

#[derive(Debug)]
struct ReadyPair {
    diff: DirDiff,
    children: Children,
}

/// Admitted children are always a prefix of remaining sorted names. Because
/// reserved jobs are drained (not canceled), there are no holes to index/map.
#[derive(Debug, Default)]
struct Children {
    names: VecDeque<OsString>,
    admitted: VecDeque<Id>,
}

impl Children {
    fn from_names(names: Vec<OsString>) -> Self {
        Self {
            names: names.into(),
            admitted: VecDeque::new(),
        }
    }
    fn has_unadmitted(&self) -> bool {
        self.admitted.len() < self.names.len()
    }
}

#[derive(Debug)]
struct Frame {
    dir: PathBuf,
    children: Children,
    direct_ancestor: bool,
    subtree_changed: bool,
    open: bool,
}

#[derive(Clone, Copy, Debug)]
enum Owner {
    Frame(usize),
    Node(Id),
}

#[derive(Clone, Copy, Debug)]
enum Search {
    Enter(Owner),
    Candidate(Owner),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Stop {
    Running,
    Yield,
    Error,
    Exhausted,
}

#[derive(Debug)]
struct Job {
    id: Id,
    dir: PathBuf,
}

#[derive(Debug)]
pub(crate) struct Frontier {
    limits: ParallelLimits,
    nodes: Arena,
    frames: Vec<Frame>,
    root_pending: bool,
    demand: Option<Id>,
    speculative: usize,
    outstanding: usize,
    active: usize,
    stop: Stop,
    output: Option<Result<DirDiff, Error>>,
    prune_eligible: bool,
    summary: Summary,
    #[cfg(any(test, feature = "parallel-diagnostics"))]
    stats: ParallelDiagnostics,
    #[cfg(any(test, feature = "parallel-diagnostics"))]
    clock: Option<std::time::Instant>,
    #[cfg(any(test, feature = "parallel-diagnostics"))]
    stop_at: Option<std::time::Instant>,
}

impl Frontier {
    pub(crate) fn new(limits: ParallelLimits) -> Self {
        Self {
            limits,
            nodes: Arena::default(),
            frames: Vec::new(),
            root_pending: true,
            demand: None,
            speculative: 0,
            outstanding: 0,
            active: 0,
            stop: Stop::Running,
            output: None,
            prune_eligible: false,
            summary: Summary::default(),
            #[cfg(any(test, feature = "parallel-diagnostics"))]
            stats: ParallelDiagnostics::default(),
            #[cfg(any(test, feature = "parallel-diagnostics"))]
            clock: None,
            #[cfg(any(test, feature = "parallel-diagnostics"))]
            stop_at: None,
        }
    }

    pub(crate) fn next_with<F>(&mut self, inspect: F) -> Option<Result<DirDiff, Error>>
    where
        F: Fn(&Path) -> Result<ComparedPair, Error> + Sync,
    {
        if self.summary.state != ScanState::Incomplete {
            return None;
        }
        self.begin_epoch();
        let shared = Mutex::new(&mut *self);
        rayon::scope(|scope| {
            let jobs = shared.lock().unwrap().pump();
            dispatch(scope, &shared, &inspect, jobs);
        });
        assert_eq!(self.outstanding, 0, "scope returned with outstanding jobs");
        assert_eq!(self.active, 0);
        assert_ne!(self.stop, Stop::Running, "frontier stalled without a deliverable prefix");
        #[cfg(any(test, feature = "parallel-diagnostics"))]
        if let Some(stop_at) = self.stop_at.take() {
            let drain = stop_at.elapsed().as_nanos();
            self.stats.drain_nanoseconds += drain;
            self.stats.max_drain_nanoseconds = self.stats.max_drain_nanoseconds.max(drain);
        }
        if self.stop == Stop::Error {
            self.nodes.clear();
            self.speculative = 0;
            self.demand = None;
            for frame in &mut self.frames {
                frame.children = Children::default();
            }
        }
        self.output.take()
    }

    fn begin_epoch(&mut self) {
        self.prune_eligible = false;
        if let Some(frame) = self.frames.last_mut() {
            frame.open = true;
        }
        self.stop = Stop::Running;
        #[cfg(any(test, feature = "parallel-diagnostics"))]
        {
            self.stats.epochs += 1;
            self.stop_at = None;
        }
    }

    pub(crate) fn prune_here(&mut self) -> bool {
        if !self.prune_eligible {
            return false;
        }
        self.prune_eligible = false;
        let frame = self.frames.last_mut().expect("yielded frame");
        assert!(!frame.open);
        assert!(frame.children.admitted.is_empty(), "descendant admitted below divergence seal");
        frame.children = Children::default();
        frame.open = true;
        self.summary.pruned.push(frame.dir.clone());
        true
    }

    pub(crate) fn into_summary(mut self) -> Summary {
        assert_eq!(self.outstanding, 0);
        while !self.frames.is_empty() {
            self.finish_frame();
        }
        self.summary
    }

    #[cfg(feature = "parallel-diagnostics")]
    pub(crate) fn diagnostics(&self) -> &ParallelDiagnostics {
        &self.stats
    }

    fn finish_frame(&mut self) {
        let frame = self.frames.pop().expect("committed frame");
        if frame.subtree_changed {
            self.summary.changed_dirs.push(frame.dir);
            if let Some(parent) = self.frames.last_mut() {
                parent.subtree_changed = true;
            }
        }
    }

    /// Only this reducer changes public coverage. Speculation never does.
    fn reduce(&mut self) {
        loop {
            if let Some(id) = self.demand {
                if !matches!(self.nodes.get(id).state, NodeState::Ready(_)) {
                    return;
                }
                let node = self.nodes.remove(id);
                assert!(!node.speculative);
                self.demand = None;
                let NodeState::Ready(result) = node.state else { unreachable!() };
                match result {
                    Err(error) => {
                        self.summary.state = ScanState::Failed;
                        self.output = Some(Err(error));
                        self.set_stop(Stop::Error);
                        return;
                    }
                    Ok(pair) => {
                        let changed = !pair.diff.is_empty();
                        let direct_ancestor = self.frames.last().is_some_and(|frame| frame.direct_ancestor);
                        self.summary.examined_directories += 1;
                        self.summary.counts.add(&pair.diff);
                        if changed && !direct_ancestor {
                            self.summary.topmost_divergent.push(node.dir.clone());
                        }
                        #[cfg(any(test, feature = "parallel-diagnostics"))]
                        {
                            self.stats.committed += 1;
                        }
                        if changed {
                            assert!(pair.children.admitted.is_empty(), "sealed result has child jobs");
                        }
                        self.frames.push(Frame {
                            dir: node.dir,
                            children: pair.children,
                            direct_ancestor: direct_ancestor || changed,
                            subtree_changed: changed,
                            open: !changed,
                        });
                        if changed {
                            self.prune_eligible = true;
                            self.output = Some(Ok(pair.diff));
                            self.set_stop(Stop::Yield);
                            return;
                        }
                    }
                }
            } else if self.root_pending {
                return;
            } else if let Some(frame) = self.frames.last_mut() {
                assert!(frame.open);
                if frame.children.names.is_empty() {
                    self.finish_frame();
                } else if let Some(id) = frame.children.admitted.pop_front() {
                    frame.children.names.pop_front();
                    let node = self.nodes.get_mut(id);
                    assert!(node.speculative);
                    node.speculative = false;
                    self.speculative -= 1;
                    self.demand = Some(id);
                } else {
                    return;
                }
            } else {
                self.summary.state = ScanState::Complete;
                self.set_stop(Stop::Exhausted);
                return;
            }
        }
    }

    fn set_stop(&mut self, stop: Stop) {
        self.stop = stop;
        #[cfg(any(test, feature = "parallel-diagnostics"))]
        {
            self.stop_at = Some(std::time::Instant::now());
        }
    }

    fn children(&self, owner: Owner) -> Option<&Children> {
        match owner {
            Owner::Frame(i) => self.frames[i].open.then_some(&self.frames[i].children),
            Owner::Node(id) => match &self.nodes.get(id).state {
                NodeState::Ready(Ok(pair)) if pair.diff.is_empty() => Some(&pair.children),
                _ => None,
            },
        }
    }

    fn children_mut(&mut self, owner: Owner) -> &mut Children {
        match owner {
            Owner::Frame(i) => {
                assert!(self.frames[i].open);
                &mut self.frames[i].children
            }
            Owner::Node(id) => match &mut self.nodes.get_mut(id).state {
                NodeState::Ready(Ok(pair)) if pair.diff.is_empty() => &mut pair.children,
                _ => panic!("children requested below unresolved node"),
            },
        }
    }

    /// DFS priority by child positions, never Path ordering. Stack traversal is
    /// iterative even for arbitrarily deep clean wrapper chains.
    fn candidate(&self) -> Option<Owner> {
        let mut search = (0..self.frames.len()).map(|i| Search::Enter(Owner::Frame(i))).collect::<Vec<_>>();
        while let Some(action) = search.pop() {
            match action {
                Search::Candidate(owner) => return Some(owner),
                Search::Enter(owner) => {
                    if let Some(children) = self.children(owner) {
                        if children.has_unadmitted() {
                            search.push(Search::Candidate(owner));
                        }
                        for id in children.admitted.iter().rev() {
                            search.push(Search::Enter(Owner::Node(*id)));
                        }
                    }
                }
            }
        }
        None
    }

    fn reserve(&mut self, dir: PathBuf, speculative: bool) -> Job {
        let id = self.nodes.insert(Node {
            dir: dir.clone(),
            speculative,
            state: NodeState::Queued,
        });
        self.outstanding += 1;
        if speculative {
            self.speculative += 1;
        } else {
            assert!(self.demand.replace(id).is_none());
        }
        #[cfg(any(test, feature = "parallel-diagnostics"))]
        {
            self.stats.admitted += 1;
            self.stats.peak_outstanding = self.stats.peak_outstanding.max(self.outstanding);
            self.stats.peak_speculative = self.stats.peak_speculative.max(self.speculative);
            self.stats.peak_uncommitted = self.stats.peak_uncommitted.max(self.nodes.len);
        }
        Job { id, dir }
    }

    fn pump(&mut self) -> Vec<Job> {
        if self.stop != Stop::Running {
            #[cfg(any(test, feature = "parallel-diagnostics"))]
            self.sample_diagnostics();
            return Vec::new();
        }
        self.reduce();
        let mut jobs = Vec::new();
        while self.stop == Stop::Running && self.outstanding < self.limits.max_directory_jobs.get() {
            if self.demand.is_none() {
                let dir = if self.root_pending {
                    self.root_pending = false;
                    PathBuf::new()
                } else {
                    let frame = self.frames.last_mut().expect("unmet demand has parent");
                    assert!(frame.open && frame.children.admitted.is_empty());
                    frame.dir.join(frame.children.names.pop_front().expect("unmet demand has child"))
                };
                #[cfg(any(test, feature = "parallel-diagnostics"))]
                if self.speculative == self.limits.max_speculative_directories && self.speculative > 0 {
                    self.stats.demand_reserve_uses += 1;
                }
                jobs.push(self.reserve(dir, false));
            } else if self.speculative < self.limits.max_speculative_directories {
                let Some(owner) = self.candidate() else {
                    break;
                };
                let parent = match owner {
                    Owner::Frame(i) => &self.frames[i].dir,
                    Owner::Node(id) => &self.nodes.get(id).dir,
                };
                let children = self.children(owner).unwrap();
                let dir = parent.join(&children.names[children.admitted.len()]);
                let job = self.reserve(dir, true);
                self.children_mut(owner).admitted.push_back(job.id);
                jobs.push(job);
            } else {
                break;
            }
        }
        debug_assert!(self.outstanding <= self.limits.max_directory_jobs.get());
        debug_assert!(self.speculative <= self.limits.max_speculative_directories);
        debug_assert!(self.nodes.len <= self.limits.max_speculative_directories.saturating_add(1));
        debug_assert!(self.stop != Stop::Running || self.outstanding > 0, "unmet demand without a reserved job");
        #[cfg(any(test, feature = "parallel-diagnostics"))]
        self.sample_diagnostics();
        jobs
    }

    fn start(&mut self, id: Id) {
        assert!(matches!(self.nodes.get(id).state, NodeState::Queued));
        self.nodes.get_mut(id).state = NodeState::Running;
        #[cfg(any(test, feature = "parallel-diagnostics"))]
        self.update_clock();
        self.active += 1;
        #[cfg(any(test, feature = "parallel-diagnostics"))]
        {
            self.stats.peak_active = self.stats.peak_active.max(self.active);
            if let Some(worker) = rayon::current_thread_index() {
                self.stats.worker_ids.insert(worker);
            }
        }
    }

    fn complete(&mut self, id: Id, result: Result<ComparedPair, Error>) -> Vec<Job> {
        assert!(matches!(self.nodes.get(id).state, NodeState::Running));
        #[cfg(any(test, feature = "parallel-diagnostics"))]
        {
            self.update_clock();
            self.stats.completed += 1;
        }
        self.active -= 1;
        self.outstanding -= 1;
        self.nodes.get_mut(id).state = NodeState::Ready(result.map(|pair| ReadyPair {
            diff: pair.diff,
            children: Children::from_names(pair.children),
        }));
        self.pump()
    }

    #[cfg(any(test, feature = "parallel-diagnostics"))]
    fn update_clock(&mut self) {
        let now = std::time::Instant::now();
        if let Some(previous) = self.clock.replace(now) {
            self.stats.active_job_nanoseconds += now.duration_since(previous).as_nanos() * self.active as u128;
        }
    }

    #[cfg(any(test, feature = "parallel-diagnostics"))]
    fn sample_diagnostics(&mut self) {
        if self.stop == Stop::Running
            && self.outstanding < self.limits.max_directory_jobs.get()
            && self.speculative == self.limits.max_speculative_directories
            && self.candidate().is_some()
        {
            self.stats.speculative_budget_stalls += 1;
        }
        let mut bytes = self
            .frames
            .iter()
            .map(|frame| frame.dir.as_os_str().as_encoded_bytes().len() + frame.children.names.iter().map(|n| n.as_encoded_bytes().len()).sum::<usize>())
            .sum::<usize>();
        for slot in &self.nodes.slots {
            if let Some(node) = &slot.node {
                bytes += node.dir.as_os_str().as_encoded_bytes().len();
                if let NodeState::Ready(Ok(pair)) = &node.state {
                    bytes += pair.children.names.iter().map(|n| n.as_encoded_bytes().len()).sum::<usize>();
                }
            }
        }
        self.stats.peak_retained_name_bytes = self.stats.peak_retained_name_bytes.max(bytes);
    }
}

fn dispatch<'scope, F>(scope: &rayon::Scope<'scope>, shared: &'scope Mutex<&mut Frontier>, inspect: &'scope F, jobs: Vec<Job>)
where
    F: Fn(&Path) -> Result<ComparedPair, Error> + Sync,
{
    // Rayon pushes locally LIFO; reserve demand first, submit it last so the
    // completing worker prefers the ordered cursor while thieves take siblings.
    for job in jobs.into_iter().rev() {
        scope.spawn(move |scope| {
            shared.lock().unwrap().start(job.id);
            let result = inspect(&job.dir);
            let successors = shared.lock().unwrap().complete(job.id, result);
            dispatch(scope, shared, inspect, successors);
        });
    }
}

#[cfg(test)]
mod tests;
