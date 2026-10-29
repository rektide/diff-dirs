use std::collections::HashSet;
use std::ffi::OsStr;
use std::fs::{self, File, Metadata, OpenOptions, Permissions};
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use diff_dirs_tree_inspect::{
    Action, Check, Container, DirectoryPlan, Entry, NoCheck, NoScope, NoVisitor, PreparedOperation as Operation, Purpose, Scope, Side, Visitor,
};

use crate::model::merge;
use crate::paths::{self, ADD_DIR, ADD_LEAF, CHANGED, REMOVE_DIR, REMOVE_LEAF};
use crate::{Absence, Applier, CopyPolicy, Failure, Policy, Progress, RemoveAll, Report, platform};

impl<C: Check, S: Scope, A: Absence, V: Visitor> Applier<C, S, A, V> {
    pub(crate) fn apply_with(&self, diff: &diff_dirs_tree_inspect::DirDiff, parallel: bool) -> Result<Progress, Failure> {
        self.apply_reported(diff, parallel).map(|report| report.completed)
    }

    /// Apply one public record with separate operation and policy outcomes.
    /// Syntax/types are validated before mutation, and every directory/name
    /// ancestor is scope-enforced before entry observation. Opaque removed dirs
    /// mean source absence under this binding, never unconditional rm-rf.
    /// Satisfaction covers only this record, not unmentioned equal entries or
    /// the whole roots. Earlier allowed work can remain on recursive I/O failure.
    pub fn apply_report(&self, diff: &diff_dirs_tree_inspect::DirDiff) -> Result<Report, Failure> {
        self.apply_reported(diff, false)
    }

    pub(crate) fn apply_reported(&self, diff: &diff_dirs_tree_inspect::DirDiff, parallel: bool) -> Result<Report, Failure> {
        self.apply_public(diff, parallel, None)
    }

    fn apply_public(&self, diff: &diff_dirs_tree_inspect::DirDiff, parallel: bool, expected: Option<&DirectoryPlan>) -> Result<Report, Failure> {
        let names = paths::names(diff)?;
        let context = Context::checked(
            self.policy,
            parallel,
            &self.original,
            &self.target,
            &self.check,
            &self.scope,
            &self.absence,
            &self.visitor,
        );
        let result = (|| {
            paths::directory(&self.original)?;
            paths::directory(&self.target)?;
            let root = diff_dirs_tree_inspect::inspect_root(&self.original, &self.target, Purpose::Compare, &self.steering()).map_err(Failure::checked)?;
            context.excluded.fetch_add(root.excluded(), Ordering::Relaxed);
            context.retained.fetch_add(root.retained(), Ordering::Relaxed);
            if !root.operations().entries().get(0).expect("root entry").descends() {
                if expected.is_some() {
                    return Err(Failure::invalid("apply plan", &self.target, "root descent is not admitted"));
                }
                return Ok(Progress::default());
            }
            let mut dir = PathBuf::new();
            for component in diff.dir.components() {
                let plan = context.plan(&dir, Some(&self.original.join(&dir)), Some(&self.target.join(&dir)))?;
                let entry = plan.operations().entries().iter().find(|e| e.name() == component.as_os_str());
                let Some(entry) = entry else {
                    if plan.preserves(component.as_os_str()) {
                        if expected.is_some() {
                            return Err(Failure::invalid("apply plan", &dir.join(component), "ancestor is preserved"));
                        }
                        return Ok(Progress::default());
                    }
                    return Err(Failure::invalid("validate record ancestor", &dir, "ancestor missing"));
                };
                if entry.action() == Some(Action::PreserveTree) {
                    if expected.is_some() {
                        return Err(Failure::invalid("apply plan", entry.relative_path(), "ancestor is preserved"));
                    }
                    context.retained.fetch_add(1, Ordering::Relaxed);
                    return Ok(Progress::default());
                }
                if !entry.source().is_some_and(|f| f.metadata().is_dir()) || !entry.target().is_some_and(|f| f.metadata().is_dir()) {
                    return Err(Failure::invalid(
                        "validate record ancestor",
                        entry.relative_path(),
                        "both ancestors must be real directories",
                    ));
                }
                if !entry.descends() {
                    if expected.is_some() {
                        return Err(Failure::invalid("apply plan", entry.relative_path(), "ancestor descent is not admitted"));
                    }
                    return Ok(Progress::default());
                }
                dir.push(component);
            }
            let plan = context.plan(&dir, Some(&self.original.join(&dir)), Some(&self.target.join(&dir)))?;
            if let Some(expected) = expected
                && plan.selection() != expected.selection()
            {
                return Err(Failure::invalid(
                    "apply plan",
                    &dir,
                    "batch projection does not match current classification and this binding's finalized visitor actions",
                ));
            }
            for (name, mask) in &names {
                if let Ok(index) = plan
                    .operations()
                    .entries()
                    .binary_search_by(|e| e.name().as_encoded_bytes().cmp(name.as_encoded_bytes()))
                {
                    let entry = plan.operations().entries().get(index).expect("located entry");
                    if shape(&entry) != Some(*mask) {
                        return Err(Failure::invalid("validate record", &dir.join(name), "entry type does not match record"));
                    }
                } else if !plan.preserves(name) {
                    return Err(Failure::invalid("validate record", &dir.join(name), "entry missing"));
                }
            }
            context.execute_batch(&plan, &self.original.join(&dir), &self.target.join(&dir), false, Some(&names))
        })();
        context.finish(result)
    }

    /// Apply an externally obtained finalized batch. This public boundary freshly
    /// validates real routing ancestors and re-finalizes under **this binding's**
    /// visitor. It compares the legacy diff, names, actions, presence and supported
    /// comparison classification; a mismatch refuses before writes. It does not
    /// compare every metadata value or file bytes with the supplied plan. Actual
    /// work always uses fresh facts, not a stale plan's metadata certificate.
    /// The internal run/walker seam reuses just-observed facts without this extra
    /// public validation. Neither interface is a concurrent-replacement sandbox.
    pub fn apply_plan(&self, plan: &DirectoryPlan) -> Result<Report, Failure> {
        if plan.operations().source_root() != self.original || plan.operations().target_root() != self.target || plan.operations().is_root() {
            return Err(Failure::invalid(
                "apply plan",
                plan.operations().dir(),
                "plan does not belong to these directory roots",
            ));
        }
        self.apply_public(plan.diff(), false, Some(plan))
    }

    pub(crate) fn apply_plan_with(&self, plan: &DirectoryPlan, parallel: bool) -> Result<Report, Failure> {
        if plan.operations().source_root() != self.original || plan.operations().target_root() != self.target || plan.operations().is_root() {
            return Err(Failure::invalid(
                "apply plan",
                plan.operations().dir(),
                "plan does not belong to these directory roots",
            ));
        }
        let context = Context::checked(
            self.policy,
            parallel,
            &self.original,
            &self.target,
            &self.check,
            &self.scope,
            &self.absence,
            &self.visitor,
        );
        context.excluded.fetch_add(plan.excluded(), Ordering::Relaxed);
        let dir = plan.operations().dir();
        let result = context.execute_batch(plan, &self.original.join(dir), &self.target.join(dir), false, None);
        context.finish(result)
    }

    /// Read-only joint preview of the same root and recursive visitor actions as
    /// run, including one-sided interiors and coupled collision blockers. It
    /// never probes clone capability, stages, chmods, or writes. Planned counters
    /// are predictions from current facts, not completed work or durable authority.
    /// This is explicit extra work for callers requesting preview, never an
    /// implicit preaudit performed by normal execution. `observed` is None: this
    /// is not the legacy both-directory comparison projection.
    pub fn preview(&self) -> Result<Report, Failure> {
        paths::directory(&self.original)?;
        paths::root_with_leaf(&self.target, true, self.replace_root)?;
        let context = Context::checked(
            self.policy,
            false,
            &self.original,
            &self.target,
            &self.check,
            &self.scope,
            &self.absence,
            &self.visitor,
        );
        let result = (|| {
            let root = diff_dirs_tree_inspect::inspect_root(&self.original, &self.target, Purpose::Compare, &self.steering()).map_err(Failure::checked)?;
            context.excluded.fetch_add(root.excluded(), Ordering::Relaxed);
            context.retained.fetch_add(root.retained(), Ordering::Relaxed);
            context.planned.fetch_add(unconditional_work(&root), Ordering::Relaxed);
            context.preview_entry(&root.operations().entries().get(0).expect("root entry"))?;
            Ok(Progress::default())
        })();
        context.finish(result)
    }

    pub(crate) fn initialize_with(&self, parallel: bool) -> Result<Progress, Failure> {
        self.initialize_reported(parallel).map(|report| report.completed)
    }

    pub(crate) fn initialize_reported(&self, parallel: bool) -> Result<Report, Failure> {
        paths::directory(&self.original)?;
        let context = Context::checked(
            self.policy,
            parallel,
            &self.original,
            &self.target,
            &self.check,
            &self.scope,
            &self.absence,
            &self.visitor,
        );
        // Recheck the bound path's ancestors before mutation.
        let rebound = paths::root(&self.target, true)?;
        if rebound != self.target {
            return Err(Failure::invalid("initialize", &self.target, "target root changed"));
        }
        if paths::exists(&self.target)? {
            return Err(Failure::invalid("initialize", &self.target, "target already exists"));
        }
        let parent = self.target.parent().expect("bound root has parent");
        paths::directory(parent)?;
        let plan = diff_dirs_tree_inspect::inspect_root(&self.original, &self.target, Purpose::Copy, &self.steering()).map_err(Failure::checked)?;
        context.excluded.fetch_add(plan.excluded(), Ordering::Relaxed);
        context.retained.fetch_add(plan.retained(), Ordering::Relaxed);
        context.planned.fetch_add(unconditional_work(&plan), Ordering::Relaxed);
        let arena = Staging::new(parent, std::iter::once(self.target.file_name().expect("root name")));
        let result = context
            .execute_entry(
                &self.original,
                &self.target,
                &plan.operations().entries().get(0).expect("root entry"),
                &arena,
                0,
                true,
            )
            .map(|(done, _)| done);
        context.finish(arena.finish(result, &context))
    }

