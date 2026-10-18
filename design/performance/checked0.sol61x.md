---
type: PerformanceReport
title: Static checker and streaming sync controls
description: Same-compiler baseline, default no-check, enabled no-op and concrete name-check controls on owned nonempty fixtures, with an explicitly preserved negative first checkpoint.
resource: /design/performance/checked0.sol61x.md
status: measured-initial-checkpoint
generated: { by: "model:openai/gpt-6.1-sol#xhigh", at: 2026-10-18 }
sources:
  - resource: "opencode://ses_eb24f98a3ffewdVqa9lao6tXQu"
    title: Implemented and measured static checked streaming, preserved the initial path-extraction penalty, then removed it with direct names and lazy relative paths and remeasured the controls
  - resource: /design/sync/checked0.sol61x.md
    title: Checked execution, metadata reuse, audit, safety and JW handoff
  - resource: /perf/results/checked/initial.csv
    title: All 480 raw initial observations, including 80 warmup rows
  - resource: /perf/results/checked/initial-identity.json
    title: Exact compiler, baseline/candidate coordinates and binary SHA-256
  - resource: /perf/results/checked/initial-bench.rs
    title: Identical owned-fixture setup, scenario edits, timing and verification code compiled against both libraries
  - resource: /perf/results/checked/clean.csv
    title: 210 interleaved clean observations comparing baseline, initial checkpoint and lazy-path correction on one shared owned target
  - resource: /perf/results/checked/live-census.txt
    title: Fresh read-only census of the actual prepared donor and export-qa node_modules roots
  - resource: /perf/results/checked/live.csv
    title: Real donor to owned reflink targets, first fill and repeated clean/sparse/subtree comparisons over 22,828 directory pairs
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

## Lazy-path correction — focused clean control

The follow-up `243be8e2` borrows the already-built full path and native name.
`Entry::relative_path()` now performs root-prefix extraction **only if called**;
ordinary JW-style name/type guards need neither extraction nor allocation. This
is smaller than propagating a new inspection-frame/path representation through
the recursive engine. Checks still use exactly the same metadata and link reads.

A separate interleaved read-only control compares all three actual executables
on **one shared owned, already-initialized target**. Every subprocess performs
one untimed clean warmup, then one timed clean mirror. Fifteen rounds rotate the
seven executable/check combinations in serial/P4; required-reflink is selected
but performs no clones on these clean scans. All 210 timed runs assert zero
completed operations. Medians and ranges, milliseconds:

| Executable / check | Serial median [range] | P4 median [range] |
|---|---:|---:|
| Before / none | 26.453 [24.899–27.371] | 7.229 [6.522–13.934] |
| Initial / NoCheck | 26.763 [25.252–27.829] | 7.124 [6.664–10.165] |
| Initial / enabled no-op | 32.004 [30.244–33.946] | 8.476 [8.012–11.338] |
| Initial / admin | 32.855 [31.375–37.089] | 8.885 [8.119–10.166] |
| Lazy / NoCheck | 26.924 [24.853–28.662] | 7.165 [6.434–8.223] |
| Lazy / enabled no-op | 26.759 [25.712–37.281] | 7.033 [6.545–8.543] |
| Lazy / admin | 27.287 [25.298–46.141] | 7.144 [6.656–7.967] |

This removes the initial **repeatable ~5–6 ms serial / ~1.4–1.8 ms P4 penalty**.
The corrected enabled no-op and concrete guard are near default/predecessor
throughput. A 1–3% difference in noisy medians is not proof of literally zero
CPU overhead or a new speedup. The interface has no extra observation syscalls;
actual operation-count attribution and the full mutation controls are retained
alongside this focused result. This does not measure the larger benefit of
removing external validation passes in JW.

## Corrected full mutation matrix

The same six-round experiment was repeated at `243be8e2` (480 rows, 400 retained).
Required-reflink medians in milliseconds, n=5; the full Copy/no-op matrix and
ranges remain in `optimized.csv`.

| Execution | Scenario | Before | Lazy default | Lazy admin |
|---|---|---:|---:|---:|
| Serial | fresh | 664.220 | 559.033 | 620.959 |
| Serial | clean | 27.573 | 28.238 | 26.346 |
| Serial | sparse | 27.236 | 30.153 | 27.883 |
| Serial | subtree | 36.602 | 43.356 | 39.591 |
| Serial | full | 1001.472 | 925.862 | 1129.702 |
| P4 | fresh | 303.960 | 328.618 | 314.987 |
| P4 | clean | 8.148 | 9.595 | 8.008 |
| P4 | sparse | 9.651 | 9.091 | 9.137 |
| P4 | subtree | 14.122 | 14.321 | 14.195 |
| P4 | full | 730.900 | 748.876 | 742.718 |

