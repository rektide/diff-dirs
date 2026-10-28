use std::ffi::OsString;
use std::path::{Component, Path, PathBuf};

use diff_dirs_tree_inspect::{Action, Container, DirectoryCoverage, DirectoryPlan, Facts, Presence, Purpose};

use super::{Error, require};

/// Acquired no-follow kind, not a saved Metadata value or enumeration hint.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Directory,
    File,
    Symlink,
    Other,
}

impl Kind {
    fn of(facts: &Facts) -> Self {
        let m = facts.metadata();
        if m.is_dir() {
            Self::Directory
        } else if m.is_file() {
            Self::File
        } else if m.is_symlink() {
            Self::Symlink
        } else {
            Self::Other
        }
    }
}

/// An admitted slot, including unchanged leaves and late-preserved entries.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EntryDecision {
    pub name: OsString,
    pub presence: Presence,
    pub source: Option<Kind>,
    pub target: Option<Kind>,
    pub comparison: Option<bool>,
    pub baseline: Option<Action>,
    pub action: Action,
}

/// Exact metadata-free immediate decision. Excluded names have deliberately not
/// acquired kinds. Together with entries they describe the complete selected union.
/// Equality validates decisions, not historical metadata, bytes or inode identity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DirectoryDecision {
    pub relative: PathBuf,
    pub purpose: Purpose,
    pub structural_root: bool,
    pub coverage: DirectoryCoverage,
    pub excluded_names: Vec<OsString>,
    pub entries: Vec<EntryDecision>,
}

impl DirectoryDecision {
    pub(super) fn of(plan: &DirectoryPlan) -> Self {
        let ops = plan.operations();
        Self {
            relative: ops.dir().to_owned(),
            purpose: ops.purpose(),
            structural_root: ops.is_root(),
            coverage: DirectoryCoverage {
                excluded: plan.excluded(),
                scoped: plan.steered(),
                planned: plan.work(),
                retained: plan.retained(),
            },
            excluded_names: plan.excluded_names().to_vec(),
            entries: ops
                .entries()
                .iter()
                .map(|e| EntryDecision {
                    name: e.name().to_owned(),
                    presence: e.presence(),
                    source: e.source().map(Kind::of),
                    target: e.target().map(Kind::of),
                    comparison: e.comparison(),
                    baseline: e.baseline(),
                    action: e.action().expect("validated action"),
                })
                .collect(),
        }
    }

