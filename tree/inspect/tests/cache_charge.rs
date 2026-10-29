//! Requested-layout regression, not an RSS or allocator-rounded-size assertion.
#![cfg(target_os = "linux")]

#[path = "../../../tests/support/fixture.rs"]
mod support;

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::path::Path;

use diff_dirs_tree_inspect::{Execution, FullVisitor, NoExecutionDemand, NoVisitor, PairedRoots, SourceCacheLimits, SourceSession, StatKey};

struct Counted;
thread_local! {
    // Only this test thread and the explicit cache-clear interval are counted.
    static REQUESTS: Cell<Option<(usize, usize)>> = const { Cell::new(None) };
}
fn record(allocated: usize, released: usize) {
    let _ = REQUESTS.try_with(|requests| {
        if let Some((a, r)) = requests.get() {
            requests.set(Some((a + allocated, r + released)));
        }
    });
}
unsafe impl GlobalAlloc for Counted {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = unsafe { System.alloc(layout) };
        if !ptr.is_null() {
            record(layout.size(), 0);
        }
        ptr
    }
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let ptr = unsafe { System.alloc_zeroed(layout) };
        if !ptr.is_null() {
            record(layout.size(), 0);
        }
        ptr
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        record(0, layout.size());
        unsafe { System.dealloc(ptr, layout) };
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        let ptr = unsafe { System.realloc(ptr, layout, size) };
        if !ptr.is_null() {
            record(size, layout.size());
        }
        ptr
    }
}
#[global_allocator]
static ALLOCATOR: Counted = Counted;

#[test]
fn idle_charge_covers_actual_requested_release_across_small_index_capacities() {
    let fixture = support::Fixture::new();
    for i in 0..32 {
        let name = format!("batch-{i:02}");
        fixture.both_dir(&name);
        fixture.both_file(Path::new(&name).join("leaf"), b"");
    }
    let roots = PairedRoots::new(&fixture.original, &fixture.target).unwrap();
    // Include both sides of the current small-table and load-factor transitions.
    for batches in [1, 2, 3, 4, 7, 8, 14, 15, 16, 28, 29, 32] {
        // First fill, then a separate eviction/churn case on the same small tree.
        for visits in [batches, 64] {
            let session = SourceSession::new(&fixture.original);
            session.set_cache_limits(SourceCacheLimits {
                batches,
                ..SourceCacheLimits::default()
            });
            for i in 0..visits {
                drop(
                    roots
                        .inspect_policy(
                            &session,
                            Path::new(&format!("batch-{:02}", i % 32)),
                            &(),
                            &FullVisitor(NoVisitor),
                            &StatKey,
                            &NoExecutionDemand,
                            Execution::Sequential,
                        )
                        .unwrap(),
                );
            }
            let diagnostics = session.diagnostics();
            assert_eq!(diagnostics.retained_batches, batches);
            assert_eq!(diagnostics.active_batches, 0);
            REQUESTS.with(|requests| requests.set(Some((0, 0))));
            session.set_cache_limits(SourceCacheLimits {
                batches: 0,
                charged_capacity: 0,
            });
            let (allocated, released) = REQUESTS.with(|requests| requests.take().unwrap());
            let net_release = released.checked_sub(allocated).unwrap();
            assert!(
                diagnostics.retained_charged_capacity >= net_release,
                "{batches} batches after {visits} visits: charge {} < actual requested release {net_release}",
                diagnostics.retained_charged_capacity,
            );
            assert_eq!(session.diagnostics().retained_charged_capacity, 0);
        }
    }
}
