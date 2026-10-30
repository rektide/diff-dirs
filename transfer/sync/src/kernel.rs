//! Policy-free physical primitives. Only the runner enumerates user trees.

use crate::{CopyPolicy, Failure, Postcheck, Progress, model::merge, platform};
use diff_dirs_tree_inspect::{DestinationGrant, InspectionEpoch, Kind, SideFacts, SlotId};
use std::collections::HashSet;
use std::ffi::OsStr;
use std::fs::{self, File, Metadata, OpenOptions, Permissions};
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

pub(crate) fn route(epoch: &InspectionEpoch, path: &Path) -> Result<std::sync::Arc<Metadata>, Failure> {
    epoch.directory(path).map_err(crate::apply::inspection_failure)
}

pub(crate) fn add_progress(result: &mut Result<Progress, Failure>, progress: Progress) {
    match result {
        Ok(done) => *done += progress,
        Err(f) => f.completed += progress,
    }
}

pub(crate) fn discard_private(mut result: Result<Progress, Failure>) -> Result<Progress, Failure> {
    match &mut result {
        Ok(p) => *p = Progress::default(),
        Err(f) => f.completed = Progress::default(),
    }
    result
}

/// Only listed retained acceptance calls this, after binding actual prepared
/// regular metadata and the manifest expectation. Caller owns stable trees.
pub(crate) fn conform_retained_executable(path: &Path, metadata: &Metadata, executable: bool) -> Result<bool, Failure> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = metadata.permissions().mode();
        if (mode & 0o111 != 0) != executable {
            let mode = if executable { mode | 0o111 } else { mode & !0o111 };
            fs::set_permissions(path, Permissions::from_mode(mode)).map_err(|e| Failure::io("conform retained executable", path, e))?;
            return Ok(true);
        }
    }
    #[cfg(not(unix))]
    {
        let _ = (path, metadata, executable);
        return Err(Failure::invalid("conform retained executable", path, "Unix executable expectation required"));
    }
    Ok(false)
}

/// A private slot can only be allocated by its containing scope. Publication is
/// bound back to that scope; an arbitrary public preparation cannot choose paths.
pub(crate) struct StageSlot {
    path: PathBuf,
    parent: PathBuf,
}
impl StageSlot {
    pub(crate) fn path(&self) -> &Path {
        &self.path
    }
}

pub(crate) struct BoundDestination<'a> {
    path: &'a Path,
    parent: &'a Path,
    grant: DestinationGrant,
    expected: Option<Kind>,
    cleared: bool,
}
impl<'a> BoundDestination<'a> {
    pub(crate) fn new(path: &'a Path, scope: &'a DirectoryScope, grant: DestinationGrant, expected: Option<Kind>) -> Result<Self, Failure> {
        if path.parent() != Some(scope.path()) {
            return Err(Failure::invalid("bind destination", path, "destination is outside its containing scope"));
        }
        Ok(Self {
            path,
            parent: scope.path(),
            grant,
            expected,
            cleared: false,
        })
    }
    pub(crate) fn cleared(&mut self) {
        self.cleared = true;
    }
    fn check(&self, slot: &StageSlot, epoch: &InspectionEpoch) -> Result<(), Failure> {
        if slot.parent != self.parent {
            return Err(Failure::invalid(
                "publish prepared node",
                self.path,
                "private slot belongs to another containing scope",
            ));
        }
        route(epoch, self.parent)?;
        let actual = match fs::symlink_metadata(self.path) {
            Ok(m) => Some(Kind::from_metadata(&m)),
            Err(e) if e.kind() == io::ErrorKind::NotFound => None,
            Err(e) => return Err(Failure::io("inspect publication destination", self.path, e)),
        };
        let allowed = if self.cleared || self.grant == DestinationGrant::AbsentOnly || self.expected.is_none() {
            actual.is_none()
        } else {
            actual == self.expected && actual.is_some_and(Kind::is_leaf) && self.grant != DestinationGrant::AbsentOnly
        };
        if !allowed {
            return Err(Failure::invalid(
                "publish prepared node",
                self.path,
                "destination changed or exceeds selected grant",
            ));
        }
        Ok(())
    }
}

pub(crate) struct PreparedLeaf {
    progress: Progress,
    _source: Option<File>,
    _output: Option<File>,
}

