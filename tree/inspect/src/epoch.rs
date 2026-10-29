//! Call-local, no-follow route admission. This is never serialized.

use std::collections::HashMap;
use std::fs::Metadata;
use std::path::{Component, Path, PathBuf};
use std::sync::{
    Arc, Mutex, OnceLock,
    atomic::{AtomicU64, Ordering},
};

use crate::Error;

type Observation = Result<Arc<Metadata>, Arc<Error>>;
type Cell = Arc<OnceLock<Observation>>;

/// One externally returned call's route and attribute observations. Cache-index
/// locks are short; only a per-coordinate cell is held during acquisition. No
/// callbacks or recursive work run under either lock. Callers must invalidate
/// their successful namespace/permission changes before further admission.
#[derive(Debug, Default)]
pub struct InspectionEpoch {
    entries: Mutex<HashMap<PathBuf, Cell>>,
    checks: AtomicU64,
}

impl InspectionEpoch {
    pub fn new() -> Self {
        Self::default()
    }

    /// Admit every absolute ancestor, not merely the final component. Symlinks
    /// (including directory links) are refused. Relative/non-normal routes refuse.
    pub fn directory(&self, path: &Path) -> Observation {
        if !path.is_absolute() || path.components().any(|c| matches!(c, Component::ParentDir | Component::CurDir)) {
            return Err(Arc::new(Error::Io {
                path: path.to_owned(),
                operation: "validate inspection route",
                source: std::io::Error::new(std::io::ErrorKind::InvalidInput, "expected absolute normal directory route"),
            }));
        }
        let mut prefix = PathBuf::new();
        let mut last = None;
        for part in path.components() {
            prefix.push(part);
            let cell = {
                let mut entries = self.entries.lock().expect("route index poisoned");
                entries.entry(prefix.clone()).or_default().clone()
            };
            let value = cell
                .get_or_init(|| {
                    self.checks.fetch_add(1, Ordering::Relaxed);
                    std::fs::symlink_metadata(&prefix)
                        .map_err(|source| Error::Io {
                            path: prefix.clone(),
                            operation: "validate inspection route",
                            source,
                        })
                        .and_then(|m| {
                            if m.is_dir() {
                                Ok(Arc::new(m))
                            } else {
                                Err(Error::NotDirectory { path: prefix.clone() })
                            }
                        })
                        .map_err(Arc::new)
                })
                .clone()?;
            last = Some(value);
        }
        Ok(last.expect("absolute directory has root component"))
    }

    /// Invalidate positive and negative route entries below a namespace change.
    /// Parent attributes change too; surviving leases keep their original modes.
    pub fn invalidate_subtree(&self, path: &Path) {
        let mut entries = self.entries.lock().expect("route index poisoned");
        entries.retain(|p, _| !p.starts_with(path) && Some(p.as_path()) != path.parent());
    }

    pub fn invalidate_attributes(&self, path: &Path) {
        self.entries.lock().expect("route index poisoned").remove(path);
    }

    pub fn metadata_checks(&self) -> u64 {
        self.checks.load(Ordering::Relaxed)
    }
}
