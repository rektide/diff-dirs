---
type: DesignProposal
title: Bounded directory frontier with an ordered commit cursor
description: Concurrent paired-directory inspection on a caller-owned Rayon pool, with deterministic DFS delivery, sealed divergence boundaries, bounded speculation, and prefix-only summaries.
resource: /design/concurrency/design0.gpt6am.md
status: proposed-for-review
generated: { by: "model:openai/gpt-6-astra#medium", at: 2026-10-18 }
sources:
  - resource: opencode://ses_eb37f1883ffeZ8HgL0aFBFJf4R
    title: Post-Sol concurrency redesign; separated directory execution from ordered commitment and specified scheduling, bounds, mutation protocol, and verification gates
  - resource: /README.md
    title: Current deterministic comparison, mutation, pruning, summary, and embedding contract
  - resource: /src/walk.rs
    title: Inspected frame traversal, comparison kernel, deterministic metadata errors, and current per-frame Rayon implementation
  - resource: /src/model.rs
    title: Execution policy, records, summary fields, and completion states
  - resource: /src/lib.rs
    title: Public exports and library trust boundary
  - resource: /src/error.rs
    title: Existing path-specific terminal errors
  - resource: /tests/walk.rs
    title: Serial contract tests, partial summaries, mutation, native names, and no-follow semantics
  - resource: /tests/parallel.rs
    title: Existing parity, apply/prune, and deterministic-error coverage
  - resource: /design/performance/parallel0.sol61x.md
    title: Primary handoff; measured per-frame concurrency limits and outstanding traversal redesign
  - resource: /design/performance/performance0.sol61x.md
    title: Environment, actual live-tree census, syscall evidence, and sequential measurements
  - resource: /perf/README.md
    title: Reproducible benchmark and strictly read-only live-root method
  - resource: /examples/perf.rs
    title: Inspected fixture shape, measurement boundaries, caller pool construction, and read-only repeat mode
  - resource: /perf/results/parallel.csv
    title: Recomputed retained 150-observation matrix with perf/report.py; no new performance observations
  - resource: /design/fit/fit0.glm53m.md
    title: Prior fit research, width-sensitive memory correction, and consumer integration distinction
  - resource: /design/fit/io0.glm53m.md
    title: Prior blocking-I/O research and separate sync-crate direction
  - resource: /design/fit/rayon0.glm53m.md
    title: Prior shared-pool and traversal-head proposal, including assumptions reconsidered here
  - resource: /design/fit/decisions0.gpt6a.md
    title: Earlier implementation direction; per-frame confinement is the choice now being redesigned
  - resource: file:///home/rektide/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rayon-core-1.13.0/src/thread_pool/mod.rs
    title: Verified install Send bounds and cross-pool cooperative execution warning
  - resource: file:///home/rektide/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rayon-core-1.13.0/src/scope/mod.rs
    title: Verified scope completion, descendant spawning, and panic-join behavior
---

# Recommendation

Build a **bounded, dynamically replenished directory frontier**, executed by
Rayon, beside a single **DFS commit cursor**. The cursor determines records,
errors, summaries, and caller decisions; it need not determine read order.
One job compares one directory pair, including its listings and metadata.
Clean results expose children immediately. Divergent results expose **no child
jobs until their record has been delivered and the caller advances**.

Keep the pull iterator and `prune_here()`. Run each demand epoch inside one
Rayon scope, joining before `next()` returns. Retain bounded sibling results
between calls, but no running tasks. This is a fresh choice for lifecycle
simplicity, **not** an assertion that background sibling reads are unsound.
Unlike the current implementation, a clean-tree `next()` can execute thousands
of directory jobs concurrently across the single root before returning `None`.

Add an explicit borrowed-pool entry point. Do not construct pools, require an
async runtime, or make independent caller roots the primary scaling mechanism.
Keep the existing correct serial implementation as the reference until this
proposal passes review and the replacement passes parity and measurement.

**Standing:** design only. No new scheduler or speedup is demonstrated here.
I read the implementation/tests/harness and recomputed the retained timing
matrix; no live tree was mutated or newly benchmarked in this session.

## Why this is materially different

Sol's live clean `node_modules` contains 22,828 directory pairs and 191,805
entries per side: about 8.4 entries per directory on average. The distribution
is not established by that mean, but the current 64-combined-entry threshold
plainly cannot be assumed to cover most directories. The observed matrix is
consistent with a granularity problem: ~690 ms serial versus ~669 ms at four
workers, while the synthetic 50-files-per-leaf fixture improves ~490 → 336 ms.
It does not prove whether scheduling or kernel contention dominates.

