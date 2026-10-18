use std::collections::BTreeMap;
use std::ffi::OsString;
use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};

use crate::Failure;

pub(crate) fn exists(path: &Path) -> Result<bool, Failure> {
    match fs::symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(Failure::io("inspect", path, error)),
    }
}

pub(crate) fn directory(path: &Path) -> Result<fs::Metadata, Failure> {
    let metadata = fs::symlink_metadata(path).map_err(|error| Failure::io("inspect directory", path, error))?;
    if !metadata.is_dir() {
        return Err(Failure::invalid("inspect directory", path, "expected real directory, not a leaf or symlink"));
    }
    Ok(metadata)
}

/// Check each ancestor before canonicalization, including the existing prefix
/// of a missing target. No creation, probes, or source writes.
pub(crate) fn root(path: &Path, allow_missing: bool) -> Result<PathBuf, Failure> {
    let absolute = std::path::absolute(path).map_err(|error| Failure::io("resolve root", path, error))?;
    let mut bound = PathBuf::new();
    for component in absolute.components() {
        match component {
            Component::ParentDir => {
                bound.pop();
            }
            Component::CurDir => {}
            component => bound.push(component),
        }
        match fs::symlink_metadata(&bound) {
            Ok(metadata) if metadata.is_dir() => {}
            Ok(_) => return Err(Failure::invalid("bind roots", &bound, "root and existing ancestors must be real directories")),
            Err(error) if allow_missing && error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(Failure::io("bind roots", &bound, error)),
        }
    }
    // Canonicalize the deepest existing ancestor, then reattach absent names.
    let mut existing = bound.as_path();
    let mut suffix = Vec::new();
    while !exists(existing)? {
        suffix.push(
            existing
                .file_name()
                .ok_or_else(|| Failure::invalid("bind roots", path, "no existing ancestor"))?
                .to_os_string(),
        );
        existing = existing.parent().ok_or_else(|| Failure::invalid("bind roots", path, "no existing ancestor"))?;
    }
    let mut canonical = fs::canonicalize(existing).map_err(|error| Failure::io("canonicalize root", existing, error))?;
    for name in suffix.into_iter().rev() {
        canonical.push(name);
    }
    Ok(canonical)
}

pub(crate) fn relative(root: &Path, dir: &Path) -> Result<PathBuf, Failure> {
    let mut path = root.to_path_buf();
    directory(&path)?;
    for component in dir.components() {
        if !matches!(component, Component::Normal(_)) {
            return Err(Failure::invalid(
                "validate record",
                dir,
                "directory must be relative with normal components only",
            ));
        }
        path.push(component);
        directory(&path)?;
    }
    Ok(path)
}

pub(crate) const CHANGED: u8 = 1;
pub(crate) const ADD_LEAF: u8 = 2;
pub(crate) const REMOVE_LEAF: u8 = 4;
pub(crate) const ADD_DIR: u8 = 8;
pub(crate) const REMOVE_DIR: u8 = 16;

pub(crate) fn names(diff: &diff_dirs::DirDiff) -> Result<Vec<(OsString, u8)>, Failure> {
    // Do this before inspecting a supplied path, not merely before writes.
    if diff.dir.components().any(|component| !matches!(component, Component::Normal(_))) {
        return Err(Failure::invalid(
            "validate record",
            &diff.dir,
            "directory must be relative with normal components only",
        ));
    }
    let mut names = BTreeMap::new();
    for (category, entries) in [
        (CHANGED, &diff.files.changed),
        (ADD_LEAF, &diff.files.added),
        (REMOVE_LEAF, &diff.files.removed),
        (ADD_DIR, &diff.dirs.added),
        (REMOVE_DIR, &diff.dirs.removed),
    ] {
        for name in entries {
            let path = Path::new(name);
            if !matches!(path.components().next(), Some(Component::Normal(_)))
                || path.components().count() != 1
                || path.as_os_str() != path.file_name().unwrap()
            {
                return Err(Failure::invalid("validate record", path, "entry name must be exactly one normal component"));
            }
            let mask = names.entry(name.clone()).or_insert(0);
            if *mask & category != 0 {
                return Err(Failure::invalid("validate record", path, "duplicate entry in category"));
            }
            *mask |= category;
        }
    }
    for (name, mask) in &names {
        if !matches!(*mask, CHANGED | ADD_LEAF | REMOVE_LEAF | ADD_DIR | REMOVE_DIR) && *mask != ADD_LEAF | REMOVE_DIR && *mask != ADD_DIR | REMOVE_LEAF {
            return Err(Failure::invalid("validate record", Path::new(name), "contradictory same-name categories"));
        }
    }
    let mut names: Vec<_> = names.into_iter().collect();
    names.sort_unstable_by(|a, b| a.0.as_encoded_bytes().cmp(b.0.as_encoded_bytes()));
    Ok(names)
}