pub(crate) fn prepare_leaf(
    from: &Path,
    facts: SideFacts<'_>,
    slot: &StageSlot,
    to: &Path,
    policy: CopyPolicy,
    postcheck: Postcheck,
) -> Result<PreparedLeaf, Failure> {
    if facts.kind == Kind::Symlink {
        platform::symlink(
            facts.literal_link.known().expect("execution inputs checked"),
            slot.path(),
            facts.attributes.known(),
        )
        .map_err(|e| Failure::io("prepare symlink", to, e))?;
        Ok(PreparedLeaf {
            progress: Progress {
                symlinks_created: 1,
                ..Progress::default()
            },
            _source: None,
            _output: None,
        })
    } else {
        prepare_file(
            from,
            slot.path(),
            to,
            facts.attributes.known().expect("execution inputs checked"),
            policy,
            postcheck,
        )
    }
}

pub(crate) fn publish_leaf(prepared: PreparedLeaf, slot: &StageSlot, destination: &BoundDestination<'_>, epoch: &InspectionEpoch) -> Result<Progress, Failure> {
    publish_borrowed_leaf(&prepared, slot, destination, epoch)
}
pub(crate) fn publish_borrowed_leaf(
    prepared: &PreparedLeaf,
    slot: &StageSlot,
    destination: &BoundDestination<'_>,
    epoch: &InspectionEpoch,
) -> Result<Progress, Failure> {
    publish(slot, destination, epoch)?;
    Ok(prepared.progress.clone())
}

pub(crate) fn publish(slot: &StageSlot, destination: &BoundDestination<'_>, epoch: &InspectionEpoch) -> Result<(), Failure> {
    destination.check(slot, epoch)?;
    fs::rename(slot.path(), destination.path).map_err(|e| Failure::io("install prepared node", destination.path, e))?;
    epoch.invalidate_subtree(slot.path());
    epoch.invalidate_subtree(destination.path);
    Ok(())
}