Directory jobs make `read_dir` and small-directory metadata runnable together.
They do not merely install a serial walker in a pool. Work replenishment is
completion-driven, not a lockstep batch waiting for its slowest directory.

Two or three *permanently assigned* subtree heads are too coarse: one large
branch can outlive the others, and an early divergent record parks its head.
Use a global job limit and a separate lookahead limit instead. Any completed
clean directory may supply the next independent job; Rayon workers have no
permanent subtree ownership. Two workers can execute two directory jobs; four
can execute four, even when every directory is below today's metadata cutoff.

## Terminology

- **Inspect:** perform a complete immediate comparison, producing a private
  result; this is physical work, not yet public coverage.
- **Commit:** accept that result at the ordered cursor, updating the public
  examined prefix exactly as the serial walker would.
- **Seal:** prevent descent below an unresolved divergent result.
- **Demand:** the next directory required by the DFS cursor.
- **Speculation:** inspection elsewhere in an already permitted branch.
- **Epoch:** work undertaken by one `next()` call, ending at a deliverable
  record/error or exhaustion; not a fixed-size batch.

# Public API and mutation protocol

Proposed surface, illustrative rather than a committed signature:

```rust,ignore
// Feature-gated; caller constructs this pool once and chooses its thread count.
let mut walk = diff_dirs::walk_in_pool(original, target, &app_pool, limits)?;
while let Some(item) = walk.next() {
    let record = item?;
    if replace_whole(&record) {
        walk.prune_here();
        replace_target_subtree(&record)?;
    } else {
        apply_classified_children(&record)?;
    }
}
let summary = walk.into_summary();
```

Prefer `ParallelWalk<'pool> { pool: &'pool rayon::ThreadPool, ... }` with its
own constructor over adding a lifetime to today's simple `Walk`. Each `next()`
calls `pool.install(|| run_epoch(...))`, not merely the constructor. This
establishes the requested explicit sharing even when construction and iteration
occur outside an ambient installation. The application may own `Arc<ThreadPool>`
and lend `&*pool`; no Arc is necessary inside a joined, borrowing walk.

Retain the existing ambient `Execution::Parallel` compatibility path, routed to
the same scheduler after review. It uses the pool at the execution site and
its documented global fallback. Explicit-pool use is the primary embedding
example; enabling a feature alone never enables parallel execution. Avoid
`threads(n)` on the walker: application worker count and per-walk lookahead
are different responsibilities.

Proposed `ParallelLimits` has nonzero `max_directory_jobs` (J) and
`max_speculative_directories` (K, zero permitted for demand-only execution).
Offer defaults derived from the selected pool, initially J=P and K=4P as
**measurement starting points**, with checked/saturating arithmetic. These
limits never create workers. An application can lower J below P to leave
capacity for other work. Validate nonsensical values at construction.

Multiple walks on the **same** pool share P workers, so their simultaneous
blocking syscalls together cannot exceed P in this design. Their queue and
memory budgets add: a shared pool is not a shared admission/memory budget.
Applications also bound concurrent walks. Shared-pool scheduling is not a
fairness or latency guarantee against unrelated long-running pool work.

### Allowed between records

1. The caller may apply the yielded record's **classified** immediate children.
   Paired directory children are not classified changes and stay intact unless
   the caller prunes that record or stops.
2. `prune_here()` seals off that record's pending descent permanently. A whole
   replacement of that directory is then allowed. Prefer prune-before-replace
   in examples, although no job is running between calls in this design.
3. The next `next()` implicitly acknowledges completion of immediate apply and
   closes the prune window. Only then can those paired children become jobs.
4. Earlier acknowledged branches, other pending siblings, and their ancestors
   must remain stable. An application that wants to change unrelated paths
   stops the walk and starts another. No asynchronous applier may outlive the
   acknowledgment unless it separately proves disjointness.

Namespace siblings are not automatically storage-independent: in-place writes
through hard links, bind-mounted aliases, or changes to shared ancestors can
invalidate buffered sibling results. The parallel contract requires mutations
not to change comparison keys in other pending paths. Unlink-and-create leaf
replacement avoids hard-link content aliasing; arbitrary in-place alias writes
need serial execution or a stronger caller proof. The core is still not a
snapshot or confinement boundary. Ordinary external concurrent writers remain
outside the stability contract.

