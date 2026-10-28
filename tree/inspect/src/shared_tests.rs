use std::error::Error as _;
use std::fs;
use std::io;
use std::sync::{Mutex, Weak};

use super::*;
use crate::PairedRoots;
use crate::test_support::{Fixture, file};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Call {
    Root,
    List,
    Stat,
    Link,
}

#[derive(Debug)]
struct Marker;
impl fmt::Display for Marker {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("original injected cause")
    }
}
impl std::error::Error for Marker {}

enum Fault {
    Raw(Call, PathBuf, i32),
    Custom(Call, PathBuf, Arc<Marker>),
}

#[derive(Default)]
struct Counted {
    calls: Mutex<Vec<(Call, PathBuf)>>,
    faults: Vec<Fault>,
    facts: Mutex<Vec<Weak<Facts>>>,
}

impl Counted {
    fn before(&self, call: Call, path: &Path, operation: &'static str) -> Result<(), Error> {
        self.calls.lock().unwrap().push((call, path.to_path_buf()));
        for fault in &self.faults {
            let error = match fault {
                Fault::Raw(c, p, raw) if *c == call && p == path => io::Error::from_raw_os_error(*raw),
                Fault::Custom(c, p, marker) if *c == call && p == path => io::Error::other(Arc::clone(marker)),
                _ => continue,
            };
            return Err(Error::io(path.to_path_buf(), operation, error));
        }
        Ok(())
    }
    fn count(&self, call: Call, path: &Path) -> usize {
        self.calls.lock().unwrap().iter().filter(|(c, p)| *c == call && p == path).count()
    }
    fn under(&self, call: Call, root: &Path) -> usize {
        self.calls.lock().unwrap().iter().filter(|(c, p)| *c == call && p.starts_with(root)).count()
    }
    fn assert_released(&self) {
        assert!(
            self.facts.lock().unwrap().iter().all(|facts| facts.upgrade().is_none()),
            "source fact retained after reduction"
        );
    }
}

impl Acquire for Counted {
    fn list(&self, path: &Path) -> Result<Vec<observe::Entry>, Error> {
        self.before(Call::List, path, "read directory")?;
        Physical.list(path)
    }
    fn metadata(&self, entry: observe::Entry) -> Result<Metadata, Error> {
        self.before(Call::Stat, &entry.path(), "inspect entry")?;
        Physical.metadata(entry)
    }
    fn root_metadata(&self, path: &Path) -> Result<Metadata, Error> {
        self.before(Call::Root, path, "inspect root")?;
        Physical.root_metadata(path)
    }
    fn link(&self, path: &Path) -> Result<PathBuf, Error> {
        self.before(Call::Link, path, "read symlink")?;
        Physical.link(path)
    }
    fn source_acquired(&self, facts: &Arc<Facts>) {
        self.facts.lock().unwrap().push(Arc::downgrade(facts));
    }
}

fn executions() -> &'static [Execution] {
    #[cfg(feature = "parallel")]
    {
        &[Execution::Sequential, Execution::Parallel]
    }
    #[cfg(not(feature = "parallel"))]
    {
        &[Execution::Sequential]
    }
}

fn targets(f: &Fixture, count: usize) -> Vec<PathBuf> {
    let mut targets = vec![f.target.clone()];
    for index in 1..count {
        let path = f.root.join(format!("target-{index}"));
        fs::create_dir(&path).unwrap();
        targets.push(path);
    }
    targets
}

#[derive(Clone, Copy, Default)]
enum Mode {
    #[default]
    Normal,
    PreserveRoot,
    RefuseEarly,
    RefuseLate,
    PreserveSpecials,
    ForceEqual,
}

#[derive(Default)]
struct Policy<'a> {
    mode: Mode,
    preserve: &'a [&'a str],
    stop: &'a [&'a str],
    census: Option<&'a Counted>,
    seen: Option<&'a Mutex<Vec<(PathBuf, usize)>>>,
}