    pub(crate) fn replace_root_with(&self, parallel: bool) -> Result<Progress, Failure> {
        self.replace_root_reported(parallel).map(|report| report.completed)
    }

    pub(crate) fn replace_root_reported(&self, parallel: bool) -> Result<Report, Failure> {
        if !self.replace_root {
            return Err(Failure::invalid("replace root", &self.target, "requires explicit replacing root binding"));
        }
        let rebound = paths::root_with_leaf(&self.target, false, true)?;
        if rebound != self.target {
            return Err(Failure::invalid("replace root", &self.target, "target root changed"));
        }
        let context = Context::checked(
            self.policy,
            parallel,
            &self.original,
            &self.target,
            &self.check,
            &self.scope,
            &self.absence,
            &self.visitor,
        );
        paths::directory(&self.original)?;
        let target = fs::symlink_metadata(&self.target).map_err(|error| Failure::io("inspect target root", &self.target, error))?;
        if !matches!(metadata_kind(&self.target, &target)?, Kind::File | Kind::Symlink) {
            return Err(Failure::invalid("replace root", &self.target, "expected existing target leaf"));
        }
        let parent = self.target.parent().expect("bound leaf root has parent");
        let plan = diff_dirs_tree_inspect::inspect_root(&self.original, &self.target, Purpose::Copy, &self.steering()).map_err(Failure::checked)?;
        context.excluded.fetch_add(plan.excluded(), Ordering::Relaxed);
        context.retained.fetch_add(plan.retained(), Ordering::Relaxed);
        context.planned.fetch_add(unconditional_work(&plan), Ordering::Relaxed);
        let arena = Staging::new(parent, std::iter::once(self.target.file_name().expect("leaf root has name")));
        let result = context
            .execute_entry(
                &self.original,
                &self.target,
                &plan.operations().entries().get(0).expect("root entry"),
                &arena,
                0,
                true,
            )
            .map(|(done, _)| done);
        let result = arena.finish(result, &context);
        context.finish(result)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Directory,
    File,
    Symlink,
}

fn inspect<C: Check, S: Scope, A: Absence, V: Visitor>(
    context: &Context<'_, C, S, A, V>,
    path: &Path,
    side: Side,
    purpose: Purpose,
) -> Result<Option<Observed>, Failure> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            let link = if metadata.is_symlink() && side == Side::Original {
                Some(fs::read_link(path).map_err(|error| Failure::io("read symlink", path, error))?)
            } else {
                None
            };
            context.observe(path, side, purpose, &metadata, link.as_deref())?;
            metadata_kind(path, &metadata)?;
            Ok(Some(Observed { metadata, link }))
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(Failure::io("inspect entry", path, error)),
    }
}

fn shape(entry: &Operation) -> Option<u8> {
    entry.comparison()?;
    let sd = entry.source().is_some_and(|f| f.metadata().is_dir());
    let td = entry.target().is_some_and(|f| f.metadata().is_dir());
    if entry.source().is_some() && entry.target().is_some() && !sd && !td {
        return Some(CHANGED);
    }
    let mut mask = 0;
    if entry.source().is_some() {
        mask |= if sd { ADD_DIR } else { ADD_LEAF };
    }
    if entry.target().is_some() {
        mask |= if td { REMOVE_DIR } else { REMOVE_LEAF };
    }
    Some(mask)
}

fn metadata_kind(path: &Path, metadata: &Metadata) -> Result<Kind, Failure> {
    if metadata.is_dir() {
        Ok(Kind::Directory)
    } else if metadata.is_file() {
        Ok(Kind::File)
    } else if metadata.is_symlink() {
        Ok(Kind::Symlink)
    } else {
        Err(Failure::invalid("inspect entry", path, "unsupported special entry"))
    }
}

struct Observed {
    metadata: Metadata,
    link: Option<PathBuf>,
}

struct Context<'a, C = NoCheck, S = NoScope, A = RemoveAll, V = NoVisitor> {
    policy: CopyPolicy,
    parallel: bool,
    stopped: AtomicBool,
    original: &'a Path,
    target: &'a Path,
    check: &'a C,
    scope: &'a S,
    absence: &'a A,
    visitor: &'a V,
    retained: AtomicU64,
    excluded: AtomicU64,
    planned: AtomicU64,
}

// Keep usable descriptors alive through final metadata and rename. A path-only
// clone helper cannot return the descriptor it has already closed.
struct PreparedLeaf {
    progress: Progress,
    _source: Option<File>,
    _output: Option<File>,
}

/// Active source-only routing ancestry. Physical output paths can be inside a
/// staging arena; policy coordinates remain on the Operation/plan. Frames live
/// only across this opaque recursive application and finalize in postorder.
struct PendingDirectory {
    path: PathBuf,
    permissions: Permissions,
    created: bool,
    relaxed: bool,
    needed: bool,
}

fn conditional(entry: &Operation) -> bool {
    matches!(
        entry.action(),
        Some(Action::Directory {
            container: Container::CreateIfNeeded,
            ..
        })
    )
}

fn unconditional_work(plan: &DirectoryPlan) -> u64 {
    plan.operations().entries().iter().filter(|e| e.has_work() && !conditional(e)).count() as u64
}

fn materialize_pending(pending: &mut [PendingDirectory], parent: &Writable<'_>) -> Result<(), Failure> {
    // The external parent is real and its batch owns this lease until recursive
    // application returns. Created routing frames stay writable until postorder.
    for frame in pending.iter_mut() {
        frame.needed = true;
    }
    parent.ensure()?;
    for frame in pending {
        if !frame.created {
            fs::create_dir(&frame.path).map_err(|error| Failure::io("create conditional routing directory", &frame.path, error))?;
            // Record ownership before any fallible metadata/permission work.
            // Each frame contributes actual creation once, when it unwinds.
            frame.created = true;
            frame.relaxed = relax(&frame.path)?.1;
        }
    }
    Ok(())
}

impl Context<'static> {
    fn new(policy: CopyPolicy, parallel: bool) -> Self {
        Self::checked(policy, parallel, Path::new(""), Path::new(""), &NoCheck, &NoScope, &RemoveAll, &NoVisitor)
    }
}

impl<'a, C: Check, S: Scope, A: Absence, V: Visitor> Context<'a, C, S, A, V> {
    #[allow(clippy::too_many_arguments)]
    fn checked(policy: CopyPolicy, parallel: bool, original: &'a Path, target: &'a Path, check: &'a C, scope: &'a S, absence: &'a A, visitor: &'a V) -> Self {
        Self {
            policy,
            parallel,
            stopped: AtomicBool::new(false),
            original,
            target,
            check,
            scope,
            absence,
            visitor,
            retained: AtomicU64::new(0),
            excluded: AtomicU64::new(0),
            planned: AtomicU64::new(0),
        }
    }

    fn finish(&self, result: Result<Progress, Failure>) -> Result<Report, Failure> {
        let policy = Policy {
            satisfied: result.is_ok(),
            retained: self.retained.load(Ordering::Relaxed),
            excluded: self.excluded.load(Ordering::Relaxed),
        };
        match result {
            Ok(completed) => Ok(Report {
                planned: self.planned.load(Ordering::Relaxed),
                completed,
                policy,
                observed: None,
            }),
            Err(mut failure) => {
                failure.planned += self.planned.load(Ordering::Relaxed);
                *failure.policy += policy;
                Err(failure)
            }
        }
    }