This makes cached sibling results valid: their ancestor decisions already
closed, and the current record's permitted changes cannot affect them.
The meaningful README change is from “fully examined listings” to **“ordered
committed prefix”** for summaries, explicitly excluding speculative inspection.
For contract-abiding input and mutations, the observable summary matches serial.

No additional record acknowledgment type is needed now. A future background
driver would benefit from explicit decision/lease ownership, but adding it now
does not buy directory parallelism that the epoch design lacks.

# Concrete scheduler

## Persistent state

The implementation should separate the pure ordered reducer from I/O dispatch:

```text
ParallelWalk
  roots, pool reference, limits
  committed DFS stack: Vec<CommitFrame>
  bounded speculative arena: slot + generation -> Node
  last_yield: optional sealed frame
  summary, terminal state

CommitFrame
  relative path; sorted paired child names
  next child to commit; next child to admit
  sparse child-index -> live node handle
  direct-divergent-ancestor; subtree-changed

Node
  parent handle/frame + child index; relative path
  state: Queued | Running | Clean | SealedDiff | Failed
  result: paired child names, optional diff/error
  for Clean: lazy child cursor + sparse admitted-child handles

EpochState (short-held Mutex; borrowed by Rayon scope)
  persistent walk state
  queued-or-running job count; speculative slot count
  optional demand slot (distinct from K speculative slots)
  stop: Running | Yield | Error | Exhausted
```

`CommitFrame` retains only names needed for pending descent, not raw metadata.
Finished prefix children and their node handles are removed. Use generational
handles rather than raw arena pointers; no whole-tree result map or permanent
visited-node tombstones. A committed ancestor frame can have speculative child
handles; a speculative clean node can also have them. Commitment transfers
that node's child state to a frame without duplication or re-inspection.

DFS order is parent-before-child with child indices assigned by native basename
sorting. Do not use `PathBuf` lexicographic order as a substitute for the walk
order; component boundaries and encoding matter. The existing stack reducer
is the authority, and candidate priority follows component-index routes.

## Directory operation

Extract an owned `inspect_pair(path) -> Result<ComparedPair, Error>` from
`entries` + `compare`. It has no summary access and starts no descendant work.
For the first traversal implementation, do listings and metadata sequentially
*inside each directory job*, retaining current original-listing-first and
native-name/side metadata error precedence. Every successful result contains
the complete immediate diff and sorted paired child names.

This deliberately avoids nested per-entry job explosions while establishing
the missing across-directory scaling. Keep the old per-frame backend available
for comparison during development. After measurement, wide-directory metadata
chunks can share the same admission budget; do not combine J directory jobs
with J fresh metadata jobs per directory. A root with a huge listing is an
explicit weak point of the directory-only first slice, not a hidden regression.

## Admission and replenishment

All state changes below occur under the short scheduler lock; filesystem calls,
`install`, scope joining, and task execution occur outside it.

1. At epoch entry acknowledge the previous non-pruned record, opening its child
   cursor. If a cached demand result exists, run the reducer before spawning.
2. Reduce the longest available ordered prefix. A clean demand result commits
   its examined count and opens a frame, then the cursor enters its first child.
   Frame exhaustion performs the same post-order summary unwind as today.
3. A divergent demand result commits its immediate counts/topmost entry, pushes
   a **sealed** frame, and sets `stop=Yield`. An error at demand sets
   `stop=Error` without committing that directory. Exhaustion sets
   `stop=Exhausted`. Stop reducing at that exact boundary.
4. While Running and fewer than J directory jobs are outstanding, reserve work:
   first the unmet demand, then the earliest eligible unadmitted child in DFS
   order among open clean nodes and acknowledged frames. A sealed node has no
   eligible children. Enumerate child candidates lazily, one child at a time;
   never enqueue a wide directory's full child list as independent jobs.
5. Speculative nodes, whether queued, running, clean-with-children, completed,
   divergent, or failed, retain their K slot until committed/discarded. Reserve
   **one additional demand slot** which speculative work cannot consume. When
   a speculative node becomes demand, transfer its accounting to the demand
   slot and free its speculative token. Only one demand exists at a time.
6. Reserve handles and job counts under the lock, unlock, and `scope.spawn`
   the finite dispatch list. Each closure inspects its owned pair, then locks
   to publish its result, decrement the job count, reduce, and reserve newly
   runnable jobs. It unlocks before spawning those successor closures.
