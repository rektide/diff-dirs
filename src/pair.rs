//! Immediate inspection and the legacy comparison projection of visitor plans.

use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};

use crate::{Check, DirDiff, DirectoryPlan, Entry, Error, Execution, Purpose, Side, Visitor};
#[cfg(feature = "parallel")]
use crate::{CheckedScope, Scope};

#[derive(Debug)]
pub(crate) struct Roots {
    pub original: PathBuf,
    pub target: PathBuf,
    original_metadata: fs::Metadata,
    target_metadata: fs::Metadata,
}

impl Roots {
    #[cfg(all(test, feature = "parallel"))]
    pub fn new(original: impl AsRef<Path>, target: impl AsRef<Path>) -> Result<Self, Error> {
        Self::new_checked(original, target, &crate::NoCheck)
    }

    #[cfg(all(test, feature = "parallel"))]
    pub fn inspect(&self, dir: PathBuf, execution: Execution) -> Result<ComparedPair, Error> {
        self.inspect_scoped(dir, execution, &crate::NoCheck, &crate::NoScope)
    }

    pub fn new_checked<C: Check>(original: impl AsRef<Path>, target: impl AsRef<Path>, check: &C) -> Result<Self, Error> {
        let original = original.as_ref().to_path_buf();
        let target = target.as_ref().to_path_buf();
        let inspect = |path: &Path, side| {
            let metadata = fs::symlink_metadata(path).map_err(|e| Error::io(path.to_path_buf(), "inspect root", e))?;
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

    #[cfg(feature = "parallel")]
    pub fn inspect_scoped<C: Check, S: Scope>(&self, dir: PathBuf, execution: Execution, check: &C, scope: &S) -> Result<ComparedPair, Error> {
        self.inspect_visitor(dir, execution, &CheckedScope::new(check.borrowed(), scope.borrowed()).roots_checked())
    }

    pub fn inspect_visitor<V: Visitor>(&self, dir: PathBuf, execution: Execution, visitor: &V) -> Result<ComparedPair, Error> {
        if V::ENABLED && dir.as_os_str().is_empty() {
            let root = crate::visitor::inspect_bound_root(&self.original, &self.target, &self.original_metadata, &self.target_metadata, visitor)?;
            if !root.operations().entries()[0].descends() {
                return Ok(ComparedPair {
                    diff: DirDiff::default(),
                    children: Vec::new(),
                    excluded: root.excluded(),
                    scoped: visitor.scoped(),
                    planned: root.work(),
                    retained: root.retained(),
                    plan: None,
                });
            }
        }
        let plan = crate::visitor::inspect_directory_execution(
            &self.original,
            &self.target,
            &dir,
            Some(&self.original.join(&dir)),
            Some(&self.target.join(&dir)),
            Purpose::Compare,
            visitor,
            execution,
        )?;
        let diff = plan.diff().clone();
        let changed = !diff.is_empty();
        Ok(ComparedPair {
            diff,
            children: plan.children(),
            excluded: plan.excluded(),
            scoped: visitor.scoped(),
            planned: plan.work(),
            retained: plan.retained(),
            // Clean metadata is no longer needed after late and child lowering.
            // Release it in the worker, not while holding the frontier mutex.
            plan: changed.then_some(plan),
        })
    }
}

#[derive(Debug)]
pub(crate) struct ComparedPair {
    pub diff: DirDiff,
    pub children: Vec<OsString>,
    pub excluded: u64,
    pub scoped: bool,
    pub planned: u64,
    pub retained: u64,
    pub plan: Option<DirectoryPlan>,
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
        let roots = Roots::new_checked(&f.original, &f.target, &crate::NoCheck).unwrap();
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
        pool.install(|| {
            let pair = roots.inspect_scoped(PathBuf::new(), Execution::Parallel, &crate::NoCheck, &scope).unwrap();
            assert!(pair.diff.is_empty());
            assert!(pair.children.is_empty());
            assert_eq!(pair.excluded, 1);
        });
        *GATE.lock().unwrap() = None;
        let workers = WORKERS.lock().unwrap();
        assert_eq!(workers.len(), 64); // one coupled union-name metadata job
        assert!(workers.iter().all(|(name, size)| name.starts_with("diff-dirs-caller-") && *size == 4));
        assert_ne!(workers[0].0, workers[1].0);
    }
}
