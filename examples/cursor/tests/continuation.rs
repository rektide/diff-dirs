mod support;

use std::num::NonZeroUsize;
use std::path::Path;

use diff_dirs_cursor_example::{Counts, Cursor, Descriptor, Event, NativePath, Phase, RunOptions, decode, encode};
use kopacetic_core::EntityIdentity;
use kopacetic_rkyv::{Schema, Snapshot};
use support::{Fixture, file, non_utf8};

fn units(n: usize) -> NonZeroUsize {
    NonZeroUsize::new(n).unwrap()
}
fn start(f: &Fixture) -> Cursor {
    Cursor::start(&f.source, &f.target, Descriptor::directory_summary("persisted label".into())).unwrap()
}
fn native(path: &str) -> NativePath {
    NativePath::new(Path::new(path))
}

#[test]
fn one_at_a_time_roundtrips_match_uninterrupted_mixed_assessment() {
    let f = Fixture::mixed();
    let mut whole = start(&f);
    let expected = whole.advance(units(100), &RunOptions::default()).unwrap();
    assert_eq!(expected.attempted, 10);
    assert_eq!(expected.completed, 10);
    assert!(expected.done);
    assert_eq!(
        whole.final_state.as_ref().unwrap().totals,
        Counts {
            source_only: 6,
            target_only: 5,
            changed: 3,
            equal: 7
        }
    );

    let mut cursor = start(&f);
    let mut events = Vec::new();
    for _ in 0..10 {
        let run = cursor.advance(units(1), &RunOptions::default()).unwrap();
        assert_eq!(run.attempted, 1); // includes empty/clean directories
        events.extend(run.events);
        cursor = decode(encode(&cursor).unwrap()).unwrap();
    }
    assert_eq!(events, expected.events);
    assert_eq!(cursor, whole);
    let observed: Vec<_> = events
        .iter()
        .filter_map(|e| match e {
            Event::Assessment { logical, .. } => Some(logical.to_path_buf()),
            _ => None,
        })
        .collect();
    assert_eq!(
        observed,
        ["", "a", "a/deep", "b", "collision"]
            .into_iter()
            .map(Into::into)
            .chain([non_utf8()])
            .chain(["source", "source/sub", "target", "target/sub"].into_iter().map(Into::into))
            .collect::<Vec<_>>()
    );
    let completions: Vec<_> = events
        .iter()
        .filter_map(|e| match e {
            Event::Completion(c) => Some(c),
            _ => None,
        })
        .collect();
    assert_eq!(completions.last().unwrap().logical, native(""));
    assert!(completions.iter().any(|c| c.logical == NativePath::new(&non_utf8())));
    assert!(
        String::from_utf8(completions.last().unwrap().generated.contents.clone())
            .unwrap()
            .contains("persisted label")
    );
    assert!(completions.iter().all(|c| c.generated.logical_name == "summary.txt" && !c.incomplete));
    assert!(!f.source.join("summary.txt").exists());
    assert!(!f.target.join("summary.txt").exists());
}

