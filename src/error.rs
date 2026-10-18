use std::fmt;
use std::io;
use std::path::{Path, PathBuf};

/// A terminal comparison failure, always identifying the offending path.
#[derive(Debug)]
pub enum Error {
    Io {
        path: PathBuf,
        operation: &'static str,
        source: io::Error,
    },
    /// Roots must be real directories, not files or symlinks to directories.
    NotDirectory { path: PathBuf },
    /// Only directories, regular files, and symlinks are supported.
    UnsupportedEntry { path: PathBuf },
}

impl Error {
    pub fn path(&self) -> &Path {
        match self {
            Self::Io { path, .. } | Self::NotDirectory { path } | Self::UnsupportedEntry { path } => path,
        }
    }

    pub(crate) fn io(path: PathBuf, operation: &'static str, source: io::Error) -> Self {
        Self::Io { path, operation, source }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, operation, source } => write!(f, "{operation} {path:?}: {source}"),
            Self::NotDirectory { path } => write!(f, "root is not a real directory: {path:?}"),
            Self::UnsupportedEntry { path } => write!(f, "unsupported special-file entry: {path:?}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            _ => None,
        }
    }
}
