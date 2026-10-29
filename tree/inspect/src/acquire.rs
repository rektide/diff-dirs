//! Retained acquisition cells and demand-masked, independently available facts.

use std::fs::Metadata;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use diff_dirs_tree_observe as observe;

use crate::{EarlyFacts, Error, FactOrigin, Facts, Kind};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Available<T> {
    NotRequested,
    Known(T),
}
impl<T> Available<T> {
    pub fn known(self) -> Option<T> {
        match self {
            Self::Known(v) => Some(v),
            Self::NotRequested => None,
        }
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Demand {
    pub attributes: bool,
    pub literal_link: bool,
}
impl Demand {
    pub const NONE: Self = Self {
        attributes: false,
        literal_link: false,
    };
    pub const FULL: Self = Self {
        attributes: true,
        literal_link: true,
    };
    pub fn union(self, other: Self) -> Self {
        Self {
            attributes: self.attributes || other.attributes,
            literal_link: self.literal_link || other.literal_link,
        }
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SideDemands {
    pub source: Demand,
    pub target: Demand,
}
impl SideDemands {
    pub fn union(self, other: Self) -> Self {
        Self {
            source: self.source.union(other.source),
            target: self.target.union(other.target),
        }
    }
}

/// Present side; absence surrounds this value. Cached stronger facts are masked.
#[derive(Clone, Copy, Debug)]
pub struct SideFacts<'a> {
    pub kind: Kind,
    pub origin: FactOrigin,
    pub attributes: Available<&'a Metadata>,
    pub literal_link: Available<&'a Path>,
}
#[derive(Clone, Copy, Debug)]
pub struct EntryFacts<'a> {
    pub source: Option<SideFacts<'a>>,
    pub target: Option<SideFacts<'a>>,
}

#[derive(Clone, Debug)]
pub(crate) struct LiveFacts {
    pub kind: Kind,
    pub origin: FactOrigin,
    pub attributes: Option<Arc<Metadata>>,
    pub literal_link: Option<Arc<PathBuf>>,
    pub legacy: Option<Arc<Facts>>,
}
impl LiveFacts {
    pub fn view(&self, demand: Demand) -> SideFacts<'_> {
        SideFacts {
            kind: self.kind,
            origin: self.origin,
            attributes: if demand.attributes {
                self.attributes.as_deref().map_or(Available::NotRequested, Available::Known)
            } else {
                Available::NotRequested
            },
            literal_link: if demand.literal_link {
                self.literal_link
                    .as_deref()
                    .map(|p| p.as_path())
                    .map_or(Available::NotRequested, Available::Known)
            } else {
                Available::NotRequested
            },
        }
    }
}
#[derive(Clone, Debug)]
pub(crate) struct EntryRuntime {
    pub source: Option<LiveFacts>,
    pub target: Option<LiveFacts>,
    pub demand: SideDemands,
}
impl EntryRuntime {
    pub fn view(&self) -> EntryFacts<'_> {
        self.masked(self.demand)
    }
    pub fn masked(&self, demand: SideDemands) -> EntryFacts<'_> {
        EntryFacts {
            source: self.source.as_ref().map(|f| f.view(demand.source)),
            target: self.target.as_ref().map(|f| f.view(demand.target)),
        }
    }
}