impl Visitor for Policy<'_> {
    fn early(&self, names: &mut Names) -> Result<(), CheckError> {
        if names.is_root() {
            if matches!(self.mode, Mode::PreserveRoot) {
                names.entries_mut().next().unwrap().preserve_tree();
            }
            return Ok(());
        }
        if let Some(census) = self.census {
            assert!(!census.calls.lock().unwrap().iter().any(|(call, _)| matches!(call, Call::Stat | Call::Link)));
        }
        if matches!(self.mode, Mode::RefuseEarly) {
            return Err(names.entries()[0].refuse("early refused"));
        }
        for name in names.entries_mut() {
            if self.preserve.iter().any(|preserve| name.name() == *preserve) {
                name.preserve_tree();
            }
        }
        Ok(())
    }
    fn late(&self, operations: &mut Operations) -> Result<(), CheckError> {
        if operations.is_root() {
            return Ok(());
        }
        if matches!(self.mode, Mode::RefuseLate) {
            return Err(operations.entries()[0].refuse("late refused"));
        }
        for entry in operations.entries_mut() {
            if let (Some(seen), Some(source)) = (self.seen, entry.source()) {
                seen.lock()
                    .unwrap()
                    .push((entry.relative_path().to_path_buf(), std::ptr::from_ref(source) as usize));
            }
            if self.stop.iter().any(|stop| entry.name() == *stop) || (matches!(self.mode, Mode::PreserveSpecials) && entry.comparison().is_none()) {
                entry.preserve_tree();
            }
            if matches!(self.mode, Mode::ForceEqual) && entry.name() == "equal" {
                entry.set_action(Action::InstallLeaf);
            }
        }
        Ok(())
    }
}

fn requests<V>(visitors: &[V]) -> Vec<TargetVisitor<'_, V>> {
    visitors.iter().enumerate().map(|(target, visitor)| TargetVisitor { target, visitor }).collect()
}

fn io_cause(error: &Error) -> &io::Error {
    let Error::Io { source, .. } = error else {
        panic!("not physical I/O: {error:?}")
    };
    assert!(std::ptr::eq(error.source().unwrap().downcast_ref::<io::Error>().unwrap(), source));
    source
}

#[test]
fn source_once_facts_are_shared_and_released_after_reduced_publication() {
    for &execution in executions() {
        let f = Fixture::new();
        let targets = targets(&f, 2);
        file(&f.original.join("a-file"), b"source");
        file(&f.original.join("only-source"), b"source");
        file(&f.original.join("hidden"), b"excluded");
        fs::create_dir(f.original.join("c-dir")).unwrap();
        #[cfg(unix)]
        std::os::unix::fs::symlink("missing//./literal", f.original.join("b-link")).unwrap();
        for target in &targets {
            file(&target.join("a-file"), b"old");
            file(&target.join("hidden"), b"excluded");
            fs::create_dir(target.join("c-dir")).unwrap();
            #[cfg(unix)]
            {
                std::os::unix::fs::symlink("missing//./literal", target.join("b-link")).unwrap();
                std::os::unix::fs::symlink("not-needed", target.join("extra-link")).unwrap();
            }
        }
        let counted = Counted::default();
        let roots = SharedRoots::bind_using(&f.original, targets.clone(), &counted).unwrap();
        let seen = Mutex::new(Vec::new());
        let visitors = [
            Policy {
                preserve: &["hidden"],
                census: Some(&counted),
                seen: Some(&seen),
                ..Policy::default()
            },
            Policy {
                preserve: &["hidden"],
                census: Some(&counted),
                seen: Some(&seen),
                ..Policy::default()
            },
        ];
        let compared = roots.inspect_using(Path::new(""), &requests(&visitors), execution, &counted).unwrap();
        assert_eq!(counted.count(Call::Root, &f.original), 1);
        assert_eq!(counted.count(Call::List, &f.original), 1);
        assert_eq!(counted.count(Call::Stat, &f.original.join("hidden")), 0);
        for name in ["a-file", "c-dir", "only-source"] {
            assert_eq!(counted.count(Call::Stat, &f.original.join(name)), 1);
            let pointers: Vec<_> = seen
                .lock()
                .unwrap()
                .iter()
                .filter(|(path, _)| path == Path::new(name))
                .map(|(_, p)| *p)
                .collect();
            assert_eq!(pointers.len(), 2);
            assert_eq!(pointers[0], pointers[1], "full source facts were copied per lane");
        }
        #[cfg(unix)]
        {
            assert_eq!(counted.count(Call::Stat, &f.original.join("b-link")), 1);
            assert_eq!(counted.count(Call::Link, &f.original.join("b-link")), 1);
        }
        for (index, result) in compared.into_iter().enumerate() {
            assert_eq!(result.target, index);
            let comparison = result.outcome.unwrap();
            assert_eq!(comparison.coverage().excluded, 1);
            assert_eq!(comparison.paired_children(), [OsString::from("c-dir")]);
            assert_eq!(counted.count(Call::List, &targets[index]), 1);
            assert_eq!(counted.count(Call::Root, &targets[index]), 1);
            #[cfg(unix)]
            {
                assert_eq!(counted.count(Call::Link, &targets[index].join("b-link")), 1);
                assert_eq!(counted.count(Call::Link, &targets[index].join("extra-link")), 0);
            }
        }
        counted.assert_released();
    }
}

