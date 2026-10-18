use std::cmp::Ordering;
use std::ffi::OsString;
use std::fs;
use std::iter::FusedIterator;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::{DirDiff, Error, ScanState, Summary};

/// Validate two existing real directory roots and construct a lazy walker.
/// Directory contents are first read by `next()`, not by this constructor.
pub fn walk(original: impl AsRef<Path>, target: impl AsRef<Path>) -> Result<Walk, Error> {
    let original = original.as_ref().to_path_buf();
    let target = target.as_ref().to_path_buf();
    for path in [&original, &target] {
        let metadata = fs::symlink_metadata(path).map_err(|e| Error::io(path.clone(), "inspect root", e))?;
        if !metadata.is_dir() {
            return Err(Error::NotDirectory { path: path.clone() });
        }
    }
    Ok(Walk {
        original,
        target,
        frames: Vec::new(),
        start: true,
        prune_eligible: false,
        summary: Summary::default(),
    })
}

/// Sequential paired DFS. Only pending paired child directory names on the
/// active descent and the current listings are retained (plus the summary).
#[derive(Debug)]
pub struct Walk {
    original: PathBuf,
    target: PathBuf,
    frames: Vec<Frame>,
    start: bool,
    prune_eligible: bool,
    summary: Summary,
}

#[derive(Debug)]
struct Frame {
    dir: PathBuf,
    children: std::vec::IntoIter<OsString>,
    direct_ancestor: bool,
    subtree_changed: bool,
}

impl Walk {
    /// Skip the last record's pending paired child directories. Legal exactly
    /// once immediately after `Some(Ok(_))`; otherwise returns false and does
    /// nothing. Even a leaf prune is recorded as an explicit caller decision.
    pub fn prune_here(&mut self) -> bool {
        if !self.prune_eligible {
            return false;
        }
        self.prune_eligible = false;
        let frame = self.frames.last_mut().expect("yielded record has a frame");
        frame.children = Vec::new().into_iter();
        self.summary.pruned.push(frame.dir.clone());
        true
    }

    /// Return the examined prefix, never finishing the walk or performing IO.
    /// Normal completion requires calling `next()` until it returns `None`.
    pub fn into_summary(mut self) -> Summary {
        while !self.frames.is_empty() {
            self.finish_frame();
        }
        self.summary
    }

    fn finish_frame(&mut self) {
        let frame = self.frames.pop().expect("nonempty frames");
        if frame.subtree_changed {
            self.summary.changed_dirs.push(frame.dir);
            if let Some(parent) = self.frames.last_mut() {
                parent.subtree_changed = true;
            }
        }
    }

    fn examine(&mut self, dir: PathBuf, direct_ancestor: bool) -> Result<Option<DirDiff>, Error> {
        // Fully capture both immediate listings before yielding. The caller may
        // then replace classified immediate children without invalidating them.
        let original = entries(&self.original.join(&dir))?;
        let target = entries(&self.target.join(&dir))?;
        let (diff, children) = compare(dir, original, target);
        let changed = !diff.is_empty();
        self.summary.examined_directories += 1;
        self.summary.counts.add(&diff);
        if changed && !direct_ancestor {
            self.summary.topmost_divergent.push(diff.dir.clone());
        }
        self.frames.push(Frame {
            dir: diff.dir.clone(),
            children: children.into_iter(),
            direct_ancestor: direct_ancestor || changed,
            subtree_changed: changed,
        });
        Ok(changed.then_some(diff))
    }

    fn advance(&mut self) -> Result<Option<DirDiff>, Error> {
        loop {
            let (dir, direct_ancestor) = if self.start {
                self.start = false;
                (PathBuf::new(), false)
            } else if let Some(frame) = self.frames.last_mut() {
                if let Some(name) = frame.children.next() {
                    (frame.dir.join(name), frame.direct_ancestor)
                } else {
                    self.finish_frame();
                    continue;
                }
            } else {
                self.summary.state = ScanState::Complete;
                return Ok(None);
            };
            if let Some(diff) = self.examine(dir, direct_ancestor)? {
                return Ok(Some(diff));
            }
        }
    }
}

