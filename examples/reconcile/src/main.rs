use std::ffi::OsString;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use clap::{Parser, Subcommand, ValueEnum};
use diff_dirs_examples_reconcile::{
    Policy, PolicyConfig, Result, Rules,
    native::{NativeString, assignment},
    runtime, transport,
};
use diff_dirs_transfer_sync::coordinated::{Drive, Error, Limits, Stop};
use diff_dirs_transfer_sync::{Applier, CopyPolicy, SyncCursor, SyncRuntime, SyncTarget};
use diff_dirs_tree_compare::Execution;

#[derive(Parser)]
#[command(about = "Bounded production source-to-many reconciliation and fresh-process continuation")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Clone, Copy, ValueEnum)]
enum CopyMode {
    Copy,
    ReflinkRequired,
    ReflinkOrCopy,
}
impl From<CopyMode> for CopyPolicy {
    fn from(v: CopyMode) -> Self {
        match v {
            CopyMode::Copy => Self::Copy,
            CopyMode::ReflinkRequired => Self::ReflinkRequired,
            CopyMode::ReflinkOrCopy => Self::ReflinkOrCopy,
        }
    }
}

#[derive(Subcommand)]
enum Command {
    Start {
        #[arg(long)]
        source: PathBuf,
        #[arg(long = "target", required = true)]
        targets: Vec<OsString>,
        #[arg(long, value_enum)]
        copy_policy: CopyMode,
        /// Preserve ID=RELATIVE before child metadata acquisition.
        #[arg(long)]
        preserve: Vec<OsString>,
        /// Refuse an actual ID=RELATIVE slot; useful for returned-failure examples.
        #[arg(long)]
        refuse: Vec<OsString>,
        #[arg(long)]
        cursor: PathBuf,
        #[arg(long)]
        groups: u64,
        #[arg(long)]
        applications: u64,
    },
    Resume {
        #[arg(long)]
        cursor: PathBuf,
        #[arg(long)]
        groups: u64,
        #[arg(long)]
        applications: u64,
        /// Apply only these pending IDs, then publish without closing the control
        /// window. Requires groups=0 and enough application budget.
        #[arg(long)]
        apply_only: Vec<String>,
        #[arg(long)]
        prune: Vec<String>,
        #[arg(long)]
        stop: Vec<String>,
    },
}