#[test]
fn full_group_reduction_matches_independent_pairs_for_different_shapes_and_policies() {
    for &execution in executions() {
        let f = Fixture::new();
        let targets = targets(&f, 3);
        for name in ["paired", "stopped", "missing", "collision"] {
            fs::create_dir(f.original.join(name)).unwrap();
        }
        file(&f.original.join("equal"), b"same");
        file(&f.original.join("changed"), b"different source");
        for target in &targets {
            fs::create_dir(target.join("paired")).unwrap();
            fs::create_dir(target.join("stopped")).unwrap();
            fs::create_dir(target.join("extra")).unwrap();
            file(&target.join("collision"), b"leaf");
            file(&target.join("equal"), b"same");
            file(&target.join("changed"), b"old");
        }
        #[cfg(unix)]
        {
            use std::os::unix::ffi::OsStringExt;
            let name = OsString::from_vec(vec![b'n', 0xff]);
            file(&f.original.join(name), b"native source-only");
            std::os::unix::fs::symlink("literal//./missing", f.original.join("link")).unwrap();
            for target in &targets {
                std::os::unix::fs::symlink("literal/missing", target.join("link")).unwrap();
            }
        }
        let visitors = [
            Policy::default(),
            Policy {
                preserve: &["missing", "changed"],
                stop: &["stopped"],
                ..Policy::default()
            },
            Policy {
                mode: Mode::PreserveRoot,
                ..Policy::default()
            },
        ];
        let roots = SharedRoots::new(&f.original, &targets).unwrap();
        let grouped = roots.inspect_visitor(Path::new(""), &requests(&visitors), execution).unwrap();
        for (index, outcome) in grouped.into_iter().enumerate() {
            let paired = PairedRoots::new(&f.original, &targets[index])
                .unwrap()
                .inspect_visitor(PathBuf::new(), execution, &visitors[index])
                .unwrap()
                .into_comparison();
            assert_eq!(outcome.outcome.unwrap(), paired);
        }
    }
}

#[test]
fn failed_source_fact_only_reaches_demanding_lanes_and_all_preservation_avoids_acquisition() {
    let f = Fixture::new();
    let targets = targets(&f, 2);
    file(&f.original.join("secret"), b"not universally excluded");
    for preserve_both in [false, true] {
        let counted = Counted {
            faults: vec![Fault::Raw(Call::Stat, f.original.join("secret"), 5)],
            ..Counted::default()
        };
        let roots = SharedRoots::bind_using(&f.original, targets.clone(), &counted).unwrap();
        let visitors = [
            Policy {
                preserve: &["secret"],
                ..Policy::default()
            },
            Policy {
                preserve: if preserve_both { &["secret"] } else { &[] },
                ..Policy::default()
            },
        ];
        let mut results = roots
            .inspect_using(Path::new(""), &requests(&visitors), Execution::Sequential, &counted)
            .unwrap()
            .into_iter();
        assert_eq!(results.next().unwrap().outcome.unwrap().coverage().excluded, 1);
        let second = results.next().unwrap().outcome;
        if preserve_both {
            assert_eq!(second.unwrap().coverage().excluded, 1);
        } else {
            let error = second.unwrap_err();
            assert_eq!(error.path(), f.original.join("secret"));
            assert_eq!(io_cause(&error).raw_os_error(), Some(5));
        }
        assert_eq!(counted.count(Call::Stat, &f.original.join("secret")), usize::from(!preserve_both));
        counted.assert_released();
    }
}

