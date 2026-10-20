use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::path::{Path, PathBuf};

use crate::{Error, Manifest, ManifestConfig, ManifestState, add, require};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[cfg_attr(feature = "codec", derive(facet::Facet))]
#[cfg_attr(feature = "cli", derive(serde::Serialize))]
pub struct NativePath {
    pub bytes: Vec<u8>,
}

impl NativePath {
    pub fn new(path: &Path) -> Self {
        Self { bytes: path.as_os_str().as_bytes().to_vec() }
    }

    pub fn to_path_buf(&self) -> PathBuf {
        std::ffi::OsString::from_vec(self.bytes.clone()).into()
    }

    pub fn validate(&self, absolute: bool) -> Result<(), Error> {
        require(!self.bytes.contains(&0), "NUL in path")?;
        require(self.bytes.starts_with(b"/") == absolute, "wrong path kind")?;
        let bytes = if absolute { &self.bytes[1..] } else { &self.bytes[..] };
        if !bytes.is_empty() {
            require(bytes.split(|b| *b == b'/').all(|p| !p.is_empty() && p != b"." && p != b".."), "path must be normalized")?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "codec", derive(facet::Facet))]
#[cfg_attr(feature = "cli", derive(serde::Serialize))]
pub struct Bindings {
    pub source: NativePath,
    pub target: NativePath,
    pub reports: NativePath,
}

impl Bindings {
    /// Lexical validation only. Roots must already be absolute and normalized.
    pub fn new(source: impl AsRef<Path>, target: impl AsRef<Path>, reports: impl AsRef<Path>) -> Result<Self, Error> {
        let value = Self { source: NativePath::new(source.as_ref()), target: NativePath::new(target.as_ref()), reports: NativePath::new(reports.as_ref()) };
        value.validate()?;
        Ok(value)
    }

    pub fn roots(&self) -> [PathBuf; 3] {
        [self.source.to_path_buf(), self.target.to_path_buf(), self.reports.to_path_buf()]
    }

    pub fn validate(&self) -> Result<(), Error> {
        for root in [&self.source, &self.target, &self.reports] { root.validate(true)?; }
        disjoint(&self.roots())
    }
}

pub(crate) fn disjoint(roots: &[PathBuf; 3]) -> Result<(), Error> {
    for i in 0..3 {
        for j in (i + 1)..3 {
            require(!roots[i].starts_with(&roots[j]) && !roots[j].starts_with(&roots[i]), "roots must be distinct and non-nested")?;
        }
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "codec", derive(facet::Facet))]
#[cfg_attr(feature = "cli", derive(serde::Serialize))]
#[repr(C)]
pub enum Kind { Directory, File }

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "codec", derive(facet::Facet))]
#[cfg_attr(feature = "cli", derive(serde::Serialize))]
pub struct Node {
    pub relative: NativePath,
    pub kind: Kind,
    pub ensure: bool,
}

impl Node {
    pub fn validate(&self) -> Result<(), Error> {
        self.relative.validate(false)?;
        require(self.kind == Kind::Directory || (!self.ensure && !self.relative.bytes.is_empty()), "invalid leaf node")?;
        if self.kind == Kind::Directory { report_name(&self.relative)?; }
        require(!self.relative.bytes.is_empty() || !self.ensure, "root cannot need creation")
    }
}

/// Fixed injective direct-child mapping; the explicit demo basename cap is 255 bytes.
pub fn report_name(relative: &NativePath) -> Result<String, Error> {
    relative.validate(false)?;
    if relative.bytes.is_empty() { return Ok("root.txt".into()); }
    require(relative.bytes.len() <= 123, "directory report route exceeds the 255-byte demo cap")?;
    let hex: String = relative.bytes.iter().map(|b| format!("{b:02x}")).collect();
    Ok(format!("dir-{hex}.txt"))
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[cfg_attr(feature = "codec", derive(facet::Facet))]
#[cfg_attr(feature = "cli", derive(serde::Serialize))]
pub struct Aggregate {
    pub leaves: u64,
    pub directories: u64,
    pub physical_differences: u64,
    pub retained_entries: u64,
}

impl Aggregate {
    pub(crate) fn fold(self, other: Self) -> Result<Self, Error> {
        Ok(Self {
            leaves: add(self.leaves, other.leaves)?, directories: add(self.directories, other.directories)?,
            physical_differences: add(self.physical_differences, other.physical_differences)?, retained_entries: add(self.retained_entries, other.retained_entries)?,
        })
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[cfg_attr(feature = "codec", derive(facet::Facet))]
#[cfg_attr(feature = "cli", derive(serde::Serialize))]
pub struct Local {
    pub entries: u64,
    pub physical_differences: u64,
    pub retained_entries: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "codec", derive(facet::Facet))]
#[cfg_attr(feature = "cli", derive(serde::Serialize))]
#[repr(C)]
pub enum Phase { Before, Assess, Children, Finish, Leaf }

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "codec", derive(facet::Facet))]
#[cfg_attr(feature = "cli", derive(serde::Serialize))]
pub struct Frame {
    pub node: Node,
    pub phase: Phase,
    pub remaining: Vec<Node>,
    pub local: Option<Local>,
    pub aggregate: Aggregate,
    pub incomplete: bool,
}

impl Frame {
    pub(crate) fn new(node: Node) -> Self {
        let phase = match node.kind { Kind::File => Phase::Leaf, Kind::Directory if node.ensure => Phase::Before, _ => Phase::Assess };
        Self { node, phase, remaining: Vec::new(), local: None, aggregate: Aggregate::default(), incomplete: false }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "codec", derive(facet::Facet))]
