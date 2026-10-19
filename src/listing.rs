//! Private, single-pass listing and deferred no-follow metadata. No handles escape
//! into visitor payloads or plans. Native entries share one parent descriptor.

use std::ffi::OsString;
use std::fs::Metadata;
use std::io;
use std::path::{Path, PathBuf};

use crate::{EarlyFacts, Error, FactOrigin, FileKind};

#[cfg(all(target_os = "linux", feature = "native-dirent"))]
mod backend {
    use super::*;
    use std::fs::Dir;
    use std::mem::MaybeUninit;
    use std::os::unix::ffi::OsStrExt;
    use std::sync::Arc;

    pub(super) struct Parent {
        dir: Dir,
        path: PathBuf,
    }

    pub(super) struct Handle {
        parent: Arc<Parent>,
        name: OsString,
    }

    impl Handle {
        pub(super) fn file_name(&self) -> OsString {
            self.name.clone()
        }
        pub(super) fn path(&self) -> PathBuf {
            self.parent.path.join(&self.name)
        }
        pub(super) fn metadata(self) -> io::Result<Metadata> {
            self.parent.dir.symlink_metadata(&self.name)
        }
    }

    fn kind(kind: rustix::fs::FileType) -> FileKind {
        use rustix::fs::FileType as T;
        match kind {
            T::Directory => FileKind::Directory,
            T::RegularFile => FileKind::RegularFile,
            T::Symlink => FileKind::Symlink,
            T::Fifo => FileKind::Fifo,
            T::Socket => FileKind::Socket,
            T::BlockDevice => FileKind::BlockDevice,
            T::CharacterDevice => FileKind::CharacterDevice,
            _ => FileKind::Unknown,
        }
    }

    pub(super) fn read(path: &Path) -> Result<Vec<Entry>, Error> {
        let parent = Arc::new(Parent {
            dir: Dir::open(path).map_err(|e| Error::io(path.to_path_buf(), "read directory", e))?,
            path: path.to_path_buf(),
        });
        let mut entries = Vec::new();
        let mut buffer = vec![MaybeUninit::uninit(); 8192];
        // RawDir owns parsing and returns cached d_type/ino without stat fallback.
        // On EINVAL grow and continue from the same directory offset, as its API
        // specifies; never restart the listing or pair it with a std listing.
        loop {
            let mut raw = rustix::fs::RawDir::new(&parent.dir, &mut buffer);
            let grow = loop {
                let entry = match raw.next() {
                    None => break false,
                    Some(Err(rustix::io::Errno::INTR)) => continue,
                    Some(Err(rustix::io::Errno::INVAL)) => break true,
                    Some(Err(e)) => return Err(Error::io(path.to_path_buf(), "read directory entry", e.into())),
                    Some(Ok(entry)) => entry,
                };
                if matches!(entry.file_name().to_bytes(), b"." | b"..") {
                    continue;
                }
                let hints = EarlyFacts {
                    kind: kind(entry.file_type()),
                    inode: Some(entry.ino()),
                    origin: FactOrigin::Listing,
                };
                let handle = Handle {
                    parent: Arc::clone(&parent),
                    name: std::ffi::OsStr::from_bytes(entry.file_name().to_bytes()).to_os_string(),
                };
                entries.push(Entry::new(handle, hints));
            };
            if !grow {
                return Ok(entries);
            }
            buffer.resize(buffer.len().checked_mul(2).expect("directory buffer capacity overflow"), MaybeUninit::uninit());
        }
    }
}

#[cfg(not(all(target_os = "linux", feature = "native-dirent")))]
mod backend {
    use super::*;
    use std::fs;

    pub(super) struct Handle {
        entry: fs::DirEntry,
        #[cfg(windows)]
        cached: Metadata,
    }
    impl Handle {
        pub(super) fn file_name(&self) -> OsString {
            self.entry.file_name()
        }
        pub(super) fn path(&self) -> PathBuf {
            self.entry.path()
        }
        pub(super) fn metadata(self) -> io::Result<Metadata> {
            #[cfg(windows)]
            return Ok(self.cached);
            #[cfg(not(windows))]
            self.entry.metadata()
        }
    }

    pub(super) fn read(path: &Path) -> Result<Vec<Entry>, Error> {
        fs::read_dir(path)
            .map_err(|e| Error::io(path.to_path_buf(), "read directory", e))?
            .map(|entry| {
                let entry = entry.map_err(|e| Error::io(path.to_path_buf(), "read directory entry", e))?;
                // Unix ino() reads d_ino, with no observation fallback. Never use
                // std file_type() here: DT_UNKNOWN may silently call metadata().
                #[cfg(unix)]
                let inode = Some(std::os::unix::fs::DirEntryExt::ino(&entry));
                #[cfg(not(unix))]
                let inode = None;
                #[cfg(not(windows))]
                let hints = EarlyFacts {
                    kind: FileKind::Unknown,
                    inode,
                    origin: FactOrigin::Listing,
                };
                // std documents Windows DirEntry::metadata as using the cached
                // enumeration data, without a syscall. Reuse that value late.
                #[cfg(windows)]
                let cached = entry.metadata().map_err(|e| Error::io(entry.path(), "inspect entry", e))?;
                #[cfg(windows)]
                let hints = EarlyFacts {
                    kind: FileKind::from_metadata(&cached),
                    inode,
                    origin: FactOrigin::Listing,
                };
                Ok(Entry::new(
                    Handle {
                        entry,
                        #[cfg(windows)]
                        cached,
                    },
                    hints,
                ))
            })
            .collect()
    }
}

pub(crate) struct Entry {
    handle: backend::Handle,
    hints: EarlyFacts,
    #[cfg(test)]
    pub(crate) calls: Option<std::sync::Arc<std::sync::atomic::AtomicUsize>>,
}
impl Entry {
    fn new(handle: backend::Handle, hints: EarlyFacts) -> Self {
        Self {
            handle,
            hints,
            #[cfg(test)]
            calls: None,
        }
    }
    pub(crate) fn file_name(&self) -> OsString {
        self.handle.file_name()
    }
    pub(crate) fn path(&self) -> PathBuf {
        self.handle.path()
    }
    pub(crate) fn hints(&self) -> EarlyFacts {
        self.hints
    }
    pub(crate) fn metadata(self) -> io::Result<Metadata> {
        #[cfg(test)]
        if let Some(calls) = self.calls {
            calls.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }
        self.handle.metadata()
    }
    #[cfg(test)]
    pub(crate) fn set_kind(&mut self, kind: FileKind) {
        self.hints.kind = kind;
    }
}

pub(crate) fn read(path: &Path) -> Result<Vec<Entry>, Error> {
    backend::read(path)
}
