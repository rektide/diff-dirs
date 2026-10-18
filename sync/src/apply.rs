use std::collections::HashSet;
use std::ffi::OsStr;
use std::fs::{self, File, Metadata, OpenOptions, Permissions};
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use diff_dirs::{Check, Entry, NoCheck, NoScope, Purpose, Scope, Side};

use crate::model::merge;
use crate::paths::{self, ADD_DIR, ADD_LEAF, CHANGED, REMOVE_DIR, REMOVE_LEAF};
use crate::{Absence, Applier, CopyPolicy, Failure, Policy, Progress, RemoveAll, Report, platform};

impl<C: Check, S: Scope, A: Absence> Applier<C, S, A> {
    pub(crate) fn apply_with(&self, diff: &diff_dirs::DirDiff, parallel: bool) -> Result<Progress, Failure> {
        self.apply_reported(diff, parallel).map(|report| report.completed)
    }

    pub fn apply_report(&self, diff: &diff_dirs::DirDiff) -> Result<Report, Failure> {
        self.apply_reported(diff, false)
    }

    pub(crate) fn apply_reported(&self, diff: &diff_dirs::DirDiff, parallel: bool) -> Result<Report, Failure> {
        let mut names = paths::names(diff)?;
        let context = Context::checked(self.policy, parallel, &self.original, &self.target, &self.check, &self.scope, &self.absence);
        // Even fabricated public records cannot reopen a preserved ancestor.
        // Root identity/type is still validated before honoring root scope.
        let source = paths::directory(&self.original)?;
        context.observe(&self.original, Side::Original, Purpose::Copy, &source, None)?;
        let target = paths::directory(&self.target)?;
        context.observe(&self.target, Side::Target, Purpose::Remove, &target, None)?;
        if !context.included(&diff.dir) {
            return context.finish(Ok(Progress::default()));
        }
        names.retain(|(name, _)| context.included(&diff.dir.join(name)));
        let from = context.relative(&self.original, &diff.dir, Side::Original, Purpose::Copy)?;
        let to = context.relative(&self.target, &diff.dir, Side::Target, Purpose::Remove)?;
        // Validate actual immediate types before any chmod, staging or removal.
        let mut validated = Vec::with_capacity(names.len());
        for (name, mask) in &names {
            validated.push(validate_entry(&context, &from.join(name), &to.join(name), *mask)?);
        }
        if names.is_empty() {
            return context.finish(Ok(Progress::default()));
        }
        let arena = Staging::new(&to, names.iter().map(|(name, _)| name.as_os_str()));
        let result = context.batch(&names, |index, (name, mask)| {
            context.named_with(&from.join(name), &to.join(name), *mask, &arena, index, &validated[index])
        });
        let result = arena.finish(result, &context);
        context.finish(result)
    }

    pub(crate) fn initialize_with(&self, parallel: bool) -> Result<Progress, Failure> {
        self.initialize_reported(parallel).map(|report| report.completed)
    }

