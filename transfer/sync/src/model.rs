use std::error::Error;
use std::fmt;
use std::io;
use std::ops::AddAssign;
use std::path::{Path, PathBuf};

/// Explicit per-regular-file policy. Symlinks are always recreated literally.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CopyPolicy {
    Copy,
    ReflinkRequired,
    ReflinkOrCopy,
}

/// Completed operations on visible target entries, including partial subtrees.
/// Replacements count an installed leaf, not an additional removal of the old
/// leaf. Byte counts are logical bytes, not physical I/O or allocated extents.
/// Failed preparations/temporary artifacts are not completed installations.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Progress {
    pub files_copied: u64,
    pub files_reflinked: u64,
    pub bytes_copied: u64,
    pub bytes_reflinked: u64,
    pub symlinks_created: u64,
    pub directories_created: u64,
    pub files_removed: u64,
    pub symlinks_removed: u64,
    pub directories_removed: u64,
    /// Successful directory chmod calls, including temporary relaxation and
    /// restoration. File metadata is part of its installed-leaf operation.
    pub directory_permissions_set: u64,
}

impl AddAssign for Progress {
    fn add_assign(&mut self, rhs: Self) {
        self.files_copied += rhs.files_copied;
        self.files_reflinked += rhs.files_reflinked;
        self.bytes_copied += rhs.bytes_copied;
        self.bytes_reflinked += rhs.bytes_reflinked;
        self.symlinks_created += rhs.symlinks_created;
        self.directories_created += rhs.directories_created;
        self.files_removed += rhs.files_removed;
        self.symlinks_removed += rhs.symlinks_removed;
        self.directories_removed += rhs.directories_removed;
        self.directory_permissions_set += rhs.directory_permissions_set;
    }
}

/// Three distinct results of a declared operation: pre-application comparison,
/// policy completion, and actual completed filesystem work. Successful scoped
/// reconciliation may retain target extras and never certifies full-root equality.
/// No retained/processed per-file catalog is built. See [`crate::Applier::run`].
#[derive(Debug)]
pub struct Report {
    /// Finalized immediate namespace actions encountered by application,
    /// including recursive batches. Not completed work; collisions are coupled.
    pub planned: u64,
    /// Visible completed operations, not hypothetical actions from a diff or
    /// discarded private preparations. A clean/retained-only run is all zero.
    pub completed: Progress,
    /// Policy completion is not a full-root equality certificate.
    pub policy: Policy,
    /// Pre-application observed delta, not a verification scan. Opaque directory
    /// records count boundaries, not recursively applied children. Scoped scans
    /// cannot claim full clean coverage. None for initialization, root replacement,
    /// explicit clear, joint preview, and per-record application (no both-sided walk).
    pub observed: Option<diff_dirs_tree_compare::Summary>,
}

/// Bounded counters describing encountered policy outcomes, independently of
/// actual operations or raw differences. Counts are not unique-path inventories
/// and must not be added to comparison exclusions as if they were disjoint.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Policy {
    /// The declared operation exhausted its policy successfully. For
    /// `apply_report`, this covers only the supplied record, not the whole tree.
    /// This is false on Failure and is not a full-root cleanliness certificate.
    pub satisfied: bool,
    /// Retained committed comparison slots (including uncomparable specials),
    /// recursive retained entries, and necessary structural containers. Slots in
    /// yielded batches are not counted twice when combining walk/application.
    pub retained: u64,
    /// Excluded boundaries encountered during application, not subtree sizes.
    /// Comparison exclusions are separately in `Report.observed`.
    pub excluded: u64,
}

impl AddAssign for Policy {
    fn add_assign(&mut self, rhs: Self) {
        self.retained += rhs.retained;
        self.excluded += rhs.excluded;
    }
}

/// An operation/path and its original underlying cause. Filesystem operations
/// identify absolute bound paths; syntax/duplicate-list validation identifies
/// relative input coordinates. A comparison-construction issue may have an
/// empty path. Native spellings and original refusal/I/O causes are retained.
#[derive(Debug)]
pub struct Issue {
    pub operation: &'static str,
    pub path: PathBuf,
    pub cause: Box<dyn Error + Send + Sync>,
}

impl fmt::Display for Issue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {:?}: {}", self.operation, self.path, self.cause)
    }
}

impl Error for Issue {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&*self.cause)
    }
}

/// Partial progress, primary error, and joined sibling/cleanup/restoration
/// errors. Not a transaction or a promise that only a serial prefix changed.
#[derive(Debug)]
pub struct Failure {
    /// Finalized encountered namespace proposals, independently of completed work.
    pub planned: u64,
    /// Encountered retention/exclusion counters; satisfaction is always false.
    pub policy: Box<Policy>,
    pub primary: Box<Issue>,
    /// Cold error details are boxed to keep streaming Result values compact.
    pub secondary: Box<Vec<Issue>>,
    /// Actual completed operations, including earlier batches and partial
    /// subtrees. Private failed preparations are not successful installations.
    pub completed: Progress,
    pub observed: Option<Box<diff_dirs_tree_compare::Summary>>,
}

impl Failure {
    pub(crate) fn io(operation: &'static str, path: &Path, cause: io::Error) -> Self {
        Self {
            planned: 0,
            policy: Box::default(),
            primary: Box::new(Issue {
                operation,
                path: path.to_path_buf(),
                cause: Box::new(cause),
            }),
            secondary: Box::default(),
            completed: Progress::default(),
            observed: None,
        }
    }

    pub(crate) fn invalid(operation: &'static str, path: &Path, reason: &'static str) -> Self {
        Self::io(operation, path, io::Error::new(io::ErrorKind::InvalidInput, reason))
    }

    pub(crate) fn comparison(cause: diff_dirs_tree_inspect::Error) -> Self {
        Self::from_check_error("compare", cause)
    }

    pub(crate) fn checked(cause: diff_dirs_tree_inspect::Error) -> Self {
        if let diff_dirs_tree_inspect::Error::Io { path, operation, source } = cause {
            return Self::io(operation, &path, source);
        }
        Self::from_check_error("check", cause)
    }

    fn from_check_error(operation: &'static str, cause: diff_dirs_tree_inspect::Error) -> Self {
        let path = cause.path().to_path_buf();
        Self {
            planned: 0,
            policy: Box::default(),
            primary: Box::new(Issue {
                operation,
                path,
                cause: Box::new(cause),
            }),
            secondary: Box::default(),
            completed: Progress::default(),
            observed: None,
        }
    }

    pub(crate) fn append(&mut self, other: Self) {
        self.planned += other.planned;
        *self.policy += *other.policy;
        self.completed += other.completed;
        self.secondary.push(*other.primary);
        self.secondary.extend(*other.secondary);
    }
}

impl fmt::Display for Failure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.primary.fmt(f)
    }
}

impl Error for Failure {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&*self.primary)
    }
}

pub(crate) fn merge(results: impl IntoIterator<Item = Result<Progress, Failure>>) -> Result<Progress, Failure> {
    let mut completed = Progress::default();
    let mut failure: Option<Failure> = None;
    for result in results {
        match result {
            Ok(progress) => completed += progress,
            Err(error) => match &mut failure {
                Some(failure) => failure.append(error),
                None => failure = Some(error),
            },
        }
    }
    match failure {
        Some(mut failure) => {
            failure.completed += completed;
            Err(failure)
        }
        None => Ok(completed),
    }
}
