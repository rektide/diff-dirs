use std::path::Path;

/// Pair-wide name admission, before metadata, readlink, or child admission.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Visit {
    Reconcile,
    PreserveTree,
}

/// Pure, stable, command-local path policy. Paths are logical root-relative
/// names, never private staging paths. A preserved ancestor cannot be reopened.
pub trait Scope: Sync {
    const ENABLED: bool = true;
    fn at(&self, relative: &Path) -> Visit;

    fn borrowed(&self) -> ScopeRef<'_, Self> {
        ScopeRef(self)
    }

    /// Include the path only when it and every ancestor participate. Public
    /// record consumers must use this rather than checking just the basename.
    fn includes(&self, relative: &Path) -> bool {
        !Self::ENABLED || relative.ancestors().all(|path| self.at(path) == Visit::Reconcile)
    }
}

impl<F: Fn(&Path) -> Visit + Sync> Scope for F {
    fn at(&self, relative: &Path) -> Visit {
        self(relative)
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct NoScope;

impl Scope for NoScope {
    const ENABLED: bool = false;
    fn at(&self, _: &Path) -> Visit {
        Visit::Reconcile
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ScopeRef<'a, S: ?Sized>(pub &'a S);

impl<S: Scope + ?Sized> Scope for ScopeRef<'_, S> {
    const ENABLED: bool = S::ENABLED;
    fn at(&self, relative: &Path) -> Visit {
        self.0.at(relative)
    }
}
