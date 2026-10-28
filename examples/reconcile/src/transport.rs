//! Explicit version-one application DTOs. These are not derives on runtime
//! objects or an archive of private Rust layouts. Decode, then validate meaning.

use std::collections::VecDeque;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use diff_dirs_transfer_sync::CopyPolicy;
use diff_dirs_transfer_sync::coordinated::{
    Application, ApplicationState, Barrier, DirectoryDecision, End, EntryDecision, Kind, LaneTotals, SyncParts, WorkTotals,
};
use diff_dirs_tree_compare::coordinated::{Bindings, ChildEdge, ControlWindow, GroupFrame, LaneFrame, LaneState, LaneStatus, Parts, TargetBinding};
use diff_dirs_tree_compare::{Action, Container, DirectoryCoverage, EntryCounts, Presence, Purpose, ScanState, Summary};
use serde::{Deserialize, Serialize};

use crate::{PolicyConfig, Result, native::NativeString, native_path, validate_config};

pub const MAX_BYTES: u64 = 32 * 1024 * 1024;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Snapshot {
    pub schema: String,
    pub version: u32,
    pub policies: Vec<PolicyConfig>,
    pub state: State,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct State {
    pub version: u32,
    pub comparison: Comparison,
    pub copy_policies: Vec<String>,
    pub lanes: Vec<Totals>,
    pub current: Option<PendingGroup>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Comparison {
    pub version: u32,
    pub source: NativeString,
    pub targets: Vec<Binding>,
    pub lanes: Vec<ObservedLane>,
    pub root_pending: bool,
    pub frames: Vec<Frame>,
    pub window: Option<Window>,
    pub next_sequence: u64,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Binding {
    pub id: String,
    pub root: NativeString,
    pub policy: String,
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
    pub retained: u64,
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
pub struct Frame {
    pub sequence: u64,
    pub relative: NativeString,
    pub lanes: Vec<FrameLane>,
    pub remaining: Vec<Edge>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FrameLane {
    pub target: usize,
    pub direct_changed: bool,
    pub direct_ancestor: bool,
    pub subtree_changed: bool,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Edge {
    pub name: NativeString,
    pub members: Vec<usize>,
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
    pub recursive_retained: u128,
    pub excluded: u128,
    pub end: Option<String>,
    pub pruned: bool,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PendingGroup {
    pub sequence: u64,
    pub applications: Vec<PendingApplication>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PendingApplication {
    pub target: usize,
    pub decision: Decision,
    pub state: String,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Decision {
    pub relative: NativeString,
    pub purpose: String,
    pub structural_root: bool,
    pub excluded: u64,
    pub scoped: bool,
    pub planned: u64,
    pub retained: u64,
    pub excluded_names: Vec<NativeString>,
    pub entries: Vec<Entry>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    pub name: NativeString,
    pub presence: String,
    pub source: Option<String>,
    pub target: Option<String>,
    pub comparison: Option<bool>,
    pub baseline: Option<String>,
    pub action: String,
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
vocabulary!(end, parse_end, End, { End::ComparisonFailed => "comparison-failed", End::ApplicationFailed => "application-failed", End::RefreshFailed => "refresh-failed", End::DecisionChanged => "decision-changed", End::Abandoned => "abandoned" });
vocabulary!(application, parse_application, ApplicationState, { ApplicationState::Unstarted => "unstarted", ApplicationState::Accepted => "accepted", ApplicationState::Failed => "failed", ApplicationState::Abandoned => "abandoned" });
vocabulary!(kind, parse_kind, Kind, { Kind::Directory => "directory", Kind::File => "file", Kind::Symlink => "symlink", Kind::Other => "other" });
vocabulary!(purpose, parse_purpose, Purpose, { Purpose::Compare => "compare", Purpose::Copy => "copy", Purpose::Remove => "remove", Purpose::Audit => "audit" });
vocabulary!(presence, parse_presence, Presence, { Presence::SourceOnly => "source-only", Presence::Both => "both", Presence::TargetOnly => "target-only" });

fn action(v: Action) -> String {
    match v {
        Action::PreserveTree => "preserve",
        Action::KeepLeaf => "keep-leaf",
        Action::InstallLeaf => "install-leaf",
        Action::RemoveLeaf => "remove-leaf",
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
            retained: v.retained,
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
            retained: self.retained,
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
impl From<&DirectoryDecision> for Decision {
    fn from(v: &DirectoryDecision) -> Self {
        Self {
            relative: native_path(&v.relative),
            purpose: purpose(v.purpose),
            structural_root: v.structural_root,
            excluded: v.coverage.excluded,
            scoped: v.coverage.scoped,
            planned: v.coverage.planned,
            retained: v.coverage.retained,
            excluded_names: v.excluded_names.iter().map(|n| NativeString::new(n)).collect(),
            entries: v
                .entries
                .iter()
                .map(|e| Entry {
                    name: NativeString::new(&e.name),
                    presence: presence(e.presence),
                    source: e.source.map(kind),
                    target: e.target.map(kind),
                    comparison: e.comparison,
                    baseline: e.baseline.map(action),
                    action: action(e.action),
                })
                .collect(),
        }
    }
}
impl Decision {
    fn decode(self) -> Result<DirectoryDecision> {
        Ok(DirectoryDecision {
            relative: self.relative.into_os()?.into(),
            purpose: parse_purpose(&self.purpose)?,
            structural_root: self.structural_root,
            coverage: DirectoryCoverage {
                excluded: self.excluded,
                scoped: self.scoped,
                planned: self.planned,
                retained: self.retained,
            },
            excluded_names: self.excluded_names.into_iter().map(NativeString::into_os).collect::<Result<_>>()?,
            entries: self
                .entries
                .into_iter()
                .map(|e| {
                    Ok(EntryDecision {
                        name: e.name.into_os()?,
                        presence: parse_presence(&e.presence)?,
                        source: e.source.as_deref().map(parse_kind).transpose()?,
                        target: e.target.as_deref().map(parse_kind).transpose()?,
                        comparison: e.comparison,
                        baseline: e.baseline.as_deref().map(parse_action).transpose()?,
                        action: parse_action(&e.action)?,
                    })
                })
                .collect::<Result<_>>()?,
        })
    }
}

impl Snapshot {
    pub fn new(parts: &SyncParts, policies: Vec<PolicyConfig>) -> Result<Self> {
        validate_config(parts, &policies)?;
        let p = &parts.comparison;
        Ok(Self {
            schema: "diff-dirs.reconcile-example".into(),
            version: 1,
            policies,
            state: State {
                version: parts.version,
                copy_policies: parts.copy_policies.iter().copied().map(copy).collect(),
                lanes: parts
                    .lanes
                    .iter()
                    .map(|t| Totals {
                        applications: t.applications,
                        planned: t.planned,
                        completed: (&t.completed).into(),
                        recursive_retained: t.recursive_retained,
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
                comparison: Comparison {
                    version: p.version,
                    source: native_path(&p.bindings.source),
                    targets: p
                        .bindings
                        .targets
                        .iter()
                        .map(|b| Binding {
                            id: b.id.0.clone(),
                            root: native_path(&b.root),
                            policy: b.policy.clone(),
                        })
                        .collect(),
                    lanes: p
                        .lanes
                        .iter()
                        .map(|l| ObservedLane {
                            status: status(l.status),
                            summary: (&l.summary).into(),
                        })
                        .collect(),
                    root_pending: p.root_pending,
                    frames: p
                        .frames
                        .iter()
                        .map(|f| Frame {
                            sequence: f.sequence,
                            relative: native_path(&f.relative),
                            lanes: f
                                .lanes
                                .iter()
                                .map(|l| FrameLane {
                                    target: l.target,
                                    direct_changed: l.direct_changed,
                                    direct_ancestor: l.direct_ancestor,
                                    subtree_changed: l.subtree_changed,
                                })
                                .collect(),
                            remaining: f
                                .remaining
                                .iter()
                                .map(|e| Edge {
                                    name: NativeString::new(&e.name),
                                    members: e.members.clone(),
                                })
                                .collect(),
                        })
                        .collect(),
                    window: p.window.as_ref().map(|w| Window {
                        sequence: w.sequence,
                        successful: w.successful.clone(),
                        pruned: w.pruned.clone(),
                    }),
                    next_sequence: p.next_sequence,
                },
            },
        })
    }
    pub fn decode(self) -> Result<(SyncParts, Vec<PolicyConfig>)> {
        if self.schema != "diff-dirs.reconcile-example" || self.version != 1 {
            return Err("unsupported reconcile transport schema/version".into());
        }
        let s = self.state;
        let p = s.comparison;
        let parts = SyncParts {
            version: s.version,
            copy_policies: s.copy_policies.iter().map(|v| parse_copy(v)).collect::<Result<_>>()?,
            lanes: s
                .lanes
                .into_iter()
                .map(|t| {
                    Ok(LaneTotals {
                        applications: t.applications,
                        planned: t.planned,
                        completed: t.completed.into(),
                        recursive_retained: t.recursive_retained,
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
            comparison: Parts {
                version: p.version,
                bindings: Bindings {
                    source: p.source.into_os()?.into(),
                    targets: p
                        .targets
                        .into_iter()
                        .map(|b| {
                            Ok(TargetBinding {
                                id: b.id.as_str().into(),
                                root: b.root.into_os()?.into(),
                                policy: b.policy,
                            })
                        })
                        .collect::<Result<_>>()?,
                },
                lanes: p
                    .lanes
                    .into_iter()
                    .map(|l| {
                        Ok(LaneState {
                            status: parse_status(&l.status)?,
                            summary: l.summary.decode()?,
                        })
                    })
                    .collect::<Result<_>>()?,
                root_pending: p.root_pending,
                frames: p
                    .frames
                    .into_iter()
                    .map(|f| {
                        Ok(GroupFrame {
                            sequence: f.sequence,
                            relative: f.relative.into_os()?.into(),
                            lanes: f
                                .lanes
                                .into_iter()
                                .map(|l| LaneFrame {
                                    target: l.target,
                                    direct_changed: l.direct_changed,
                                    direct_ancestor: l.direct_ancestor,
                                    subtree_changed: l.subtree_changed,
                                })
                                .collect(),
                            remaining: f
                                .remaining
                                .into_iter()
                                .map(|e| {
                                    Ok(ChildEdge {
                                        name: e.name.into_os()?,
                                        members: e.members,
                                    })
                                })
                                .collect::<Result<VecDeque<_>>>()?,
                        })
                    })
                    .collect::<Result<_>>()?,
                window: p.window.map(|w| ControlWindow {
                    sequence: w.sequence,
                    successful: w.successful,
                    pruned: w.pruned,
                }),
                next_sequence: p.next_sequence,
            },
        };
        validate_config(&parts, &self.policies)?;
        Ok((parts, self.policies))
    }
}

pub fn read(path: &Path) -> Result<(SyncParts, Vec<PolicyConfig>)> {
    let mut bytes = Vec::new();
    File::open(path)?.take(MAX_BYTES + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_BYTES {
        return Err("cursor exceeds 32 MiB transport limit".into());
    }
    serde_json::from_slice::<Snapshot>(&bytes)?.decode()
}

/// Resolves only the cursor parent, not tree roots. Also usable after terminal
/// trees have been removed. Must run before effects, not merely before publishing.
pub fn publication_path(path: &Path, parts: &SyncParts) -> Result<PathBuf> {
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
pub fn write(path: &Path, parts: &SyncParts, policies: Vec<PolicyConfig>) -> Result<()> {
    let path = publication_path(path, parts)?;
    let snapshot = Snapshot::new(parts, policies)?;
    let bytes = serde_json::to_vec(&snapshot)?;
    if bytes.len() as u64 > MAX_BYTES {
        return Err("cursor exceeds 32 MiB transport limit".into());
    }
    let mut staged = tempfile::NamedTempFile::new_in(path.parent().ok_or("cursor parent")?)?;
    staged.write_all(&bytes)?;
    staged.write_all(b"\n")?;
    staged.as_file().sync_all()?;
    staged.persist(path)?;
    Ok(())
}
