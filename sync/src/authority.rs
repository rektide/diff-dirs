use std::path::Path;

/// Authority for source absence, independent of presence updates and scope.
/// Custom policies may select nested deletions below retained directory names.
pub trait Absence: Sync {
    /// Set true only when every path is retained. This permits pruning a whole
    /// target-only branch without querying descendants (as in NeverRemove).
    const NEVER: bool = false;
    fn remove(&self, relative: &Path) -> bool;
}

impl<F: Fn(&Path) -> bool + Sync> Absence for F {
    fn remove(&self, relative: &Path) -> bool {
        self(relative)
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct NeverRemove;
impl Absence for NeverRemove {
    const NEVER: bool = true;
    fn remove(&self, _: &Path) -> bool {
        false
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct RemoveAll;
impl Absence for RemoveAll {
    fn remove(&self, _: &Path) -> bool {
        true
    }
}
