//! Read-only, one-tree-at-a-time coverage, including opaque diff interiors.

use std::fmt;
use std::fs;
use std::path::Path;

use crate::{Check, Entry, Error, NoScope, Purpose, Scope, Side};

/// Successfully checked observations, including the root. Not diff counts.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Audit {
    pub excluded: u64,
    pub directories: u64,
    pub files: u64,
    pub symlinks: u64,
}

#[derive(Debug)]
pub struct AuditFailure {
    pub error: Error,
    /// Successfully checked DFS prefix. A refused/unsupported entry is excluded;
    /// an accepted directory is counted even if reading its contents later fails.
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
    audit_scoped(root, side, check, &NoScope)
}

/// Read-only [`audit`] with the same pre-observation scope as paired comparison.
/// Includes participating one-sided interiors; excluded boundaries increment
/// [`Audit::excluded`] without inspecting hidden descendants. Root observation
/// still runs. This is entry validation, not a deletion plan or a certificate
/// authorizing later writes after writers resume. Callers own ancestor stability.
pub fn audit_scoped<C: Check, S: Scope>(root: impl AsRef<Path>, side: Side, check: &C, scope: &S) -> Result<Audit, AuditFailure> {
    fn visit<C: Check, S: Scope>(root: &Path, path: &Path, side: Side, check: &C, scope: &S, inspected: &mut Audit) -> Result<(), Error> {
        if path != root && S::ENABLED && scope.at(path.strip_prefix(root).expect("audit descendant")) == crate::Visit::PreserveTree {
            inspected.excluded += 1;
            return Ok(());
        }
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
            if path == root && !crate::scope_includes(scope, Path::new("")) {
                inspected.excluded += 1;
                return Ok(());
            }
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
                visit(root, &path.join(name), side, check, scope, inspected)?;
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
    match visit(root, root, side, check, scope, &mut inspected) {
        Ok(()) => Ok(inspected),
        Err(error) => Err(AuditFailure { error, inspected }),
    }
}
