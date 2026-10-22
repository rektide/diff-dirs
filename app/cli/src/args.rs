use std::ffi::OsString;
use std::path::PathBuf;

pub const HELP: &str = "Usage: diff-dirs ORIGINAL TARGET [--format json|text]\n\
    Compare existing real directories without reading file contents.\n\
    Added = original-only; removed = target-only.\n\
    Output: streamed records followed by a summary (JSON is JSONL).\n\
    Exit: 0 clean, 1 differences, 2 usage/IO error.\n\
    Use -- before paths beginning with '-'.";

#[derive(Clone, Copy)]
pub enum Format {
    Json,
    Text,
}

pub enum Command {
    Help,
    Version,
    Compare { original: PathBuf, target: PathBuf, format: Format },
}

pub fn parse(args: impl IntoIterator<Item = OsString>) -> Result<Command, String> {
    let mut args = args.into_iter();
    let mut paths = Vec::new();
    let mut format = Format::Text;
    let mut positional = false;
    while let Some(arg) = args.next() {
        if !positional {
            match arg.to_str() {
                Some("--help" | "-h") => return Ok(Command::Help),
                Some("--version" | "-V") => return Ok(Command::Version),
                Some("--") => {
                    positional = true;
                    continue;
                }
                Some("--format") => {
                    let value = args.next().ok_or("--format requires json or text")?;
                    format = parse_format(value.to_str())?;
                    continue;
                }
                Some(value) if value.starts_with("--format=") => {
                    format = parse_format(value.strip_prefix("--format="))?;
                    continue;
                }
                _ if arg.as_encoded_bytes().starts_with(b"-") => {
                    return Err(format!("unknown option {arg:?}\n{HELP}"));
                }
                _ => {}
            }
        }
        paths.push(PathBuf::from(arg));
    }
    if paths.len() != 2 {
        return Err(HELP.to_string());
    }
    let target = paths.pop().unwrap();
    let original = paths.pop().unwrap();
    Ok(Command::Compare { original, target, format })
}

fn parse_format(value: Option<&str>) -> Result<Format, String> {
    match value {
        Some("json") => Ok(Format::Json),
        Some("text") => Ok(Format::Text),
        _ => Err("--format requires json or text".to_string()),
    }
}
