mod support;
use diff_dirs_tree_api as diff_dirs;

#[cfg(feature = "parallel")]
use std::path::Path;
use std::sync::Mutex;

use diff_dirs::{Action, CheckError, Names, Operations, Presence, Visitor};
use support::Fixture;

struct Select<'a> {
    seen: &'a Mutex<Vec<String>>,
}
impl Visitor for Select<'_> {
    fn early(&self, batch: &mut Names) -> Result<(), CheckError> {
        if !batch.is_root() && batch.dir().as_os_str().is_empty() {
            let facts: Vec<_> = batch
                .entries()
                .iter()
                .map(|n| (n.name().to_string_lossy().into_owned(), n.presence()))
                .collect();
            assert_eq!(
                facts,
                [
                    (".jj".into(), Presence::Both),
                    ("only-source".into(), Presence::SourceOnly),
                    ("only-target".into(), Presence::TargetOnly),
                    ("same".into(), Presence::Both),
                    ("stop".into(), Presence::Both)
                ]
            );
        }
        for entry in batch.entries_mut() {
            if entry.name() == ".jj" {
                entry.preserve_tree();
            }
        }
        Ok(())
    }
    fn late(&self, batch: &mut Operations) -> Result<(), CheckError> {
        for entry in batch.entries_mut() {
            assert!(!entry.relative_path().starts_with(".jj"));
            assert!(!entry.relative_path().starts_with("stop/never"));
            self.seen.lock().unwrap().push(entry.relative_path().to_string_lossy().into_owned());
            if entry.name() == "same" {
                assert!(!entry.is_different());
                assert_eq!(entry.action(), Some(Action::KeepLeaf));
                assert!(entry.source().unwrap().metadata().is_file());
                assert!(entry.target().unwrap().metadata().is_file());
            }
            if entry.name() == "stop" {
                entry.preserve_tree();
            }
        }
        Ok(())
    }
}

#[test]
fn early_union_presence_is_immutable_and_clean_late_decisions_gate_descent() {
    let f = Fixture::new();
    f.both_file("same", b"equal");
    f.both_dir("stop/never");
    f.both_file("stop/never/payload", b"not admitted");
    support::file(&f.original.join("only-source"), b"source");
    support::file(&f.target.join("only-target"), b"target");
    #[cfg(unix)]
    let _specials = (
        std::os::unix::net::UnixListener::bind(f.original.join(".jj")).unwrap(),
        std::os::unix::net::UnixListener::bind(f.target.join(".jj")).unwrap(),
    );
    #[cfg(not(unix))]
    f.both_dir(".jj");
    let seen = Mutex::new(Vec::new());
    let mut walk = diff_dirs::walk_with_visitor(&f.original, &f.target, Select { seen: &seen }).unwrap();
    let plan = walk.next().unwrap().unwrap();
    assert_eq!(plan.excluded(), 1);
    assert_eq!(plan.work(), 2);
    assert_eq!(plan.diff().files.added, ["only-source"]);
    assert!(walk.next().is_none());
    let summary = walk.into_summary();
    assert_eq!(summary.examined_directories, 1);
    assert_eq!(summary.planned, 2);
    assert_eq!(summary.retained, 1);
    assert!(!summary.is_complete());
    assert!(!summary.is_clean());
    assert!(seen.lock().unwrap().iter().any(|p| p == "same"));
}

struct ForceEqual;
impl Visitor for ForceEqual {
    fn late(&self, batch: &mut Operations) -> Result<(), CheckError> {
        for entry in batch.entries_mut() {
            if entry.name() == "equal" {
                entry.set_action(Action::InstallLeaf);
            }
        }
        Ok(())
    }
}

#[test]
fn equal_leaf_force_install_is_not_implicitly_authorized() {
    let f = Fixture::new();
    f.both_file("equal", b"same stat keys");
    let mut walk = diff_dirs::walk_with_visitor(&f.original, &f.target, ForceEqual).unwrap();
    let error = walk.next().unwrap().unwrap_err();
    assert_eq!(error.path(), f.target.join("equal"));
    assert!(walk.next().is_none());
    assert_eq!(walk.into_summary().examined_directories, 0);
}

