//! Traversal's private projection of finalized immediate inspection.

use std::ffi::OsString;
use std::path::Path;
#[cfg(feature = "parallel")]
use std::path::PathBuf;
use std::sync::Arc;

use crate::{
    ApplicationNeed, Check, CheckedScope, Comparator, DirDiff, DirectoryCoverage, Error, Execution, ExecutionDemand, PairedRoots, Policy, PreparedDirectory,
    Scope, SemanticId, SourceSession,
};
#[cfg(feature = "parallel")]
use crate::{FullVisitor, NoExecutionDemand, StatKey, Visitor};

#[derive(Debug)]
pub(crate) struct Roots {
    paired: PairedRoots,
    source: Arc<SourceSession>,
}

impl Roots {
    pub fn new(original: impl AsRef<Path>, target: impl AsRef<Path>) -> Result<Self, Error> {
        Self::with_source(Arc::new(SourceSession::new(original)), target)
    }

    pub fn with_source(source: Arc<SourceSession>, target: impl AsRef<Path>) -> Result<Self, Error> {
        Ok(Self {
            paired: PairedRoots::new(source.root(), target)?,
            source,
        })
    }

    pub fn source_session(&self) -> &Arc<SourceSession> {
        &self.source
    }

    pub fn bind_scoped<C: Check, S: Scope>(
        original: impl AsRef<Path>,
        target: impl AsRef<Path>,
        check: C,
        scope: S,
    ) -> Result<(Self, CheckedScope<C, S>), Error> {
        PairedRoots::bind_scoped(&original, target, check, scope).map(|(paired, visitor)| {
            (
                Self {
                    paired,
                    source: Arc::new(SourceSession::new(original)),
                },
                visitor,
            )
        })
    }

    #[cfg(all(test, feature = "parallel"))]
    pub fn inspect(&self, dir: PathBuf, execution: Execution) -> Result<ComparedPair, Error> {
        self.inspect_visitor(dir, execution, &crate::NoVisitor)
    }

    #[cfg(feature = "parallel")]
    pub fn inspect_visitor<V: Visitor>(&self, dir: PathBuf, execution: Execution, visitor: &V) -> Result<ComparedPair, Error> {
        self.inspect_policy(&dir, &(), execution, &FullVisitor(visitor.borrowed()), &StatKey, &NoExecutionDemand)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn inspect_policy<P: Policy, M: Comparator, E: ExecutionDemand>(
        &self,
        dir: &Path,
        context: &P::Context,
        execution: Execution,
        policy: &P,
        comparator: &M,
        demand: &E,
    ) -> Result<ComparedPair<P::Context>, Error> {
        let plan = self.paired.inspect_policy(&self.source, dir, context, policy, comparator, demand, execution)?;
        let diff = plan.selection().diff().clone();
        let changed = !diff.is_empty();
        let publish = changed || plan.selection().coverage().application_need != ApplicationNeed::None;
        Ok(ComparedPair {
            diff,
            children: plan
                .selection()
                .paired_children()
                .map(|entry| Child {
                    name: entry.name().to_owned(),
                    context: entry.interior.as_ref().expect("paired").context.clone(),
                })
                .collect(),
            coverage: plan.selection().coverage(),
            comparator: plan.selection().comparator().clone(),
            // Clean metadata is no longer needed after late and child lowering.
            // Release it in the worker, not while holding the frontier mutex.
            plan: publish.then_some(plan),
        })
    }
}

#[derive(Debug)]
pub(crate) struct ComparedPair<C = ()> {
    pub diff: DirDiff,
    pub children: Vec<Child<C>>,
    pub coverage: DirectoryCoverage,
    pub comparator: SemanticId,
    pub plan: Option<PreparedDirectory<C>>,
}

#[derive(Clone, Debug)]
pub(crate) struct Child<C> {
    pub name: OsString,
    pub context: C,
}
