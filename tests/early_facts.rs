mod support;

use std::path::Path;
use std::sync::Mutex;

use diff_dirs::{CheckError, EarlyFacts, FactOrigin, FileKind, Names, NoVisitor, Operations, Presence, Purpose, Side, Visitor};
use support::Fixture;

fn expected_kind(kind: FileKind) -> FileKind {
    if cfg!(all(target_os = "linux", feature = "native-dirent")) || cfg!(windows) {
        kind
    } else {
        FileKind::Unknown
    }
}

type Captured = (String, Option<EarlyFacts>, Option<EarlyFacts>);
struct Capture<'a>(&'a Mutex<Vec<Captured>>);
impl Visitor for Capture<'_> {
    fn early(&self, names: &mut Names) -> Result<(), CheckError> {
        for name in names.entries_mut() {
            assert_eq!(
                name.presence(),
                match (name.source(), name.target()) {
                    (Some(_), Some(_)) => Presence::Both,
                    (Some(_), None) => Presence::SourceOnly,
                    (None, Some(_)) => Presence::TargetOnly,
                    _ => panic!("empty union slot"),
                }
            );
            for facts in [name.source(), name.target()].into_iter().flatten() {
                assert_eq!(
                    facts.origin(),
                    if names_root(name.relative_path()) {
                        FactOrigin::RootMetadata
                    } else {
                        FactOrigin::Listing
                    }
                );
            }
            self.0.lock().unwrap().push((
                name.relative_path().to_string_lossy().into_owned(),
                name.source().copied(),
                name.target().copied(),
            ));
            if name.name() == "socket" {
                name.preserve_tree();
            }
        }
        Ok(())
    }
    fn late(&self, operations: &mut Operations) -> Result<(), CheckError> {
        for operation in operations.entries() {
            if operation.name() == "link" {
                let facts = operation.source().unwrap();
                assert!(facts.metadata().is_symlink());
                assert_eq!(facts.link_target().unwrap().as_os_str(), "missing//./literal");
            }
        }
        Ok(())
    }
}
fn names_root(relative: &Path) -> bool {
    relative.as_os_str().is_empty()
}

#[test]
fn actual_listings_expose_side_specific_hints_without_manufacturing_absence() {
    let f = Fixture::new();
    std::fs::create_dir(f.original.join("collision")).unwrap();
    support::file(&f.target.join("collision"), b"file peer");
    support::file(&f.original.join("source"), b"source only");
    std::fs::create_dir(f.target.join("target")).unwrap();
    #[cfg(unix)]
    std::os::unix::fs::symlink("missing//./literal", f.original.join("link")).unwrap();
    #[cfg(unix)]
    let _socket = std::os::unix::net::UnixListener::bind(f.target.join("socket")).unwrap();
    let seen = Mutex::new(Vec::new());
    let plan = diff_dirs::inspect_directory(
        &f.original,
        &f.target,
        Path::new(""),
        Some(&f.original),
        Some(&f.target),
        Purpose::Compare,
        &Capture(&seen),
    )
    .unwrap();
    assert_eq!(plan.diff().dirs.added, ["collision"]);
    let seen = seen.lock().unwrap();
    let entry = |name: &str| seen.iter().find(|e| e.0 == name).unwrap();
    assert_eq!(entry("collision").1.unwrap().kind(), expected_kind(FileKind::Directory));
    assert_eq!(entry("collision").2.unwrap().kind(), expected_kind(FileKind::RegularFile));
    assert!(entry("source").2.is_none());
    assert!(entry("target").1.is_none());
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        for (relative, source, target) in &*seen {
            for (root, facts) in [(&f.original, source), (&f.target, target)] {
                if let Some(facts) = facts {
                    assert_eq!(facts.inode(), Some(std::fs::symlink_metadata(root.join(relative)).unwrap().ino()));
                }
            }
        }
        assert_eq!(entry("link").1.unwrap().kind(), expected_kind(FileKind::Symlink));
        assert_eq!(entry("socket").2.unwrap().kind(), expected_kind(FileKind::Socket));
    }
}

#[test]
fn root_singletons_have_mandatory_metadata_provenance_on_each_present_side() {
    let f = Fixture::new();
    let seen = Mutex::new(Vec::new());
    let visitor = Capture(&seen);
    let root = diff_dirs::inspect_root(&f.original, &f.target, Purpose::Compare, &visitor).unwrap();
    assert!(root.operations().is_root());
    for side in [Side::Original, Side::Target] {
        diff_dirs::inspect_single_root(&f.original, side, Purpose::Audit, &visitor).unwrap();
    }
    diff_dirs::inspect_root(&f.original, &f.target.join("missing"), Purpose::Compare, &visitor).unwrap();
    let seen = seen.lock().unwrap();
    assert_eq!(seen.len(), 4);
    assert!(seen[1].2.is_none());
    assert!(seen[2].1.is_none());
    assert!(seen[3].2.is_none());
    for (_, source, target) in &*seen {
        for facts in [source, target].into_iter().flatten() {
            assert_eq!(facts.kind(), FileKind::Directory);
            assert_eq!(facts.origin(), FactOrigin::RootMetadata);
        }
    }
    // Native-enabled and default visitors preserve the same baseline semantics.
    let mut walk = diff_dirs::walk_with_visitor(&f.original, &f.target, NoVisitor).unwrap();
    assert!(walk.next().is_none());
    assert!(walk.into_summary().is_clean());
}
