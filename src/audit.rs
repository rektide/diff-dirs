//! Read-only, one-tree-at-a-time coverage, including opaque diff interiors.

use std::fmt;
use std::path::Path;

use crate::{Check, CheckedScope, Error, Facts, NoScope, Purpose, Scope, Side, Visitor};

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
    audit_with_visitor(root, side, &CheckedScope::new(check.borrowed(), scope.borrowed()))
}

/// Read-only recursive coverage using the exact early/late action contexts.
/// Actions are never applied; finalized descent determines coverage, including
/// explicit PreserveTree. Audit deliberately supplies literal link text on both
/// sides. Counts describe successfully finalized batches, not predicted work or
/// a certificate authorizing future writes. A refused batch contributes no child
/// observations; earlier completed directory batches remain in the prefix.
pub fn audit_with_visitor<V: Visitor>(root: impl AsRef<Path>, side: Side, visitor: &V) -> Result<Audit, AuditFailure> {
    fn count(facts: &Facts, inspected: &mut Audit) {
        if facts.metadata().is_dir() {
            inspected.directories += 1;
        } else if facts.metadata().is_file() {
            inspected.files += 1;
        } else if facts.metadata().is_symlink() {
            inspected.symlinks += 1;
        }
    }
    fn visit<V: Visitor>(root: &Path, relative: &Path, side: Side, visitor: &V, inspected: &mut Audit) -> Result<(), Error> {
        let path = root.join(relative);
        let plan = crate::inspect_directory(
            root,
            root,
            relative,
            (side == Side::Original).then_some(path.as_path()),
            (side == Side::Target).then_some(path.as_path()),
            Purpose::Audit,
            visitor,
        )?;
        inspected.excluded += plan.excluded();
        for entry in plan.operations().entries() {
            count(
                if side == Side::Original {
                    entry.source().unwrap()
                } else {
                    entry.target().unwrap()
                },
                inspected,
            );
            if entry.descends() {
                visit(root, entry.relative_path(), side, visitor, inspected)?;
            }
        }
        Ok(())
    }

    let root = root.as_ref();
    let mut inspected = Audit::default();
    let result = (|| {
        let plan = crate::inspect_single_root(root, side, Purpose::Audit, visitor)?;
        let entry = &plan.operations().entries()[0];
        count(
            if side == Side::Original {
                entry.source().unwrap()
            } else {
                entry.target().unwrap()
            },
            &mut inspected,
        );
        inspected.excluded += plan.excluded();
        if entry.descends() {
            visit(root, Path::new(""), side, visitor, &mut inspected)?;
        }
        Ok(())
    })();
    match result {
        Ok(()) => Ok(inspected),
        Err(error) => Err(AuditFailure { error, inspected }),
    }
}
