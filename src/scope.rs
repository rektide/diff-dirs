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
}

/// Include the path only when it and every ancestor participate. Public record
/// consumers must use this rather than checking only the leaf. Ancestors are
/// evaluated root-first; excluded descendants are never reopened.
pub fn scope_includes<S: Scope + ?Sized>(scope: &S, relative: &Path) -> bool {
    if !S::ENABLED {
        return true;
    }
    if scope.at(Path::new("")) == Visit::PreserveTree {
        return false;
    }
    let mut prefix = std::path::PathBuf::new();
    for component in relative.components() {
        prefix.push(component);
        if scope.at(&prefix) == Visit::PreserveTree {
            return false;
        }
    }
    true
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
