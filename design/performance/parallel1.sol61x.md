---
type: PerformanceReport
title: Directory frontier — single-root scale-out, admission limits, and measured costs
description: Same-build interleaved serial/per-frame/frontier measurements on actual supplied live roots and owned topology fixtures, with independent physical-job diagnostics, CPU, sampled RSS, and explicit limits.
resource: /design/performance/parallel1.sol61x.md
status: measured-awaiting-parent-review
generated: { by: "model:openai/gpt-6.1-sol#xhigh", at: 2026-10-18 }
sources:
  - resource: opencode://ses_eb3948bedffe0T37Xc2wI2S5Jg
    title: Implemented accepted bounded scheduler, verified ordering/liveness/mutation barriers, measured repeated actual single-root scale-out and topology costs, and collected physical/resource observations separately
  - resource: /design/concurrency/design0.gpt6am.md
    title: Accepted directory-frontier design and discriminating performance gate
  - resource: /design/implementation/implementation1.sol61x.md
    title: Implemented borrowed pool, J/K admission, demand reserve, seals, and ordered-prefix reducer
  - resource: /perf/README.md
    title: Reproducible commands, timing boundaries, instrumented counters, topology fixtures, and resource method
  - resource: /perf/results/frontier/live.csv
    title: Initial live node_modules P/J/K curve, five interleaved observations per backend/configuration
  - resource: /perf/results/frontier/shapes.csv
    title: Small, skewed, dense, wide, legacy synthetic, and whole-live-root observations
  - resource: /perf/results/frontier/guided.csv
    title: Guided K/J sweeps and an independent live P4/J4/K256 repeat
  - resource: /perf/results/frontier/confirmation.csv
    title: Final timing-build live-root confirmation and K=0 cost
  - resource: /perf/results/frontier/README.md
    title: Data bundle, separate diagnostic/resource files, and live-scope safeguards
  - resource: /perf/results/frontier/census.txt
    title: Fresh independent metadata census of both supplied whole roots and node_modules roots
  - resource: /design/performance/parallel0.sol61x.md
    title: Preserved per-frame-only baseline and redesign handoff
---

# Single-root directory scale-out

## Result and review boundary

The accepted directory frontier delivers a repeatable **single-root** improvement,
including the actual many-small-directory `node_modules`: final five-observation
median **247.010 ms versus 722.817 ms serial** at P=4/J=4/K=256 (~2.93×).
Whole supplied checkouts measured **282.329 versus 808.672 ms** (~2.86×).
Full records and summaries matched across all three backends outside timing.
Actual directory-job instrumentation, not the time curve alone, established
across-directory work and bounded physical concurrency.

This is a scheduler checkpoint for parent architectural review, **not a sync,
reseed, or mutation benchmark**. No jj-waltz edits, live copy/delete operations,
new scheduler family, wide-directory chunking, or separate sync crate are part
of it. The latter remains the next distinct authorized slice after review.

## Environment, binaries, and timing

2026-10-18, same host as the [preserved baseline](/design/performance/performance0.sol61x.md):
Linux `7.1.0-debplus.1` x86_64 on an AMD Ryzen 7 5800X, 8 physical / 16 logical
CPUs; btrfs `/dev/nvme0n1p6`, ~50–51 GB available on a 3.4 TB filesystem reporting
99% usage. No tmpfs speed claims, cache dropping, CPU affinity, exclusive-host
reservation, cold-cache results, or network-filesystem results. Host scheduling
noise and several large outliers are retained in raw data.

Actual compiler: Rust **1.88.0**, Cargo default release profile. Timing builds
enable only `parallel-bench`, not physical diagnostics. One caller pool is built
outside timing. Each compare checks complete record vectors and summaries,
warms all backends, then rotates serial / predecessor per-frame / frontier
order across five iterations in the same process. Root validation, iterator
and pool-install overhead, and summary consumption are timed; gathering parity
data and printing are not. Clean scans have no first-record latency value.
RUSTFLAGS, encoded Rust flags, and release-profile environment overrides were
unset; no repository release-profile override is present.