    fn steering(&self) -> crate::visitor::Steering<'_, C, S, A, V> {
        crate::visitor::Steering {
            check: self.check,
            scope: self.scope,
            absence: self.absence,
            visitor: self.visitor,
        }
    }

    fn plan(&self, relative: &Path, source: Option<&Path>, target: Option<&Path>) -> Result<DirectoryPlan, Failure> {
        let purpose = match (source.is_some(), target.is_some()) {
            (true, false) => Purpose::Copy,
            (false, true) => Purpose::Remove,
            _ => Purpose::Compare,
        };
        let plan = diff_dirs_tree_inspect::inspect_directory(self.original, self.target, relative, source, target, purpose, &self.steering())
            .map_err(Failure::checked)?;
        self.excluded.fetch_add(plan.excluded(), Ordering::Relaxed);
        Ok(plan)
    }

    fn execute_batch(
        &self,
        plan: &DirectoryPlan,
        from: &Path,
        to: &Path,
        recurse_pairs: bool,
        selected: Option<&[(std::ffi::OsString, u8)]>,
    ) -> Result<Progress, Failure> {
        self.retained.fetch_add(plan.retained(), Ordering::Relaxed);
        let entries: Vec<_> = plan
            .operations()
            .entries()
            .iter()
            .filter(|e| {
                selected.is_none_or(|names| {
                    names
                        .binary_search_by(|(name, _)| name.as_encoded_bytes().cmp(e.name().as_encoded_bytes()))
                        .is_ok()
                })
            })
            .collect();
        self.planned
            .fetch_add(entries.iter().filter(|e| e.has_work() && !conditional(e)).count() as u64, Ordering::Relaxed);
        let arena = Staging::new(to, plan.operations().entries().iter().map(|e| e.name()));
        let result = self.batch(&entries, |index, entry| {
            self.execute_entry(&from.join(entry.name()), &to.join(entry.name()), entry, &arena, index, recurse_pairs)
                .map(|(done, _)| done)
        });
        arena.finish(result, self)
    }

    fn preview_entry(&self, entry: &Operation) -> Result<bool, Failure> {
        let mut needed = entry.has_work() && !conditional(entry);
        let relative = entry.relative_path();
        let from = self.original.join(relative);
        let to = self.target.join(relative);
        let (source, target) = match entry.action().expect("finalized entry has an action") {
            Action::Directory { descend: true, .. } => (
                entry.source().is_some_and(|f| f.metadata().is_dir()).then_some(from.as_path()),
                entry.target().is_some_and(|f| f.metadata().is_dir()).then_some(to.as_path()),
            ),
            Action::ReplaceWithDirectory { descend: true } => (Some(from.as_path()), None),
            Action::ReplaceWithLeaf => {
                self.removable(&to, relative, true)?;
                (None, Some(to.as_path()))
            }
            _ => return Ok(needed),
        };
        let plan = self.plan(relative, source, target)?;
        self.retained.fetch_add(plan.retained(), Ordering::Relaxed);
        self.planned.fetch_add(unconditional_work(&plan), Ordering::Relaxed);
        for child in plan.operations().entries() {
            needed |= self.preview_entry(&child)?;
        }
        if conditional(entry) && needed {
            self.planned.fetch_add(1, Ordering::Relaxed);
        }
        Ok(needed)
    }

    fn execute_entry(
        &self,
        from: &Path,
        to: &Path,
        entry: &Operation,
        arena: &Staging<'_>,
        index: usize,
        recurse_pairs: bool,
    ) -> Result<(Progress, bool), Failure> {
        #[cfg(all(test, feature = "parallel"))]
        tests::observe(to);
        let action = entry.action().expect("finalized entry has an action");
        match action {
            Action::ConfirmAbsent => Err(Failure::invalid(
                "apply physical plan",
                entry.relative_path(),
                "listed absence obligation in physical execution",
            )),
            Action::PreserveTree | Action::KeepLeaf => Ok((Progress::default(), entry.target().is_some())),
            Action::InstallLeaf | Action::ReplaceWithLeaf => {
                if action == Action::ReplaceWithLeaf {
                    self.removable(to, entry.relative_path(), true)?;
                }
                let source = entry.source().expect("validated source leaf");
                self.copy_leaf_with(
                    from,
                    to,
                    action == Action::ReplaceWithLeaf,
                    source.metadata(),
                    source.link_target(),
                    &arena.entry(index, to)?,
                )
                .map(|done| (done, false))
            }
            Action::RemoveLeaf => {
                arena.writable.ensure()?;
                self.remove_leaf(to, entry.target().expect("validated target leaf").metadata())
                    .map(|done| (done, false))
            }
            Action::Directory {
                container: Container::Create,
                descend,
            } => {
                arena.writable.ensure()?;
                self.copy_directory_selected(from, to, entry.relative_path(), entry.source().expect("source directory").metadata(), descend)
                    .map(|done| (done, false))
            }
            Action::Directory {
                container: Container::CreateIfNeeded,
                ..
            } => self
                .copy_directory_pending(
                    from,
                    to,
                    entry.relative_path(),
                    entry.source().expect("source directory").metadata(),
                    &arena.writable,
                    &mut Vec::new(),
                )
                .map(|done| (done, false)),
            Action::ReplaceWithDirectory { descend } => {
                let staged = arena.entry(index, to)?;
                let prepared = self.copy_directory_selected(
                    from,
                    &staged,
                    entry.relative_path(),
                    entry.source().expect("source directory").metadata(),
                    descend,
                );
                match prepared {
                    Ok(progress) if !self.stopped.load(Ordering::Acquire) => {
                        let removed = self.remove_leaf(to, entry.target().expect("target leaf").metadata())?;
                        fs::rename(&staged, to).map_err(|error| {
                            let mut failure = Failure::io("install directory", to, error);
                            failure.completed = removed.clone();
                            failure
                        })?;
                        let mut progress = progress;
                        progress += removed;
                        Ok((progress, false))
                    }
                    Ok(_) => Ok((Progress::default(), false)),
                    Err(mut failure) => {
                        failure.completed = Progress::default();
                        Err(failure)
                    }
                }
            }
            Action::Directory { container, descend } => {
                if !descend {
                    return Ok((Progress::default(), true));
                }
                let paired = entry.source().is_some_and(|f| f.metadata().is_dir());
                if paired && !recurse_pairs {
                    return Ok((Progress::default(), true));
                }
                let source = paired.then_some(from);
                self.reconcile_directory(source, to, entry.relative_path(), container == Container::Remove, &arena.writable)
            }
        }
    }

    fn remove_leaf(&self, path: &Path, metadata: &Metadata) -> Result<Progress, Failure> {
        platform::remove_leaf(path, metadata).map_err(|error| Failure::io("remove leaf", path, error))?;
        Ok(if metadata.is_symlink() {
            Progress {
                symlinks_removed: 1,
                ..Progress::default()
            }
        } else {
            Progress {
                files_removed: 1,
                ..Progress::default()
            }
        })
    }

    fn copy_directory_selected(&self, from: &Path, to: &Path, relative: &Path, metadata: &Metadata, descend: bool) -> Result<Progress, Failure> {
        // The parent batch has authorized creating this container. Its children
        // are a separate batch; a later refusal can leave this legal ancestor.
        fs::create_dir(to).map_err(|error| Failure::io("create directory", to, error))?;
        let mut relaxed = false;
        let result = (|| {
            relaxed = relax(to)?.1;
            if !descend {
                return Ok(Progress::default());
            }
            let plan = self.plan(relative, Some(from), None)?;
            self.execute_batch(&plan, from, to, true, None)
        })();
        let mut result = finish_permissions(result, to, metadata.permissions(), relaxed, false);
        add_progress(
            &mut result,
            Progress {
                directories_created: 1,
                ..Progress::default()
            },
        );
        result
    }

    /// Opaque source-only recursion, not a discovery pass. A mutable depth-sized
    /// chain deliberately makes this part sequential even in a parallel batch;
    /// independent top-level entries can still execute in parallel. Once an
    /// admitted ordinary effect is encountered, use the same production entry
    /// kernel and parent-local staging as eager copying.
    fn copy_directory_pending(
        &self,
        from: &Path,
        to: &Path,
        relative: &Path,
        metadata: &Metadata,
        parent: &Writable<'_>,
        pending: &mut Vec<PendingDirectory>,
    ) -> Result<Progress, Failure> {
        if self.stopped.load(Ordering::Acquire) {
            return Ok(Progress::default());
        }
        pending.push(PendingDirectory {
            path: to.to_owned(),
            permissions: metadata.permissions(),
            created: false,
            relaxed: false,
            needed: false,
        });
        let result = (|| {
            // No target directory is needed to inspect source-only children.
            let plan = self.plan(relative, Some(from), None)?;
            self.retained.fetch_add(plan.retained(), Ordering::Relaxed);
            self.planned.fetch_add(unconditional_work(&plan), Ordering::Relaxed);
            let arena = Staging::new(to, plan.operations().entries().iter().map(|e| e.name()));
            let mut progress = Progress::default();
            let result = (|| {
                for (index, entry) in plan.operations().entries().iter().enumerate() {
                    if self.stopped.load(Ordering::Acquire) {
                        break;
                    }
                    let from = from.join(entry.name());
                    let to = to.join(entry.name());
                    let result = if conditional(&entry) {
                        self.copy_directory_pending(
                            &from,
                            &to,
                            entry.relative_path(),
                            entry.source().expect("source directory").metadata(),
                            parent,
                            pending,
                        )
                    } else {
                        (|| {
                            if entry.has_work() {
                                materialize_pending(pending, parent)?;
                            }
                            self.execute_entry(&from, &to, &entry, &arena, index, true).map(|(done, _)| done)
                        })()
                    };
                    match result {
                        Ok(done) => progress += done,
                        Err(mut failure) => {
                            failure.completed += progress;
                            return Err(failure);
                        }
                    }
                }
                Ok(progress)
            })();
            arena.finish(result, self)
        })();
        let frame = pending.pop().expect("active pending directory");
        if frame.needed {
            self.planned.fetch_add(1, Ordering::Relaxed);
        }
        if !frame.created {
            return result;
        }
        // Even failure finalizes every created routing ancestor, only after all
        // of its descendants/siblings have returned. Never chmod a pending name.
        let mut result = finish_permissions(result, &frame.path, frame.permissions, frame.relaxed, false);
        add_progress(
            &mut result,
            Progress {
                directories_created: 1,
                ..Progress::default()
            },
        );
        result
    }

    fn reconcile_directory(
        &self,
        source: Option<&Path>,
        path: &Path,
        relative: &Path,
        remove: bool,
        parent: &Writable<'_>,
    ) -> Result<(Progress, bool), Failure> {
        let plan = self.plan(relative, source, Some(path))?;
        self.retained.fetch_add(plan.retained(), Ordering::Relaxed);
        self.planned.fetch_add(unconditional_work(&plan), Ordering::Relaxed);
        let arena = Staging::new(path, plan.operations().entries().iter().map(|e| e.name()));
        let mut kept = !remove || plan.excluded() != 0;
        let mut progress = Progress::default();
        let result = (|| {
            for (index, entry) in plan.operations().entries().iter().enumerate() {
                let from = source.unwrap_or(self.original).join(entry.name());
                match self.execute_entry(&from, &path.join(entry.name()), &entry, &arena, index, true) {
                    Ok((done, retained)) => {
                        progress += done;
                        kept |= retained;
                    }
                    Err(mut failure) => {
                        failure.completed += progress.clone();
                        return Err(failure);
                    }
                }
            }
            Ok(progress.clone())
        })();
        let (result, writable) = arena.finish_preparation(result, self);
        let result = result.and_then(|mut progress| {
            if !kept {
                parent.ensure().map_err(|mut failure| {
                    failure.completed += progress.clone();
                    failure
                })?;
                fs::remove_dir(path).map_err(|error| {
                    let mut failure = Failure::io("remove directory", path, error);
                    failure.completed = progress.clone();
                    failure
                })?;
                progress.directories_removed += 1;
            } else {
                self.retained.fetch_add(1, Ordering::Relaxed);
            }
            Ok(progress)
        });
        // Keep the lease until removal actually succeeds. Retained or failed
        // directories regain their mode; removed ones need no redundant chmod.
        let result = if !kept && result.is_ok() {
            writable.removed(result)
        } else {
            writable.finish(result)
        };
        result.map(|progress| (progress, kept))
    }

    #[cfg(test)]
    fn source_relative<'p>(&self, path: &'p Path) -> &'p Path {
        path.strip_prefix(self.original).expect("bound source descendant")
    }

    fn target_relative<'p>(&self, path: &'p Path) -> &'p Path {
        path.strip_prefix(self.target).expect("bound target descendant")
    }

    /// Offer already-needed facts to the caller's accept/refuse policy.
    /// This direct compatibility helper is used by the explicit known-path seed
    /// installer; general tree work uses finalized Visitor batches instead.
    ///
    /// `C::ENABLED` is an associated constant of the concrete generic checker,
    /// not a per-entry runtime flag or a trait-object dispatch. For `NoCheck`
    /// (and borrowed wrappers forwarding false), this construction-site gate
    /// skips side-to-root selection, basename extraction, `Entry` construction,
    /// and the policy/error-mapping call. It does NOT skip metadata/readlink
    /// needed by the actual filesystem operation, scope selection, or type checks.
    ///
    /// `Entry::check` has its own ENABLED gate for callers that already have an
    /// observation. The two gates are semantically redundant for accept/refuse:
    /// in optimized monomorphizations, inlining and dead-code elimination could
    /// also remove the outer construction if we called it unconditionally. We
    /// keep the outer gate to state the boundary explicitly and avoid evaluating
    /// guard-only fields without relying on that cross-method optimization
    /// (including less-optimized builds and future field construction changes).
    /// This is a structural rationale, not a measured speedup claim.
    ///
    /// A checker declaring ENABLED=false promises unconditional acceptance and
    /// must not rely on callback invocation or side effects; it may never run.
    /// All policies that can refuse must retain ENABLED=true.
    #[inline]
    fn observe(&self, path: &Path, side: Side, purpose: Purpose, metadata: &Metadata, link_target: Option<&Path>) -> Result<(), Failure> {
        if C::ENABLED {
            let root = if side == Side::Original { self.original } else { self.target };
            Entry {
                root,
                path,
                name: path.file_name().unwrap_or(path.as_os_str()),
                side,
                purpose,
                metadata,
                link_target,
            }
            .check(self.check)
            .map_err(Failure::checked)?;
        }
        Ok(())
    }

    fn batch<T: Sync>(&self, items: &[T], operation: impl Fn(usize, &T) -> Result<Progress, Failure> + Sync) -> Result<Progress, Failure> {
        let run = |(index, item): (usize, &T)| {
            if self.stopped.load(Ordering::Acquire) {
                return Ok(Progress::default());
            }
            let result = operation(index, item);
            if result.is_err() {
                self.stopped.store(true, Ordering::Release);
            }
            result
        };
        #[cfg(feature = "parallel")]
        if self.parallel {
            use rayon::prelude::*;
            // Indexed collection joins all started siblings and retains native
            // name error order among encountered failures. Lazy Rayon splitting,
            // not one detached task per file or a whole-tree operation list.
            let results: Vec<_> = items.par_iter().enumerate().map(run).collect();
            return merge(results);
        }
        let _ = self.parallel;
        merge(items.iter().enumerate().map(run))
    }

    #[cfg(test)]
    fn named(&self, from: &Path, to: &Path, mask: u8, arena: &Staging<'_>, index: usize) -> Result<Progress, Failure> {
        if mask & ADD_DIR != 0 {
            if mask & REMOVE_LEAF != 0 {
                return self.staged_directory(from, to, &arena.entry(index, to)?);
            }
            self.copy_directory(from, to)
        } else if mask & (ADD_LEAF | CHANGED) != 0 {
            let metadata = fs::symlink_metadata(from).map_err(|error| Failure::io("inspect source leaf", from, error))?;
            self.copy_leaf_with(from, to, mask & REMOVE_DIR != 0, &metadata, None, &arena.entry(index, to)?)
        } else {
            self.remove(to)
        }
    }

    #[cfg(test)]
    fn copy_directory(&self, from: &Path, to: &Path) -> Result<Progress, Failure> {
        let metadata = paths::directory(from)?;
        self.copy_directory_with(from, to, &metadata)
    }

    #[cfg(test)]
    fn copy_directory_with(&self, from: &Path, to: &Path, metadata: &Metadata) -> Result<Progress, Failure> {
        self.copy_directory_selected(from, to, self.source_relative(from), metadata, true)
    }

    #[cfg(test)]
    fn staged_directory(&self, from: &Path, to: &Path, staged: &Path) -> Result<Progress, Failure> {
        let result = self.copy_directory(from, staged);
        match result {
            Ok(progress) if !self.stopped.load(Ordering::Acquire) => {
                let removed = self.remove_present(to);
                match removed {
                    Ok(mut removed) => match fs::rename(staged, to) {
                        Ok(()) => {
                            removed += progress;
                            Ok(removed)
                        }
                        Err(error) => {
                            let mut failure = Failure::io("install directory", to, error);
                            failure.completed = removed;
                            Err(failure)
                        }
                    },
                    Err(error) => Err(error),
                }
            }
            // Another sibling failed; discard private work without installation.
            Ok(_) => Ok(Progress::default()),
            Err(mut failure) => {
                failure.completed = Progress::default();
                Err(failure)
            }
        }
    }

    fn copy_leaf_with(
        &self,
        from: &Path,
        to: &Path,
        remove_directory: bool,
        metadata: &Metadata,
        link: Option<&Path>,
        staged: &Path,
    ) -> Result<Progress, Failure> {
        let read_link;
        let link = if metadata.is_symlink() && link.is_none() {
            read_link = fs::read_link(from).map_err(|error| Failure::io("read symlink", from, error))?;
            Some(read_link.as_path())
        } else {
            link
        };
        let kind = metadata_kind(from, metadata)?;
        if kind == Kind::Directory {
            return Err(Failure::invalid("copy leaf", from, "expected a regular file or symlink"));
        }
        let prepared = if kind == Kind::Symlink {
            platform::symlink(link.expect("observed symlink text"), staged, metadata).map_err(|error| Failure::io("prepare symlink", to, error))?;
            PreparedLeaf {
                progress: Progress {
                    symlinks_created: 1,
                    ..Progress::default()
                },
                _source: None,
                _output: None,
            }
        } else {
            self.prepare_file(from, staged, to, metadata)?
        };
        let removed = if remove_directory { self.remove_present(to)? } else { Progress::default() };
        if let Err(error) = fs::rename(staged, to) {
            let mut failure = Failure::io("install leaf", to, error);
            failure.completed = removed;
            return Err(failure);
        }
        let mut progress = prepared.progress;
        progress += removed;
        Ok(progress)
    }

    fn prepare_file(&self, from: &Path, staged: &Path, to: &Path, metadata: &Metadata) -> Result<PreparedLeaf, Failure> {
        let (copied, source, output) = match self.policy {
            policy if policy == CopyPolicy::Copy || cfg!(target_os = "linux") => {
                let operation = if policy == CopyPolicy::Copy { "copy file" } else { "prepare reflink" };
                let mut source = File::open(from).map_err(|error| Failure::io(operation, from, error))?;
                let mut output = OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create_new(true)
                    .open(staged)
                    .map_err(|error| Failure::io("prepare file", to, error))?;
                let copied = if policy == CopyPolicy::Copy {
                    Some(io::copy(&mut source, &mut output).map_err(|error| Failure::io(operation, from, error))?)
                } else {
                    #[cfg(target_os = "linux")]
                    {
                        match rustix::fs::ioctl_ficlone(&output, &source).map_err(io::Error::from) {
                            Ok(()) => None,
                            Err(error)
                                if policy == CopyPolicy::ReflinkOrCopy
                                    && !matches!(
                                        error.kind(),
                                        io::ErrorKind::NotFound | io::ErrorKind::PermissionDenied | io::ErrorKind::AlreadyExists
                                    ) =>
                            {
                                // Whole-file cloning does not consume either
                                // cursor. Discard any output before fallback;
                                // io::copy retains std's kernel offload/EOF rules.
                                output.set_len(0).map_err(|error| Failure::io(operation, to, error))?;
                                Some(io::copy(&mut source, &mut output).map_err(|error| Failure::io(operation, from, error))?)
                            }
                            Err(error) => return Err(Failure::io(operation, to, error)),
                        }
                    }
                    #[cfg(not(target_os = "linux"))]
                    unreachable!()
                };
                preserve_file(&output, metadata, to)?;
                (copied, Some(source), output)
            }
            policy => {
                let copied = match policy {
                    CopyPolicy::ReflinkRequired => reflink_copy::reflink(from, staged).map(|()| None),
                    CopyPolicy::ReflinkOrCopy => reflink_copy::reflink_or_copy(from, staged),
                    CopyPolicy::Copy => unreachable!(),
                }
                .map_err(|error| Failure::io("prepare reflink", to, error))?;
                // The crate finalizes source permissions before returning. This
                // inode is private/owned: relax it BEFORE opening, including 000
                // files, then retain that descriptor through final metadata.
                fs::set_permissions(staged, platform::writable(metadata.permissions(), false))
                    .map_err(|error| Failure::io("open prepared file permissions", to, error))?;
                let output = OpenOptions::new()
                    .read(true)
                    .write(true)
                    .open(staged)
                    .map_err(|error| Failure::io("open prepared file", to, error))?;
                preserve_file(&output, metadata, to)?;
                (copied, None, output)
            }
        };
        let progress = match copied {
            Some(bytes) => Progress {
                files_copied: 1,
                bytes_copied: bytes,
                ..Progress::default()
            },
            None => Progress {
                files_reflinked: 1,
                bytes_reflinked: metadata.len(),
                ..Progress::default()
            },
        };
        Ok(PreparedLeaf {
            progress,
            _source: source,
            _output: Some(output),
        })
    }

    fn remove(&self, path: &Path) -> Result<Progress, Failure> {
        // Private owned cleanup only. This is deliberately not user policy:
        // failed preparations may need permission repair to be enumerable.
        let metadata = fs::symlink_metadata(path).map_err(|error| Failure::io("inspect removal", path, error))?;
        if !metadata.is_dir() {
            return self.remove_leaf(path, &metadata);
        }
        relax(path)?;
        let mut progress = Progress::default();
        for name in children(path)? {
            progress += self.remove(&path.join(name))?;
        }
        fs::remove_dir(path).map_err(|error| Failure::io("remove owned directory", path, error))?;
        progress.directories_removed += 1;
        Ok(progress)
    }

    fn remove_present(&self, path: &Path) -> Result<Progress, Failure> {
        let metadata = fs::symlink_metadata(path).map_err(|error| Failure::io("inspect removal", path, error))?;
        if !metadata.is_dir() {
            return self.remove_leaf(path, &metadata);
        }
        let parent = Writable::new(path.parent().expect("removal parent"));
        let result = self
            .reconcile_directory(None, path, self.target_relative(path), true, &parent)
            .and_then(|(progress, retained)| {
                if retained {
                    let mut failure = Failure::invalid("replace directory", path, "target subtree retained by policy");
                    failure.completed = progress;
                    Err(failure)
                } else {
                    Ok(progress)
                }
            });
        parent.finish(result)
    }

    /// Local read-only precheck for directory -> leaf. Root presence grants
    /// replacement authority for its own name, not for target-only descendants.
    fn removable(&self, path: &Path, relative: &Path, root: bool) -> Result<(), Failure> {
        let _ = root; // The coupled parent action already authorizes this name.
        let plan = self.plan(relative, None, Some(path))?;
        if let Some(excluded) = plan.first_excluded() {
            return Err(Failure::invalid(
                "replace directory",
                &self.target.join(excluded),
                "target descendant preserved before observation",
            ));
        }
        for entry in plan.operations().entries() {
            match entry.action().expect("finalized entry has an action") {
                Action::RemoveLeaf => {}
                Action::Directory {
                    container: Container::Remove,
                    descend: true,
                } => self.removable(&path.join(entry.name()), entry.relative_path(), false)?,
                _ => {
                    return Err(Failure::invalid(
                        "replace directory",
                        &self.target.join(entry.relative_path()),
                        "target descendant retained by visitor",
                    ));
                }
            }
        }
        Ok(())
    }
}

