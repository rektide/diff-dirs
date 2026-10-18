---
type: PerformanceReport
title: Optional caller-pool Rayon execution — parity and measured tradeoffs
description: Same-build sequential versus shared 1/2/4/8-thread Rayon metadata batches, on synthetic fixtures and strictly read-only supplied live checkouts.
resource: /design/performance/parallel0.sol61x.md
status: measured
generated: { by: "model:openai/gpt-6.1-sol#xhigh", at: 2026-10-18 }
sources:
  - resource: opencode://ses_eb3948bedffe0T37Xc2wI2S5Jg
    title: Implemented current-directory joined Rayon execution, verified deterministic parity and caller-pool use, and measured the shared-pool matrix
  - resource: /README.md#optional-shared-pool-parallel-execution
    title: Public execution policy and caller integration contract
  - resource: /src/walk.rs
    title: Sorted metadata jobs, indexed result collection, threshold, and test-only metadata-boundary pool observation
  - resource: /tests/parallel.rs
    title: Record/summary/error parity and no-prefetch apply/prune tests
  - resource: /perf/README.md#sequential-versus-caller-owned-rayon-pools
    title: Exact matrix command and execution/timing boundaries
  - resource: /perf/results/parallel.csv
    title: All 150 raw observations, including counts, for same-build sequential and 1/2/4/8-worker comparisons
  - resource: /design/performance/performance0.sol61x.md
    title: Environment, fixture census, actual live tree census, and original sequential evidence
  - resource: /design/fit/io0.glm53m.md
    title: IO research motivating explicit blocking parallelism rather than mandatory async runtime
  - resource: /design/fit/rayon0.glm53m.md
    title: Verified ambient-pool and indexed-error-selection APIs; caller-side independent root pairs and bounded-head alternatives
---

# Optional caller-pool execution

## Model and verification

`parallel` adds Rayon optionally; default builds still have no dependencies.
`walk()` remains sequential. `walk_with_options` selects `Execution::Parallel`
without changing records or summary semantics. The application must consume
the walk inside its existing `ThreadPool::install`. Core code never builds a
pool or calls `build_global`; outside an installed pool Rayon uses its normal
global fallback. All current-directory jobs join before yielding, so callers
can still mutate immediate classified children or prune pending child descent.

Both raw listings are captured, combined, and sorted before metadata lookup.
Names sort by native encoded bytes; equal names use original then target.
Listing errors (original then target) precede metadata errors. Metadata work
collects indexed, ordered `Result` values, then selects the first deterministic
error, never whichever worker fails first. Per-directory batching begins at 64
combined entries with minimum split length 16; small directories and pruned
root fixtures avoid dispatch. This heuristic amortizes scheduling on the
50-files-per-directory synthetic shape; no threshold sweep or universal
optimality is claimed.

All 36 all-features tests pass. Actual metadata-boundary observation verifies
every inspected entry runs on caller-named four-thread pool workers. Parity
tests compare complete records/summaries, deliberate competing error paths,
literal/non-UTF-8 symlinks and names via the shared fixture tests, and applying
immediate records followed by pruning a subsequently removed pending child.
No worker is detached and there is no subtree prefetch.

## Matrix method

Same machine/filesystem and fixtures as the [sequential report](/design/performance/performance0.sol61x.md).
One executable compiled by Rust 1.88.0 with `--release --features parallel`,
with execution selected at runtime. One pool is built per process **outside
the timing interval**, then shared by all warmups, measurements, and fixture
checks in that process. Each row has one untimed warmup plus n=3 observations.
Sequential and parallel-1 are separately selected paths in that same build.
The exact [matrix command](/perf/README.md#sequential-versus-caller-owned-rayon-pools)
uses `--fixture` only on owned scratch; both user live pairs use read-only
`repeat`, never mutation. Pool counts are 1/2/4/8, with no cache dropping,
pinning, host isolation, or claim of statistical confidence from three samples.
Executable SHA-256:
`b0ecd6262a918629dc886922a66dc98afb044db2e926a9c835b7b03c25156bc4`.

## Results

Median milliseconds; full ranges, variance, raw timings, and counts can be
recomputed using `python3 perf/report.py perf/results/parallel.csv`.

| Fixture / scenario | Sequential | Parallel 1 | Parallel 2 | Parallel 4 | Parallel 8 |
| --- | ---: | ---: | ---: | ---: | ---: |
| 29,916 clean | 78.713 | 76.512 | 60.593 | 53.256 | 66.350 |
| 29,916 sparse | 78.430 | 77.658 | 64.268 | 52.827 | 53.503 |
| 29,916 one-sided | 78.924 | 77.544 | 61.053 | 54.582 | 51.685 |
| 29,916 prune | 0.013151 | 0.010490 | 0.010730 | 0.017080 | 0.010901 |
| 191,891 clean | 490.234 | 489.975 | 454.846 | 335.961 | 359.282 |
| 191,891 sparse | 488.229 | 493.850 | 393.901 | 409.577 | 409.338 |
| 191,891 one-sided | 485.129 | 496.383 | 410.579 | 377.904 | 406.521 |
| 191,891 prune | 0.010700 | 0.011190 | 0.017821 | 0.012340 | 0.010510 |
| Live whole checkouts | 779.973 | 784.996 | 745.781 | 711.955 | 826.773 |
| Live node_modules | 689.849 | 704.071 | 666.595 | 668.887 | 636.048 |

All counts across policies matched exactly. Small/large fixture coverage is
774/4,954 clean or sparse directory pairs; 766/4,946 one-sided pairs; one
directory with one explicit prune in the prune case. Sparse counts remain
one changed/one added/one removed file, one-sided counts one added/one removed
directory. Whole live checkouts remain 49 records, 25,203 examined directories,
files changed 6 / added 11 / removed 2 and dirs added 69 / removed 0. Live
`node_modules` remains a clean 22,828-directory-pair comparison over the actual
191,805-entry / 8,291,060,621-byte tree per side. No live traversal failures,
hidden ignores, or mutations were introduced by parallel measurement.

## Policy decision and limits

Four workers helped the wide synthetic fixture substantially (~1.46× on the
large clean median). Live gains were modest: about 9% whole-tree at four
workers and 3% for `node_modules`; eight workers helped `node_modules` by ~8%
but regressed whole-tree by ~6%. Per-directory-only parallelism leaves many
small directories sequential and intentionally sacrifices speculative subtree
throughput to retain the consumer mutation boundary. Large-clean parallel-2
samples also varied 390.856–688.056 ms: noise matters.

Therefore **sequential stays the default**; callers with an existing pool can
opt into parallel and choose pool sizing based on their workload. Four is a
reasonable experimental starting point on this eight-core machine, not an
internal default or a global-pool recommendation. Different warm-cache state
and run order mean the lower current sequential live median must not be
attributed causally to this change by comparison with the earlier report.
Cold/network filesystem behavior and threshold tuning remain unmeasured.
