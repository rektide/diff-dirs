//! Compatibility source-absence policy, projected into late target-only actions.
//! The filesystem kernel consumes final Visitor actions, not this trait directly.

use std::path::Path;

/// Authority for source absence, independent of presence updates and scope.
/// Custom policies may select nested deletions below retained directory names.
///
/// [`diff_dirs::Scope`] always wins: preserved names/interiors are never observed
/// or removed. For participating target-only leaves, false retains the leaf;
/// for directories it retains the name but does **not** retain all descendants.
/// Arbitrary mixed policies therefore require target-only descent to discover
/// nested deletable regions. Necessary ancestors survive retained descendants.
/// Only policies declaring [`Self::NEVER`] (such as [`NeverRemove`]) can prune
/// an entire such branch.
///
/// Paths are logical final root-relative paths, never staging names. Decisions
/// must be pure, stable, and Sync over frozen caller facts, like Scope/Check;
/// callbacks are not ordered or exactly-once notifications. Source presence
/// normally replaces matching unpreserved paths irrespective of this policy.
/// Directory-to-leaf replacement is exceptional: target-only descendants are
/// locally prechecked for removability before dismantling the old subtree.
/// Retention there blocks replacement with its path, not partial sibling removal.
///
/// ```
/// use diff_dirs_sync::Absence;
/// use std::path::Path;
/// let cache_authority = |p: &Path| p.starts_with("old/cache");
/// assert!(!cache_authority.remove(Path::new("old")));
/// assert!(cache_authority.remove(Path::new("old/cache/stale")));
/// ```
///
/// Retained-only reconciliation makes no namespace, staging, or chmod writes.
/// Other failures remain partial progress; see [`crate::Report`] and
/// [`crate::Failure`], not a whole-root equality/rollback promise.
pub trait Absence: Sync {
    /// False promises unconditional removal authority; compatibility forwarding
    /// can then elide this late visitor adapter.
    const ENABLED: bool = true;
    /// Set true only when every path is retained. This permits pruning a whole
    /// target-only branch without querying descendants (as in NeverRemove).
    const NEVER: bool = false;
    /// Authorize removing a participating source-absent name. Keeping a
    /// directory name alone cannot suppress a nested true result.
    fn remove(&self, relative: &Path) -> bool;
}

impl<F: Fn(&Path) -> bool + Sync> Absence for F {
    fn remove(&self, relative: &Path) -> bool {
        self(relative)
    }
}

#[derive(Clone, Copy, Debug, Default)]
/// Update authority: retain every target-only branch without interior descent.
/// This still permits source-presence updates and replacing an empty directory
/// with a source leaf; nonempty-directory replacement has retained descendants.
pub struct NeverRemove;
impl Absence for NeverRemove {
    const NEVER: bool = true;
    fn remove(&self, _: &Path) -> bool {
        false
    }
}

#[derive(Clone, Copy, Debug, Default)]
/// Mirror authority for absence everywhere that the pair-wide scope permits.
/// Preservation still retains protected descendants and their real ancestors.
pub struct RemoveAll;
impl Absence for RemoveAll {
    const ENABLED: bool = false;
    fn remove(&self, _: &Path) -> bool {
        true
    }
}