The [method](/perf/README.md#joined-frontier-versus-retained-controls) gives exact
commands. Raw data contains **495 timed observations**, all unfiltered:

| Experiment | Scope/configurations | Observations |
| --- | --- | ---: |
| `live.csv` | node_modules: P2/J2/K8, P4/J4/K16/64/256, P1/J1/K4, P8/J8/K32 | 90 |
| `shapes.csv` | six scopes × P2/J2/K8, P4/J4/K16, P4/J4/K256 | 270 |
| `guided.csv` | live P2/K32/128, P4/J2/K256, P4/J1/K256, P4/J4/K256 repeat | 75 |
| `confirmation.csv` | both live scopes × P4/J4/K0 and P4/J4/K256 | 60 |

SHA-256 values (same kernel/scheduler throughout; H2 adds the single-backend
resource helper and harness smoke tests, not a runtime scheduling change):

- Initial/guided/shape timing executable H1:
  `6fdad5d554a4203b7c6c4a678d67a4920c0b83f8c870fdbfa80b421b19a49b5c`.
- Confirmation/resource timing executable H2:
  `82c9ce556885fe4ffc07ecd978089f8a21c452e721df2b93952ef001484f748c`.
- Final separately instrumented executable:
  `a4a90b41d738b901cb772490f17298277a0f70503d02788c0b4730b775d9c3c8`.

Core checkpoint: `2a80af3d`; interleaved harness: `1629b130`; diagnostic stall
definition correction: `bd55938f`. Old per-frame scheduling remains only as
the hidden benchmark control, not a supported public strategy choice.

## Supplied live roots — strictly read-only

Original `/home/rektide/a/a/opencode`; target
`/home/rektide/src/opencode-export-qa`. Both whole-root and top-level
`node_modules` comparisons use `compare`/`diagnose`/`backend-once`, never fixture
mutation commands. There are no admin or ignore filters. One-sided directories
remain opaque units by the comparison contract; census enumerates them fully.

Fresh [census](/perf/results/frontier/census.txt), excluding each root itself:

| Root | Files | Directories | Symlinks | Specials | Entries | Logical file bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Original whole | 222,281 | 26,367 | 9,607 | 0 | 258,255 | 17,693,137,723 |
| Target whole | 180,812 | 25,312 | 9,604 | 0 | 215,728 | 11,500,520,536 |
| Either node_modules | 160,407 | 22,827 | 8,571 | 0 | 191,805 | 8,291,060,621 |

All current matrices agreed on scope/counts:

- node_modules: **clean**, 22,828 committed directory pairs, no records.
- whole roots: **132 records**, 25,243 committed directory pairs; files changed
  **524**, added **1,123**, removed **126**; directories added **36**, removed
  **7**. No prune and no traversal/parity error occurred.

The whole live roots changed between the older baseline account and this round:
these are not the earlier 49-record / 6-changed-file inputs. Historical reports
are preserved, not retroactively rewritten. Within each new comparison the
complete vectors/summaries matched; live roots are still not snapshots against
external writers. The common node_modules census remained unchanged.

Final H2 confirmation, milliseconds, n=5:

| Scope / P4 J4 K256 | Serial median (range) | Per-frame median (range) | Frontier median (range) |
| --- | ---: | ---: | ---: |
| node_modules | 722.817 (710.004–787.712) | 655.544 (616.810–772.584) | **247.010 (228.335–294.155)** |
| Whole roots | 808.672 (799.399–875.512) | 761.808 (713.061–837.358) | **282.329 (264.768–334.725)** |

Independent H1 node_modules K256 repeats measured frontier medians **263.886**
and **224.419** ms against their same-run serial controls **843.018** and
**707.902** ms. The win is not one favorable wall-time observation.

## Admission sweeps and actual utilization

Selected live node_modules medians in ms; each row compares three backends
within one process/build (cross-row host/cache variance remains):

| P / J / K | Serial | Per-frame | Frontier |
| --- | ---: | ---: | ---: |
| 1 / 1 / 4 | 807.859 | 821.494 | 792.899 |
| 2 / 2 / 8 (initial default) | 753.773 | 712.699 | 757.565 |
| 2 / 2 / 32 | 719.494 | 735.594 | 466.439 |
| 2 / 2 / 128 | 726.599 | 680.307 | 401.131 |
| 4 / 4 / 16 (initial default) | 755.245 | 648.919 | 606.201 |
| 4 / 4 / 64 | 756.337 | 726.534 | 324.646 |
| 4 / 4 / 256 | 843.018 | 788.125 | 263.886 |
| 8 / 8 / 32 | 786.996 | 721.229 | 483.674 |
| 4 / 2 / 256 | 716.103 | 672.089 | 415.598 |
| 4 / 1 / 256 | 748.611 | 643.834 | 772.380 |
| 4 / 4 / 0 (H2) | 726.400 | 671.771 | 771.152 |

The first weak P2/P4 results triggered K sweeps, not a new scheduler. Separate
instrumented runs in `diagnostics-live-*.txt` establish the concrete reason:

| P/J/K | Mean active jobs | Peak active / outstanding | Peak speculative / private nodes | Workers observed | Demand-reserve uses | K-stall observations |
| --- | ---: | --- | --- | ---: | ---: | ---: |
| 4/4/0 | 0.959 | 1 / 1 | 0 / 1 | 2 | 0 | 22,826 |
| 1/1/4 | 0.974 | 1 / 1 | 0 / 1 | 1 | 0 | 0 |
| 2/2/8 | 0.957 | 2 / 2 | 8 / 9 | 2 | 22,817 | 22,814 |
| 2/2/128 | 1.868 | 2 / 2 | 128 / 129 | 2 | 130 | 133 |
| 4/4/16 | 1.373 | 4 / 4 | 16 / 17 | 4 | 9,493 | 14,581 |
| 4/4/64 | 2.889 | 4 / 4 | 64 / 65 | 4 | 850 | 1,067 |
| 4/4/256 | 3.242 | 4 / 4 | 256 / 257 | 4 | 380 | 392 |
| 4/2/256 | 1.870 | 2 / 2 | 194 / 195 | 4 | 0 | 0 |

Every full clean run admitted/completed/committed exactly 22,828 directory jobs.
No re-inspection/eviction hides the budget cost. Mean active jobs is cumulative
inspection-window duration divided by instrumented wall time, not CPU or
kernel-syscall utilization. Windows include completion publication. K-stall
observations require unused J capacity plus an eligible child blocked by full
K; they are counts, not distinct stalled intervals or stalled duration.

This distinguishes **worker migration** from parallelism: P4/J2 used four worker
IDs over time but never exceeded two active jobs; K=0 used two IDs but only one
active job. Small-directory tests additionally force two different directory
jobs to rendezvous on distinct caller workers below the old metadata cutoff.

Defaults remain the accepted conservative starting point J=P/K=4P. They are
**not** a good throughput setting for this live topology at P2. Explicit K=256
at P4 is a measured configuration, not an automatic universal tuning rule.
Raising the public default is a parent review choice with width-sensitive
memory and multi-walker admission consequences. No global pool is resized.
A cursor-local speculative reserve or clean-subtree compression is another
possible future admission design from the proposal, not implemented here; it
would need separate bounds/prefix review rather than silently freeing K slots.

## Owned topology results and unfavorable cases

Fixture setup was separate and all subsequent comparisons read-only. Each small
leaf has two empty files; skewed places almost all work in the first branch.
Dense makes one source file per leaf one byte larger. Root-wide has no child
directories. Counts exclude roots:

| Fixture | Entries/side | Committed pairs | Records |
| --- | ---: | ---: | ---: |
| small, 16,000 leaves | 48,251 | 16,252 | 0 |
| skewed, 8,000 leaves | 24,129 | 8,130 | 0 |
| dense, 4,000 leaves | 12,064 | 4,065 | 4,000 changed files |
| wide, 30,000 root files | 30,000 | 1 | 0 |
| preserved ~192k shape | 191,891 | 4,954 | 0 |

P4/J4/K256 medians, ms, n=5:

| Fixture | Serial | Per-frame | Frontier |
| --- | ---: | ---: | ---: |
| small | 293.419 | 260.311 | **84.618** |
| skewed | 128.919 | 133.279 | **48.453** |
| dense | 63.430 | 62.428 | **49.481** |
| wide | 80.115 | **40.119** | 79.911 |
| preserved ~192k | 570.340 | 372.504 | **144.151** |

Small-directory frontier samples 81.092–150.313 ms were all below serial's
258.559–317.209 ms: the required many-small-directory benefit is discriminating.
At conservative P4/K16 the same fixture still improved 272.376 → 102.699 ms.
Dynamic first-branch fanout is separately proven by the skew test, not inferred
from this median. Legacy K256 had a 706.842 ms outlier; its median is not a
tail-latency guarantee. P4/K16 legacy samples were 142.573–147.361 ms.

Costs remain explicit:

- **Root-wide:** one directory job cannot scale out. Diagnostic peak active is
  one despite P4/J4/K256; the predecessor metadata backend is ~2× faster. This
  first slice deliberately has no nested metadata fanout that bypasses J.
- **Demand-only:** K=0 progresses but adds ~6% to the live median. J=1 similarly
  removes directory overlap. Pool installation alone is not scale-out.
- **Dense seals:** P2/K8 regressed slightly (62.127 → 64.669 ms). At P4/K256
  mean active jobs was only 1.136; 4,001 epochs accumulated 26.771 ms from ordered
  stop to scope join, max 2.889 ms. Peak speculation reached only 15, not 256:
  seal/epoch boundaries, rather than just K, limit this case.
- **First record:** dense P4/K256 medians were 0.447 ms serial, 0.298 per-frame,
  0.548 frontier (frontier range 0.529–4.870 ms). Whole-root first-record medians
  were 0.252, 0.212, 0.278 ms respectively in that matrix. Throughput improvement
  does not promise lower first-record latency.

For whole live roots the separate instrumented run had 133 epochs, mean active
3.278, cumulative scope drain 23.499 ms and max 7.676 ms. Drain includes join
scheduling/wakeup, not just time inside filesystem calls; the final outer
`install` return is outside that counter but inside recorded first-record time.

## CPU and memory observations

Separate one-backend H2 processes, not the parity collector; exact process CPU
includes startup. RSS is the largest child `/proc` VmHWM sampled every 5 ms,
**not** a strict peak bound (a final-gap peak can be missed).
All modes use the same feature-enabled executable and P4 pool (idle for serial),
not the minimal zero-dependency default-build footprint.

| Scope/backend | User s | System s | Sampled VmHWM KiB |
| --- | ---: | ---: | ---: |
| live node / serial | 0.175 | 0.593 | 3,884 |
| live node / per-frame | 0.247 | 0.699 | 4,104 |
| live node / frontier P4/K256 | 0.271 | 0.646 | 6,448 |
| wide / serial | 0.024 | 0.056 | 14,004 |
| wide / per-frame | 0.038 | 0.066 | 13,876 |
| wide / frontier | 0.024 | 0.060 | 13,796 |

On this live sample frontier used more total CPU than serial (~0.916 vs 0.768 s)
and more RSS while reducing wall time. The scheduler's retained-name high-water
was 89,230 bytes at live K256, versus 72,464 at K16, but that counter excludes
raw listings, diff data, capacities, allocator state, and worker stacks. The
wide fixture has zero pending child-name bytes yet ~14 MB sampled RSS: **J/K
never constituted a byte-memory limit**. Multiple walkers add their budgets.

## Verification and remaining review choices

Final checks: **53** all-feature nextest tests and actual Rust 1.88 tests;
**33** default-feature tests; default/all-feature clippy `-D warnings`; formatting;
two all-feature compiled doctests. Default `cargo tree -p diff-dirs
--no-default-features` still contains only diff-dirs. Windows/macOS and stalled,
cold, or network filesystem behavior are not claimed to have been exercised.

The accepted scheduler gate is met: actual distinct-directory work, repeatable
single-root many-small-directory/live benefit, strict DFS and failed-prefix
parity, global admission bounds, demand progress at K=0/P1/saturation, no reads
below speculative seals, and joined returns/drop. Review should decide default
K policy and whether root-wide shared-budget metadata chunks deserve a later
slice. No extra scheduler complexity was added to hide the measured weak cases.
The separate generic sync crate remains pending that review; live trees remain
read-only test inputs throughout.
