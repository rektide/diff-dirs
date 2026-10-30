//! Explicit comparison meaning and finite physical fact demand.

use std::path::Path;
use std::sync::LazyLock;

use crate::{Available, CheckError, Demand, EntryFacts, Kind, Shape, SideDemands, SourceObservation};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SemanticId {
    pub family: String,
    pub version: u32,
    pub parameters: Vec<u8>,
}
impl SemanticId {
    pub fn valid(&self) -> bool {
        !self.family.is_empty() && self.version != 0
    }
    pub fn new(family: impl Into<String>, version: u32, parameters: Vec<u8>) -> Self {
        Self {
            family: family.into(),
            version,
            parameters,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Comparison {
    NotAssessed,
    Equal,
    Different,
    Unsupported { code: u32 },
    Unresolved { code: u32 },
}
impl Comparison {
    /// The old full-visitor projection is defined only for assessed equality/difference.
    pub fn legacy(self) -> Option<bool> {
        match self {
            Self::Equal => Some(false),
            Self::Different => Some(true),
            _ => None,
        }
    }
}
pub const UNSUPPORTED_PHYSICAL_KIND: u32 = 1;

#[derive(Clone, Copy, Debug)]
pub struct ComparisonInput<'a> {
    pub shape: Shape,
    pub facts: EntryFacts<'a>,
    pub source_path: &'a Path,
    pub target_path: &'a Path,
}
/// Only compatible leaves reach `compare`; structural conclusions belong to inspection.
pub trait Comparator: Sync {
    fn identity(&self) -> &SemanticId;
    fn demand(&self, shape: Shape) -> SideDemands;
    fn compare(&self, input: ComparisonInput<'_>) -> Result<Comparison, CheckError>;
}

impl<M: Comparator + ?Sized> Comparator for &M {
    fn identity(&self) -> &SemanticId {
        (**self).identity()
    }
    fn demand(&self, shape: Shape) -> SideDemands {
        (**self).demand(shape)
    }
    fn compare(&self, input: ComparisonInput<'_>) -> Result<Comparison, CheckError> {
        (**self).compare(input)
    }
}
#[derive(Clone, Copy, Debug, Default)]
pub struct StatKey;
#[derive(Clone, Copy, Debug, Default)]
pub struct SizeAndLink;
/// Selected regular-file metadata checks. Links always use literal targets.
/// Parsing rejects empty, duplicate and unknown names; display is canonical.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StatChecks(u8);
impl StatChecks {
    pub const MTIME: Self = Self(1);
    pub const SIZE: Self = Self(2);
    pub const MODE: Self = Self(4);
    pub fn contains(self, check: Self) -> bool {
        self.0 & check.0 == check.0
    }
}
impl Default for StatChecks {
    fn default() -> Self {
        Self(3)
    }
}
impl std::str::FromStr for StatChecks {
    type Err = &'static str;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let mut bits = 0;
        for name in value.split(',') {
            let bit = match name {
                "mtime" => 1,
                "size" => 2,
                "mode" => 4,
                _ => return Err("checks must be a nonempty comma-separated selection of mtime,size,mode"),
            };
            if bits & bit != 0 {
                return Err("duplicate stat check");
            }
            bits |= bit;
        }
        Ok(Self(bits))
    }
}
impl std::fmt::Display for StatChecks {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut separator = "";
        for (check, name) in [(Self::MTIME, "mtime"), (Self::SIZE, "size"), (Self::MODE, "mode")] {
            if self.contains(check) {
                write!(f, "{separator}{name}")?;
                separator = ",";
            }
        }
        Ok(())
    }
}
/// Metadata heuristic, not a content or storage-sharing proof. Unix mode checks
/// compare permission and special bits (0o7777); other platforms use readonly.
#[derive(Clone, Debug)]
pub struct StatComparator {
    checks: StatChecks,
    identity: SemanticId,
}
impl StatComparator {
    pub fn new(checks: StatChecks) -> Self {
        Self {
            checks,
            identity: SemanticId::new("diff-dirs.stat-checks", 1, vec![checks.0]),
        }
    }
    pub fn checks(&self) -> StatChecks {
        self.checks
    }
}
impl Default for StatComparator {
    fn default() -> Self {
        Self::new(StatChecks::default())
    }
}
static STAT_ID: LazyLock<SemanticId> = LazyLock::new(|| SemanticId::new("diff-dirs.stat-key", 1, Vec::new()));
static SIZE_ID: LazyLock<SemanticId> = LazyLock::new(|| SemanticId::new("diff-dirs.size-and-link", 1, Vec::new()));
fn leaf_demand(shape: Shape) -> SideDemands {
    let demand = match (shape.source, shape.target) {
        (SourceObservation::Present(Kind::File), Some(Kind::File)) => Demand {
            attributes: true,
            literal_link: false,
        },
        (SourceObservation::Present(Kind::Symlink), Some(Kind::Symlink)) => Demand {
            attributes: false,
            literal_link: true,
        },
        _ => Demand::NONE,
    };
    SideDemands {
        source: demand,
        target: demand,
    }
}
impl Comparator for StatKey {
    fn identity(&self) -> &SemanticId {
        &STAT_ID
    }
    fn demand(&self, shape: Shape) -> SideDemands {
        leaf_demand(shape)
    }
    fn compare(&self, input: ComparisonInput<'_>) -> Result<Comparison, CheckError> {
        compare_leaves(input, StatChecks::default())
    }
}
impl Comparator for SizeAndLink {
    fn identity(&self) -> &SemanticId {
        &SIZE_ID
    }
    fn demand(&self, shape: Shape) -> SideDemands {
        leaf_demand(shape)
    }
    fn compare(&self, input: ComparisonInput<'_>) -> Result<Comparison, CheckError> {
        compare_leaves(input, StatChecks::SIZE)
    }
}
impl Comparator for StatComparator {
    fn identity(&self) -> &SemanticId {
        &self.identity
    }
    fn demand(&self, shape: Shape) -> SideDemands {
        leaf_demand(shape)
    }
    fn compare(&self, input: ComparisonInput<'_>) -> Result<Comparison, CheckError> {
        compare_leaves(input, self.checks)
    }
}
fn compare_leaves(input: ComparisonInput<'_>, checks: StatChecks) -> Result<Comparison, CheckError> {
    let source = input.facts.source.ok_or("missing compatible source facts")?;
    let target = input.facts.target.ok_or("missing compatible target facts")?;
    let equal = if source.kind == Kind::File {
        let s = source.attributes.known().ok_or("source attributes were not requested")?;
        let t = target.attributes.known().ok_or("target attributes were not requested")?;
        // Preserve the legacy short circuit: timestamp extraction is not requested
        // by the stat-key predicate when size already establishes a difference.
        (!checks.contains(StatChecks::SIZE) || s.len() == t.len())
            && (!checks.contains(StatChecks::MTIME)
                || s.modified()
                    .map_err(|e| crate::Error::io(input.source_path.to_owned(), "read modification time", e))?
                    == t.modified()
                        .map_err(|e| crate::Error::io(input.target_path.to_owned(), "read modification time", e))?)
            && (!checks.contains(StatChecks::MODE) || same_mode(s, t))
    } else {
        let (Available::Known(s), Available::Known(t)) = (source.literal_link, target.literal_link) else {
            return Err("literal link was not requested".into());
        };
        s.as_os_str() == t.as_os_str()
    };
    Ok(if equal { Comparison::Equal } else { Comparison::Different })
}

fn same_mode(s: &std::fs::Metadata, t: &std::fs::Metadata) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        s.permissions().mode() & 0o7777 == t.permissions().mode() & 0o7777
    }
    #[cfg(not(unix))]
    {
        s.permissions().readonly() == t.permissions().readonly()
    }
}

pub(crate) fn structural(shape: Shape) -> Option<Comparison> {
    if !shape.supported() {
        return Some(Comparison::Unsupported {
            code: UNSUPPORTED_PHYSICAL_KIND,
        });
    }
    match (shape.source, shape.target) {
        (SourceObservation::Present(Kind::Directory), Some(Kind::Directory)) => Some(Comparison::Equal),
        (SourceObservation::Present(s), Some(t)) if s == t => None,
        _ => Some(Comparison::Different),
    }
}