#[cfg_attr(feature = "cli", derive(serde::Serialize))]
pub struct Outcome {
    pub aggregate: Aggregate,
    pub incomplete: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "codec", derive(facet::Facet))]
#[cfg_attr(feature = "cli", derive(serde::Serialize))]
pub struct Recipe {
    pub relative: NativePath,
    pub label: String,
    pub ordinal: u64,
    pub leaves: u64,
    pub incomplete: bool,
}

impl Recipe {
    pub fn render(&self) -> Vec<u8> {
        format!("manifest/v1\nlabel_bytes: {}\n{}\nordinal: {}\nleaves: {}\nincomplete: {}\n", self.label.len(), self.label, self.ordinal, self.leaves, self.incomplete).into_bytes()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "codec", derive(facet::Facet))]
#[cfg_attr(feature = "cli", derive(serde::Serialize))]
#[repr(C)]
pub enum Work { EnsureDirectory(Node), SettleSourceFile(Node), InstallReport(Recipe) }

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "codec", derive(facet::Facet))]
#[cfg_attr(feature = "cli", derive(serde::Serialize))]
#[repr(C)]
pub enum Unit { Inspect(Node), Effect(Work) }

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "codec", derive(facet::Facet))]
#[cfg_attr(feature = "cli", derive(serde::Serialize))]
pub struct Pending {
    pub ticket: u64,
    pub unit: Unit,
    pub blocked: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "codec", derive(facet::Facet))]
#[cfg_attr(feature = "cli", derive(serde::Serialize))]
pub struct SavedWalk {
    pub version: u32,
    pub platform: String,
    pub bindings: Bindings,
    pub program: String,
    pub manifest_config: ManifestConfig,
    pub manifest_state: ManifestState,
    pub stack: Vec<Frame>,
    pub pending: Option<Pending>,
    pub next_ticket: u64,
    pub terminal: Option<Outcome>,
}

impl SavedWalk {
    pub fn manifest_config(&self) -> &ManifestConfig { &self.manifest_config }

    /// Data-only shape/consistency checks. This cannot prove past physical effects.
    pub fn validate(&self) -> Result<(), Error> {
        require(self.version == 1 && self.platform == "unix", "unsupported walking version/platform")?;
        require(self.program == crate::DesiredCopy::v1().descriptor(), "unsupported desired-copy descriptor")?;
        self.bindings.validate()?;
        let manifest = Manifest::from_config(self.manifest_config.clone())?;
        require(self.manifest_state.next_ordinal > 0 && self.next_ticket > 0, "zero ordinal/ticket")?;
        require((self.terminal.is_some()) == self.stack.is_empty(), "terminal/live state mismatch")?;
        if self.terminal.is_some() { return require(self.pending.is_none(), "terminal state has pending work"); }
        require(self.stack[0].node.relative.bytes.is_empty() && self.stack[0].node.kind == Kind::Directory, "missing root frame")?;
        let mut combined = Aggregate::default();
        for (i, frame) in self.stack.iter().enumerate() {
            frame.node.validate()?;
            combined = combined.fold(frame.aggregate)?;
            let assessed = matches!(frame.phase, Phase::Children | Phase::Finish);
            require(frame.local.is_some() == assessed, "phase/local mismatch")?;
            require((frame.phase == Phase::Leaf) == (frame.node.kind == Kind::File), "phase/kind mismatch")?;
            require(frame.phase != Phase::Before || frame.node.ensure, "before without creation goal")?;
            if let Some(local) = frame.local {
                require(local.physical_differences <= local.entries && local.retained_entries <= local.entries, "invalid local counts")?;
                require(frame.aggregate.directories > 0 && frame.aggregate.physical_differences >= local.physical_differences && frame.aggregate.retained_entries >= local.retained_entries, "local aggregate mismatch")?;
                require(frame.remaining.len() as u64 <= local.entries - local.retained_entries, "too many remaining children")?;
            } else {
                require(frame.aggregate == Aggregate::default(), "unassessed frame has aggregate")?;
            }
            require(frame.phase == Phase::Children || frame.remaining.is_empty(), "children outside children phase")?;
            let mut previous: Option<&[u8]> = None;
            for child in &frame.remaining {
                child.validate()?;
                let parent = child.relative.to_path_buf().parent().map(Path::to_owned);
                require(parent.as_deref() == Some(frame.node.relative.to_path_buf().as_path()), "child not immediate")?;
                if let Some(p) = previous { require(p < child.relative.bytes.as_slice(), "children not sorted/unique")?; }
                previous = Some(&child.relative.bytes);
            }
            if let Some(child) = self.stack.get(i + 1) {
                require(frame.phase == Phase::Children && frame.remaining.first() == Some(&child.node), "active front-edge mismatch")?;
                require(!child.incomplete || frame.incomplete, "incomplete flag not propagated")?;
            }
        }
        if let Some(pending) = &self.pending {
            require(pending.ticket > 0 && pending.ticket < self.next_ticket, "invalid pending ticket progression")?;
            let top = self.stack.last().expect("live stack");
            require(Some(pending.unit.clone()) == expected_unit(top, &manifest, &self.manifest_state), "pending unit/phase/recipe mismatch")?;
        }
        Ok(())
    }
}

pub(crate) fn expected_unit(frame: &Frame, manifest: &Manifest, state: &ManifestState) -> Option<Unit> {
    Some(match frame.phase {
        Phase::Before => Unit::Effect(Work::EnsureDirectory(frame.node.clone())),
        Phase::Assess => Unit::Inspect(frame.node.clone()),
        Phase::Leaf => Unit::Effect(Work::SettleSourceFile(frame.node.clone())),
        Phase::Finish => Unit::Effect(Work::InstallReport(manifest.recipe(state, frame))),
        Phase::Children => return None,
    })
}
