#[path = "../../../tests/support/fixture.rs"]
mod support;

use std::path::Path;
use std::sync::Mutex;

use diff_dirs_tree_inspect::{CheckError, Execution, FactOrigin, Names, Operations, SharedRoots, TargetVisitor, Visitor};
use support::Fixture;

#[test]
fn public_shared_inspector_accepts_selected_heads_without_advancing_any_traversal() {
    struct Facts<'a>(&'a Mutex<usize>);
    impl Visitor for Facts<'_> {
        fn early(&self, names: &mut Names) -> Result<(), CheckError> {
            assert!(!names.is_root()); // explicitly selected nonroot head
            assert_eq!(names.dir(), Path::new("head"));
            assert_eq!(names.entries().len(), 2);
            for name in names.entries() {
                assert_eq!(name.source().unwrap().origin(), FactOrigin::Listing);
                #[cfg(all(unix, not(all(target_os = "linux", feature = "native-dirent"))))]
                assert_eq!(name.source().unwrap().kind(), diff_dirs_tree_inspect::FileKind::Unknown);
            }
            Ok(())
        }
        fn late(&self, operations: &mut Operations) -> Result<(), CheckError> {
            assert_eq!(operations.dir(), Path::new("head"));
            assert!(!operations.is_root());
            for operation in operations.entries() {
                assert!(operation.source().is_some());
                if operation.name() == "child" {
                    assert!(operation.source().unwrap().metadata().is_dir());
                    assert!(operation.target().unwrap().metadata().is_dir());
                }
                *self.0.lock().unwrap() += 1;
            }
            Ok(())
        }
    }
    let f = Fixture::new();
    f.both_dir("head/child");
    support::file(&f.original.join("head/new"), b"source");
    let count = Mutex::new(0);
    let visitor = Facts(&count);
    let roots = SharedRoots::new(&f.original, [&f.target, &f.target]).unwrap();
    assert_eq!(roots.source_root(), f.original);
    assert_eq!(roots.target_roots().count(), 2);
    let outcomes = roots
        .inspect_visitor(Path::new("head"), &[TargetVisitor { target: 1, visitor: &visitor }], Execution::Sequential)
        .unwrap();
    assert_eq!(outcomes.len(), 1);
    let outcome = outcomes.into_iter().next().unwrap();
    assert_eq!(outcome.target, 1);
    let (diff, coverage, children, root) = outcome.outcome.unwrap().into_parts();
    assert_eq!(diff.dir, Path::new("head"));
    assert_eq!(diff.files.added, ["new"]);
    assert_eq!(children, ["child"]);
    assert!(!root);
    assert_eq!(coverage.planned, 1);
    assert_eq!(*count.lock().unwrap(), 2); // neither binding 0 nor child visited
}