fn children(path: &Path) -> Result<Vec<std::ffi::OsString>, Failure> {
    let mut names = fs::read_dir(path)
        .map_err(|error| Failure::io("read directory", path, error))?
        .map(|entry| {
            entry
                .map(|entry| entry.file_name())
                .map_err(|error| Failure::io("read directory entry", path, error))
        })
        .collect::<Result<Vec<_>, _>>()?;
    names.sort_unstable_by(|a, b| a.as_encoded_bytes().cmp(b.as_encoded_bytes()));
    Ok(names)
}

/// Explicitly delete a real container's participating contents, not the container.
///
/// [`Scope`] excludes both a selected name and its descendants **before** entry
/// metadata, readlink, checks, or descent. Preserved children retain their real
/// routing ancestors. The same recursive remover is used by [`Applier::run`];
/// this API grants deletion authority explicitly, without a missing donor or
/// fabricated diff. There are no built-in repository/local-name rules.
///
/// The root and its ancestors must be real directories and remain stable. Checks
/// accept/refuse observed entries, including the bound root; root type validation
/// still runs when scope preserves the empty relative path. A retained-only run
/// makes no namespace, staging, or chmod writes. Earlier allowed removals can
/// remain on failure; [`Failure`] reports completed work and restoration issues,
/// not rollback. Inspection itself never relaxes caller-directory permissions.
///
/// ```no_run
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use diff_dirs_tree_inspect::{NoCheck, Visit};
/// let scope = |p: &std::path::Path| if p == std::path::Path::new("local") {
///     Visit::PreserveTree
/// } else { Visit::Reconcile };
/// let report = diff_dirs_transfer_sync::clear("target", scope, NoCheck)?;
/// assert!(report.policy.satisfied); // not a claim that target is empty
/// # Ok(()) }
/// ```
pub fn clear<S: Scope, C: Check>(target: impl AsRef<Path>, scope: S, check: C) -> Result<Report, Failure> {
    clear_with_visitor(target, diff_dirs_tree_inspect::CheckedScope::new(check, scope))
}

