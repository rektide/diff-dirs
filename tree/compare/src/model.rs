use std::path::PathBuf;

use crate::{DirectoryCoverage, EntryCounts, Execution, SemanticId};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WalkOptions {
    pub execution: Execution,
}

/// Iterator termination, independent of deliberate pruning.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ScanState {
    /// `None` has not been reached; the examined prefix is not a full scan.
    #[default]
    Incomplete,
    /// The iterator reached `None` normally. Check `pruned` for skipped subtrees.
    Complete,
    /// One terminal `Err` was yielded; subsequent calls yield `None`.
    Failed,
}

/// Aggregate of the committed DFS prefix, excluding physical speculative work.
/// Consuming the walk never does more IO.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Summary {
    /// A scoped scan never certifies full-root coverage, even if no excluded
    /// name happened to be encountered in its committed prefix.
    pub scoped: bool,
    /// Union-name boundaries excluded before observation (not subtree sizes).
    pub excluded: u64,
    /// Immediate finalized namespace actions in the committed comparison prefix,
    /// not recursive application work or completed filesystem operations.
    pub planned: u64,
    /// Committed slots whose finalized action differs from normal mirror work.
    pub deviated: u64,
    /// Bound predicate; None only before an unbound/default summary is assessed.
    pub comparator: Option<SemanticId>,
    pub equal: u64,
    pub different: u64,
    pub not_assessed: u64,
    pub unsupported: u64,
    pub unresolved: u64,
    pub stopped_paired: u64,
    pub counts: EntryCounts,
    /// Directly divergent directories with no directly divergent ancestor,
    /// in depth-first pre-order. Empty path denotes the root.
    pub topmost_divergent: Vec<PathBuf>,
    /// Both-sided directories whose examined subtree contains a difference,
    /// including the root, in depth-first post-order. Not an entry count.
    pub changed_dirs: Vec<PathBuf>,
    pub examined_directories: u64,
    pub state: ScanState,
    /// Records for which the caller explicitly suppressed further descent.
    pub pruned: Vec<PathBuf>,
}

impl Summary {
    pub(crate) fn add_coverage(&mut self, coverage: DirectoryCoverage, comparator: &SemanticId) {
        debug_assert!(self.comparator.as_ref().is_none_or(|bound| bound == comparator));
        self.comparator = Some(comparator.clone());
        self.scoped |= coverage.scoped;
        self.excluded += coverage.excluded;
        self.planned += coverage.planned;
        self.deviated += coverage.deviated;
        self.equal += coverage.equal;
        self.different += coverage.different;
        self.not_assessed += coverage.not_assessed;
        self.unsupported += coverage.unsupported;
        self.unresolved += coverage.unresolved;
        self.stopped_paired += coverage.stopped_paired;
    }

    /// Traversal exhaustion, independent of comparison coverage or work choice.
    pub fn is_exhausted(&self) -> bool {
        self.state == ScanState::Complete
    }
    /// Complete assessment coverage under the bound comparator. Selected work
    /// may deviate from baseline (for example equal renewal) without a gap.
    pub fn is_complete(&self) -> bool {
        self.is_exhausted()
            && self.pruned.is_empty()
            && !self.scoped
            && self.excluded == 0
            && self.stopped_paired == 0
            && self.not_assessed == 0
            && self.unsupported == 0
            && self.unresolved == 0
    }

    /// Known assessed differences. Missing/unsupported/unresolved assessment has
    /// no normal delta, so false alone is not a clean-root certificate.
    pub fn has_differences(&self) -> bool {
        self.different != 0 || self.counts != EntryCounts::default()
    }

    /// A partial, failed, or pruned scan is never certified clean.
    pub fn is_clean(&self) -> bool {
        self.is_complete() && !self.has_differences()
    }
}
