use super::{DesiredLeaf, LeafExpectation, LeafKind};
use crate::Failure;
use diff_dirs_tree_inspect::{DesiredInput, LeafId, ManifestId, NativePlatform, NodeId};
use std::ffi::{OsStr, OsString};
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;

#[derive(Debug)]
pub(super) struct Node {
    pub relative: PathBuf,
    pub leaf: Option<LeafId>,
    pub children: Vec<(OsString, NodeId)>,
    pub leaves: std::ops::Range<usize>,
}
#[derive(Debug)]
pub(super) struct Index {
    pub id: ManifestId,
    pub nodes: Vec<Node>,
    pub leaves: Vec<DesiredLeaf>,
}
impl Index {
    fn build(specs: impl IntoIterator<Item = DesiredLeaf>, owned: bool) -> Result<Arc<Self>, Failure> {
        if !cfg!(any(unix, windows)) {
            return Err(Failure::invalid(
                "prepare manifest",
                Path::new(""),
                "declared native-name encoding is supported on Unix and Windows only",
            ));
        }
        let mut leaves = specs
            .into_iter()
            .map(|mut spec| {
                let path = &spec.relative;
                if path.as_os_str().is_empty()
                    || path.as_os_str().as_encoded_bytes().contains(&0)
                    || path.components().any(|c| !matches!(c, Component::Normal(_)))
                {
                    return Err(Failure::invalid("prepare manifest", path, "expected a nonempty normal relative leaf path"));
                }
                spec.relative = path.components().collect();
                if let LeafExpectation::Exact { kind, executable: Some(_) } = spec.expected
                    && (kind != LeafKind::RegularFile || !cfg!(unix))
                {
                    return Err(Failure::invalid("prepare manifest", &spec.relative, "unsupported executable expectation"));
                }
                Ok(spec)
            })
            .collect::<Result<Vec<_>, _>>()?;
        fn components(path: &Path) -> impl Iterator<Item = &[u8]> {
            path.components().map(|c| c.as_os_str().as_encoded_bytes())
        }
        leaves.sort_unstable_by(|a, b| components(&a.relative).cmp(components(&b.relative)));
        for pair in leaves.windows(2) {
            if pair[1].relative.starts_with(&pair[0].relative) {
                return Err(Failure::invalid(
                    "prepare manifest",
                    &pair[1].relative,
                    "duplicate alias or leaf/ancestor conflict",
                ));
            }
        }
        let id = digest(&leaves, owned);
        let mut result = Self {
            id,
            nodes: vec![Node {
                relative: PathBuf::new(),
                leaf: None,
                children: Vec::new(),
                leaves: 0..leaves.len(),
            }],
            leaves,
        };
        for leaf in 0..result.leaves.len() {
            let components: Vec<_> = result.leaves[leaf].relative.components().map(|c| c.as_os_str().to_owned()).collect();
            let mut node = NodeId(0);
            for name in components {
                let next = match result.nodes[node.0 as usize]
                    .children
                    .binary_search_by(|(n, _)| n.as_encoded_bytes().cmp(name.as_encoded_bytes()))
                {
                    Ok(index) => result.nodes[node.0 as usize].children[index].1,
                    Err(index) => {
                        let next = NodeId(result.nodes.len() as u64);
                        let relative = result.nodes[node.0 as usize].relative.join(&name);
                        result.nodes.push(Node {
                            relative,
                            leaf: None,
                            children: Vec::new(),
                            leaves: leaf..leaf + 1,
                        });
                        result.nodes[node.0 as usize].children.insert(index, (name, next));
                        next
                    }
                };
                result.nodes[next.0 as usize].leaves.end = leaf + 1;
                node = next;
            }
            result.nodes[node.0 as usize].leaf = Some(LeafId(leaf as u64));
        }
        Ok(Arc::new(result))
    }
    pub fn node(&self, id: NodeId) -> Option<&Node> {
        self.nodes.get(usize::try_from(id.0).ok()?)
    }
    pub fn find(&self, path: &Path) -> Option<NodeId> {
        let mut node = NodeId(0);
        for component in path.components() {
            node = self.child(node, component.as_os_str())?;
        }
        Some(node)
    }
    pub fn child(&self, node: NodeId, name: &OsStr) -> Option<NodeId> {
        let children = &self.node(node)?.children;
        Some(children[children.binary_search_by(|(n, _)| n.as_encoded_bytes().cmp(name.as_encoded_bytes())).ok()?].1)
    }
    pub fn input(&self, node: NodeId) -> DesiredInput {
        self.node(node)
            .unwrap()
            .leaf
            .map_or(DesiredInput::Directory { node }, |leaf| DesiredInput::Leaf {
                leaf,
                expected: self.leaves[leaf.0 as usize].expected,
            })
    }
    pub fn exact_leaf(&self, relative: &Path) -> Option<LeafId> {
        self.node(self.find(relative)?)?.leaf
    }
}