pub(crate) fn remove_leaf(path: &Path, facts: SideFacts<'_>, epoch: &InspectionEpoch) -> Result<Progress, Failure> {
    remove_leaf_kind(path, facts.kind, facts.attributes.known(), epoch)
}
pub(crate) fn remove_leaf_kind(path: &Path, kind: Kind, attributes: Option<&Metadata>, epoch: &InspectionEpoch) -> Result<Progress, Failure> {
    platform::remove_leaf(path, kind, attributes).map_err(|e| Failure::io("remove leaf", path, e))?;
    epoch.invalidate_subtree(path);
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

pub(crate) fn create_directory(scope: &DirectoryScope, epoch: &InspectionEpoch) -> Result<(), Failure> {
    fs::create_dir(scope.path()).map_err(|e| Failure::io("create directory", scope.path(), e))?;
    epoch.invalidate_subtree(scope.path());
    Ok(())
}

pub(crate) fn remove_directory(path: &Path, epoch: &InspectionEpoch) -> Result<Progress, Failure> {
    fs::remove_dir(path).map_err(|e| Failure::io("remove directory", path, e))?;
    epoch.invalidate_subtree(path);
    Ok(Progress {
        directories_removed: 1,
        ..Progress::default()
    })
}

pub(crate) enum DirectoryEnd {
    Retained,
    Removed,
    Created { donor: Permissions },
}

/// One containing-directory lease spans every child and arena cleanup. Merely
/// constructing it, including for an absent conditional ancestor, does no I/O.
pub(crate) struct DirectoryScope {
    path: PathBuf,
    prefix: Mutex<String>,
    temporary: Mutex<Option<tempfile::TempDir>>,
    lease: Mutex<Option<(Permissions, bool)>>,
}
impl DirectoryScope {
    pub(crate) fn new<'n>(path: &Path, names: impl Iterator<Item = &'n OsStr>) -> Self {
        let reserved: HashSet<usize> = names
            .filter_map(|name| {
                let bytes = name.as_encoded_bytes();
                let base = b".diff-dirs-sync-";
                if !bytes.get(..base.len())?.eq_ignore_ascii_case(base) {
                    return None;
                }
                let suffix = &bytes[base.len()..];
                let end = suffix.iter().position(|b| *b == b'-')?;
                std::str::from_utf8(&suffix[..end]).ok()?.parse().ok()
            })
            .collect();
        let index = (0..).find(|i| !reserved.contains(i)).unwrap();
        Self {
            path: path.to_owned(),
            prefix: Mutex::new(format!(".diff-dirs-sync-{index}-")),
            temporary: Mutex::new(None),
            lease: Mutex::new(None),
        }
    }
    pub(crate) fn path(&self) -> &Path {
        &self.path
    }
    pub(crate) fn reserve_names<'n>(&self, names: impl Iterator<Item = &'n OsStr>) {
        // Newly owned containers are created before child inspection. Install
        // that inspection's reservations before any child can allocate an arena.
        let reserved = Self::new(&self.path, names);
        *self.prefix.lock().expect("arena prefix poisoned") = reserved.prefix.into_inner().unwrap();
    }
    pub(crate) fn ensure_writable(&self, epoch: &InspectionEpoch) -> Result<(), Failure> {
        let mut lease = self.lease.lock().expect("directory lease poisoned");
        if lease.is_none() {
            let original = route(epoch, &self.path)?.permissions();
            let writable = platform::writable(original.clone(), true);
            let changed = !platform::same_permissions(&original, &writable);
            if changed {
                fs::set_permissions(&self.path, writable).map_err(|e| Failure::io("relax target directory permissions", &self.path, e))?;
                epoch.invalidate_attributes(&self.path);
            }
            *lease = Some((original, changed));
        }
        Ok(())
    }
    pub(crate) fn stage_slot(&self, index: SlotId, to: &Path, epoch: &InspectionEpoch) -> Result<StageSlot, Failure> {
        self.ensure_writable(epoch)?;
        let mut temporary = self.temporary.lock().expect("arena poisoned");
        if temporary.is_none() {
            let mut builder = tempfile::Builder::new();
            let prefix = self.prefix.lock().expect("arena prefix poisoned").clone();
            builder.prefix(&prefix);
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                builder.permissions(Permissions::from_mode(0o700));
            }
            *temporary = Some(
                builder
                    .tempdir_in(&self.path)
                    .map_err(|e| Failure::io("create private staging directory", to, e))?,
            );
            let path = temporary.as_ref().unwrap().path();
            epoch.invalidate_subtree(path);
            // Ownership is recorded before post-umask observation/repair can fail.
            let actual = route(epoch, path)?.permissions();
            fs::set_permissions(path, platform::writable(actual, true)).map_err(|e| Failure::io("relax private staging directory", path, e))?;
            epoch.invalidate_attributes(path);
        }
        Ok(StageSlot {
            path: temporary.as_ref().unwrap().path().join(index.0.to_string()),
            parent: self.path.clone(),
        })
    }
    pub(crate) fn clean(&self, mut result: Result<Progress, Failure>, complete: bool, epoch: &InspectionEpoch) -> Result<Progress, Failure> {
        if let Some(temporary) = self.temporary.lock().expect("arena poisoned").take() {
            let path = temporary.keep();
            let cleaned = if complete {
                fs::remove_dir(&path).map_err(|e| Failure::io("clean private staging directory", &path, e))
            } else {
                remove_private(&path, epoch)
            };
            epoch.invalidate_subtree(&path);
            if let Err(mut failure) = cleaned {
                failure.primary.operation = "clean private staging directory";
                failure.completed = Progress::default();
                result = merge([result, Err(failure)]);
            }
        }
        result
    }
    pub(crate) fn finish(self, result: Result<Progress, Failure>, end: DirectoryEnd, complete: bool, epoch: &InspectionEpoch) -> Result<Progress, Failure> {
        let mut result = self.clean(result, complete, epoch);
        let lease = self.lease.into_inner().expect("directory lease poisoned");
        let relaxed = lease.as_ref().is_some_and(|(_, changed)| *changed);
        if relaxed {
            add_progress(
                &mut result,
                Progress {
                    directory_permissions_set: 1,
                    ..Progress::default()
                },
            );
        }
        let permissions = match end {
            DirectoryEnd::Created { donor } => Some(donor),
            DirectoryEnd::Retained if relaxed => lease.map(|(p, _)| p),
            _ => None,
        };
        if let Some(permissions) = permissions {
            match fs::set_permissions(&self.path, permissions) {
                Ok(()) => {
                    epoch.invalidate_attributes(&self.path);
                    add_progress(
                        &mut result,
                        Progress {
                            directory_permissions_set: 1,
                            ..Progress::default()
                        },
                    );
                }
                Err(e) => result = merge([result, Err(Failure::io("finalize directory permissions", &self.path, e))]),
            }
        }
        result
    }
}

