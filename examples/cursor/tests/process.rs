mod support;

use std::ffi::OsStr;
use std::path::Path;
use std::process::{Command, Output};

use diff_dirs_examples_cursor::{Cursor, MAX_ARCHIVE_BYTES, read_cursor};
use serde_json::Value;
use support::{Fixture, non_utf8};

fn cli(args: &[&OsStr]) -> (Output, Vec<Value>) {
    let output = Command::new(env!("CARGO_BIN_EXE_cursor")).args(args).output().unwrap();
    let records = output
        .stdout
        .split(|b| *b == b'\n')
        .filter(|line| !line.is_empty())
        .map(|line| serde_json::from_slice(line).unwrap())
        .collect();
    (output, records)
}

fn start(f: &Fixture, state: &Path, units: &str, extra: &[&OsStr]) -> (Output, Vec<Value>) {
    let mut args = vec![
        OsStr::new("start"),
        OsStr::new("--source"),
        f.source.as_os_str(),
        OsStr::new("--target"),
        f.target.as_os_str(),
        OsStr::new("--label"),
        OsStr::new("process label"),
        OsStr::new("--units"),
        OsStr::new(units),
        OsStr::new("--cursor-out"),
        state.as_os_str(),
    ];
    args.extend_from_slice(extra);
    cli(&args)
}

fn resume(state: &Path, units: &str, extra: &[&OsStr]) -> (Output, Vec<Value>) {
    let mut args = vec![
        OsStr::new("resume"),
        OsStr::new("--cursor-in"),
        state.as_os_str(),
        OsStr::new("--units"),
        OsStr::new(units),
        OsStr::new("--cursor-out"),
        state.as_os_str(),
    ];
    args.extend_from_slice(extra);
    cli(&args)
}

fn succeeded(output: &Output) {
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
}
fn events(records: &[Value]) -> Vec<Value> {
    records.iter().filter(|v| v["type"] != "invocation").cloned().collect()
}

#[test]
fn successive_processes_match_uninterrupted_output_and_native_paths() {
    let f = Fixture::mixed();
    let full = f.directory.path().join("full.rkyv");
    let partial = f.directory.path().join("partial.rkyv");
    let (output, expected) = start(&f, &full, "100", &[]);
    succeeded(&output);
    let (output, mut records) = start(&f, &partial, "1", &[]);
    succeeded(&output);
    assert_eq!(records.last().unwrap()["attempted"], 1);
    assert!(!records.last().unwrap()["done"].as_bool().unwrap());
    let mut observed = events(&records);
    let first_bytes = std::fs::metadata(&partial).unwrap().len();
    assert!(first_bytes > 56 && first_bytes < MAX_ARCHIVE_BYTES as u64);
    for _ in 0..9 {
        let (output, next) = resume(&partial, "1", &[]);
        succeeded(&output);
        assert_eq!(next.last().unwrap()["attempted"], 1);
        observed.extend(events(&next));
        records = next;
    }
    assert_eq!(observed, events(&expected));
    assert_eq!(read_cursor(&partial).unwrap(), read_cursor(&full).unwrap());
    assert!(records.last().unwrap()["done"].as_bool().unwrap());
    let bytes = diff_dirs_examples_cursor::NativePath::new(&non_utf8()).bytes;
    assert!(observed.iter().any(|v| v["type"] == "completion" && v["logical"] == serde_json::json!(bytes)));
    let root = observed
        .iter()
        .find(|v| v["type"] == "completion" && v["logical"] == serde_json::json!([]))
        .unwrap();
    let generated: Vec<u8> = serde_json::from_value(root["generated"]["contents"].clone()).unwrap();
    assert!(String::from_utf8(generated).unwrap().contains("process label"));
    // Terminal continuation succeeds without revisiting either deleted root.
    std::fs::remove_dir_all(&f.source).unwrap();
    std::fs::remove_dir_all(&f.target).unwrap();
    let (output, terminal) = resume(&partial, "1", &[]);
    succeeded(&output);
    assert_eq!(terminal.len(), 1);
    assert_eq!(terminal[0]["attempted"], 0);
    assert_eq!(terminal[0]["completed"], 0);
    assert_eq!(terminal[0]["done"], true);
}

#[test]
fn process_failure_publishes_cursor_then_retry_or_skip_continues() {
    let f = Fixture::empty();
    f.both_dir("a/deep");
    f.both_dir("b");
    let blocked = f.directory.path().join("blocked.rkyv");
    let (output, records) = start(&f, &blocked, "9", &[OsStr::new("--fail-at"), OsStr::new("a")]);
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(records.last().unwrap()["attempted"], 2);
    assert_eq!(records.last().unwrap()["blocked"]["logical"], serde_json::json!([97]));
    let initial: Cursor = read_cursor(&blocked).unwrap();
    let retry = f.directory.path().join("retry.rkyv");
    std::fs::copy(&blocked, &retry).unwrap();
    let (output, records) = resume(&retry, "1", &[]);
    succeeded(&output);
    assert_eq!(records[0]["logical"], serde_json::json!([97]));
    assert_eq!(records.last().unwrap()["attempted"], 1);
    assert!(records.last().unwrap()["blocked"].is_null());
    assert_eq!(read_cursor(&retry).unwrap().attempted_total, initial.attempted_total + 1);
    let (output, records) = resume(&blocked, "1", &[OsStr::new("--skip-blocked")]);
    succeeded(&output);
    assert_eq!(records[0]["type"], "skipped");
    assert_eq!(records.last().unwrap()["attempted"], 1);
    assert_eq!(records.last().unwrap()["done"], true);
    assert_eq!(records.last().unwrap()["final_state"]["incomplete"], true);
    assert_eq!(records.last().unwrap()["skipped_total"], 1);
}

#[test]
fn command_errors_do_not_publish_new_cursor_and_failed_output_is_reported() {
    let f = Fixture::empty();
    let state = f.directory.path().join("state.rkyv");
    let (output, _) = start(&f, &state, "0", &[]);
    assert_eq!(output.status.code(), Some(2));
    assert!(!state.exists());
    let (output, _) = start(&f, &state, "1", &[]);
    succeeded(&output);
    let old = std::fs::read(&state).unwrap();
    let (output, _) = resume(&state, "1", &[OsStr::new("--skip-blocked")]);
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(std::fs::read(&state).unwrap(), old);
    std::fs::write(&state, b"corrupt").unwrap();
    let (output, _) = resume(&state, "1", &[]);
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(std::fs::read(&state).unwrap(), b"corrupt");
    let missing_parent = f.directory.path().join("absent/state.rkyv");
    let (output, records) = start(&f, &missing_parent, "1", &[]);
    assert_eq!(output.status.code(), Some(2));
    assert!(records.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("prepare cursor output"));
    assert!(!missing_parent.exists());
    let too_large = std::fs::File::create(&state).unwrap();
    too_large.set_len(MAX_ARCHIVE_BYTES as u64 + 1).unwrap();
    assert!(read_cursor(&state).is_err());
}