#[test]
fn disabled_visitor_is_a_semantic_no_callback_promise() {
    struct Disabled;
    impl Visitor for Disabled {
        const ENABLED: bool = false;
        fn early(&self, _: &mut Names) -> Result<(), CheckError> {
            panic!("disabled early")
        }
        fn late(&self, _: &mut Operations) -> Result<(), CheckError> {
            panic!("disabled late")
        }
    }
    let f = Fixture::new();
    f.both_file("same", b"same");
    let mut walk = diff_dirs::walk_with_visitor(&f.original, &f.target, Disabled).unwrap();
    assert!(walk.next().is_none());
    assert!(walk.into_summary().is_clean());
}

#[test]
fn swapping_opaque_slots_is_rejected_rather_than_changing_native_error_order() {
    struct Reorder(bool);
    impl Visitor for Reorder {
        fn early(&self, batch: &mut Names) -> Result<(), CheckError> {
            if self.0 && !batch.is_root() {
                let mut entries = batch.entries_mut();
                if let (Some(a), Some(b)) = (entries.next(), entries.next()) {
                    std::mem::swap(a, b);
                }
            }
            Ok(())
        }
        fn late(&self, batch: &mut Operations) -> Result<(), CheckError> {
            if !self.0 && !batch.is_root() {
                let mut entries = batch.entries_mut();
                if let (Some(a), Some(b)) = (entries.next(), entries.next()) {
                    std::mem::swap(a, b);
                }
            }
            Ok(())
        }
    }
    let f = Fixture::new();
    f.both_file("a", b"a");
    f.both_file("b", b"b");
    for early in [false, true] {
        let mut walk = diff_dirs::walk_with_visitor(&f.original, &f.target, Reorder(early)).unwrap();
        assert!(walk.next().unwrap().is_err());
        assert_eq!(walk.into_summary().examined_directories, 0);
    }
}

#[test]
fn named_refusal_uses_actual_slot_while_arbitrary_error_paths_remain_only_causes() {
    struct Refuse {
        early: bool,
        inject: bool,
    }
    impl Visitor for Refuse {
        fn early(&self, batch: &mut Names) -> Result<(), CheckError> {
            if self.early && !batch.is_root() {
                return Err(batch.entries()[0].refuse("early name refusal"));
            }
            Ok(())
        }
        fn late(&self, batch: &mut Operations) -> Result<(), CheckError> {
            if !self.early && !batch.is_root() {
                if self.inject {
                    return Err(Box::new(diff_dirs::Error::UnsupportedEntry { path: "invented-path".into() }));
                }
                return Err(batch.entries()[0].refuse("late observed refusal"));
            }
            Ok(())
        }
    }
    let f = Fixture::new();
    f.both_file("leaf", b"same");
    for early in [false, true] {
        let mut walk = diff_dirs::walk_with_visitor(&f.original, &f.target, Refuse { early, inject: false }).unwrap();
        assert_eq!(walk.next().unwrap().unwrap_err().path(), f.target.join("leaf"));
    }
    let mut walk = diff_dirs::walk_with_visitor(&f.original, &f.target, Refuse { early: false, inject: true }).unwrap();
    assert_eq!(walk.next().unwrap().unwrap_err().path(), f.target);
}