    pub(crate) fn initialize_reported(&self, parallel: bool) -> Result<Report, Failure> {
        let metadata = paths::directory(&self.original)?;
        let context = Context::checked(self.policy, parallel, &self.original, &self.target, &self.check, &self.scope, &self.absence);
        context.observe(&self.original, Side::Original, Purpose::Copy, &metadata, None)?;
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
        if !context.included(Path::new("")) {
            return context.finish(Ok(Progress::default()));
        }
        let (permissions, relaxed) = relax(parent)?;
        let result = context.copy_directory_with(&self.original, &self.target, &metadata);
        context.finish(finish_permissions(result, parent, permissions, relaxed, true))
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
        let context = Context::checked(self.policy, parallel, &self.original, &self.target, &self.check, &self.scope, &self.absence);
        let metadata = paths::directory(&self.original)?;
        context.observe(&self.original, Side::Original, Purpose::Copy, &metadata, None)?;
        let target = fs::symlink_metadata(&self.target).map_err(|error| Failure::io("inspect target root", &self.target, error))?;
        context.observe(&self.target, Side::Target, Purpose::Remove, &target, None)?;
        if !matches!(metadata_kind(&self.target, &target)?, Kind::File | Kind::Symlink) {
            return Err(Failure::invalid("replace root", &self.target, "expected existing target leaf"));
        }
        if !context.included(Path::new("")) {
            return context.finish(Ok(Progress::default()));
        }
        let parent = self.target.parent().expect("bound leaf root has parent");
        let (permissions, relaxed) = relax(parent)?;
        let arena = Staging::new(parent, std::iter::once(self.target.file_name().expect("leaf root has name")));
        let result = (|| {
            let staged = arena.entry(0, &self.target)?;
            context.staged_directory(&self.original, &self.target, &staged)
        })();
        let result = arena.finish(result, &context);
        context.finish(finish_permissions(result, parent, permissions, relaxed, true))
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Directory,
    File,
    Symlink,
}

fn inspect<C: Check, S: Scope, A: Absence>(context: &Context<'_, C, S, A>, path: &Path, side: Side, purpose: Purpose) -> Result<Option<Observed>, Failure> {
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

struct Validated {
    source: Option<Observed>,
    target: Option<Observed>,
}

fn validate_entry<C: Check, S: Scope, A: Absence>(context: &Context<'_, C, S, A>, from: &Path, to: &Path, mask: u8) -> Result<Validated, Failure> {
    let validated = Validated {
        source: inspect(context, from, Side::Original, Purpose::Copy)?,
        target: inspect(context, to, Side::Target, Purpose::Remove)?,
    };
    let source = validated.source.as_ref().map(|entry| metadata_kind(from, &entry.metadata)).transpose()?;
    let target = validated.target.as_ref().map(|entry| metadata_kind(to, &entry.metadata)).transpose()?;
    let leaf = |kind| matches!(kind, Some(Kind::File | Kind::Symlink));
    let source_valid = if mask & ADD_DIR != 0 {
        source == Some(Kind::Directory)
    } else if mask & (ADD_LEAF | CHANGED) != 0 {
        leaf(source)
    } else {
        source.is_none()
    };
    let target_valid = if mask & REMOVE_DIR != 0 {
        target == Some(Kind::Directory)
    } else if mask & (REMOVE_LEAF | CHANGED) != 0 {
        leaf(target)
    } else {
        target.is_none()
    };
    if !source_valid {
        return Err(Failure::invalid("validate source entry", from, "entry type does not match record"));
    }
    if !target_valid {
        return Err(Failure::invalid("validate target entry", to, "entry type does not match record"));
    }
    Ok(validated)
}

struct Context<'a, C = NoCheck, S = NoScope, A = RemoveAll> {
    owned_cleanup: bool,
    policy: CopyPolicy,
    parallel: bool,
    stopped: AtomicBool,
    original: &'a Path,
    target: &'a Path,
    check: &'a C,
    scope: &'a S,
    absence: &'a A,
    retained: AtomicU64,
    excluded: AtomicU64,
}

// Keep usable descriptors alive through final metadata and rename. A path-only
// clone helper cannot return the descriptor it has already closed.
struct PreparedLeaf {
    progress: Progress,
    _source: Option<File>,
    _output: Option<File>,
}

impl Context<'static> {
    fn new(policy: CopyPolicy, parallel: bool) -> Self {
        let mut context = Self::checked(policy, parallel, Path::new(""), Path::new(""), &NoCheck, &NoScope, &RemoveAll);
        context.owned_cleanup = true;
        context
    }
}

