---
type: PerformanceEvidence
title: Actual wide clean group ownership and allocation receipt
description: Bounded 0.2 versus 0.7 production SyncCursor allocation, decision-clone, fact-release and caller-retained-view evidence, including increased transient allocation and cache retention.
resource: /design/performance/selection0.sol61x.md
status: measured-bounded-clean-only
generated: { by: "model:openai/gpt-6.1-sol#xhigh", at: 2026-10-29 }
sources:
  - resource: "opencode://ses_e779e67c4ffep3LwYT7qpuVLbK"
    title: Independently builds exact old/new production sources, measures tiny reused N1/N2 clean groups with a counting allocator and nonretaining fact-address observer, and records both benefits and negative memory findings
  - resource: "opencode://ses_e7a2771c3ffez3sv1oPgD5gPH4"
    title: Parent commissions only the bounded light-state receipt and retains maintained documentation, native integration and ticket closure
  - resource: /design/architecture/annealing0.gpt6ax.md#annealing-present
    title: Selection ownership is more than an Arc substitution; caller-held views and fact lifetimes need actual evidence
  - resource: /design/architecture/annealing0.gpt6ax.md#annealing-delivery
    title: Accepted wide-group copy and allocation evidence boundary, preserving the persisted model
  - resource: /design/architecture/execution0.sol61x.md
    title: Implemented selection/facts foundation and previously verified plan/group/barrier identity discriminator
  - resource: /design/architecture/execution1.sol61x.md
    title: Contextual frontier and saved-state ownership, distinct from this unit-context measurement
  - resource: /design/architecture/execution2.sol61x.md
    title: Direct production runtime, exact refresh sharing and promoted fact release
  - resource: /design/architecture/execution3.sol61x.md
    title: Grouped execution, bounded successful source retention and separately delivered no-effect autoaccept
  - resource: /design/architecture/execution4.sol61x.md
    revision: 23c18da7b9e3054f59a88f778cf6d6ebe624a677
    title: Exact stable 0.7 provider checkpoint; declared execution is not exercised here
  - resource: /perf/README.md
    title: Existing timing, diagnostic and process-resource scopes; no scoped allocator instrument found
  - resource: /perf/runner/README.md
    title: Existing archived controls, source identities and fixture discipline; its payload-import and timing campaign are intentionally not reused
  - resource: /design/observation/names0.gpt61sxh.md
    title: Prior allocator method and warning that fewer copies or packed names do not guarantee less retained memory
  - resource: /design/performance/inspection0.sol61x.md
    title: Prior distinction between userspace allocation and actual filesystem observation work
  - resource: /design/performance/reassessment0.sol61x.md
    title: Larger historical clean timing evidence, not a workload or performance claim reproduced by this receipt
  - resource: /.test-agent/selection-ownership/README.md
    title: Owned ignored harness, counting and address-watch method, exact builds and retained raw receipts
verification:
  method: Exact-source archives checked against committed blobs, same configured compiler and default library features, allocator self-check, eight bounded processes with two measured repetitions each, actual open-window Parts rebind and complete zero-effect reconciliation
  scope: Linux Standard backend, unit-context full compatibility policy, 64 empty equal leaves per side, N1/N2; allocator requested sizes and object lifetimes, not time, RSS, native command cost, syscall totals or all-feature performance
extensions:
  related_work: [diffdir-tree-coordinated-light-state, diffdir-tree-coordinated, diffdir-tree-observe-fact-reuse]
---

# Wide clean group: sharing works, but memory does not uniformly shrink

**The real production cursor now shares the selected body and releases clean
facts before returning. It does not make the whole 0.7 inspection path cheaper
in allocator operations or transient requested bytes.** On the bounded N2 group:

- Cloning both returned decisions: **132 allocations / 5,122 requested bytes →
  zero allocations / zero requested bytes**.
- Wide-call return residency above its starting state: **87,707 → 24,167 bytes**
  with the candidate source cache off. The old clean barrier is still pending;
  the new proven-no-effect barrier is already accepted. This is not all an Arc win.
