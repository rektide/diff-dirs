---
type: PerformanceReport
title: Sequential paired walk — synthetic btrfs fixtures and read-only live trees
description: Five warmed release observations at 29,916 and 191,891 synthetic entries per side, syscall audit, and actual opencode checkout/node_modules comparisons without mutation.
resource: /design/performance/performance0.sol61x.md
status: measured
generated: { by: "model:openai/gpt-6.1-sol#xhigh", at: 2026-10-18 }
sources:
  - resource: opencode://ses_eb3948bedffe0T37Xc2wI2S5Jg
    title: Implemented release harness, measured synthetic and supplied live roots read-only, audited payload IO, and preserved raw observations
  - resource: /perf/README.md
    title: Reproducible fixtures, scenarios, exact timing boundaries, and audit commands
  - resource: /examples/perf.rs
    title: Executed release benchmark and independent metadata census
  - resource: /perf/results/btrfs-30k.csv
    title: Five raw observations per small synthetic scenario
  - resource: /perf/results/btrfs-192k.csv
    title: Five raw observations per large synthetic scenario
  - resource: /perf/results/btrfs-192k-strace.txt
    title: Full large clean-walk syscall census (traced durations excluded from performance results)
  - resource: /perf/results/live-whole.csv
    title: Five actual whole-checkout read-only comparisons
  - resource: /perf/results/live-node-modules.csv
    title: Five actual paired node_modules read-only comparisons
  - resource: /design/implementation/implementation0.sol61x.md
    title: Core algorithm and completion/pruning interpretation
---

# Sequential walk measurements

## Environment and method

Measured 2026-10-18 on `workhorse.hosts.voodoowarez.com`, Linux
`7.1.0-debplus.1` x86_64, AMD Ryzen 7 5800X (8 cores/16 logical CPUs), btrfs on
`/dev/nvme0n1p6`. At measurement the filesystem reported 51 GB available of
3.4 TB (99% used). Fixtures were under
`/home/rektide/src/diff-dirs/.test-agent/perf/btrfs-{30k,192k}`. No tmpfs timings
are presented. The host was not isolated or CPU-pinned; cache and scheduling
noise are visible. No caches were dropped.

Compiler: actual `rustc 1.88.0 (6b00bc388 2025-06-23)`, default Cargo release
profile, `cargo +1.88.0 build --release --example perf`. The sequential core is
std-only. Each scenario has one untimed warmup then five in-process timings.
Setup, scenario mutations, printing, result validation, and a post-scenario
restoration check are outside timing. Root validation, full iterator
consumption, and summary construction are inside.

Synthetic-run executable SHA-256:
`782597cfff7ff0fb6dc277bac19b331b4fbc97551d63ef0cf66e62e399706ef1`.
After adding read-only repeat/census commands for the live request, the hash
was `b7c0c8b0f2b5913ae6223931cebb5b7f79e8186afacfcefcc71f4b4d5afc4048`.
The comparison core and timing boundary were unchanged.

## Synthetic results

The [method](/perf/README.md) gives exact generation/run commands. Each file is
empty: these are **metadata-shape fixtures, not 8 GB node_modules reproductions**.
Per-side entries exclude the root:

| Approximate fixture | Regular files | Directories | Symlinks | Total entries |
| --- | ---: | ---: | ---: | ---: |
| 30k | 28,950 | 773 | 193 | 29,916 |
| 192k | 185,700 | 4,953 | 1,238 | 191,891 |

Milliseconds, n=5; raw data is linked in Sources:

| Fixture | Scenario | Median | Min–max | Population stddev |
| --- | --- | ---: | ---: | ---: |
| 30k | clean | 76.893 | 71.707–93.276 | 7.471 |
| 30k | sparse | 69.266 | 68.180–71.634 | 1.286 |
| 30k | one-sided | 67.588 | 67.017–69.169 | 0.744 |
| 30k | prune | 0.010140 | 0.009710–0.015670 | 0.002278 |
| 192k | clean | 537.452 | 468.131–688.558 | 91.573 |
| 192k | sparse | 442.567 | 440.445–479.696 | 14.871 |
| 192k | one-sided | 443.921 | 442.164–521.693 | 31.095 |
| 192k | prune | 0.010330 | 0.009950–0.010921 | 0.000346 |

Clean examines 774 / 4,954 directory pairs and emits no records. Sparse examines
the same pairs and reports one changed, one added, one removed leaf in two
records. One-sided reports one added/one removed directory in one record and
examines 766 / 4,946 directory pairs: the two opaque package subtrees are not
walked. Prune examines only the root, reports the injected leaf, and skips all
paired child directories; it is **not a complete scan**, even though its
iterator exhausts normally. Different scenario timings are not evidence that
finding sparse changes is intrinsically cheaper than clean comparison; order,
cache warming, and nonisolated host variance confound that comparison.

