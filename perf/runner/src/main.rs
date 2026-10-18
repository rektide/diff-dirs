use clap::{Parser, Subcommand};
use diff_dirs_perf_runner::{Result, artifacts, execution, fixture};
use std::path::PathBuf;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    action: Action,
}

#[derive(Subcommand)]
enum Action {
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
        #[arg(long,num_args=5,action=clap::ArgAction::Append,required_unless_present="frozen")]
        control: Vec<String>,
    },
}

fn main() -> Result<()> {
    match Cli::parse().action {
        Action::Setup { root, source } => fixture::setup(&root, source.as_deref())?,
        Action::Run {
            root,
            output,
            blocks,
            seed,
            threads,
            frozen,
            control,
        } => {
            let controls = if let Some(path) = frozen {
                artifacts::frozen_controls(&path)?
            } else {
                artifacts::controls(&control)?
            };
            if !execution::run(&root, &output, &controls, blocks, seed, threads)? {
                return Err("failed samples retained; no successful campaign claim".into());
            }
        }
    }
    Ok(())
}