- The wide call's allocation/reallocation events instead rise **2,345 → 4,699**;
  its extra requested-live peak rises **96,585 → 169,192 bytes**.
- Holding the returned group after cursor disposal retains **6,440 → 24,006 bytes**
  of group/semantic/token storage. The new, richer selection is larger, even
  though its fact arena is not retained with it.

All values are allocator-request accounting for these exact artifacts, **not RSS,
bytes copied, wall time or a speed claim**. No production or JW code changed.

## 1. Matched inputs and immutable identities

| Control | Exact committed provider source |
| --- | --- |
| Old 0.2 | `6e01c7da203a47fdb28aa24e38d614ddd471108c` |
| New 0.7 | `23c18da7b9e3054f59a88f778cf6d6ebe624a677` |

Both build configured `rustc 1.101.0-nightly`, commit
`822182db4c4a1827256881d8c7aa685f0bcf2b11`, LLVM 23.1.3,
`x86_64-unknown-linux-gnu`; Cargo `1.101.0-nightly (29c5daa1a 2026-10-07)`.
Default release profile; no extra rustflags/profile overrides. Resolved feature
sets for all four provider libraries are empty in both builds: **Standard listing,
no native-dirent, parallel or diagnostics feature**. The candidate observation
resource configuration is its actual default 64 KiB / 16 idle buffers / 1 MiB;
the Standard backend retains **zero raw-buffer bytes** throughout these runs.
The old provider has no corresponding pool or cross-call source-cache API.

Selected committed packages were archived into owned scratch, not checked out over
the working copy. Only each archive's workspace member list was narrowed to
observe/inspect/compare/sync. All other archived files, including package manifests,
source, compiled README inputs and original lock, match committed blobs: **56 old /
83 new files checked**. Harness locks are retained separately; common registry
dependencies have matching versions/source identities. New-only dependencies
include 0.7's BLAKE3 closure; no manifest hashing or listed runner is invoked.

Executable SHA-256:

```text
old 3acbdd13ebb69bbd8112ad4a6a8aa7b1dca2c041a53ca3e89660903cbe943089
new c82bf8c05bf5781c5f9e65cf62aade6c51c728a9efc4d028ea378607f38c70a9
```

The same Btrfs fixture is reused by both binaries:
`.test-agent/selection-ownership/fixture/{source,target-a,target-b}/Q/leaf-000..063`.
Each side has one Q directory plus 64 empty regular files: **195 ordinary entries
across the three roots, zero payload bytes**. N1 uses the first receiver without
changing the fixture; N2 uses both. Regular-file mtimes are fixed identically at
`1700000000000000000` ns. The final independent census verifies exact names/kinds,
empty sizes and those mtimes. Final inode values were recorded, but no initial/final
inode-equivalence claim is made. No production tree, large fixture, content import
or payload hash was used.

Both adapters call actual `Applier::new(..., CopyPolicy::Copy).visitor(Recorder)`,
`SyncRuntime`, `SyncCursor::new`, `next_group`, pending `apply`, `settle` and real
`SyncParts` validation/restoration. The callback only observes supplied references,
declares `scoped() == false`, and never changes a choice, excludes or refuses work.
Both use full-fact compatibility/StatKey semantics: equal leaves select KeepLeaf;
all groups have empty diffs, and completed effect counters are zero. No weaker
nonpersistent driver, synthetic selection or imitation inspection loop is used.

The old runtime's four policy type parameters adapt to 0.7's owned `FullPolicy`;
the candidate `DirectoryDecision` is an Arc alias rather than an owned DTO.
Candidate cache-off explicitly sets **0 batches / 0 bytes** before root inspection.
Cache-on uses actual defaults **32 batches / 8 MiB charged capacity**. Each trial
starts a fresh cursor/session, not a Q-cache-warmed generation.

## 2. Instrumentation and exact interval

The scratch allocator forwards to `System` and counts every successful Rust
`alloc`/`alloc_zeroed`, `realloc` and `dealloc` from process startup. Realloc is one
event and replaces its old requested Layout size with the new size, even in place.
Requested bytes sum new allocation/reallocation sizes; live bytes sum outstanding
requested sizes. Allocator rounding/metadata, libc-internal allocations, stack,
statics and realloc-internal temporary coexistence are outside this model.

