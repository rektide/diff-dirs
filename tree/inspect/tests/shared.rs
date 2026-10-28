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
