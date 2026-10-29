use super::*;
use std::collections::HashSet;
use std::num::NonZeroUsize;
use std::sync::Condvar;
use std::time::Duration;

use crate::pair::Roots;
use crate::{Execution, ParallelLimits};

fn limits(j: usize, k: usize) -> ParallelLimits {
    ParallelLimits::new(NonZeroUsize::new(j).unwrap(), k)
}

fn pair(dir: &Path, children: &[&str], changed: bool) -> ComparedPair {
    let mut diff = DirDiff {
        dir: dir.to_path_buf(),
        ..DirDiff::default()
    };
    if changed {
        diff.files.added.push("new".into());
    }
    ComparedPair {
        plan: None,
        coverage: DirectoryCoverage {
            different: u64::from(changed),
            ..DirectoryCoverage::default()
        },
        comparator: crate::Comparator::identity(&crate::StatKey).clone(),
        diff,
        children: children
            .iter()
            .map(|name| Child {
                name: (*name).into(),
                context: (),
            })
            .collect(),
    }
}

fn assert_bounds(state: &Frontier) {
    assert!(state.outstanding <= state.limits.max_directory_jobs.get());
    assert!(state.speculative <= state.limits.max_speculative_directories);
    assert_eq!(state.nodes.len, state.speculative + usize::from(state.demand.is_some()));
    assert_eq!(state.nodes.len + state.nodes.free.len(), state.nodes.slots.len());
    let mut references = HashSet::new();
    for frame in &state.frames {
        assert!(frame.children.admitted.len() <= frame.children.names.len());
        if !frame.open {
            assert!(frame.children.admitted.is_empty());
        }
        for id in &frame.children.admitted {
            assert!(references.insert(*id));
        }
    }
    for slot in &state.nodes.slots {
        if let Some(Node {
            state: NodeState::Ready(Ok(pair)),
            ..
        }) = &slot.node
        {
            assert!(pair.children.admitted.len() <= pair.children.names.len());
            if !pair.diff.is_empty() {
                assert!(pair.children.admitted.is_empty());
            }
            for id in &pair.children.admitted {
                assert!(references.insert(*id));
            }
        }
    }
    assert_eq!(references.len(), state.speculative);
    for id in references {
        assert!(state.nodes.get(id).speculative);
    }
}

fn snapshot(state: &Frontier) -> Summary {
    let mut summary = state.summary.clone();
    let mut descendant_changed = false;
    for frame in state.frames.iter().rev() {
        descendant_changed |= frame.subtree_changed;
        if descendant_changed {
            summary.changed_dirs.push(frame.dir.clone());
        }
    }
    summary
}

fn model(dir: &Path) -> Result<ComparedPair, Error> {
    match dir.to_str().unwrap() {
        "" => Ok(pair(dir, &["a", "b", "c"], false)),
        "a" => Ok(pair(dir, &["x", "y"], false)),
        "a/x" => Ok(pair(dir, &["forbidden"], true)),
        "a/y" | "b/inner" => Ok(pair(dir, &[], false)),
        "b" => Ok(pair(dir, &["inner"], true)),
        "c" => Err(Error::UnsupportedEntry { path: dir.join("socket") }),
        _ => panic!("inspection crossed a divergence seal: {dir:?}"),
    }
}

fn next_permutation(items: &mut [&str]) -> bool {
    let Some(i) = (0..items.len() - 1).rev().find(|&i| items[i] < items[i + 1]) else {
        return false;
    };
    let j = (i + 1..items.len()).rev().find(|&j| items[j] > items[i]).unwrap();
    items.swap(i, j);
    items[i + 1..].reverse();
    true
}

