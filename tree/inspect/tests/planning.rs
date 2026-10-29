#[path = "../../../tests/support/fixture.rs"]
mod support;

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use diff_dirs_tree_inspect::{Action, CheckError, Execution, NoCheck, NoScope, NoVisitor, Operations, PairedRoots, Visitor};
use support::Fixture;

#[test]
fn paired_child_lowering_admits_only_finalized_paired_directories() {
    struct Stop;
    impl Visitor for Stop {
        fn late(&self, batch: &mut Operations) -> Result<(), CheckError> {
            for entry in batch.entries_mut() {
                if entry.name() == "stopped" {
                    entry.preserve_tree();
                }
            }
            Ok(())
        }
    }
    let f = Fixture::new();
    for name in ["a", "z", "stopped"] {
        f.both_dir(name);
    }
    std::fs::create_dir(f.original.join("source-only")).unwrap();
    std::fs::create_dir(f.target.join("target-only")).unwrap();
    std::fs::create_dir(f.original.join("collision")).unwrap();
    support::file(&f.target.join("collision"), b"leaf");
    let roots = PairedRoots::new(&f.original, &f.target).unwrap();
    let plan = roots.inspect_visitor(PathBuf::new(), Execution::Sequential, &Stop).unwrap();
    let children: Vec<_> = plan.paired_children().collect();
    assert_eq!(children, ["a", "z"]);
    assert_eq!(plan.retained(), 1);
    assert_eq!(plan.work(), 3); // opaque source/target directories and collision
    assert!(plan.operations().entries().iter().all(|entry| entry.action().is_some()));
    // Immediate inspection did not traverse even the admitted child directories.
    assert!(plan.operations().entries().iter().all(|entry| entry.relative_path().components().count() == 1));
}

#[test]
fn binding_checks_run_once_in_order_and_preserved_root_is_not_listed() {
    let f = Fixture::new();
    let seen = Mutex::new(Vec::new());
    let check = |entry: diff_dirs_tree_inspect::Entry<'_>| {
        assert_eq!(entry.relative_path(), Path::new(""));
        seen.lock().unwrap().push(entry.side);
        Ok(())
    };
    let scope = |_: &Path| diff_dirs_tree_inspect::Visit::PreserveTree;
    let (roots, visitor) = PairedRoots::bind_scoped(&f.original, &f.target, check, scope).unwrap();
    // Owned fixture only: mandatory root observations must be reused and the
    // preserved root must not trigger listing. No live-writer guarantee is made.
    std::fs::rename(&f.original, f.original.with_file_name("moved-source")).unwrap();
    std::fs::rename(&f.target, f.target.with_file_name("moved-target")).unwrap();
    let plan = roots.inspect_visitor(PathBuf::new(), Execution::Sequential, &visitor).unwrap();
    assert_eq!(
        *seen.lock().unwrap(),
        [diff_dirs_tree_inspect::Side::Original, diff_dirs_tree_inspect::Side::Target]
    );
    assert!(plan.operations().is_root());
    assert!(plan.diff().is_empty());
    assert_eq!(plan.excluded(), 1);
    assert_eq!(plan.retained(), 1);
    assert_eq!(plan.paired_children().count(), 0);
    assert_eq!(plan.operations().entries().get(0).unwrap().action(), Some(Action::PreserveTree));
}

#[test]
fn binding_failure_precedence_is_unchanged_and_cannot_be_skipped_by_scope() {
    let f = Fixture::new();
    let original = f.original.join("file");
    support::file(&original, b"not a directory");
    let refuse = |_: diff_dirs_tree_inspect::Entry<'_>| -> Result<(), CheckError> { Err("root refusal before structural type rejection".into()) };
    let Err(error) = PairedRoots::bind_scoped(&original, f.target.join("missing"), refuse, NoScope) else {
        panic!("binding must refuse the original root")
    };
    assert!(matches!(error, diff_dirs_tree_inspect::Error::Refused { .. }));
    assert_eq!(error.path(), original);
    let error = PairedRoots::bind_scoped(&original, f.target.join("missing"), NoCheck, NoScope).unwrap_err();
    assert!(matches!(error, diff_dirs_tree_inspect::Error::NotDirectory { .. }));
    assert_eq!(error.path(), original);

    let roots = PairedRoots::new(&f.original, &f.target).unwrap();
    let error = roots
        .inspect_visitor(PathBuf::from("../outside"), Execution::Sequential, &NoVisitor)
        .unwrap_err();
    assert!(matches!(error, diff_dirs_tree_inspect::Error::Refused { .. }));
}