impl Iterator for Walk {
    type Item = Result<DirDiff, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.prune_eligible = false;
        if self.summary.state != ScanState::Incomplete {
            return None;
        }
        match self.advance() {
            Ok(Some(diff)) => {
                self.prune_eligible = true;
                Some(Ok(diff))
            }
            Ok(None) => None,
            Err(error) => {
                self.summary.state = ScanState::Failed;
                Some(Err(error))
            }
        }
    }
}

impl FusedIterator for Walk {}

#[derive(Debug, PartialEq, Eq)]
enum Kind {
    Directory,
    File { size: u64, modified: SystemTime },
    Symlink(OsString),
}

struct Entry {
    name: OsString,
    kind: Kind,
}

fn entries(dir: &Path) -> Result<Vec<Entry>, Error> {
    let read = fs::read_dir(dir).map_err(|e| Error::io(dir.to_path_buf(), "read directory", e))?;
    let mut entries = Vec::new();
    for entry in read {
        let entry = entry.map_err(|e| Error::io(dir.to_path_buf(), "read directory entry", e))?;
        let path = entry.path();
        // DirEntry::metadata does not follow symlinks, unlike fs::metadata.
        let metadata = entry.metadata().map_err(|e| Error::io(path.clone(), "inspect entry", e))?;
        let kind = if metadata.is_dir() {
            Kind::Directory
        } else if metadata.is_file() {
            Kind::File {
                size: metadata.len(),
                modified: metadata.modified().map_err(|e| Error::io(path.clone(), "read modification time", e))?,
            }
        } else if metadata.is_symlink() {
            Kind::Symlink(fs::read_link(&path).map_err(|e| Error::io(path, "read symlink", e))?.into_os_string())
        } else {
            return Err(Error::UnsupportedEntry { path });
        };
        entries.push(Entry { name: entry.file_name(), kind });
    }
    entries.sort_unstable_by(|a, b| a.name.as_encoded_bytes().cmp(b.name.as_encoded_bytes()));
    Ok(entries)
}

fn compare(dir: PathBuf, original: Vec<Entry>, target: Vec<Entry>) -> (DirDiff, Vec<OsString>) {
    let mut diff = DirDiff { dir, ..DirDiff::default() };
    let mut children = Vec::new();
    let mut original = original.into_iter().peekable();
    let mut target = target.into_iter().peekable();
    loop {
        match (original.peek(), target.peek()) {
            (Some(a), Some(b)) => match a.name.as_encoded_bytes().cmp(b.name.as_encoded_bytes()) {
                Ordering::Less => one_sided(&mut diff, original.next().unwrap(), true),
                Ordering::Greater => one_sided(&mut diff, target.next().unwrap(), false),
                Ordering::Equal => {
                    let a = original.next().unwrap();
                    let b = target.next().unwrap();
                    match (&a.kind, &b.kind) {
                        (Kind::Directory, Kind::Directory) => children.push(a.name),
                        (Kind::Directory, _) | (_, Kind::Directory) => {
                            one_sided(&mut diff, a, true);
                            one_sided(&mut diff, b, false);
                        }
                        _ if a.kind != b.kind => diff.files.changed.push(a.name),
                        _ => {}
                    }
                }
            },
            (Some(_), None) => one_sided(&mut diff, original.next().unwrap(), true),
            (None, Some(_)) => one_sided(&mut diff, target.next().unwrap(), false),
            (None, None) => break,
        }
    }
    (diff, children)
}

fn one_sided(diff: &mut DirDiff, entry: Entry, added: bool) {
    match (entry.kind, added) {
        (Kind::Directory, true) => diff.dirs.added.push(entry.name),
        (Kind::Directory, false) => diff.dirs.removed.push(entry.name),
        (_, true) => diff.files.added.push(entry.name),
        (_, false) => diff.files.removed.push(entry.name),
    }
}