7. Once Yield/Error/Exhausted is selected, no successor work is admitted.
   Already reserved jobs finish their one directory and store the result,
   including jobs still queued when stop was selected. Join the scope, then
   return the selected item. Do not commit those trailing sibling completions
   while draining the scope. This avoids cancellation holes in child admission
   cursors in the first implementation.

Initial dispatch is the root demand. Completion supplies the first fanout.
There is **no dedicated coordinator thread and no receiving worker loop**:
the completing worker briefly performs coordination and then dispatches work.
The external caller waits using Rayon's scope join, not a blocking result
channel. The state outlives that scope. A scope ending with `Running`, no jobs,
and unmet demand is an invariant failure, not a signal to silently finish.

An implementation can tail-loop one successor in the completing closure to
amortize tiny directories, but must expose other reserved successors to Rayon
first and retain the same counters. Do not make unbounded recursive Rust calls
along directory chains. Start without this optimization and measure dispatch
cost before introducing a bounded directory quantum.

### Why the demand reservation is necessary

Suppose later siblings occupy all K slots while the cursor reaches an unread
child of the first subtree. Without a dedicated demand slot, no result can
commit, no slot can free, and a nominally bounded frontier deadlocks. The
reserved slot lets that child run and the cursor advance. It guarantees
progress, not full utilization: later sealed records can still occupy the
entire speculative budget and leave only demand work runnable.

First implementation retains results rather than evicting and re-reading them.
If measurements show prolonged budget saturation, compare a near-cursor
reserved sub-budget or clean-subtree completion compression in a follow-up.
Neither should silently discard errors or change prefix accounting. Increasing
K is an explicit throughput/memory choice, not a cure for arbitrary skew.

## Lifecycle and invariants

```text
unadmitted -> Queued -> Running -> Clean -> committed/open frame -> exhausted
                               -> SealedDiff -> delivered/sealed
                                                  -> acknowledged/open
                                                  -> pruned/exhausted
                               -> Failed -> delivered terminal error
```

Clean speculative nodes may expose children before their own commitment, but
only when every divergent ancestor is already acknowledged. A `SealedDiff`
node never has child jobs. Consequently pruning a just-delivered frame cannot
invalidate descendant results: **there must be none**. Assert this rather than
building an expensive normal-path subtree cancellation mechanism. A pruned
frame drops its unadmitted child names and follows the existing summary unwind.
Sibling results remain valid under the mutation protocol.

Cancellation when a record/error becomes deliverable stops admissions. The
first slice finishes each already-reserved directory: at most J such jobs,
not an unlimited traversal tail. A later refinement can abandon private
speculative listings between syscalls, with an explicit unread-node state and
retry bookkeeping. There is no cancellation of an in-progress
blocking filesystem syscall, and no finite wall-time guarantee on a stalled
filesystem. Never turn an aborted private comparison into a public I/O error.

At every public yield there are zero jobs. `Drop`, `break`, and `into_summary`
therefore trigger no new reads or waits; they discard private results.
`into_summary` only unwinds committed frames. Panics follow Rayon propagation
after scope joining; do not catch them and pretend to produce a normal partial
record. No user callback runs inside the scheduler lock.

# Bounds, ordering, and progress

## Resource bounds

Let P be pool workers, J the per-walk queued-or-running directory limit, K the
speculative slot limit, D the committed cursor depth, and W(v) the retained
names/path/result storage for directory v.

- Simultaneous blocking filesystem calls from all walks on one pool: at most P.
  One walk uses at most min(P,J). Each directory job issues sequential calls.
- Reserved/queued/running directory jobs per walk: at most J and at most K+1.
  Effective concurrency is therefore min(P,J,K+1).
- Uncommitted directory nodes: at most K speculative plus one demand.
  Their raw listings while running are charged to those same slots.
- Buffered, unreturned divergent records/errors: at most K+1; no unbounded
  channel or result Vec. The currently returned record is owned by the caller.
- Persistent traversal memory is O(D+K) structural nodes plus
  `sum W(committed frames) + sum W(uncommitted nodes) + Summary`.
  Each running pair can temporarily retain both raw listings, result vectors,
  and comparison output: the constant-factor allocation peak must be measured.
- Open directory resources scale with running directory widths too: Rust
  `DirEntry` retention can keep directory resources alive. Drop raw entries as
  soon as converted; do not claim only two descriptors for the entire walk.