// Test-only substitution remains private: no public invented-facts constructor.
pub(crate) trait Acquire: Sync {
    fn list(&self, path: &Path) -> Result<Vec<observe::Entry>, Error>;
    fn metadata(&self, entry: observe::Entry) -> Result<Metadata, Error>;
    fn root_metadata(&self, path: &Path) -> Result<Metadata, Error>;
    fn path_metadata(&self, path: &Path) -> Result<Metadata, Error> {
        observe::symlink_metadata(path).map_err(Error::from)
    }
    fn link(&self, path: &Path) -> Result<PathBuf, Error>;
    #[cfg(test)]
    fn before_compare(&self, _: &Path) -> Result<(), Error> {
        Ok(())
    }
    #[cfg(test)]
    fn source_acquired(&self, _: &Arc<Facts>) {}
}
pub(crate) struct Physical<'a>(pub &'a observe::ObservationResources);
impl Acquire for Physical<'_> {
    fn list(&self, path: &Path) -> Result<Vec<observe::Entry>, Error> {
        observe::read_directory_with(path, self.0).map_err(Error::from)
    }
    fn metadata(&self, entry: observe::Entry) -> Result<Metadata, Error> {
        entry.metadata().map_err(Error::from)
    }
    fn root_metadata(&self, path: &Path) -> Result<Metadata, Error> {
        observe::symlink_metadata(path).map_err(Error::root_observation)
    }
    fn link(&self, path: &Path) -> Result<PathBuf, Error> {
        observe::read_link(path).map_err(Error::from)
    }
}
type Outcome<T> = Result<Arc<T>, Arc<Error>>;
struct CellState {
    listed: Option<observe::Entry>,
    metadata: Option<Outcome<Metadata>>,
    link: Option<Outcome<PathBuf>>,
    plain: Option<Arc<Facts>>,
    linked: Option<Arc<Facts>>,
}
/// One present entry's retained outcomes. A failed optional fact does not poison kind.
pub(crate) struct AcquisitionCell {
    path: PathBuf,
    hints: EarlyFacts,
    state: Mutex<CellState>,
}
impl AcquisitionCell {
    pub fn listed(entry: observe::Entry) -> Self {
        Self {
            path: entry.path(),
            hints: entry.hints(),
            state: Mutex::new(CellState {
                listed: Some(entry),
                metadata: None,
                link: None,
                plain: None,
                linked: None,
            }),
        }
    }
    pub fn bound(path: PathBuf, metadata: Metadata) -> Self {
        Self {
            hints: EarlyFacts::from_root_metadata(&metadata),
            path,
            state: Mutex::new(CellState {
                listed: None,
                metadata: Some(Ok(Arc::new(metadata))),
                link: None,
                plain: None,
                linked: None,
            }),
        }
    }
    pub fn hints(&self) -> EarlyFacts {
        self.hints
    }
    pub fn path(&self) -> &Path {
        &self.path
    }
    fn metadata_locked(&self, state: &mut CellState, acquire: &impl Acquire) -> Outcome<Metadata> {
        state
            .metadata
            .get_or_insert_with(|| {
                state
                    .listed
                    .take()
                    .map_or_else(|| acquire.path_metadata(&self.path), |entry| acquire.metadata(entry))
                    .map(Arc::new)
                    .map_err(Arc::new)
            })
            .clone()
    }
    fn link_locked(&self, state: &mut CellState, acquire: &impl Acquire) -> Outcome<PathBuf> {
        state
            .link
            .get_or_insert_with(|| acquire.link(&self.path).map(Arc::new).map_err(Arc::new))
            .clone()
    }
    pub fn kind(&self, acquire: &impl Acquire) -> Result<(Kind, FactOrigin), Arc<Error>> {
        if let Some(kind) = Kind::from_hint(self.hints.kind()) {
            return Ok((kind, self.hints.origin()));
        }
        let mut state = self.state.lock().expect("acquisition lock poisoned");
        self.metadata_locked(&mut state, acquire)
            .map(|m| (Kind::from_metadata(&m), FactOrigin::AcquiredMetadata))
    }
    pub fn acquire(&self, demand: Demand, acquire: &impl Acquire) -> Result<LiveFacts, Arc<Error>> {
        let (kind, origin) = self.kind(acquire)?;
        let mut state = self.state.lock().expect("acquisition lock poisoned");
        let attributes = demand.attributes.then(|| self.metadata_locked(&mut state, acquire)).transpose()?;
        let literal_link = (demand.literal_link && kind == Kind::Symlink)
            .then(|| self.link_locked(&mut state, acquire))
            .transpose()?;
        Ok(LiveFacts {
            kind,
            origin,
            attributes,
            literal_link,
            legacy: None,
        })
    }
    /// The explicit full schedule acquires attributes before readlink even when
    /// listing already knows kind, matching the old per-name observation order.
    pub fn full(&self, read_link: bool, source: bool, acquire: &impl Acquire) -> Result<LiveFacts, Arc<Error>> {
        let mut state = self.state.lock().expect("acquisition lock poisoned");
        let metadata = self.metadata_locked(&mut state, acquire)?;
        let kind = Kind::from_metadata(&metadata);
        let link = (read_link && kind == Kind::Symlink)
            .then(|| self.link_locked(&mut state, acquire))
            .transpose()?;
        let cached = if read_link { &mut state.linked } else { &mut state.plain };
        let legacy = cached
            .get_or_insert_with(|| {
                let facts = Arc::new(Facts {
                    metadata: metadata.clone(),
                    link: link.clone(),
                });
                #[cfg(test)]
                if source {
                    acquire.source_acquired(&facts);
                }
                let _ = source;
                facts
            })
            .clone();
        Ok(LiveFacts {
            kind,
            origin: if self.hints.origin() == FactOrigin::RootMetadata {
                FactOrigin::RootMetadata
            } else {
                FactOrigin::AcquiredMetadata
            },
            attributes: Some(metadata),
            literal_link: link,
            legacy: Some(legacy),
        })
    }
}
