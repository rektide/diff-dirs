mod support;

use diff_dirs_tree_compare as compare;
use diff_dirs_tree_inspect as inspect;

#[test]
fn facade_and_owning_crates_share_types_and_finalized_plans() {
    let f = support::Fixture::new();
    f.both_dir("paired");
    support::file(&f.original.join("paired/new"), b"payload");
    let _: inspect::Execution = diff_dirs::WalkOptions::default().execution;
    let visitor: diff_dirs::NoVisitor = inspect::NoVisitor;
    let mut walk: diff_dirs::PlanWalk<_> = compare::walk_with_visitor(&f.original, &f.target, visitor).unwrap();
    let plan: inspect::DirectoryPlan = walk.next().unwrap().unwrap();
    let diff: &diff_dirs::DirDiff = plan.diff();
    assert_eq!(diff.dir, std::path::Path::new("paired"));
    assert_eq!(diff.files.added, ["new"]);
    assert!(walk.next().is_none());
    let summary: compare::Summary = walk.into_summary();
    let _: diff_dirs::EntryCounts = summary.counts.clone();
    assert_eq!(summary.counts.files.added, 1);
    assert!(summary.is_complete());

    let failure: inspect::Error = diff_dirs::walk(f.original.join("missing"), &f.target).unwrap_err();
    assert_eq!(failure.path(), f.original.join("missing"));
}
