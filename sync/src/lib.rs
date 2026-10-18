//! Explicit filesystem synchronization; see the crate README for the stable-tree
//! safety model, metadata contract, partial failures, and caller-pool execution.
#![doc = include_str!("../README.md")]

mod apply;
mod model;
mod paths;
mod platform;

pub use model::{CopyPolicy, Failure, Issue, Progress, Report};

use std::path::{Path, PathBuf};

use diff_dirs::DirDiff;

/// Validated, non-overlapping roots. Construction performs no writes.
/// The source must be a real directory; the target may be absent.
#[derive(Debug)]
pub struct Applier {
    original: PathBuf,
    target: PathBuf,
    policy: CopyPolicy,
}

impl Applier {
    pub fn new(original: impl AsRef<Path>, target: impl AsRef<Path>, policy: CopyPolicy) -> Result<Self, Failure> {
        let original = paths::root(original.as_ref(), false)?;
        let target = paths::root(target.as_ref(), true)?;
        if original.starts_with(&target) || target.starts_with(&original) {
            return Err(Failure::invalid("bind roots", &target, "source and target roots overlap"));
        }
        Ok(Self { original, target, policy })
    }

    pub fn original(&self) -> &Path {
        &self.original
    }

    pub fn target(&self) -> &Path {
        &self.target
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

    #[cfg(feature = "parallel")]
    pub fn apply_in_pool(&self, diff: &DirDiff, pool: &rayon::ThreadPool) -> Result<Progress, Failure> {
        pool.install(|| self.apply_with(diff, true))
    }

    #[cfg(feature = "parallel")]
    pub fn initialize_in_pool(&self, pool: &rayon::ThreadPool) -> Result<Progress, Failure> {
        pool.install(|| self.initialize_with(true))
    }
}

/// Mirror source to target, including exact target-only removals. Application
/// finishes before each next comparison. A successful report's comparison
/// describes the *observed delta*, not a post-sync verification scan.
pub fn mirror(original: impl AsRef<Path>, target: impl AsRef<Path>, policy: CopyPolicy) -> Result<Report, Failure> {
    let applier = Applier::new(original, target, policy)?;
    if !paths::exists(applier.target())? {
        return applier.initialize().map(|completed| Report { completed, observed: None });
    }
    let mut walk = diff_dirs::walk(applier.original(), applier.target()).map_err(Failure::comparison)?;
    let mut completed = Progress::default();
    while let Some(item) = walk.next() {
        let result = item.map_err(Failure::comparison).and_then(|diff| applier.apply(&diff));
        match result {
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
    let applier = Applier::new(original, target, policy)?;
    if !paths::exists(applier.target())? {
        return applier.initialize_in_pool(pool).map(|completed| Report { completed, observed: None });
    }
    let mut walk = diff_dirs::walk_in_pool(applier.original(), applier.target(), pool, limits).map_err(Failure::comparison)?;
    let mut completed = Progress::default();
    while let Some(item) = walk.next() {
        let result = item.map_err(Failure::comparison).and_then(|diff| applier.apply_in_pool(&diff, pool));
        match result {
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
