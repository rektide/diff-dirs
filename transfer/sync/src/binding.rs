//! Physical placement/copy interpretation independent of ordinary selection.
use crate::{CopyPolicy, Failure, paths};
use std::path::{Path, PathBuf};

/// Optional regular-install read-back. Attributes are always set, independently.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Postcheck {
    #[default]
    None,
    Mtime,
}

#[derive(Clone, Debug)]
pub struct EffectBinding {
    pub(crate) original: PathBuf,
    pub(crate) target: PathBuf,
    pub(crate) policy: CopyPolicy,
    pub(crate) postcheck: Postcheck,
}
impl EffectBinding {
    pub fn new(source: impl AsRef<Path>, target: impl AsRef<Path>, policy: CopyPolicy) -> Result<Self, Failure> {
        Self::bind(source.as_ref(), target.as_ref(), policy, false)
    }
    pub(crate) fn bind(source: &Path, target: &Path, policy: CopyPolicy, replace_root: bool) -> Result<Self, Failure> {
        let original = paths::root(source, false)?;
        let target = paths::root_with_leaf(target, true, replace_root)?;
        if original.starts_with(&target) || target.starts_with(&original) {
            return Err(Failure::invalid("bind roots", &target, "source and target roots overlap"));
        }
        Ok(Self {
            original,
            target,
            policy,
            postcheck: Postcheck::None,
        })
    }
    pub fn source(&self) -> &Path {
        &self.original
    }
    pub fn target(&self) -> &Path {
        &self.target
    }
    pub fn copy_policy(&self) -> CopyPolicy {
        self.policy
    }
    pub fn with_postcheck(mut self, postcheck: Postcheck) -> Self {
        self.postcheck = postcheck;
        self
    }
    pub fn postcheck(&self) -> Postcheck {
        self.postcheck
    }
}