#[test]
fn root_preservation_reuses_binding_metadata_and_does_not_demand_failed_listing() {
    let f = Fixture::new();
    let targets = targets(&f, 2);
    for preserve_both in [false, true] {
        let counted = Counted {
            faults: vec![Fault::Raw(Call::List, f.original.clone(), 5)],
            ..Counted::default()
        };
        let roots = SharedRoots::bind_using(&f.original, targets.clone(), &counted).unwrap();
        let visitors = [
            Policy {
                mode: Mode::PreserveRoot,
                ..Policy::default()
            },
            Policy {
                mode: if preserve_both { Mode::PreserveRoot } else { Mode::Normal },
                ..Policy::default()
            },
        ];
        let mut results = roots
            .inspect_using(Path::new(""), &requests(&visitors), Execution::Sequential, &counted)
            .unwrap()
            .into_iter();
        let preserved = results.next().unwrap().outcome.unwrap();
        assert!(preserved.is_root());
        assert_eq!(preserved.coverage().excluded, 1);
        assert_eq!(preserved.coverage().retained, 1);
        assert!(preserved.paired_children().is_empty());
        assert_eq!(results.next().unwrap().outcome.is_ok(), preserve_both);
        assert_eq!(counted.under(Call::List, &f.original), usize::from(!preserve_both));
        assert_eq!(counted.under(Call::Stat, &f.original), 0);
        assert_eq!(counted.count(Call::Root, &f.original), 1);
    }
}

#[test]
fn source_listing_failure_is_shared_original_cause_before_target_listing_or_early() {
    let f = Fixture::new();
    let targets = targets(&f, 2);
    let marker = Arc::new(Marker);
    let counted = Counted {
        faults: vec![
            Fault::Custom(Call::List, f.original.clone(), Arc::clone(&marker)),
            Fault::Raw(Call::List, targets[0].clone(), 13),
        ],
        ..Counted::default()
    };
    let roots = SharedRoots::bind_using(&f.original, targets.clone(), &counted).unwrap();
    let visitors = [
        Policy {
            mode: Mode::RefuseEarly,
            ..Policy::default()
        },
        Policy::default(),
    ];
    let results = roots
        .inspect_using(Path::new(""), &requests(&visitors), Execution::Sequential, &counted)
        .unwrap();
    let a = results[0].outcome.as_ref().unwrap_err();
    let b = results[1].outcome.as_ref().unwrap_err();
    assert!(Arc::ptr_eq(a, b));
    let io = io_cause(a);
    assert!(Arc::ptr_eq(io.get_ref().unwrap().downcast_ref::<Arc<Marker>>().unwrap(), &marker));
    assert_eq!(counted.count(Call::List, &targets[0]), 0);
    assert_eq!(counted.count(Call::List, &targets[1]), 0);
}

#[test]
fn target_listing_and_early_failures_precede_cached_source_failures_per_lane() {
    let f = Fixture::new();
    let targets = targets(&f, 3);
    file(&f.original.join("z-source"), b"demanded by the third lane");
    let counted = Counted {
        faults: vec![
            Fault::Raw(Call::List, targets[0].clone(), 13),
            Fault::Raw(Call::Stat, f.original.join("z-source"), 5),
        ],
        ..Counted::default()
    };
    let roots = SharedRoots::bind_using(&f.original, targets.clone(), &counted).unwrap();
    let visitors = [
        Policy::default(),
        Policy {
            mode: Mode::RefuseEarly,
            ..Policy::default()
        },
        Policy::default(),
    ];
    let results = roots
        .inspect_using(Path::new(""), &requests(&visitors), Execution::Sequential, &counted)
        .unwrap();
    let a = results[0].outcome.as_ref().unwrap_err();
    assert_eq!(a.path(), targets[0]);
    assert_eq!(io_cause(a).raw_os_error(), Some(13));
    let b = results[1].outcome.as_ref().unwrap_err();
    assert!(matches!(b.as_ref(), Error::Refused { .. }));
    assert_eq!(b.path(), targets[1].join("z-source"));
    let c = results[2].outcome.as_ref().unwrap_err();
    assert_eq!(c.path(), f.original.join("z-source"));
    assert_eq!(io_cause(c).raw_os_error(), Some(5));
    assert_eq!(counted.count(Call::Stat, &f.original.join("z-source")), 1);
}

