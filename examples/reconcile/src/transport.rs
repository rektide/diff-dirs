//! Explicit version-four application DTOs. These are not derives on runtime
//! objects or an archive of private Rust layouts. Decode, then validate meaning.

use std::collections::VecDeque;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use diff_dirs_transfer_sync::coordinated::{Application, ApplicationState, Barrier, DirectoryDecision, End, Kind, LaneTotals, SyncParts, WorkTotals};
use diff_dirs_transfer_sync::{CopyPolicy, Postcheck};
use diff_dirs_tree_compare::coordinated::{
    Bindings, ChildEdge, ChildMember, ControlWindow, GroupFrame, LaneFrame, LaneState, LaneStatus, Parts, TargetBinding,
};
use diff_dirs_tree_compare::{
    Action, Container, DestinationGrant, DirectorySelection, EntryCounts, EntryParts, EntrySelection, InstallAuthority, InstallIntent, Interior, InteriorInput,
    InteriorRole, Purpose, ScanState, SelectionDomain, SelectionParts, SemanticId, SourceObservation, Summary,
};
use serde::{Deserialize, Serialize};

use crate::{PolicyConfig, Result, native::NativeString, native_path, validate_config};

pub const MAX_BYTES: u64 = 32 * 1024 * 1024;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Snapshot<C = ()> {
    pub schema: String,
    pub version: u32,
    pub policies: Vec<PolicyConfig>,
    pub state: State<C>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct State<C = ()> {
    pub version: u32,
    pub comparison: Comparison<C>,
    pub copy_policies: Vec<String>,
    pub postchecks: Vec<String>,
    pub lanes: Vec<Totals>,
    pub current: Option<PendingGroup<C>>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Comparison<C = ()> {
    pub version: u32,
    pub source: NativeString,
    pub targets: Vec<Binding<C>>,
    pub lanes: Vec<ObservedLane>,
    pub root_pending: bool,
    pub frames: Vec<Frame<C>>,
    pub window: Option<Window>,
    pub next_sequence: u64,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Binding<C = ()> {
    pub id: String,
    pub root: NativeString,
    pub policy: String,
    pub policy_id: Meaning,
    pub context_schema: Meaning,
    pub comparator: Meaning,
    pub initial_context: C,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObservedLane {
    pub status: String,
    pub summary: Observation,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Observation {
    pub scoped: bool,
    pub excluded: u64,
    pub planned: u64,
    pub deviated: u64,
    pub retained: u64,
    pub comparator: Option<Meaning>,
    pub equal: u64,
    pub different: u64,
    pub not_assessed: u64,
    pub unsupported: u64,
    pub unresolved: u64,
    pub stopped_paired: u64,
    pub files_changed: u64,
    pub files_added: u64,
    pub files_removed: u64,
    pub dirs_added: u64,
    pub dirs_removed: u64,
    pub topmost_divergent: Vec<NativeString>,
    pub changed_dirs: Vec<NativeString>,
    pub examined_directories: u64,
    pub state: String,
    pub pruned: Vec<NativeString>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Frame<C = ()> {
    pub sequence: u64,
    pub relative: NativeString,
    pub lanes: Vec<FrameLane<C>>,
    pub remaining: Vec<Edge<C>>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FrameLane<C = ()> {
    pub target: usize,
    pub context: C,
    pub direct_changed: bool,
    pub direct_ancestor: bool,
    pub subtree_changed: bool,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Edge<C = ()> {
    pub name: NativeString,
    pub members: Vec<Member<C>>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Member<C = ()> {
    pub target: usize,
    pub context: C,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Window {
    pub sequence: u64,
    pub successful: Vec<usize>,
    pub pruned: Vec<usize>,
}

macro_rules! wire_work {
    ($($field:ident),* $(,)?) => {
        #[derive(Serialize, Deserialize)]
        #[serde(deny_unknown_fields)]
        pub struct Work { $(pub $field: u128,)* }
        impl From<&WorkTotals> for Work { fn from(v: &WorkTotals) -> Self { Self { $($field: v.$field,)* } } }
        impl From<Work> for WorkTotals { fn from(v: Work) -> Self { Self { $($field: v.$field,)* } } }
    }
}
wire_work!(
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
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Totals {
    pub applications: u64,
    pub planned: u128,
    pub completed: Work,
    pub opaque_deviated: u128,
    pub additional_retained: u128,
    pub excluded: u128,
    pub end: Option<String>,
    pub pruned: bool,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PendingGroup<C = ()> {
    pub sequence: u64,
    pub applications: Vec<PendingApplication<C>>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PendingApplication<C = ()> {
    pub target: usize,
    pub decision: Decision<C>,
    pub state: String,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Decision<C = ()> {
    pub relative: NativeString,
    pub purpose: String,
    pub structural_root: bool,
    pub domain: String,
    pub input_context: C,
    pub policy: Meaning,
    pub context_schema: Meaning,
    pub comparator: Meaning,
    pub scoped: bool,
    pub root_excluded: bool,
    pub excluded_names: Vec<NativeString>,
    pub entries: Vec<Entry<C>>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Entry<C = ()> {
    pub name: NativeString,
    pub source: String,
    pub target: Option<String>,
    pub comparison: String,
    pub comparison_code: Option<u32>,
    pub baseline: Option<String>,
    pub action: String,
    pub install: Option<Installation>,
    pub source_absent_removal: bool,
    pub interior: Option<Inside<C>>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Meaning {
    pub family: String,
    pub version: u32,
    pub parameters: Vec<u8>,
}
impl From<&SemanticId> for Meaning {
    fn from(v: &SemanticId) -> Self {
        Self {
            family: v.family.clone(),
            version: v.version,
            parameters: v.parameters.clone(),
        }
    }
}
impl From<Meaning> for SemanticId {
    fn from(v: Meaning) -> Self {
        Self {
            family: v.family,
            version: v.version,
            parameters: v.parameters,
        }
    }
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Installation {
    pub intent: String,
    pub destination: String,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Inside<C = ()> {
    pub role: String,
    pub input: String,
    pub context: C,
}

macro_rules! vocabulary {
    ($encode:ident, $decode:ident, $ty:ty, {$($variant:path => $word:literal),* $(,)?}) => {
        fn $encode(v: $ty) -> String { match v { $($variant => $word.into(),)* } }
        fn $decode(v: &str) -> Result<$ty> { match v { $($word => Ok($variant),)* _ => Err(format!("unknown {}: {v}", stringify!($ty)).into()) } }
    }
}
vocabulary!(status, parse_status, LaneStatus, { LaneStatus::Active => "active", LaneStatus::Complete => "complete", LaneStatus::Failed => "failed", LaneStatus::Stopped => "stopped" });
vocabulary!(scan, parse_scan, ScanState, { ScanState::Incomplete => "incomplete", ScanState::Complete => "complete", ScanState::Failed => "failed" });
vocabulary!(copy, parse_copy, CopyPolicy, { CopyPolicy::Copy => "copy", CopyPolicy::ReflinkRequired => "reflink-required", CopyPolicy::ReflinkOrCopy => "reflink-or-copy" });
vocabulary!(postcheck, parse_postcheck, Postcheck, { Postcheck::None => "none", Postcheck::Mtime => "mtime" });
vocabulary!(end, parse_end, End, { End::ComparisonFailed => "comparison-failed", End::ApplicationFailed => "application-failed", End::RefreshFailed => "refresh-failed", End::DecisionChanged => "decision-changed", End::Abandoned => "abandoned" });
vocabulary!(application, parse_application, ApplicationState, { ApplicationState::Unstarted => "unstarted", ApplicationState::Accepted => "accepted", ApplicationState::Failed => "failed", ApplicationState::Abandoned => "abandoned" });
vocabulary!(kind, parse_kind, Kind, { Kind::Directory => "directory", Kind::File => "file", Kind::Symlink => "symlink", Kind::Other => "other" });
vocabulary!(purpose, parse_purpose, Purpose, { Purpose::Compare => "compare", Purpose::Copy => "copy", Purpose::Remove => "remove", Purpose::Audit => "audit" });
vocabulary!(intent,parse_intent,InstallIntent,{ InstallIntent::ReconcileDifference=>"reconcile-difference",InstallIntent::Renew=>"renew" });
vocabulary!(grant,parse_grant,DestinationGrant,{ DestinationGrant::AbsentOnly=>"absent-only",DestinationGrant::ReplaceLeaf=>"replace-leaf",DestinationGrant::ReplaceTree=>"replace-tree" });
vocabulary!(role,parse_role,InteriorRole,{ InteriorRole::Paired=>"paired",InteriorRole::SourceOnly=>"source-only",InteriorRole::TargetOnly=>"target-only",InteriorRole::PrepareReplacement=>"prepare-replacement",InteriorRole::RemoveForReplacement=>"remove-for-replacement" });

fn action(v: Action) -> String {
    match v {
        Action::PreserveTree => "preserve",
        Action::KeepLeaf => "keep-leaf",
        Action::InstallLeaf => "install-leaf",
        Action::RemoveLeaf => "remove-leaf",
        Action::ConfirmAbsent => "confirm-absent",
        Action::ReplaceWithLeaf => "replace-with-leaf",
        Action::ReplaceWithDirectory { descend: false } => "replace-with-directory/stop",
        Action::ReplaceWithDirectory { descend: true } => "replace-with-directory/descend",
        Action::Directory {
            container: Container::Keep,
            descend: false,
        } => "directory/keep/stop",
        Action::Directory {
            container: Container::Keep,
            descend: true,
        } => "directory/keep/descend",
        Action::Directory {
            container: Container::Create,
            descend: false,
        } => "directory/create/stop",
        Action::Directory {
            container: Container::Create,
            descend: true,
        } => "directory/create/descend",
        Action::Directory {
            container: Container::CreateIfNeeded,
            descend: false,
        } => "directory/create-if-needed/stop",
        Action::Directory {
            container: Container::CreateIfNeeded,
            descend: true,
        } => "directory/create-if-needed/descend",
        Action::Directory {
            container: Container::Remove,
            descend: false,
        } => "directory/remove/stop",
        Action::Directory {
            container: Container::Remove,
            descend: true,
        } => "directory/remove/descend",
    }
    .into()
}
fn parse_action(v: &str) -> Result<Action> {
    Ok(match v {
        "preserve" => Action::PreserveTree,
        "keep-leaf" => Action::KeepLeaf,
        "install-leaf" => Action::InstallLeaf,
        "remove-leaf" => Action::RemoveLeaf,
        "confirm-absent" => Action::ConfirmAbsent,
        "replace-with-leaf" => Action::ReplaceWithLeaf,
        "replace-with-directory/stop" => Action::ReplaceWithDirectory { descend: false },
        "replace-with-directory/descend" => Action::ReplaceWithDirectory { descend: true },
        "directory/keep/stop" => Action::Directory {
            container: Container::Keep,
            descend: false,
        },
        "directory/keep/descend" => Action::Directory {
            container: Container::Keep,
            descend: true,
        },
        "directory/create/stop" => Action::Directory {
            container: Container::Create,
            descend: false,
        },
        "directory/create/descend" => Action::Directory {
            container: Container::Create,
            descend: true,
        },
        "directory/create-if-needed/stop" => Action::Directory {
            container: Container::CreateIfNeeded,
            descend: false,
        },
        "directory/create-if-needed/descend" => Action::Directory {
            container: Container::CreateIfNeeded,
            descend: true,
        },
        "directory/remove/stop" => Action::Directory {
            container: Container::Remove,
            descend: false,
        },
        "directory/remove/descend" => Action::Directory {
            container: Container::Remove,
            descend: true,
        },
        _ => return Err(format!("unknown action: {v}").into()),
    })
}

fn paths(v: &[PathBuf]) -> Vec<NativeString> {
    v.iter().map(|p| native_path(p)).collect()
}
fn read_paths(v: Vec<NativeString>) -> Result<Vec<PathBuf>> {
    v.into_iter().map(|p| Ok(p.into_os()?.into())).collect()
}

impl From<&Summary> for Observation {
    fn from(v: &Summary) -> Self {
        Self {
            scoped: v.scoped,
            excluded: v.excluded,
            planned: v.planned,
            deviated: v.deviated,
            retained: v.retained,
            comparator: v.comparator.as_ref().map(Meaning::from),
            equal: v.equal,
            different: v.different,
            not_assessed: v.not_assessed,
            unsupported: v.unsupported,
            unresolved: v.unresolved,
            stopped_paired: v.stopped_paired,
            files_changed: v.counts.files.changed,
            files_added: v.counts.files.added,
            files_removed: v.counts.files.removed,
            dirs_added: v.counts.dirs.added,
            dirs_removed: v.counts.dirs.removed,
            topmost_divergent: paths(&v.topmost_divergent),
            changed_dirs: paths(&v.changed_dirs),
            examined_directories: v.examined_directories,
            state: scan(v.state),
            pruned: paths(&v.pruned),
        }
    }
}
impl Observation {
    fn decode(self) -> Result<Summary> {
        Ok(Summary {
            scoped: self.scoped,
            excluded: self.excluded,
            planned: self.planned,
            deviated: self.deviated,
            retained: self.retained,
            comparator: self.comparator.map(SemanticId::from),
            equal: self.equal,
            different: self.different,
            not_assessed: self.not_assessed,
            unsupported: self.unsupported,
            unresolved: self.unresolved,
            stopped_paired: self.stopped_paired,
            counts: EntryCounts {
                files: diff_dirs_tree_compare::ChangedAddedRemoved {
                    changed: self.files_changed,
                    added: self.files_added,
                    removed: self.files_removed,
                },
                dirs: diff_dirs_tree_compare::AddedRemoved {
                    added: self.dirs_added,
                    removed: self.dirs_removed,
                },
            },
            topmost_divergent: read_paths(self.topmost_divergent)?,
            changed_dirs: read_paths(self.changed_dirs)?,
            examined_directories: self.examined_directories,
            state: parse_scan(&self.state)?,
            pruned: read_paths(self.pruned)?,
        })
    }
}
impl<C: Clone> From<&DirectoryDecision<C>> for Decision<C> {
    fn from(v: &DirectoryDecision<C>) -> Self {
        Self {
            relative: native_path(v.relative()),
            purpose: purpose(v.purpose()),
            structural_root: v.is_root(),
            domain: "physical-union".into(),
            input_context: v.input_context().clone(),
            policy: v.policy().into(),
            context_schema: v.context_schema().into(),
            comparator: v.comparator().into(),
            scoped: v.coverage().scoped,
            root_excluded: v.is_root() && v.coverage().excluded != 0,
            excluded_names: v.excluded_names().iter().map(|n| NativeString::new(n)).collect(),
            entries: v
                .entries()
                .map(|e| {
                    let (comparison, comparison_code) = match e.comparison {
                        diff_dirs_tree_compare::Comparison::NotAssessed => ("not-assessed", None),
                        diff_dirs_tree_compare::Comparison::Equal => ("equal", None),
                        diff_dirs_tree_compare::Comparison::Different => ("different", None),
                        diff_dirs_tree_compare::Comparison::Unsupported { code } => ("unsupported", Some(code)),
                        diff_dirs_tree_compare::Comparison::Unresolved { code } => ("unresolved", Some(code)),
                    };
                    Entry {
                        name: NativeString::new(e.name()),
                        source: match e.source {
                            SourceObservation::Present(k) => kind(k),
                            SourceObservation::Absent => "absent".into(),
                            SourceObservation::NotRequested => "not-requested".into(),
                        },
                        target: e.target.map(kind),
                        comparison: comparison.into(),
                        comparison_code,
                        baseline: e.baseline.map(action),
                        action: action(e.action),
                        install: e.install.map(|a| Installation {
                            intent: intent(a.intent),
                            destination: grant(a.destination),
                        }),
                        source_absent_removal: e.removal == Some(diff_dirs_tree_compare::RemovalAuthority::SourceAbsent),
                        interior: e.interior.as_ref().map(|i| Inside {
                            role: role(i.role),
                            input: "physical-directory".into(),
                            context: i.context.clone(),
                        }),
                    }
                })
                .collect(),
        }
    }
}
impl<C: Clone + Eq> Decision<C> {
    fn decode(self) -> Result<DirectoryDecision<C>> {
        if self.domain != "physical-union" {
            return Err("example comparison codec only accepts physical unions".into());
        }
        let selected = DirectorySelection::from_parts(SelectionParts {
            relative: self.relative.into_os()?.into(),
            purpose: parse_purpose(&self.purpose)?,
            structural_root: self.structural_root,
            domain: SelectionDomain::PhysicalUnion,
            input_context: self.input_context,
            policy: self.policy.into(),
            context_schema: self.context_schema.into(),
            comparator: self.comparator.into(),
            scoped: self.scoped,
            root_excluded: self.root_excluded,
            excluded_names: self.excluded_names.into_iter().map(NativeString::into_os).collect::<Result<_>>()?,
            entries: self
                .entries
                .into_iter()
                .map(|e| {
                    let comparison = match (e.comparison.as_str(), e.comparison_code) {
                        ("equal", None) => diff_dirs_tree_compare::Comparison::Equal,
                        ("different", None) => diff_dirs_tree_compare::Comparison::Different,
                        ("not-assessed", None) => diff_dirs_tree_compare::Comparison::NotAssessed,
                        ("unsupported", Some(code)) => diff_dirs_tree_compare::Comparison::Unsupported { code },
                        ("unresolved", Some(code)) => diff_dirs_tree_compare::Comparison::Unresolved { code },
                        _ => return Err("invalid comparison outcome/code".into()),
                    };
                    let interior = e
                        .interior
                        .map(|i| -> Result<_> {
                            if i.input != "physical-directory" {
                                return Err("invalid physical interior input".into());
                            }
                            Ok(Interior {
                                role: parse_role(&i.role)?,
                                context: i.context,
                                input: InteriorInput::PhysicalDirectory,
                            })
                        })
                        .transpose()?;
                    let source = match e.source.as_str() {
                        "absent" => SourceObservation::Absent,
                        "not-requested" => SourceObservation::NotRequested,
                        k => SourceObservation::Present(parse_kind(k)?),
                    };
                    Ok(EntryParts {
                        name: e.name.into_os()?,
                        entry: EntrySelection {
                            source,
                            target: e.target.as_deref().map(parse_kind).transpose()?,
                            comparison,
                            baseline: e.baseline.as_deref().map(parse_action).transpose()?,
                            action: parse_action(&e.action)?,
                            install: e
                                .install
                                .map(|a| -> Result<_> {
                                    Ok(InstallAuthority {
                                        intent: parse_intent(&a.intent)?,
                                        destination: parse_grant(&a.destination)?,
                                    })
                                })
                                .transpose()?,
                            removal: e.source_absent_removal.then_some(diff_dirs_tree_compare::RemovalAuthority::SourceAbsent),
                            desired: None,
                            interior,
                        },
                    })
                })
                .collect::<Result<_>>()?,
        })?;
        Ok(Arc::new(selected))
    }
}

impl<C: Clone + Eq> Comparison<C> {
    /// Comparison-only applications use this same explicit v2 DTO without
    /// fabricating an effect barrier or treating selection as completed work.
    pub fn new(parts: &Parts<C>) -> Result<Self> {
        parts.validate()?;
        Ok(Self {
            version: parts.version,
            source: native_path(&parts.bindings.source),
            targets: parts
                .bindings
                .targets
                .iter()
                .map(|b| Binding {
                    id: b.id.0.clone(),
                    root: native_path(&b.root),
                    policy: b.policy.clone(),
                    policy_id: (&b.policy_id).into(),
                    context_schema: (&b.context_schema).into(),
                    comparator: (&b.comparator).into(),
                    initial_context: b.initial_context.clone(),
                })
                .collect(),
            lanes: parts
                .lanes
                .iter()
                .map(|lane| ObservedLane {
                    status: status(lane.status),
                    summary: (&lane.summary).into(),
                })
                .collect(),
            root_pending: parts.root_pending,
            frames: parts
                .frames
                .iter()
                .map(|frame| Frame {
                    sequence: frame.sequence,
                    relative: native_path(&frame.relative),
                    lanes: frame
                        .lanes
                        .iter()
                        .map(|lane| FrameLane {
                            target: lane.target,
                            context: lane.context.clone(),
                            direct_changed: lane.direct_changed,
                            direct_ancestor: lane.direct_ancestor,
                            subtree_changed: lane.subtree_changed,
                        })
                        .collect(),
                    remaining: frame
                        .remaining
                        .iter()
                        .map(|edge| Edge {
                            name: NativeString::new(&edge.name),
                            members: edge
                                .members
                                .iter()
                                .map(|member| Member {
                                    target: member.target,
                                    context: member.context.clone(),
                                })
                                .collect(),
                        })
                        .collect(),
                })
                .collect(),
            window: parts.window.as_ref().map(|window| Window {
                sequence: window.sequence,
                successful: window.successful.clone(),
                pruned: window.pruned.clone(),
            }),
            next_sequence: parts.next_sequence,
        })
    }
    pub fn decode(self) -> Result<Parts<C>> {
        let parts = Parts {
            version: self.version,
            bindings: Bindings {
                source: self.source.into_os()?.into(),
                targets: self
                    .targets
                    .into_iter()
                    .map(|binding| {
                        Ok(TargetBinding {
                            id: binding.id.as_str().into(),
                            root: binding.root.into_os()?.into(),
                            policy: binding.policy,
                            policy_id: binding.policy_id.into(),
                            context_schema: binding.context_schema.into(),
                            comparator: binding.comparator.into(),
                            initial_context: binding.initial_context,
                        })
                    })
                    .collect::<Result<_>>()?,
            },
            lanes: self
                .lanes
                .into_iter()
                .map(|lane| {
                    Ok(LaneState {
                        status: parse_status(&lane.status)?,
                        summary: lane.summary.decode()?,
                    })
                })
                .collect::<Result<_>>()?,
            root_pending: self.root_pending,
            frames: self
                .frames
                .into_iter()
                .map(|frame| {
                    Ok(GroupFrame {
                        sequence: frame.sequence,
                        relative: frame.relative.into_os()?.into(),
                        lanes: frame
                            .lanes
                            .into_iter()
                            .map(|lane| LaneFrame {
                                target: lane.target,
                                context: lane.context,
                                direct_changed: lane.direct_changed,
                                direct_ancestor: lane.direct_ancestor,
                                subtree_changed: lane.subtree_changed,
                            })
                            .collect(),
                        remaining: frame
                            .remaining
                            .into_iter()
                            .map(|edge| {
                                Ok(ChildEdge {
                                    name: edge.name.into_os()?,
                                    members: edge
                                        .members
                                        .into_iter()
                                        .map(|member| ChildMember {
                                            target: member.target,
                                            context: member.context,
                                        })
                                        .collect(),
                                })
                            })
                            .collect::<Result<VecDeque<_>>>()?,
                    })
                })
                .collect::<Result<_>>()?,
            window: self.window.map(|window| ControlWindow {
                sequence: window.sequence,
                successful: window.successful,
                pruned: window.pruned,
            }),
            next_sequence: self.next_sequence,
        };
        parts.validate()?;
        Ok(parts)
    }
}

impl<C: Clone + Eq> Snapshot<C> {
    pub fn new(parts: &SyncParts<C>, policies: Vec<PolicyConfig>) -> Result<Self> {
        validate_config(parts, &policies)?;
        let p = &parts.comparison;
        Ok(Self {
            schema: "diff-dirs.reconcile-example".into(),
            version: 4,
            policies,
            state: State {
                version: parts.version,
                copy_policies: parts.copy_policies.iter().copied().map(copy).collect(),
                postchecks: parts.postchecks.iter().copied().map(postcheck).collect(),
                lanes: parts
                    .lanes
                    .iter()
                    .map(|t| Totals {
                        applications: t.applications,
                        planned: t.planned,
                        completed: (&t.completed).into(),
                        opaque_deviated: t.opaque_deviated,
                        additional_retained: t.additional_retained,
                        excluded: t.excluded,
                        end: t.end.map(end),
                        pruned: t.pruned,
                    })
                    .collect(),
                current: parts.current.as_ref().map(|b| PendingGroup {
                    sequence: b.sequence,
                    applications: b
                        .applications
                        .iter()
                        .map(|a| PendingApplication {
                            target: a.target,
                            decision: (&a.decision).into(),
                            state: application(a.state),
                        })
                        .collect(),
                }),
                comparison: Comparison::new(p)?,
            },
        })
    }
    pub fn decode(self) -> Result<(SyncParts<C>, Vec<PolicyConfig>)> {
        if self.schema != "diff-dirs.reconcile-example" || self.version != 4 {
            return Err("unsupported reconcile transport schema/version".into());
        }
        let s = self.state;
        let p = s.comparison;
        let parts = SyncParts {
            version: s.version,
            copy_policies: s.copy_policies.iter().map(|v| parse_copy(v)).collect::<Result<_>>()?,
            postchecks: s.postchecks.iter().map(|v| parse_postcheck(v)).collect::<Result<_>>()?,
            lanes: s
                .lanes
                .into_iter()
                .map(|t| {
                    Ok(LaneTotals {
                        applications: t.applications,
                        planned: t.planned,
                        completed: t.completed.into(),
                        opaque_deviated: t.opaque_deviated,
                        additional_retained: t.additional_retained,
                        excluded: t.excluded,
                        end: t.end.as_deref().map(parse_end).transpose()?,
                        pruned: t.pruned,
                    })
                })
                .collect::<Result<_>>()?,
            current: s
                .current
                .map(|b| {
                    Ok::<_, Box<dyn std::error::Error>>(Barrier {
                        sequence: b.sequence,
                        applications: b
                            .applications
                            .into_iter()
                            .map(|a| {
                                Ok(Application {
                                    target: a.target,
                                    decision: a.decision.decode()?,
                                    state: parse_application(&a.state)?,
                                })
                            })
                            .collect::<Result<_>>()?,
                    })
                })
                .transpose()?,
            comparison: p.decode()?,
        };
        validate_config(&parts, &self.policies)?;
        Ok((parts, self.policies))
    }
}

pub fn read(path: &Path) -> Result<(SyncParts, Vec<PolicyConfig>)> {
    read_context(path)
}

pub fn read_context<C: Clone + Eq + serde::de::DeserializeOwned>(path: &Path) -> Result<(SyncParts<C>, Vec<PolicyConfig>)> {
    let mut bytes = Vec::new();
    File::open(path)?.take(MAX_BYTES + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_BYTES {
        return Err("cursor exceeds 32 MiB transport limit".into());
    }
    serde_json::from_slice::<Snapshot<C>>(&bytes)?.decode()
}

/// Resolves only the cursor parent, not tree roots. Also usable after terminal
/// trees have been removed. Must run before effects, not merely before publishing.
pub fn publication_path<C>(path: &Path, parts: &SyncParts<C>) -> Result<PathBuf> {
    let absolute = std::path::absolute(path)?;
    let parent = absolute.parent().ok_or("cursor needs a parent")?;
    let name = absolute.file_name().ok_or("cursor needs a basename")?;
    let resolved = std::fs::canonicalize(parent)?.join(name);
    for root in std::iter::once(&parts.comparison.bindings.source).chain(parts.comparison.bindings.targets.iter().map(|t| &t.root)) {
        if resolved.starts_with(root) {
            return Err("cursor publication must be outside every tree root".into());
        }
    }
    if std::fs::symlink_metadata(&resolved).is_ok_and(|m| m.is_dir()) {
        return Err("cursor publication path is a directory".into());
    }
    Ok(resolved)
}

/// Separate complete-file publication. A failure here cannot roll back prior
/// tree effects. No directory-fsync, crash recovery or exactly-once claim.
pub fn write<C: Clone + Eq + Serialize>(path: &Path, parts: &SyncParts<C>, policies: Vec<PolicyConfig>) -> Result<()> {
    let path = publication_path(path, parts)?;
    let snapshot = Snapshot::new(parts, policies)?;
    let mut bytes = serde_json::to_vec(&snapshot)?;
    bytes.push(b'\n');
    if bytes.len() as u64 > MAX_BYTES {
        return Err("cursor exceeds 32 MiB transport limit".into());
    }
    let mut staged = tempfile::NamedTempFile::new_in(path.parent().ok_or("cursor parent")?)?;
    staged.write_all(&bytes)?;
    staged.as_file().sync_all()?;
    staged.persist(path)?;
    Ok(())
}

#[cfg(test)]
mod action_tests {
    use super::*;
    #[test]
    fn conditional_creation_vocabulary_round_trips_without_losing_descent() {
        for descend in [false, true] {
            let value = Action::Directory {
                container: Container::CreateIfNeeded,
                descend,
            };
            assert_eq!(parse_action(&action(value)).unwrap(), value);
        }
        // Structural legality (including the invalid stop spelling) is enforced
        // by decoded current-group decision validation, not vocabulary parsing.
        assert!(parse_action("directory/create-if-needed/unknown").is_err());
    }
}
