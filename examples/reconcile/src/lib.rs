//! Application-owned process/transport consumer of the production sync cursor.

pub mod native;
pub mod transport;

use std::path::{Component, Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use diff_dirs_transfer_sync::coordinated::SyncParts;
use diff_dirs_transfer_sync::{Applier, SyncRuntime, SyncTarget};
use diff_dirs_tree_compare::coordinated::LaneStatus;
use diff_dirs_tree_compare::{CheckError, Names, NoCheck, NoScope, Operations, Visitor};
use serde::{Deserialize, Serialize};

use native::NativeString;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rules {
    pub preserve: Vec<NativeString>,
    pub refuse: Option<NativeString>,
}
impl Rules {
    pub fn descriptor(&self) -> Result<String> {
        Ok(format!("reconcile-example/v1:{}", serde_json::to_string(self)?))
    }
    pub fn validate(&self) -> Result<()> {
        for p in self.preserve.iter().chain(&self.refuse) {
            let p = p.path()?;
            if p.as_os_str().as_encoded_bytes().contains(&0)
                || p.components().any(|c| !matches!(c, Component::Normal(_)))
                || p.components().collect::<PathBuf>().as_os_str() != p.as_os_str()
            {
                return Err("invalid example relative rule path".into());
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyConfig {
    pub target: String,
    pub rules: Rules,
}

pub struct Policy {
    preserve: Vec<PathBuf>,
    refuse: Option<PathBuf>,
    hooks: Arc<AtomicU64>,
}
impl Policy {
    pub fn new(rules: &Rules, hooks: Arc<AtomicU64>) -> Result<Self> {
        rules.validate()?;
        Ok(Self {
            preserve: rules.preserve.iter().map(NativeString::path).collect::<Result<_>>()?,
            refuse: rules.refuse.as_ref().map(NativeString::path).transpose()?,
            hooks,
        })
    }
}
impl Visitor for Policy {
    fn early(&self, names: &mut Names) -> std::result::Result<(), CheckError> {
        // Diagnostic only; it never affects a decision.
        if names.is_root() {
            self.hooks.fetch_add(1, Ordering::Relaxed);
        }
        for e in names.entries_mut() {
            if self.preserve.iter().any(|p| p == e.relative_path()) {
                e.preserve_tree();
            }
        }
        Ok(())
    }
    fn late(&self, ops: &mut Operations) -> std::result::Result<(), CheckError> {
        for e in ops.entries() {
            if self.refuse.as_deref() == Some(e.relative_path()) {
                return Err(e.refuse("example policy refusal"));
            }
        }
        Ok(())
    }
}

pub type Runtime = SyncRuntime<NoCheck, NoScope, diff_dirs_transfer_sync::RemoveAll, Policy>;

/// Validate every frozen rule descriptor before constructing any filesystem-bound
/// Applier, including inactive lane descriptors. Executable policies are rebuilt.
pub fn validate_config(parts: &SyncParts, config: &[PolicyConfig]) -> Result<()> {
    parts.validate()?;
    if config.len() != parts.lanes.len() {
        return Err("policy config count mismatch".into());
    }
    for (b, c) in parts.comparison.bindings.targets.iter().zip(config) {
        c.rules.validate()?;
        if b.id.0 != c.target || b.policy != c.rules.descriptor()? {
            return Err("incompatible example policy descriptor".into());
        }
    }
    Ok(())
}

pub fn runtime(parts: &SyncParts, config: &[PolicyConfig], hooks: Arc<AtomicU64>) -> Result<Runtime> {
    validate_config(parts, config)?;
    let mut targets = Vec::new();
    for (i, b) in parts.comparison.bindings.targets.iter().enumerate() {
        if parts.comparison.lanes[i].status != LaneStatus::Active {
            continue;
        }
        targets.push(SyncTarget {
            id: b.id.clone(),
            descriptor: b.policy.clone(),
            applier: Applier::new(&parts.comparison.bindings.source, &b.root, parts.copy_policies[i])?.visitor(Policy::new(&config[i].rules, hooks.clone())?),
        });
    }
    Ok(SyncRuntime::new(targets))
}

pub fn native_path(path: &Path) -> NativeString {
    NativeString::new(path.as_os_str())
}
