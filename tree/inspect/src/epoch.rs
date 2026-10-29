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

fn normalized(path: &Path) -> PathBuf {
    let mut result = PathBuf::new();
    for part in path.components() {
        match part {
            Component::ParentDir => {
                result.pop();
            }
            Component::CurDir => {}
            part => result.push(part),
        }
    }
    result
}

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
    /// (including directory links) are refused, even immediately before `..`.
    /// Relative routes refuse; real parent components are admitted before ascent.
    pub fn directory(&self, path: &Path) -> Observation {
        if !path.is_absolute() {
            return Err(Arc::new(Error::Io {
                path: path.to_owned(),
                operation: "validate inspection route",
                source: std::io::Error::new(std::io::ErrorKind::InvalidInput, "expected absolute normal directory route"),
            }));
        }
        let mut prefix = PathBuf::new();
        let mut last = None;
        for part in path.components() {
            match part {
                Component::ParentDir => {
                    prefix.pop();
                }
                Component::CurDir => {}
                part => prefix.push(part),
            }
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
        let path = normalized(path);
        let mut entries = self.entries.lock().expect("route index poisoned");
        entries.retain(|p, _| !p.starts_with(&path) && Some(p.as_path()) != path.parent());
    }

    pub fn invalidate_attributes(&self, path: &Path) {
        self.entries.lock().expect("route index poisoned").remove(&normalized(path));
    }

    pub fn metadata_checks(&self) -> u64 {
        self.checks.load(Ordering::Relaxed)
    }

    /// Inspect-owned acquisition reuse, never a public metadata certificate.
    /// The actual no-follow observation may replace an older attribute result;
    /// every ancestor must still have been admitted within this same epoch.
    pub(crate) fn acquired_directory(&self, path: &Path, metadata: &Metadata) -> Result<(), Arc<Error>> {
        if !metadata.is_dir() {
            return Ok(());
        }
        let absolute = std::path::absolute(path).map_err(|source| {
            Arc::new(Error::Io {
                path: path.to_owned(),
                operation: "resolve acquired route",
                source,
            })
        })?;
        if let Some(parent) = absolute.parent() {
            self.directory(parent)?;
        }
        let cell = Arc::new(OnceLock::new());
        let _ = cell.set(Ok(Arc::new(metadata.clone())));
        self.entries.lock().expect("route index poisoned").insert(normalized(&absolute), cell);
        Ok(())
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    #[test]
    fn absolute_ancestry_negative_entries_and_namespace_permission_invalidation() {
        use std::os::unix::fs::{PermissionsExt, symlink};
        let root = tempfile_root();
        std::fs::create_dir(root.join("real")).unwrap();
        symlink(root.join("real"), root.join("link")).unwrap();
        std::fs::create_dir(root.join("real/child")).unwrap();
        let epoch = InspectionEpoch::new();
        assert!(matches!(
            epoch.directory(&root.join("link/child")).unwrap_err().as_ref(),
            Error::NotDirectory { .. }
        ));
        let real = root.join("real");
        let first = epoch.directory(&real).unwrap();
        let count = epoch.metadata_checks();
        assert!(Arc::ptr_eq(&first, &epoch.directory(&real).unwrap()));
        assert_eq!(count, epoch.metadata_checks());
        let missing = real.join("missing");
        assert!(matches!(epoch.directory(&missing).unwrap_err().as_ref(),Error::Io { source,.. } if source.kind()==std::io::ErrorKind::NotFound));
        std::fs::create_dir(&missing).unwrap();
        epoch.invalidate_subtree(&missing);
        assert!(epoch.directory(&missing).is_ok());
        std::fs::set_permissions(&real, std::fs::Permissions::from_mode(0o755)).unwrap();
        epoch.invalidate_attributes(&real);
        assert_eq!(epoch.directory(&real).unwrap().permissions().mode() & 0o777, 0o755);
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn inspected_directory_metadata_is_reused_without_child_route_stat() {
        let root = tempfile_root();
        let child = root.join("child");
        std::fs::create_dir(&child).unwrap();
        let epoch = InspectionEpoch::new();
        epoch.directory(&root).unwrap();
        let before = epoch.metadata_checks();
        epoch.acquired_directory(&child, &std::fs::symlink_metadata(&child).unwrap()).unwrap();
        epoch.directory(&child).unwrap();
        assert_eq!(before, epoch.metadata_checks());
        std::fs::remove_dir_all(root).unwrap();
    }
    fn tempfile_root() -> PathBuf {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let root = std::path::absolute(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.test-agent/epoch-tests")).unwrap();
        std::fs::create_dir_all(&root).unwrap();
        let path = root.join(format!("{}-{}", std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)));
        std::fs::create_dir(&path).unwrap();
        path
    }
}
