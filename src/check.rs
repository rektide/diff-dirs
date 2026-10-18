//! Borrowed observations and statically dispatched accept/refuse checks.

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
    Compare,
    Copy,
    Remove,
    Audit,
}

/// A short-lived observation, never a retained plan or permission certificate.
/// `relative` is empty for a root. `metadata` never follows a symlink.
#[derive(Clone, Copy, Debug)]
pub struct Entry<'a> {
    pub root: &'a Path,
    pub relative: &'a Path,
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
    /// Evaluate the policy using only this borrowed observation. Also used by
    /// the optional sync crate, so recursive operations share the same contract.
    #[inline]
    pub fn check<C: Check + ?Sized>(self, check: &C) -> Result<(), Error> {
        if C::ENABLED {
            check.check(self).map_err(|source| Error::Refused {
                path: self.root.join(self.relative),
                source,
            })?;
        }
        Ok(())
    }
}
