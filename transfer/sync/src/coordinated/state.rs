use diff_dirs_tree_compare::coordinated::{LaneStatus, Parts as ComparisonParts};

use super::{DirectoryDecision, Error, require};
use crate::{CopyPolicy, Progress};

/// Why a destination has no more reconciliation obligations in this lineage.
/// Original executable error causes are returned live, not serialized here.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum End {
    ComparisonFailed,
    ApplicationFailed,
    RefreshFailed,
    DecisionChanged,
    Abandoned,
}

/// Current-group disposition, never an interior copy/removal recovery position.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ApplicationState {
    Unstarted,
    Accepted,
    Failed,
    Abandoned,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Application<C = ()> {
    /// Original saved binding index, never a compact runtime index.
    pub target: usize,
    pub decision: DirectoryDecision<C>,
    pub state: ApplicationState,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Barrier<C = ()> {
    pub sequence: u64,
    pub applications: Vec<Application<C>>,
}

macro_rules! work_totals {
    ($($field:ident),* $(,)?) => {
        /// Wide cumulative counts of actual returned production progress. Each
        /// application returns u64 counters; at most u64::MAX calls are accepted.
        #[derive(Clone, Debug, Default, PartialEq, Eq)]
        pub struct WorkTotals { $(pub $field: u128,)* }
        impl WorkTotals {
            pub(super) fn add(&mut self, progress: &Progress) { $(self.$field += u128::from(progress.$field);)* }
            fn bounded(&self, bound: u128) -> bool { true $(&& self.$field <= bound)* }
        }
    }
}
work_totals!(
    files_copied,
    files_reflinked,
    bytes_copied,
    bytes_reflinked,
    symlinks_created,
    directories_created,
    files_removed,
    symlinks_removed,
    directories_removed,
    directory_permissions_set
);

/// Application accounting, separate from committed pre-application observations.
/// Wide counters avoid losing an actual returned result to cumulative u64 overflow.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LaneTotals {
    pub applications: u64,
    pub planned: u128,
    pub completed: WorkTotals,
    /// Recursive retention beyond the immediate slots already in comparison.
    pub recursive_retained: u128,
    pub excluded: u128,
    pub end: Option<End>,
    /// An explicit traversal prune abandons reconciliation coverage, unlike a
    /// visitor-preserved region which is part of the declared policy.
    pub pruned: bool,
}

/// Data-only continuation. Full plans, causes, policies, pools and filesystem
/// resources are deliberately absent. Old/forked parts can repeat effects; this
/// is neither a journal nor an exactly-once publication mechanism.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SyncParts<C = ()> {
    pub version: u32,
    pub comparison: ComparisonParts<C>,
    pub copy_policies: Vec<CopyPolicy>,
    pub lanes: Vec<LaneTotals>,
    pub current: Option<Barrier<C>>,
}

