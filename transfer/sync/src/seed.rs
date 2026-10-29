//! Compatibility seed is a CreateAbsent manifest, not another executor.
use crate::{
    CopyPolicy, EffectBinding, Failure, ListedCandidate, ListedGuard, ListedIntent, ListedTarget, OwnedLeafSet, PreparedLeafList, Progress, reconcile_listed,
};
use diff_dirs_tree_compare::coordinated::TargetId;
use diff_dirs_tree_inspect::{Check, CheckError, DeclaredCandidate, Demand, Entry, EntryFacts, Kind, Purpose, Side, SideDemands, SourceSession};
use std::{
    collections::HashSet,
    fs::Metadata,
    path::{Path, PathBuf},
    sync::Mutex,
};

struct CheckedListedGuard<C> {
    check: C,
    directories: Mutex<HashSet<(u8, PathBuf)>>,
}
impl<C: Check> CheckedListedGuard<C> {
    fn visit(&self, root: &Path, relative: &Path, side: Side, metadata: &Metadata, link: Option<&Path>) -> Result<(), CheckError> {
        if !C::ENABLED {
            return Ok(());
        }
        let key = (side as u8, relative.to_owned());
        if metadata.is_dir() && self.directories.lock().unwrap().contains(&key) {
            return Ok(());
        }
        let path = root.join(relative);
        self.check
            .check(Entry {
                root,
                path: &path,
                name: path.file_name().unwrap_or(path.as_os_str()),
                side,
                purpose: Purpose::Copy,
                metadata,
                link_target: link,
            })
            .map_err(|source| -> CheckError { Box::new(diff_dirs_tree_inspect::Error::Refused { path: path.clone(), source }) })?;
        if metadata.is_dir() {
            self.directories.lock().unwrap().insert(key);
        }
        Ok(())
    }
}
impl<C: Check> ListedGuard for CheckedListedGuard<C> {
    fn demand(&self, candidate: ListedCandidate<'_>) -> SideDemands {
        if !C::ENABLED {
            return SideDemands::default();
        }
        match candidate.observed {
            DeclaredCandidate::Route(route) => {
                if route.side == Side::Original {
                    SideDemands {
                        source: Demand {
                            attributes: true,
                            literal_link: false,
                        },
                        target: Demand::NONE,
                    }
                } else {
                    SideDemands {
                        target: Demand {
                            attributes: true,
                            literal_link: false,
                        },
                        source: Demand::NONE,
                    }
                }
            }
            DeclaredCandidate::Entry(entry) => SideDemands {
                source: if entry.desired.is_some() { Demand::FULL } else { Demand::NONE },
                target: Demand {
                    attributes: entry.shape.target == Some(Kind::Directory),
                    literal_link: false,
                },
            },
        }
    }
    fn check(&self, candidate: ListedCandidate<'_>, facts: EntryFacts<'_>) -> Result<(), CheckError> {
        if !C::ENABLED {
            return Ok(());
        }
        match candidate.observed {
            DeclaredCandidate::Route(route) => {
                let (root, facts) = if route.side == Side::Original {
                    (route.info.source_root, facts.source)
                } else {
                    (route.info.target_root, facts.target)
                };
                self.visit(root, route.relative, route.side, facts.unwrap().attributes.known().unwrap(), None)
            }
            DeclaredCandidate::Entry(entry) => {
                let relative = entry.coordinate.to_path_buf();
                if let Some(source) = facts.source {
                    self.visit(
                        entry.info.source_root,
                        &relative,
                        Side::Original,
                        source.attributes.known().unwrap(),
                        source.literal_link.known(),
                    )?;
                }
                if let Some(target) = facts.target
                    && target.kind == Kind::Directory
                {
                    self.visit(entry.info.target_root, &relative, Side::Target, target.attributes.known().unwrap(), None)?;
                }
                Ok(())
            }
        }
    }
}

/// Install only declared absent leaves through the complete manifest executor.
/// Existing routing directories may be retained; occupied leaves always refuse.
///
/// The entire immediate declared batch (including later sibling source/guard
/// failures) is now prepared before its first effect, rather than committing a
/// leaf prefix within that batch. Earlier completed batches remain visible.
/// Compatibility Check still sees actual full source facts/literal links and
/// actual source/existing-target routing directories once in this coarse call.
pub fn seed_leaves<I, P, C>(original: impl AsRef<Path>, target: impl AsRef<Path>, leaves: I, policy: CopyPolicy, checker: C) -> Result<Progress, Failure>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
    C: Check,
{
    let desired = PreparedLeafList::new(leaves)?;
    let owned = OwnedLeafSet::new(Vec::<PathBuf>::new())?;
    let binding = EffectBinding::new(original, target, policy)?;
    let guard = CheckedListedGuard {
        check: checker,
        directories: Mutex::default(),
    };
    // Retain seed's existing real-root/check semantics even for an empty list.
    let epoch = diff_dirs_tree_inspect::InspectionEpoch::new();
    for (root, side) in [(binding.source(), Side::Original), (binding.target(), Side::Target)] {
        let metadata = crate::kernel::route(&epoch, root)?;
        guard
            .visit(root, Path::new(""), side, &metadata, None)
            .map_err(|source| Failure::checked(diff_dirs_tree_inspect::Error::Refused { path: root.to_owned(), source }))?;
    }
    let id = TargetId("seed".into());
    let report = reconcile_listed(
        &SourceSession::new(binding.source()),
        &desired,
        &[ListedTarget {
            id: &id,
            binding: &binding,
            owned: &owned,
            intent: ListedIntent::CreateAbsent,
            guard: &guard,
        }],
    )
    .map_err(|error| {
        Failure::io(
            "bind seed manifest",
            binding.target(),
            std::io::Error::new(std::io::ErrorKind::InvalidInput, error),
        )
    })?;
    report.outcomes.into_iter().next().unwrap().outcome.map(|r| r.completed)
}