These are global per-walk bounds, never per-depth multiplied permits. They
bound outstanding units, **not arbitrary filesystem width in bytes**. A single
giant directory already defeats a constant-byte claim in today's sorted API;
so does the caller-requested summary on a highly divergent tree. If a hard byte
limit becomes a requirement, add a documented oversized-directory failure or
external sorting; a “memory semaphore” acquired after reading is not a bound.
A soft retained-byte high-water mark may stop new speculative admissions, but
can overshoot by the unknown widths of J in-flight directories and must be
labeled accordingly. It is not necessary for the initial count-bounded slice.

## Determinism and prefix summaries

Workers never update `Summary`. Only the cursor does so, in the same enter and
exit order as the current serial stack. Thus:

- Successful immediate comparisons increment examined/counts on entry.
- Direct-divergent-ancestor state is assigned at commitment, not guessed by a
  worker. Topmost divergence and post-order changed ancestors remain identical.
- A failure in later sibling `z` can finish before `a` but cannot be returned
  before the cursor reaches `z`. An earlier failed directory wins regardless
  of which physical call first encountered trouble.
- At an early record in `a`, inspected clean/error/diff results in `z` contribute
  nothing to `into_summary()`. Even clean inspections need ordered commitment.
- After terminal error, discard speculative state, preserve `Failed`, and fuse.
  After explicit pruning, normal exhaustion is Complete with the pruned list,
  hence not full coverage, exactly as today.

Within a directory retain current error selection: original listing errors
before target listing errors, listing errors before metadata, metadata by
native name then side. Parallel directory execution does not change that rule.
Physical error observations can differ on externally changing trees; no
scheduler can promise snapshot-equivalent errors without a snapshot.

## Pool safety and liveness

No worker waits for a channel message, a queue vacancy, an admission token, or
another job while holding the state lock. Full admission means stop spawning;
it never means block a worker. The lock only serializes finite state updates.
Reserved successor closures are submitted before the parent closure finishes,
so scope completion covers all descendants. A one-thread pool executes them
sequentially and makes progress. Nested consumption on the same pool likewise
uses cooperative scope/install waits rather than occupying all workers with
blocking receivers.

Calling the explicit API from a different pool works through `install`, but
Rayon can run other work on the calling pool while waiting. Embedders must not
hold application locks across such calls when reentrant work needs those locks.
The library itself holds no scheduler lock across `install` or scope join.
Reusing one pool avoids adding worker budgets; selecting a different pool is
the application's explicit resource decision.

Assuming finite directory operations and fair eventual execution, progress
follows: the demand is already running, has a reducible result, or can use its
reserved slot as outstanding jobs complete. Every successful commitment or
frame exit advances DFS; a divergent/error result ends the epoch. No recursive
fanout can consume demand capacity. Filesystem stalls and unrelated pool tasks
that never finish are external limits, not solved with extra worker threads.

# Expected behavior and real tradeoffs

### Two/four workers on many small directories

With a clean root containing `a,b,c,d,...`, root inspection initially uses one
worker. Its completion admits up to J child pairs: at J=2, `a` and `b`; at J=4,
`a,b,c,d`. Each job performs real enumeration/stat/readlink calls concurrently.
When `a` finishes clean, the cursor commits it and exposes `a`'s first child;
that job immediately replaces it. A worker finishing `b` can expose `b`'s child
even while the cursor is inside `a`. There is no wait for the other initial
siblings to finish. Discovered clean fanout inside a huge first subtree can
also recruit free slots; scheduling is not frozen to root partitions.

The actual `node_modules` topology must be profiled rather than assuming these
four example siblings describe its root. A chain of wrappers remains serial
until it reveals fanout. A large root listing is serial in the first slice.
Once independent directory pairs exist, metadata counts below 64 do not stop
scale-out. Clean steady-state work can keep two/four workers busy through most
of the traversal **if** the bounded window supplies enough independent work.

Things that stop that progress are concrete: unresolved divergence at an
ancestor; lack of fanout; K retained ahead-of-cursor results; a slow demand
directory; J/P limits; epoch drain at a record; kernel/storage contention; or
other users of the shared pool. Record order itself causes head-of-line
blocking. Increasing workers cannot remove a consumer decision barrier.

### Sparse, dense, and skewed trees

