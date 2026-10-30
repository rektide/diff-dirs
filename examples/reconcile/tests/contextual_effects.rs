//! Fresh-process production contextual A-accepted/B-unstarted continuation.
use diff_dirs_examples_reconcile::{PolicyConfig, Result, Rules, transport};
use diff_dirs_transfer_sync::coordinated::{ApplicationState, Error, Limits};
use diff_dirs_transfer_sync::{Applier, CopyPolicy, SyncCursor, SyncRuntime, SyncTarget};
use diff_dirs_tree_compare::{
    Action, Choices, Container, ContextSite, Execution, Interior, InteriorInput, InteriorRole, Kind, Names, Policy, SemanticId, SizeAndLink, SourceObservation,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, LazyLock, Mutex};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Context {
    admitted: bool,
    depth: usize,
    region: bool,
    residual: u32,
}
#[derive(Clone)]
struct Region {
    residual: u32,
    events: Arc<Mutex<Vec<(PathBuf, bool)>>>,
}
static ID: LazyLock<SemanticId> = LazyLock::new(|| SemanticId::new("example.contextual-effects", 1, vec![]));
impl Policy for Region {
    type Context = Context;
    fn identity(&self) -> &SemanticId {
        &ID
    }
    fn initial_context(&self) -> Context {
        Context {
            admitted: false,
            depth: 0,
            region: false,
            residual: self.residual,
        }
    }
    fn validate_context(&self, at: ContextSite<'_>, c: &Context) -> std::result::Result<(), diff_dirs_tree_compare::CheckError> {
        let initial = at.structural_root && at.role.is_none();
        if c.admitted == initial || c.residual != self.residual || c.depth != if initial { 0 } else { at.relative.components().count() + 1 } {
            return Err("context interpretation changed".into());
        }
        Ok(())
    }
    fn names(&self, _: &Context, names: &mut Names) -> std::result::Result<(), diff_dirs_tree_compare::CheckError> {
        for entry in names.entries_mut() {
            if entry.name() == "protected" {
                entry.preserve_tree();
            }
        }
        Ok(())
    }
    fn select(&self, input: &Context, batch: &mut Choices<'_, Context>) -> std::result::Result<(), diff_dirs_tree_compare::CheckError> {
        let root = batch.info().structural_root;
        self.events.lock().unwrap().push((batch.info().relative.to_owned(), root));
        for mut entry in batch.entries_mut() {
            let relative = entry.coordinate().to_path_buf();
            let region = input.region || relative == Path::new("region");
            if !root && !region && entry.shape().source == SourceObservation::Absent {
                if entry.shape().target == Some(Kind::Directory) {
                    entry.set_action(Action::Directory {
                        container: Container::Keep,
                        descend: true,
                    });
                } else {
                    entry.preserve_tree();
                    continue;
                }
            }
            if let Some(role) = entry.action().and_then(|a| diff_dirs_tree_compare::selection::interior_role(entry.shape(), a)) {
                entry.set_interior(Interior {
                    role,
                    context: Context {
                        admitted: true,
                        depth: input.depth + 1,
                        region,
                        residual: input.residual,
                    },
                    input: InteriorInput::PhysicalDirectory,
                });
            }
        }
        Ok(())
    }
}
fn policies() -> Vec<Region> {
    [11, 22]
        .into_iter()
        .map(|residual| Region {
            residual,
            events: Arc::default(),
        })
        .collect()
}
fn runtime(base: &Path, policies: &[Region]) -> Result<SyncRuntime<Region, SizeAndLink>> {
    Ok(SyncRuntime::new(
        policies
            .iter()
            .enumerate()
            .map(|(i, p)| {
                Ok(SyncTarget {
                    id: format!("lane-{i}").as_str().into(),
                    descriptor: Rules::default().descriptor()?,
                    applier: Applier::new_policy(base.join("source"), base.join(format!("target-{i}")), CopyPolicy::Copy, p.clone(), SizeAndLink)?,
                })
            })
            .collect::<Result<Vec<_>>>()?,
    ))
}
fn configs() -> Vec<PolicyConfig> {
    (0..2)
        .map(|i| PolicyConfig {
            target: format!("lane-{i}"),
            rules: Rules::default(),
        })
        .collect()
}

