---
type: PerformanceEvidence
title: Conservative source-cache index charge correction
description: Bounded correction of a missing HashMap control-group trailer, with actual requested-layout regression and matched original N2 cache-clear evidence.
resource: /design/performance/cache-charge1.sol61x.md
status: measured-bounded-linux-standard
generated: { by: "model:openai/gpt-6.1-sol#xhigh", at: 2026-10-29 }
sources:
  - resource: "opencode://ses_e77885cc0ffeFqE7lKAwqkV7Oa"
    title: Reproduces the 16-byte undercharge, reviews exact current std/hashbrown sources, commits a private conservative index envelope and allocator regression, and records matched fixed-source measurements
  - resource: "opencode://ses_e7a2771c3ffez3sv1oPgD5gPH4"
    title: Parent commissions only the bounded provider accounting correction and retains maintained documentation and ticket integration
  - resource: /design/performance/selection0.sol61x.md
    revision: 2c892dad
    title: Original frozen source-sharing evidence and the 38933 versus 38949 byte accounting limitation
  - resource: /tree/inspect/src/source.rs
    revision: 48e8fd10952783ff57031c9aafd000499b3c64c7
    title: Corrected private index charge, unchanged batch ownership and cache lifecycle
  - resource: /tree/inspect/src/acquire.rs
    title: Successful-cell owned-capacity model, unchanged by this correction
  - resource: /tree/inspect/tests/cache_charge.rs
    revision: 48e8fd10952783ff57031c9aafd000499b3c64c7
    title: Actual cache-clear requested-layout regression across small fill and eviction cases
  - resource: "https://github.com/rust-lang/rust/blob/822182db4c4a1827256881d8c7aa685f0bcf2b11/library/Cargo.lock"
    title: This compiler's std uses hashbrown 0.17.1
  - resource: "https://github.com/rust-lang/rust/blob/822182db4c4a1827256881d8c7aa685f0bcf2b11/library/std/src/collections/hash/map.rs"
    title: HashMap delegates to hashbrown and does not promise an allocation-layout ABI
  - resource: "https://github.com/rust-lang/hashbrown/blob/c62a63a61b7caf2de8f9ecb7b06a66b0ab6bdf3d/src/raw.rs"
    title: Bucket load factors, effective capacity, shrinking and aligned allocation plus trailing control group
  - resource: "https://github.com/rust-lang/hashbrown/blob/c62a63a61b7caf2de8f9ecb7b06a66b0ab6bdf3d/src/control/group/mod.rs"
    title: Current group backend selection; reviewed SSE2, generic and NEON implementations fit the 16-byte envelope
  - resource: /.test-agent/cache-charge1/README.md
    title: Separate fixed-source archive, unchanged original fixture, allocator method and preserved raw receipts
verification:
  method: Failing-before allocator regression, 11 focused passing tests, strict affected Clippy and formatting, isolated HashMap layout census, and three matched N2 processes with two measured repetitions each
  scope: Current x86_64 Linux Standard backend and allocator-requested layouts; no timing, RSS, allocator-rounding or universal std-layout guarantee
---

# Source-cache charge now covers the observed release

**Code fix:** `48e8fd10952783ff57031c9aafd000499b3c64c7`. Internal only:
all package/API versions remain **0.7.0**. No public limit fields, policies,
source-sharing ownership, active pins, LRU selection or cache-off semantics change.
The corrected charge can intentionally evict sooner at a tight byte limit.

## Defect and chosen envelope