- Clean: one long epoch, no per-record scope overhead. This is the primary
  supplied `node_modules` target, and the best case for dynamic replenishment.
- Sparse: broad clean stretches still parallelize. Stop at each ordered diff,
  preserving cached siblings; reopening the acknowledged branch restarts work.
- Dense: up to K future records can park; every branch seals at its own record.
  Frequent scope drains add latency, and serial may win. This is the necessary
  cost of unrestricted immediate prune decisions under ordered delivery, not
  grounds to speculate into sealed descendants.
- Skewed: later siblings can consume K while a slow first branch advances only
  through its demand slot. This is an honest bounded-lookahead limitation. It
  deserves a deliberately adversarial benchmark, not only balanced fixtures.
- Cold/network storage: blocking overlap may help much more, or too many jobs
  may increase queueing/tail latency. Warm btrfs results cannot select J for it.
- Tiny directories: mutex/arena/spawn cost can dominate. Directory quanta or
  safe wide-frame chunks are optimizations to measure after the scheduler is
  proven, not reasons to preserve a serial traversal architecture.

Background sibling tasks could overlap caller application work, but cost a
long-lived scope/driver or owned tasks, explicit quiescence on cancellation,
and potentially blocking Drop. They also consume the shared pool while the
applier wants it. Since the clean target has no application gaps to overlap,
that complexity does not address its primary bottleneck. Reconsider only if
measured sparse apply workloads justify it. Similarly, io_uring would change
the backend/platform burden without removing the ordering and decision problem.

# Implementation and verification gate

## Coherent first implementation slice after review

1. Extract the comparison kernel without changing classification/error rules.
   Build a pure cursor/frontier reducer with simulated inspection completions.
2. Add the explicit-pool `ParallelWalk`, global J/K limits, demand reservation,
   scoped completion-driven dispatch, and no internal metadata parallelism.
   Keep serial and current per-frame modes available to the benchmark until
   the comparison is made. Production naming/removal follows parent review.
3. Preserve full records, prefix summaries, apply/prune, and native path handling.
   Update README pool/mutation/summary/memory explanations with the accepted
   design, and measure against the baseline before choosing defaults.

This is one architectural slice: kernel + bounded scheduler + existing external
protocol. It does not introduce generic task traits, a public event graph,
async streams, copy operations, or a new synchronization framework.

## Correctness experiments

Use a test-only inspection seam to supply trees and completion permutations;
do not depend on filesystem timing to exercise scheduler state transitions.

| Property | Discriminating test |
| --- | --- |
| Actual across-directory work | Four small sibling directories, each below the old threshold; gate the directory-inspection boundary so two distinct caller-pool workers must enter distinct pairs before either completes. Bound the test gate timeout. |
| DFS/prefix parity | Enumerate completion permutations on small branching trees; after every record compare serial records and `into_summary`, including clean speculative predecessors/siblings. |
| Demand progress | Fill K with later ready results, make next cursor child unread, use P=1 and K=0/1 as well as P=4; assert completion without stealing speculative capacity. |
| Dynamic balancing | One huge first subtree, several tiny siblings; prove jobs from the huge subtree run on multiple workers after tiny siblings finish. |
| Sealed boundary | Return a divergent parent with an instrumented forbidden descendant; prune and replace parent in an owned fixture; assert zero descendant inspections, even before the record was delivered. |
| Sibling caching | Complete later sibling first; apply/prune current branch; verify the sibling result remains correct, and no duplicate committed counts occur. |
| Error ordering | Force a later sibling error to finish first; introduce earlier directory and same-directory side/name errors; select serial-first error and exactly the serial failed prefix. |
| Stop/drop | Force one result to become deliverable while J-1 siblings run; after next returns assert no active jobs and no later inspection events. Drop/into_summary starts no I/O and omits sibling results. |
| Resource bounds | Assert queued+running<=J, speculative<=K, demand<=1, no descendants under sealed nodes, arena generation validity, and no growing finished-prefix tombstones. |
| Shared-pool liveness | P=1/2/4; invoke from outside pool, inside same pool, and several same-pool parallel walks; no channel waits. Observe caller thread names at actual I/O boundary. |
| No-follow and opacity | Existing native symlink/dangling/cycle tests plus symlink-to-directory and one-sided special-file subtrees: never admit them as paired children. |
| Application convergence | Reuse the test applier on owned fixtures across sequential/parallel policies, then run a clean second diff. No mutation experiments on supplied live roots. |

