mod args;
mod render;

use std::error::Error;
use std::io::{self, BufWriter, Write};
use std::process::ExitCode;

use args::Command;

fn main() -> ExitCode {
    match run() {
        Ok(code) => ExitCode::from(code),
        Err(error) => {
            eprintln!("diff-dirs: {error}");
            ExitCode::from(2)
        }
    }
}

fn run() -> Result<u8, Box<dyn Error>> {
    let command = args::parse(std::env::args_os().skip(1)).map_err(|message| io::Error::new(io::ErrorKind::InvalidInput, message))?;
    let mut out = BufWriter::new(io::stdout().lock());
    match command {
        Command::Help => writeln!(out, "{}", args::HELP)?,
        Command::Version => writeln!(out, "diff-dirs {}", env!("CARGO_PKG_VERSION"))?,
        Command::Compare { original, target, format } => {
            let mut walk = diff_dirs::walk(original, target)?;
            while let Some(item) = walk.next() {
                match item {
                    Ok(diff) => render::record(&mut out, &diff, format)?,
                    Err(error) => {
                        render::summary(&mut out, &walk.into_summary(), format)?;
                        out.flush()?;
                        return Err(error.into());
                    }
                }
            }
            let summary = walk.into_summary();
            render::summary(&mut out, &summary, format)?;
            out.flush()?;
            return Ok(u8::from(summary.has_differences()));
        }
    }
    out.flush()?;
    Ok(0)
}