#[cfg(unix)]
#[test]
fn late_special_preservation_is_uncompared_not_a_normal_leaf_delta() {
    use std::os::unix::fs::FileTypeExt;
    use std::os::unix::net::UnixListener;

    struct PreserveSpecials<'a>(&'a Mutex<Vec<Presence>>);
    impl Visitor for PreserveSpecials<'_> {
        fn late(&self, batch: &mut Operations) -> Result<(), CheckError> {
            for entry in batch.entries_mut() {
                assert!(!entry.relative_path().starts_with("d-special-dir") || entry.relative_path() == std::path::Path::new("d-special-dir"));
                if [entry.source(), entry.target()]
                    .into_iter()
                    .flatten()
                    .any(|facts| facts.metadata().file_type().is_socket())
                {
                    self.0.lock().unwrap().push(entry.presence());
                    assert_eq!(entry.comparison(), None);
                    assert!(!entry.is_different()); // not a claim of equality
                    assert_eq!(entry.baseline(), None);
                    assert_eq!(entry.action(), None);
                    entry.preserve_tree();
                    assert_eq!(entry.action(), Some(Action::PreserveTree));
                }
            }
            Ok(())
        }
    }
    let f = Fixture::new();
    let _sockets = [
        UnixListener::bind(f.original.join("a-source")).unwrap(),
        UnixListener::bind(f.target.join("b-target")).unwrap(),
        UnixListener::bind(f.original.join("c-special-file")).unwrap(),
        UnixListener::bind(f.original.join("d-special-dir")).unwrap(),
    ];
    support::file(&f.target.join("c-special-file"), b"preserved peer");
    std::fs::create_dir(f.target.join("d-special-dir")).unwrap();
    support::file(&f.target.join("d-special-dir/never"), b"no child admission");
    let seen = Mutex::new(Vec::new());
    let visitor = PreserveSpecials(&seen);
    let plan = diff_dirs::inspect_directory(
        &f.original,
        &f.target,
        std::path::Path::new(""),
        Some(&f.original),
        Some(&f.target),
        diff_dirs::Purpose::Compare,
        &visitor,
    )
    .unwrap();
    assert!(plan.diff().is_empty());
    assert_eq!(plan.work(), 0);
    assert_eq!(plan.retained(), 4);
    assert_eq!(plan.excluded(), 0); // metadata was intentionally observed late
    assert_eq!(
        *seen.lock().unwrap(),
        [Presence::SourceOnly, Presence::TargetOnly, Presence::Both, Presence::Both]
    );
    let mut walk = diff_dirs::walk_with_visitor(&f.original, &f.target, visitor.borrowed()).unwrap();
    assert!(walk.next().is_none());
    let summary = walk.into_summary();
    assert_eq!(summary.counts, diff_dirs::EntryCounts::default());
    assert_eq!(summary.planned, 0);
    assert_eq!(summary.retained, 4);
    assert!(!summary.is_complete());
    assert!(!summary.is_clean());
    let audited = diff_dirs::audit_with_visitor(&f.original, diff_dirs::Side::Original, &visitor).unwrap();
    assert_eq!(audited.retained_specials, 3);
    assert_eq!(audited.files, 0);
    assert_eq!(audited.directories, 1);
    let error = diff_dirs::walk(&f.original, &f.target).unwrap().next().unwrap().unwrap_err();
    assert!(matches!(error, diff_dirs::Error::UnsupportedEntry { .. }));
    assert_eq!(error.path(), f.original.join("a-source"));
    assert!(matches!(
        diff_dirs::walk_with_visitor(f.original.join("a-source"), &f.target, visitor),
        Err(diff_dirs::Error::NotDirectory { .. })
    ));
}

#[cfg(feature = "parallel")]
#[test]
fn finalized_clean_descent_is_identical_at_k_zero_and_speculative_lookahead() {
    struct Steer<'a>(&'a Mutex<Vec<std::path::PathBuf>>);
    impl Visitor for Steer<'_> {
        fn late(&self, batch: &mut Operations) -> Result<(), CheckError> {
            for entry in batch.entries_mut() {
                assert!(!entry.relative_path().starts_with("closed/never"));
                self.0.lock().unwrap().push(entry.relative_path().to_path_buf());
                if entry.name() == "closed" {
                    entry.preserve_tree();
                }
            }
            Ok(())
        }
    }
    let f = Fixture::new();
    for name in ["a", "b", "closed/never"] {
        f.both_dir(name);
    }
    support::file(&f.original.join("a/new"), b"a");
    support::file(&f.original.join("b/new"), b"b");
    let pool = rayon::ThreadPoolBuilder::new().num_threads(2).build().unwrap();
    let mut reference = None;
    for k in [0, 1, 8] {
        let seen = Mutex::new(Vec::new());
        let limits = diff_dirs::ParallelLimits::new(std::num::NonZeroUsize::new(2).unwrap(), k);
        let mut walk = diff_dirs::walk_in_pool_visitor(&f.original, &f.target, &pool, limits, Steer(&seen)).unwrap();
        let paths: Vec<_> = walk.by_ref().map(|p| p.unwrap().diff().dir.clone()).collect();
        assert_eq!(paths, [Path::new("a"), Path::new("b")]);
        let calls = seen.lock().unwrap().len();
        assert!(walk.next().is_none());
        let summary = walk.into_summary();
        assert_eq!(seen.lock().unwrap().len(), calls); // joined, no detached visitor
        assert_eq!(summary.examined_directories, 3);
        if let Some(reference) = &reference {
            assert_eq!(&summary, reference);
        } else {
            reference = Some(summary);
        }
    }
}