The [original N2 receipt](/design/performance/selection0.sol61x.md#actual-retained-requested-bytes-n2)
reported 38,933 charged bytes while clearing the idle cache released 38,949 net
allocator-requested bytes. Reviewing this compiler's std and hashbrown 0.17.1
confirms the missing **16-byte trailing control group**, not a fact/body defect.
The old `buckets * (bucket_size + 1)` covered ordinary control bytes but omitted
that group. `SourceBatch::charge` and `SuccessfulCell::charge` are unchanged.

The private `cache_index_charge` now charges:

```text
B = max(4, next_power_of_two(floor(8 * reported_capacity / 7)))
A = max(16, align_of::<(PathBuf, Cached)>())
index envelope = B * (size_of::<(PathBuf, Cached)>() + 1) + 16 + (A - 1)
```

Empty capacity charges zero; overflow saturates rather than wrapping or panicking.
The 7/8 adjustment also covers effective-capacity loss from deletions under the
existing shrink-after-removal lifecycle. The final terms cover a control group
and worst-case alignment slack, rather than attempting exact layout replication.
The complete index is still attributed only when at least one idle batch exists;
active-width storage remains outside the idle-retention charge.

This is an explicit **current-implementation envelope**, not a stable Rust HashMap
ABI or a universal heap bound. Recheck it when std/backends change. It does not
include allocator size-class rounding, allocator metadata, libc-internal storage,
RSS or simultaneous active inspection width.

## Bounded evidence

The actual public cache-clear regression failed before the fix at **one batch:
948 charged < 964 requested bytes released**. Afterward it passes 24 cases:
idle batch counts **1, 2, 3, 4, 7, 8, 14, 15, 16, 28, 29, 32**, each once on
initial fill and once after 64 visits through a 32-directory tiny test fixture.
It compares the reported charge against real thread-local allocator requests
released by `set_cache_limits(0, 0)`, not against a copied constant or formula.
Those tested source/test inputs match the fixed committed blobs byte-for-byte.

A separate empty-HashMap census reuses the original counting allocator and the
exact extracted private bound. Its value is explicitly an **index-layout proxy**,
not a fabricated source batch: the same four pointer/integer-sized value fields,
56-byte `(PathBuf, value)` buckets and alignment 8 on this platform. No filesystem
fixture is needed for this isolated table allocation:

| Reported HashMap capacity | Actual requested index bytes | Old charge | Fixed envelope |
| ---: | ---: | ---: | ---: |
| 0 | 0 | 0 | 0 |
| 3 | 244 | 228 | 259 |
| 7 | 472 | 456 | 487 |
| 14 | 928 | 912 | 943 |
| 28 | 1,840 | 1,824 | 1,855 |
| 56 | 3,664 | 3,648 | 3,679 |

For the production N2 measurement, the original 195-entry, zero-payload fixture
is reused **without modification**, with a before/after metadata census. The new
named archive pins all provider source to the fix commit; 85 archived files are
checked against it, with only the workspace member list narrowed. The harness
copies the original allocator and trial code. Compiler, default features, release
profile, arguments and registry versions/checksums match the frozen control.

| N2 control | Before charge | Fixed charge | Net requested cache-clear release |
| --- | ---: | ---: | ---: |
| Default cache, group kept | 38,933 | **38,964** | **38,949** |
| Default cache, group dropped | 38,933 | **38,964** | **38,949** |
| Cache off, group kept | 0 | **0** | **0** |

Each process performs the original warmup, open-window Parts rebind and two
measured repetitions. Every previous CSV field and phase value matches exactly
except the intended **+31-byte idle charge**. This includes allocation events,
requested live/peak values, zero-cost decision clones, Metadata/Facts lifetimes,
2 source listings, 65 source metadata acquisitions, zero cache hits and zero
effects. The default cache clears to the same 474-byte session state; the new
charge exceeds its observed net release by 15 bytes. This corrects accounting,
not allocation cost or memory residency.

Fixed executable SHA-256:
`a8625a0e645379e19a4dc6a55158c2130ee3df00560c525a1ba5e8f0fa0ec685`.
Raw CSV, commands, compiler/features/locks, hashes, isolated census and upstream
source snapshots live in [the separate scratch archive](/.test-agent/cache-charge1/README.md).
Original reports, source archives and frozen receipts remain untouched.

## Checks and stopping boundary

- **11 focused nextest tests pass:** cache owner/concurrency/LRU/pins, sparse and
  grouped reuse, fact release, public grouped lowering and the allocator regression.
- **Strict affected Clippy passes:** inspect package, all targets/features,
  `-D warnings`. Owned Rust files pass rustfmt with the native 160-column style.
- No full-workspace or native runtime suite, JW/production edits, timing or RSS
  measurement. No scheduler/index redesign, richer budget API, 4,699-allocation
  optimization or semantic-body shrinking.

The [original selection tradeoff](/design/performance/selection0.sol61x.md#wide-clean-group-sharing-works-but-memory-does-not-uniformly-shrink)
is unchanged. Broader allocation/body optimization needs its own selection and
evidence. Parent owns maintained-guide integration, tracking and any next slice.