#[test]
fn earlier_target_fact_wins_over_later_source_fact_and_same_name_source_wins() {
    for &execution in executions() {
        for source_first in [false, true] {
            let f = Fixture::new();
            let targets = targets(&f, 2);
            for name in ["a-error", "z-error"] {
                file(&f.original.join(name), b"source");
                for target in &targets {
                    file(&target.join(name), b"target");
                }
            }
            // Force wide source and lane metadata phases, not merely the
            // Execution flag on a tiny serial batch.
            for index in 0..80 {
                let name = format!("m-{index:03}");
                file(&f.original.join(&name), b"same");
                for target in &targets {
                    file(&target.join(&name), b"same");
                }
            }
            let mut faults = vec![
                Fault::Raw(Call::Stat, targets[0].join("a-error"), 13),
                Fault::Raw(Call::Stat, f.original.join("z-error"), 5),
            ];
            if source_first {
                faults.push(Fault::Raw(Call::Stat, f.original.join("a-error"), 2));
            }
            let counted = Counted { faults, ..Counted::default() };
            let roots = SharedRoots::bind_using(&f.original, targets.clone(), &counted).unwrap();
            let visitors = [
                Policy::default(),
                Policy {
                    preserve: &["a-error"],
                    ..Policy::default()
                },
            ];
            let results = roots.inspect_using(Path::new(""), &requests(&visitors), execution, &counted).unwrap();
            let first = results[0].outcome.as_ref().unwrap_err();
            assert_eq!(
                first.path(),
                if source_first {
                    f.original.join("a-error")
                } else {
                    targets[0].join("a-error")
                }
            );
            assert_eq!(io_cause(first).raw_os_error(), Some(if source_first { 2 } else { 13 }));
            assert_eq!(results[1].outcome.as_ref().unwrap_err().path(), f.original.join("z-error"));
            assert_eq!(counted.count(Call::Stat, &f.original.join("z-error")), 1);
            if source_first {
                assert_eq!(counted.count(Call::Stat, &targets[0].join("a-error")), 0);
            }
            counted.assert_released();
        }
    }
}

#[cfg(unix)]
#[test]
fn source_readlink_failure_is_cached_once_and_keeps_name_side_precedence() {
    let f = Fixture::new();
    let targets = targets(&f, 3);
    file(&f.original.join("a-file"), b"source");
    std::os::unix::fs::symlink("missing", f.original.join("z-link")).unwrap();
    for target in &targets {
        file(&target.join("a-file"), b"target");
        file(&target.join("z-link"), b"target");
    }
    let counted = Counted {
        faults: vec![
            Fault::Raw(Call::Link, f.original.join("z-link"), 5),
            Fault::Raw(Call::Stat, targets[0].join("a-file"), 13),
        ],
        ..Counted::default()
    };
    let roots = SharedRoots::bind_using(&f.original, targets.clone(), &counted).unwrap();
    let visitors = [Policy::default(), Policy::default(), Policy::default()];
    let results = roots
        .inspect_using(Path::new(""), &requests(&visitors), Execution::Sequential, &counted)
        .unwrap();
    assert_eq!(results[0].outcome.as_ref().unwrap_err().path(), targets[0].join("a-file"));
    let a = results[1].outcome.as_ref().unwrap_err();
    let b = results[2].outcome.as_ref().unwrap_err();
    assert!(Arc::ptr_eq(a, b));
    assert_eq!(a.path(), f.original.join("z-link"));
    assert_eq!(io_cause(a).raw_os_error(), Some(5));
    assert_eq!(counted.count(Call::Stat, &f.original.join("z-link")), 1);
    assert_eq!(counted.count(Call::Link, &f.original.join("z-link")), 1);
    assert_eq!(counted.count(Call::Stat, &targets[1].join("z-link")), 0);
}

