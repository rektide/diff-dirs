use std::io::Write;
use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};
use diff_dirs_walk_example::{Bindings, ByteExecutor, DesiredCopy, FullInspector, Limits, Manifest, StartParts, Stop, codec, cursor};

#[derive(Parser)]
#[command(about = "Bounded Unix byte-copy/manifest walking specimen")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Start {
        #[arg(long)] source: PathBuf,
        #[arg(long)] target: PathBuf,
        #[arg(long)] reports: PathBuf,
        #[arg(long)] label: String,
        #[arg(long)] cursor: PathBuf,
        #[arg(long)] assessments: u64,
        #[arg(long)] effects: u64,
    },
    Resume {
        #[arg(long)] cursor: PathBuf,
        #[arg(long)] assessments: u64,
        #[arg(long)] effects: u64,
        #[arg(long, conflicts_with = "skip")] retry: bool,
        #[arg(long)] skip: bool,
    },
}

fn absolute(path: &Path) -> std::io::Result<PathBuf> {
    if path.is_absolute() { Ok(path.to_owned()) } else { Ok(std::env::current_dir()?.join(path)) }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (parts, manifest, output, limits, retry, skip) = match Cli::parse().command {
        Command::Start { source, target, reports, label, cursor, assessments, effects } => (
            StartParts::Fresh(Bindings::new(absolute(&source)?, absolute(&target)?, absolute(&reports)?)?),
            Manifest::builder().label(label).build(), absolute(&cursor)?, Limits { assessments, effects }, false, false,
        ),
        Command::Resume { cursor, assessments, effects, retry, skip } => {
            let output = absolute(&cursor)?;
            let saved = codec::read_cursor(&output)?;
            let manifest = Manifest::from_config(saved.manifest_config().clone())?;
            (StartParts::Restored(saved), manifest, output, Limits { assessments, effects }, retry, skip)
        }
    };
    let mut run = cursor().parts(parts).inspector(FullInspector).executor(ByteExecutor).program(DesiredCopy::v1()).manifest(manifest).build()?;
    codec::validate_cursor_path(&output, run.saved())?;
    if retry { run.retry_blocked_unit()?; }
    if skip { run.skip_blocked_unit()?; }
    run.bind_filesystem()?;
    let progress = run.drive(limits)?;
    codec::write_cursor(&output, run.saved())?;
    let stdout = std::io::stdout();
    let mut stdout = stdout.lock();
    serde_json::to_writer(&mut stdout, &serde_json::json!({ "progress": progress, "saved": run.saved() }))?;
    writeln!(stdout)?;
    if progress.stop == Stop::Blocked { std::process::exit(2); }
    Ok(())
}
