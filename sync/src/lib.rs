//! Explicit filesystem synchronization; see the crate README for the stable-tree
//! safety model, metadata contract, partial failures, and caller-pool execution.
#![doc = include_str!("../README.md")]

mod apply;
mod model;
mod paths;
mod platform;

pub use model::{CopyPolicy, Failure, Issue, Progress, Report};

use std::path::{Path, PathBuf};

use diff_dirs::{Check, DirDiff, NoCheck};

/// Validated, non-overlapping roots. Construction performs no writes.
/// The source must be a real directory; the target may be absent.
#[derive(Debug)]
pub struct Applier<C = NoCheck> {
    original: PathBuf,
    target: PathBuf,
    policy: CopyPolicy,
    check: C,
    replace_root: bool,
}

impl Applier {
    pub fn new(original: impl AsRef<Path>, target: impl AsRef<Path>, policy: CopyPolicy) -> Result<Self, Failure> {
        Self::new_checked(original, target, policy, NoCheck)
    }
}

impl<C: Check> Applier<C> {
    /// Bind roots without traversing interiors or probing clone capability.
    pub fn new_checked(original: impl AsRef<Path>, target: impl AsRef<Path>, policy: CopyPolicy, check: C) -> Result<Self, Failure> {
        Self::bind(original.as_ref(), target.as_ref(), policy, check, false)
    }

    /// Explicitly authorize replacing an existing target file/link root with a
    /// staged source directory. Ancestors must be real directories; the selected
    /// target symlink itself is NEVER followed. No replacement occurs here.
    pub fn new_replacing_checked(original: impl AsRef<Path>, target: impl AsRef<Path>, policy: CopyPolicy, check: C) -> Result<Self, Failure> {
        Self::bind(original.as_ref(), target.as_ref(), policy, check, true)
    }

    fn bind(original: &Path, target: &Path, policy: CopyPolicy, check: C, replace_root: bool) -> Result<Self, Failure> {
        let original = paths::root(original, false)?;
        let target = paths::root_with_leaf(target, true, replace_root)?;
        if original.starts_with(&target) || target.starts_with(&original) {
            return Err(Failure::invalid("bind roots", &target, "source and target roots overlap"));
        }
        Ok(Self {
            original,
            target,
            policy,
            check,
            replace_root,
        })
    }

    pub fn original(&self) -> &Path {
        &self.original
    }

    pub fn target(&self) -> &Path {
        &self.target
    }

    pub fn check(&self) -> &C {
        &self.check
    }

    /// Apply only this record. Both roots must now exist. Validates the entire
    /// record and its immediate entry types before any mutation. Does not prune,
    /// advance, or retain the caller's iterator.
    pub fn apply(&self, diff: &DirDiff) -> Result<Progress, Failure> {
        self.apply_with(diff, false)
    }

    /// Whole-root initialization, only for an absent target with an existing
    /// parent. On failure a partial target may remain; retry with `mirror`.
    pub fn initialize(&self) -> Result<Progress, Failure> {
        self.initialize_with(false)
    }

    /// Explicit whole-root leaf-to-directory transition. Requires a replacing
    /// binding. A refused descendant or failed preparation preserves the leaf.
    pub fn replace_root(&self) -> Result<Progress, Failure> {
        self.replace_root_with(false)
    }

    /// Read-only full coverage of both existing sides, source first, including
    /// one-sided interiors. Does not compare, apply, or build an operation map.
    pub fn audit(&self) -> Result<AuditReport, AuditFailure> {
        let mut report = AuditReport::default();
        match diff_dirs::audit(&self.original, diff_dirs::Side::Original, &self.check) {
            Ok(inspected) => report.original = inspected,
            Err(failure) => {
                report.original = failure.inspected;
                return Err(AuditFailure {
                    error: failure.error,
                    inspected: report,
                });
            }
        }
        match paths::exists(&self.target) {
            Ok(false) => return Ok(report),
            Err(failure) => {
                return Err(AuditFailure {
                    error: diff_dirs::Error::Io {
                        path: self.target.clone(),
                        operation: "inspect audit target",
                        source: std::io::Error::other(failure),
                    },
                    inspected: report,
                });
            }
            Ok(true) => {}
        }
        match diff_dirs::audit(&self.target, diff_dirs::Side::Target, &self.check) {
            Ok(inspected) => {
                report.target = inspected;
                Ok(report)
            }
            Err(failure) => {
                report.target = failure.inspected;
                Err(AuditFailure {
                    error: failure.error,
                    inspected: report,
                })
            }
        }
    }

