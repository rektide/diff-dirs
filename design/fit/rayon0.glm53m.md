---
type: ResearchAssessment
title: "Rayon in diff-dirs: integration patterns, per-frame parallelism, and bounded lookahead heads"
description: Verified Rayon APIs and integration patterns for sharing the runtime (ambient global pool vs install-scoped private pool, feature gating, thread budget ownership, nested work), the safe v0 per-frame parallel design for the pull-iterator walk, and an architectural assessment of bounded multi-head lookahead answering whether DFS underutilization justifies 2–3 independent traversal heads — soundness against the mutation/prune contract, ordering/error determinism, memory bounds, and the caller-side root-pair alternative.
resource: /design/fit/rayon0.glm53m.md
tags: [diff-dirs, rayon, parallelism, dfs, lookahead, jj-waltz, performance]
status: draft
generated: { by: "model:zai-coding-plan/glm-5.3#max", at: 2026-10-18 }
sources:
  - resource: opencode://ses_eb39c53ccffeOncs3vs6bocqfU
    title: Rayon-research session — verified cached rayon/rayon-core APIs, designed integration patterns and the heads assessment in response to user and parent direction
  - resource: /README.md
    title: The diff-dirs specification — pull iterator, prune_here, record order, error contract the parallel layer must preserve
  - resource: /design/fit/io0.glm53m.md
    title: IO assessment (corrected) — async options, jj's rayon usage, and the source-referenced waltz execution map this builds on
  - resource: /design/fit/fit0.glm53m.md
    title: Fit assessment — embedder seam and memory-bound correction (C3) the heads analysis extends
  - resource: "crates.io cache:rayon-1.12.0"
    title: Verified with_min_len (src/iter/mod.rs:3159), rust-version 1.80, order-preserving collect and short-circuiting Result collection documented by example
  - resource: "crates.io cache:rayon-core-1.13.0"
    title: Verified ThreadPool::install implementation (thread_pool/mod.rs:137–143, registry.in_worker), ThreadPoolBuilder::num_threads/build/build_global, and RAYON_NUM_THREADS handling in lib.rs
  - resource: "archive:jj-vcs/jj@803ef00a"
    title: jj's rayon usage as precedent — rayon::scope work queue, into_par_iter().with_min_len(100), OnceLock first-error, sorted-merge determinism (cited in io0)
---

# Rayon in diff-dirs

## Situation

User direction: use rayon ourselves, and figure out integration patterns so we
*share* the runtime rather than build a private one. Follow-up question: rayon
gives parallelism, but is it *bounded*, and does a DFS walker underutilize it —
should 2–3 heads traverse independently? This document verifies the rayon APIs
from cached source, prescribes integration patterns, specifies the safe v0
per-frame design (what Sol is implementing), and assesses the heads idea
architecturally. No code is modified; no performance prototype is built here —
Sol's live-fixture measurements decide adoption.

## Verified rayon facts (cached source)

- **`ThreadPool::install`** (rayon-core `src/thread_pool/mod.rs:137–143`) is
  `registry.in_worker(|_, _| op())`: the pool that executes work is determined
  by the **execution site**, not by where an iterator was constructed or where
  a handle is held. Consequence for us: a walker holding a private
  `Arc<ThreadPool>` must wrap each parallel gather **inside
  `pool.install(|| …)` at the point the work runs** (inside `next()`); merely
  building the walker under a pool pins nothing.
- **Global pool**: `ThreadPoolBuilder::build_global()` initializes the process
  default; `RAYON_NUM_THREADS` overrides the thread count (rayon-core
  `lib.rs:30,110,172` — with `=1`, `join` degenerates to sequential). Bare
  `par_iter`/`join` outside any `install` uses the global pool, lazily
  spawned.
- **Nesting/ambient participation**: parallel work started from inside an
  `install` closure or another rayon job joins that pool. A library calling
  bare `par_iter` therefore *shares* whatever pool the embedder is currently in
  — the exact "share the runtime" semantics requested.
- **Order**: rayon's ordered/indexed iterators preserve input order on
  `collect` (documented by example in `src/iter/mod.rs`; `take`'s doc states
  items "maintain their relative order"). `with_min_len` (`src/iter/mod.rs:3165`)
  bounds job splitting — jj batches directory entries at 100 with it.
- **Errors**: collecting `Result` short-circuits; *which* error surfaces first
  is scheduling-dependent. Deterministic error selection must be built
  explicitly (below).
- **Version/MSRV**: rayon 1.12.0 declares `rust-version = "1.80"` — inside the
  embedder's 1.88. Any 1.x in an embedder's graph semver-unifies to one copy.

## Integration patterns (the recipe)

