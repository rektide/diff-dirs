#![allow(dead_code)]

// Dependency-free fixture code is also source-included by the owning crates'
// unit tests. Only this facade-level helper depends on traversal, so inspection
// never needs a test-only dependency pointing back up the crate graph.
#[path = "../../../../tests/support/fixture.rs"]
mod fixture;
pub use fixture::*;

use diff_dirs_tree_api as diff_dirs;

pub fn gather(f: &Fixture) -> (Vec<diff_dirs::DirDiff>, diff_dirs::Summary) {
    let mut walk = diff_dirs::walk(&f.original, &f.target).unwrap();
    let diffs = walk.by_ref().collect::<Result<Vec<_>, _>>().unwrap();
    let summary = walk.into_summary();
    #[cfg(feature = "parallel")]
    {
        let mut parallel = diff_dirs::walk_with_options(
            &f.original,
            &f.target,
            diff_dirs::WalkOptions {
                execution: diff_dirs::Execution::Parallel,
            },
        )
        .unwrap();
        let parallel_diffs = parallel.by_ref().collect::<Result<Vec<_>, _>>().unwrap();
        assert_eq!(diffs, parallel_diffs);
        assert_eq!(summary, parallel.into_summary());
    }
    (diffs, summary)
}
