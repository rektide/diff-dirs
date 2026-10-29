//! Contextual frontier/process and data-codec evidence, not an effect-runtime test.
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{LazyLock, Mutex};

use diff_dirs_examples_reconcile::{PolicyConfig, Result, Rules, transport};
use diff_dirs_tree_compare::coordinated::{Bindings, Coordinated, Error, Head, TargetBinding, TargetPolicy};
use diff_dirs_tree_compare::{
    Choices, Comparator, ContextSite, Execution, Interior, InteriorInput, InteriorRole, Kind, Policy, SemanticId, SizeAndLink, SourceObservation, StatKey,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Context {
    admitted: bool,
    depth: usize,
    residual: u32,
}

struct Region {
    residual: u32,
    events: Mutex<Vec<(PathBuf, bool)>>,
}
static POLICY: LazyLock<SemanticId> = LazyLock::new(|| SemanticId::new("example.contextual-frontier", 1, vec![]));
static SCHEMA: LazyLock<SemanticId> = LazyLock::new(|| SemanticId::new("example.contextual-frontier.context", 1, vec![]));
impl Policy for Region {
    type Context = Context;
    fn identity(&self) -> &SemanticId {
        &POLICY
    }
    fn context_schema(&self) -> &SemanticId {
        &SCHEMA
    }
    fn initial_context(&self) -> Context {
        Context {
            admitted: false,
            depth: 0,
            residual: self.residual,
        }
    }
    fn scoped(&self) -> bool {
        false
    }
    fn validate_context(&self, at: ContextSite<'_>, context: &Context) -> std::result::Result<(), diff_dirs_tree_compare::CheckError> {
        let initial = at.structural_root && at.role.is_none();
        if context.residual != self.residual || context.admitted == initial || context.depth != if initial { 0 } else { at.relative.components().count() + 1 } {
            return Err("changed residual or invalid admitted context".into());
        }
        Ok(())
    }
    fn select(&self, input: &Context, choices: &mut Choices<'_, Context>) -> std::result::Result<(), diff_dirs_tree_compare::CheckError> {
        let root = choices.info().structural_root;
        self.events.lock().unwrap().push((choices.info().relative.to_owned(), root));
        for mut entry in choices.entries_mut() {
            if !root && input.residual == 22 && entry.coordinate().to_path_buf() == Path::new("a") {
                entry.preserve_tree();
                continue;
            }
            if entry.shape().source == SourceObservation::Present(Kind::Directory) && entry.shape().target == Some(Kind::Directory) {
                entry.set_interior(Interior {
                    role: InteriorRole::Paired,
                    context: Context {
                        admitted: true,
                        depth: input.depth + 1,
                        residual: input.residual,
                    },
                    input: InteriorInput::PhysicalDirectory,
                });
            }
        }
        Ok(())
    }
}
fn regions() -> [Region; 2] {
    [
        Region {
            residual: 11,
            events: Mutex::new(vec![]),
        },
        Region {
            residual: 22,
            events: Mutex::new(vec![]),
        },
    ]
}
fn loans<'a>(bindings: &'a Bindings<Context>, policies: &'a [Region]) -> Vec<TargetPolicy<'a, &'a Region, SizeAndLink>> {
    bindings
        .targets
        .iter()
        .zip(policies)
        .map(|(b, policy)| TargetPolicy {
            target: &b.id,
            descriptor: &b.policy,
            policy,
            comparator: SizeAndLink,
        })
        .collect()
}
fn bindings(base: &Path, policies: &[Region]) -> Result<Bindings<Context>> {
    let descriptor = Rules::default().descriptor()?;
    Ok(Bindings {
        source: base.join("source"),
        targets: policies
            .iter()
            .enumerate()
            .map(|(i, p)| {
                TargetBinding::new(
                    format!("lane-{i}").as_str().into(),
                    base.join(format!("target-{i}")),
                    descriptor.clone(),
                    p,
                    &SizeAndLink,
                )
            })
            .collect(),
    })
}
fn read(base: &Path) -> Result<diff_dirs_tree_compare::coordinated::Parts<Context>> {
    serde_json::from_slice::<transport::Comparison<Context>>(&std::fs::read(base.join("frontier.json"))?)?.decode()
}