/// Only private, exclusively owned staging trees are recursively cleaned here.
fn remove_private(path: &Path, epoch: &InspectionEpoch) -> Result<(), Failure> {
    let metadata = fs::symlink_metadata(path).map_err(|e| Failure::io("inspect private removal", path, e))?;
    if !metadata.is_dir() {
        platform::remove_leaf(path, Kind::from_metadata(&metadata), Some(&metadata)).map_err(|e| Failure::io("remove private leaf", path, e))?;
    } else {
        fs::set_permissions(path, platform::writable(metadata.permissions(), true)).map_err(|e| Failure::io("relax private directory", path, e))?;
        epoch.invalidate_attributes(path);
        let mut names = fs::read_dir(path)
            .map_err(|e| Failure::io("read private directory", path, e))?
            .map(|e| e.map(|e| e.file_name()).map_err(|e| Failure::io("read private directory entry", path, e)))
            .collect::<Result<Vec<_>, _>>()?;
        names.sort_unstable_by(|a, b| a.as_encoded_bytes().cmp(b.as_encoded_bytes()));
        for name in names {
            remove_private(&path.join(name), epoch)?;
        }
        fs::remove_dir(path).map_err(|e| Failure::io("remove private directory", path, e))?;
    }
    epoch.invalidate_subtree(path);
    Ok(())
}

fn preserve_file(output: &File, metadata: &Metadata, to: &Path, postcheck: Postcheck) -> Result<(), Failure> {
    let modified = metadata.modified().expect("whole-batch execution inputs checked before effects");
    output.set_modified(modified).map_err(|e| Failure::io("set file mtime", to, e))?;
    if postcheck == Postcheck::Mtime {
        let actual = output
            .metadata()
            .and_then(|m| m.modified())
            .map_err(|e| Failure::io("verify file mtime", to, e))?;
        if modified != actual {
            return Err(Failure::io(
                "verify file mtime",
                to,
                io::Error::new(io::ErrorKind::Unsupported, "target cannot represent exact source modification time"),
            ));
        }
    }
    output
        .set_permissions(metadata.permissions())
        .map_err(|e| Failure::io("set file permissions", to, e))
}

fn prepare_file(from: &Path, staged: &Path, to: &Path, metadata: &Metadata, policy: CopyPolicy, postcheck: Postcheck) -> Result<PreparedLeaf, Failure> {
    let (copied, source, output) = match policy {
        policy if policy == CopyPolicy::Copy || cfg!(target_os = "linux") => {
            let operation = if policy == CopyPolicy::Copy { "copy file" } else { "prepare reflink" };
            let mut source = File::open(from).map_err(|e| Failure::io(operation, from, e))?;
            let mut output = OpenOptions::new()
                .read(true)
                .write(true)
                .create_new(true)
                .open(staged)
                .map_err(|e| Failure::io("prepare file", to, e))?;
            let copied = if policy == CopyPolicy::Copy {
                Some(io::copy(&mut source, &mut output).map_err(|e| Failure::io(operation, from, e))?)
            } else {
                #[cfg(target_os = "linux")]
                {
                    match rustix::fs::ioctl_ficlone(&output, &source).map_err(io::Error::from) {
                        Ok(()) => None,
                        Err(e)
                            if policy == CopyPolicy::ReflinkOrCopy
                                && !matches!(
                                    e.kind(),
                                    io::ErrorKind::NotFound | io::ErrorKind::PermissionDenied | io::ErrorKind::AlreadyExists
                                ) =>
                        {
                            output.set_len(0).map_err(|e| Failure::io(operation, to, e))?;
                            Some(io::copy(&mut source, &mut output).map_err(|e| Failure::io(operation, from, e))?)
                        }
                        Err(e) => return Err(Failure::io(operation, to, e)),
                    }
                }
                #[cfg(not(target_os = "linux"))]
                unreachable!()
            };
            preserve_file(&output, metadata, to, postcheck)?;
            (copied, Some(source), output)
        }
        policy => {
            let copied = match policy {
                CopyPolicy::ReflinkRequired => reflink_copy::reflink(from, staged).map(|()| None),
                CopyPolicy::ReflinkOrCopy => reflink_copy::reflink_or_copy(from, staged),
                CopyPolicy::Copy => unreachable!(),
            }
            .map_err(|e| Failure::io("prepare reflink", to, e))?;
            fs::set_permissions(staged, platform::writable(metadata.permissions(), false)).map_err(|e| Failure::io("open prepared file permissions", to, e))?;
            let output = OpenOptions::new()
                .read(true)
                .write(true)
                .open(staged)
                .map_err(|e| Failure::io("open prepared file", to, e))?;
            preserve_file(&output, metadata, to, postcheck)?;
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

#[cfg(all(test, unix))]
#[path = "kernel_tests.rs"]
mod tests;
