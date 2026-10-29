//! Public binding/refresh boundary. The runner and physical kernel stay private.

use crate::{
    Applier, CopyPolicy, Failure, Progress, Report, kernel,
    opaque::{BatchPlace, EffectCall, LaneCall, RecursionOwner},
    paths,
};
use diff_dirs_tree_inspect::{
    Action, Check, Comparator, Container, Demand, DirectorySelection, EntrySelectionRef, Execution, ExecutionDemand, FullVisitor, InspectionEpoch, Kind,
    Policy as SelectionPolicy, PreparedDirectory, Scope, SideDemands, SlotId, SourceSession, Visitor,
};
use std::path::Path;
use std::sync::Arc;

/// Finite additional physical effect inputs. Unix unlink needs kind alone; link
/// subtype attributes are an explicit Windows requirement, not full-fact padding.
#[derive(Clone, Copy, Debug, Default)]
pub struct PhysicalExecutionDemand;
impl ExecutionDemand for PhysicalExecutionDemand {
    fn for_entry<C>(&self, entry: EntrySelectionRef<'_, C>) -> SideDemands {
        let mut demand = SideDemands::default();
        match entry.action {
            Action::InstallLeaf | Action::ReplaceWithLeaf => {
                demand.source = match entry.source.kind() {
                    Some(Kind::File) => Demand {
                        attributes: true,
                        literal_link: false,
                    },
                    Some(Kind::Symlink) => Demand {
                        attributes: cfg!(windows),
                        literal_link: true,
                    },
                    _ => Demand::NONE,
                }
            }
            Action::Directory {
                container: Container::Create | Container::CreateIfNeeded,
                ..
            }
            | Action::ReplaceWithDirectory { .. } => demand.source.attributes = true,
            _ => {}
        }
        if cfg!(windows) && entry.target == Some(Kind::Symlink) && matches!(entry.action, Action::RemoveLeaf | Action::ReplaceWithDirectory { .. }) {
            demand.target.attributes = true;
        }
        demand
    }
}

/// No observation, scope setup or mutation. All required fields of ALL siblings
/// (including fallible modified()) are extracted before an immediate effect.
pub(crate) fn require_execution_inputs<C>(batch: &PreparedDirectory<C>) -> Result<(), Failure> {
    for i in 0..batch.selection().len() {
        let entry = batch.selection().entry(SlotId(i));
        let needed = PhysicalExecutionDemand.for_entry(entry);
        let facts = batch.facts(SlotId(i));
        let relative = if batch.selection().is_root() {
            batch.selection().relative().to_owned()
        } else {
            batch.selection().relative().join(entry.name())
        };
        for (required, actual) in [(needed.source, facts.source), (needed.target, facts.target)] {
            if (required.attributes && actual.and_then(|s| s.attributes.known()).is_none())
                || (required.literal_link && actual.and_then(|s| s.literal_link.known()).is_none())
            {
                return Err(Failure::invalid(
                    "require execution inputs",
                    &relative,
                    "prepared batch lacks selected execution facts",
                ));
            }
        }
        if matches!(entry.action, Action::InstallLeaf | Action::ReplaceWithLeaf) && entry.source.kind() == Some(Kind::File) {
            facts
                .source
                .unwrap()
                .attributes
                .known()
                .unwrap()
                .modified()
                .map_err(|e| Failure::io("read source mtime", &batch.source_root().join(&relative), e))?;
        }
    }
    Ok(())
}

pub(crate) fn inspection_failure(error: Arc<diff_dirs_tree_inspect::Error>) -> Failure {
    match Arc::try_unwrap(error) {
        Ok(error) => Failure::checked(error),
        Err(error) => Failure {
            planned: 0,
            policy: Box::default(),
            primary: Box::new(crate::Issue {
                operation: "check",
                path: error.path().to_owned(),
                cause: Box::new(error),
            }),
            secondary: Box::default(),
            completed: Progress::default(),
            observed: None,
        },
    }
}
pub(crate) fn execution(parallel: bool) -> Execution {
    #[cfg(feature = "parallel")]
    if parallel {
        return Execution::Parallel;
    }
    let _ = parallel;
    Execution::Sequential
}

