use std::cmp::Ordering;
use std::ffi::OsString;
use std::fs;
use std::iter::FusedIterator;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::{DirDiff, Error, Execution, ScanState, Summary, WalkOptions};

/// Validate two existing real directory roots and construct a lazy walker.
/// Directory contents are first read by `next()`, not by this constructor.
pub fn walk(original: impl AsRef<Path>, target: impl AsRef<Path>) -> Result<Walk, Error> {
    walk_with_options(original, target, WalkOptions::default())
}

/// Construct a walk with explicit execution policy. With `parallel`, metadata
/// jobs use the current Rayon pool and join before any record/error is yielded.
/// Reading the current two directory listings precedes metadata inspection;
/// metadata errors select the first native-name / original-before-target pair,
/// independent of worker scheduling. Listing errors precede metadata errors.
pub fn walk_with_options(original: impl AsRef<Path>, target: impl AsRef<Path>, options: WalkOptions) -> Result<Walk, Error> {
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
        options,
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
    options: WalkOptions,
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
        let (original, target) = entries(&self.original.join(&dir), &self.target.join(&dir), self.options.execution)?;
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

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Side {
    Original,
    Target,
}

struct RawEntry {
    entry: fs::DirEntry,
    name: OsString,
    side: Side,
}

fn list(dir: &Path, side: Side, entries: &mut Vec<RawEntry>) -> Result<(), Error> {
    let read = fs::read_dir(dir).map_err(|e| Error::io(dir.to_path_buf(), "read directory", e))?;
    for entry in read {
        let entry = entry.map_err(|e| Error::io(dir.to_path_buf(), "read directory entry", e))?;
        entries.push(RawEntry {
            name: entry.file_name(),
            entry,
            side,
        });
    }
    Ok(())
}

fn entries(original: &Path, target: &Path, execution: Execution) -> Result<(Vec<Entry>, Vec<Entry>), Error> {
    let mut raw = Vec::new();
    list(original, Side::Original, &mut raw)?;
    list(target, Side::Target, &mut raw)?;
    raw.sort_unstable_by(|a, b| a.name.as_encoded_bytes().cmp(b.name.as_encoded_bytes()).then(a.side.cmp(&b.side)));
    #[cfg(feature = "parallel")]
    if execution == Execution::Parallel && raw.len() >= 64 {
        use rayon::prelude::*;
        // Vec's indexed parallel iterator collects in original index order.
        // Do not collect directly into Result: racing error selection would
        // lose deterministic name/side priority. No jobs survive this collect.
        let results: Vec<_> = raw.into_par_iter().with_min_len(16).map(inspect_entry).collect();
        return split_entries(results.into_iter());
    }
    let _ = execution;
    split_entries(raw.into_iter().map(inspect_entry))
}

fn inspect_entry(raw: RawEntry) -> Result<(Side, Entry), Error> {
    #[cfg(all(test, feature = "parallel"))]
    observe_worker();
    let path = raw.entry.path();
    // DirEntry::metadata does not follow symlinks, unlike fs::metadata.
    let metadata = raw.entry.metadata().map_err(|e| Error::io(path.clone(), "inspect entry", e))?;
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
    Ok((raw.side, Entry { name: raw.name, kind }))
}

fn split_entries(results: impl Iterator<Item = Result<(Side, Entry), Error>>) -> Result<(Vec<Entry>, Vec<Entry>), Error> {
    let (mut original, mut target) = (Vec::new(), Vec::new());
    for result in results {
        let (side, entry) = result?;
        match side {
            Side::Original => original.push(entry),
            Side::Target => target.push(entry),
        }
    }
    Ok((original, target))
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

// Test-only observation at the actual metadata boundary, not merely at the
// ThreadPool::install entry. No instrumentation exists in production builds.
#[cfg(all(test, feature = "parallel"))]
static OBSERVED_WORKERS: std::sync::Mutex<Vec<(String, usize)>> = std::sync::Mutex::new(Vec::new());

#[cfg(all(test, feature = "parallel"))]
static METADATA_GATE: std::sync::Mutex<Option<std::sync::Arc<MetadataGate>>> = std::sync::Mutex::new(None);

#[cfg(all(test, feature = "parallel"))]
#[derive(Default)]
struct MetadataGate {
    arrivals: std::sync::Mutex<usize>,
    changed: std::sync::Condvar,
}

#[cfg(all(test, feature = "parallel"))]
impl MetadataGate {
    fn wait(&self) {
        let mut arrivals = self.arrivals.lock().unwrap();
        *arrivals += 1;
        self.changed.notify_all();
        let (arrivals, _) = self
            .changed
            .wait_timeout_while(arrivals, std::time::Duration::from_secs(5), |arrivals| *arrivals < 2)
            .unwrap();
        assert!(*arrivals >= 2, "no concurrent metadata job entered within five seconds");
    }
}

#[cfg(all(test, feature = "parallel"))]
fn observe_worker() {
    let gate = {
        let mut observed = OBSERVED_WORKERS.lock().unwrap();
        observed.push((std::thread::current().name().unwrap_or("unnamed").to_string(), rayon::current_num_threads()));
        if observed.len() <= 2 { METADATA_GATE.lock().unwrap().clone() } else { None }
    };
    // Only the first two jobs in this unit test rendezvous. A serial map merely
    // wrapped in install cannot pass: another worker must enter inspect_entry.
    // This hook is absent from production and integration-test library builds.
    if let Some(gate) = gate {
        gate.wait();
    }
}

#[cfg(all(test, feature = "parallel"))]
mod tests {
    use super::*;

    #[test]
    fn metadata_jobs_use_the_callers_named_pool() {
        let f = crate::test_support::Fixture::new();
        for i in 0..256 {
            f.both_file(format!("file-{i:03}"), b"");
        }
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(4)
            .thread_name(|i| format!("diff-dirs-caller-{i}"))
            .build()
            .unwrap();
        OBSERVED_WORKERS.lock().unwrap().clear();
        *METADATA_GATE.lock().unwrap() = Some(std::sync::Arc::new(MetadataGate::default()));
        pool.install(|| {
            let mut walk = walk_with_options(
                &f.original,
                &f.target,
                WalkOptions {
                    execution: Execution::Parallel,
                },
            )
            .unwrap();
            assert!(walk.next().is_none());
            assert!(walk.into_summary().is_clean());
        });
        *METADATA_GATE.lock().unwrap() = None;
        let observed = OBSERVED_WORKERS.lock().unwrap();
        assert_eq!(observed.len(), 512);
        assert!(observed.iter().all(|(name, size)| name.starts_with("diff-dirs-caller-") && *size == 4));
        assert_ne!(observed[0].0, observed[1].0, "metadata jobs must actually run on different workers");
    }
}
