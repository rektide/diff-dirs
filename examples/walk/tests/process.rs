#![cfg(feature = "cli")]

use std::fs;
use std::path::Path;
use std::process::Command;

use diff_dirs_walk_example::{Phase, SavedWalk, Unit, Work, codec};

fn invoke(args: &[&std::ffi::OsStr]) -> serde_json::Value {
    let output = Command::new(env!("CARGO_BIN_EXE_walk")).args(args).output().unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    serde_json::from_slice(&output.stdout).unwrap()
}

fn resume(cursor: &Path, effects: &str) -> serde_json::Value {
    invoke(&["resume".as_ref(), "--cursor".as_ref(), cursor.as_os_str(), "--assessments".as_ref(), "0".as_ref(), "--effects".as_ref(), effects.as_ref()])
}

fn report_request(saved: &SavedWalk, relative: &[u8], ordinal: u64) {
    match &saved.pending.as_ref().unwrap().unit {
        Unit::Effect(Work::InstallReport(recipe)) => { assert_eq!(recipe.relative.bytes, relative); assert_eq!(recipe.ordinal, ordinal); assert_eq!(recipe.label, "A"); }
        other => panic!("expected finish request, got {other:?}"),
    }
}

#[test]
fn four_fresh_process_gates_preserve_accepted_frames_bytes_and_component_state() {
    let temp = tempfile::tempdir_in("/tmp/opencode").unwrap();
    let source = temp.path().join("source");
    let target = temp.path().join("target");
    let reports = temp.path().join("reports");
    for path in [&source, &target, &reports] { fs::create_dir(path).unwrap(); }
    fs::create_dir(source.join("d")).unwrap();
    fs::write(source.join("d/a"), b"one").unwrap();
    fs::write(source.join("d/b"), b"two").unwrap();
    let cursor = temp.path().join("cursor.bin");

    let first = invoke(&["start".as_ref(), "--source".as_ref(), source.as_os_str(), "--target".as_ref(), target.as_os_str(), "--reports".as_ref(), reports.as_os_str(), "--label".as_ref(), "A".as_ref(), "--cursor".as_ref(), cursor.as_os_str(), "--assessments".as_ref(), "2".as_ref(), "--effects".as_ref(), "2".as_ref()]);
    assert_eq!(first["progress"]["stop"], "Budget");
    assert_eq!(first["progress"]["attempted_assessments"], 2);
    assert_eq!(first["progress"]["accepted"]["leaves"], 1);
    assert_eq!(first["progress"]["writes"]["directories_created"], 1);
    assert_eq!(first["progress"]["writes"]["file_installations"], 1);
    let saved = codec::read_cursor(&cursor).unwrap();
    assert_eq!(saved.stack.len(), 3);
    assert_eq!(saved.stack[0].aggregate.leaves, 0);
    assert_eq!(saved.stack[1].aggregate.leaves, 1);
    assert_eq!(saved.stack[1].remaining.len(), 1);
    assert_eq!(saved.stack[1].remaining[0].relative.bytes, b"d/b");
    assert_eq!(saved.stack[2].phase, Phase::Leaf);
    assert_eq!(saved.manifest_state.next_ordinal, 1);
    assert!(saved.stack.iter().all(|f| !f.incomplete));
    assert_eq!(fs::read(target.join("d/a")).unwrap(), b"one");
    assert!(!target.join("d/b").exists());
    assert_eq!(fs::read_dir(&reports).unwrap().count(), 0);

    let second = resume(&cursor, "1");
    assert_eq!(second["progress"]["attempted_assessments"], 0);
    let saved = codec::read_cursor(&cursor).unwrap();
    assert_eq!(saved.stack.len(), 2);
    assert_eq!(saved.stack[1].phase, Phase::Finish);
    assert_eq!(saved.stack[1].aggregate.leaves, 2);
    assert_eq!(saved.stack[0].aggregate.leaves, 0);
    assert_eq!(saved.manifest_state.next_ordinal, 1);
    report_request(&saved, b"d", 1);
    assert_eq!(fs::read(target.join("d/b")).unwrap(), b"two");
    assert!(!reports.join("dir-64.txt").exists());

    let third = resume(&cursor, "1");
    assert_eq!(third["progress"]["attempted_assessments"], 0);
    let saved = codec::read_cursor(&cursor).unwrap();
    assert_eq!(saved.stack.len(), 1);
    assert_eq!(saved.stack[0].aggregate.leaves, 2);
    assert_eq!(saved.manifest_state.next_ordinal, 2);
    report_request(&saved, b"", 2);
    assert_eq!(fs::read(reports.join("dir-64.txt")).unwrap(), b"manifest/v1\nlabel_bytes: 1\nA\nordinal: 1\nleaves: 2\nincomplete: false\n");
    assert!(!reports.join("root.txt").exists());

    let fourth = resume(&cursor, "1");
    assert_eq!(fourth["progress"]["stop"], "Done");
    let saved = codec::read_cursor(&cursor).unwrap();
    assert!(saved.stack.is_empty() && saved.pending.is_none());
    assert_eq!(saved.terminal.as_ref().unwrap().aggregate.leaves, 2);
    assert!(!saved.terminal.as_ref().unwrap().incomplete);
    assert_eq!(saved.manifest_state.next_ordinal, 3);
    assert_eq!(fs::read(reports.join("root.txt")).unwrap(), b"manifest/v1\nlabel_bytes: 1\nA\nordinal: 2\nleaves: 2\nincomplete: false\n");

    // Fifth process exercises the terminal no-tree-I/O guarantee.
    for path in [&source, &target, &reports] { fs::remove_dir_all(path).unwrap(); }
    let terminal = resume(&cursor, "0");
    assert_eq!(terminal["progress"]["stop"], "Done");
    assert_eq!(codec::read_cursor(&cursor).unwrap(), saved);
}
