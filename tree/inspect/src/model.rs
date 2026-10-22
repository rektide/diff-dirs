use std::ffi::OsString;
use std::path::PathBuf;

/// Inspection execution, also selected by traversal options. Sequential remains
/// default when `parallel` is enabled.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Execution {
    #[default]
    Sequential,
    /// Rayon execution on the current pool (global fallback outside an install).
    /// Immediate inspection distributes sufficiently large admitted name batches.
    /// Comparison's parallel feature instead uses a bounded directory frontier;
    /// prefer its `walk_in_pool` API for explicit borrowed sharing. Default J/K
    /// limits are selected at the first `next()` execution site.
    #[cfg(feature = "parallel")]
    Parallel,
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
    /// Accumulate one immediate record, not its opaque subtrees or descendants.
    pub fn add(&mut self, diff: &DirDiff) {
        self.files.changed += diff.files.changed.len() as u64;
        self.files.added += diff.files.added.len() as u64;
        self.files.removed += diff.files.removed.len() as u64;
        self.dirs.added += diff.dirs.added.len() as u64;
        self.dirs.removed += diff.dirs.removed.len() as u64;
    }
}