/// Clear a real container's contents through the same finalized visitor batches
/// as mirror, retaining the bound container. Early names have TargetOnly facts;
/// late may keep a container while pruning selected descendants, or preserve its
/// whole tree. No missing source, synthetic observations, or fake diff is used.
pub fn clear_with_visitor<V: Visitor>(target: impl AsRef<Path>, visitor: V) -> Result<Report, Failure> {
    let target = paths::root(target.as_ref(), false)?;
    let context = Context::checked(CopyPolicy::Copy, false, &target, &target, &NoCheck, &NoScope, &RemoveAll, &visitor);
    paths::directory(&target)?;
    let root = diff_dirs_tree_inspect::inspect_single_root(&target, Side::Target, Purpose::Remove, &visitor).map_err(Failure::checked)?;
    context.excluded.fetch_add(root.excluded(), Ordering::Relaxed);
    context.retained.fetch_add(root.retained(), Ordering::Relaxed);
    let writable = Writable::new(target.parent().expect("bound parent"));
    let result = if root.operations().entries().get(0).expect("root entry").descends() {
        context
            .reconcile_directory(None, &target, Path::new(""), false, &writable)
            .map(|(done, _)| done)
    } else {
        Ok(Progress::default())
    };
    context.finish(result)
}

/// Create only the caller-listed regular-file/symlink leaves, using the shared
/// staged copy/clone/link, permissions, and exact-file-mtime installer.
///
/// Both roots must be existing real directories. No unlisted source directories
/// are enumerated. Only missing real target parents are created; any existing
/// destination leaf, including a dangling symlink, is refused rather than
/// replaced. Every leaf uses freshly observed source metadata/link text; cached
/// routing contexts are not a caller-supplied leaf validation certificate.
///
/// # Ordering, batching, and failure
///
/// The known input names are collected (O(listed names), not a discovered payload
/// map), validated, normalized by normal path components, and duplicate-checked
/// before writes. Duplicate aliases such as `p/a` and `p/./a` are rejected too.
/// Invalid/duplicate names yield zero progress. Parent groups are sorted by native
/// encoded **component** order, then basenames by encoded bytes; this is not input
/// order or flat full-path byte order. Runtime errors stop that deterministic
/// installation prefix. Missing source, existing destination, and directory-as-
/// leaf errors identify `seed source missing`, `seed destination exists`, or
/// `seed source type`; parent errors keep their native path and cause.
///
/// Each group shares one lazy parent-local arena and permission lease, reserving
/// all of its listed basenames against arena collisions. A depth-bounded verified
/// ancestor stack reuses real source/target routing directories across nested
/// groups. Source leaves are still observed separately. Caller quiescence and
/// nonhostile aliasing are required: cached directories are not a security or
/// concurrent-replacement boundary. Existing roots/ancestors and source leaves
/// pass [`Check`]; created output parents follow recursive-copy semantics.
///
/// Parent creation and earlier installed leaves may remain on error, including
/// a strict clone failure. [`Failure::completed`] counts that real work; failed
/// private preparation is cleaned, permission restoration/cleanup failures remain
/// visible. [`CopyPolicy::ReflinkRequired`] never falls back. Empty input makes no
/// writes. List processing is sequential, including inside a caller's pool.
///
/// ```no_run
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use diff_dirs_transfer_sync::{seed_leaves, CopyPolicy};
/// let done = seed_leaves("source", "target", ["src/a.rs", "src/b.rs"],
///                        CopyPolicy::ReflinkRequired, diff_dirs_tree_inspect::NoCheck)?;
/// assert_eq!(done.files_reflinked, 2);
/// # Ok(()) }
/// ```
pub fn seed_leaves<I, P, C>(original: impl AsRef<Path>, target: impl AsRef<Path>, leaves: I, policy: CopyPolicy, check: C) -> Result<Progress, Failure>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
    C: Check,
{
    let binding = Applier::new_checked(original, target, policy, check)?;
    let context = Context::checked(
        policy,
        false,
        binding.original(),
        binding.target(),
        binding.check(),
        &NoScope,
        &RemoveAll,
        &NoVisitor,
    );
    // Only caller-declared names are collected, never discovered filesystem
    // entries or an operation/metadata catalog. Component ordering keeps routing
    // ancestors contiguous, so a depth-bounded verified prefix suffices.
    let mut leaves = leaves
        .into_iter()
        .map(|leaf| {
            let relative = leaf.as_ref();
            if relative.as_os_str().is_empty() || relative.components().any(|part| !matches!(part, std::path::Component::Normal(_))) {
                return Err(Failure::invalid(
                    "seed relative path",
                    relative,
                    "expected a nonempty normal relative leaf path",
                ));
            }
            Ok(relative.components().collect::<PathBuf>())
        })
        .collect::<Result<Vec<_>, _>>()?;
    fn parent_names(path: &Path) -> impl Iterator<Item = &[u8]> {
        path.parent()
            .expect("validated leaf parent")
            .components()
            .map(|part| part.as_os_str().as_encoded_bytes())
    }
    leaves.sort_unstable_by(|a, b| {
        parent_names(a)
            .cmp(parent_names(b))
            .then_with(|| a.file_name().unwrap().as_encoded_bytes().cmp(b.file_name().unwrap().as_encoded_bytes()))
    });
    for pair in leaves.windows(2) {
        if pair[0] == pair[1] {
            return Err(Failure::invalid("seed duplicate path", &pair[1], "listed leaf occurs more than once"));
        }
    }
    let source = paths::directory(binding.original())?;
    context.observe(binding.original(), Side::Original, Purpose::Copy, &source, None)?;
    let target = paths::directory(binding.target())?;
    context.observe(binding.target(), Side::Target, Purpose::Copy, &target, None)?;
    let mut parents = vec![SeedParent {
        relative: PathBuf::new(),
        from: binding.original().to_path_buf(),
        to: binding.target().to_path_buf(),
        permissions: target.permissions(),
    }];
    let mut completed = Progress::default();
    let result = (|| {
        let mut remaining = leaves.as_slice();
        while let Some(first) = remaining.first() {
            let relative = first.parent().unwrap();
            let count = remaining.iter().take_while(|leaf| leaf.parent() == Some(relative)).count();
            let (group, rest) = remaining.split_at(count);
            let shared = parents
                .last()
                .unwrap()
                .relative
                .components()
                .zip(relative.components())
                .take_while(|(a, b)| a == b)
                .count();
            parents.truncate(shared + 1);
            for component in relative.components().skip(shared) {
                let parent = parents.last().unwrap();
                let from = parent.from.join(component);
                let to = parent.to.join(component);
                let metadata = paths::directory(&from)?;
                context.observe(&from, Side::Original, Purpose::Copy, &metadata, None)?;
                let metadata = match fs::symlink_metadata(&to) {
                    Ok(metadata) => {
                        if !metadata.is_dir() {
                            return Err(Failure::invalid("seed target ancestor", &to, "expected real directory"));
                        }
                        context.observe(&to, Side::Target, Purpose::Copy, &metadata, None)?;
                        metadata
                    }
                    Err(error) if error.kind() == io::ErrorKind::NotFound => {
                        let writable = Writable::verified(&parent.to, parent.permissions.clone());
                        let created = (|| {
                            writable.ensure()?;
                            fs::create_dir(&to).map_err(|error| Failure::io("seed create parent", &to, error))?;
                            Ok(Progress {
                                directories_created: 1,
                                ..Progress::default()
                            })
                        })();
                        completed += writable.finish(created)?;
                        paths::directory(&to)?
                    }
                    Err(error) => return Err(Failure::io("seed target ancestor", &to, error)),
                };
                parents.push(SeedParent {
                    relative: parent.relative.join(component),
                    from,
                    to,
                    permissions: metadata.permissions(),
                });
            }
            let parent = parents.last().unwrap();
            completed += seed_batch(&context, parent, group)?;
            remaining = rest;
        }
        Ok(())
    })();
    match result {
        Ok(()) => Ok(completed),
        Err(mut failure) => {
            failure.completed += completed;
            Err(failure)
        }
    }
}

