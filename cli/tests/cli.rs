#[path = "../../tests/support/mod.rs"]
mod support;

use std::process::{Command, Output};

use serde_json::Value;
use support::{Fixture, file};

fn run(f: &Fixture, format: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_diff-dirs"))
        .arg(&f.original)
        .arg(&f.target)
        .args(["--format", format])
        .output()
        .unwrap()
}

fn json_lines(output: &Output) -> Vec<Value> {
    std::str::from_utf8(&output.stdout)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}

#[test]
fn clean_json_exits_zero_and_emits_only_complete_summary() {
    let f = Fixture::new();
    f.both_file("same", b"same");
    let output = run(&f, "json");
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());
    let lines = json_lines(&output);
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0]["type"], "summary");
    assert_eq!(lines[0]["state"], "complete");
    assert_eq!(lines[0]["complete"], true);
    assert_eq!(lines[0]["clean"], true);
}

#[test]
fn differences_exit_one_and_json_preserves_categories_and_summary() {
    let f = Fixture::new();
    file(&f.original.join("new"), b"");
    file(&f.target.join("stale"), b"");
    let output = run(&f, "json");
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stderr.is_empty());
    let lines = json_lines(&output);
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0]["type"], "diff");
    assert_eq!(lines[0]["dir"], "");
    assert_eq!(lines[0]["files"]["added"], serde_json::json!(["new"]));
    assert_eq!(lines[0]["files"]["removed"], serde_json::json!(["stale"]));
    assert_eq!(lines[1]["counts"]["files"]["added"], 1);
    assert_eq!(lines[1]["counts"]["files"]["removed"], 1);
    assert_eq!(lines[1]["topmost_divergent"], serde_json::json!([""]));
    assert_eq!(lines[1]["complete"], true);
    assert_eq!(lines[1]["clean"], false);
}

#[test]
fn text_escapes_control_characters() {
    let f = Fixture::new();
    file(&f.original.join("line\nwith\ttab"), b"");
    let output = run(&f, "text");
    assert_eq!(output.status.code(), Some(1));
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(text.contains("\"line\\nwith\\ttab\""));
    assert_eq!(text.lines().count(), 3);
}

#[test]
fn missing_root_exits_two_and_names_the_path() {
    let f = Fixture::new();
    std::fs::remove_dir(&f.target).unwrap();
    let output = run(&f, "json");
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8(output.stderr).unwrap().contains("target"));
}

#[test]
fn bad_arguments_exit_two_but_help_and_version_exit_zero() {
    let exe = env!("CARGO_BIN_EXE_diff-dirs");
    for args in [vec![], vec!["a"], vec!["--unknown"], vec!["a", "b", "--format", "yaml"], vec!["--format"]] {
        let output = Command::new(exe).args(args).output().unwrap();
        assert_eq!(output.status.code(), Some(2));
        assert!(!output.stderr.is_empty());
    }
    for arg in ["--help", "--version"] {
        let output = Command::new(exe).arg(arg).output().unwrap();
        assert_eq!(output.status.code(), Some(0));
        assert!(output.stderr.is_empty());
        assert!(!output.stdout.is_empty());
    }
}

#[test]
fn double_dash_accepts_paths_that_look_like_options() {
    let f = Fixture::new();
    std::fs::rename(&f.original, f.root.join("-original")).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_diff-dirs"))
        .current_dir(&f.root)
        .args(["--format=json", "--", "-original", "target"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(json_lines(&output)[0]["clean"], true);
}

#[cfg(unix)]
#[test]
fn non_utf8_paths_are_tagged_native_bytes_not_lossy_strings() {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;

    let f = Fixture::new();
    let name = OsString::from_vec(vec![0xff]);
    f.both_dir(&name);
    file(&f.original.join(&name).join(&name), b"");
    let output = run(&f, "json");
    assert_eq!(output.status.code(), Some(1));
    let lines = json_lines(&output);
    let expected = serde_json::json!({ "encoding": "unix-bytes", "bytes": [255] });
    assert_eq!(lines[0]["dir"], expected);
    assert_eq!(lines[0]["files"]["added"][0], expected);
    assert_eq!(lines[1]["topmost_divergent"][0], expected);
    let text = String::from_utf8(run(&f, "text").stdout).unwrap();
    assert!(text.contains("\\xFF"));
    assert!(!text.contains('\u{fffd}'));
}

#[cfg(unix)]
#[test]
fn special_file_error_emits_failed_summary_and_exits_two() {
    let f = Fixture::new();
    let _socket = std::os::unix::net::UnixListener::bind(f.original.join("socket")).unwrap();
    let output = run(&f, "json");
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8(output.stderr.clone()).unwrap().contains("socket"));
    let lines = json_lines(&output);
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0]["type"], "summary");
    assert_eq!(lines[0]["state"], "failed");
    assert_eq!(lines[0]["complete"], false);
    assert_eq!(lines[0]["clean"], false);
}