#[test]
fn all_completion_priorities_preserve_records_prefixes_errors_and_seals() {
    // Every total priority permutation induces a legal completion order for
    // currently admitted jobs. Dependencies and stop/drain boundaries remain
    // real; this is not a second implementation of the reducer.
    let mut priorities = ["a", "a/x", "a/y", "b", "b/inner", "c"];
    let mut permutations = 0;
    loop {
        let mut state = Frontier::new(limits(3, 4));
        let mut delivered = Vec::new();
        loop {
            state.begin_epoch();
            let mut pending = state.pump();
            while !pending.is_empty() {
                let index = pending
                    .iter()
                    .enumerate()
                    .min_by_key(|(_, job)| priorities.iter().position(|p| Path::new(p) == job.dir).unwrap_or(0))
                    .unwrap()
                    .0;
                let job = pending.swap_remove(index);
                state.start(job.id);
                pending.extend(state.complete(job.id, model(&job.dir)));
                assert_bounds(&state);
            }
            assert_ne!(state.stop, Stop::Running);
            let outcome = state.output.take().unwrap();
            match outcome {
                Ok((diff, _)) => {
                    delivered.push(diff.dir.clone());
                    let summary = snapshot(&state);
                    assert_eq!(summary.state, ScanState::Incomplete);
                    if delivered.len() == 1 {
                        assert_eq!(diff.dir, Path::new("a/x"));
                        assert_eq!(summary.examined_directories, 3);
                        assert_eq!(summary.counts.files.added, 1);
                        assert_eq!(summary.changed_dirs, ["a/x", "a", ""].map(PathBuf::from));
                        assert!(summary.pruned.is_empty());
                        assert!(state.prune_here());
                    } else {
                        assert_eq!(diff.dir, Path::new("b"));
                        assert_eq!(summary.examined_directories, 5);
                        assert_eq!(summary.counts.files.added, 2);
                        assert_eq!(summary.changed_dirs, ["a/x", "a", "b", ""].map(PathBuf::from));
                        assert_eq!(summary.pruned, [PathBuf::from("a/x")]);
                    }
                }
                Err(error) => {
                    assert_eq!(error.path(), Path::new("c/socket"));
                    assert_eq!(delivered, ["a/x", "b"].map(PathBuf::from));
                    let summary = state.into_summary();
                    assert_eq!(summary.state, ScanState::Failed);
                    assert_eq!(summary.examined_directories, 6);
                    assert_eq!(summary.counts.files.added, 2);
                    assert_eq!(summary.topmost_divergent, ["a/x", "b"].map(PathBuf::from));
                    assert_eq!(summary.changed_dirs, ["a/x", "a", "b", ""].map(PathBuf::from));
                    break;
                }
            }
        }
        permutations += 1;
        if !next_permutation(&mut priorities) {
            break;
        }
    }
    assert_eq!(permutations, 720);
}

#[test]
fn ready_later_sibling_saturates_k_but_cannot_steal_demand_slot() {
    let mut state = Frontier::new(limits(2, 1));
    state.begin_epoch();
    let root = state.pump().pop().unwrap();
    state.start(root.id);
    let mut children = state.complete(root.id, Ok(pair(Path::new(""), &["a", "z"], false)));
    let z = children.pop().unwrap();
    let a = children.pop().unwrap();
    assert_eq!(z.dir, Path::new("z"));
    state.start(z.id);
    assert!(state.complete(z.id, Ok(pair(&z.dir, &[], false))).is_empty());
    assert_eq!(state.speculative, 1);
    state.start(a.id);
    let deep = state.complete(a.id, Ok(pair(&a.dir, &["deep"], false))).pop().unwrap();
    assert_eq!(deep.dir, Path::new("a/deep"));
    assert!(!state.nodes.get(deep.id).speculative);
    assert_eq!(state.speculative, 1);
    assert!(state.stats.demand_reserve_uses > 0);
    state.start(deep.id);
    assert!(state.complete(deep.id, Ok(pair(&deep.dir, &[], false))).is_empty());
    assert_bounds(&state);
    assert!(state.into_summary().is_clean());
}

#[test]
fn drained_sibling_names_count_as_private_memory_but_not_public_coverage() {
    let mut state = Frontier::new(limits(2, 1));
    state.begin_epoch();
    let root = state.pump().pop().unwrap();
    state.start(root.id);
    let mut jobs = state.complete(root.id, Ok(pair(Path::new(""), &["a", "z"], false)));
    let z = jobs.pop().unwrap();
    let a = jobs.pop().unwrap();
    state.start(z.id);
    state.start(a.id);
    assert!(state.complete(a.id, Ok(pair(&a.dir, &[], true))).is_empty());
    assert_eq!(state.stop, Stop::Yield);
    let names = (0..100).map(|i| format!("late-child-{i:03}")).collect::<Vec<_>>();
    assert!(
        state
            .complete(z.id, Ok(pair(&z.dir, &names.iter().map(String::as_str).collect::<Vec<_>>(), false)))
            .is_empty()
    );
    assert!(state.stats.peak_retained_name_bytes >= names.iter().map(String::len).sum::<usize>());
    assert_eq!(state.into_summary().examined_directories, 2);
}