/// One active, real source/target routing prefix under caller quiescence. Only
/// paths and target write-lease permissions survive a leaf installation, never
/// source leaf metadata or a processed-file inventory. Component-ordered groups
/// allow truncating this stack on ascent without retaining visited directories.
struct SeedParent {
    relative: PathBuf,
    from: PathBuf,
    to: PathBuf,
    permissions: Permissions,
}

/// A single parent owns the arena until every sequential leaf has completed or
/// failed. Merge installed-prefix progress before arena finalization so cleanup
/// failure cannot hide installations. Only this group's names are reserved.
fn seed_batch<C: Check>(context: &Context<'_, C>, parent: &SeedParent, leaves: &[PathBuf]) -> Result<Progress, Failure> {
    let mut arena = Staging::new(&parent.to, leaves.iter().map(|leaf| leaf.file_name().unwrap()));
    arena.writable = Writable::verified(&parent.to, parent.permissions.clone());
    let mut completed = Progress::default();
    let result = (|| {
        for (index, relative) in leaves.iter().enumerate() {
            let name = relative.file_name().unwrap();
            let from = parent.from.join(name);
            let to = parent.to.join(name);
            let source = inspect(context, &from, Side::Original, Purpose::Copy)?.ok_or_else(|| {
                Failure::io(
                    "seed source missing",
                    &from,
                    io::Error::new(io::ErrorKind::NotFound, "listed source leaf is absent"),
                )
            })?;
            if source.metadata.is_dir() {
                return Err(Failure::invalid("seed source type", &from, "listed source must be a regular file or symlink"));
            }
            if paths::exists(&to)? {
                return Err(Failure::io(
                    "seed destination exists",
                    &to,
                    io::Error::new(io::ErrorKind::AlreadyExists, "listed destination must be absent"),
                ));
            }
            completed += context.copy_leaf_with(&from, &to, false, &source.metadata, source.link.as_deref(), &arena.entry(index, &to)?)?;
        }
        Ok(())
    })();
    let result = match result {
        Ok(()) => Ok(completed),
        Err(mut failure) => {
            failure.completed += completed;
            Err(failure)
        }
    };
    arena.finish(result, context)
}

fn preserve_file(output: &File, metadata: &Metadata, to: &Path) -> Result<(), Failure> {
    let modified = metadata.modified().map_err(|error| Failure::io("read source mtime", to, error))?;
    output.set_modified(modified).map_err(|error| Failure::io("set file mtime", to, error))?;
    let actual = output
        .metadata()
        .and_then(|metadata| metadata.modified())
        .map_err(|error| Failure::io("verify file mtime", to, error))?;
    if modified != actual {
        return Err(Failure::io(
            "verify file mtime",
            to,
            io::Error::new(io::ErrorKind::Unsupported, "target cannot represent exact source modification time"),
        ));
    }
    output
        .set_permissions(metadata.permissions())
        .map_err(|error| Failure::io("set file permissions", to, error))
}

/// One lazy parent-local arena per joined sibling batch. Slots are batch indices,
/// so independent jobs never share a prepared inode or need a per-leaf lock.
struct Staging<'a> {
    parent: &'a Path,
    prefix: String,
    temporary: Mutex<Option<tempfile::TempDir>>,
    writable: Writable<'a>,
}

impl<'a> Staging<'a> {
    fn new<'n>(parent: &'a Path, names: impl Iterator<Item = &'n OsStr>) -> Self {
        // Exclude planned names BEFORE mkdir: a random basename can otherwise
        // collide with a concurrently added/transitioned source entry. Existing
        // unmentioned names are protected by tempfile's exclusive creation.
        let reserved: HashSet<usize> = names
            .filter_map(|name| {
                let bytes = name.as_encoded_bytes();
                let base = b".diff-dirs-sync-";
                // Also protect case-insensitive target filesystems. The chosen
                // prefix consists only of ASCII plus decimal digits.
                if !bytes.get(..base.len())?.eq_ignore_ascii_case(base) {
                    return None;
                }
                let suffix = &bytes[base.len()..];
                let end = suffix.iter().position(|byte| *byte == b'-')?;
                std::str::from_utf8(&suffix[..end]).ok()?.parse().ok()
            })
            .collect();
        let index = (0..).find(|index| !reserved.contains(index)).unwrap();
        Self {
            parent,
            prefix: format!(".diff-dirs-sync-{index}-"),
            temporary: Mutex::new(None),
            writable: Writable::new(parent),
        }
    }

    fn entry(&self, index: usize, to: &Path) -> Result<PathBuf, Failure> {
        let mut temporary = self.temporary.lock().unwrap();
        if temporary.is_none() {
            self.writable.ensure()?;
            let mut builder = tempfile::Builder::new();
            builder.prefix(&self.prefix);
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                builder.permissions(Permissions::from_mode(0o700));
            }
            *temporary = Some(
                builder
                    .tempdir_in(self.parent)
                    .map_err(|error| Failure::io("create private staging directory", to, error))?,
            );
            // Retain ownership before a fallible permission repair so batch
            // cleanup still runs under a restrictive process umask.
            let path = temporary.as_ref().unwrap().path();
            relax(path)?;
        }
        Ok(temporary.as_ref().unwrap().path().join(index.to_string()))
    }

    fn finish<C: Check, S: Scope, A: Absence, V: Visitor>(
        self,
        result: Result<Progress, Failure>,
        context: &Context<'_, C, S, A, V>,
    ) -> Result<Progress, Failure> {
        let (result, writable) = self.finish_preparation(result, context);
        writable.finish(result)
    }

    fn finish_preparation<C: Check, S: Scope, A: Absence, V: Visitor>(
        self,
        result: Result<Progress, Failure>,
        context: &Context<'_, C, S, A, V>,
    ) -> (Result<Progress, Failure>, Writable<'a>) {
        let result = match self.temporary.into_inner().unwrap() {
            Some(temporary) => {
                // A stopped sibling can leave a successfully prepared directory
                // deliberately uninstalled. Only an unstopped successful batch
                // proves that every allocated slot was renamed out.
                let empty = result.is_ok() && !context.stopped.load(Ordering::Acquire);
                cleanup(temporary, result, empty)
            }
            None => result,
        };
        (result, self.writable)
    }
}

