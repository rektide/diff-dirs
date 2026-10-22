use std::ffi::OsStr;
use std::io::{self, Write};

use diff_dirs_tree_api::{DirDiff, ScanState, Summary};
use serde_json::{Value, json};

use crate::args::Format;

pub fn record(out: &mut impl Write, diff: &DirDiff, format: Format) -> io::Result<()> {
    match format {
        Format::Json => line(
            out,
            &json!({
                "type": "diff",
                "dir": native(diff.dir.as_os_str()),
                "files": {
                    "changed": values(&diff.files.changed),
                    "added": values(&diff.files.added),
                    "removed": values(&diff.files.removed),
                },
                "dirs": {
                    "added": values(&diff.dirs.added),
                    "removed": values(&diff.dirs.removed),
                },
            }),
        ),
        Format::Text => {
            writeln!(out, "diff dir={:?}", diff.dir)?;
            for (category, names) in [
                ("files.changed", &diff.files.changed),
                ("files.added", &diff.files.added),
                ("files.removed", &diff.files.removed),
                ("dirs.added", &diff.dirs.added),
                ("dirs.removed", &diff.dirs.removed),
            ] {
                for name in names {
                    writeln!(out, "  {category} {name:?}")?;
                }
            }
            Ok(())
        }
    }
}

pub fn summary(out: &mut impl Write, summary: &Summary, format: Format) -> io::Result<()> {
    let state = match summary.state {
        ScanState::Incomplete => "incomplete",
        ScanState::Complete => "complete",
        ScanState::Failed => "failed",
    };
    match format {
        Format::Json => line(
            out,
            &json!({
                "type": "summary",
                "state": state,
                "complete": summary.is_complete(),
                "clean": summary.is_clean(),
                "examined_directories": summary.examined_directories,
                "counts": {
                    "files": {
                        "changed": summary.counts.files.changed,
                        "added": summary.counts.files.added,
                        "removed": summary.counts.files.removed,
                    },
                    "dirs": {
                        "added": summary.counts.dirs.added,
                        "removed": summary.counts.dirs.removed,
                    },
                },
                "topmost_divergent": values(&summary.topmost_divergent),
                "changed_dirs": values(&summary.changed_dirs),
                "pruned": values(&summary.pruned),
            }),
        ),
        Format::Text => writeln!(
            out,
            "summary state={state} complete={} clean={} examined_directories={} files(changed={},added={},removed={}) dirs(added={},removed={}) topmost={:?} changed_dirs={:?} pruned={:?}",
            summary.is_complete(),
            summary.is_clean(),
            summary.examined_directories,
            summary.counts.files.changed,
            summary.counts.files.added,
            summary.counts.files.removed,
            summary.counts.dirs.added,
            summary.counts.dirs.removed,
            summary.topmost_divergent,
            summary.changed_dirs,
            summary.pruned,
        ),
    }
}

fn line(out: &mut impl Write, value: &Value) -> io::Result<()> {
    serde_json::to_writer(&mut *out, value).map_err(io::Error::other)?;
    writeln!(out)
}

fn values<T: AsRef<OsStr>>(items: &[T]) -> Vec<Value> {
    items.iter().map(|item| native(item.as_ref())).collect()
}

fn native(value: &OsStr) -> Value {
    if let Some(value) = value.to_str() {
        return Value::String(value.to_string());
    }
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        json!({ "encoding": "unix-bytes", "bytes": value.as_bytes() })
    }
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        json!({ "encoding": "windows-wide", "units": value.encode_wide().collect::<Vec<_>>() })
    }
    #[cfg(not(any(unix, windows)))]
    {
        json!({ "encoding": "rust-native", "bytes": value.as_encoded_bytes() })
    }
}