## Syscall audit

`strace 6.13` clean large-fixture census: 383,784 `statx` calls exactly equals
2 root checks + 2 × 191,891 entry stats. There were 19,818 `getdents64`, 9,912
`openat` (including startup), and 2,476 `readlink` (2 × 1,238 links).
Five `read` and two `pread64` calls came from process startup, not payloads.

A separate fd-annotated audit on the small fixture found 1,548 fixture
directory opens, 386 symlink reads, and **zero fixture payload opens, data
reads, or mmaps**. [`perf/audit.py`](/perf/audit.py) checked all matching open
flags and read/mmap fd paths. The trace shows startup reads of dynamic libraries
and `/proc/self/maps`, which are expected. Traced durations are not used in the
timing table. Raw large census is retained; the larger annotated trace remains
gitignored scratch.

## Supplied live checkouts — strictly read-only

User-selected original: `/home/rektide/a/a/opencode`; target:
`/home/rektide/src/opencode-export-qa`. No contents in either checkout were
copied, reconciled, unlinked, or otherwise deliberately changed. The commands
use ordinary metadata/directory/readlink APIs; they are not snapshots against
external writers.

```sh
target/release/examples/perf repeat /home/rektide/a/a/opencode /home/rektide/src/opencode-export-qa 5
target/release/examples/perf repeat /home/rektide/a/a/opencode/node_modules /home/rektide/src/opencode-export-qa/node_modules 5
target/release/examples/perf inspect /home/rektide/a/a/opencode
target/release/examples/perf inspect /home/rektide/src/opencode-export-qa
target/release/examples/perf inspect /home/rektide/a/a/opencode/node_modules
target/release/examples/perf inspect /home/rektide/src/opencode-export-qa/node_modules
target/release/diff-dirs /home/rektide/a/a/opencode /home/rektide/src/opencode-export-qa --format json
```

The CLI was also compiled in release with Rust 1.88.0; SHA-256
`eb8aa7ba5fa23159cb56299cc30e78cf2ba85242d078cbd1e076618f4125e0fc`.
Its whole-root JSONL check emitted 49 records plus a complete summary and
returned **1 (differences)**, not an IO failure. No root, special-file,
permission, or traversal errors were encountered in these runs. All five
observations per scope reported identical counts:

| Scope | Median ms | Min–max ms | Stddev ms | Records | Examined directory pairs |
| --- | ---: | ---: | ---: | ---: | ---: |
| Whole checkouts | 1,078.270 | 914.680–1,353.231 | 141.573 | 49 | 25,203 |
| Paired top-level node_modules | 1,033.277 | 840.188–1,125.692 | 95.450 | 0 | 22,828 |

Whole-root difference counts: files changed **6**, added **11**, removed **2**;
directories added **69**, removed **0**. Topmost directly divergent directory
is root, because root itself differs. No `.git`, `.jj`, or other admin paths
were silently ignored. Root-only source directories `.beads`, `.git`,
`.test-agent`, and `.turbo` are each reported as opaque added entries; both-sided
`.jj` is actually descended and differs. Other records occur under ordinary
project packages/services. These differences are not solely an admin-tree
artifact. The selected `node_modules` pair is a separately labeled complete,
clean comparison, with no hidden filtering inside either root.

Independent metadata census (full enumeration, including opaque one-sided
subtrees; sizes sum logical regular-file lengths, not allocated bytes):

| Root/scope | Files | Directories | Symlinks | Specials | Entries | File bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Original whole | 223,185 | 26,367 | 9,607 | 0 | 259,159 | 17,605,711,065 |
| Target whole | 180,086 | 25,202 | 9,604 | 0 | 214,892 | 11,495,282,577 |
| Original node_modules | 160,407 | 22,827 | 8,571 | 0 | 191,805 | 8,291,060,621 |
| Target node_modules | 160,407 | 22,827 | 8,571 | 0 | 191,805 | 8,291,060,621 |

## Interpretation and next boundaries

The real supplied `node_modules` pair now supplies a reproducible local method
for an actual roughly-192k-entry/8.29 GB tree: clean warm median ~1.03s on this
machine. It is **not** retrospective validation of the original spec's lost
measurement. Neither these nor the synthetic results measure sync, reflinking,
reseed, or a comparison against rsync. They do not establish optimality of the
sequential backend; optional shared-pool parallelism is a separately authorized
follow-on with parity and comparative measurements required.