#[test]
fn resumes_frames_without_relisting_assessed_ancestors_and_drains_completions() {
    let f = Fixture::empty();
    f.both_dir("a/b");
    let mut cursor = start(&f);
    cursor.advance(units(1), &RunOptions::default()).unwrap();
    assert_eq!(cursor.stack.len(), 2);
    assert_eq!(cursor.stack[0].phase, Phase::Assessed);
    assert_eq!(cursor.stack[1].phase, Phase::Pending);
    cursor = decode(encode(&cursor).unwrap()).unwrap();
    f.both_dir("new-root-sibling");
    let run = cursor.advance(units(2), &RunOptions::default()).unwrap();
    assert!(run.done);
    assert_eq!(run.attempted, 2);
    assert_eq!(run.completed, 3); // last unit drains b, a, root
    let assessed: Vec<_> = run
        .events
        .iter()
        .filter_map(|e| match e {
            Event::Assessment { logical, .. } => Some(logical.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(assessed, [native("a"), native("a/b")]);
    // A terminal cursor performs no filesystem work, even after roots disappear.
    std::fs::remove_dir_all(&f.source).unwrap();
    std::fs::remove_dir_all(&f.target).unwrap();
    let run = decode(encode(&cursor).unwrap()).unwrap().advance(units(1), &RunOptions::default()).unwrap();
    assert!(run.done && run.events.is_empty());
    assert_eq!(run.attempted, 0);
}

#[test]
fn blocked_attempts_stop_budget_and_retry_or_skip_the_scheduled_subtree() {
    let f = Fixture::empty();
    f.both_dir("a/deep");
    f.both_dir("b");
    let mut cursor = start(&f);
    let options = RunOptions {
        fail_at: Some(native("a")),
        ..RunOptions::default()
    };
    let run = cursor.advance(units(20), &options).unwrap();
    assert_eq!(run.attempted, 2);
    assert_eq!(run.completed, 0);
    assert_eq!(cursor.blocked.as_ref().unwrap().logical, native("a"));
    assert_eq!(cursor.stack.last().unwrap().phase, Phase::Pending);
    let bytes = encode(&cursor).unwrap();
    let mut retry = decode(bytes.clone()).unwrap();
    let run = retry.advance(units(1), &RunOptions::default()).unwrap();
    assert_eq!(run.attempted, 1);
    assert!(matches!(&run.events[0], Event::Assessment { logical, observed: Some(_), .. } if logical == &native("a")));
    assert!(retry.blocked.is_none());
    assert_eq!(retry.stack.len(), 3); // root, assessed a, pending deep
    assert_eq!(decode(encode(&retry).unwrap()).unwrap(), retry);

    let mut skip = decode(bytes).unwrap();
    let run = skip
        .advance(
            units(1),
            &RunOptions {
                skip_blocked: true,
                ..RunOptions::default()
            },
        )
        .unwrap();
    assert!(run.done);
    assert_eq!((run.attempted, run.completed, run.skipped), (1, 2, 1));
    assert!(matches!(&run.events[0], Event::Skipped { logical } if logical == &native("a")));
    assert!(skip.final_state.as_ref().unwrap().incomplete);
    assert_eq!((skip.attempted_total, skip.completed_total, skip.skipped_total), (3, 2, 1));
    assert!(
        !run.events
            .iter()
            .any(|e| matches!(e, Event::Completion(c) if c.logical == native("a") || c.logical == native("a/deep")))
    );
    assert!(
        skip.advance(
            units(1),
            &RunOptions {
                skip_blocked: true,
                ..RunOptions::default()
            }
        )
        .is_err()
    );
}

#[test]
fn deep_skip_marks_all_active_ancestors_incomplete_before_they_finish() {
    let f = Fixture::empty();
    f.both_dir("a/blocked/deep");
    f.both_dir("a/z/next");
    let mut cursor = start(&f);
    let options = RunOptions {
        fail_at: Some(native("a/blocked")),
        ..RunOptions::default()
    };
    assert_eq!(cursor.advance(units(10), &options).unwrap().attempted, 3);
    let run = cursor
        .advance(
            units(1),
            &RunOptions {
                skip_blocked: true,
                ..RunOptions::default()
            },
        )
        .unwrap();
    assert!(!run.done);
    assert_eq!(cursor.stack.len(), 4); // root, a, assessed z, pending next
    assert!(cursor.stack[0].incomplete && cursor.stack[1].incomplete);
    assert!(!cursor.stack[2].incomplete);
    let mut cursor = decode(encode(&cursor).unwrap()).unwrap();
    let run = cursor.advance(units(1), &RunOptions::default()).unwrap();
    assert!(run.done);
    assert!(cursor.final_state.as_ref().unwrap().incomplete);
    let incomplete: Vec<_> = run
        .events
        .iter()
        .filter_map(|e| match e {
            Event::Completion(c) if c.incomplete => Some(c.logical.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(incomplete, [native("a"), native("")]);
}

#[test]
fn actual_inspector_failure_keeps_the_whole_directory_pending_without_partial_counts() {
    let f = Fixture::empty();
    f.both_dir("a");
    f.both_file("a/equal", b"same");
    let socket_path = f.source.join("a/socket");
    let socket = std::os::unix::net::UnixListener::bind(&socket_path).unwrap();
    let mut cursor = start(&f);
    let run = cursor.advance(units(10), &RunOptions::default()).unwrap();
    assert_eq!((run.attempted, run.completed), (2, 0));
    assert_eq!(cursor.blocked.as_ref().unwrap().logical, native("a"));
    let frame = cursor.stack.last().unwrap();
    assert_eq!(frame.phase, Phase::Pending);
    assert_eq!(frame.totals, Counts::default());
    drop(socket);
    std::fs::remove_file(socket_path).unwrap();
    let mut cursor = decode(encode(&cursor).unwrap()).unwrap();
    let run = cursor.advance(units(1), &RunOptions::default()).unwrap();
    assert!(run.done);
    assert_eq!((run.attempted, run.completed), (1, 2));
    assert_eq!(cursor.final_state.as_ref().unwrap().totals.equal, 2);
}

#[test]
fn missing_and_leaf_replaced_scheduled_directories_block_as_real_attempts() {
    let f = Fixture::empty();
    f.both_dir("a");
    let mut cursor = start(&f);
    cursor.advance(units(1), &RunOptions::default()).unwrap();
    std::fs::remove_dir(f.source.join("a")).unwrap();
    let run = cursor.advance(units(5), &RunOptions::default()).unwrap();
    assert_eq!(run.attempted, 1);
    assert!(cursor.blocked.is_some());
    file(&f.source.join("a"), b"not a directory");
    let run = cursor.advance(units(5), &RunOptions::default()).unwrap();
    assert_eq!(run.attempted, 1);
    assert!(cursor.blocked.as_ref().unwrap().message.contains("not a real directory"));
    std::fs::remove_file(f.source.join("a")).unwrap();
    std::fs::create_dir(f.source.join("a")).unwrap();
    assert!(cursor.advance(units(1), &RunOptions::default()).unwrap().done);
    assert_eq!(cursor.attempted_total, 4);
}

#[test]
fn skipping_blocked_root_is_terminal_and_incomplete() {
    let f = Fixture::empty();
    let mut cursor = start(&f);
    cursor
        .advance(
            units(5),
            &RunOptions {
                fail_at: Some(native("")),
                ..RunOptions::default()
            },
        )
        .unwrap();
    let run = cursor
        .advance(
            units(1),
            &RunOptions {
                skip_blocked: true,
                ..RunOptions::default()
            },
        )
        .unwrap();
    assert!(run.done);
    assert_eq!((run.attempted, run.completed, run.skipped), (0, 0, 1));
    assert!(cursor.final_state.as_ref().unwrap().incomplete);
    assert_eq!(decode(encode(&cursor).unwrap()).unwrap(), cursor);
}

fn unchecked_snapshot(cursor: &Cursor, schema: &Schema, identity: &EntityIdentity) -> Vec<u8> {
    Snapshot::from_value(identity, schema, cursor).unwrap().into_bytes()
}

#[test]
fn codec_rejects_corruption_schema_identity_and_semantically_malformed_cursors() {
    let f = Fixture::empty();
    f.both_dir("a");
    let mut cursor = start(&f);
    cursor.advance(units(1), &RunOptions::default()).unwrap();
    let schema = Schema::new("diff-dirs.cursor-example", 1).unwrap();
    let identity = EntityIdentity::standalone("DirectoryCursor", "example");
    let good = encode(&cursor).unwrap();
    assert_eq!(&good[..8], b"KOPARKYV");
    assert!(decode(good[..good.len() - 1].to_vec()).is_err());
    let mut corrupt = good;
    *corrupt.last_mut().unwrap() ^= 1;
    assert!(decode(corrupt).is_err());
    assert!(decode(unchecked_snapshot(&cursor, &Schema::new("diff-dirs.cursor-example", 2).unwrap(), &identity)).is_err());
    assert!(decode(unchecked_snapshot(&cursor, &schema, &EntityIdentity::standalone("Wrong", "example"))).is_err());

    let mut bad = cursor.clone();
    bad.stack[0].next_child = 999;
    assert!(decode(unchecked_snapshot(&bad, &schema, &identity)).is_err());
    let mut bad = cursor.clone();
    bad.stack[1].tip.logical = native("../escape");
    assert!(decode(unchecked_snapshot(&bad, &schema, &identity)).is_err());
    for invalid in ["/absolute", "a/../escape", "a/./b", "a//b", "nul\0name"] {
        let mut bad = cursor.clone();
        bad.stack[1].tip.logical = native(invalid);
        assert!(decode(unchecked_snapshot(&bad, &schema, &identity)).is_err(), "{invalid:?}");
    }
    let mut bad = cursor.clone();
    bad.stack[1].tip.source = None;
    assert!(decode(unchecked_snapshot(&bad, &schema, &identity)).is_err()); // mismatches selected parent child
    let mut bad = cursor.clone();
    bad.generator.parameters.insert("unsupported".into(), "value".into());
    assert!(decode(unchecked_snapshot(&bad, &schema, &identity)).is_err());
    let mut bad = cursor.clone();
    bad.generator.kind = "unknown/v1".into();
    assert!(decode(unchecked_snapshot(&bad, &schema, &identity)).is_err());
    let mut bad = cursor.clone();
    bad.platform = "windows".into();
    assert!(decode(unchecked_snapshot(&bad, &schema, &identity)).is_err());
    let mut bad = cursor;
    bad.stack[1].phase = Phase::Assessed;
    bad.stack[0].totals.source_only = u64::MAX;
    bad.stack[1].totals.source_only = 1;
    // Semantic progress need not authenticate old counts, but arithmetic fails
    // instead of panicking even on a well-formed archive with hostile totals.
    let mut bad = decode(unchecked_snapshot(&bad, &schema, &identity)).unwrap();
    assert!(bad.advance(units(1), &RunOptions::default()).is_err());
}

#[test]
fn root_binding_rejects_overlap_leaf_and_symlink() {
    let f = Fixture::empty();
    let generator = Descriptor::directory_summary("label".into());
    assert!(Cursor::start(&f.source, &f.source, generator.clone()).is_err());
    f.both_dir("nested");
    assert!(Cursor::start(&f.source, &f.source.join("nested"), generator.clone()).is_err());
    file(&f.source.join("leaf"), b"file");
    assert!(Cursor::start(&f.source.join("leaf"), &f.target, generator.clone()).is_err());
    let link = f.directory.path().join("symlink-root");
    std::os::unix::fs::symlink(&f.source, &link).unwrap();
    assert!(Cursor::start(&link, &f.target, generator).is_err());
}
