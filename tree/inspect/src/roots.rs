//! Validated paired-root binding and immediate inspection, without traversal.

use std::fs;
use std::path::{Path, PathBuf};

use crate::{Check, CheckedScope, DirectoryPlan, Entry, Error, Execution, Purpose, Scope, Side, Visitor};

/// Two existing real directory roots, with their mandatory no-follow binding
/// facts retained privately. Contents are inspected only on explicit request.
/// This is not a snapshot: callers own quiescence and ancestor confinement.
#[derive(Debug)]
pub struct PairedRoots {
    original: PathBuf,
    target: PathBuf,
    original_metadata: fs::Metadata,
    target_metadata: fs::Metadata,
}

impl PairedRoots {
    /// Validate original then target as real directories, without listing them.
    pub fn new(original: impl AsRef<Path>, target: impl AsRef<Path>) -> Result<Self, Error> {
        Self::new_checked(original, target, &crate::NoCheck)
    }

    /// Bind compatibility checks in their original source-first order, before
    /// inspecting contents. Return the visitor with those root checks already
    /// accounted for, so its later root steering does not repeat them. Use that
    /// visitor with this same binding. Only this binding path can construct that
    /// state; finalized-plan validation is never bypassed. Ownership and static
    /// dispatch of the supplied policies survive.
    pub fn bind_scoped<C: Check, S: Scope>(
        original: impl AsRef<Path>,
        target: impl AsRef<Path>,
        check: C,
        scope: S,
    ) -> Result<(Self, CheckedScope<C, S>), Error> {
        let roots = Self::new_checked(original, target, &check)?;
        Ok((roots, CheckedScope::new(check, scope).roots_checked()))
    }

    fn new_checked<C: Check>(original: impl AsRef<Path>, target: impl AsRef<Path>, check: &C) -> Result<Self, Error> {
        let original = original.as_ref().to_path_buf();
        let target = target.as_ref().to_path_buf();
        let inspect = |path: &Path, side| {
            let metadata = diff_dirs_tree_observe::symlink_metadata(path).map_err(Error::root_observation)?;
            if C::ENABLED {
                Entry {
                    root: path,
                    path,
                    name: path.file_name().unwrap_or(path.as_os_str()),
                    side,
                    purpose: Purpose::Compare,
                    metadata: &metadata,
                    link_target: None,
                }
                .check(check)?;
            }
            if !metadata.is_dir() {
                return Err(Error::NotDirectory { path: path.to_path_buf() });
            }
            Ok(metadata)
        };
        let original_metadata = inspect(&original, Side::Original)?;
        let target_metadata = inspect(&target, Side::Target)?;
        Ok(Self {
            original,
            target,
            original_metadata,
            target_metadata,
        })
    }

    /// Finalize one immediate directory pair under this binding, never its
    /// descendants. The caller admits routing ancestors and controls recursion.
    /// At the empty relative path, root steering reuses mandatory binding facts;
    /// stopped root descent returns that finalized root batch without listing.
    /// Otherwise the result describes the directory's immediate children.
    pub fn inspect_visitor<V: Visitor>(&self, dir: PathBuf, execution: Execution, visitor: &V) -> Result<DirectoryPlan, Error> {
        if V::ENABLED && dir.as_os_str().is_empty() {
            let root = crate::visitor::inspect_bound_root(&self.original, &self.target, &self.original_metadata, &self.target_metadata, visitor)?;
            if !root.operations().entries().get(0).expect("root entry").descends() {
                return Ok(root);
            }
        }
        crate::visitor::inspect_directory_execution(
            &self.original,
            &self.target,
            &dir,
            Some(&self.original.join(&dir)),
            Some(&self.target.join(&dir)),
            Purpose::Compare,
            visitor,
            execution,
        )
    }
}

#[cfg(all(test, feature = "parallel"))]
pub(crate) mod tests {
    use super::*;
    use std::sync::{Arc, Condvar, Mutex};

    static WORKERS: Mutex<Vec<(String, usize)>> = Mutex::new(Vec::new());
    static GATE: Mutex<Option<Arc<Gate>>> = Mutex::new(None);
    #[derive(Default)]
    struct Gate {
        arrivals: Mutex<usize>,
        changed: Condvar,
    }
    impl Gate {
        fn wait(&self) {
            let mut arrivals = self.arrivals.lock().unwrap();
            *arrivals += 1;
            self.changed.notify_all();
            let (arrivals, _) = self
                .changed
                .wait_timeout_while(arrivals, std::time::Duration::from_secs(5), |n| *n < 2)
                .unwrap();
            assert!(*arrivals >= 2, "no concurrent metadata job entered within five seconds");
        }
    }
    pub(crate) fn observe_worker() {
        // Other unit tests inspect batches concurrently in this test process.
        // Only this fixture's caller-owned pool participates in its gate/census.
        if !std::thread::current().name().is_some_and(|name| name.starts_with("diff-dirs-caller-")) {
            return;
        }
        let gate = {
            let mut workers = WORKERS.lock().unwrap();
            workers.push((std::thread::current().name().unwrap_or("unnamed").to_string(), rayon::current_num_threads()));
            if workers.len() <= 2 { GATE.lock().unwrap().clone() } else { None }
        };
        if let Some(gate) = gate {
            gate.wait();
        }
    }
    #[test]
    fn per_frame_metadata_jobs_use_the_callers_named_pool() {
        let f = crate::test_support::Fixture::new();
        for i in 0..64 {
            f.both_file(format!("file-{i:03}"), b"");
        }
        #[cfg(unix)]
        let _excluded = (
            std::os::unix::net::UnixListener::bind(f.original.join(".jj")).unwrap(),
            std::os::unix::net::UnixListener::bind(f.target.join(".jj")).unwrap(),
        );
        #[cfg(not(unix))]
        f.both_dir(".jj");
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(4)
            .thread_name(|i| format!("diff-dirs-caller-{i}"))
            .build()
            .unwrap();
        WORKERS.lock().unwrap().clear();
        *GATE.lock().unwrap() = Some(Arc::new(Gate::default()));
        let scope = |path: &Path| {
            if path.file_name() == Some(std::ffi::OsStr::new(".jj")) {
                crate::Visit::PreserveTree
            } else {
                crate::Visit::Reconcile
            }
        };
        let (roots, visitor) = PairedRoots::bind_scoped(&f.original, &f.target, crate::NoCheck, scope).unwrap();
        pool.install(|| {
            let plan = roots.inspect_visitor(PathBuf::new(), Execution::Parallel, &visitor).unwrap();
            assert!(plan.diff().is_empty());
            assert_eq!(plan.paired_children().count(), 0);
            assert_eq!(plan.excluded(), 1);
        });
        *GATE.lock().unwrap() = None;
        let workers = WORKERS.lock().unwrap();
        assert_eq!(workers.len(), 64); // one coupled union-name metadata job
        assert!(workers.iter().all(|(name, size)| name.starts_with("diff-dirs-caller-") && *size == 4));
        assert_ne!(workers[0].0, workers[1].0);
    }
}
