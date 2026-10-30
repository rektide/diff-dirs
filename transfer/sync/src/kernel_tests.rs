use super::*;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
fn scratch() -> tempfile::TempDir {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.test-agent/sync-tests");
    fs::create_dir_all(&path).unwrap();
    tempfile::tempdir_in(fs::canonicalize(path).unwrap()).unwrap()
}
fn install(source: &Path, target: &Path, scope: &DirectoryScope, index: usize, epoch: &InspectionEpoch, policy: CopyPolicy) -> Result<Progress, Failure> {
    let metadata = fs::symlink_metadata(source).map_err(|e| Failure::io("inspect source leaf", source, e))?;
    metadata.modified().map_err(|e| Failure::io("read source mtime", source, e))?;
    let slot = scope.stage_slot(SlotId(index), target, epoch)?;
    let prepared = prepare_file(source, slot.path(), target, &metadata, policy, Postcheck::None)?;
    publish_leaf(
        prepared,
        &slot,
        &BoundDestination::new(
            target,
            scope,
            DestinationGrant::ReplaceLeaf,
            fs::symlink_metadata(target).ok().map(|m| Kind::from_metadata(&m)),
        )?,
        epoch,
    )
}
#[test]
fn cleanup_failure_retains_installations_and_primary_error() {
    let owned = scratch();
    let source = owned.path().join("source");
    fs::write(&source, b"actual installed payload").unwrap();
    for primary in [false, true] {
        let target = owned.path().join(if primary { "with-primary" } else { "cleanup-only" });
        fs::create_dir(&target).unwrap();
        let scope = DirectoryScope::new(&target, std::iter::empty());
        let epoch = InspectionEpoch::new();
        let installed = install(&source, &target.join("a"), &scope, 0, &epoch, CopyPolicy::Copy).unwrap();
        let path = scope.temporary.lock().unwrap().as_ref().unwrap().path().to_owned();
        let mut result = if primary {
            Err(Failure::invalid("inspect source leaf", &source, "injected sibling fault"))
        } else {
            Ok(Progress::default())
        };
        add_progress(&mut result, installed.clone());
        fs::set_permissions(&target, Permissions::from_mode(0o500)).unwrap();
        let result = scope.finish(result, DirectoryEnd::Retained, !primary, &epoch);
        fs::set_permissions(&target, Permissions::from_mode(0o700)).unwrap();
        assert_eq!(fs::read(target.join("a")).unwrap(), b"actual installed payload");
        if let Err(f) = result {
            assert_eq!(f.completed, installed);
            assert_eq!(
                f.primary.operation,
                if primary { "inspect source leaf" } else { "clean private staging directory" }
            );
            if primary && path.exists() {
                assert_eq!(f.secondary.len(), 1);
                assert_eq!(f.secondary[0].operation, "clean private staging directory");
            }
            if path.exists() {
                fs::remove_dir(&path).unwrap();
            }
        } else {
            assert!(File::open("/root").is_ok(), "unprivileged cleanup fault did not fire");
        }
    }
}
#[test]
fn failed_private_preparation_is_recursively_cleaned_without_progress() {
    let owned = scratch();
    let epoch = InspectionEpoch::new();
    let scope = DirectoryScope::new(owned.path(), std::iter::empty());
    let slot = scope.stage_slot(SlotId(0), &owned.path().join("target"), &epoch).unwrap();
    fs::create_dir(slot.path()).unwrap();
    fs::write(slot.path().join("readonly"), b"discarded").unwrap();
    fs::set_permissions(slot.path().join("readonly"), Permissions::from_mode(0o000)).unwrap();
    fs::set_permissions(slot.path(), Permissions::from_mode(0o000)).unwrap();
    let f = scope
        .finish(
            Err(Failure::invalid("prepare fault", slot.path(), "fault")),
            DirectoryEnd::Retained,
            false,
            &epoch,
        )
        .unwrap_err();
    assert_eq!(f.primary.operation, "prepare fault");
    assert!(f.secondary.is_empty());
    assert_eq!(f.completed, Progress::default());
    assert_eq!(fs::read_dir(owned.path()).unwrap().count(), 0);
}
#[test]
fn arena_excludes_planned_names_before_creation_and_slots_are_exclusive() {
    let owned = scratch();
    let names = [".diff-dirs-sync-0-source", ".DIFF-DIRS-SYNC-1-SOURCE", ".diff-dirs-sync-2-source"];
    let scope = DirectoryScope::new(owned.path(), names.iter().map(OsStr::new));
    let epoch = InspectionEpoch::new();
    for name in names {
        assert!(!name.to_ascii_lowercase().starts_with(scope.prefix.lock().unwrap().as_str()));
    }
    std::thread::scope(|threads| {
        for index in 0..8 {
            let scope = &scope;
            let epoch = &epoch;
            threads.spawn(move || {
                let slot = scope.stage_slot(SlotId(index), Path::new("logical-target"), epoch).unwrap();
                let output = OpenOptions::new().create_new(true).write(true).open(slot.path()).unwrap();
                assert!(OpenOptions::new().create_new(true).write(true).open(slot.path()).is_err());
                drop(output);
                fs::remove_file(slot.path()).unwrap();
            });
        }
    });
    assert_eq!(fs::read_dir(owned.path()).unwrap().count(), 1);
    scope.finish(Ok(Progress::default()), DirectoryEnd::Retained, true, &epoch).unwrap();
    assert_eq!(fs::read_dir(owned.path()).unwrap().count(), 0);
}
#[test]
fn retained_output_finalizes_exact_mtime_and_mode000_before_installation() {
    let owned = scratch();
    let source = owned.path().join("source");
    fs::write(&source, b"source").unwrap();
    let source_file = OpenOptions::new().read(true).write(true).open(&source).unwrap();
    source_file
        .set_modified(std::time::SystemTime::UNIX_EPOCH - std::time::Duration::new(1, 123_456_789))
        .unwrap();
    source_file.set_permissions(Permissions::from_mode(0o000)).unwrap();
    let metadata = source_file.metadata().unwrap();
    let staged = owned.path().join("staged");
    let output = OpenOptions::new().create_new(true).read(true).write(true).open(&staged).unwrap();
    preserve_file(&output, &metadata, &staged, Postcheck::Mtime).unwrap();
    fs::rename(&staged, owned.path().join("installed")).unwrap();
    assert_eq!(output.metadata().unwrap().mode() & 0o777, 0);
    assert_eq!(output.metadata().unwrap().modified().unwrap(), metadata.modified().unwrap());
    output.set_modified(metadata.modified().unwrap()).unwrap();
}
#[cfg(target_os = "linux")]
#[test]
fn descriptor_preparation_does_not_clobber_existing_slots_or_fallback_on_missing_source() {
    let owned = scratch();
    let source = owned.path().join("source");
    fs::write(&source, vec![b'x'; 4097]).unwrap();
    let metadata = fs::metadata(&source).unwrap();
    for policy in [CopyPolicy::Copy, CopyPolicy::ReflinkRequired, CopyPolicy::ReflinkOrCopy] {
        let staged = owned.path().join("existing");
        fs::write(&staged, b"sentinel").unwrap();
        let f = prepare_file(&source, &staged, &staged, &metadata, policy, Postcheck::None).err().unwrap();
        assert_eq!(f.primary.cause.downcast_ref::<io::Error>().unwrap().kind(), io::ErrorKind::AlreadyExists);
        assert_eq!(fs::read(&staged).unwrap(), b"sentinel");
        fs::remove_file(&staged).unwrap();
        let f = prepare_file(&owned.path().join("missing"), &staged, &staged, &metadata, policy, Postcheck::None)
            .err()
            .unwrap();
        assert_eq!(f.primary.cause.downcast_ref::<io::Error>().unwrap().kind(), io::ErrorKind::NotFound);
        assert!(!staged.exists());
        assert_eq!(f.completed, Progress::default());
    }
}
#[cfg(all(target_os = "linux", feature = "parallel"))]
#[test]
fn shared_arena_joins_real_clone_failure_and_successful_sibling() {
    let owned = scratch();
    let external = tempfile::tempdir_in("/tmp/opencode").unwrap();
    let local = owned.path().join("local");
    fs::write(&local, vec![b'x'; 4097]).unwrap();
    if let Err(e) = reflink_copy::reflink(&local, owned.path().join("probe")) {
        assert!(std::env::var_os("DIFF_DIRS_SYNC_REQUIRE_REFLINK").is_none(), "real reflink required: {e}");
        return;
    }
    let remote = external.path().join("remote");
    fs::write(&remote, b"cross-filesystem").unwrap();
    let target = owned.path().join("target");
    fs::create_dir(&target).unwrap();
    fs::write(target.join("a-fail"), b"old survives").unwrap();
    let names = ["a-fail", "z-good"];
    let scope = DirectoryScope::new(&target, names.iter().map(OsStr::new));
    let epoch = InspectionEpoch::new();
    let barrier = std::sync::Barrier::new(2);
    let pool = rayon::ThreadPoolBuilder::new().num_threads(2).build().unwrap();
    let results = pool.install(|| {
        use rayon::prelude::*;
        names
            .par_iter()
            .enumerate()
            .map(|(index, name)| {
                barrier.wait();
                install(
                    if index == 0 { &remote } else { &local },
                    &target.join(name),
                    &scope,
                    index,
                    &epoch,
                    CopyPolicy::ReflinkRequired,
                )
            })
            .collect::<Vec<_>>()
    });
    let f = scope.finish(merge(results), DirectoryEnd::Retained, false, &epoch).unwrap_err();
    assert_eq!(f.primary.operation, "prepare reflink");
    assert!(f.secondary.is_empty());
    assert_eq!(f.completed.files_reflinked, 1);
    assert_eq!(f.completed.bytes_reflinked, 4097);
    assert_eq!(fs::read(target.join("a-fail")).unwrap(), b"old survives");
    assert_eq!(fs::read(target.join("z-good")).unwrap(), fs::read(local).unwrap());
    assert_eq!(fs::read_dir(target).unwrap().count(), 2);
}

#[test]
fn absent_only_publication_refuses_new_collisions_without_widening_grant() {
    let owned = scratch();
    let source = owned.path().join("source");
    fs::write(&source, b"new").unwrap();
    let target = owned.path().join("target");
    fs::create_dir(&target).unwrap();
    let to = target.join("leaf");
    let scope = DirectoryScope::new(&target, std::iter::empty());
    let epoch = InspectionEpoch::new();
    let slot = scope.stage_slot(SlotId(0), &to, &epoch).unwrap();
    let metadata = fs::metadata(&source).unwrap();
    let prepared = prepare_file(&source, slot.path(), &to, &metadata, CopyPolicy::Copy, Postcheck::None).unwrap();
    let destination = BoundDestination::new(&to, &scope, DestinationGrant::AbsentOnly, None).unwrap();
    fs::write(&to, b"collision").unwrap();
    let f = publish_leaf(prepared, &slot, &destination, &epoch).unwrap_err();
    assert_eq!(f.completed, Progress::default());
    assert_eq!(fs::read(&to).unwrap(), b"collision");
    scope.finish(Err(f), DirectoryEnd::Retained, false, &epoch).unwrap_err();
}
