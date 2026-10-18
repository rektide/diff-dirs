---
type: PerformanceReport
title: Own the leaf pipeline — controlled clone and staging experiments
description: Same-compiler controls isolate successful cleanup, held Linux descriptors, and parent-batch staging without changing public comparison or synchronization contracts.
resource: /design/performance/leaf0.sol61x.md
status: measured-awaiting-parent-review
generated: { by: "model:openai/gpt-6.1-sol#xhigh", at: 2026-10-18 }
sources:
  - resource: "opencode://ses_eb2c07f5cffe2uTLIC7b1rrD99"
    title: Implemented isolated leaf ownership variants, retained same-compiler controls and unfiltered owned-fixture measurements, and verified safety and failure behavior
  - resource: /design/performance/inspection0.sol61x.md
    title: Actual syscall anatomy, current compiler identity, and corrected std copying and dependency behavior
  - resource: /design/performance/opportunities0.gpt6am.md#2-establish-the-right-leaf-operation-boundary
    title: Selected operation ownership experiment and independently attributable cleanup, clone, and staging variants
  - resource: /design/performance/opportunities0.sol61x.md#a01
    title: Independent cleanup, shared staging, held clone descriptor hypotheses and rejection criteria
  - resource: /design/sync/brief0.gpt6a.md
    title: Preparation safety, exact timestamp verification, hardlink safety, joined progress, and unpinned compiler direction
  - resource: /design/sync/implementation0.sol61x.md
    title: Existing implementation and test contracts; historical Rust 1.88 evidence remains historical
  - resource: /design/performance/sync0.sol61x.md
    title: Prior 30k owned fixture shape, not a current-compiler control
  - resource: /perf/leaf.rs
    title: New owned-source and private-target benchmark with independent verification and process CPU clocks
  - resource: /perf/results/leaf/README.md
    title: Exact controls, compiler, fixture shapes, timing boundaries, raw data and executable identities
  - resource: /perf/results/leaf/observations.csv
    title: All 2350 main observations, five repetitions per cell without filtering
  - resource: /perf/results/leaf/confirmation.csv
    title: Separate final-source endpoint confirmation, three repetitions per cell
  - resource: /perf/results/leaf/resources.csv
    title: Whole-process wait4 CPU, wall and peak-RSS resource observations, separate from API timers
  - resource: /perf/results/leaf/descriptor-isolation.csv
    title: Nine interleaved repetitions price held clone descriptors after staging and metadata handoff already landed
  - resource: /perf/results/leaf/syscalls.csv
    title: Separate tiny-fixture syscall census, including unchanged ordinary Copy kernel offload
  - resource: /perf/results/leaf/leaf-lifecycle.txt
    title: Normalized actual first-leaf clone lifecycles, including close ordering and exact mtime readback
---

# Leaf pipeline experiment

Selected work is internal preparation-through-install ownership, not donor reuse,
rich public records, FD-native traversal, copy-length specialization, or timestamp
verification weakening. Root README is owned by the parent session.

## Controls preserved before implementation

Starting checkout was clean at `7614ea08`. Configured compiler is actual
`rustc 1.101.0-nightly (822182db4 2026-10-13)`, LLVM 23.1.3. The new probe builds
against unchanged production code and saves its executable before edits. Scratch
manifest lock, binaries, raw traces, generated sources, and private targets live
only under `.test-agent/leaf-pipeline/`. Sources in prior fixture/live roots are
not modified or synchronized.

Variants: unchanged; known-empty-success cleanup; that cleanup plus held Linux
clone descriptors with per-leaf staging; then parent-batch staging. Source sizes
and syscall operations are held constant. Every successful sync is checked using
the independent core walk outside timing. CPU uses the process clock including
caller-pool workers. One P4 pool is created outside the timers and reused for
the run's scenarios. Initialization/setup and target damage are outside timing.

Initial baseline verification: 43 default workspace nextest tests passed with
`DIFF_DIRS_SYNC_REQUIRE_REFLINK=1`; nonempty clone tests ran on real btrfs.
Tiny two-file/one-link strict and Copy traces were captured separately from
performance. Original Rust 1.88 reports are not causal controls for these results.

## Main controlled results

Milliseconds, median of five, without removing outliers. Full ranges and process
CPU are recoverable from the raw CSV. All variants keep exact timestamp readback,
hardlink-safe staged rename, public validation and joined execution constant.

