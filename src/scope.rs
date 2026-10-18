use std::path::Path;

/// Pair-wide name admission, before metadata, readlink, or child admission.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Visit {
    /// Observe/compare this name and admit any paired-directory descent.
    Reconcile,
    /// Exclude this name and its entire subtree on both sides. Hidden source
    /// presence must never become apparent source absence/deletion authority.
    PreserveTree,
}

/// Pure, stable, command-local path policy. Paths are logical root-relative
/// names, never private staging paths. A preserved ancestor cannot be reopened.
///
/// Selection is pair-wide and precedes child metadata/readlink, [`crate::Check`],
/// and scheduler admission, even for equal names or opaque one-sided directories.
/// Root identity/type validation remains mandatory. Preserving the empty path
/// suppresses root contents, not that structural validation. Mutation consumers
/// must enforce the same scope through recursive operations, not just filter
/// comparison records; the optional sync crate does so even inside staging.
///
/// Like Check, scope decisions must be pure and deterministic over frozen caller
/// facts: no filesystem I/O/mutation, processes, printing, or dependence on call
/// order/count. Parallel workers may invoke them speculatively and repeatedly,
/// outside scheduler locks. There is no ordered notification guarantee.
///
/// ```
/// use diff_dirs::{scope_includes, Visit};
/// use std::path::Path;
/// let scope = |p: &Path| if p == Path::new("local") {
///     Visit::PreserveTree
/// } else { Visit::Reconcile };
/// assert!(!scope_includes(&scope, Path::new("local/deep/reincluded")));
/// assert!(scope_includes(&scope, Path::new("shared/file")));
/// ```
///
/// [`crate::Summary::scoped`] distinguishes scoped exhaustion from full-root
/// cleanliness. Excluded boundaries are not an inventory of their interiors.
pub trait Scope: Sync {
    /// False promises unconditional Reconcile with no reliance on invocation.
    /// Implementations may be skipped entirely; leave true for any exclusion.
    /// This associated constant enables static no-scope specialization.
    const ENABLED: bool = true;
    /// Classify one logical relative name (empty for root). Normal traversal has
    /// already admitted its ancestors; public-record consumers must separately
    /// enforce them with [`scope_includes`].
    fn at(&self, relative: &Path) -> Visit;

    /// Lend the same frozen policy without losing its ENABLED specialization.
    fn borrowed(&self) -> ScopeRef<'_, Self> {
        ScopeRef(self)
    }
}

/// Include the path only when it and every ancestor participate. Public record
/// consumers must use this rather than checking only the leaf. Ancestors are
/// evaluated root-first; excluded descendants are never reopened.
/// The caller must validate a public path's normal relative syntax before using
/// it for filesystem access. This helper makes no observations and grants no
/// mutation authority. Traversals with already-admitted ancestors can call `at`
/// directly instead of rechecking every prefix for each child.
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
/// Include every name; the disabled associated constant elides name selection.
pub struct NoScope;

impl Scope for NoScope {
    const ENABLED: bool = false;
    fn at(&self, _: &Path) -> Visit {
        Visit::Reconcile
    }
}

#[derive(Clone, Copy, Debug)]
/// Borrowed policy forwarding both decisions and no-op specialization.
pub struct ScopeRef<'a, S: ?Sized>(pub &'a S);

impl<S: Scope + ?Sized> Scope for ScopeRef<'_, S> {
    const ENABLED: bool = S::ENABLED;
    fn at(&self, relative: &Path) -> Visit {
        self.0.at(relative)
    }
}
