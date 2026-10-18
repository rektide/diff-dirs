//! Read-only, one-tree-at-a-time coverage, including opaque diff interiors.

use std::fmt;
use std::fs;
use std::path::Path;

use crate::{Check, Entry, Error, Purpose, Side};

/// Successfully checked observations, including the root. Not diff counts.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Audit {
    pub directories: u64,
    pub files: u64,
    pub symlinks: u64,
}

#[derive(Debug)]
pub struct AuditFailure {
    pub error: Error,
    /// Successfully checked DFS prefix, excluding the offending entry.
    pub inspected: Audit,
}

impl fmt::Display for AuditFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.error.fmt(f)
    }
}

impl std::error::Error for AuditFailure {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.error)
    }
}

/// Inspect every entry of one existing tree in native-name DFS order, without
/// writing or constructing a diff/operation plan. A root symlink is observed as
/// a leaf, never followed. The caller owns ancestor confinement and stability.
/// This is an explicit audit, not an implicit phase of checked synchronization.
pub fn audit<C: Check>(root: impl AsRef<Path>, side: Side, check: &C) -> Result<Audit, AuditFailure> {
    fn visit<C: Check>(root: &Path, path: &Path, side: Side, check: &C, inspected: &mut Audit) -> Result<(), Error> {
        let metadata = fs::symlink_metadata(path).map_err(|e| Error::io(path.to_path_buf(), "inspect audit entry", e))?;
        let link = if metadata.is_symlink() {
            Some(fs::read_link(path).map_err(|e| Error::io(path.to_path_buf(), "read symlink", e))?)
        } else {
            None
        };
        Entry {
            root,
            path,
            name: path.file_name().unwrap_or(path.as_os_str()),
            side,
            purpose: Purpose::Audit,
            metadata: &metadata,
            link_target: link.as_deref(),
        }
        .check(check)?;
        if metadata.is_dir() {
            inspected.directories += 1;
            let mut children = fs::read_dir(path)
                .map_err(|e| Error::io(path.to_path_buf(), "read audit directory", e))?
                .map(|entry| {
                    entry
                        .map(|entry| entry.file_name())
                        .map_err(|e| Error::io(path.to_path_buf(), "read directory entry", e))
                })
                .collect::<Result<Vec<_>, _>>()?;
            children.sort_unstable_by(|a, b| a.as_encoded_bytes().cmp(b.as_encoded_bytes()));
            for name in children {
                visit(root, &path.join(name), side, check, inspected)?;
            }
        } else if metadata.is_file() {
            inspected.files += 1;
        } else if metadata.is_symlink() {
            inspected.symlinks += 1;
        } else {
            return Err(Error::UnsupportedEntry { path: path.to_path_buf() });
        }
        Ok(())
    }

    let root = root.as_ref();
    let mut inspected = Audit::default();
    match visit(root, root, side, check, &mut inspected) {
        Ok(()) => Ok(inspected),
        Err(error) => Err(AuditFailure { error, inspected }),
    }
}