| 1000 × 4-KiB siblings | Unchanged | Cleanup only | + descriptors | + shared arena | + recursive metadata handoff |
| --- | ---: | ---: | ---: | ---: | ---: |
| Required, serial fresh | 114.374 | 102.144 | 96.535 | 56.291 | 51.542 |
| Required, serial dense | 150.199 | 137.024 | 133.331 | 85.542 | 93.709 |
| Required, P4 fresh | 73.369 | 79.974 | 74.827 | 40.478 | 41.316 |
| Required, P4 dense | 104.132 | 97.709 | 90.427 | 62.704 | 55.901 |
| Copy, serial fresh | 115.178 | 105.413 | 137.019 | 56.925 | 54.852 |
| Copy, P4 fresh | 81.402 | 76.018 | 79.152 | 41.941 | 39.507 |
| Copy, P4 dense | 100.588 | 99.389 | 97.578 | 59.088 | 60.041 |

Shared staging is the decisive mechanism. Its strict P4 fresh CPU median falls
from 255.396 ms with per-leaf staging to 133.767 ms, dense from 266.820 to
167.257 ms. The descriptor-only control shows a smaller serial fresh benefit,
but **not a consistent independent P4 fresh wall win**. Copy has no new clone
work to remove: its descriptor-stage differences are a negative/noise control,
not an ordinary-copy acceleration. No claim is based on summed percentage gains.

Recursive handoff removes one duplicate source stat during opaque copying. Its
fresh serial strict CPU median falls 55.148 → 51.334 ms after shared staging,
but dense application does not use that handoff and its slower median is noise
or storage scheduling, not evidence that the removed call harmed dense work.
Keep this small internal handoff without inventing a rich public record API.

### Does the descriptor backend earn its cost after staging?

Because the independent P4 fresh signal was weak, an additional narrow control
switches only the Linux file backend back to the existing path helper, retaining
final batch staging, metadata handoff and prefix handling. Nine alternating
repetitions on 1000 × 4-KiB siblings, Required policy only, followed explicit
warmups. Median milliseconds:

| Execution/scenario | Staging + existing clone backend | Staging + held descriptors |
| --- | ---: | ---: |
| Serial fresh | 59.211 | 53.352 |
| Serial dense | 97.034 | 92.129 |
| P4 fresh | 42.723 | 47.429 |
| P4 dense | 61.833 | 65.025 |

The serial result supports retaining the small native adapter (~10% fresh / ~5%
dense at these medians). Paired per-round serial median ratios are 1.075 fresh
and 1.068 dense, with 6/9 and 8/9 wall wins. P4 fresh is a **negative result**
(2/9 wins, ~0.916 paired ratio); dense has mixed noisy median/paired behavior
(6/9 wins, 1.064 paired ratio despite the slower aggregate median). P4 CPU is
essentially flat fresh, not a claimed latency benefit. This does not justify a
broad native backend: retain only the small safe whole-file primitive, with
portable fallback unchanged. The reason is measured serial return plus a simpler
single descriptor/metadata lifetime, not pretending all parallel runs improved.

## Syscall attribution, not traced wall-time claims

The separate tiny probe installs two 4-KiB files and one link fresh, two changed
files dense, then one changed file sparse: five regular installations, six total
leaf installations. Comparison, setup, verification and disposal are present in
the **whole-process** census; identical fixed work is not claimed as API-only.

| Required-policy syscall | Unchanged | Cleanup | + descriptors | + shared arena | + handoff/final |
| --- | ---: | ---: | ---: | ---: | ---: |
| openat / close, each | 72 | 66 | 61 | 61 | 61 |
| statx | 249 | 237 | 232 | 235 | 231 |
| chmod | 7 | 7 | 2 | 2 | 2 |
| fchmod | 10 | 10 | 5 | 5 | 5 |
| mkdir | 9 | 9 | 9 | 6 | 6 |
| rmdir | 6 | 6 | 6 | 3 | 3 |
| getdents64 | 98 | 86 | 86 | 86 | 86 |
| FICLONE / utimensat, each | 5 | 5 | 5 | 5 | 5 |
| rename | 6 | 6 | 6 | 6 | 6 |

- Empty success cleanup removes six unnecessary calls per arena (two stats,
  open, two getdents, close). Failed preparations still use recursive cleanup.
- Each regular Linux clone loses one reopen/close pair, one source-fd stat and
  two permission calls. Exact timestamp setting and descriptor readback remain.
- Shared arenas reduce temporary mkdir/rmdir from six leaf arenas to three
  touched batches. Private umask-safe ownership checks add one stat per arena;
  recursive handoff removes four direct duplicate source stats on this fixture.
- Copy still makes **10 copy_file_range calls** across its five regular copies
  (data plus EOF), with no ioctls, under every variant. Descriptor-only Copy has
  exactly the cleanup control's syscall counts. Nothing introduces kernel offload
  that std already had, and there is no manual known-length optimization.
- The path-filtered census records zero arena-mentioned closes after held
  descriptors rename: strace prints the **new visible path** on their closes.
  Whole-process opens/closes match; this is changed close ordering, not leaked FDs.

