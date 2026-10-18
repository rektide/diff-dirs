use std::collections::HashSet;
use std::ffi::OsStr;
use std::fs::{self, File, Metadata, OpenOptions, Permissions};
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::model::merge;
use crate::paths::{self, ADD_DIR, ADD_LEAF, CHANGED, REMOVE_DIR, REMOVE_LEAF};
use crate::{Applier, CopyPolicy, Failure, Progress, platform};

impl Applier {
    pub(crate) fn apply_with(&self, diff: &diff_dirs::DirDiff, parallel: bool) -> Result<Progress, Failure> {
        let names = paths::names(diff)?;
        let from = paths::relative(&self.original, &diff.dir)?;
        let to = paths::relative(&self.target, &diff.dir)?;
        // Validate actual immediate types before any chmod, staging or removal.
        for (name, mask) in &names {
            validate_entry(&from.join(name), &to.join(name), *mask)?;
        }
        if names.is_empty() {
            return Ok(Progress::default());
        }
        let context = Context::new(self.policy, parallel);
        let (permissions, relaxed) = relax(&to)?;
        let arena = Staging::new(&to, names.iter().map(|(name, _)| name.as_os_str()));
        let result = context.batch(&names, |index, (name, mask)| {
            context.named(&from.join(name), &to.join(name), *mask, &arena, index)
        });
        let result = arena.finish(result, &context);
        finish_permissions(result, &to, permissions, relaxed, true)
    }

    pub(crate) fn initialize_with(&self, parallel: bool) -> Result<Progress, Failure> {
        paths::directory(&self.original)?;
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
        let (permissions, relaxed) = relax(parent)?;
        let context = Context::new(self.policy, parallel);
        let result = context.copy_directory(&self.original, &self.target);
        finish_permissions(result, parent, permissions, relaxed, true)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Directory,
    File,
    Symlink,
}

fn kind(path: &Path) -> Result<Option<Kind>, Failure> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => Ok(Some(metadata_kind(path, &metadata)?)),
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

fn validate_entry(from: &Path, to: &Path, mask: u8) -> Result<(), Failure> {
    let source = kind(from)?;
    let target = kind(to)?;
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
    Ok(())
}

struct Context {
    policy: CopyPolicy,
    parallel: bool,
    stopped: AtomicBool,
}

// Keep usable descriptors alive through final metadata and rename. A path-only
// clone helper cannot return the descriptor it has already closed.
struct PreparedLeaf {
    progress: Progress,
    _source: Option<File>,
    _output: Option<File>,
}

impl Context {
    fn new(policy: CopyPolicy, parallel: bool) -> Self {
        Self {
            policy,
            parallel,
            stopped: AtomicBool::new(false),
        }
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

    fn named(&self, from: &Path, to: &Path, mask: u8, arena: &Staging<'_>, index: usize) -> Result<Progress, Failure> {
        #[cfg(all(test, feature = "parallel"))]
        tests::observe(to);
        if mask & ADD_DIR != 0 {
            if mask & REMOVE_LEAF != 0 {
                // Prepare the whole replacement before removing an old leaf.
                // A failed strict reflink must not destroy that leaf.
                return self.staged_directory(from, to, &arena.entry(index, to)?);
            }
            self.copy_directory(from, to)
        } else if mask & (ADD_LEAF | CHANGED) != 0 {
            self.copy_leaf(from, to, mask & REMOVE_DIR != 0, &arena.entry(index, to)?)
        } else {
            self.remove(to)
        }
    }

    fn copy_directory(&self, from: &Path, to: &Path) -> Result<Progress, Failure> {
        let metadata = paths::directory(from)?;
        self.copy_directory_with(from, to, &metadata)
    }

    fn copy_directory_with(&self, from: &Path, to: &Path, metadata: &Metadata) -> Result<Progress, Failure> {
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
            let arena = Staging::new(to, children.iter().map(|name| name.as_os_str()));
            let result = self.batch(&children, |index, name| {
                let from = from.join(name);
                let to = to.join(name);
                #[cfg(all(test, feature = "parallel"))]
                tests::observe(&to);
                let metadata = fs::symlink_metadata(&from).map_err(|error| Failure::io("inspect source entry", &from, error))?;
                match metadata_kind(&from, &metadata)? {
                    Kind::Directory => self.copy_directory_with(&from, &to, &metadata),
                    _ => self.copy_leaf_with(&from, &to, false, &metadata, &arena.entry(index, &to)?),
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
                let removed = self.remove(to);
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

    fn copy_leaf(&self, from: &Path, to: &Path, remove_directory: bool, staged: &Path) -> Result<Progress, Failure> {
        let metadata = fs::symlink_metadata(from).map_err(|error| Failure::io("inspect source leaf", from, error))?;
        self.copy_leaf_with(from, to, remove_directory, &metadata, staged)
    }

    fn copy_leaf_with(&self, from: &Path, to: &Path, remove_directory: bool, metadata: &Metadata, staged: &Path) -> Result<Progress, Failure> {
        let kind = metadata_kind(from, metadata)?;
        if kind == Kind::Directory {
            return Err(Failure::invalid("copy leaf", from, "expected a regular file or symlink"));
        }
        let prepared = if kind == Kind::Symlink {
            let link = fs::read_link(from).map_err(|error| Failure::io("read symlink", from, error))?;
            platform::symlink(&link, staged, metadata).map_err(|error| Failure::io("prepare symlink", to, error))?;
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
        let removed = if remove_directory { self.remove(to)? } else { Progress::default() };
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
        match metadata_kind(path, &metadata)? {
            Kind::Directory => {
                let (permissions, relaxed) = relax(path)?;
                let result = (|| {
                    let mut progress = Progress::default();
                    for name in children(path)? {
                        match self.remove(&path.join(name)) {
                            Ok(done) => progress += done,
                            Err(mut failure) => {
                                failure.completed += progress;
                                return Err(failure);
                            }
                        }
                    }
                    fs::remove_dir(path).map_err(|error| {
                        let mut failure = Failure::io("remove directory", path, error);
                        failure.completed = progress.clone();
                        failure
                    })?;
                    progress.directories_removed += 1;
                    Ok(progress)
                })();
                // On failure, restore surviving directories; on success none remains.
                if result.is_err() {
                    finish_permissions(result, path, permissions, relaxed, true)
                } else {
                    let mut result = result;
                    if relaxed {
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
            kind => {
                platform::remove_leaf(path, &metadata).map_err(|error| Failure::io("remove leaf", path, error))?;
                Ok(if kind == Kind::Symlink {
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
        }
    }

    fn entry(&self, index: usize, to: &Path) -> Result<PathBuf, Failure> {
        let mut temporary = self.temporary.lock().unwrap();
        if temporary.is_none() {
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

    fn finish(self, result: Result<Progress, Failure>, context: &Context) -> Result<Progress, Failure> {
        match self.temporary.into_inner().unwrap() {
            Some(temporary) => {
                // A stopped sibling can leave a successfully prepared directory
                // deliberately uninstalled. Only an unstopped successful batch
                // proves that every allocated slot was renamed out.
                let empty = result.is_ok() && !context.stopped.load(Ordering::Acquire);
                cleanup(temporary, result, empty)
            }
            None => result,
        }
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