impl<P: SelectionPolicy, M: Comparator> Applier<P, M> {
    pub(crate) fn apply_walk_ready(
        &self,
        source: &SourceSession,
        epoch: &InspectionEpoch,
        prepared: PreparedDirectory<P::Context>,
        parallel: bool,
    ) -> Result<Report, Failure> {
        if prepared.selection().coverage().application_need == diff_dirs_tree_inspect::ApplicationNeed::None {
            return Ok(Report {
                planned: 0,
                completed: Progress::default(),
                policy: crate::Policy {
                    satisfied: true,
                    ..crate::Policy::default()
                },
                observed: None,
            });
        }
        self.apply_ready_accounted(source, epoch, prepared, parallel).only_additions()
    }
    pub(crate) fn apply_ready_accounted(
        &self,
        source: &SourceSession,
        epoch: &InspectionEpoch,
        prepared: PreparedDirectory<P::Context>,
        parallel: bool,
    ) -> crate::opaque::ApplicationResult {
        let lane = LaneCall::new(self);
        let call = EffectCall {
            source,
            epoch,
            execution: execution(parallel),
        };
        let relative = prepared.selection().relative().to_owned();
        let from = self.original.join(&relative);
        let to = self.target.join(&relative);
        let result = lane.apply_ready(
            &call,
            prepared,
            BatchPlace {
                relative: &relative,
                source_directory: Some(&from),
                observed_target_directory: Some(&to),
                output_directory: &to,
            },
            RecursionOwner::PairedFrontier,
            None,
        );
        lane.finish_accounted(result)
    }
    pub fn apply_report(&self, diff: &diff_dirs_tree_inspect::DirDiff) -> Result<Report, Failure> {
        self.apply_public(diff, false, None)
    }

    pub(crate) fn apply_public(
        &self,
        diff: &diff_dirs_tree_inspect::DirDiff,
        parallel: bool,
        expected: Option<&Arc<DirectorySelection<P::Context>>>,
    ) -> Result<Report, Failure> {
        let names = paths::names(diff)?;
        let source = SourceSession::new(&self.original);
        let epoch = InspectionEpoch::new();
        let call = EffectCall {
            source: &source,
            epoch: &epoch,
            execution: execution(parallel),
        };
        let lane = LaneCall::new(self);
        let mut excluded = 0;
        let result = (|| {
            kernel::route(&epoch, &self.original)?;
            kernel::route(&epoch, &self.target)?;
            let mut relative = std::path::PathBuf::new();
            let mut prepared = lane.inspect(
                &call,
                &BatchPlace {
                    relative: &relative,
                    source_directory: Some(&self.original),
                    observed_target_directory: Some(&self.target),
                    output_directory: &self.target,
                },
                &self.selection.initial_context(),
                true,
                true,
            )?;
            excluded += prepared.selection().coverage().excluded;
            if prepared.selection().is_root() {
                if expected.is_some() {
                    return Err(Failure::invalid("apply plan", &self.target, "root descent is not admitted"));
                }
                return Ok(Progress::default());
            }
            for component in diff.dir.components() {
                let selected = prepared.selection();
                let Some(entry) = selected.entries().find(|e| e.name() == component.as_os_str()) else {
                    if selected.excluded_names().iter().any(|n| n == component.as_os_str()) {
                        return if expected.is_none() {
                            Ok(Progress::default())
                        } else {
                            Err(Failure::invalid("apply plan", &relative, "ancestor is preserved"))
                        };
                    }
                    return Err(Failure::invalid("validate record ancestor", &relative, "ancestor missing"));
                };
                if entry.action == Action::PreserveTree {
                    return if expected.is_none() {
                        Ok(Progress::default())
                    } else {
                        Err(Failure::invalid("apply plan", &relative, "ancestor is preserved"))
                    };
                }
                if entry.source.kind() != Some(Kind::Directory) || entry.target != Some(Kind::Directory) {
                    return Err(Failure::invalid(
                        "validate record ancestor",
                        &relative,
                        "both ancestors must be real directories",
                    ));
                }
                let Some(interior) = &entry.interior else {
                    return Ok(Progress::default());
                };
                let context = interior.context.clone();
                relative.push(component);
                let from = self.original.join(&relative);
                let to = self.target.join(&relative);
                prepared = lane.inspect(
                    &call,
                    &BatchPlace {
                        relative: &relative,
                        source_directory: Some(&from),
                        observed_target_directory: Some(&to),
                        output_directory: &to,
                    },
                    &context,
                    false,
                    true,
                )?;
                excluded += prepared.selection().coverage().excluded;
            }
            if expected.is_some_and(|e| prepared.selection() != e) {
                return Err(Failure::invalid(
                    "apply plan",
                    &relative,
                    "batch does not match current selected interpretation",
                ));
            }
            for (name, mask) in &names {
                if let Some(entry) = prepared.selection().entries().find(|e| e.name() == name) {
                    if shape(&entry) != Some(*mask) {
                        return Err(Failure::invalid("validate record", &relative.join(name), "entry type does not match record"));
                    }
                } else if !prepared.selection().excluded_names().contains(name) {
                    return Err(Failure::invalid("validate record", &relative.join(name), "entry missing"));
                }
            }
            let from = self.original.join(&relative);
            let to = self.target.join(&relative);
            excluded -= prepared.selection().coverage().excluded;
            lane.apply_ready(
                &call,
                prepared,
                BatchPlace {
                    relative: &relative,
                    source_directory: Some(&from),
                    observed_target_directory: Some(&to),
                    output_directory: &to,
                },
                RecursionOwner::PairedFrontier,
                Some(&names),
            )
        })();
        lane.add_excluded(excluded);
        lane.finish(result)
    }

