use crate::Absence;
use diff_dirs::{Action, Check, CheckError, CheckedScope, Container, Names, Operations, Presence, Purpose, Scope, Visitor};

/// Compiled compatibility adapters and the caller's visitor, sharing the same
/// immutable facts and mutable action slots. No filesystem engine consults old
/// scope/absence rules separately from this finalization.
pub(crate) struct Steering<'a, C, S, A, V> {
    pub check: &'a C,
    pub scope: &'a S,
    pub absence: &'a A,
    pub visitor: &'a V,
}

impl<C: Check, S: Scope, A: Absence, V: Visitor> Visitor for Steering<'_, C, S, A, V> {
    const ENABLED: bool = C::ENABLED || S::ENABLED || A::ENABLED || V::ENABLED;
    fn scoped(&self) -> bool {
        S::ENABLED || A::ENABLED || self.visitor.scoped()
    }
    fn early(&self, names: &mut Names) -> Result<(), CheckError> {
        CheckedScope::new(self.check.borrowed(), self.scope.borrowed()).early(names)?;
        if V::ENABLED {
            self.visitor.early(names)?;
        }
        Ok(())
    }
    fn late(&self, operations: &mut Operations) -> Result<(), CheckError> {
        CheckedScope::new(self.check.borrowed(), self.scope.borrowed()).late(operations)?;
        if A::ENABLED && operations.purpose() != Purpose::Audit {
            for entry in operations.entries_mut() {
                if entry.presence() == Presence::TargetOnly {
                    if A::NEVER {
                        entry.preserve_tree();
                    } else if !self.absence.remove(entry.relative_path()) {
                        entry.retain();
                    }
                }
            }
        }
        if V::ENABLED {
            self.visitor.late(operations)?;
        }
        Ok(())
    }
}

/// General recursive tree seed: create absent names, retain existing leaves and
/// type collisions, and descend matched directories. Target-only trees remain
/// untouched. This is not the caller-listed `seed_leaves` operation.
#[derive(Clone, Copy, Debug, Default)]
pub struct Seed;
impl Visitor for Seed {
    fn late(&self, operations: &mut Operations) -> Result<(), CheckError> {
        for entry in operations.entries_mut() {
            if entry.presence() != Presence::SourceOnly {
                if entry.source().is_some_and(|f| f.metadata().is_dir()) && entry.target().is_some_and(|f| f.metadata().is_dir()) {
                    entry.set_action(Action::Directory {
                        container: Container::Keep,
                        descend: true,
                    });
                } else {
                    entry.preserve_tree();
                }
            }
        }
        Ok(())
    }
}

/// Create and update source-present names, preserving whole target-only trees.
/// A directory-to-leaf collision still requires its target-only descendants to
/// be removable, so this preset refuses a nonempty such collision intact.
#[derive(Clone, Copy, Debug, Default)]
pub struct Update;
impl Visitor for Update {
    fn late(&self, operations: &mut Operations) -> Result<(), CheckError> {
        for entry in operations.entries_mut() {
            if entry.presence() == Presence::TargetOnly {
                entry.preserve_tree();
            }
        }
        Ok(())
    }
}

/// Remove target-only names while retaining source-present leaves and type
/// collisions. Matched directory containers are kept and descended, allowing
/// pruning below them. Source-only containers are not created. A collision is
/// never reinterpreted as absence permission for its removed-directory half.
#[derive(Clone, Copy, Debug, Default)]
pub struct Prune;
impl Visitor for Prune {
    fn late(&self, operations: &mut Operations) -> Result<(), CheckError> {
        for entry in operations.entries_mut() {
            if entry.presence() != Presence::TargetOnly {
                if entry.source().is_some_and(|f| f.metadata().is_dir()) && entry.target().is_some_and(|f| f.metadata().is_dir()) {
                    entry.retain();
                } else {
                    entry.preserve_tree();
                }
            }
        }
        Ok(())
    }
}