    pub fn validate(&self) -> Result<(), Error> {
        require(normal_relative(&self.relative), "invalid decision coordinate")?;
        require(self.purpose == Purpose::Compare, "sync requires comparison decisions")?;
        require(
            self.entries.windows(2).all(|p| p[0].name.as_encoded_bytes() < p[1].name.as_encoded_bytes()),
            "unordered decision entries",
        )?;
        require(
            self.excluded_names.windows(2).all(|p| p[0].as_encoded_bytes() < p[1].as_encoded_bytes()),
            "unordered excluded names",
        )?;
        require(
            self.excluded_names.iter().all(|n| native_name(n) && self.entries.iter().all(|e| e.name != *n)),
            "invalid or overlapping excluded name",
        )?;
        if self.structural_root {
            require(
                self.relative.as_os_str().is_empty() && self.entries.len() == 1 && self.excluded_names.is_empty(),
                "invalid structural root decision",
            )?;
            require(
                self.coverage.excluded <= 1 && (self.entries[0].action == Action::PreserveTree || self.coverage.excluded == 0),
                "invalid root exclusion",
            )?;
            require(
                self.entries[0].source == Some(Kind::Directory) && self.entries[0].target == Some(Kind::Directory),
                "coordinated sync roots must both be directories",
            )?;
        } else {
            require(self.coverage.excluded == self.excluded_names.len() as u64, "excluded count mismatch")?;
        }
        let mut work = 0;
        let mut retained = 0;
        for e in &self.entries {
            // A filesystem root singleton can have '/' as its name.
            require(
                !e.name.as_encoded_bytes().contains(&0) && (self.structural_root || native_name(&e.name)),
                "invalid decision basename",
            )?;
            let presence = match (e.source.is_some(), e.target.is_some()) {
                (true, true) => Presence::Both,
                (true, false) => Presence::SourceOnly,
                (false, true) => Presence::TargetOnly,
                _ => return Err(Error::InvalidState("decision has no present side")),
            };
            require(e.presence == presence, "decision presence mismatch")?;
            let sd = e.source == Some(Kind::Directory);
            let td = e.target == Some(Kind::Directory);
            let sl = matches!(e.source, Some(Kind::File | Kind::Symlink));
            let tl = matches!(e.target, Some(Kind::File | Kind::Symlink));
            let special = e.source == Some(Kind::Other) || e.target == Some(Kind::Other);
            let baseline = if special {
                None
            } else {
                Some(match (e.source.is_some(), e.target.is_some(), sd, td) {
                    (true, true, true, true) => Action::Directory {
                        container: Container::Keep,
                        descend: true,
                    },
                    (true, true, true, false) => Action::ReplaceWithDirectory { descend: true },
                    (true, true, false, true) => Action::ReplaceWithLeaf,
                    (true, _, true, _) => Action::Directory {
                        container: Container::Create,
                        descend: true,
                    },
                    (_, true, _, true) => Action::Directory {
                        container: Container::Remove,
                        descend: true,
                    },
                    (true, _, _, _) if e.comparison == Some(true) => Action::InstallLeaf,
                    (false, true, _, _) => Action::RemoveLeaf,
                    _ => Action::KeepLeaf,
                })
            };
            require(e.baseline == baseline && e.comparison.is_none() == special, "invalid baseline or comparison")?;
            if !special && (e.presence != Presence::Both || e.source != e.target) {
                require(e.comparison == Some(true), "different shape marked equal")?;
            }
            if sd && td {
                require(e.comparison == Some(false), "paired directories marked different")?;
            }
            let valid = match e.action {
                Action::PreserveTree => true,
                Action::KeepLeaf => tl || (e.target.is_none() && sl),
                Action::InstallLeaf => sl && !td && e.comparison == Some(true),
                Action::RemoveLeaf => e.source.is_none() && tl,
                Action::ReplaceWithLeaf => sl && td,
                Action::ReplaceWithDirectory { .. } => sd && tl,
                Action::Directory {
                    container: Container::Create, ..
                } => sd && e.target.is_none(),
                Action::Directory {
                    container: Container::CreateIfNeeded,
                    descend,
                } => sd && e.target.is_none() && descend,
                Action::Directory {
                    container: Container::Keep, ..
                } => td,
                Action::Directory {
                    container: Container::Remove,
                    descend,
                } => e.source.is_none() && td && descend,
            };
            require(valid && (!special || e.action == Action::PreserveTree), "invalid saved action")?;
            work += u64::from(matches!(
                e.action,
                Action::InstallLeaf
                    | Action::RemoveLeaf
                    | Action::ReplaceWithLeaf
                    | Action::ReplaceWithDirectory { .. }
                    | Action::Directory {
                        container: Container::Create | Container::CreateIfNeeded | Container::Remove,
                        ..
                    }
            ));
            retained += u64::from(Some(e.action) != e.baseline);
        }
        require(
            self.coverage.planned == work && self.coverage.retained == retained,
            "decision coverage mismatch",
        )
    }

    pub(super) fn paired_children(&self) -> impl Iterator<Item = &std::ffi::OsStr> {
        self.entries
            .iter()
            .filter(|e| {
                !self.structural_root
                    && e.source == Some(Kind::Directory)
                    && e.target == Some(Kind::Directory)
                    && matches!(e.action, Action::Directory { descend: true, .. })
            })
            .map(|e| e.name.as_os_str())
    }
}

pub(super) fn normal_relative(path: &Path) -> bool {
    !path.as_os_str().as_encoded_bytes().contains(&0)
        && path.components().all(|c| matches!(c, Component::Normal(_)))
        && path.components().collect::<PathBuf>().as_os_str() == path.as_os_str()
}

pub(super) fn native_name(name: &std::ffi::OsStr) -> bool {
    !name.as_encoded_bytes().contains(&0)
        && matches!(Path::new(name).components().next(), Some(Component::Normal(_)))
        && Path::new(name).components().count() == 1
        && Path::new(name).file_name() == Some(name)
}