1. **Feature-gated, default off**: `parallel` feature adds `rayon` and the
   parallel gather. Default-off keeps the core's zero-dependency promise and
   leaves thread-shy embedders untouched. jj precedent: rayon is a hard dep
   there, but jj is an application; a library should opt embedders in.
2. **Ambient by default**: with the feature on, parallel gathers use bare
   `join`/`par_iter` — joining the caller's current pool when called inside
   one, else the global pool (`RAYON_NUM_THREADS`-respecting). This is shared-
   runtime participation with no configuration.
3. **Private pool as explicit opt-in only**: a builder `threads(n)` /
   `with_pool(Arc<ThreadPool>)` creates/holds a pool; **every** gather site
   must run inside `pool.install(...)` per the verified dispatch semantics.
   Document that private-pool threads *add* to the process total — the classic
   nested-pool oversubscription trap — so ambient is the recommendation and a
   private pool is for consumers with a deliberate budget (e.g., a CLI pinning
   walk threads below `nproc` while it also does process-spawn work).
4. **Thread budget ownership**: default budget is the global pool's
   (`num_cpus`, env-overridable); the library never sizes pools unless asked.
   jj-waltz, which has no rayon today, can enable the feature and immediately
   share a global pool across *its own* future loops (targets, selected root
   pairs) without coordination — no jj-lib reuse claim involved; jj is a
   subprocess.
5. **No persistent tasks**: all parallel work happens as scoped batches inside
   `next()` (`install` + `join`/`par_iter` complete before `next()` returns).
   Dropping the walker mid-iteration therefore leaves nothing in flight to
   cancel — cancellation-by-construction, no actors, no channels.

## v0: bounded per-frame parallelism (what Sol is implementing)

For each directory-pair frame, inside `next()`:

- `rayon::join(|| list_and_sort(original), || list_and_sort(target))` — the
  two listings of one frame gather in parallel.