fn digest(leaves: &[DesiredLeaf], owned: bool) -> ManifestId {
    let platform = if cfg!(windows) { NativePlatform::Windows } else { NativePlatform::Unix };
    let mut hash = blake3::Hasher::new();
    hash.update(if owned { b"diff-dirs.owned\0" } else { b"diff-dirs.desired\0" });
    hash.update(&1u32.to_le_bytes());
    hash.update(&[if cfg!(windows) { 2 } else { 1 }]);
    hash.update(&(leaves.len() as u64).to_le_bytes());
    for leaf in leaves {
        hash.update(&(leaf.relative.components().count() as u64).to_le_bytes());
        for component in leaf.relative.components() {
            #[cfg(windows)]
            let bytes = {
                use std::os::windows::ffi::OsStrExt;
                component.as_os_str().encode_wide().flat_map(u16::to_le_bytes).collect::<Vec<_>>()
            };
            #[cfg(not(windows))]
            let bytes = component.as_os_str().as_encoded_bytes();
            hash.update(&(bytes.len() as u64).to_le_bytes());
            hash.update(bytes.as_ref());
        }
        if !owned {
            let encoding = match leaf.expected {
                LeafExpectation::SupportedLeaf => [0, 0, 0],
                LeafExpectation::Exact { kind, executable } => [
                    1,
                    if kind == LeafKind::RegularFile { 1 } else { 2 },
                    match executable {
                        None => 0,
                        Some(false) => 1,
                        Some(true) => 2,
                    },
                ],
            };
            hash.update(&encoding);
        }
    }
    ManifestId {
        version: 1,
        platform,
        digest: *hash.finalize().as_bytes(),
    }
}

/// Immutable declared desired names/expectations and one shared component index.
/// Construction is filesystem-free; identity is not proof of source contents.
#[derive(Clone, Debug)]
pub struct PreparedLeafList {
    pub(super) index: Arc<Index>,
}
impl PreparedLeafList {
    pub fn new<I, P>(paths: I) -> Result<Self, Failure>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        Self::from_specs(paths.into_iter().map(|path| DesiredLeaf {
            relative: path.as_ref().to_owned(),
            expected: LeafExpectation::SupportedLeaf,
        }))
    }
    pub fn from_specs(leaves: impl IntoIterator<Item = DesiredLeaf>) -> Result<Self, Failure> {
        Ok(Self {
            index: Index::build(leaves, false)?,
        })
    }
    pub fn id(&self) -> ManifestId {
        self.index.id
    }
    pub fn leaves(&self) -> &[DesiredLeaf] {
        &self.index.leaves
    }
    pub fn len(&self) -> usize {
        self.index.leaves.len()
    }
    pub fn is_empty(&self) -> bool {
        self.index.leaves.is_empty()
    }
}
/// Exact previously-owned leaves and structural prefixes. The caller, not this
/// provider, establishes the truth of that ownership declaration.
#[derive(Clone, Debug)]
pub struct OwnedLeafSet {
    pub(super) index: Arc<Index>,
}
impl OwnedLeafSet {
    pub fn new<I, P>(paths: I) -> Result<Self, Failure>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        Ok(Self {
            index: Index::build(
                paths.into_iter().map(|p| DesiredLeaf {
                    relative: p.as_ref().to_owned(),
                    expected: LeafExpectation::SupportedLeaf,
                }),
                true,
            )?,
        })
    }
    pub fn id(&self) -> ManifestId {
        self.index.id
    }
    pub fn paths(&self) -> impl ExactSizeIterator<Item = &Path> {
        self.index.leaves.iter().map(|l| l.relative.as_path())
    }
    pub fn len(&self) -> usize {
        self.index.leaves.len()
    }
    pub fn is_empty(&self) -> bool {
        self.index.leaves.is_empty()
    }
}