fn main() -> Result<()> {
    let hooks = Arc::new(AtomicU64::new(0));
    let (mut run, config, cursor, limits, apply_only, prune, stop, initial_runtime) = match Cli::parse().command {
        Command::Start {
            source,
            targets,
            copy_policy,
            preserve,
            refuse,
            cursor,
            groups,
            applications,
        } => {
            let targets: Vec<_> = targets.iter().map(|v| assignment(v)).collect::<Result<_>>()?;
            let mut config: Vec<_> = targets
                .iter()
                .map(|(id, _)| PolicyConfig {
                    target: id.clone(),
                    rules: Rules::default(),
                })
                .collect();
            for value in preserve {
                let (id, p) = assignment(&value)?;
                let c = config.iter_mut().find(|c| c.target == id).ok_or("preserve names unknown target")?;
                c.rules.preserve.push(NativeString::new(p.as_os_str()));
            }
            for value in refuse {
                let (id, p) = assignment(&value)?;
                let c = config.iter_mut().find(|c| c.target == id).ok_or("refuse names unknown target")?;
                if c.rules.refuse.replace(NativeString::new(p.as_os_str())).is_some() {
                    return Err("only one refusal per target".into());
                }
            }
            let runtime = SyncRuntime::new(
                targets
                    .into_iter()
                    .zip(&config)
                    .map(|((id, target), c)| {
                        Ok(SyncTarget {
                            id: id.as_str().into(),
                            descriptor: c.rules.descriptor()?,
                            applier: Applier::new(&source, target, copy_policy.into())?.visitor(Policy::new(&c.rules, hooks.clone())?),
                        })
                    })
                    .collect::<Result<_>>()?,
            );
            let run = SyncCursor::new(&runtime, Execution::Sequential)?;
            (
                run,
                config,
                cursor,
                Limits { groups, applications },
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Some(runtime),
            )
        }
        Command::Resume {
            cursor,
            groups,
            applications,
            apply_only,
            prune,
            stop,
        } => {
            let (parts, config) = transport::read(&cursor)?;
            (
                SyncCursor::from_parts(parts, Execution::Sequential)?,
                config,
                cursor,
                Limits { groups, applications },
                apply_only,
                prune,
                stop,
                None,
            )
        }
    };
    // Publication outside every root is checked before any tree effects.
    let cursor = transport::publication_path(&cursor, &run.parts())?;
    if !prune.is_empty() {
        let token = run.current_token().ok_or("prune needs a current control window")?;
        run.prune_here(&token, &prune.iter().map(|v| v.as_str().into()).collect::<Vec<_>>())?;
    }
    if !stop.is_empty() {
        run.stop_lanes(&stop.iter().map(|v| v.as_str().into()).collect::<Vec<_>>())?;
    }
    if !apply_only.is_empty() {
        if limits.groups != 0 || apply_only.len() as u64 > limits.applications {
            return Err("apply-only requires groups=0 and sufficient application budget".into());
        }
        let pending = run.pending();
        let mut seen = std::collections::HashSet::new();
        for target in &apply_only {
            if !seen.insert(target) || !pending.iter().any(|p| p.0 == *target) {
                return Err("apply-only requires distinct current pending targets".into());
            }
        }
    }
    let progress = if run.is_terminal() {
        Drive {
            groups: 0,
            applications: 0,
            stop: Stop::Done,
            failures: Vec::new(),
        }
    } else {
        let runtime = match initial_runtime {
            Some(runtime) => runtime,
            None => runtime(&run.parts(), &config, hooks.clone())?,
        };
        // A fresh cursor was already bound; rebinding is harmless and never
        // invokes root policy, but do not discard a just-inspected plan here.
        run.bind(&runtime)?;
        if apply_only.is_empty() {
            run.drive(&runtime, limits)?
        } else {
            let token = run.current_token().ok_or("apply-only needs a current control window")?;
            let mut failures = Vec::new();
            let mut applications = 0;
            for target in &apply_only {
                // An earlier request's shared refresh may already have failed
                // this selected peer. Do not redispatch it or lose that returned
                // failure to a global invalid-control error before publication.
                if !run.pending().iter().any(|pending| pending.0 == *target) {
                    continue;
                }
                applications += 1;
                match run.apply(&runtime, &token, &target.as_str().into()) {
                    Ok(_) => {}
                    Err(e @ (Error::Application { .. } | Error::Refresh(_))) => failures.push(e),
                    Err(e) => return Err(e.into()),
                }
            }
            Drive {
                groups: 0,
                applications,
                stop: if run.is_terminal() { Stop::Done } else { Stop::Budget },
                failures,
            }
        }
    };
    let reports = run.reports();
    let failed = !progress.failures.is_empty() || (run.is_terminal() && reports.iter().any(|r| !r.satisfied));
    let output = serde_json::json!({ "groups": progress.groups, "applications": progress.applications, "stop": format!("{:?}", progress.stop),
        "root_hooks": hooks.load(Ordering::Relaxed), "failures": progress.failures.iter().map(ToString::to_string).collect::<Vec<_>>(),
        "reports": reports.iter().map(|r| serde_json::json!({ "target": r.target.0, "satisfied": r.satisfied, "applications": r.totals.applications,
            "planned": r.totals.planned, "deviated": r.deviated, "retained": r.retained, "excluded": r.totals.excluded, "completed": transport::Work::from(&r.totals.completed), "observed": transport::Observation::from(&r.observed) })).collect::<Vec<_>>() });
    transport::write(&cursor, &run.into_parts(), config)?;
    let stdout = std::io::stdout();
    let mut stdout = stdout.lock();
    serde_json::to_writer(&mut stdout, &output)?;
    writeln!(stdout)?;
    if failed {
        std::process::exit(2);
    }
    Ok(())
}