    pub fn preview(&self) -> Result<Report, Failure> {
        let source = SourceSession::new(&self.original);
        let epoch = InspectionEpoch::new();
        let lane = LaneCall::new(self);
        let call = EffectCall {
            source: &source,
            epoch: &epoch,
            execution: Execution::Sequential,
        };
        let target = paths::exists(&self.target)?.then_some(self.target.as_path());
        let prepared = lane.inspect(
            &call,
            &BatchPlace {
                relative: Path::new(""),
                source_directory: Some(&self.original),
                observed_target_directory: target,
                output_directory: &self.target,
            },
            &self.selection.initial_context(),
            true,
            false,
        )?;
        lane.finish(lane.preview(&call, prepared).map(|_| Progress::default()))
    }

    pub(crate) fn initialize_reported(&self, parallel: bool) -> Result<Report, Failure> {
        if paths::exists(&self.target)? {
            return Err(Failure::invalid("initialize", &self.target, "target already exists"));
        }
        self.root_reported(&SourceSession::new(&self.original), false, parallel)
    }
    pub(crate) fn replace_root_reported(&self, parallel: bool) -> Result<Report, Failure> {
        if !self.replace_root {
            return Err(Failure::invalid("replace root", &self.target, "requires explicit replacing root binding"));
        }
        self.root_reported(&SourceSession::new(&self.original), true, parallel)
    }
    fn root_reported(&self, source: &SourceSession, present: bool, parallel: bool) -> Result<Report, Failure> {
        let epoch = InspectionEpoch::new();
        let lane = LaneCall::new(self);
        let call = EffectCall {
            source,
            epoch: &epoch,
            execution: execution(parallel),
        };
        kernel::route(&epoch, self.target.parent().expect("bound root has parent"))?;
        let place = BatchPlace {
            relative: Path::new(""),
            source_directory: Some(&self.original),
            observed_target_directory: present.then_some(self.target.as_path()),
            output_directory: &self.target,
        };
        let prepared = lane.inspect(&call, &place, &self.selection.initial_context(), true, true)?;
        lane.finish(lane.apply_ready(&call, prepared, place, RecursionOwner::ThisCall, None))
    }
    pub(crate) fn whole_root(&self, source: &SourceSession, parallel: bool) -> Result<Option<Report>, Failure> {
        match std::fs::symlink_metadata(&self.target) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => self.root_reported(source, false, parallel).map(Some),
            Err(e) => Err(Failure::io("inspect target root", &self.target, e)),
            Ok(m) if m.is_dir() => Ok(None),
            Ok(_) if self.replace_root => self.root_reported(source, true, parallel).map(Some),
            Ok(_) => Err(Failure::invalid("replace root", &self.target, "requires explicit replacing root binding")),
        }
    }
}

fn shape<C>(entry: &EntrySelectionRef<'_, C>) -> Option<u8> {
    entry.comparison.legacy()?;
    let source = entry.source.kind();
    let target = entry.target;
    if source.is_some_and(Kind::is_leaf) && target.is_some_and(Kind::is_leaf) {
        return Some(paths::CHANGED);
    }
    let mut mask = 0;
    if let Some(kind) = source {
        mask |= if kind == Kind::Directory { paths::ADD_DIR } else { paths::ADD_LEAF };
    }
    if let Some(kind) = target {
        mask |= if kind == Kind::Directory { paths::REMOVE_DIR } else { paths::REMOVE_LEAF };
    }
    Some(mask)
}