Self-checks exercise ordinary allocation, growth, zeroed allocation, interior-address
free detection and complete byte balance. Snapshots, diagnostic reads and fact
watching allocate nothing; the recorded no-work `before_value_clone` interval has
zero allocator operations. Fixed stack arrays retain receipts and clones. There
are no worker threads, formatter calls or logs inside intervals; output follows
both trials. These are observed optimized-artifact operations, not universal
source-level allocation guarantees.

Before accounting, one cursor initializes library statics. Then one complete trial
is discarded, a separate open-window restore witness runs, and two trials are
recorded. Eight processes cover N1 cache-off kept-view old/new; N2 cache-off
kept/dropped-view old/new; and N2 candidate-default-cache kept/dropped views:
**16 measured Q returns**. Every counter/phase value agrees exactly between its
two repetitions. This is a small allocator census, not a timing matrix.

In each trial:

1. Baseline is after constructing the executable runtime, before cursor binding.
2. The root group is normally disposed and settled; its paired Q edge survives.
3. The wide interval is exactly the next production `next_group` call, through its
   returned group, before assertions, deliberate extra clones or application.
4. Clone each returned decision once into a fixed array, observe it, then drop it.
5. Keep or drop the original group; call `apply` only for actual Unstarted lanes.
6. Settle, assert terminal clean/satisfied and zero completed effects; validate and
   drop terminal Parts/cursor; drop the group, clear cache, drop the held session.

Each final snapshot exactly returns to its initial requested-live baseline.

Two failed **discarded warmups** are preserved: the observer initially inherited
conservative scoped coverage; then the balancing assertion exposed 62 process-
lifetime bytes from lazily initialized policy/schema/comparator IDs. Explicit
unscoped observational behavior and preinitialization corrected the harness.
Neither failure produced measured rows or required a production change.

## 3. Source proof versus measured copy/allocation evidence

At the exact old revision, `DirectoryDecision::of(plan)` allocates the metadata-free
projection, and `decision.clone()` deep-clones it into the barrier. The live
`DirectoryPlan` still owns its own Operations/names/paths/facts. For 64 clean entries
that is **three entry representations per lane**: the plan, projected group decision
and cloned barrier decision. Already-old source Facts are shared between lanes;
this receipt does not credit that existing sharing to 0.7.

At 0.7, [PreparedDirectory](/tree/inspect/src/prepared.rs) owns the fact vector and
one `Arc<DirectorySelection<C>>`. The
[sync decision alias](/transfer/sync/src/coordinated/decision.rs) introduces no
second semantic body. [Production next_group](/transfer/sync/src/coordinated/mod.rs)
clones that Arc into the group/barrier; application-needed work retains the same
preparation. These are **one body per lane**, not one policy body for all receivers.

