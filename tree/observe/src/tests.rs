use super::*;
use std::error::Error as _;

#[test]
fn physical_errors_retain_path_operation_and_io_cause() {
    let f = test_support::Fixture::new();
    let missing = f.original.join("missing");
    for (error, operation) in [
        (read_directory(&missing).err().unwrap(), "read directory"),
        (symlink_metadata(&missing).unwrap_err(), "inspect entry"),
        (read_link(&missing).unwrap_err(), "read symlink"),
    ] {
        assert_eq!(error.path(), missing);
        assert_eq!(error.operation(), operation);
        assert_eq!(error.io_error().kind(), std::io::ErrorKind::NotFound);
        let cause = error.source().unwrap().downcast_ref::<std::io::Error>().unwrap();
        assert!(std::ptr::eq(cause, error.io_error()));
        let raw = cause.raw_os_error();
        let (path, actual_operation, cause) = error.into_parts();
        assert_eq!(path, missing);
        assert_eq!(actual_operation, operation);
        assert_eq!(cause.raw_os_error(), raw);
    }
}

#[test]
fn listing_accessors_reuse_cached_facts_and_metadata_reports_the_listed_path() {
    let f = test_support::Fixture::new();
    let path = f.original.join("leaf");
    fs::write(&path, b"listed then removed").unwrap();
    let entry = read_directory(&f.original).unwrap().pop().unwrap();
    let hints = entry.hints();
    assert_eq!(hints.origin(), FactOrigin::Listing);
    #[cfg(all(unix, not(all(target_os = "linux", feature = "native-dirent"))))]
    assert_eq!(hints.kind(), FileKind::Unknown);
    fs::remove_file(&path).unwrap();
    assert_eq!(entry.file_name(), "leaf");
    assert_eq!(entry.path(), path);
    assert_eq!(entry.hints(), hints); // no lookup, even after the entry disappears
    #[cfg(not(windows))]
    {
        let error = entry.metadata().unwrap_err();
        assert_eq!(error.path(), path);
        assert_eq!(error.operation(), "inspect entry");
        assert_eq!(error.io_error().kind(), std::io::ErrorKind::NotFound);
        assert!(error.source().unwrap().is::<std::io::Error>());
    }
    // Windows deliberately retains enumeration metadata even after removal.
    #[cfg(windows)]
    assert!(entry.metadata().unwrap().is_file());
}

#[cfg(target_os = "linux")]
#[test]
fn listed_metadata_uses_the_held_parent_not_a_reconstructed_lookup() {
    let f = test_support::Fixture::new();
    fs::write(f.original.join("leaf"), b"held directory metadata").unwrap();
    let entry = read_directory(&f.original).unwrap().pop().unwrap();
    let old_path = entry.path();
    // Owned fixture only, not public traversal under concurrent writers.
    fs::rename(&f.original, f.original.with_file_name("renamed-source")).unwrap();
    assert!(!old_path.exists());
    let metadata = entry.metadata().unwrap();
    assert!(metadata.is_file());
    assert_eq!(metadata.len(), b"held directory metadata".len() as u64);
}

#[cfg(unix)]
#[test]
fn metadata_never_follows_links_and_readlink_stays_literal() {
    let f = test_support::Fixture::new();
    let path = f.original.join("link");
    let literal = Path::new("../target//./missing");
    std::os::unix::fs::symlink(literal, &path).unwrap();
    let entry = read_directory(&f.original).unwrap().pop().unwrap();
    let listed = entry.metadata().unwrap();
    let direct = symlink_metadata(&path).unwrap();
    for metadata in [&listed, &direct] {
        assert!(metadata.is_symlink());
        assert!(!metadata.is_dir());
        let root_facts = EarlyFacts::from_root_metadata(metadata);
        assert_eq!(root_facts.kind(), FileKind::Symlink);
        assert_eq!(root_facts.origin(), FactOrigin::RootMetadata);
        assert_eq!(root_facts.inode(), Some(std::os::unix::fs::MetadataExt::ino(metadata)));
    }
    assert_eq!(read_link(&path).unwrap().as_os_str(), literal.as_os_str());
}