/// Explicit selected removal of a real container's contents; no missing donor.
pub fn clear<S: Scope, C: Check>(target: impl AsRef<Path>, scope: S, check: C) -> Result<Report, Failure> {
    clear_with_visitor(target, diff_dirs_tree_inspect::CheckedScope::new(check, scope))
}
pub fn clear_with_visitor<V: Visitor>(target: impl AsRef<Path>, visitor: V) -> Result<Report, Failure> {
    let target = paths::root(target.as_ref(), false)?;
    let binding = Applier {
        original: target.clone(),
        target: target.clone(),
        policy: CopyPolicy::Copy,
        selection: FullVisitor(visitor),
        comparator: diff_dirs_tree_inspect::StatKey,
        replace_root: false,
    };
    let source = SourceSession::new(&target);
    let epoch = InspectionEpoch::new();
    let lane = LaneCall::new(&binding);
    let call = EffectCall {
        source: &source,
        epoch: &epoch,
        execution: Execution::Sequential,
    };
    let place = BatchPlace {
        relative: Path::new(""),
        source_directory: None,
        observed_target_directory: Some(&target),
        output_directory: &target,
    };
    let prepared = lane.inspect(&call, &place, &(), true, true)?;
    lane.finish(lane.apply_ready(&call, prepared, place, RecursionOwner::ThisCall, None))
}

#[cfg(all(test, feature = "parallel"))]
#[path = "apply_parallel_tests.rs"]
pub(crate) mod tests;

#[cfg(test)]
mod readiness_tests {
    use super::*;
    use diff_dirs_tree_inspect::{
        Choices, DestinationGrant, DirectoryAdmission, DirectoryRequest, InstallAuthority, InstallIntent, NoExecutionDemand, SemanticId, ShapeChoices,
        SizeAndLink, TargetInspection, inspect_group,
    };
    struct Renew;
    impl SelectionPolicy for Renew {
        type Context = ();
        fn identity(&self) -> &SemanticId {
            static ID: std::sync::LazyLock<SemanticId> = std::sync::LazyLock::new(|| SemanticId::new("test.sparse-renew", 1, vec![]));
            &ID
        }
        fn initial_context(&self) {}
        fn shape(&self, _: &(), batch: &mut ShapeChoices<'_, ()>) -> Result<(), diff_dirs_tree_inspect::CheckError> {
            for mut e in batch.entries_mut() {
                if e.shape().source.kind() == Some(Kind::File) {
                    e.omit_comparison();
                }
            }
            Ok(())
        }
        fn select(&self, _: &(), batch: &mut Choices<'_, ()>) -> Result<(), diff_dirs_tree_inspect::CheckError> {
            for mut e in batch.entries_mut() {
                if e.shape().source.kind() == Some(Kind::File) {
                    e.set_action(Action::InstallLeaf);
                    e.set_install(InstallAuthority {
                        intent: InstallIntent::Renew,
                        destination: DestinationGrant::AbsentOnly,
                    });
                }
            }
            Ok(())
        }
    }
    #[test]
    fn a_prepared_selection_without_required_inputs_refuses_without_fallback_observation_or_effect() {
        let scratch = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.test-agent/sync-tests");
        std::fs::create_dir_all(&scratch).unwrap();
        let fixture = tempfile::tempdir_in(std::fs::canonicalize(scratch).unwrap()).unwrap();
        let from = fixture.path().join("source");
        let to = fixture.path().join("target");
        std::fs::create_dir(&from).unwrap();
        std::fs::create_dir(&to).unwrap();
        std::fs::write(from.join("a"), b"a").unwrap();
        std::fs::write(from.join("z"), b"z").unwrap();
        let source = SourceSession::new(&from);
        let epoch = InspectionEpoch::new();
        let prepared = inspect_group(
            &source,
            DirectoryRequest {
                relative: Path::new(""),
                source_directory: Some(&from),
                purpose: diff_dirs_tree_inspect::Purpose::Compare,
                admission: DirectoryAdmission::RootGated,
            },
            &[TargetInspection {
                target: 0,
                logical_root: &to,
                physical_directory: Some(&to),
                context: &(),
                policy: &Renew,
                comparator: &SizeAndLink,
            }],
            &NoExecutionDemand,
            &epoch,
            Execution::Sequential,
        )
        .unwrap()
        .pop()
        .unwrap()
        .outcome
        .unwrap();
        assert!(prepared.facts(SlotId(0)).source.unwrap().attributes.known().is_none());
        let binding = Applier::new_policy(&from, &to, CopyPolicy::Copy, Renew, SizeAndLink).unwrap();
        let result = binding.apply_ready_accounted(&source, &epoch, prepared, false);
        assert_eq!(result.additions.deviated, 0);
        assert_eq!(result.additions.retained, 0);
        let f = result.result.unwrap_err();
        assert_eq!(f.policy.deviated, 2);
        assert_eq!(f.policy.retained, 0);
        assert_eq!(f.primary.operation, "require execution inputs");
        assert_eq!(f.completed, Progress::default());
        assert_eq!(std::fs::read_dir(to).unwrap().count(), 0);
    }
}