#[test]
fn budget_stall_counter_requires_free_job_capacity_not_just_full_k() {
    let mut state = Frontier::new(limits(2, 1));
    state.begin_epoch();
    let root = state.pump().pop().unwrap();
    state.start(root.id);
    let mut jobs = state.complete(root.id, Ok(pair(Path::new(""), &["a", "b", "c"], false)));
    assert_eq!(state.stats.speculative_budget_stalls, 0); // J is fully reserved
    let b = jobs.pop().unwrap();
    state.start(b.id);
    assert!(state.complete(b.id, Ok(pair(&b.dir, &[], false))).is_empty());
    assert_eq!(state.outstanding, 1);
    assert_eq!(state.speculative, 1);
    assert_eq!(state.stats.speculative_budget_stalls, 1); // J free, eligible c, K full
}

#[test]
fn k_zero_and_reclaimed_arena_do_not_accumulate_finished_tree_nodes() {
    for k in [0, 1, 8] {
        let mut state = Frontier::new(limits(4, k));
        let names = (0..500).map(|i| format!("dir-{i:03}")).collect::<Vec<_>>();
        state.next_with(|dir| {
            if dir.as_os_str().is_empty() {
                Ok(pair(dir, &names.iter().map(String::as_str).collect::<Vec<_>>(), false))
            } else {
                Ok(pair(dir, &[], false))
            }
        });
        assert_eq!(state.nodes.len, 0);
        assert!(state.nodes.slots.len() <= k + 1);
        assert_eq!(state.summary.examined_directories, 501);
        assert!(state.into_summary().is_clean());
    }
}

#[derive(Default)]
struct Gate {
    arrivals: Mutex<usize>,
    changed: Condvar,
}

impl Gate {
    fn arrive(&self) {
        *self.arrivals.lock().unwrap() += 1;
        self.changed.notify_all();
    }
    fn wait_for(&self, count: usize) {
        let (arrivals, _) = self
            .changed
            .wait_timeout_while(self.arrivals.lock().unwrap(), Duration::from_secs(5), |n| *n < count)
            .unwrap();
        assert!(*arrivals >= count, "directory jobs did not concurrently reach the test gate");
    }
}

#[test]
fn real_small_directory_jobs_run_on_two_distinct_caller_workers() {
    let f = crate::test_support::Fixture::new();
    for dir in ["a", "b"] {
        f.both_dir(dir);
        f.both_file(Path::new(dir).join("file"), b"");
    }
    let roots = Roots::new(&f.original, &f.target).unwrap();
    let gate = Gate::default();
    let workers = Mutex::new(HashSet::new());
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(2)
        .thread_name(|i| format!("directory-pool-{i}"))
        .build()
        .unwrap();
    let mut state = Frontier::new(limits(2, 4));
    pool.install(|| {
        assert!(
            state
                .next_with(|dir| {
                    if !dir.as_os_str().is_empty() {
                        workers.lock().unwrap().insert(std::thread::current().name().unwrap().to_string());
                        gate.arrive();
                        gate.wait_for(2);
                    }
                    roots.inspect(dir.to_path_buf(), Execution::Sequential)
                })
                .is_none()
        )
    });
    assert_eq!(workers.lock().unwrap().len(), 2);
    assert_eq!(state.stats.peak_active, 2);
    assert_eq!(state.stats.worker_ids.len(), 2);
    assert_bounds(&state);
    assert!(state.into_summary().is_clean());
}