#[cfg(unix)]
#[test]
fn unsupported_facts_reach_each_late_visitor_before_lane_local_validation() {
    let f = Fixture::new();
    let targets = targets(&f, 3);
    let _socket = std::os::unix::net::UnixListener::bind(f.original.join("special")).unwrap();
    let counted = Counted::default();
    let roots = SharedRoots::bind_using(&f.original, targets.clone(), &counted).unwrap();
    let visitors = [
        Policy {
            mode: Mode::PreserveSpecials,
            ..Policy::default()
        },
        Policy::default(),
        Policy {
            mode: Mode::RefuseLate,
            ..Policy::default()
        },
    ];
    let mut results = roots
        .inspect_using(Path::new(""), &requests(&visitors), Execution::Sequential, &counted)
        .unwrap()
        .into_iter();
    let preserved = results.next().unwrap().outcome.unwrap();
    assert!(preserved.diff().is_empty());
    assert_eq!(preserved.coverage().retained, 1);
    assert_eq!(preserved.coverage().planned, 0);
    assert!(matches!(results.next().unwrap().outcome.unwrap_err().as_ref(), Error::UnsupportedEntry { path } if *path == f.original.join("special")));
    assert!(matches!(results.next().unwrap().outcome.unwrap_err().as_ref(), Error::Refused { path, .. } if *path == targets[2].join("special")));
    assert_eq!(counted.count(Call::Stat, &f.original.join("special")), 1);
    counted.assert_released();
}

#[test]
fn invalid_action_refuses_only_its_lane_without_a_partial_reduced_result() {
    let f = Fixture::new();
    let targets = targets(&f, 2);
    file(&f.original.join("equal"), b"equal");
    file(&f.original.join("new"), b"must not be partially published");
    for target in &targets {
        file(&target.join("equal"), b"equal");
    }
    let visitors = [
        Policy {
            mode: Mode::ForceEqual,
            ..Policy::default()
        },
        Policy::default(),
    ];
    let roots = SharedRoots::new(&f.original, &targets).unwrap();
    let mut results = roots
        .inspect_visitor(Path::new(""), &requests(&visitors), Execution::Sequential)
        .unwrap()
        .into_iter();
    let error = results.next().unwrap().outcome.unwrap_err();
    assert_eq!(error.path(), targets[0].join("equal"));
    assert!(matches!(error.as_ref(), Error::Refused { .. }));
    assert_eq!(results.next().unwrap().outcome.unwrap().diff().files.added, [OsString::from("new")]);
}

#[test]
fn invalid_input_is_atomic_and_empty_or_subset_requests_do_not_observe_other_lanes() {
    let f = Fixture::new();
    let targets = targets(&f, 2);
    let counted = Counted::default();
    let roots = SharedRoots::bind_using(&f.original, targets.clone(), &counted).unwrap();
    let visitor = NoVisitor;
    let before = counted.calls.lock().unwrap().len();
    for requests in [
        vec![TargetVisitor { target: 2, visitor: &visitor }],
        vec![TargetVisitor { target: 1, visitor: &visitor }, TargetVisitor { target: 0, visitor: &visitor }],
        vec![TargetVisitor { target: 0, visitor: &visitor }, TargetVisitor { target: 0, visitor: &visitor }],
    ] {
        assert!(roots.inspect_using(Path::new(""), &requests, Execution::Sequential, &counted).is_err());
    }
    assert!(
        roots
            .inspect_using(
                Path::new("../escape"),
                &[TargetVisitor { target: 0, visitor: &visitor }],
                Execution::Sequential,
                &counted
            )
            .is_err()
    );
    assert!(
        roots
            .inspect_using::<NoVisitor>(Path::new(""), &[], Execution::Sequential, &counted)
            .unwrap()
            .is_empty()
    );
    assert_eq!(counted.calls.lock().unwrap().len(), before);
    let results = roots
        .inspect_using(
            Path::new(""),
            &[TargetVisitor { target: 1, visitor: &visitor }],
            Execution::Sequential,
            &counted,
        )
        .unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].target, 1);
    assert_eq!(counted.count(Call::List, &targets[0]), 0);
    assert_eq!(counted.count(Call::List, &targets[1]), 1);
}

