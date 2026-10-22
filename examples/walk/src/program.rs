use diff_dirs_tree_api::{Action, DirectoryPlan};

use crate::{Error, Frame, Kind, Local, NativePath, Node, Recipe, add, require};

#[derive(Clone, Copy, Debug)]
pub struct DesiredCopy;

impl DesiredCopy {
    pub fn v1() -> Self {
        Self
    }
    pub fn descriptor(&self) -> &'static str {
        "desired-copy/v1;exact-bytes;retain-extras;refuse-shapes;reports-hex-255"
    }

    pub(crate) fn lower(&self, node: &Node, plan: &DirectoryPlan, bindings: &crate::Bindings) -> Result<(Local, Vec<Node>), Error> {
        let operations = plan.operations();
        require(
            operations.dir() == node.relative.to_path_buf()
                && operations.source_root() == bindings.source.to_path_buf()
                && operations.target_root() == bindings.target.to_path_buf(),
            "inspection coordinates do not match request",
        )?;
        require(
            !operations.is_root() && plan.excluded() == 0,
            "expected complete immediate-directory inspection",
        )?;
        let mut local = Local::default();
        let mut children = Vec::new();
        for entry in operations.entries() {
            local.entries = add(local.entries, 1)?;
            local.physical_differences = add(local.physical_differences, u64::from(entry.is_different()))?;
            let Some(source) = entry.source() else {
                require(
                    entry.action() == Some(Action::PreserveTree) || entry.action() == entry.baseline(),
                    "unexpected target-only steering",
                )?;
                local.retained_entries = add(local.retained_entries, 1)?;
                continue;
            };
            require(entry.action() == entry.baseline(), "unexpected source-present steering")?;
            let source = source.metadata();
            require(source.is_file() || source.is_dir(), "source links/specials are outside the walking domain")?;
            if let Some(target) = entry.target() {
                let target = target.metadata();
                require(
                    (source.is_dir() && target.is_dir()) || (source.is_file() && target.is_file()),
                    "target shape collision or unsupported admitted target",
                )?;
            }
            let child = Node {
                relative: NativePath::new(entry.relative_path()),
                kind: if source.is_dir() { Kind::Directory } else { Kind::File },
                ensure: source.is_dir() && entry.target().is_none(),
            };
            child.validate()?;
            children.push(child);
        }
        Ok((local, children))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "codec", derive(facet::Facet))]
#[cfg_attr(feature = "cli", derive(serde::Serialize))]
pub struct ManifestConfig {
    pub version: u32,
    pub label: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "codec", derive(facet::Facet))]
#[cfg_attr(feature = "cli", derive(serde::Serialize))]
pub struct ManifestState {
    pub next_ordinal: u64,
}

/// Executable rendering/config only. The ordinal has exactly one owner in SavedWalk.
#[derive(Clone, Debug, bon::Builder)]
pub struct Manifest {
    pub label: String,
}

impl Manifest {
    pub fn config(&self) -> ManifestConfig {
        ManifestConfig {
            version: 1,
            label: self.label.clone(),
        }
    }
    pub fn from_config(config: ManifestConfig) -> Result<Self, Error> {
        require(config.version == 1, "unsupported manifest version")?;
        Ok(Self { label: config.label })
    }
    pub fn recipe(&self, state: &ManifestState, frame: &Frame) -> Recipe {
        Recipe {
            relative: frame.node.relative.clone(),
            label: self.label.clone(),
            ordinal: state.next_ordinal,
            leaves: frame.aggregate.leaves,
            incomplete: frame.incomplete,
        }
    }
}