/// One deferred permission lease per joined sibling batch. Merely inspecting a
/// retained subtree never chmods it. The mutex is never held across caller code.
struct Writable<'a> {
    path: &'a Path,
    state: Mutex<Option<(Permissions, bool)>>,
    verified: Option<Permissions>,
}

impl<'a> Writable<'a> {
    fn new(path: &'a Path) -> Self {
        Self {
            path,
            state: Mutex::new(None),
            verified: None,
        }
    }
    fn ensure(&self) -> Result<(), Failure> {
        let mut state = self.state.lock().unwrap();
        if state.is_none() {
            *state = Some(match &self.verified {
                Some(permissions) => relax_with(self.path, permissions.clone())?,
                None => relax(self.path)?,
            });
        }
        Ok(())
    }

    /// Reuse permissions from an already observed real routing directory, under
    /// the same stable-tree contract. Avoid a second parent stat merely to open
    /// its write lease; this is never evidence about a source/destination leaf.
    fn verified(path: &'a Path, permissions: Permissions) -> Self {
        Self {
            path,
            state: Mutex::new(None),
            verified: Some(permissions),
        }
    }

    fn finish(self, result: Result<Progress, Failure>) -> Result<Progress, Failure> {
        match self.state.into_inner().unwrap() {
            Some((permissions, relaxed)) => finish_permissions(result, self.path, permissions, relaxed, true),
            None => result,
        }
    }

    fn removed(self, mut result: Result<Progress, Failure>) -> Result<Progress, Failure> {
        if self.state.into_inner().unwrap().is_some_and(|(_, relaxed)| relaxed) {
            add_progress(
                &mut result,
                Progress {
                    directory_permissions_set: 1,
                    ..Progress::default()
                },
            );
        }
        result
    }
}

/// Clean only the temporary directory we created, without following links or
/// chmodding hardlinked leaves. Cleanup failures are retained, never concealed.
fn cleanup(temporary: tempfile::TempDir, mut result: Result<Progress, Failure>, empty: bool) -> Result<Progress, Failure> {
    let path = temporary.keep();
    let cleaned = if empty {
        fs::remove_dir(&path)
            .map(|()| Progress::default())
            .map_err(|error| Failure::io("clean private staging directory", &path, error))
    } else {
        Context::new(CopyPolicy::Copy, false).remove(&path)
    };
    if let Err(mut cleanup) = cleaned {
        cleanup.primary.operation = "clean private staging directory";
        cleanup.completed = Progress::default();
        result = merge([result, Err(cleanup)]);
    }
    result
}

fn relax(path: &Path) -> Result<(Permissions, bool), Failure> {
    let permissions = paths::directory(path)?.permissions();
    relax_with(path, permissions)
}

fn relax_with(path: &Path, permissions: Permissions) -> Result<(Permissions, bool), Failure> {
    let writable = platform::writable(permissions.clone(), true);
    let changed = !platform::same_permissions(&permissions, &writable);
    if changed {
        fs::set_permissions(path, writable).map_err(|error| Failure::io("relax target directory permissions", path, error))?;
    }
    Ok((permissions, changed))
}

fn finish_permissions(
    mut result: Result<Progress, Failure>,
    path: &Path,
    permissions: Permissions,
    relaxed: bool,
    restore_only: bool,
) -> Result<Progress, Failure> {
    if relaxed {
        add_progress(
            &mut result,
            Progress {
                directory_permissions_set: 1,
                ..Progress::default()
            },
        );
    }
    if !restore_only || relaxed {
        // Called only after all children have joined, even when a child failed.
        match fs::set_permissions(path, permissions) {
            Ok(()) => add_progress(
                &mut result,
                Progress {
                    directory_permissions_set: 1,
                    ..Progress::default()
                },
            ),
            Err(error) => result = merge([result, Err(Failure::io("finalize directory permissions", path, error))]),
        }
    }
    result
}

fn add_progress(result: &mut Result<Progress, Failure>, progress: Progress) {
    match result {
        Ok(completed) => *completed += progress,
        Err(failure) => failure.completed += progress,
    }
}

#[cfg(all(test, unix))]
mod staging_tests {
    use super::*;
    use std::os::unix::fs::{MetadataExt, PermissionsExt};

    fn scratch() -> tempfile::TempDir {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.test-agent/sync-tests");
        fs::create_dir_all(&root).unwrap();
        tempfile::tempdir_in(root).unwrap()
    }

    #[test]
    fn cleanup_failure_retains_installations_and_primary_error() {
        let owned = scratch();
        let source = owned.path().join("source");
        let missing = owned.path().join("absent-source");
        fs::write(&source, b"actual installed payload").unwrap();
        for primary in [false, true] {
            let target = owned.path().join(if primary { "with-primary" } else { "cleanup-only" });
            fs::create_dir(&target).unwrap();
            let names = if primary { &['a', 'b'][..] } else { &['a'][..] };
            let arena = Staging::new(&target, std::iter::empty());
            let context = Context::new(CopyPolicy::Copy, false);
            let result = context.batch(names, |index, name| {
                context.named(
                    if index == 0 { &source } else { &missing },
                    &target.join(name.to_string()),
                    ADD_LEAF,
                    &arena,
                    index,
                )
            });
            let path = arena.temporary.lock().unwrap().as_ref().unwrap().path().to_owned();
            // Fail arena cleanup AFTER a real installation and an optional
            // started sibling error, testing both fast and recursive finalizers.
            fs::set_permissions(&target, Permissions::from_mode(0o500)).unwrap();
            let progress = Progress {
                files_copied: 1,
                bytes_copied: b"actual installed payload".len() as u64,
                ..Progress::default()
            };
            let result = arena.finish(result, &context);
            fs::set_permissions(&target, Permissions::from_mode(0o700)).unwrap();
            assert_eq!(fs::read(target.join("a")).unwrap(), b"actual installed payload");
            if let Err(failure) = result {
                assert_eq!(failure.completed, progress);
                if primary {
                    assert_eq!(failure.primary.operation, "inspect source leaf");
                    if path.exists() {
                        assert_eq!(failure.secondary.len(), 1);
                        assert_eq!(failure.secondary[0].operation, "clean private staging directory");
                    } else {
                        assert!(File::open("/root").is_ok(), "unprivileged cleanup fault did not fire");
                    }
                } else {
                    assert_eq!(failure.primary.operation, "clean private staging directory");
                }
                if path.exists() {
                    fs::remove_dir(path).unwrap();
                }
            } else {
                assert!(File::open("/root").is_ok(), "unprivileged cleanup fault did not fire");
            }
        }
    }

    #[test]
    fn failed_private_preparation_is_recursively_cleaned_without_progress() {
        let owned = scratch();
        let arena = Staging::new(owned.path(), std::iter::empty());
        let slot = arena.entry(0, &owned.path().join("logical-target")).unwrap();
        fs::create_dir(&slot).unwrap();
        fs::write(slot.join("readonly"), b"discarded private data").unwrap();
        fs::set_permissions(slot.join("readonly"), Permissions::from_mode(0o000)).unwrap();
        fs::set_permissions(&slot, Permissions::from_mode(0o000)).unwrap();
        let context = Context::new(CopyPolicy::Copy, false);
        context.stopped.store(true, Ordering::Release);
        let failure = arena.finish(Err(Failure::invalid("prepare fault", &slot, "fault")), &context).unwrap_err();
        assert_eq!(failure.primary.operation, "prepare fault");
        assert!(failure.secondary.is_empty());
        assert_eq!(failure.completed, Progress::default());
        assert_eq!(fs::read_dir(owned.path()).unwrap().count(), 0);
    }

