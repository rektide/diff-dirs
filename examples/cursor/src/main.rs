use std::io::{self, Write};
use std::num::NonZeroUsize;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Args, Parser, Subcommand};
use diff_dirs_examples_cursor::{Cursor, Descriptor, NativePath, RunOptions, read_cursor, write_cursor};
use serde_json::json;

#[derive(Parser)]
#[command(about = "Bounded read-only directory assessment with a kopacetic-rkyv continuation")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Bind disjoint real roots and assess up to N immediate directories.
    Start {
        #[arg(long)]
        source: PathBuf,
        #[arg(long)]
        target: PathBuf,
        #[arg(long, default_value = "demo")]
        label: String,
        #[command(flatten)]
        step: Step,
    },
    /// Continue stored frames/roots; ordinary resume retries a blocked tip.
    Resume {
        #[arg(long)]
        cursor_in: PathBuf,
        /// Skip the blocked scheduled directory subtree, not an individual file.
        #[arg(long)]
        skip_blocked: bool,
        #[command(flatten)]
        step: Step,
    },
}

#[derive(Args)]
struct Step {
    /// Positive count of attempted directory assessments, including failures.
    #[arg(long)]
    units: NonZeroUsize,
    #[arg(long)]
    cursor_out: PathBuf,
    /// Invocation-local injection before inspecting this relative directory.
    #[arg(long)]
    fail_at: Option<PathBuf>,
}

fn main() -> ExitCode {
    match run(Cli::parse()) {
        Ok(code) => ExitCode::from(code),
        Err(error) => {
            eprintln!("cursor: {error}");
            ExitCode::from(2)
        }
    }
}

fn run(cli: Cli) -> Result<u8, Box<dyn std::error::Error>> {
    let (mut cursor, step, skip_blocked) = match cli.command {
        Command::Start { source, target, label, step } => (Cursor::start(&source, &target, Descriptor::directory_summary(label))?, step, false),
        Command::Resume { cursor_in, skip_blocked, step } => (read_cursor(&cursor_in)?, step, skip_blocked),
    };
    let result = cursor.advance(
        step.units,
        &RunOptions {
            skip_blocked,
            fail_at: step.fail_at.as_deref().map(NativePath::new),
        },
    )?;
    // Input is fully read before publication; input and output may be the same.
    // Publication and stdout delivery are deliberately not one transaction.
    write_cursor(&step.cursor_out, &cursor)?;
    let mut out = io::BufWriter::new(io::stdout().lock());
    for event in &result.events {
        serde_json::to_writer(&mut out, event)?;
        writeln!(out)?;
    }
    serde_json::to_writer(
        &mut out,
        &json!({
            "type": "invocation", "attempted": result.attempted, "completed": result.completed,
            "skipped": result.skipped, "done": result.done, "blocked": cursor.blocked,
            "attempted_total": cursor.attempted_total, "completed_total": cursor.completed_total,
            "skipped_total": cursor.skipped_total, "final_state": cursor.final_state,
            "cursor_file": NativePath::new(&step.cursor_out),
        }),
    )?;
    writeln!(out)?;
    out.flush()?;
    Ok(if cursor.blocked.is_some() { 1 } else { 0 })
}