impl<'a, C: Check, S: Scope, A: Absence> Context<'a, C, S, A> {
    fn checked(policy: CopyPolicy, parallel: bool, original: &'a Path, target: &'a Path, check: &'a C, scope: &'a S, absence: &'a A) -> Self {
        Self {
            owned_cleanup: false,
            policy,
            parallel,
            stopped: AtomicBool::new(false),
            original,
            target,
            check,
            scope,
            absence,
            retained: AtomicU64::new(0),
            excluded: AtomicU64::new(0),
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
                completed,
                policy,
                observed: None,
            }),
            Err(mut failure) => {
                *failure.policy += policy;
                Err(failure)
            }
        }
    }

    fn included(&self, relative: &Path) -> bool {
        let include = diff_dirs::scope_includes(self.scope, relative);
        if !include {
            self.excluded.fetch_add(1, Ordering::Relaxed);
        }
        include
    }

    // Recursive callers already admitted the routing ancestors. Do not repeat
    // prefix matching for them in the hot loop; includes is for public records.
    fn admit(&self, relative: &Path) -> bool {
        let include = !S::ENABLED || self.scope.at(relative) == diff_dirs::Visit::Reconcile;
        if !include {
            self.excluded.fetch_add(1, Ordering::Relaxed);
        }
        include
    }

    fn source_relative<'p>(&self, path: &'p Path) -> &'p Path {
        path.strip_prefix(self.original).expect("bound source descendant")
    }

    fn target_relative<'p>(&self, path: &'p Path) -> &'p Path {
        path.strip_prefix(self.target).expect("bound target descendant")
    }

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

    fn relative(&self, root: &Path, dir: &Path, side: Side, purpose: Purpose) -> Result<PathBuf, Failure> {
        // Record syntax was validated before any path lookup.
        let mut path = root.to_path_buf();
        let metadata = paths::directory(&path)?;
        self.observe(&path, side, purpose, &metadata, None)?;
        for component in dir.components() {
            path.push(component);
            let metadata = paths::directory(&path)?;
            self.observe(&path, side, purpose, &metadata, None)?;
        }
        Ok(path)
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

    fn named_with(&self, from: &Path, to: &Path, mask: u8, arena: &Staging<'_>, index: usize, observed: &Validated) -> Result<Progress, Failure> {
        #[cfg(all(test, feature = "parallel"))]
        tests::observe(to);
        if mask & ADD_DIR != 0 {
            if mask & REMOVE_LEAF != 0 {
                // Prepare the whole replacement before removing an old leaf.
                // A failed strict reflink must not destroy that leaf.
                return self.staged_directory(from, to, &arena.entry(index, to)?);
            }
            arena.writable.ensure()?;
            self.copy_directory_with(from, to, &observed.source.as_ref().expect("validated source directory").metadata)
        } else if mask & (ADD_LEAF | CHANGED) != 0 {
            if mask & REMOVE_DIR != 0 {
                // Only this affected subtree is inspected twice. Static blockers
                // are found before staging/chmod or dismantling any of it.
                self.removable(to, self.target_relative(to), true)?;
            }
            let source = observed.source.as_ref().expect("validated source leaf");
            self.copy_leaf_with(
                from,
                to,
                mask & REMOVE_DIR != 0,
                &source.metadata,
                source.link.as_deref(),
                &arena.entry(index, to)?,
            )
        } else {
            self.remove_tree(
                to,
                self.target_relative(to),
                &observed.target.as_ref().expect("validated target entry").metadata,
                &arena.writable,
                false,
            )
            .map(|(progress, _)| progress)
        }
    }

    fn copy_directory(&self, from: &Path, to: &Path) -> Result<Progress, Failure> {
        let metadata = paths::directory(from)?;
        self.copy_directory_with(from, to, &metadata)
    }

    fn copy_directory_with(&self, from: &Path, to: &Path, metadata: &Metadata) -> Result<Progress, Failure> {
        self.observe(from, Side::Original, Purpose::Copy, metadata, None)?;
        fs::create_dir(to).map_err(|error| Failure::io("create directory", to, error))?;
        let created = Progress {
            directories_created: 1,
            ..Progress::default()
        };
        // A restrictive process umask must not block our own child creation.
        let mut relaxed = false;
        let result = (|| {
            relaxed = relax(to)?.1;
            let children = children(from)?;
            let children: Vec<_> = children
                .into_iter()
                .filter(|name| !S::ENABLED || self.admit(&self.source_relative(from).join(name)))
                .collect();
            let arena = Staging::new(to, children.iter().map(|name| name.as_os_str()));
            let result = self.batch(&children, |index, name| {
                let from = from.join(name);
                let to = to.join(name);
                #[cfg(all(test, feature = "parallel"))]
                tests::observe(&to);
                let metadata = fs::symlink_metadata(&from).map_err(|error| Failure::io("inspect source entry", &from, error))?;
                if metadata.is_dir() {
                    self.copy_directory_with(&from, &to, &metadata)
                } else {
                    self.copy_leaf_with(&from, &to, false, &metadata, None, &arena.entry(index, &to)?)
                }
            });
            arena.finish(result, self)
        })();
        // Account for a successful relaxation even if listing failed before
        // child admission; finalization still follows every joined result.
        let mut result = finish_permissions(result, to, metadata.permissions(), relaxed, false);
        add_progress(&mut result, created);
        result
    }

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
        self.observe(from, Side::Original, Purpose::Copy, metadata, link)?;
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
        let metadata = fs::symlink_metadata(path).map_err(|error| Failure::io("inspect removal", path, error))?;
        // Owned cleanup must not relax the caller-owned containing directory.
        let parent = Writable::unmanaged(path.parent().expect("removal parent"));
        let relative = if self.target.as_os_str().is_empty() {
            path
        } else {
            self.target_relative(path)
        };
        let result = self.remove_tree(path, relative, &metadata, &parent, false).map(|(progress, _)| progress);
        parent.finish(result)
    }

    fn remove_present(&self, path: &Path) -> Result<Progress, Failure> {
        let metadata = fs::symlink_metadata(path).map_err(|error| Failure::io("inspect removal", path, error))?;
        let parent = Writable::new(path.parent().expect("removal parent"));
        let result = self
            .remove_tree(path, self.target_relative(path), &metadata, &parent, true)
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
        if (S::ENABLED && self.scope.at(relative) == diff_dirs::Visit::PreserveTree) || (!root && !self.absence.remove(relative)) {
            return Err(Failure::invalid(
                "replace directory",
                path,
                "target descendant retained by scope or absence authority",
            ));
        }
        let metadata = fs::symlink_metadata(path).map_err(|error| Failure::io("inspect replacement subtree", path, error))?;
        self.observe(path, Side::Target, Purpose::Remove, &metadata, None)?;
        if metadata_kind(path, &metadata)? == Kind::Directory {
            for name in children(path)? {
                self.removable(&path.join(&name), &relative.join(name), false)?;
            }
        }
        Ok(())
    }

    fn remove_tree(&self, path: &Path, relative: &Path, metadata: &Metadata, parent: &Writable<'_>, force_root: bool) -> Result<(Progress, bool), Failure> {
        if !self.admit(relative) {
            return Ok((Progress::default(), true));
        }
        self.observe(path, Side::Target, Purpose::Remove, metadata, None)?;
        if !force_root && A::NEVER {
            self.retained.fetch_add(1, Ordering::Relaxed);
            return Ok((Progress::default(), true));
        }
        let authorized = force_root || self.absence.remove(relative);
        match metadata_kind(path, metadata)? {
            Kind::Directory => {
                let writable = Writable::new(path);
                let mut retained = !authorized;
                let result = (|| {
                    // Privately owned failed preparations can be mode 000.
                    // Only cleanup may relax for inspection, never caller data.
                    if self.owned_cleanup {
                        writable.ensure()?;
                    }
                    let mut progress = Progress::default();
                    for name in children(path)? {
                        let child = path.join(&name);
                        let relative = relative.join(name);
                        if S::ENABLED && self.scope.at(&relative) == diff_dirs::Visit::PreserveTree {
                            self.excluded.fetch_add(1, Ordering::Relaxed);
                            retained = true;
                            continue;
                        }
                        let metadata = fs::symlink_metadata(&child).map_err(|error| {
                            let mut failure = Failure::io("inspect removal", &child, error);
                            failure.completed = progress.clone();
                            failure
                        })?;
                        match self.remove_tree(&child, &relative, &metadata, &writable, false) {
                            Ok((done, kept)) => {
                                progress += done;
                                retained |= kept;
                            }
                            Err(mut failure) => {
                                failure.completed += progress;
                                return Err(failure);
                            }
                        }
                    }
                    if retained {
                        self.retained.fetch_add(1, Ordering::Relaxed);
                        return Ok(progress);
                    }
                    // Removing the directory mutates its parent, not itself.
                    if let Err(mut failure) = parent.ensure() {
                        failure.completed += progress;
                        return Err(failure);
                    }
                    fs::remove_dir(path).map_err(|error| {
                        let mut failure = Failure::io("remove directory", path, error);
                        failure.completed = progress.clone();
                        failure
                    })?;
                    progress.directories_removed += 1;
                    Ok(progress)
                })();
                let result = if retained || result.is_err() {
                    writable.finish(result)
                } else {
                    writable.removed(result)
                };
                result.map(|progress| (progress, retained))
            }
            kind => {
                if !authorized {
                    self.retained.fetch_add(1, Ordering::Relaxed);
                    return Ok((Progress::default(), true));
                }
                parent.ensure()?;
                platform::remove_leaf(path, metadata).map_err(|error| Failure::io("remove leaf", path, error))?;
                Ok((
                    if kind == Kind::Symlink {
                        Progress {
                            symlinks_removed: 1,
                            ..Progress::default()
                        }
                    } else {
                        Progress {
                            files_removed: 1,
                            ..Progress::default()
                        }
                    },
                    false,
                ))
            }
        }
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

/// Explicit deletion of a bound container's contents; the container is never
/// removed. Not a missing-source mirror or a fabricated comparison record.
pub fn clear<S: Scope, C: Check>(target: impl AsRef<Path>, scope: S, check: C) -> Result<Report, Failure> {
    let target = paths::root(target.as_ref(), false)?;
    let context = Context::checked(CopyPolicy::Copy, false, &target, &target, &check, &scope, &RemoveAll);
    let metadata = paths::directory(&target)?;
    context.observe(&target, Side::Target, Purpose::Remove, &metadata, None)?;
    let result = (|| {
        if !context.included(Path::new("")) {
            return Ok(Progress::default());
        }
        let writable = Writable::new(&target);
        let result = (|| {
            let mut progress = Progress::default();
            for name in children(&target)? {
                if !context.included(Path::new(&name)) {
                    continue;
                }
                let path = target.join(&name);
                let done = (|| {
                    let metadata = fs::symlink_metadata(&path).map_err(|error| Failure::io("inspect clear entry", &path, error))?;
                    context.remove_tree(&path, Path::new(&name), &metadata, &writable, false).map(|(done, _)| done)
                })();
                match done {
                    Ok(done) => progress += done,
                    Err(mut failure) => {
                        failure.completed += progress;
                        return Err(failure);
                    }
                }
            }
            Ok(progress)
        })();
        writable.finish(result)
    })();
    context.finish(result)
}

/// Install only explicitly listed relative regular-file/symlink leaves into an
/// existing real target directory. Destinations must be absent. No source
/// directories are enumerated and no metadata supplied by callers is trusted.
pub fn seed_leaves<I, P, C>(original: impl AsRef<Path>, target: impl AsRef<Path>, leaves: I, policy: CopyPolicy, check: C) -> Result<Progress, Failure>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
    C: Check,
{
    let binding = Applier::new_checked(original, target, policy, check)?;
    paths::directory(binding.target())?;
    let context = Context::checked(policy, false, binding.original(), binding.target(), binding.check(), &NoScope, &RemoveAll);
    let mut completed = Progress::default();
    for leaf in leaves {
        let relative = leaf.as_ref();
        let result = (|| {
            if relative.as_os_str().is_empty() || relative.components().any(|part| !matches!(part, std::path::Component::Normal(_))) {
                return Err(Failure::invalid(
                    "seed relative path",
                    relative,
                    "expected a nonempty normal relative leaf path",
                ));
            }
            let parent = relative.parent().expect("relative leaf parent");
            let source_parent = context.relative(binding.original(), parent, Side::Original, Purpose::Copy)?;
            let from = source_parent.join(relative.file_name().unwrap());
            let source = inspect(&context, &from, Side::Original, Purpose::Copy)?.ok_or_else(|| {
                Failure::io(
                    "seed source missing",
                    &from,
                    io::Error::new(io::ErrorKind::NotFound, "listed source leaf is absent"),
                )
            })?;
            if source.metadata.is_dir() {
                return Err(Failure::invalid("seed source type", &from, "listed source must be a regular file or symlink"));
            }
            let mut to_parent = binding.target().to_path_buf();
            let root_metadata = paths::directory(&to_parent)?;
            context.observe(&to_parent, Side::Target, Purpose::Copy, &root_metadata, None)?;
            let mut created = Progress::default();
            let install = (|| {
                for component in parent.components() {
                    let child = to_parent.join(component);
                    let metadata = match fs::symlink_metadata(&child) {
                        Ok(metadata) => {
                            if !metadata.is_dir() {
                                return Err(Failure::invalid("seed target ancestor", &child, "expected real directory"));
                            }
                            metadata
                        }
                        Err(error) if error.kind() == io::ErrorKind::NotFound => {
                            let writable = Writable::new(&to_parent);
                            let result = (|| {
                                writable.ensure()?;
                                fs::create_dir(&child).map_err(|error| Failure::io("seed create parent", &child, error))?;
                                Ok(Progress {
                                    directories_created: 1,
                                    ..Progress::default()
                                })
                            })();
                            created += writable.finish(result)?;
                            // Like recursive copy, newly created output parents
                            // are not fabricated observed-entry certificates.
                            to_parent = child;
                            continue;
                        }
                        Err(error) => return Err(Failure::io("seed target ancestor", &child, error)),
                    };
                    context.observe(&child, Side::Target, Purpose::Copy, &metadata, None)?;
                    to_parent = child;
                }
                let to = to_parent.join(relative.file_name().unwrap());
                if paths::exists(&to)? {
                    return Err(Failure::io(
                        "seed destination exists",
                        &to,
                        io::Error::new(io::ErrorKind::AlreadyExists, "listed destination must be absent"),
                    ));
                }
                let arena = Staging::new(&to_parent, std::iter::once(relative.file_name().unwrap()));
                let result = (|| context.copy_leaf_with(&from, &to, false, &source.metadata, source.link.as_deref(), &arena.entry(0, &to)?))();
                arena.finish(result, &context)
            })();
            match install {
                Ok(mut progress) => {
                    progress += created;
                    Ok(progress)
                }
                Err(mut failure) => {
                    failure.completed += created;
                    Err(failure)
                }
            }
        })();
        match result {
            Ok(done) => completed += done,
            Err(mut failure) => {
                failure.completed += completed;
                return Err(failure);
            }
        }
    }
    Ok(completed)
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

    fn finish<C: Check, S: Scope, A: Absence>(self, result: Result<Progress, Failure>, context: &Context<'_, C, S, A>) -> Result<Progress, Failure> {
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
        self.writable.finish(result)
    }
}

/// One deferred permission lease per joined sibling batch. Merely inspecting a
/// retained subtree never chmods it. The mutex is never held across caller code.
struct Writable<'a> {
    path: &'a Path,
    state: Mutex<Option<(Permissions, bool)>>,
    enabled: bool,
}

impl<'a> Writable<'a> {
    fn new(path: &'a Path) -> Self {
        Self {
            path,
            state: Mutex::new(None),
            enabled: true,
        }
    }
    fn unmanaged(path: &'a Path) -> Self {
        Self {
            path,
            state: Mutex::new(None),
            enabled: false,
        }
    }

    fn ensure(&self) -> Result<(), Failure> {
        if !self.enabled {
            return Ok(());
        }
        let mut state = self.state.lock().unwrap();
        if state.is_none() {
            *state = Some(relax(self.path)?);
        }
        Ok(())
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
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../.test-agent/sync-tests");
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
        let scratch = Path::new(env!("CARGO_MANIFEST_DIR")).join("../.test-agent/sync-tests");
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
            crate::mirror_in_pool(&source, &target, CopyPolicy::Copy, &pool, diff_dirs::ParallelLimits::for_pool(&pool)).unwrap();
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
                let result = crate::mirror_in_pool(source, target, CopyPolicy::Copy, pool, diff_dirs::ParallelLimits::for_pool(pool));
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
        let mut walk = diff_dirs::walk(&source, &target).unwrap();
        assert!(walk.next().is_none());
    }
}
