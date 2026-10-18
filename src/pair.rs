//! Immediate directory inspection. No traversal, summary access, or child jobs.

use std::cmp::Ordering;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::{Check, DirDiff, Entry as Observation, Error, Execution, Purpose, Scope, Side};

#[derive(Debug)]
pub(crate) struct Roots {
    pub original: PathBuf,
    pub target: PathBuf,
}

impl Roots {
    #[cfg(all(test, feature = "parallel"))]
    pub fn new(original: impl AsRef<Path>, target: impl AsRef<Path>) -> Result<Self, Error> {
        Self::new_checked(original, target, &crate::NoCheck)
    }

    #[cfg(all(test, feature = "parallel"))]
    pub fn inspect(&self, dir: PathBuf, execution: Execution) -> Result<ComparedPair, Error> {
        self.inspect_checked(dir, execution, &crate::NoCheck)
    }

    pub fn new_checked<C: Check>(original: impl AsRef<Path>, target: impl AsRef<Path>, check: &C) -> Result<Self, Error> {
        let roots = Self {
            original: original.as_ref().to_path_buf(),
            target: target.as_ref().to_path_buf(),
        };
        for (path, side) in [(&roots.original, Side::Original), (&roots.target, Side::Target)] {
            let metadata = fs::symlink_metadata(path).map_err(|e| Error::io(path.clone(), "inspect root", e))?;
            Observation {
                root: path,
                path,
                name: path.file_name().unwrap_or(path.as_os_str()),
                side,
                purpose: Purpose::Compare,
                metadata: &metadata,
                link_target: None,
            }
            .check(check)?;
            if !metadata.is_dir() {
                return Err(Error::NotDirectory { path: path.clone() });
            }
        }
        Ok(roots)
    }

    #[cfg(all(test, feature = "parallel"))]
    pub fn inspect_checked<C: Check>(&self, dir: PathBuf, execution: Execution, check: &C) -> Result<ComparedPair, Error> {
        self.inspect_scoped(dir, execution, check, &crate::NoScope)
    }

    pub fn inspect_scoped<C: Check, S: Scope>(&self, dir: PathBuf, execution: Execution, check: &C, scope: &S) -> Result<ComparedPair, Error> {
        let (original, target, excluded) = entries(self, &dir, execution, check, scope)?;
        let mut pair = compare(dir, original, target);
        pair.excluded = excluded;
        pair.scoped = S::ENABLED;
        Ok(pair)
    }
}

#[derive(Debug)]
pub(crate) struct ComparedPair {
    pub diff: DirDiff,
    pub children: Vec<OsString>,
    pub excluded: u64,
    pub scoped: bool,
}

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

fn entries<C: Check, S: Scope>(roots: &Roots, dir: &Path, execution: Execution, check: &C, scope: &S) -> Result<(Vec<Entry>, Vec<Entry>, u64), Error> {
    if !scope.includes(dir) {
        return Ok((Vec::new(), Vec::new(), 1));
    }
    let mut raw = Vec::new();
    list(&roots.original.join(dir), Side::Original, &mut raw)?;
    list(&roots.target.join(dir), Side::Target, &mut raw)?;
    raw.sort_unstable_by(|a, b| {
        a.name
            .as_encoded_bytes()
            .cmp(b.name.as_encoded_bytes())
            .then((a.side as u8).cmp(&(b.side as u8)))
    });
    // Decide once per union name, symmetrically, before any entry observation.
    let mut excluded = 0;
    if S::ENABLED {
        let mut previous = None;
        let mut include = true;
        raw.retain(|entry| {
            if previous.as_ref() != Some(&entry.name) {
                include = scope.includes(&dir.join(&entry.name));
                excluded += u64::from(!include);
                previous = Some(entry.name.clone());
            }
            include
        });
    }
    #[cfg(feature = "parallel")]
    if execution == Execution::Parallel && raw.len() >= 64 {
        use rayon::prelude::*;
        let results: Vec<_> = raw
            .into_par_iter()
            .with_min_len(16)
            .map(|entry| {
                #[cfg(test)]
                tests::observe_worker();
                inspect_entry(entry, roots, check)
            })
            .collect();
        return split_entries(results.into_iter()).map(|(a, b)| (a, b, excluded));
    }
    let _ = execution;
    split_entries(raw.into_iter().map(|entry| inspect_entry(entry, roots, check))).map(|(a, b)| (a, b, excluded))
}