#[test]
fn binding_errors_are_source_first_target_tagged_and_never_list_interiors() {
    let f = Fixture::new();
    let targets = targets(&f, 2);
    let missing = f.root.join("missing");
    let counted = Counted::default();
    assert!(matches!(
        SharedRoots::bind_using(&missing, targets.clone(), &counted),
        Err(SharedBindError::Source(_))
    ));
    assert_eq!(counted.calls.lock().unwrap().as_slice(), [(Call::Root, missing)]);
    let counted = Counted::default();
    let leaf = f.root.join("leaf");
    file(&leaf, b"not a directory");
    assert!(
        matches!(SharedRoots::bind_using(&f.original, vec![targets[0].clone(), leaf.clone()], &counted), Err(SharedBindError::Target { target: 1, error: Error::NotDirectory { path } }) if path == leaf)
    );
    assert_eq!(counted.count(Call::Root, &f.original), 1);
    assert_eq!(counted.count(Call::Root, &targets[0]), 1);
    assert!(!counted.calls.lock().unwrap().iter().any(|(call, _)| *call == Call::List));
    assert!(matches!(SharedRoots::new(&f.original, Vec::<PathBuf>::new()), Err(SharedBindError::NoTargets)));
    #[cfg(unix)]
    {
        let link = f.root.join("root-link");
        std::os::unix::fs::symlink(&f.original, &link).unwrap();
        assert!(matches!(
            SharedRoots::new(&link, &targets),
            Err(SharedBindError::Source(Error::NotDirectory { .. }))
        ));
    }
}

#[test]
fn equal_read_only_roots_remain_distinct_bindings_and_disabled_visitors_never_run() {
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
    file(&f.original.join("source"), b"source");
    let roots = SharedRoots::new(&f.original, [&f.target, &f.target, &f.original]).unwrap();
    let visitors = [Disabled, Disabled, Disabled];
    let results = roots.inspect_visitor(Path::new(""), &requests(&visitors), Execution::Sequential).unwrap();
    assert_eq!(results.iter().map(|result| result.target).collect::<Vec<_>>(), [0, 1, 2]);
    assert_eq!(results[0].outcome.as_ref().unwrap(), results[1].outcome.as_ref().unwrap());
    assert!(results[2].outcome.as_ref().unwrap().diff().is_empty());
    assert!(!results[2].outcome.as_ref().unwrap().coverage().scoped);
}

#[test]
fn caller_selected_full_plans_share_source_facts_until_the_caller_lowers_them() {
    for &execution in executions() {
        let f = Fixture::new();
        let targets = targets(&f, 2);
        file(&f.original.join("source"), b"source");
        let counted = Counted::default();
        let roots = SharedRoots::bind_using(&f.original, targets, &counted).unwrap();
        let visitors = [NoVisitor, NoVisitor];
        let plans = roots
            .inspect_lowered_using(InspectionAt::RootGated(Path::new("")), &requests(&visitors), execution, &counted, |_, plan| {
                plan
            })
            .unwrap();
        assert_eq!(counted.count(Call::List, &f.original), 1);
        assert_eq!(counted.count(Call::Stat, &f.original.join("source")), 1);
        let source = |index: usize| plans[index].outcome.as_ref().unwrap().operations().entries()[0].source().unwrap();
        assert!(std::ptr::eq(source(0), source(1)));
        assert!(counted.facts.lock().unwrap().iter().all(|facts| facts.upgrade().is_some()));
        for plan in plans {
            let comparison = plan.outcome.unwrap().into_comparison();
            assert_eq!(comparison.diff().files.added, [OsString::from("source")]);
        }
        counted.assert_released();
    }
}

