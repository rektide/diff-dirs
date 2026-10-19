use std::ffi::OsString;
use std::path::PathBuf;

/// Traversal execution. Sequential remains default when `parallel` is enabled.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Execution {
    #[default]
    Sequential,
    /// Bounded directory frontier on the current Rayon pool (global fallback
    /// outside an install). Prefer `walk_in_pool` for explicit borrowed sharing.
    /// Default J/K limits are selected at the first `next()` execution site.
    #[cfg(feature = "parallel")]
    Parallel,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WalkOptions {
    pub execution: Execution,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AddedRemoved<T> {
    pub added: T,
    pub removed: T,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ChangedAddedRemoved<T> {
    pub changed: T,
    pub added: T,
    pub removed: T,
}

/// Immediate differences in one both-sided directory. Names are lossless native
/// basenames, individually bytewise sorted within each category.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DirDiff {
    /// Relative directory; the empty path denotes the root pair.
    pub dir: PathBuf,
    /// Regular files and symlinks share the leaf categories.
    pub files: ChangedAddedRemoved<Vec<OsString>>,
    pub dirs: AddedRemoved<Vec<OsString>>,
}

impl DirDiff {
    pub fn is_empty(&self) -> bool {
        self.files.changed.is_empty()
            && self.files.added.is_empty()
            && self.files.removed.is_empty()
            && self.dirs.added.is_empty()
            && self.dirs.removed.is_empty()
    }
}

/// Counts of immediate classified entries, not recursive subtree sizes.
/// One-sided subtrees contribute one directory, never their descendants.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EntryCounts {
    pub files: ChangedAddedRemoved<u64>,
    pub dirs: AddedRemoved<u64>,
}

impl EntryCounts {
    pub(crate) fn add(&mut self, diff: &DirDiff) {
        self.files.changed += diff.files.changed.len() as u64;
        self.files.added += diff.files.added.len() as u64;
        self.files.removed += diff.files.removed.len() as u64;
        self.dirs.added += diff.dirs.added.len() as u64;
        self.dirs.removed += diff.dirs.removed.len() as u64;
    }
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

    pub fn has_differences(&self) -> bool {
        self.counts != EntryCounts::default()
    }

    /// A partial, failed, or pruned scan is never certified clean.
    pub fn is_clean(&self) -> bool {
        self.is_complete() && !self.has_differences()
    }
}