Normalized [first-leaf traces](/perf/results/leaf/leaf-lifecycle.txt) preserve the
source/output opens, FICLONE, exact utimensat/readback, final chmod, rename and
close ordering. Full raw traces remain in the owned scratch subtree. Census
durations are never mixed with the untraced timing observations.

### Larger same-shape confirmation

30k nonempty 4-KiB files, 300 directories and 300 links; P4/J4/K256. This is a
fresh owned fixture with the prior report's shape, not its retained source or a
cross-compiler comparison. Median (min–max), milliseconds, five observations:

| Policy/scenario | Unchanged | Descriptor + batch staging + handoff |
| --- | ---: | ---: |
| Copy fresh | 1719.543 (1673.120–3558.346) | 881.839 (874.821–3699.572) |
| Required fresh | 1788.745 (1564.897–2066.101) | 954.482 (850.518–1937.154) |
| Copy dense | 3527.572 (3208.082–5933.098) | 1790.361 (1753.867–4816.015) |
| Required dense | 3218.822 (3047.873–3633.715) | 1961.726 (1867.369–4573.237) |
| Copy clean | 22.096 (21.067–29.275) | 21.822 (21.071–25.880) |
| Required clean | 23.802 (21.170–25.609) | 22.458 (21.930–27.576) |
| Copy sparse: one leaf | 22.681 (21.574–26.089) | 22.737 (22.107–23.769) |
| Required sparse: one leaf | 22.817 (21.516–23.288) | 22.918 (22.042–29.880) |

Fresh is about **1.95× Copy / 1.87× Required** faster at these medians. Fresh
process CPU medians fall 5702.399 → 2971.085 ms Copy, 5623.076 → 2985.423 ms
Required. Dense is ~1.97× / 1.64× at the medians, with large tails retained.
Clean and sparse stay walk-dominated: there is no scan cache or no-op speed claim.
The main comparison-only controls remain roughly 22–25 ms, with an unfiltered
122.787-ms candidate outlier.

After the case-insensitive prefix guard and final failure tests, a separate
three-round final-build confirmation retains the fresh effect: Copy 1712.210 →
885.715 ms, Required 1935.760 → 999.796 ms. Dense Copy 3144.160 → 2136.799 ms,
Required 3274.691 → 2077.900 ms. This is supporting evidence, not replacement
of the original five-sample results or a reason to filter their outliers.

The separate whole-process resource sample includes setup, deliberate target
damage, verification and final disposal. Median system CPU is lower (Copy
16.753 → 10.555 s; Required 16.825 → 10.228 s). `wait4` peak RSS is 12904–13036
KiB for the controls and 13036 KiB for candidates. The uniform floor may include
pre-exec launcher memory; it supports **no useful memory improvement claim**,
and is not a bound on wider/deeper operation lists or allocator retention.

## Negative and limited cases

- Single-leaf batches cannot amortize mkdir/rmdir. Strict serial sparse median
  is 0.182 ms in descriptor-only versus 0.205 ms shared / 0.227 ms handoff;
  shared housekeeping is not universally free. Absolute changes are small and
  ranges overlap; no special small-batch scheduler is justified by this sample.
- Eight 4-KiB siblings benefit clearly in fresh serial Required (1.281 →
  0.667 ms end-to-end), but sparse is mostly fixed root/walk cost.
- Eight 4-MiB files are few clone operations, not a physical-throughput study.
  Required P4 fresh median is 1.636 ms unchanged versus 1.641 ms handoff; dense
  is 1.740 versus 2.225 ms. These noisy/negative cases are retained. Copy can
  exploit filesystem CoW via std/kernel offload; it makes no allocation promise.
- Successful fallback-policy rows on this btrfs pair report actual clones, not
  copies. Real unsupported cross-filesystem fallback is verified in tests, not
  given a new cross-filesystem performance claim.
- Source content and hardlink safety are checked by dedicated tests. Benchmark
  verification uses the independent core's declared metadata/link contract,
  not hashing or an extent-sharing assertion.

## Selection and limits

Keep the small internal ownership boundary and parent-local lazy arenas; do not
generalize this into FD-native traversal, a backend trait, donor catalogs, trusted
compare/apply tokens, known-length copying, timestamp-readback removal, clone
capability caching, or a new application scheduler. Public APIs are unchanged.

Linux/btrfs is actual execution evidence. Other platforms keep the prior clone
backend and are cfg-based best effort. Parent-local staging preserves placement
around potential nested mounts, but no new mount namespace/bind-mount test or
hostile-race protection is claimed. No live root was synchronized or modified;
there is no jj-waltz or end-to-end consumer speedup claim.

Implementation and final verification: [leaf ownership account](/design/sync/leaf0.sol61x.md).