    #[cfg(feature = "parallel")]
    pub fn apply_in_pool(&self, diff: &DirDiff, pool: &rayon::ThreadPool) -> Result<Progress, Failure> {
        pool.install(|| self.apply_with(diff, true))
    }

    #[cfg(feature = "parallel")]
    pub fn initialize_in_pool(&self, pool: &rayon::ThreadPool) -> Result<Progress, Failure> {
        pool.install(|| self.initialize_with(true))
    }

    #[cfg(feature = "parallel")]
    pub fn replace_root_in_pool(&self, pool: &rayon::ThreadPool) -> Result<Progress, Failure> {
        pool.install(|| self.replace_root_with(true))
    }
}

/// Full read-only checked coverage; counters include each present root.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AuditReport {
    pub original: diff_dirs::Audit,
    pub target: diff_dirs::Audit,
}

#[derive(Debug)]
pub struct AuditFailure {
    pub error: diff_dirs::Error,
    pub inspected: AuditReport,
}

impl std::fmt::Display for AuditFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.error.fmt(f)
    }
}

impl std::error::Error for AuditFailure {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.error)
    }
}

/// Mirror source to target, including exact target-only removals. Application
/// finishes before each next comparison. A successful report's comparison
/// describes the *observed delta*, not a post-sync verification scan.
pub fn mirror(original: impl AsRef<Path>, target: impl AsRef<Path>, policy: CopyPolicy) -> Result<Report, Failure> {
    Applier::new(original, target, policy)?.mirror()
}

pub fn mirror_checked<C: Check>(original: impl AsRef<Path>, target: impl AsRef<Path>, policy: CopyPolicy, check: C) -> Result<Report, Failure> {
    Applier::new_checked(original, target, policy, check)?.mirror()
}

/// Optional borrowed-pool comparison and application. Creates/configures no
/// pool. J/K limit comparison lookahead, not application workers or bytes.
#[cfg(feature = "parallel")]
pub fn mirror_in_pool(
    original: impl AsRef<Path>,
    target: impl AsRef<Path>,
    policy: CopyPolicy,
    pool: &rayon::ThreadPool,
    limits: diff_dirs::ParallelLimits,
) -> Result<Report, Failure> {
    Applier::new(original, target, policy)?.mirror_in_pool(pool, limits)
}

impl<C: Check> Applier<C> {
    /// Stream compare/apply, or initialize/explicitly replace a selected root.
    /// Does not run `audit` first. Every visited entry uses this binding's check.
    pub fn mirror(&self) -> Result<Report, Failure> {
        if let Some(report) = self.whole_root(false)? {
            return Ok(report);
        }
        // Preserve NoCheck specialization across the comparison/application seam.
        let mut walk = diff_dirs::walk_checked(self.original(), self.target(), self.check.borrowed()).map_err(Failure::comparison)?;
        let mut completed = Progress::default();
        while let Some(item) = walk.next() {
            match item.map_err(Failure::comparison).and_then(|diff| self.apply(&diff)) {
                Ok(progress) => completed += progress,
                Err(mut failure) => {
                    failure.completed += completed;
                    failure.observed = Some(Box::new(walk.into_summary()));
                    return Err(failure);
                }
            }
        }
        Ok(Report {
            completed,
            observed: Some(walk.into_summary()),
        })
    }

    #[cfg(feature = "parallel")]
    pub fn mirror_in_pool(&self, pool: &rayon::ThreadPool, limits: diff_dirs::ParallelLimits) -> Result<Report, Failure> {
        if let Some(report) = pool.install(|| self.whole_root(true))? {
            return Ok(report);
        }
        let mut walk = diff_dirs::walk_in_pool_checked(self.original(), self.target(), pool, limits, self.check.borrowed()).map_err(Failure::comparison)?;
        let mut completed = Progress::default();
        while let Some(item) = walk.next() {
            match item.map_err(Failure::comparison).and_then(|diff| self.apply_in_pool(&diff, pool)) {
                Ok(progress) => completed += progress,
                Err(mut failure) => {
                    failure.completed += completed;
                    failure.observed = Some(Box::new(walk.into_summary()));
                    return Err(failure);
                }
            }
        }
        Ok(Report {
            completed,
            observed: Some(walk.into_summary()),
        })
    }

    fn whole_root(&self, parallel: bool) -> Result<Option<Report>, Failure> {
        let completed = match std::fs::symlink_metadata(&self.target) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => self.initialize_with(parallel)?,
            Err(error) => return Err(Failure::io("inspect target root", &self.target, error)),
            Ok(metadata) if metadata.is_dir() => return Ok(None),
            Ok(_) => self.replace_root_with(parallel)?,
        };
        Ok(Some(Report { completed, observed: None }))
    }
}