fn inspect_entry<C: Check>(raw: RawEntry, roots: &Roots, check: &C) -> Result<(Side, Entry), Error> {
    let path = raw.entry.path();
    let metadata = raw.entry.metadata().map_err(|e| Error::io(path.clone(), "inspect entry", e))?;
    let link = if metadata.is_symlink() {
        Some(fs::read_link(&path).map_err(|e| Error::io(path.clone(), "read symlink", e))?)
    } else {
        None
    };
    if C::ENABLED {
        let root = match raw.side {
            Side::Original => &roots.original,
            Side::Target => &roots.target,
        };
        Observation {
            root,
            path: &path,
            name: &raw.name,
            side: raw.side,
            purpose: Purpose::Compare,
            metadata: &metadata,
            link_target: link.as_deref(),
        }
        .check(check)?;
    }
    let kind = if metadata.is_dir() {
        Kind::Directory
    } else if metadata.is_file() {
        Kind::File {
            size: metadata.len(),
            modified: metadata.modified().map_err(|e| Error::io(path.clone(), "read modification time", e))?,
        }
    } else if metadata.is_symlink() {
        Kind::Symlink(link.unwrap().into_os_string())
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

fn compare(dir: PathBuf, original: Vec<Entry>, target: Vec<Entry>) -> ComparedPair {
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
    ComparedPair {
        diff,
        children,
        excluded: 0,
        scoped: false,
    }
}

fn one_sided(diff: &mut DirDiff, entry: Entry, added: bool) {
    match (entry.kind, added) {
        (Kind::Directory, true) => diff.dirs.added.push(entry.name),
        (Kind::Directory, false) => diff.dirs.removed.push(entry.name),
        (_, true) => diff.files.added.push(entry.name),
        (_, false) => diff.files.removed.push(entry.name),
    }
}

#[cfg(all(test, feature = "parallel"))]
mod tests {
    use super::*;
    use std::sync::{Arc, Condvar, Mutex};

    static WORKERS: Mutex<Vec<(String, usize)>> = Mutex::new(Vec::new());
    static GATE: Mutex<Option<Arc<Gate>>> = Mutex::new(None);

    #[derive(Default)]
    struct Gate {
        arrivals: Mutex<usize>,
        changed: Condvar,
    }

    impl Gate {
        fn wait(&self) {
            let mut arrivals = self.arrivals.lock().unwrap();
            *arrivals += 1;
            self.changed.notify_all();
            let (arrivals, _) = self
                .changed
                .wait_timeout_while(arrivals, std::time::Duration::from_secs(5), |n| *n < 2)
                .unwrap();
            assert!(*arrivals >= 2, "no concurrent metadata job entered within five seconds");
        }
    }

    pub(super) fn observe_worker() {
        let gate = {
            let mut workers = WORKERS.lock().unwrap();
            workers.push((std::thread::current().name().unwrap_or("unnamed").to_string(), rayon::current_num_threads()));
            if workers.len() <= 2 { GATE.lock().unwrap().clone() } else { None }
        };
        if let Some(gate) = gate {
            gate.wait();
        }
    }

    #[test]
    fn per_frame_metadata_jobs_use_the_callers_named_pool() {
        let f = crate::test_support::Fixture::new();
        for i in 0..256 {
            f.both_file(format!("file-{i:03}"), b"");
        }
        let roots = Roots::new_checked(&f.original, &f.target, &crate::NoCheck).unwrap();
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(4)
            .thread_name(|i| format!("diff-dirs-caller-{i}"))
            .build()
            .unwrap();
        WORKERS.lock().unwrap().clear();
        *GATE.lock().unwrap() = Some(Arc::new(Gate::default()));
        pool.install(|| {
            assert!(
                roots
                    .inspect_checked(PathBuf::new(), Execution::Parallel, &crate::NoCheck)
                    .unwrap()
                    .diff
                    .is_empty()
            )
        });
        *GATE.lock().unwrap() = None;
        let workers = WORKERS.lock().unwrap();
        assert_eq!(workers.len(), 512);
        assert!(workers.iter().all(|(name, size)| name.starts_with("diff-dirs-caller-") && *size == 4));
        assert_ne!(workers[0].0, workers[1].0);
    }
}