The no-follow argument is unchanged: only two `Directory` kinds, obtained via
non-following entry metadata, create paired-child candidates. `read_link` is
literal leaf data. The scheduler must never invent a task from a path suffix
or follow a link to manufacture fanout. Concurrent ancestor replacement remains
the documented external confinement limitation.

Run existing workspace nextest, all-feature checks, doctests, fmt, clippy, and
Rust 1.88 tests as in the [development instructions](/README.md#development-and-evidence).
These future tests are required because scheduling changes observability and
lifecycle, not merely reversible surface formatting.

## Performance experiment that can reject this proposal

Extend the existing release harness to compare **in one build**: sequential,
current per-frame, and frontier; one application pool built outside timing.
Sweep P=1/2/3/4/8, J<=P, and K=0/P/4P/16P, beginning with P=2/4 and K=4P.
Interleave policy order, perform warmups, retain raw observations, and report
medians/ranges plus process CPU, peak RSS, first-record latency, and epoch-drain
latency. Use more repetitions than the existing n=3 for close comparisons.

Instrumentation should record active directory jobs over time, actual worker
IDs, admitted/completed/committed counts, peak arena/name bytes, and time stalled
for demand, seal, or budget. Keep instrumented utilization runs separate from
uninstrumented timing. A throughput curve without these counters cannot
distinguish kernel contention from a starved scheduler.

Use existing synthetic fixtures plus owned small-directory, deep-chain,
wide-root, dense-delta, and skewed fixtures. The live pairs are strictly
read-only, with original `/home/rektide/a/a/opencode` and target
`/home/rektide/src/opencode-export-qa`, both whole-root and paired
`node_modules`; use repeat-style commands only. Never pass these roots to
fixture setup/mutation helpers, never drop host caches for this exercise, and
never modify jj-waltz. Compare full output/summary parity outside timing, not
just aggregate entry counts.

Success requires both actual concurrent distinct-directory inspection and a
repeatable benefit on the supplied single-root `node_modules`, with explicit
memory/latency cost. If four workers are consistently active but wall time
barely improves, investigate filesystem contention before expanding K. If
workers idle with eligible children, fix scheduling/admission. If lock/spawn
overhead dominates, test bounded quanta. If only wide-root cases regress, test
shared-budget metadata chunks. No speculative numeric speedup is promised.

# Separate sync consumer and review choices

A separate optional `sync/` crate can use exactly the loop above. It applies
one delivered record, restores mtimes according to its own policy, optionally
prunes before wholesale replacement, and calls `next()` only after apply joins.
It may use the **same pool** for independent immediate child operations later;
diff tasks have already joined, so there is no blocked producer competing for
those workers. Copy/reflink, deletion order, confinement, retry, and progress
remain outside the diff core.

Parent review should resolve three substantive choices before implementation:

1. Accept directory-frontier concurrency with an ordered reducer and mandatory
   divergence seals, retaining the pull/prune protocol.
2. Accept borrowed explicit-pool `ParallelWalk` alongside ambient compatibility,
   and prefix-summary plus mutation-stability wording. An owned Arc wrapper is
   optional convenience, not required for shared budgeting.
3. Accept joined epochs and count-bounded K/J as the first slice, measuring
   dense/skewed costs before adding background execution, eviction/compression,
   or wide-frame chunk sharing.

## Connections

- [Sol's handoff](/design/performance/parallel0.sol61x.md#handoff-boundary) is the
  measured reason to redesign; its baseline remains the comparison control.
- [Earlier heads analysis](/design/fit/rayon0.glm53m.md#the-heads-question-bounded-multi-head-lookahead)
  supplies the valid “stop at divergence” insight. This proposal replaces fixed
  heads and the expectation that per-frame work captures most speedup with
  dynamically admitted directory work and explicit prefix accounting.
- [Earlier decisions](/design/fit/decisions0.gpt6a.md#shared-rayon-execution)
  explain why the current implementation confines work to one frame. This is
  a proposed revision of that implementation choice, not a silent override of
  the maintained specification.
- [Fit research](/design/fit/fit0.glm53m.md#corrected-integration-requirements-supersedes-the-first-wave-seam-advice)
  remains relevant to eventual jj-waltz adoption: faster comparison alone
  cannot recover payload already deleted by the embedder's earlier phase.
- [Design reading index](/design/index.md) links this proposal alongside its
  independent predecessors and the existing implementation evidence.