- Merge-join the sorted names; for both-sided non-dir entries, stat both sides
  via a positional `par_iter` over the merged list with `.with_min_len(100)`
  (jj's threshold — "for a small directory, sequential scan should be fast
  enough"), collecting `(Option<Meta>, Option<Meta>)` per slot. Positional
  `collect` preserves order, so record contents are byte-identical to the
  sequential walker regardless of completion order.
- **Deterministic errors**: do *not* rely on `collect::<Result<..>>`
  short-circuiting (nondeterministic winner). Collect per-slot
  `Result`s into a `Vec`, then report the first `Err` in index order — error
  text and ordering match the sequential contract exactly.
- **Consumer-race safety by construction**: a pull iterator does work only
  while the consumer is blocked in `next()`; the consumer mutates only between
  yields (immediate children of records it holds). Therefore frame gathering
  races with nothing, `prune_here()` merely discards pending descent state,
  and `break`/drop abandon the walker with no background work. This is the
  load-bearing invariant: **no work outside `next()`, ever.**

Underutilization honesty: this parallelizes the *inside* of one frame. Top-of-
-tree frames are narrow (a workspace root has a handful of entries), so early
`next()` calls use little parallelism; deep wide frames (vendor packages)
amortize well. For wide payload trees (node_modules ≈ 192k entries, fanout in
the hundreds to thousands below the top), per-frame parallelism is expected to
capture most of the available speedup — an expectation to be *measured*, not
assumed.

## The heads question: bounded multi-head lookahead

### The idea, formalized

2–3 logical heads walk sibling subtrees ahead of the consumer. Rules (user's
sketch, made precise):

- A head may only exist under an ancestor whose record was **yielded, consumed,
  and not pruned** (prune's window closes when the consumer calls `next()`
  again).
- A head may traverse **clean** directories freely — clean dirs emit no
  records, so nothing the consumer can do (per contract) touches them.
- A head **must freeze at its first divergent record**: it buffers that one
  record (and its frame) and descends no further — the consumer may prune or
  wholesale-replace exactly there. No descent under an unresolved divergent
  ancestor.
- Emission stays strict DFS pre-order; each head contributes at most **one
  pending record** to a bounded buffer; errors are delivered in DFS order
  (same first-in-order selection as v0).

### Soundness against the mutation contract

A contract-abiding consumer mutates only immediate children of records it has
received, plus wholesale replacement of directories it just pruned. Sibling
subtrees under a consumed, non-pruned parent are therefore untouched until
their own records are yielded — heads reading them race with nothing, *given
the pull-iterator discipline above* (heads are logical cursors advanced inside
`next()`, not background threads; the consumer is still blocked during all
library work). The residual risk is contract-violating consumers mutating
un-yielded subtrees early; that already corrupts sequential semantics, and the
spec should say so plainly rather than the library defending against it.

### Bounded *what*, exactly

Three different bounds get conflated and shouldn't be:

- **Bounded threads**: rayon bounds these (global pool size or private
  `num_threads`). Heads do not add threads.
- **Bounded tasks**: scoped per-`next()` batches bound in-flight rayon jobs to
  the frames currently being gathered (≤ head count). No persistent tasks.
- **Bounded results/memory**: the real cost. Each live head materializes the
  sorted listings along its root-to-leaf path — the fit0 C3 bound
  (Σ current-path directory widths) **multiplied by the head count** — plus
  ≤1 buffered record each. A global head budget of 2–3 keeps this a small
  constant factor; a *per-depth* budget would multiply unpredictably with
  depth and is rejected.

### Costs and pathologies (the honest list)

- **Head-of-line stalls**: if the DFS-first head's divergent subtree is being
  slowly replaced by the consumer, later heads fill their single-record
  buffers and stop. Utilization decays to sequential — which is correct, not a
  bug: walking further would only speculate into subtree the consumer may
  prune.
- **Clean-tree full sweep**: in the steady-state *clean* payload there is no
  divergent record to freeze any head, so 2–3 heads race through the entire
  tree at pool speed and finish. That is the good case (the 1–2 s walk
  parallelized), not a pathology — but it is also the case where heads buy the
  most, and it is worth knowing that per-frame parallelism alone already
  parallelizes the wide portions of that same sweep.
- **Absent-diff scanning until first diff**: after a rebuild, the walk scans
  potentially large clean regions before the first divergence; heads hide none
  of that cost, they only overlap it.
- **Nesting**: heads' gathers run inside the same ambient/install pool as
  per-frame work — one work-stealing budget, balanced automatically. No
  per-head pools, ever.
- **Drop/cancellation**: scoped batches make this a non-issue — no in-flight
  work exists between `next()` calls.
- **Prune invalidation**: `prune_here()` must discard the pruned directory's
  descent state *and* any buffered record/frame under it. With ≤1 pending
  record per head this is a small, well-defined discard; it is nonetheless
  bookkeeping the v0 walker does not have.

### API obligations and risks

Obligations (all preservable): strict DFS record order; errors in DFS order;
`prune_here` legal immediately after any yielded record and discarding that
subtree's buffered work; `into_summary` counting only the examined portion;
memory bound stated as heads × path-width listings. Risks: the discard-on-prune
and ordered-delivery bookkeeping is where bugs will live; any accidental work
outside `next()` reintroduces consumer races; a future contributor turning
heads into persistent actors would break cancellation-by-construction. This is
why heads should be an explicit, documented design — not a clever weekend.

### The mechanically sympathetic alternative for waltz

jj-waltz's actual parallelism seam is *outside* the walk: `plan.ignored` is a
list of disjoint selected roots (node_modules, `**/dist`, …) and reseed runs
multiple targets sequentially (`engine.rs:561–589`, per the execution map in
[io0](/design/fit/io0.glm53m.md)). A caller that walks/applies those root
pairs — or targets — on the same ambient rayon pool gets embarassingly-
parallel speedup with zero library complexity, no ordering constraints (the
pairs are disjoint), and no new contract surface. For the embedder that
matters most, this is the first move.

## Recommendation

1. **v0 ships bounded per-frame parallelism** (the design above), feature-
   gated `parallel`, default off, ambient pool, `install`-wrapped only if a
   private pool is explicitly configured. Sol's live-fixture measurements then
   give: sequential baseline, per-frame speedup, and pool-utilization shape
   across the tree's depth.
2. **Caller-side parallelism for waltz**: parallel selected-root pairs
   (and/or targets) on the same pool — embedder change, not library change;
   mechanically sympathetic with its existing loop structure.
3. **Heads only on evidence**: add bounded lookahead (2 heads, scoped
   per-`next()` batches, ≤2 buffered records, global head budget) *if* the
   measurements show narrow-dir serialization — specifically, poor pool
   utilization during the tree's upper frames and a large gap between
   wall-clock and the syscall-count model. It is simple enough to be a v0.1
   addition behind the same feature, but it is real bookkeeping (prune
   discard, ordered errors, memory statement) and should not be pre-paid
   without the bottleneck.

## Open performance question

Does warm-cache btrfs stat work actually parallelize? The walk is ~2 lstats
per entry, ~400k syscalls at 192k entries; dentry/inode cache lookups are
kernel-side and may serialize on locks, so the achievable speedup is an
empirical curve (cores → wall time), not a given. The deciding measurements
(Sol's): (a) sequential walk on the 192k fixture; (b) per-frame rayon at 1/2/
4/8 threads on the same fixture; (c) utilization profile by depth (narrow top
vs wide vendor levels); (d) multi-root caller-side parallelism on the same
pool. If (b) plateaus far below the core count while (c) shows idle upper
frames, heads earn their complexity; if (b) scales or the plateau is
kernel-side, they never will.