#[test]
fn huge_first_branch_recruits_workers_after_tiny_siblings_finish() {
    let f = crate::test_support::Fixture::new();
    for dir in ["b", "c"] {
        f.both_dir(dir);
    }
    for i in 0..32 {
        f.both_dir(format!("a/{i:02}"));
    }
    let roots = Roots::new(&f.original, &f.target).unwrap();
    let tiny = Gate::default();
    let large = Gate::default();
    let workers = Mutex::new(HashSet::new());
    let pool = rayon::ThreadPoolBuilder::new().num_threads(4).build().unwrap();
    let mut state = Frontier::new(limits(4, 12));
    pool.install(|| {
        assert!(
            state
                .next_with(|dir| {
                    if dir == Path::new("a") {
                        tiny.wait_for(2);
                    }
                    if dir == Path::new("a/00") || dir == Path::new("a/01") {
                        workers.lock().unwrap().insert(rayon::current_thread_index().unwrap());
                        large.arrive();
                        large.wait_for(2);
                    }
                    let result = roots.inspect(dir.to_path_buf(), Execution::Sequential);
                    if dir == Path::new("b") || dir == Path::new("c") {
                        tiny.arrive();
                    }
                    result
                })
                .is_none()
        )
    });
    assert_eq!(workers.lock().unwrap().len(), 2);
    assert!(state.stats.peak_active >= 2);
    assert!(state.into_summary().is_clean());
}

#[test]
fn speculative_divergence_is_sealed_and_prefix_omits_drained_siblings() {
    let f = crate::test_support::Fixture::new();
    for dir in ["a/forbidden", "b/forbidden"] {
        f.both_dir(dir);
    }
    crate::test_support::file(&f.original.join("a/new"), b"");
    crate::test_support::file(&f.original.join("b/new"), b"");
    let roots = Roots::new(&f.original, &f.target).unwrap();
    let b_finished = Gate::default();
    let seen = Mutex::new(Vec::new());
    let pool = rayon::ThreadPoolBuilder::new().num_threads(2).build().unwrap();
    let mut state = Frontier::new(limits(2, 2));
    let inspect = |dir: &Path| {
        assert!(!dir.ends_with("forbidden"), "descended under an unacknowledged difference");
        seen.lock().unwrap().push(dir.to_path_buf());
        if dir == Path::new("a") {
            b_finished.wait_for(1);
        }
        let result = roots.inspect(dir.to_path_buf(), Execution::Sequential);
        if dir == Path::new("b") {
            b_finished.arrive();
        }
        result
    };
    let first = pool.install(|| state.next_with(inspect)).unwrap().unwrap();
    assert_eq!(first.dir, Path::new("a"));
    assert_eq!(state.active, 0);
    assert_eq!(state.outstanding, 0);
    assert!(state.stats.completed >= 3);
    assert_eq!(snapshot(&state).examined_directories, 2); // root + a, not b
    let count = seen.lock().unwrap().len();
    std::thread::sleep(Duration::from_millis(10));
    assert_eq!(seen.lock().unwrap().len(), count);
    assert!(state.prune_here());
    std::fs::remove_dir_all(f.target.join("a")).unwrap();
    std::fs::create_dir(f.target.join("a")).unwrap();
    let second = pool.install(|| state.next_with(inspect)).unwrap().unwrap();
    assert_eq!(second.dir, Path::new("b"));
    assert_eq!(seen.lock().unwrap().len(), count); // cached b, no reinspection
    assert!(state.prune_here());
    assert!(pool.install(|| state.next_with(inspect)).is_none());
    assert!(!state.into_summary().is_complete());
    assert_eq!(seen.lock().unwrap().len(), count);
}

#[test]
fn dropping_an_early_walk_does_not_start_or_leave_directory_jobs() {
    let f = crate::test_support::Fixture::new();
    for dir in ["a", "z"] {
        f.both_dir(dir);
    }
    crate::test_support::file(&f.original.join("a/new"), b"");
    let roots = Roots::new(&f.original, &f.target).unwrap();
    let reads = std::sync::atomic::AtomicUsize::new(0);
    let pool = rayon::ThreadPoolBuilder::new().num_threads(2).build().unwrap();
    let mut state = Frontier::new(limits(2, 4));
    pool.install(|| {
        assert_eq!(
            state
                .next_with(|dir| {
                    reads.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    roots.inspect(dir.to_path_buf(), Execution::Sequential)
                })
                .unwrap()
                .unwrap()
                .dir,
            Path::new("a")
        )
    });
    assert_eq!(state.active, 0);
    assert_eq!(state.outstanding, 0);
    let before = reads.load(std::sync::atomic::Ordering::Relaxed);
    drop(state);
    std::thread::sleep(Duration::from_millis(10));
    assert_eq!(reads.load(std::sync::atomic::Ordering::Relaxed), before);
}