#[test]
fn contextual_effect_worker() -> Result<()> {
    let Ok(mode) = std::env::var("DIFF_DIRS_CONTEXT_EFFECT_PHASE") else {
        return Ok(());
    };
    let base = PathBuf::from(std::env::var_os("DIFF_DIRS_CONTEXT_EFFECT_ROOT").ok_or("missing fixture")?);
    let policies = policies();
    let runtime = runtime(&base, &policies)?;
    let saved = base.join("effects.json");
    if mode == "start" {
        let mut cursor = SyncCursor::new(&runtime, Execution::Sequential)?;
        let group = cursor.next_group(&runtime)?.unwrap();
        assert_eq!(group.relative, Path::new(""));
        cursor.apply(&runtime, &group.token, &"lane-0".into())?;
        assert!(base.join("target-0/root").is_file());
        assert!(!base.join("target-1/root").exists());
        transport::write(&saved, &cursor.into_parts(), configs())?;
    } else {
        let (mut parts, _) = transport::read_context::<Context>(&saved)?;
        assert_eq!(parts.current.as_ref().unwrap().applications[0].state, ApplicationState::Accepted);
        assert_eq!(parts.current.as_ref().unwrap().applications[1].state, ApplicationState::Unstarted);
        assert_eq!(parts.version, 4);
        assert_eq!(parts.lanes[0].additional_retained, 1, "already accepted opaque outside/keep preservation");
        assert_eq!(parts.lanes[0].opaque_deviated, 1);
        assert_eq!(parts.lanes[1].additional_retained, 0);
        let accepted_totals = parts.lanes[0].clone();
        let a_before = fs::metadata(base.join("target-0/root"))?;
        if mode == "bad-context" {
            // Alter an opaque child context without changing the comparator or
            // schema: pure interpretation refusal must precede route access.
            let app = &mut parts.current.as_mut().unwrap().applications[1];
            let mut selected = app.decision.to_parts();
            let child = selected
                .entries
                .iter_mut()
                .find(|e| e.entry.interior.as_ref().is_some_and(|i| i.role == InteriorRole::TargetOnly))
                .unwrap();
            child.entry.interior.as_mut().unwrap().context.residual += 1;
            // Paired membership validation would catch paired corruption; choose
            // a target-only opaque entry to discriminate executable validation.
            app.decision = Arc::new(diff_dirs_tree_compare::DirectorySelection::from_parts(selected)?);
            let mut cursor = SyncCursor::from_parts(parts, Execution::Sequential)?;
            fs::rename(base.join("source"), base.join("source-away"))?;
            let refused = matches!(cursor.bind(&runtime), Err(Error::Comparison(_)));
            fs::rename(base.join("source-away"), base.join("source"))?;
            assert!(refused);
            assert!(policies.iter().all(|p| p.events.lock().unwrap().is_empty()));
            return Ok(());
        }
        let mut cursor = SyncCursor::from_parts(parts, Execution::Sequential)?;
        cursor.bind(&runtime)?;
        assert!(policies.iter().all(|p| p.events.lock().unwrap().is_empty()));
        let token = cursor.current_token().unwrap();
        cursor.apply(&runtime, &token, &"lane-1".into())?;
        assert_eq!(cursor.parts().lanes[0], accepted_totals, "a fresh process must not add A's counters again");
        assert_eq!(cursor.current().unwrap().applications[0].state, ApplicationState::Accepted);
        let drive = cursor.drive(
            &runtime,
            Limits {
                groups: 100,
                applications: 100,
            },
        )?;
        assert!(drive.failures.is_empty(), "{:?}", drive.failures);
        assert!(cursor.reports().iter().all(|r| r.satisfied));
        for (i, policy) in policies.iter().enumerate() {
            let target = base.join(format!("target-{i}"));
            assert_eq!(fs::read(target.join("root"))?, b"root");
            assert_eq!(fs::read(target.join("region/paired/deep/new"))?, b"new");
            assert!(target.join("region/protected/local").is_file());
            assert!(!target.join("region/protected/no").exists());
            assert!(!target.join("region/obsolete").exists());
            assert!(target.join("outside/keep").is_file());
            assert!(policy.events.lock().unwrap().iter().all(|(_, root)| !*root));
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            assert_eq!(a_before.ino(), fs::metadata(base.join("target-0/root"))?.ino());
            assert_eq!(fs::read_link(base.join("target-1/region/link"))?, Path::new("literal/../missing"));
        }
        transport::write(&base.join("complete.json"), &cursor.into_parts(), configs())?;
    }
    Ok(())
}
fn phase(base: &Path, mode: &str) {
    let output = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "contextual_effect_worker", "--nocapture"])
        .env("DIFF_DIRS_CONTEXT_EFFECT_PHASE", mode)
        .env("DIFF_DIRS_CONTEXT_EFFECT_ROOT", base)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{mode}: {}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
fn file(path: impl AsRef<Path>, bytes: &[u8]) {
    let p = path.as_ref();
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(p, bytes).unwrap();
}
#[test]
fn actual_contextual_effects_resume_in_a_fresh_process_without_replaying_a_or_root_admission() {
    let fixture = tempfile::tempdir_in("/tmp/opencode").unwrap();
    let base = fixture.path();
    for root in ["source", "target-0", "target-1"] {
        fs::create_dir(base.join(root)).unwrap();
    }
    file(base.join("source/root"), b"root");
    file(base.join("source/region/paired/deep/new"), b"new");
    file(base.join("source/region/protected/no"), b"excluded");
    fs::create_dir_all(base.join("target-0/region/paired/deep")).unwrap();
    for i in 0..2 {
        file(base.join(format!("target-{i}/region/protected/local")), b"local");
        file(base.join(format!("target-{i}/region/obsolete/deep/gone")), b"gone");
        file(base.join(format!("target-{i}/outside/keep")), b"keep");
    }
    #[cfg(unix)]
    std::os::unix::fs::symlink("literal/../missing", base.join("source/region/link")).unwrap();
    phase(base, "start");
    phase(base, "bad-context");
    phase(base, "resume");
    let (parts, _) = transport::read_context::<Context>(&base.join("complete.json")).unwrap();
    assert!(
        parts
            .comparison
            .lanes
            .iter()
            .all(|l| l.status == diff_dirs_tree_compare::coordinated::LaneStatus::Complete)
    );
}
