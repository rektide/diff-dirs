//! Traversal's private projection of finalized immediate inspection.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use crate::{ApplicationNeed, Check, CheckedScope, DirDiff, DirectoryCoverage, DirectoryPlan, Error, Execution, PairedRoots, Scope, SemanticId, Visitor};

#[derive(Debug)]
pub(crate) struct Roots(PairedRoots);

impl Roots {
    pub fn new(original: impl AsRef<Path>, target: impl AsRef<Path>) -> Result<Self, Error> {
        PairedRoots::new(original, target).map(Self)
    }

    pub fn bind_scoped<C: Check, S: Scope>(
        original: impl AsRef<Path>,
        target: impl AsRef<Path>,
        check: C,
        scope: S,
    ) -> Result<(Self, CheckedScope<C, S>), Error> {
        PairedRoots::bind_scoped(original, target, check, scope).map(|(roots, visitor)| (Self(roots), visitor))
    }

    #[cfg(all(test, feature = "parallel"))]
    pub fn inspect(&self, dir: PathBuf, execution: Execution) -> Result<ComparedPair, Error> {
        self.inspect_visitor(dir, execution, &crate::NoVisitor)
    }

    pub fn inspect_visitor<V: Visitor>(&self, dir: PathBuf, execution: Execution, visitor: &V) -> Result<ComparedPair, Error> {
        let plan = self.0.inspect_visitor(dir, execution, visitor)?;
        let diff = plan.diff().clone();
        let changed = !diff.is_empty();
        let publish = changed || plan.selection().coverage().application_need != ApplicationNeed::None;
        Ok(ComparedPair {
            diff,
            children: plan.paired_children().map(|name| name.to_os_string()).collect(),
            coverage: plan.selection().coverage(),
            comparator: plan.selection().comparator().clone(),
            // Clean metadata is no longer needed after late and child lowering.
            // Release it in the worker, not while holding the frontier mutex.
            plan: publish.then_some(plan),
        })
    }
}

#[derive(Debug)]
pub(crate) struct ComparedPair {
    pub diff: DirDiff,
    pub children: Vec<OsString>,
    pub coverage: DirectoryCoverage,
    pub comparator: SemanticId,
    pub plan: Option<DirectoryPlan>,
}