    #[test]
    fn arena_excludes_planned_names_before_creation_and_slots_are_exclusive() {
        let owned = scratch();
        let names = [".diff-dirs-sync-0-source", ".DIFF-DIRS-SYNC-1-SOURCE", ".diff-dirs-sync-2-source"];
        let arena = Staging::new(owned.path(), names.iter().map(OsStr::new));
        for name in names {
            assert!(!name.to_ascii_lowercase().starts_with(&arena.prefix));
        }
        std::thread::scope(|scope| {
            for index in 0..8 {
                let arena = &arena;
                scope.spawn(move || {
                    let path = arena.entry(index, owned_path()).unwrap();
                    let output = OpenOptions::new().create_new(true).write(true).open(&path).unwrap();
                    assert!(OpenOptions::new().create_new(true).write(true).open(&path).is_err());
                    drop(output);
                    fs::remove_file(path).unwrap();
                });
            }
        });
        assert_eq!(fs::read_dir(owned.path()).unwrap().count(), 1);
        arena.finish(Ok(Progress::default()), &Context::new(CopyPolicy::Copy, true)).unwrap();
        assert_eq!(fs::read_dir(owned.path()).unwrap().count(), 0);
        fn owned_path() -> &'static Path {
            Path::new("logical-target")
        }
    }

    #[test]
    fn retained_output_finalizes_exact_mtime_and_mode000_before_installation() {
        let owned = scratch();
        let source = owned.path().join("source");
        fs::write(&source, b"nonempty source").unwrap();
        let source_file = OpenOptions::new().read(true).write(true).open(&source).unwrap();
        source_file
            .set_modified(std::time::SystemTime::UNIX_EPOCH - std::time::Duration::new(1, 123_456_789))
            .unwrap();
        source_file.set_permissions(Permissions::from_mode(0o000)).unwrap();
        let metadata = source_file.metadata().unwrap();
        let staged = owned.path().join("staged");
        let output = OpenOptions::new().create_new(true).read(true).write(true).open(&staged).unwrap();
        preserve_file(&output, &metadata, &staged).unwrap();
        fs::rename(&staged, owned.path().join("installed")).unwrap();
        assert_eq!(output.metadata().unwrap().mode() & 0o777, 0);
        assert_eq!(output.metadata().unwrap().modified().unwrap(), metadata.modified().unwrap());
        output.set_modified(metadata.modified().unwrap()).unwrap(); // still usable
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn descriptor_preparation_does_not_clobber_existing_slots_or_fallback_on_missing_source() {
        let owned = scratch();
        let source = owned.path().join("source");
        fs::write(&source, vec![b'x'; 4097]).unwrap();
        let metadata = fs::metadata(&source).unwrap();
        for policy in [CopyPolicy::Copy, CopyPolicy::ReflinkRequired, CopyPolicy::ReflinkOrCopy] {
            let context = Context::new(policy, false);
            let staged = owned.path().join("existing");
            fs::write(&staged, b"owned sentinel").unwrap();
            let failure = context.prepare_file(&source, &staged, &staged, &metadata).err().unwrap();
            assert_eq!(failure.primary.cause.downcast_ref::<io::Error>().unwrap().kind(), io::ErrorKind::AlreadyExists);
            assert_eq!(fs::read(&staged).unwrap(), b"owned sentinel");
            fs::remove_file(&staged).unwrap();
            let failure = context.prepare_file(&owned.path().join("missing"), &staged, &staged, &metadata).err().unwrap();
            assert_eq!(failure.primary.cause.downcast_ref::<io::Error>().unwrap().kind(), io::ErrorKind::NotFound);
            assert!(!staged.exists());
            assert_eq!(failure.completed, Progress::default());
        }
    }

    #[cfg(all(target_os = "linux", feature = "parallel"))]
    #[test]
    fn shared_arena_joins_real_clone_failure_and_successful_sibling() {
        let owned = scratch();
        let external = tempfile::tempdir_in("/tmp/opencode").unwrap();
        let local = owned.path().join("local");
        fs::write(&local, vec![b'x'; 4097]).unwrap();
        if let Err(error) = reflink_copy::reflink(&local, owned.path().join("clone-probe")) {
            assert!(std::env::var_os("DIFF_DIRS_SYNC_REQUIRE_REFLINK").is_none(), "real reflink required: {error}");
            eprintln!("shared-arena real-clone test unavailable: {error}");
            return;
        }
        let remote = external.path().join("remote");
        fs::write(&remote, b"cross-filesystem failure").unwrap();
        let target = owned.path().join("target");
        fs::create_dir(&target).unwrap();
        fs::write(target.join("a-fail"), b"old survives").unwrap();
        let names = ["a-fail", "z-good"];
        let arena = Staging::new(&target, names.iter().map(OsStr::new));
        let context = Context::new(CopyPolicy::ReflinkRequired, true);
        let barrier = std::sync::Barrier::new(2);
        let pool = rayon::ThreadPoolBuilder::new().num_threads(2).build().unwrap();
        let result = pool.install(|| {
            context.batch(&names, |index, name| {
                barrier.wait(); // both operations started before either can fail
                context.named(
                    if index == 0 { &remote } else { &local },
                    &target.join(name),
                    if index == 0 { CHANGED } else { ADD_LEAF },
                    &arena,
                    index,
                )
            })
        });
        let failure = arena.finish(result, &context).unwrap_err();
        assert_eq!(failure.primary.operation, "prepare reflink");
        assert_eq!(failure.primary.path, target.join("a-fail"));
        assert!(failure.secondary.is_empty());
        assert_eq!(failure.completed.files_reflinked, 1);
        assert_eq!(failure.completed.bytes_reflinked, 4097);
        assert_eq!(fs::read(target.join("a-fail")).unwrap(), b"old survives");
        assert_eq!(fs::read(target.join("z-good")).unwrap(), fs::read(local).unwrap());
        assert_eq!(fs::read_dir(target).unwrap().count(), 2);
    }
}

#[cfg(all(test, feature = "parallel"))]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::{Arc, Condvar, Mutex};
    use std::time::Duration;

    static HOOK: Mutex<Option<Arc<Hook>>> = Mutex::new(None);

    #[derive(Default)]
    struct State {
        workers: Vec<String>,
        deleted: bool,
        release: bool,
    }

    struct Hook {
        parent: PathBuf,
        delete: Option<PathBuf>,
        state: Mutex<State>,
        changed: Condvar,
    }

    pub(super) fn observe(path: &Path) {
        let hook = HOOK.lock().unwrap().clone();
        let Some(hook) = hook.filter(|hook| path.parent() == Some(hook.parent.as_path())) else {
            return;
        };
        let mut state = hook.state.lock().unwrap();
        state.workers.push(std::thread::current().name().unwrap().to_string());
        hook.changed.notify_all();
        let (mut state, timeout) = hook
            .changed
            .wait_timeout_while(state, Duration::from_secs(5), |state| state.workers.len() < 2)
            .unwrap();
        assert!(!timeout.timed_out(), "no distinct concurrent apply worker");
        if let Some(delete) = &hook.delete {
            if path.file_name().unwrap() == "a-fail" {
                fs::remove_file(delete).unwrap();
                state.deleted = true;
                hook.changed.notify_all();
            } else {
                let (_state, timeout) = hook.changed.wait_timeout_while(state, Duration::from_secs(5), |state| !state.release).unwrap();
                assert!(!timeout.timed_out(), "test did not release started sibling");
            }
        }
    }

    // One test owns the global hook. Covers both returned-record siblings and
    // nested opaque-subtree work, not just pool installation/worker migration.
    #[test]
    fn actual_independent_apply_and_recursive_copy_workers_join_on_failure() {
        let scratch = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.test-agent/sync-tests");
        fs::create_dir_all(&scratch).unwrap();
        let fixture = tempfile::tempdir_in(fs::canonicalize(scratch).unwrap()).unwrap();
        let source = fixture.path().join("source");
        let target = fixture.path().join("target");
        fs::create_dir(&source).unwrap();
        fs::create_dir(&target).unwrap();
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(2)
            .thread_name(|i| format!("sync-owned-{i}"))
            .build()
            .unwrap();
        for nested in [false, true] {
            let input = if nested { source.join("opaque") } else { source.clone() };
            fs::create_dir_all(&input).unwrap();
            for name in ["a-fail", "b-work"] {
                fs::write(input.join(name), b"actual nonempty apply work").unwrap();
            }
            let hook = Arc::new(Hook {
                parent: if nested { target.join("opaque") } else { target.clone() },
                delete: None,
                state: Mutex::new(State::default()),
                changed: Condvar::new(),
            });
            *HOOK.lock().unwrap() = Some(hook.clone());
            crate::mirror_in_pool(
                &source,
                &target,
                CopyPolicy::Copy,
                &pool,
                diff_dirs_tree_compare::ParallelLimits::for_pool(&pool),
            )
            .unwrap();
            *HOOK.lock().unwrap() = None;
            let state = hook.state.lock().unwrap();
            assert_eq!(state.workers.len(), 2);
            assert_ne!(state.workers[0], state.workers[1]);
            assert!(state.workers.iter().all(|worker| worker.starts_with("sync-owned-")));
        }
        // Ensure both jobs entered before one loses its source; keep the other
        // in progress while the first fails. A returned failure must wait.
        for name in ["a-fail", "b-work"] {
            fs::write(source.join(name), b"changed larger source payload").unwrap();
        }
        let hook = Arc::new(Hook {
            parent: target.clone(),
            delete: Some(source.join("a-fail")),
            state: Mutex::new(State::default()),
            changed: Condvar::new(),
        });
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&target, Permissions::from_mode(0o555)).unwrap();
        }
        *HOOK.lock().unwrap() = Some(hook.clone());
        std::thread::scope(|scope| {
            let (sender, receiver) = std::sync::mpsc::channel();
            let (source, target, pool) = (&source, &target, &pool);
            scope.spawn(move || {
                let result = crate::mirror_in_pool(source, target, CopyPolicy::Copy, pool, diff_dirs_tree_compare::ParallelLimits::for_pool(pool));
                sender.send(result).unwrap();
            });
            let state = hook.state.lock().unwrap();
            let (mut state, timeout) = hook.changed.wait_timeout_while(state, Duration::from_secs(5), |state| !state.deleted).unwrap();
            assert!(!timeout.timed_out());
            assert!(
                receiver.recv_timeout(Duration::from_millis(50)).is_err(),
                "returned while a started sibling was blocked"
            );
            state.release = true;
            hook.changed.notify_all();
            drop(state);
            let failure = receiver.recv_timeout(Duration::from_secs(5)).unwrap().unwrap_err();
            assert!(failure.primary.path.ends_with("a-fail"));
            assert_eq!(failure.completed.files_copied, 1);
            assert_eq!(fs::read(target.join("b-work")).unwrap(), b"changed larger source payload");
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                assert_eq!(fs::metadata(target).unwrap().permissions().mode() & 0o777, 0o555);
                assert_eq!(failure.completed.directory_permissions_set, 2);
                fs::set_permissions(target, Permissions::from_mode(0o700)).unwrap();
            }
        });
        *HOOK.lock().unwrap() = None;
        crate::mirror(&source, &target, CopyPolicy::Copy).unwrap();
        let mut walk = diff_dirs_tree_compare::walk(&source, &target).unwrap();
        assert!(walk.next().is_none());
    }
}