For this clean group, no preparation remains at return: `ApplicationNeed::None`
autoaccepts and drops it. Actual pointer checks find group and barrier pointer-equal,
strong count **2 per lane**; old group/barrier entry-vector pointers differ. The
three-way plan/group/barrier identity and refresh case is the existing production
test `plan_group_and_barrier_share_one_immutable_selection_body` in
[the owner tests](/transfer/sync/src/coordinated/tests.rs), previously run by
[the foundation/runtime owners](/design/architecture/execution2.sol61x.md#verification-receipts).
Its source was read, not redundantly rerun or counted as a new measurement here.

The deliberately isolated caller-clone interval measures:

| Returned decisions cloned once | Old alloc / realloc / requested bytes | New alloc / realloc / requested bytes |
| --- | ---: | ---: |
| N1: one 64-entry body | 66 / 0 / 2,561 | 0 / 0 / 0 |
| N2: two 64-entry bodies | 132 / 0 / 5,122 | 0 / 0 / 0 |

The old per-lane count comprises its vector, 64 leaf-name allocations and containing
coordinate. This is real avoided deep-clone allocation, not a count inferred from
Arc's name. It does not imply zero atomic operations or zero cost to clone all Parts.

### Full wide-call census: increased allocation is real too

Values below are identical in both repetitions. Keep/drop choice happens later and
does not change the wide-call values. `alloc` includes alloc_zeroed; events count
alloc plus realloc. Live increase and peak are relative to **immediately before Q inspection**.

| Control | alloc | realloc | Events | Requested bytes | Live increase at return | Extra requested-live peak |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Old N1, no cache API | 950 | 464 | 1,414 | 212,325 | 51,698 | 64,862 |
| New N1, cache off | 2,019 | 824 | 2,843 | 291,556 | 12,712 | 137,574 |
| Old N2, no cache API | 1,550 | 795 | 2,345 | 357,757 | 87,707 | 96,585 |
| New N2, cache off | 3,197 | 1,502 | 4,699 | 479,563 | 24,167 | 169,192 |
| New N2, default cache | 3,196 | 1,502 | 4,698 | 479,319 | 62,128 | 168,948 |

Thus compact/shared selected state does **not** establish lower total allocation,
lower transient peak or lower long-held-view residency. The comparison spans the
real acquisition cells, compatibility adapter, richer selected schema, route epochs
and runtime resources introduced between 0.2 and 0.7. It is not a feature-toggle
isolation of Arc, packed names, cache bookkeeping or route checks. Source inspection
can identify those owners; these numbers cannot apportion their individual cost.

## 4. Fact release and caller-held views

The observer records integer addresses of actual Q source/target `Metadata` and
`Facts` supplied to the final compatibility callback. Fixed atomic slots deduplicate
source references across lanes. The allocator marks a watch dead when its containing
allocation is freed/reallocated. No Arc/Weak is retained, freed memory is never
dereferenced, and Metadata size/layout is not guessed. Old target Facts/Metadata
are inline in stable finalized Operations; new Metadata and compatibility Facts
are independently Arc-backed. This evidence concerns **Q facts**, not every root
binding observation, file descriptor, or recursive arena.

N2 observed **64 source / 128 target** Metadata identities and the same numbers of
compatibility Facts identities in both implementations. Their actual live counts:

| Boundary | Source Metadata | Target Metadata | Source Facts | Target Facts |
| --- | ---: | ---: | ---: | ---: |
| Old wide return, group kept or dropped | 64 | 128 | 64 | 128 |
| Old after both pending clean applications | 0 | 0 | 0 | 0 |
| New cache-off wide return, either view choice | 0 | 0 | 0 | 0 |
| New default-cache wide return | 64 | 0 | 0 | 0 |
| New default cache, after cursor **and group** drop | 64 | 0 | 0 | 0 |
| New after cache clear | 0 | 0 | 0 | 0 |

The old wide group has **N pending applications**. Applying its equal-leaf plans
does zero completed effects, but N2 disposal still makes **286 alloc + 304 realloc
calls / 65,442 requested bytes**, then releases those facts. The candidate has
**zero pending applications** and makes no application call or allocator operation
in that disposal interval. This [separately delivered no-effect behavior](/design/architecture/execution3.sol61x.md#separate-reporting-correction-and-v3)
is an essential contributor to earlier fact release and lower returned-call live
state. It must not be mislabeled as savings caused solely by Arc sharing. The
zero-planned opaque-work and equal-renewal counterexamples remain application-needed;
they are source/contracts from the runtime accounts, not weakened for this receipt.

### Actual retained requested bytes, N2

These values are relative to the trial's baseline **before cursor construction**,
so they also include that cursor's surviving state. Runtime bindings predate it.
The candidate's session Arc is deliberately held separately until the last phase.

| Control / view policy | Wide return | After disposal | Cursor/Parts dropped | Group dropped | Cache cleared | Session dropped |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Old, group kept | 91,328 | 15,760 | 6,440 | 0 | 0 | 0 |
| Old, group dropped before disposal | 91,328 | 9,336 | 0 | 0 | 0 | 0 |
| New cache off, group kept | 27,260 | 27,260 | 24,480 | 474 | 474 | 0 |
| New cache off, group dropped before disposal | 27,260 | 26,150 | 474 | 474 | 474 | 0 |
| New default cache, group kept | 66,209 | 66,209 | 63,429 | 39,423 | 474 | 0 |
| New default cache, group dropped before disposal | 66,209 | 65,099 | 39,423 | 39,423 | 474 | 0 |

Holding the new group therefore prolongs **24,006 bytes** of semantic/group/token
allocations after its cursor is gone, versus **6,440** old bytes. The rich new
selection includes explicit domain, authority, interior/context and interpretation
fields; a cheaper clone does not make its single retained body smaller. N1 similarly
retains 12,559 new versus 3,872 old group-owned bytes after cursor disposal.
With cache off, none of the observed Q facts survive with either view. Default-cache
source Metadata survives **independently of the view**, as intended.

Default diagnostics report **2 idle batches / 38,933 charged bytes / 0 active batches**;
off reports zero for all three. Both candidate configurations make two paired
source listings and 65 source-entry metadata acquisition attempts (root's Q entry
and Q's 64 leaves), with zero complete-listing cache hits. The retained default cache
buys no extra reuse on this one-pass shape. These are SourceSession diagnostics,
not total syscall counts, root/route checks, target observations or effects.

**Small accounting limitation worth retaining:** disabling that default cache
actually frees a net **38,949 allocator-requested live bytes**, 16 more than its
38,933-byte diagnostic charge. Clearing returns exactly to the same 474-byte
cache-off session state. Thus this receipt verifies finite intentional retention
under the configured charge model, but does **not** certify that charged capacity
is an exact conservative allocator bound. The source's cache-index envelope is a
candidate for later owner review; no production fix was attempted or needed to
obtain a sound measurement. The cache charge is never substituted for peak/RSS.

## 5. Preserved continuation/control boundary and stopping scope

Every process additionally uses actual `into_parts → validate → from_parts → bind`
at the open Q window. It checks saved barrier/window presence, fresh runtime token,
stale-token control refusal and restored `Head::Control(Q)`, then disposes actual
pending work, settles and verifies clean/satisfied zero-effect completion. Old
pending decisions refresh under the original interpretation; new accepted
no-effect decisions remain accepted. This is a **same-process data-only rebind**,
not serialization, fresh-process persistence, contextual policy or a claim of
V1/V3 wire compatibility. Existing richer contextual/process proofs remain with
[the runtime](/design/architecture/execution2.sol61x.md) and
[V3 owner](/design/architecture/execution3.sol61x.md).

Raw evidence remains ignored and locally retained under
[the scratch README](/.test-agent/selection-ownership/README.md):
`receipts/*-n*-*.csv` and stderr, `summary.json`, `identities.json`, feature trees,
resolved metadata, compiler/build/run receipts, original/resolved locks, final
fixture census, exact archived source and the two executables. This report embeds
the discriminating numeric values so the outcome is not dependent on scratch
survival. No reusable perf frontend is promoted without its owner's review.

Build/run shape, using those retained manifests and the same fixture:

```sh
cargo build --offline --release --manifest-path .test-agent/selection-ownership/harness-old/Cargo.toml --target-dir .test-agent/selection-ownership/build-old
cargo build --offline --release --manifest-path .test-agent/selection-ownership/harness-new/Cargo.toml --target-dir .test-agent/selection-ownership/build-new
.test-agent/selection-ownership/build-old/release/selection-ownership-old "$PWD/.test-agent/selection-ownership/fixture" 2 off keep
.test-agent/selection-ownership/build-new/release/selection-ownership-new "$PWD/.test-agent/selection-ownership/fixture" 2 off keep
.test-agent/selection-ownership/build-new/release/selection-ownership-new "$PWD/.test-agent/selection-ownership/fixture" 2 default drop
```

Not run: native/JW mutation, opaque/listed effects, a new full test matrix,
content/storage probes, native-dirent/parallel variants, a large benchmark, timing,
RSS, descriptor/syscall tracing or an independent fresh-process codec campaign.
The parent owns ticket closure, maintained-guide promotion and any follow-on
selection. The measured tradeoff is sufficient to replace an unverified blanket
“sharing reduces memory” claim with precise copy, residency and lifetime evidence;
it does not authorize another optimization slice.
