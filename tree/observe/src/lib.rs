#![cfg_attr(all(target_os = "linux", feature = "native-dirent"), feature(dirfd))]

//! Physical filesystem observations without comparison policy or traversal.
//!
//! Owned listing entries expose cached hints and explicit, consuming no-follow
//! metadata acquisition. Backend handles and directory descriptors stay private.
//! These observations are not snapshots; callers own quiescence and confinement.

use std::fs::{self, Metadata};
use std::path::{Path, PathBuf};

mod error;
mod facts;
mod listing;
mod names;
mod resources;

pub use error::Error;
pub use facts::{EarlyFacts, FactOrigin, FileKind};
pub use listing::{Entry, read_directory, read_directory_with};
pub use names::{NameId, NameTable};
pub use resources::{ObservationBackend, ObservationConfig, ObservationResources, ResourceError};

/// Obtain metadata for the named entry itself, never following its final symlink.
/// Ancestor confinement remains the caller's responsibility.
pub fn symlink_metadata(path: &Path) -> Result<Metadata, Error> {
    fs::symlink_metadata(path).map_err(|source| Error::io(path.to_path_buf(), "inspect entry", source))
}

/// Read a symlink's literal native target spelling, without following it.
/// The caller decides whether its operation requires link text at all.
pub fn read_link(path: &Path) -> Result<PathBuf, Error> {
    fs::read_link(path).map_err(|source| Error::io(path.to_path_buf(), "read symlink", source))
}

#[cfg(test)]
mod tests;

#[cfg(test)]
#[path = "../../../tests/support/fixture.rs"]
mod test_support;
