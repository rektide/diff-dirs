//! Existing explicit known-path leaf convenience, using the same private kernel.

use crate::{
    Applier, CopyPolicy, Failure, Progress,
    kernel::{self, BoundDestination, DirectoryEnd, DirectoryScope},
    paths,
};
use diff_dirs_tree_inspect::{Available, Check, DestinationGrant, Entry, FactOrigin, InspectionEpoch, Kind, Purpose, Side, SideFacts, SlotId};
use std::fs;
use std::path::{Path, PathBuf};

fn check<C: Check>(check: &C, root: &Path, path: &Path, side: Side, metadata: &fs::Metadata, link_target: Option<&Path>) -> Result<(), Failure> {
    if C::ENABLED {
        Entry {
            root,
            path,
            name: path.file_name().unwrap_or(path.as_os_str()),
            side,
            purpose: Purpose::Copy,
            metadata,
            link_target,
        }
        .check(check)
        .map_err(Failure::checked)?;
    }
    Ok(())
}

/// Install only caller-listed absent regular-file/symlink leaves. Real routing
/// parents are verified depth-first, never discovered by recursive enumeration.
/// Existing destinations refuse. Earlier installed leaves/created parents remain
/// on failure; private staging is not credited as installation.
pub fn seed_leaves<I, P, C>(original: impl AsRef<Path>, target: impl AsRef<Path>, leaves: I, policy: CopyPolicy, checker: C) -> Result<Progress, Failure>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
    C: Check,
{
    let binding = Applier::new_checked(original, target, policy, checker)?;
    let checker = binding.check();
    let mut leaves = leaves
        .into_iter()
        .map(|leaf| {
            let path = leaf.as_ref();
            if path.as_os_str().is_empty() || path.components().any(|c| !matches!(c, std::path::Component::Normal(_))) {
                return Err(Failure::invalid("seed relative path", path, "expected a nonempty normal relative leaf path"));
            }
            Ok(path.components().collect::<PathBuf>())
        })
        .collect::<Result<Vec<_>, _>>()?;
    fn parent_names(path: &Path) -> impl Iterator<Item = &[u8]> {
        path.parent().unwrap().components().map(|c| c.as_os_str().as_encoded_bytes())
    }
    leaves.sort_unstable_by(|a, b| {
        parent_names(a)
            .cmp(parent_names(b))
            .then_with(|| a.file_name().unwrap().as_encoded_bytes().cmp(b.file_name().unwrap().as_encoded_bytes()))
    });
    for pair in leaves.windows(2) {
        if pair[0] == pair[1] {
            return Err(Failure::invalid("seed duplicate path", &pair[1], "listed leaf occurs more than once"));
        }
    }
    let epoch = InspectionEpoch::new();
    for (root, side) in [(binding.original(), Side::Original), (binding.target(), Side::Target)] {
        let metadata = kernel::route(&epoch, root)?;
        check(checker, root, root, side, &metadata, None)?;
    }
    let mut parents = vec![PathBuf::new()];
    let mut completed = Progress::default();
    let result = (|| {
        let mut remaining = leaves.as_slice();
        while let Some(first) = remaining.first() {
            let relative = first.parent().unwrap();
            let count = remaining.iter().take_while(|leaf| leaf.parent() == Some(relative)).count();
            let (group, rest) = remaining.split_at(count);
            let shared = parents
                .last()
                .unwrap()
                .components()
                .zip(relative.components())
                .take_while(|(a, b)| a == b)
                .count();
            parents.truncate(shared + 1);
            for component in relative.components().skip(shared) {
                let parent = parents.last().unwrap();
                let relative = parent.join(component);
                let from = binding.original().join(&relative);
                let to = binding.target().join(&relative);
                let metadata = kernel::route(&epoch, &from)?;
                check(checker, binding.original(), &from, Side::Original, &metadata, None)?;
                match fs::symlink_metadata(&to) {
                    Ok(m) => {
                        if !m.is_dir() {
                            return Err(Failure::invalid("seed target ancestor", &to, "expected real directory"));
                        }
                        check(checker, binding.target(), &to, Side::Target, &m, None)?;
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                        let scope = DirectoryScope::new(&binding.target().join(parent), std::iter::empty());
                        let created = (|| {
                            scope.ensure_writable(&epoch)?;
                            fs::create_dir(&to).map_err(|e| Failure::io("seed create parent", &to, e))?;
                            epoch.invalidate_subtree(&to);
                            Ok(Progress {
                                directories_created: 1,
                                ..Progress::default()
                            })
                        })();
                        completed += scope.finish(created, DirectoryEnd::Retained, true, &epoch)?;
                        kernel::route(&epoch, &to)?;
                    }
                    Err(e) => return Err(Failure::io("seed target ancestor", &to, e)),
                }
                parents.push(relative);
            }
            let to = binding.target().join(relative);
            let scope = DirectoryScope::new(&to, group.iter().map(|p| p.file_name().unwrap()));
            let mut progress = Progress::default();
            let result = (|| {
                for (index, relative) in group.iter().enumerate() {
                    let from = binding.original().join(relative);
                    let to = binding.target().join(relative);
                    let metadata = fs::symlink_metadata(&from).map_err(|e| {
                        Failure::io(
                            if e.kind() == std::io::ErrorKind::NotFound {
                                "seed source missing"
                            } else {
                                "inspect entry"
                            },
                            &from,
                            e,
                        )
                    })?;
                    let kind = Kind::from_metadata(&metadata);
                    if !kind.is_leaf() {
                        return Err(Failure::invalid("seed source type", &from, "listed source must be a regular file or symlink"));
                    }
                    let link = if kind == Kind::Symlink {
                        Some(fs::read_link(&from).map_err(|e| Failure::io("read symlink", &from, e))?)
                    } else {
                        None
                    };
                    check(checker, binding.original(), &from, Side::Original, &metadata, link.as_deref())?;
                    if paths::exists(&to)? {
                        return Err(Failure::io(
                            "seed destination exists",
                            &to,
                            std::io::Error::new(std::io::ErrorKind::AlreadyExists, "listed destination must be absent"),
                        ));
                    }
                    if kind == Kind::File {
                        metadata.modified().map_err(|e| Failure::io("read source mtime", &from, e))?;
                    }
                    let facts = SideFacts {
                        kind,
                        origin: FactOrigin::AcquiredMetadata,
                        attributes: Available::Known(&metadata),
                        literal_link: link.as_deref().map_or(Available::NotRequested, Available::Known),
                    };
                    let slot = scope.stage_slot(SlotId(index), &to, &epoch)?;
                    let leaf = kernel::prepare_leaf(&from, facts, &slot, &to, policy)?;
                    progress += kernel::publish_leaf(leaf, &slot, &BoundDestination::new(&to, &scope, DestinationGrant::AbsentOnly, None)?, &epoch)?;
                }
                Ok(())
            })();
            let mut result = result.map(|()| Progress::default());
            kernel::add_progress(&mut result, progress);
            let complete = result.is_ok();
            completed += scope.finish(result, DirectoryEnd::Retained, complete, &epoch)?;
            remaining = rest;
        }
        Ok(())
    })();
    match result {
        Ok(()) => Ok(completed),
        Err(mut f) => {
            f.completed += completed;
            Err(f)
        }
    }
}
