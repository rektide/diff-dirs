//! Bounded successful source observations. Active width is not retained capacity.
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicU64, Ordering},
};
use std::{collections::HashMap, mem::size_of};

use crate::acquire::{Acquire, AcquisitionCell, SuccessfulCell};
use crate::{EarlyFacts, Error, ObservationResources};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SourceCacheLimits {
    pub batches: usize,
    pub charged_capacity: usize,
}
impl Default for SourceCacheLimits {
    fn default() -> Self {
        Self {
            batches: 32,
            charged_capacity: 8 * 1024 * 1024,
        }
    }
}

/// Diagnostic attribution only; it is not scheduling or persistence state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourceWork {
    Paired,
    Opaque,
    Declared,
}
impl SourceWork {
    fn index(self) -> usize {
        match self {
            Self::Paired => 0,
            Self::Opaque => 1,
            Self::Declared => 2,
        }
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SourceWorkDiagnostics {
    pub listings: u64,
    pub metadata_acquisitions: u64,
    pub link_acquisitions: u64,
    pub fact_hits: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SourceDiagnostics {
    pub paired: SourceWorkDiagnostics,
    pub opaque: SourceWorkDiagnostics,
    pub declared: SourceWorkDiagnostics,
    /// Idle reusable batches only. Pinned active observations are reported separately.
    pub retained_batches: usize,
    pub retained_charged_capacity: usize,
    pub active_batches: usize,
    pub evictions: u64,
}
#[derive(Debug, Default)]
struct WorkCounters {
    listings: AtomicU64,
    metadata: AtomicU64,
    links: AtomicU64,
    facts: AtomicU64,
    hits: AtomicU64,
    misses: AtomicU64,
}
#[derive(Clone)]
pub(crate) struct SourceCounter {
    counters: Arc<[WorkCounters; 3]>,
    work: SourceWork,
}
impl SourceCounter {
    fn at(&self) -> &WorkCounters {
        &self.counters[self.work.index()]
    }
    pub(crate) fn metadata(&self) {
        self.at().metadata.fetch_add(1, Ordering::Relaxed);
    }
    pub(crate) fn link(&self) {
        self.at().links.fetch_add(1, Ordering::Relaxed);
    }
    pub(crate) fn hit(&self) {
        self.at().facts.fetch_add(1, Ordering::Relaxed);
    }
}
impl WorkCounters {
    fn snapshot(&self) -> SourceWorkDiagnostics {
        SourceWorkDiagnostics {
            listings: self.listings.load(Ordering::Relaxed),
            metadata_acquisitions: self.metadata.load(Ordering::Relaxed),
            link_acquisitions: self.links.load(Ordering::Relaxed),
            fact_hits: self.facts.load(Ordering::Relaxed),
            cache_hits: self.hits.load(Ordering::Relaxed),
            cache_misses: self.misses.load(Ordering::Relaxed),
        }
    }
}

/// Enumeration hints never change when a cell later obtains stronger facts.
struct OriginalListing {
    names: Vec<(OsString, EarlyFacts)>,
}
#[derive(Default)]
struct SourceBatch {
    complete_names: Option<OriginalListing>,
    // Sorted sparse positive cells. A future Exact lookup can populate this vector
    // without claiming enumeration completeness or absence of any other name.
    cells: Vec<(OsString, Arc<SuccessfulCell>)>,
}
impl SourceBatch {
    fn charge(&self) -> usize {
        let mut bytes = size_of::<Mutex<Self>>() + 2 * size_of::<usize>() + self.cells.capacity() * size_of::<(OsString, Arc<SuccessfulCell>)>();
        for (name, cell) in &self.cells {
            bytes = bytes.saturating_add(name.capacity()).saturating_add(cell.charge());
        }
        if let Some(listing) = &self.complete_names {
            bytes = bytes.saturating_add(listing.names.capacity() * size_of::<(OsString, EarlyFacts)>());
            for (name, _) in &listing.names {
                bytes = bytes.saturating_add(name.capacity());
            }
        }
        bytes
    }
}
struct Cached {
    batch: Arc<Mutex<SourceBatch>>,
    users: usize,
    charge: usize,
    touched: u64,
}
struct Cache {
    limits: SourceCacheLimits,
    entries: HashMap<PathBuf, Cached>,
    clock: u64,
    evictions: u64,
}
impl Cache {
    fn trim(&mut self) {
        loop {
            self.entries.shrink_to_fit();
            let (count, bytes) = self.retained();
            if count <= self.limits.batches
                && bytes <= self.limits.charged_capacity
                && (count == 0 || (self.limits.batches != 0 && self.limits.charged_capacity != 0))
            {
                break;
            }
            let Some(key) = self
                .entries
                .iter()
                .filter(|(_, e)| e.users == 0)
                .min_by_key(|(_, e)| e.touched)
                .map(|(k, _)| k.clone())
            else {
                break;
            };
            self.entries.remove(&key);
            self.evictions += 1;
        }
        // A HashMap otherwise keeps a historical high-water catalog allocation.
        self.entries.shrink_to_fit();
    }
    fn retained(&self) -> (usize, usize) {
        let (count, bytes) = self
            .entries
            .values()
            .filter(|e| e.users == 0)
            .fold((0, 0usize), |(n, bytes), e| (n + 1, bytes.saturating_add(e.charge)));
        // Conservatively charge the complete cache index to retained storage if
        // any idle entry exists, including control bytes and load-factor slack.
        let index = if count == 0 {
            0
        } else {
            self.entries
                .capacity()
                .saturating_add(1)
                .next_power_of_two()
                .saturating_mul(size_of::<(PathBuf, Cached)>() + 1)
        };
        (count, bytes.saturating_add(index))
    }
}

/// One physical source generation; neither policy, traversal nor target state.
pub struct SourceSession {
    root: PathBuf,
    generation: u64,
    resources: ObservationResources,
    cache: Mutex<Cache>,
    counters: Arc<[WorkCounters; 3]>,
}
impl std::fmt::Debug for SourceSession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SourceSession")
            .field("root", &self.root)
            .field("generation", &self.generation)
            .field("diagnostics", &self.diagnostics())
            .finish()
    }
}
static GENERATION: AtomicU64 = AtomicU64::new(1);
impl SourceSession {
    pub fn new(root: impl AsRef<Path>) -> Self {
        Self::with_resources(root, ObservationResources::default())
    }
    pub fn with_resources(root: impl AsRef<Path>, resources: ObservationResources) -> Self {
        Self::with_cache_limits(root, resources, SourceCacheLimits::default())
    }
    pub fn with_cache_limits(root: impl AsRef<Path>, resources: ObservationResources, limits: SourceCacheLimits) -> Self {
        let generation = GENERATION
            .try_update(Ordering::Relaxed, Ordering::Relaxed, |v| v.checked_add(1))
            .expect("source generation space exhausted");
        Self {
            root: root.as_ref().to_owned(),
            generation,
            resources,
            cache: Mutex::new(Cache {
                limits,
                entries: HashMap::new(),
                clock: 0,
                evictions: 0,
            }),
            counters: Arc::new(std::array::from_fn(|_| WorkCounters::default())),
        }
    }
    pub fn root(&self) -> &Path {
        &self.root
    }
    pub fn generation(&self) -> u64 {
        self.generation
    }
    pub fn resources(&self) -> &ObservationResources {
        &self.resources
    }
    pub fn cache_limits(&self) -> SourceCacheLimits {
        self.cache.lock().expect("source cache poisoned").limits
    }
    /// Pinned batches are never waited for or evicted; on release they obey the
    /// current limits. Zero in either dimension disables cross-call retention.
    pub fn set_cache_limits(&self, limits: SourceCacheLimits) {
        let mut cache = self.cache.lock().expect("source cache poisoned");
        cache.limits = limits;
        cache.trim();
    }
    pub fn diagnostics(&self) -> SourceDiagnostics {
        let cache = self.cache.lock().expect("source cache poisoned");
        let (retained_batches, retained_charged_capacity) = cache.retained();
        SourceDiagnostics {
            paired: self.counters[0].snapshot(),
            opaque: self.counters[1].snapshot(),
            declared: self.counters[2].snapshot(),
            retained_batches,
            retained_charged_capacity,
            active_batches: cache.entries.values().filter(|e| e.users != 0).count(),
            evictions: cache.evictions,
        }
    }
    pub(crate) fn checkout(&self, relative: &Path, work: SourceWork) -> SourceLease<'_> {
        let mut cache = self.cache.lock().expect("source cache poisoned");
        cache.clock = cache.clock.wrapping_add(1);
        let clock = cache.clock;
        let counter = SourceCounter {
            counters: self.counters.clone(),
            work,
        };
        let entry = cache.entries.entry(relative.to_owned()).or_insert_with(|| Cached {
            batch: Arc::default(),
            users: 0,
            charge: 0,
            touched: clock,
        });
        entry.users += 1;
        entry.touched = clock;
        SourceLease {
            source: self,
            relative: relative.to_owned(),
            batch: entry.batch.clone(),
            counter,
        }
    }
}
pub(crate) struct SourceLease<'a> {
    source: &'a SourceSession,
    relative: PathBuf,
    batch: Arc<Mutex<SourceBatch>>,
    counter: SourceCounter,
}
impl SourceLease<'_> {
    pub(crate) fn listed(&self, path: &Path, acquire: &impl Acquire) -> Result<Vec<(OsString, Arc<AcquisitionCell>)>, Error> {
        // Only a per-directory lock spans enumeration, never the cache index.
        let mut batch = self.batch.lock().expect("source batch poisoned");
        if batch.complete_names.is_none() {
            self.counter.at().misses.fetch_add(1, Ordering::Relaxed);
            self.counter.at().listings.fetch_add(1, Ordering::Relaxed);
            let mut entries = acquire.list(path)?;
            entries.sort_unstable_by(|a, b| a.file_name().as_encoded_bytes().cmp(b.file_name().as_encoded_bytes()));
            if entries.windows(2).any(|p| p[0].file_name() == p[1].file_name()) {
                return Err(Error::io(
                    path.to_owned(),
                    "read directory entry",
                    std::io::Error::new(std::io::ErrorKind::InvalidData, "duplicate physical listing name"),
                ));
            }
            let mut names = Vec::with_capacity(entries.len());
            for entry in entries {
                let name = entry.file_name();
                names.push((name.clone(), entry.hints()));
                match batch.cells.binary_search_by(|(n, _)| n.as_encoded_bytes().cmp(name.as_encoded_bytes())) {
                    Ok(index) => batch.cells[index].1.attach_listing(entry),
                    Err(index) => batch.cells.insert(index, (name, Arc::new(SuccessfulCell::listed(entry)))),
                }
            }
            batch.complete_names = Some(OriginalListing { names });
        } else {
            self.counter.at().hits.fetch_add(1, Ordering::Relaxed);
        }
        Ok(batch
            .complete_names
            .as_ref()
            .unwrap()
            .names
            .iter()
            .map(|(name, hints)| {
                let index = batch
                    .cells
                    .binary_search_by(|(n, _)| n.as_encoded_bytes().cmp(name.as_encoded_bytes()))
                    .expect("complete listing cell");
                (
                    name.clone(),
                    Arc::new(AcquisitionCell::source(batch.cells[index].1.clone(), *hints, self.counter.clone())),
                )
            })
            .collect())
    }
}
impl Drop for SourceLease<'_> {
    fn drop(&mut self) {
        // Close enumeration handles at *each* joined inspection boundary. A peer
        // still active can acquire an unrequested fact by its admitted path.
        let (charge, successful) = {
            let batch = self.batch.lock().expect("source batch poisoned");
            for (_, cell) in &batch.cells {
                cell.close_listing();
            }
            (
                batch.charge().saturating_add(self.relative.capacity()),
                batch.complete_names.is_some() || !batch.cells.is_empty(),
            )
        };
        let mut cache = self.source.cache.lock().expect("source cache poisoned");
        let entry = cache.entries.get_mut(&self.relative).expect("pinned source batch");
        entry.users -= 1;
        entry.charge = charge;
        if entry.users == 0 && (!successful || charge > cache.limits.charged_capacity || cache.limits.batches == 0 || cache.limits.charged_capacity == 0) {
            cache.entries.remove(&self.relative);
        }
        cache.trim();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::acquire::{Demand, Physical};
    use crate::test_support::{Fixture, file};
    #[test]
    fn sparse_positive_cells_never_claim_a_complete_listing_or_unqueried_absence() {
        let f = Fixture::new();
        file(&f.original.join("known"), b"known");
        file(&f.original.join("unqueried"), b"must enumerate");
        let session = SourceSession::new(&f.original);
        let lease = session.checkout(Path::new(""), SourceWork::Declared);
        let physical = Physical(session.resources());
        let entry = physical.list(&f.original).unwrap().into_iter().find(|e| e.file_name() == "known").unwrap();
        let hints = entry.hints();
        let cell = Arc::new(SuccessfulCell::listed(entry));
        let active = AcquisitionCell::source(cell.clone(), hints, lease.counter.clone());
        active.acquire(Demand::FULL, &physical).unwrap();
        lease.batch.lock().unwrap().cells.push(("known".into(), cell));
        assert!(lease.batch.lock().unwrap().complete_names.is_none());
        drop(lease);
        let lease = session.checkout(Path::new(""), SourceWork::Paired);
        let listed = lease.listed(&f.original, &physical).unwrap();
        assert_eq!(listed.iter().map(|(n, _)| n.to_str().unwrap()).collect::<Vec<_>>(), ["known", "unqueried"]);
        assert_eq!(session.diagnostics().paired.listings, 1);
        assert_eq!(session.diagnostics().paired.cache_hits, 0);
    }
    #[test]
    fn limits_charge_owned_capacities_evict_lru_never_wait_for_pins_and_drop_catalogs() {
        let f = Fixture::new();
        for name in ["a", "b", "c", "wide"] {
            std::fs::create_dir(f.original.join(name)).unwrap();
        }
        for i in 0..30 {
            file(&f.original.join("wide").join(format!("entry-{i}-{}", "long".repeat(30))), b"wide");
        }
        let session = SourceSession::with_cache_limits(
            &f.original,
            ObservationResources::default(),
            SourceCacheLimits {
                batches: 2,
                charged_capacity: 1024 * 1024,
            },
        );
        let list = |name: &str| {
            let lease = session.checkout(Path::new(name), SourceWork::Opaque);
            lease.listed(&f.original.join(name), &Physical(session.resources())).unwrap();
        };
        list("a");
        list("b");
        list("a");
        list("c");
        assert!(session.cache.lock().unwrap().entries.contains_key(Path::new("a")));
        assert!(!session.cache.lock().unwrap().entries.contains_key(Path::new("b")));
        assert_eq!(session.diagnostics().retained_batches, 2);
        assert_eq!(session.diagnostics().evictions, 1);
        let active = session.checkout(Path::new("a"), SourceWork::Opaque);
        session.set_cache_limits(SourceCacheLimits {
            batches: 0,
            charged_capacity: 0,
        });
        assert_eq!(session.diagnostics().retained_batches, 0);
        assert_eq!(session.diagnostics().active_batches, 1);
        active.listed(&f.original.join("a"), &Physical(session.resources())).unwrap();
        drop(active);
        assert!(session.cache.lock().unwrap().entries.is_empty());
        assert_eq!(session.cache.lock().unwrap().entries.capacity(), 0);
        session.set_cache_limits(SourceCacheLimits {
            batches: 2,
            charged_capacity: 1024,
        });
        for _ in 0..3 {
            list("wide");
            assert_eq!(session.diagnostics().retained_batches, 0);
            assert!(session.diagnostics().retained_charged_capacity <= 1024);
        }
        assert!(session.cache.lock().unwrap().entries.is_empty());
        let fresh = SourceSession::new(&f.original);
        assert_ne!(fresh.generation(), session.generation());
        assert_eq!(fresh.diagnostics(), SourceDiagnostics::default());
    }
}
