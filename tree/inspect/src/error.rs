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
    /// Caller policy refused an observed entry, before descent or mutation.
    Refused { path: PathBuf, source: crate::CheckError },
}

impl Error {
    pub fn path(&self) -> &Path {
        match self {
            Self::Io { path, .. } | Self::NotDirectory { path } | Self::UnsupportedEntry { path } | Self::Refused { path, .. } => path,
        }
    }

    pub(crate) fn io(path: PathBuf, operation: &'static str, source: io::Error) -> Self {
        Self::Io { path, operation, source }
    }

    pub(crate) fn root_observation(error: diff_dirs_tree_observe::Error) -> Self {
        let (path, _, source) = error.into_parts();
        Self::io(path, "inspect root", source)
    }
}

impl From<diff_dirs_tree_observe::Error> for Error {
    fn from(error: diff_dirs_tree_observe::Error) -> Self {
        let (path, operation, source) = error.into_parts();
        Self::io(path, operation, source)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, operation, source } => write!(f, "{operation} {path:?}: {source}"),
            Self::NotDirectory { path } => write!(f, "root is not a real directory: {path:?}"),
            Self::UnsupportedEntry { path } => write!(f, "unsupported special-file entry: {path:?}"),
            Self::Refused { path, source } => write!(f, "check refused {path:?}: {source}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::Refused { source, .. } => Some(&**source),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error as _;

    #[test]
    fn observation_mapping_preserves_physical_context_and_io_cause() {
        let f = crate::test_support::Fixture::new();
        let path = f.original.join("missing");
        let observed = diff_dirs_tree_observe::read_link(&path).unwrap_err();
        let raw = observed.io_error().raw_os_error();
        let mapped = Error::from(observed);
        assert_eq!(mapped.path(), path);
        let Error::Io { operation, source, .. } = &mapped else {
            panic!("physical observation became a policy error")
        };
        assert_eq!(*operation, "read symlink");
        assert_eq!(source.kind(), io::ErrorKind::NotFound);
        assert_eq!(source.raw_os_error(), raw);
        assert!(std::ptr::eq(mapped.source().unwrap().downcast_ref::<io::Error>().unwrap(), source));
    }

    #[test]
    fn root_binding_adds_root_context_without_replacing_the_io_cause() {
        let f = crate::test_support::Fixture::new();
        let path = f.original.join("missing");
        let observed = diff_dirs_tree_observe::symlink_metadata(&path).unwrap_err();
        let raw = observed.io_error().raw_os_error();
        let mapped = crate::PairedRoots::new(&path, &f.target).unwrap_err();
        assert_eq!(mapped.path(), path);
        let Error::Io { operation, source, .. } = &mapped else {
            panic!("physical root observation became a policy error")
        };
        assert_eq!(*operation, "inspect root");
        assert_eq!(source.kind(), io::ErrorKind::NotFound);
        assert_eq!(source.raw_os_error(), raw);
        assert!(mapped.source().unwrap().is::<io::Error>());
    }
}