#[test]
fn consumer_lowering_preserves_each_validated_action_and_exclusion_without_metadata() {
    use diff_dirs_tree_inspect::{Action, Container, DirectoryComparison, Presence};
    use std::ffi::OsString;

    struct Select(bool);
    impl Visitor for Select {
        fn early(&self, names: &mut Names) -> Result<(), CheckError> {
            if names.is_root() {
                return Ok(());
            }
            for name in names.entries_mut() {
                if name.name() == "excluded" {
                    name.preserve_tree();
                }
            }
            Ok(())
        }
        fn late(&self, operations: &mut Operations) -> Result<(), CheckError> {
            if !operations.is_root() && self.0 {
                for entry in operations.entries_mut() {
                    if entry.name() == "leaf" {
                        entry.preserve_tree();
                    }
                    if entry.name() == "paired" {
                        entry.set_action(Action::Directory {
                            container: Container::Keep,
                            descend: false,
                        });
                    }
                }
            }
            Ok(())
        }
    }
    // These owned, metadata-free types stand in for a consumer's semantic
    // decision signature. No persistence or application protocol is implemented.
    #[derive(Debug, PartialEq, Eq)]
    struct Slot {
        name: OsString,
        presence: Presence,
        comparison: Option<bool>,
        action: Option<Action>,
        baseline: Option<Action>,
        descend: bool,
    }
    struct Decision {
        excluded: Vec<OsString>,
        slots: Vec<Slot>,
        comparison: DirectoryComparison,
    }
    let f = Fixture::new();
    f.both_dir("paired");
    f.both_file("excluded", b"excluded");
    support::file(&f.original.join("leaf"), b"source");
    support::file(&f.target.join("leaf"), b"old");
    let roots = SharedRoots::new(&f.original, [&f.target, &f.target]).unwrap();
    let visitors = [Select(true), Select(false)];
    let requests = visitors
        .iter()
        .enumerate()
        .map(|(target, visitor)| TargetVisitor { target, visitor })
        .collect::<Vec<_>>();
    let mut results = roots
        .inspect_with(Path::new(""), &requests, Execution::Sequential, |target, plan| {
            assert_eq!(plan.operations().target_root(), f.target);
            assert!(target < 2);
            let excluded = plan.excluded_names().to_vec();
            let slots = plan
                .operations()
                .entries()
                .iter()
                .map(|entry| Slot {
                    name: entry.name().to_os_string(),
                    presence: entry.presence(),
                    comparison: entry.comparison(),
                    action: entry.action(),
                    baseline: entry.baseline(),
                    descend: entry.descends(),
                })
                .collect();
            Decision {
                excluded,
                slots,
                comparison: plan.into_comparison(),
            }
        })
        .unwrap()
        .into_iter();
    let a = results.next().unwrap().outcome.unwrap();
    let b = results.next().unwrap().outcome.unwrap();
    assert_eq!(a.excluded, ["excluded"]);
    assert_eq!(b.excluded, a.excluded);
    assert_eq!(
        a.slots[0],
        Slot {
            name: OsString::from("leaf"),
            presence: Presence::Both,
            comparison: Some(true),
            action: Some(Action::PreserveTree),
            baseline: Some(Action::InstallLeaf),
            descend: false,
        }
    );
    assert_eq!(b.slots[0].action, Some(Action::InstallLeaf));
    assert_eq!(
        a.slots[1].action,
        Some(Action::Directory {
            container: Container::Keep,
            descend: false
        })
    );
    assert_eq!(
        b.slots[1].action,
        Some(Action::Directory {
            container: Container::Keep,
            descend: true
        })
    );
    assert!(a.comparison.paired_children().is_empty());
    assert_eq!(b.comparison.paired_children(), [OsString::from("paired")]);
    assert_eq!(a.comparison.diff(), b.comparison.diff()); // action != observation
}

#[test]
fn public_already_admitted_paths_only_run_directory_hooks_and_lower_successes() {
    struct AlreadyAccepted(bool);
    impl Visitor for AlreadyAccepted {
        fn early(&self, names: &mut Names) -> Result<(), CheckError> {
            assert!(!names.is_root(), "structural hooks must not replay");
            if self.0 {
                return Err(names.entries()[0].refuse("directory batch refused"));
            }
            Ok(())
        }
        fn late(&self, operations: &mut Operations) -> Result<(), CheckError> {
            assert!(!operations.is_root(), "structural hooks must not replay");
            Ok(())
        }
    }
    let f = Fixture::new();
    support::file(&f.original.join("source"), b"source");
    let roots = SharedRoots::new(&f.original, [&f.target, &f.target]).unwrap();
    let visitors = [AlreadyAccepted(false), AlreadyAccepted(true)];
    let requests = visitors
        .iter()
        .enumerate()
        .map(|(target, visitor)| TargetVisitor { target, visitor })
        .collect::<Vec<_>>();
    let mut lowered = Vec::new();
    let results = roots
        .inspect_admitted_with(Path::new(""), &requests, Execution::Sequential, |target, plan| {
            lowered.push(target);
            plan.into_comparison()
        })
        .unwrap();
    assert_eq!(lowered, [0]);
    assert!(results[0].outcome.is_ok());
    assert!(results[1].outcome.is_err());
    let results = roots.inspect_admitted_visitor(Path::new(""), &requests[..1], Execution::Sequential).unwrap();
    assert_eq!(results.len(), 1);
    assert!(!results[0].outcome.as_ref().unwrap().is_root());
}