All clean runs perform zero completed operations; every successful mutation is
fully re-compared outside timing. This matrix **does not show a consistent
mutation wall-time win**. Negative cases remain: serial admin/full reflink is
1129.7 ms versus 1001.5 ms before; P4 default/full ordinary copy is 705.4 ms versus
641.1 ms; serial default/fresh copy is 648.8 ms versus 581.6 ms. Other medians
improve, often with broad overlapping ranges. Do not promote a universal speedup
from these filesystem-heavy samples. The defensible outcomes are full checked
coverage without an eager validation traversal, zero clean writes, and removal
of the initial repeatable hook-only CPU penalty. The actual user workload is
measured separately below rather than inferred from this 10k-file fixture.

## Actual opencode payload shape, read-only donor and owned write targets

Following the human's renewed emphasis on the real fleet workload, both the
prepared donor `~/a/a/opencode/node_modules` and existing feature workspace
`~/src/opencode-export-qa/node_modules` were censused **read-only**. Each currently
has **191,805 entries excluding root: 160,407 files, 22,827 directories, 8,571
symlinks, zero specials, 8,291,060,621 logical file bytes**. The resolved donor
path is `/home/rektide/archive/anomalyco/opencode/node_modules`; the primary
checkout route is an alias. Sync binds the canonical real path. No donor snapshot,
mutation, chmod, probe, source-file write or live-workspace refill was performed.
The feature workspace was only inspected, never synchronized.

All write targets were newly owned beneath
`.test-agent/checked/live-node-modules`. They are strict reflink copies of that
**actual donor**, not flattened/generated metadata lookalikes. Every mutation
was re-compared outside timing. Source stability is required and was sufficient
for all these validations; this is not a snapshot claim. Enumeration for choosing
two target-only edits does not follow symlinks, and no source path is edited.

The first fills each clone 160,407 files / 8.29 GB logical, create 22,828 dirs and
8,571 links, with zero ordinary-copy fallbacks. Initial single observations:

| Execution | Before | Lazy admin |
|---|---:|---:|
| Serial | 16.934 s | 15.459 s |
| P4/J4/K256 | 6.936 s | 9.151 s |

These single samples are **not a speedup proof**. In particular the negative P4
result warrants repetition, not dismissal; repeated controls are retained below.

The repeated real-shape matrix has one entire warmup round and five retained
rounds, rotating baseline/default/admin serial and P4 configurations on a shared
owned initialized target. Each clean and sparse walk compares **22,828 directory
pairs**. Sparse repairs exactly two files (8,355 logical bytes) and removes one
stale leaf. Subtree removes the owned clone of `@actions/artifact` and adds an
owned obsolete tree: sync copies 75 files / 359,798 bytes, creates 14 directories,
removes 100 leaves and one directory, and examines 22,814 paired directories.
Exact native paths and counters are in the command log/raw CSV.

Medians [ranges], milliseconds, n=5:

| Execution / scenario | Before | Lazy default | Lazy admin |
|---|---:|---:|---:|
| Serial clean | 947.110 [772.956–1142.506] | 831.291 [747.453–865.222] | 768.524 [728.512–841.813] |
| Serial sparse | 900.959 [816.162–1337.625] | 973.371 [750.341–1197.901] | 783.549 [750.360–913.547] |
| Serial subtree | 843.649 [780.915–1076.880] | 818.329 [754.729–1066.775] | 798.523 [726.130–1004.893] |
| P4 clean | 252.662 [231.490–377.766] | 355.740 [235.880–974.763] | 251.021 [238.934–400.151] |
| P4 sparse | 250.204 [245.762–338.714] | 329.372 [250.433–535.273] | 344.370 [248.170–656.690] |
| P4 subtree | 337.293 [252.889–390.060] | 288.173 [255.708–359.231] | 261.565 [247.105–706.894] |

All clean runs have zero completed operations. The admin check does not impose
the initial repeatable path-extraction penalty on this many-small/deep-directory
shape. However variance and negative samples remain, especially P4 sparse;
there is no defensible claim that the new library alone universally improves
the predecessor's latency. JW's eliminated eager passes and process work are
separate integration benefits requiring native command measurements.

This is **N=1 actual payload root**, the dominant node_modules root, not a census
or benchmark of every smaller `**/node_modules`/`**/dist` selection or the whole
feature fleet. Existing independent callers will re-inspect this donor once per
destination; concurrent walkers share pool workers but their J/K/memory budgets
add. Coordinated source-to-many streaming remains an explicitly preserved future
direction, not an implemented cache or shared-source speedup. A flat fixture must
not be substituted for this shape in subsequent fleet decisions.
