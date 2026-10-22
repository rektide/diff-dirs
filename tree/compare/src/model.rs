use std::path::PathBuf;

use crate::{EntryCounts, Execution};

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
    pub retained: u64,
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
    /// True only after normal exhaustion without any pruning.
    pub fn is_complete(&self) -> bool {
        self.state == ScanState::Complete && self.pruned.is_empty() && !self.scoped && self.excluded == 0 && self.retained == 0
    }

    /// Known supported comparison differences. Late-preserved special kinds
    /// have no normal delta; they contribute retained coverage instead, so false
    /// here alone must not be treated as a clean-root certificate.
    pub fn has_differences(&self) -> bool {
        self.counts != EntryCounts::default()
    }

    /// A partial, failed, or pruned scan is never certified clean.
    pub fn is_clean(&self) -> bool {
        self.is_complete() && !self.has_differences()
    }
}
