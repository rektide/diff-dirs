use clap::{Parser, Subcommand};
use diff_dirs_perf_runner::{Result, artifacts, build_operation, execution, fixture, scenario::Scenario};
use std::path::PathBuf;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    action: Action,
}

#[derive(Subcommand)]
enum Action {
    /// Build this instrument from an exact archived library commit, outside all sample timers.
    BuildOperation {
        repository: PathBuf,
        revision: String,
        output: PathBuf,
        #[arg(long)]
        checks: bool,
    },
    Setup {
        root: PathBuf,
        #[arg(long)]
        source: Option<PathBuf>,
    },
    Run {
        root: PathBuf,
        output: PathBuf,
        #[arg(long, default_value_t = 8)]
        blocks: usize,
        #[arg(long, default_value_t = 1701)]
        seed: u64,
        #[arg(long, default_value_t = 0)]
        threads: usize,
        #[arg(long, conflicts_with = "control")]
        frozen: Option<PathBuf>,
        /// LABEL BINARY SHA256 MODE PROVENANCE_JSON, repeated per variant.
        #[arg(long,num_args=5,action=clap::ArgAction::Append,required_unless_present_any=["frozen","operation_builds"])]
        control: Vec<String>,
        #[arg(long,num_args=2,conflicts_with_all=["control","frozen"])]
        operation_builds: Vec<PathBuf>,
        /// New single-operation instrument, instead of frozen clean-repeat protocol.
        #[arg(long, value_enum, conflicts_with = "frozen")]
        scenario: Option<Scenario>,
        #[arg(long, default_value_t = 1)]
        destinations: usize,
    },
}

fn main() -> Result<()> {
    match Cli::parse().action {
        Action::BuildOperation {
            repository,
            revision,
            output,
            checks,
        } => build_operation::build(&repository, &revision, &output, checks)?,
        Action::Setup { root, source } => fixture::setup(&root, source.as_deref())?,
        Action::Run {
            root,
            output,
            blocks,
            seed,
            threads,
            frozen,
            control,
            scenario,
            destinations,
            operation_builds,
        } => {
            let controls = if let Some(path) = frozen {
                artifacts::frozen_controls(&path)?
            } else if !operation_builds.is_empty() {
                build_operation::controls(&operation_builds)?
            } else {
                artifacts::controls(&control)?
            };
            let task = if let Some(scenario) = scenario {
                execution::Task::Operation { scenario, destinations }
            } else {
                execution::Task::FrozenClean
            };
            if !execution::run_task(&root, &output, &controls, blocks, seed, threads, task)? {
                return Err("failed samples retained; no successful campaign claim".into());
            }
        }
    }
    Ok(())
}
