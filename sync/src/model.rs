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

#[derive(Debug)]
pub struct Report {
    pub completed: Progress,
    /// None for absent-root initialization or explicit leaf-root replacement
    /// (there was no both-sided walk).
    pub observed: Option<diff_dirs::Summary>,
}

/// An operation/path and its original underlying cause. Paths are absolute
/// bound paths (a comparison-construction issue may have an empty path).
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
    pub primary: Box<Issue>,
    pub secondary: Vec<Issue>,
    pub completed: Progress,
    pub observed: Option<Box<diff_dirs::Summary>>,
}

impl Failure {
    pub(crate) fn io(operation: &'static str, path: &Path, cause: io::Error) -> Self {
        Self {
            primary: Box::new(Issue {
                operation,
                path: path.to_path_buf(),
                cause: Box::new(cause),
            }),
            secondary: Vec::new(),
            completed: Progress::default(),
            observed: None,
        }
    }

    pub(crate) fn invalid(operation: &'static str, path: &Path, reason: &'static str) -> Self {
        Self::io(operation, path, io::Error::new(io::ErrorKind::InvalidInput, reason))
    }

    pub(crate) fn comparison(cause: diff_dirs::Error) -> Self {
        Self::from_check_error("compare", cause)
    }

    pub(crate) fn checked(cause: diff_dirs::Error) -> Self {
        Self::from_check_error("check", cause)
    }

    fn from_check_error(operation: &'static str, cause: diff_dirs::Error) -> Self {
        let path = cause.path().to_path_buf();
        Self {
            primary: Box::new(Issue {
                operation,
                path,
                cause: Box::new(cause),
            }),
            secondary: Vec::new(),
            completed: Progress::default(),
            observed: None,
        }
    }

    pub(crate) fn append(&mut self, other: Self) {
        self.completed += other.completed;
        self.secondary.push(*other.primary);
        self.secondary.extend(other.secondary);
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
