mod support;

use diff_dirs::{CheckError, Entry, NoCheck, Visit};
use std::path::Path;
use support::{Fixture, file};

fn scope(path: &Path) -> Visit {
    if path.file_name().is_some_and(|name| name == "private") {
        Visit::PreserveTree
    } else {
        Visit::Reconcile
    }
}

#[test]
fn scope_is_pair_wide_and_precedes_observation_of_equal_and_one_sided_names() {
    let f = Fixture::new();
    f.both_dir("private");
    file(&f.original.join("private/hidden"), b"source");
    file(&f.target.join("private/hidden"), b"different");
    f.both_file("equal", b"same");
    let check = |entry: Entry<'_>| -> Result<(), CheckError> {
        assert!(!entry.relative_path().starts_with("private"));
        Ok(())
    };
    let mut walk = diff_dirs::walk_scoped(&f.original, &f.target, check, scope).unwrap();
    assert!(walk.next().is_none());
    let summary = walk.into_summary();
    assert_eq!(summary.excluded, 1); // once per union name, not once per side
    assert!(!summary.is_clean());
    assert!(!summary.is_complete());
    assert_eq!(summary.state, diff_dirs::ScanState::Complete);
    std::fs::remove_dir_all(f.original.join("private")).unwrap();
    let mut walk = diff_dirs::walk_scoped(&f.original, &f.target, NoCheck, scope).unwrap();
    assert!(walk.next().is_none()); // hidden source absence is not a removal
    assert!(!diff_dirs::scope_includes(&scope, Path::new("private/fabricated/reincluded")));
}

#[cfg(unix)]
#[test]
fn excluded_special_entries_and_unreadable_interiors_are_not_inspected() {
    use std::os::unix::fs::PermissionsExt;
    let f = Fixture::new();
    let _socket = std::os::unix::net::UnixListener::bind(f.original.join("private")).unwrap();
    std::fs::create_dir(f.target.join("private")).unwrap();
    std::fs::set_permissions(f.target.join("private"), std::fs::Permissions::from_mode(0o000)).unwrap();
    let mut walk = diff_dirs::walk_scoped(&f.original, &f.target, NoCheck, scope).unwrap();
    assert!(walk.next().is_none());
    let audit = diff_dirs::audit_scoped(&f.original, diff_dirs::Side::Original, &NoCheck, &scope).unwrap();
    assert_eq!(audit.excluded, 1);
    std::fs::set_permissions(f.target.join("private"), std::fs::Permissions::from_mode(0o700)).unwrap();
}

#[cfg(feature = "parallel")]
#[test]
fn scoped_frontier_joins_policy_before_children_and_commits_only_prefix_counts() {
    let f = Fixture::new();
    for name in ["a", "b", "c"] {
        f.both_dir(format!("{name}/private"));
        file(&f.original.join(format!("{name}/private/hidden")), b"x");
    }
    file(&f.original.join("b/new"), b"x");
    let mut serial = diff_dirs::walk_scoped(&f.original, &f.target, NoCheck, scope).unwrap();
    let records = serial.by_ref().collect::<Result<Vec<_>, _>>().unwrap();
    let expected = serial.into_summary();
    let pool = rayon::ThreadPoolBuilder::new().num_threads(2).build().unwrap();
    for k in [0, 4] {
        let limits = diff_dirs::ParallelLimits::new(std::num::NonZeroUsize::new(2).unwrap(), k);
        let mut walk = diff_dirs::walk_in_pool_scoped(&f.original, &f.target, &pool, limits, NoCheck, scope).unwrap();
        assert_eq!(walk.by_ref().collect::<Result<Vec<_>, _>>().unwrap(), records);
        assert_eq!(walk.into_summary(), expected);
    }
}
