use std::fmt;
use std::io;
use std::path::{Path, PathBuf};

/// A failed physical observation, without policy refusals or comparison errors.
#[derive(Debug)]
pub struct Error {
    path: PathBuf,
    operation: &'static str,
    source: io::Error,
}

impl Error {
    /// Physical path whose acquisition failed.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Physical operation attempted at that path.
    pub fn operation(&self) -> &'static str {
        self.operation
    }

    /// Underlying I/O cause, including its OS error code when available.
    pub fn io_error(&self) -> &io::Error {
        &self.source
    }

    /// Transfer the original cause and coordinates into a consumer's boundary.
    pub fn into_parts(self) -> (PathBuf, &'static str, io::Error) {
        (self.path, self.operation, self.source)
    }

    pub(crate) fn io(path: PathBuf, operation: &'static str, source: io::Error) -> Self {
        Self { path, operation, source }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {:?}: {}", self.operation, self.path, self.source)
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.source)
    }
}
