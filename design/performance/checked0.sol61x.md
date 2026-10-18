---
type: PerformanceReport
title: Static checker and streaming sync controls
description: Same-compiler baseline, default no-check, enabled no-op and concrete name-check controls on owned nonempty fixtures, with an explicitly preserved negative first checkpoint.
resource: /design/performance/checked0.sol61x.md
status: measured-initial-checkpoint
generated: { by: "model:openai/gpt-6.1-sol#xhigh", at: 2026-10-18 }
sources:
  - resource: "opencode://ses_eb24f98a3ffewdVqa9lao6tXQu"
    title: Implemented checked streaming execution, measured identical before/after harnesses, found and preserved root-relative path extraction overhead before tuning
  - resource: /design/sync/checked0.sol61x.md
    title: Checked execution, metadata reuse, audit, safety and JW handoff
  - resource: /perf/results/checked/initial.csv
    title: All 480 raw initial observations, including 80 warmup rows
  - resource: /perf/results/checked/initial-identity.json
    title: Exact compiler, baseline/candidate coordinates and binary SHA-256
  - resource: /perf/results/checked/initial-bench.rs
    title: Identical owned-fixture setup, scenario edits, timing and verification code compiled against both libraries
---

# Static checker and streaming sync controls

This measures **diff-dirs-sync, not JW**. No live trees were mutated. It does
not measure the benefit of removing JW's separate payload preflights, mount
syscalls, probes or child processes; integration must establish those separately.

## Controlled method

Baseline: `fd277144c51700ca70d068fc88f3ff72a0d64293`. Initial candidate:
`8658b6da`. Both use configured `rustc 1.101.0-nightly (822182db4 2026-10-13)`,
LLVM 23.1.3, identical release profile and dependency versions. Source snapshots,
binary identities, exact commands and compiler are [retained](../../perf/results/checked/README.md).

The scratch harness builds identical benchmark source against the archived
baseline or current candidate. Candidate-only API branches are compile-gated;
all scenario setup/timing/verification is shared. One fresh marked fixture has
100 package directories, 10,000 nonempty 4 KiB files (40,960,000 logical bytes),
and 100 literal symlinks. It is on the checkout's btrfs filesystem. Each
configuration owns a fresh temporary target within that fixture. Both source and
target paths are long absolute scratch paths, representative of this checkout,
not a short `/x/y` microbenchmark. No CPU affinity, cache drops or isolated-host
guarantee. Filesystem/system variance is visible in the ranges.

Configurations rotate across six rounds: one entire warmup round and five
retained rounds. Each round has baseline/default, candidate/default, enabled
no-op closure and concrete admin-name checker × serial/P4 × ordinary/required
reflink. P4 uses one caller-owned pool with J4/K256. No builds or tests overlapped
timing. Setup, target edits, complete post-sync comparison and owned cleanup are
outside each scenario's measured operation. Printed results happen after timing.

- **fresh:** absent target initialization, all 10k files/100 links/101 directories.
- **clean:** repeat with zero completed operations, no staging/probe/chmod.
- **sparse:** one absent and one wrong-size target file, one stale target leaf;
  two copied files plus one removal.
- **subtree:** one absent package (100 files/link), one stale directory with
  100 leaves; recursive operations, not opaque-only comparison timing.
- **full:** every target regular file is overwritten with wrong-size content
  outside timing; all 10k leaves must be replaced, paired dirs/links remain.

## Preserved initial checkpoint — a useful negative

Medians in milliseconds; n=5 each. Columns are predecessor, candidate default
`NoCheck`, enabled no-op, concrete admin-name check.

| Execution / policy | Scenario | Before | Default | No-op | Admin |
|---|---|---:|---:|---:|---:|
| Serial copy | fresh | 592.727 | 568.888 | 624.802 | 585.964 |
| Serial copy | clean | 27.720 | 27.091 | 33.849 | 33.884 |
| Serial copy | sparse | 27.438 | 27.786 | 32.579 | 33.482 |
| Serial copy | subtree | 38.943 | 35.533 | 43.188 | 43.967 |
| Serial copy | full | 996.248 | 977.914 | 1012.817 | 970.412 |
| Serial reflink | fresh | 549.149 | 566.734 | 581.840 | 567.689 |
| Serial reflink | clean | 25.570 | 27.382 | 33.269 | 32.642 |
| Serial reflink | sparse | 26.479 | 28.446 | 32.357 | 32.704 |
| Serial reflink | subtree | 35.194 | 35.892 | 41.818 | 41.336 |
| Serial reflink | full | 949.615 | 949.400 | 933.436 | 1035.516 |
| P4 copy | fresh | 363.258 | 369.209 | 372.485 | 325.032 |
| P4 copy | clean | 7.974 | 8.765 | 9.579 | 9.936 |
| P4 copy | sparse | 8.389 | 8.647 | 9.744 | 10.511 |
| P4 copy | subtree | 15.292 | 13.454 | 15.809 | 15.440 |
| P4 copy | full | 631.636 | 737.883 | 806.098 | 674.666 |
| P4 reflink | fresh | 307.625 | 333.137 | 348.724 | 320.746 |
| P4 reflink | clean | 8.592 | 8.598 | 9.725 | 10.244 |
| P4 reflink | sparse | 8.427 | 9.266 | 9.809 | 10.661 |
| P4 reflink | subtree | 13.393 | 14.121 | 14.551 | 15.898 |
| P4 reflink | full | 729.219 | 649.255 | 747.152 | 676.796 |

Default is broadly near the predecessor, not a demonstrated universal speedup.
Enabled no-op adds about 6 ms to a ~27 ms serial clean comparison: unacceptable
avoidable overhead for a cheap name checker. Per-entry eager `strip_prefix(root)`
constructs the root-relative observation before calling even an enabled no-op;
the possible assertion failure makes that work observable. This checkpoint and
its raw results are deliberately preserved before a lazy-path observation change.
No extra metadata syscall was required by the check interface itself.

Full/fresh wall results have substantial variance, especially parallel mutation;
do not attribute their individual percent differences to checker dispatch alone.
For example P4/default copy full ranges 602–1299 ms, while baseline ranges
585–882 ms. The original observed data, not just medians, remain available.