#[test]
fn context_frontier_worker() -> Result<()> {
    let Ok(mode) = std::env::var("DIFF_DIRS_CONTEXT_PHASE") else {
        return Ok(());
    };
    let base = PathBuf::from(std::env::var_os("DIFF_DIRS_CONTEXT_ROOT").ok_or("missing process fixture")?);
    let policies = regions();
    if mode == "start" {
        let bindings = bindings(&base, &policies)?;
        let loans = loans(&bindings, &policies);
        let mut run = Coordinated::new(bindings.clone(), Execution::Sequential)?;
        run.bind(&loans)?;
        for expected in ["", "a", "a/deep"] {
            assert_eq!(run.next_group(&loans)?.unwrap().relative, Path::new(expected));
        }
        assert_eq!(run.parts().frames.len(), 3);
        assert!(matches!(&run.heads()[1].1, Head::Child { relative } if relative == Path::new("b")));
        std::fs::write(base.join("frontier.json"), serde_json::to_vec(&transport::Comparison::new(run.parts())?)?)?;
    } else if mode == "resume" {
        let parts = read(&base)?;
        let bindings = parts.bindings.clone();
        let loans = loans(&bindings, &policies);
        let mut run = Coordinated::from_parts(parts, Execution::Sequential)?;
        run.bind(&loans)?;
        assert!(policies.iter().all(|p| p.events.lock().unwrap().is_empty()));
        assert_eq!(run.next_group(&loans)?.unwrap().relative, Path::new("a/deep/next"));
        while run.next_group(&loans)?.is_some() {}
        assert!(run.parts().lanes[0].summary.is_clean());
        assert!(!run.parts().lanes[1].summary.is_complete());
        for policy in policies {
            assert!(
                policy
                    .events
                    .lock()
                    .unwrap()
                    .iter()
                    .all(|(path, root)| !*root && !["", "a", "a/deep"].iter().any(|p| path == Path::new(p)))
            );
        }
    } else {
        let mut parts = read(&base)?;
        let expected = parts.bindings.clone();
        match mode.as_str() {
            "schema" => parts.bindings.targets[0].context_schema.version += 1,
            "comparator" => {
                parts.bindings.targets[0].comparator = StatKey.identity().clone();
                parts.lanes[0].summary.comparator = Some(StatKey.identity().clone());
            }
            "residual" => parts.frames[2].remaining[0].members[0].context.residual += 1,
            _ => return Err("unknown contextual process phase".into()),
        }
        let loans = loans(&expected, &policies);
        let mut run = Coordinated::from_parts(parts, Execution::Sequential)?;
        assert!(matches!(run.bind(&loans), Err(Error::PolicyMismatch(_) | Error::Context { .. })));
        assert_eq!(run.route_metadata_checks(), 0);
        assert!(policies.iter().all(|p| p.events.lock().unwrap().is_empty()));
    }
    Ok(())
}

fn phase(base: &Path, mode: &str) {
    let output = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "context_frontier_worker", "--nocapture"])
        .env("DIFF_DIRS_CONTEXT_PHASE", mode)
        .env("DIFF_DIRS_CONTEXT_ROOT", base)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{mode}: {}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn typed_partial_frontier_restores_in_a_fresh_process_and_binds_before_io() {
    let fixture = tempfile::tempdir().unwrap();
    for root in ["source", "target-0", "target-1"] {
        for path in ["a/deep/next", "b"] {
            std::fs::create_dir_all(fixture.path().join(root).join(path)).unwrap();
        }
    }
    phase(fixture.path(), "start");
    phase(fixture.path(), "resume");
    std::fs::rename(fixture.path().join("source"), fixture.path().join("source-away")).unwrap();
    for mode in ["schema", "comparator", "residual"] {
        phase(fixture.path(), mode);
    }
}

#[test]
fn generic_unstarted_barrier_transport_preserves_the_exact_selected_contexts() -> Result<()> {
    use diff_dirs_transfer_sync::coordinated::{Application, ApplicationState, Barrier, LaneTotals, SyncParts};
    let fixture = tempfile::tempdir()?;
    for root in ["source", "target-0", "target-1"] {
        std::fs::create_dir_all(fixture.path().join(root).join("a"))?;
    }
    let policies = regions();
    let bindings = bindings(fixture.path(), &policies)?;
    let loans = loans(&bindings, &policies);
    let mut run = Coordinated::new(bindings.clone(), Execution::Sequential)?;
    run.bind(&loans)?;
    let group = run.next_group_with(&loans, |_, plan| plan)?.unwrap();
    let applications = group
        .outcomes
        .into_iter()
        .enumerate()
        .map(|(target, lane)| Application {
            target,
            decision: lane.outcome.unwrap().value.selection().clone(),
            state: ApplicationState::Unstarted,
        })
        .collect();
    let parts = SyncParts {
        version: 2,
        comparison: run.into_parts(),
        copy_policies: vec![diff_dirs_transfer_sync::CopyPolicy::Copy; 2],
        lanes: vec![LaneTotals::default(); 2],
        current: Some(Barrier { sequence: 1, applications }),
    };
    let config = bindings
        .targets
        .iter()
        .map(|b| PolicyConfig {
            target: b.id.0.clone(),
            rules: Rules::default(),
        })
        .collect();
    let wire = transport::Snapshot::new(&parts, config)?;
    let bytes = serde_json::to_vec(&wire)?;
    let (decoded, _) = serde_json::from_slice::<transport::Snapshot<Context>>(&bytes)?.decode()?;
    assert_eq!(decoded, parts);
    let mut corrupt = serde_json::from_slice::<transport::Snapshot<Context>>(&bytes)?;
    corrupt.state.current.as_mut().unwrap().applications[0].decision.input_context.residual += 1;
    assert!(corrupt.decode().is_err(), "input context must equal the accepted lane frame");
    Ok(())
}
