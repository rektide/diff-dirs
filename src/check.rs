//! Borrowed observations and statically dispatched accept/refuse checks.

use std::ffi::OsStr;
use std::fs::Metadata;
use std::path::Path;

use crate::Error;

/// Caller-owned refusal cause, allocated only on failure.
pub type CheckError = Box<dyn std::error::Error + Send + Sync + 'static>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    Original,
    Target,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Purpose {
    /// Paired comparison, including entries which turn out equal.
    Compare,
    /// Source inspection for copying, including accepted source ancestors.
    Copy,
    /// Target inspection for removal/replacement or ancestor permission changes;
    /// not a promise that this particular entry will itself be removed.
    Remove,
    /// Explicit read-only coverage, not a mutation plan or clone capability test.
    Audit,
}

/// A short-lived observation, never a retained plan or permission certificate.
/// `relative_path()` is empty for a root. `metadata` never follows a symlink.
#[derive(Clone, Copy, Debug)]
pub struct Entry<'a> {
    pub root: &'a Path,
    /// Already-built observed path, including `root`; absolute for bound sync
    /// roots, but not necessarily absolute for a caller's relative core roots.
    pub path: &'a Path,
    /// Native basename, including for a root. For a filesystem root without a
    /// basename, this is the root's spelling (for example `/`). Name/type checks
    /// need no path decomposition, allocation or root-prefix scan.
    pub name: &'a OsStr,
    pub side: Side,
    pub purpose: Purpose,
    pub metadata: &'a Metadata,
    /// Literal link text when the operation already needs it (comparison,
    /// copying, and audit). Removal does not read links merely for the check.
    pub link_target: Option<&'a Path>,
}

/// Pure, deterministic accept/refuse policy. Parallel inspection may invoke it
/// speculatively, concurrently, more than once, and outside committed DFS order.
/// Do not mutate files, print diagnostics, launch processes, or depend on call
/// order. Returning a refusal prevents descent/mutation of that entry.
/// A thread-safe diagnostic collector may deduplicate encountered observations,
/// but must not affect decisions or mistake speculation for committed progress.
///
/// Closures returning `Result<(), CheckError>` implement this trait. An error's
/// path is supplied by the library; a checker cannot accidentally blame another
/// entry. No additional filesystem observations are requested by this interface.
pub trait Check: Sync {
    /// False only for a genuinely unconditional no-op. Allows all observation
    /// construction and dispatch to disappear from unchecked monomorphizations.
    const ENABLED: bool = true;

    fn check(&self, entry: Entry<'_>) -> Result<(), CheckError>;

    /// Lend the same policy to a walker, preserving no-op specialization.
    fn borrowed(&self) -> CheckRef<'_, Self>
    where
        Self: Sized,
    {
        CheckRef(self)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CheckRef<'a, C: ?Sized>(&'a C);

impl<C: Check + ?Sized> Check for CheckRef<'_, C> {
    const ENABLED: bool = C::ENABLED;

    #[inline]
    fn check(&self, entry: Entry<'_>) -> Result<(), CheckError> {
        self.0.check(entry)
    }
}

impl<F> Check for F
where
    F: Fn(Entry<'_>) -> Result<(), CheckError> + Sync,
{
    #[inline]
    fn check(&self, entry: Entry<'_>) -> Result<(), CheckError> {
        self(entry)
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct NoCheck;

impl Check for NoCheck {
    const ENABLED: bool = false;

    #[inline]
    fn check(&self, _: Entry<'_>) -> Result<(), CheckError> {
        Ok(())
    }
}

impl Entry<'_> {
    /// Borrow the root-relative path only when needed. Name-only checkers should
    /// use `name` instead: extracting a relative path on every entry is avoidable
    /// work, especially with long absolute roots. Library observations are
    /// guaranteed under `root`; manually fabricated invalid entries may panic.
    #[inline]
    pub fn relative_path(&self) -> &Path {
        self.path.strip_prefix(self.root).expect("observed path under root")
    }

    /// Evaluate the policy using only this borrowed observation. Also used by
    /// the optional sync crate, so recursive operations share the same contract.
    #[inline]
    pub fn check<C: Check + ?Sized>(self, check: &C) -> Result<(), Error> {
        if C::ENABLED {
            check.check(self).map_err(|source| Error::Refused {
                path: self.path.to_path_buf(),
                source,
            })?;
        }
        Ok(())
    }
}