#[test]
fn already_admitted_root_refresh_skips_only_structural_hooks_without_rebinding_or_reinspection() {
    struct DirectoryOnly<'a>(&'a Mutex<Vec<&'static str>>);
    impl Visitor for DirectoryOnly<'_> {
        fn early(&self, names: &mut Names) -> Result<(), CheckError> {
            assert!(!names.is_root(), "accepted structural-root decision replayed");
            assert_eq!(names.dir(), Path::new(""));
            self.0.lock().unwrap().push("early");
            Ok(())
        }
        fn late(&self, operations: &mut Operations) -> Result<(), CheckError> {
            assert!(!operations.is_root(), "accepted structural-root decision replayed");
            assert_eq!(operations.entries().len(), 1);
            self.0.lock().unwrap().push("late");
            Ok(())
        }
    }
    let f = Fixture::new();
    let targets = targets(&f, 2);
    file(&f.original.join("source"), b"source");
    let counted = Counted::default();
    let roots = SharedRoots::bind_using(&f.original, targets.clone(), &counted).unwrap();
    let calls = Mutex::new(Vec::new());
    let visitors = [DirectoryOnly(&calls), DirectoryOnly(&calls)];
    let results = roots
        .inspect_lowered_using(
            InspectionAt::AlreadyAdmitted(Path::new("")),
            &requests(&visitors),
            Execution::Sequential,
            &counted,
            |_, plan| {
                assert!(!plan.operations().is_root());
                plan.into_comparison()
            },
        )
        .unwrap();
    assert!(results.into_iter().all(|result| result.outcome.is_ok()));
    assert_eq!(*calls.lock().unwrap(), ["early", "early", "late", "late"]);
    assert_eq!(counted.count(Call::Root, &f.original), 1);
    assert_eq!(counted.count(Call::List, &f.original), 1);
    assert_eq!(counted.count(Call::Stat, &f.original.join("source")), 1);
    for target in targets {
        assert_eq!(counted.count(Call::Root, &target), 1);
        assert_eq!(counted.count(Call::List, &target), 1);
    }
    counted.assert_released();
}

#[cfg(feature = "parallel")]
#[test]
fn wide_group_metadata_uses_and_joins_the_callers_pool() {
    struct Pooled<'a> {
        counted: &'a Counted,
        workers: Mutex<std::collections::BTreeSet<String>>,
        arrivals: Mutex<usize>,
        changed: std::sync::Condvar,
    }
    impl Acquire for Pooled<'_> {
        fn list(&self, path: &Path) -> Result<Vec<observe::Entry>, Error> {
            self.counted.list(path)
        }
        fn root_metadata(&self, path: &Path) -> Result<Metadata, Error> {
            self.counted.root_metadata(path)
        }
        fn link(&self, path: &Path) -> Result<PathBuf, Error> {
            self.counted.link(path)
        }
        fn metadata(&self, entry: observe::Entry) -> Result<Metadata, Error> {
            let name = std::thread::current().name().unwrap_or("unnamed").to_string();
            assert!(name.starts_with("shared-inspect-"));
            self.workers.lock().unwrap().insert(name);
            let mut arrivals = self.arrivals.lock().unwrap();
            *arrivals += 1;
            self.changed.notify_all();
            let (arrivals, _) = self
                .changed
                .wait_timeout_while(arrivals, std::time::Duration::from_secs(5), |arrivals| *arrivals < 2)
                .unwrap();
            assert!(*arrivals >= 2, "no second caller-pool worker reached metadata");
            drop(arrivals);
            self.counted.metadata(entry)
        }
        fn source_acquired(&self, facts: &Arc<Facts>) {
            self.counted.source_acquired(facts)
        }
    }
    let f = Fixture::new();
    let targets = targets(&f, 2);
    for index in 0..96 {
        let name = format!("file-{index:03}");
        file(&f.original.join(&name), b"same");
        for target in &targets {
            file(&target.join(&name), b"same");
        }
    }
    let counted = Counted::default();
    let pooled = Pooled {
        counted: &counted,
        workers: Mutex::default(),
        arrivals: Mutex::default(),
        changed: std::sync::Condvar::new(),
    };
    let roots = SharedRoots::bind_using(&f.original, targets, &pooled).unwrap();
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(4)
        .thread_name(|index| format!("shared-inspect-{index}"))
        .build()
        .unwrap();
    let visitors = [NoVisitor, NoVisitor];
    let results = pool
        .install(|| roots.inspect_using(Path::new(""), &requests(&visitors), Execution::Parallel, &pooled))
        .unwrap();
    assert!(results.into_iter().all(|result| result.outcome.unwrap().diff().is_empty()));
    assert!(pooled.workers.lock().unwrap().len() >= 2);
    assert_eq!(counted.under(Call::Stat, &f.original), 96);
    counted.assert_released();
    let calls = counted.calls.lock().unwrap().len();
    drop(roots);
    drop(pool);
    assert_eq!(counted.calls.lock().unwrap().len(), calls);
}
