---
type: ImplementationAccount
title: Joined directory frontier with ordered prefix commitment
description: Implementation of the accepted bounded J/K scheduler, reusable arena, reserved demand, speculative divergence seals, borrowed pool API, and discriminating correctness experiments.
resource: /design/implementation/implementation1.sol61x.md
status: implemented-awaiting-measurement
generated: { by: "model:openai/gpt-6.1-sol#xhigh", at: 2026-10-18 }
sources:
  - resource: opencode://ses_eb3948bedffe0T37Xc2wI2S5Jg
    title: Resumed after accepted Astra design; extracted inspection kernel and implemented/reviewed bounded completion-driven frontier, explicit borrowed sharing, prefix reducer, and correctness experiments
  - resource: /design/concurrency/design0.gpt6am.md
    title: Parent-accepted design and correctness/measurement gates
  - resource: /README.md
    title: Updated maintained public pool/mutation/memory/prefix contract
  - resource: /src/pair.rs
    title: Immediate inspection kernel and retained per-frame measurement control
  - resource: /src/frontier.rs
    title: Pure ordered reducer, arena, admission, seals, and scoped completion-driven dispatch
  - resource: /src/parallel.rs
    title: Borrowed-pool API and NonZero J/K limits
  - resource: /src/frontier/tests.rs
    title: Completion permutations, bounds, demand reserve, real directory concurrency, seals, skew, and lifecycle experiments
  - resource: /tests/frontier.rs
    title: Shared/nested pools, prefix and error parity, and disposable apply convergence
  - resource: /design/performance/parallel0.sol61x.md
    title: Previous per-frame limitations requiring across-directory work
---

# Directory-frontier implementation

## Accepted architecture and owned simplification

The parent accepted all three review choices in the [proposal](/design/concurrency/design0.gpt6am.md):
directory-frontier execution with ordered reduction and mandatory seals,
borrowed explicit pool plus ambient compatibility, and joined epochs with
global per-walk J/K counts and reserved demand. This implementation changes no
classification, root, local error precedence, or default sequential behavior.

One directory job performs listing, sorting, metadata/readlink, and immediate
comparison sequentially. It starts no children and writes no public summary.
The completing worker briefly publishes under the scheduler mutex, runs the
pure reducer, reserves at most J outstanding jobs, unlocks, and submits newly
runnable closures to the same Rayon scope. Scope submission is reversed so
local Rayon LIFO favors the first reserved demand while thieves take siblings.
No worker waits on a result channel or admission semaphore. No lock is held
across filesystem calls, scope join, or pool installation.

The implementation uses the proposed reusable generational arena, but children
need no sparse index map: admissions are an ordered prefix of remaining child
names, and reserved jobs drain rather than cancel. Two deques (names and live
admitted handles) encode that prefix. Consuming a child pops both; committing
its ready node moves its child deques into the DFS frame. There are no duplicate
child results, permanent visited tombstones, or recursive Rust traversal calls.
Candidate search follows child positions through open clean nodes, not Path
lexical order. This is an ownership simplification, not a changed bound or
ordering rule.

## Invariants

- Queued + running jobs <= J; speculative nodes <= K; arena live nodes <= K+1.
- Speculative nodes retain their slot through completion, errors, and clean
  descendants until promoted/committed. Promotion frees their speculative slot.
- An unmet cursor directory reserves its own slot before speculative admission,
  so later results filling K cannot prevent ordered progress.
- A differing private result exposes no descendants. The delivered frame stays
  closed until next-call acknowledgment or explicit prune. Prune asserts there
  are no descendant handles to cancel.
- Only successful ordered commitment updates `examined_directories`, counts,
  topmost entries, and post-order changed ancestors. Physical completions after
  an epoch chooses its record/error are stored, not committed.
- Every epoch joins all reserved work. At every public return there are zero
  jobs. `into_summary` unwinds committed frames only; drop discards private
  state without IO or waiting. Panics follow scoped Rayon propagation.

The bounds constrain units, not bytes. A wide listing can be arbitrarily large,
and each running job can own two raw listings and conversion buffers. Pending
committed path names and the summary also consume memory. Arena slot storage
is reused at at most K+1 high-water slots; ready results remain inline to avoid
one extra heap allocation per directory completion.

## Public surface

`walk_in_pool(original, target, &ThreadPool, ParallelLimits)` returns
`ParallelWalk<'pool>`. Each `next()` installs its scope in that stored borrowed
pool; callers need not maintain an ambient install. `ParallelLimits` makes J
nonzero at the type level and permits K=0. `for_pool` starts with J=P/K=4P.
Pool construction and process-global configuration stay with the application.

`walk()` is unchanged and sequential. Ambient `Execution::Parallel` now uses
the same frontier, creating its default limits at first iteration. The hidden
`parallel-bench` control keeps the preceding metadata-only backend measurable
without adding a supported strategy enum. `parallel-diagnostics` counts private
work, high-water units/names, occupancy, workers, budget stalls, demand reserve
use, and epoch drain separately; instrumented runs must be distinct from speed
observations.

## Correctness evidence at the scheduler checkpoint

The existing record, native-name/link, local-error, and summary tests continue
to run against the ambient frontier as well as serial mode. New experiments:

- **720 completion-priority permutations** of a branching tree with two sealed
  differing nodes and an early-completed later error: same DFS records, exact
  prefix snapshots, first error, and post-order summary. A forbidden sealed
  descendant panics if ever inspected.
- **Saturated K:** finish a later sibling first, then demand an unread child
  of the first branch. The dedicated demand slot runs while K remains full.
- **K=0/1/8 and 500 siblings:** progress and reusable arena storage <= K+1,
  with all live references checked and no finished-prefix accumulation.
- **Actual small directories:** two below-old-cutoff sibling inspections must
  rendezvous on distinct caller-named workers before either proceeds to real
  filesystem inspection. The test gate has a finite timeout.
- **Skew:** first branch waits for tiny siblings to finish, then reveals fanout;
  two different workers enter its descendants. No permanent subtree ownership.
- **Seals/lifecycle:** force a later differing sibling to finish first; forbid
  child reads, drain to zero active jobs at yield, prune-before-replace an owned
  target subtree, then use the cached sibling. Early drop starts no new work.
- **Pools:** P=1/2/4 and K=0/1/4P, outside/inside same pool, same-pool concurrent
  walkers through nested join, and cross-pool invocation all progress and match
  serial output. No application locks are held by these tests.
- **Apply:** test-side unlink/create and subtree copy across all six kind
  transitions, literal links, non-UTF-8 names, and a nested changed file; the
  next explicit-pool walk is clean. No mutation of supplied live roots occurs.

Performance is the next checkpoint, not inferred from the successful gates.
Dense seals, bounded lookahead saturation, root-wide listings, mutex/spawn cost,
and filesystem contention remain concrete possible throughput limits.
