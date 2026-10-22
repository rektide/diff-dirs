#![cfg(feature = "codec")]

use std::fs;
use std::os::unix::ffi::OsStringExt;

use diff_dirs_examples_walk::*;
use kopacetic_core::EntityIdentity;
use kopacetic_rkyv::{Schema, Snapshot};

#[test]
fn snapshot_preserves_non_utf8_and_pending_finish_and_rejects_corrupt_incompatible_state() {
    let temp = tempfile::tempdir_in("/tmp/opencode").unwrap();
    let source = temp.path().join("source");
    let target = temp.path().join("target");
    let reports = temp.path().join("reports");
    for root in [&source, &target, &reports] {
        fs::create_dir(root).unwrap();
    }
    let name = std::ffi::OsString::from_vec(vec![0xff]);
    fs::create_dir(source.join(&name)).unwrap();
    fs::write(source.join(&name).join("a"), "one").unwrap();
    let bindings = Bindings::new(&source, &target, &reports).unwrap();
    let mut run = cursor()
        .parts(StartParts::Fresh(bindings))
        .inspector(FullInspector)
        .executor(ByteExecutor)
        .program(DesiredCopy::v1())
        .manifest(Manifest::builder().label("A".into()).build())
        .build()
        .unwrap();
    run.bind_filesystem().unwrap();
    run.drive(Limits { assessments: 2, effects: 2 }).unwrap();
    let saved = run.into_saved().unwrap();
    assert_eq!(saved.stack[1].node.relative.bytes, vec![0xff]);
    assert!(matches!(saved.pending.as_ref().unwrap().unit, Unit::Effect(Work::InstallReport(_))));
    let bytes = codec::encode(&saved).unwrap();
    assert_eq!(codec::decode(bytes.clone()).unwrap(), saved);
    let mut corrupt = bytes;
    let last = corrupt.len() - 1;
    corrupt[last] ^= 1;
    assert!(codec::decode(corrupt).is_err());
    assert!(codec::decode(vec![0; 12]).is_err());
    assert!(codec::decode(vec![0; codec::MAX_ARCHIVE_BYTES + 1]).is_err());
    let schema = Schema::new("diff-dirs.walk-example", 1).unwrap();
    let identity = EntityIdentity::standalone("WalkingCursor", "example");
    let bad_identity = Snapshot::from_value(&EntityIdentity::standalone("Other", "example"), &schema, &saved)
        .unwrap()
        .into_bytes();
    assert!(codec::decode(bad_identity).is_err());
    let bad_schema = Snapshot::from_value(&identity, &Schema::new("diff-dirs.walk-example", 2).unwrap(), &saved)
        .unwrap()
        .into_bytes();
    assert!(codec::decode(bad_schema).is_err());
    let mut bad_state = saved.clone();
    bad_state.stack[0].remaining[0].relative.bytes = b"../bad".to_vec();
    let bytes = Snapshot::from_value(&identity, &schema, &bad_state).unwrap().into_bytes();
    assert!(codec::decode(bytes).is_err());
    for root in [&source, &target, &reports] {
        assert!(codec::write_cursor(&root.join("cursor.bin"), &saved).is_err());
    }
    // Reconstruct without consulting vanished roots; explicit bind then fails.
    for root in [&source, &target, &reports] {
        fs::remove_dir_all(root).unwrap();
    }
    let saved = codec::decode(codec::encode(&saved).unwrap()).unwrap();
    let manifest = Manifest::from_config(saved.manifest_config().clone()).unwrap();
    let mut run = cursor()
        .parts(StartParts::Restored(saved))
        .inspector(FullInspector)
        .executor(ByteExecutor)
        .program(DesiredCopy::v1())
        .manifest(manifest)
        .build()
        .unwrap();
    assert!(run.bind_filesystem().is_err());
}