impl<C: Clone + Eq> SyncParts<C> {
    pub fn validate(&self) -> Result<(), Error> {
        require(self.version == 2, "unsupported sync parts version")?;
        self.comparison.validate().map_err(Error::Comparison)?;
        let n = self.comparison.bindings.targets.len();
        require(self.copy_policies.len() == n && self.lanes.len() == n, "sync binding/lane count mismatch")?;
        super::disjoint(&self.comparison.bindings)?;
        for (i, totals) in self.lanes.iter().enumerate() {
            let lane = &self.comparison.lanes[i];
            require(totals.applications <= lane.summary.examined_directories, "more applications than observations")?;
            let bound = u128::from(totals.applications) * u128::from(u64::MAX);
            require(
                totals.planned <= bound && totals.recursive_retained <= bound && totals.excluded <= bound && totals.completed.bounded(bound),
                "application totals exceed returned-call bounds",
            )?;
            require(totals.pruned == !lane.summary.pruned.is_empty(), "sync prune coverage mismatch")?;
            let end_valid = match lane.status {
                LaneStatus::Active | LaneStatus::Complete => totals.end.is_none(),
                LaneStatus::Failed => totals.end == Some(End::ComparisonFailed),
                LaneStatus::Stopped => matches!(
                    totals.end,
                    Some(End::ApplicationFailed | End::RefreshFailed | End::DecisionChanged | End::Abandoned)
                ),
            };
            require(end_valid, "sync/comparison termination mismatch")?;
        }
        match (&self.current, &self.comparison.window) {
            (None, None) => {}
            (Some(barrier), Some(window)) => {
                let frame = self.comparison.frames.last().ok_or(Error::InvalidState("barrier has no frame"))?;
                require(barrier.sequence == window.sequence, "barrier/control sequence mismatch")?;
                require(
                    barrier.applications.windows(2).all(|p| p[0].target < p[1].target),
                    "unordered barrier applications",
                )?;
                let mut active = Vec::new();
                for app in &barrier.applications {
                    require(app.target < n, "unknown barrier target")?;
                    super::decision::validate_decision(&app.decision)?;
                    let binding = &self.comparison.bindings.targets[app.target];
                    require(
                        app.decision.policy() == &binding.policy_id
                            && app.decision.context_schema() == &binding.context_schema
                            && app.decision.comparator() == &binding.comparator,
                        "decision interpretation disagrees with binding",
                    )?;
                    require(app.decision.relative() == frame.relative, "barrier coordinate mismatch")?;
                    let lane = &self.lanes[app.target];
                    match app.state {
                        ApplicationState::Unstarted | ApplicationState::Accepted => {
                            if self.comparison.lanes[app.target].status == LaneStatus::Active {
                                active.push(app.target);
                            } else {
                                require(
                                    lane.end == Some(End::Abandoned) && app.state == ApplicationState::Accepted,
                                    "settled application on incompatible terminal lane",
                                )?;
                            }
                        }
                        ApplicationState::Failed => require(
                            matches!(lane.end, Some(End::ApplicationFailed | End::RefreshFailed | End::DecisionChanged)),
                            "failed application without terminal failure",
                        )?,
                        ApplicationState::Abandoned => require(lane.end == Some(End::Abandoned), "abandoned application without stopped lane")?,
                    }
                    if app.decision.is_root() {
                        let source = &self.comparison.bindings.source;
                        require(
                            app.decision.entry(diff_dirs_tree_inspect::SlotId(0)).name() == source.file_name().unwrap_or(source.as_os_str()),
                            "structural root name mismatch",
                        )?;
                        require(
                            app.state != ApplicationState::Unstarted
                                && matches!(
                                    app.decision.entry(diff_dirs_tree_inspect::SlotId(0)).action,
                                    diff_dirs_tree_inspect::Action::PreserveTree
                                        | diff_dirs_tree_inspect::Action::Directory {
                                            container: diff_dirs_tree_inspect::Container::Keep,
                                            descend: false
                                        }
                                ),
                            "unsettled structural root",
                        )?;
                    }
                    if self.comparison.lanes[app.target].status == LaneStatus::Active {
                        require(
                            frame
                                .lanes
                                .iter()
                                .find(|l| l.target == app.target)
                                .is_some_and(|l| &l.context == app.decision.input_context()),
                            "decision/frame input context mismatch",
                        )?;
                        let children: Vec<_> = app
                            .decision
                            .paired_children()
                            .map(|e| (e.name(), e.interior.as_ref().expect("paired").context.clone()))
                            .collect();
                        let actual: Vec<_> = frame
                            .remaining
                            .iter()
                            .filter(|e| e.members.iter().any(|m| m.target == app.target))
                            .map(|e| {
                                (
                                    e.name.as_os_str(),
                                    e.members.iter().find(|m| m.target == app.target).expect("member").context.clone(),
                                )
                            })
                            .collect();
                        require(
                            if window.pruned.contains(&app.target) {
                                actual.is_empty()
                            } else {
                                actual == children
                            },
                            "decision/paired edge mismatch",
                        )?;
                    }
                }
                require(active == window.successful, "barrier participants do not match current comparison window")?;
            }
            _ => return Err(Error::InvalidState("missing sync barrier or comparison control window")),
        }
        Ok(())
    }
}
