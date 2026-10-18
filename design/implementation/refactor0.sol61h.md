---
type: ImplementationDesign
title: Runtime-neutral seams — scope assessment and paused larger proposal
description: Recommends a pure engine module and separate Rayon crate; preserves the unimplemented larger package/Tokio proposal for comparison.
resource: /design/implementation/refactor0.sol61h.md
status: assessment-awaiting-scope-selection
generated: { by: "model:openai/gpt-6.1-sol#high", at: 2026-10-18 }
sources:
  - resource: opencode://ses_eb335933bffeUeaPkxqOtKDOWi
    title: Recorded the initial larger proposal, paused before code changes on user direction, and assessed a smaller two-crate scope and Tokio costs
  - resource: /design/concurrency/design0.gpt6am.md
    title: Existing ordered reducer, demand reserve, seals, and completion-driven admission design
  - resource: /design/implementation/implementation1.sol61x.md
    title: Tested generational arena and joined Rayon implementation being extracted
  - resource: /design/performance/parallel1.sol61x.md
    title: Baseline single-root performance, explicit P4/J4/K256, conservative defaults, and width costs
  - resource: /README.md
    title: Existing comparison, native-name, prefix-summary, and mutation contracts
---

# Current recommendation — smallest useful separation

**Production implementation is paused.** The user questioned the larger scope
and specifically did not want bounded blocking I/O disguised as async. No
production code, manifest, test, package, or runtime dependency has changed.
The original plan below is preserved as a proposal, not accepted direction.

Prefer **two library crates**, keeping the current root package/layout:

```text
diff-dirs (existing root)
  model + pure engine module
  shared std filesystem inspector + existing serial driver

rayon/ (diff-dirs-rayon)
  depends on diff-dirs + Rayon
  borrowed caller pool + completion-driven joined directory jobs
```

This separates runtime *dependencies* without making every conceptual module a
package. Move mutex/scope dispatch and pool-derived defaults to the Rayon crate;
extract the existing tested reducer/admission into a filesystem-free engine
module. Keep root validation, native metadata/readlink, and immediate filesystem
inspection shared in diff-dirs. Retain the proven serial implementation initially;
unifying it onto the frontier is a separate optional change, not needed to isolate
Rayon. No root move to `fs/`, virtual-workspace migration, or perf package is
necessary. Existing CLI stays serial; its manifest's unused parallel passthrough
can be deliberately migrated with the adapter change.

The necessary public sibling seam is a narrowly documented adapter-facing module:
opaque directory job IDs/paths, owned comparison result, bounded admission and
ordered reduction, acknowledgment/prune, and prefix-summary operations; plus the
shared blocking inspector. Core limits should be runtime-neutral (`Limits`),
with pool-derived policy a Rayon helper. Keep test clocks, worker IDs, and locks
outside the pure engine. This is meaningful sans-I/O at the **module** level;
it does not claim that the entire diff-dirs crate contains no filesystem code.
Avoid general executor traits or exposing arena internals.

### Why not separate core/fs now?

A third pure-core crate earns its boundary when there is an actual non-std
filesystem backend, simulated/in-memory consumer needing core alone, or a firm
requirement to enforce absence of filesystem implementation via the dependency
graph. It buys stronger packaging enforcement and independent reuse; it does
not improve traversal throughput, remove the public adapter seam, or make std
metadata portable to a different runtime. Today a pure module with simulated
completion tests obtains the useful algorithmic separation with less API and
manifest churn. A module can move to a core crate later without replacing its
protocol. Neither option entails `no_std`: native paths and SystemTime remain
part of the existing model.

### What would Tokio actually add?

The previously proposed Tokio adapter is **not a native asynchronous filesystem
backend**. It runs the same blocking std calls, adding runtime blocking tasks,
join ownership, wakeups, per-directory submission, and cancellation state.
Its distinct benefits would be async per-record consumption, caller-runtime
integration, and parallel directory jobs without a Rayon dependency. None is
an established need here, and none makes filesystem syscalls cancellable.

For a caller needing only a final report, wrapping an entire serial or Rayon
comparison in one application-owned `spawn_blocking` is the smaller integration.
One blocking caller can drive the existing borrowed-pool traversal; there is no
need for a blocking task per directory. It blocks no event-loop worker, but
cancelling its async waiter does not stop the operation. A caller needing record
streaming must additionally own a worker/task and a bounded decision handshake,
or move an owned walk in/out of per-epoch blocking tasks. Those are real lifecycle
requirements, not a reason to assume that a full Tokio library is already needed.

A Tokio adapter's extra cancellation bookkeeping can preserve a walk across
cancelled `next` futures and provide async drain, but started blocking jobs still
run. Joining before every successful yield adds wakeup/drain costs; dropping the
whole walk cannot promise synchronous quiescence without blocking the runtime.
It relocates these responsibilities from application to library, rather than
solving blocking I/O. A true native-async backend would be a different,
platform-sensitive project requiring concrete syscall/library evidence, not
relabeling `spawn_blocking` or `tokio::fs` as native async.

**Proportionate next choice:** authorize only the pure module + separate Rayon
crate if dependency isolation is wanted now. Preserve serial behavior, existing
frontier tests and performance controls; verify no Rayon in diff-dirs's dependency
tree and parity/performance after extraction. Defer Tokio and extra package
boundaries until a concrete consumer justifies them. An even smaller first
checkpoint is extracting the pure module within the current package before
deciding whether to move Rayon to a sibling crate. No option is implemented by
this assessment.

# Preserved initial larger proposal — not selected

## Package graph originally proposed

Use a virtual workspace, not a root facade:

```text
core/  diff-dirs-core   model, immediate comparison, pure frontier/admission
fs/    diff-dirs        shared blocking inspector + synchronous pull driver -> core
rayon/ diff-dirs-rayon  borrowed-pool joined driver -> core + fs + Rayon
tokio/ diff-dirs-tokio  caller-runtime blocking-task driver -> core + fs + Tokio
cli/   diff-dirs-cli    rendering -> fs
perf/  diff-dirs-perf   cross-runtime controls and measurement -> all adapters
```

The `diff_dirs::walk` consumer name survives. Runtime selection moves from root
features/options to explicit adapter crates. Serial consumers pull neither
runtime. No reconciliation, copy/sync, jj-waltz integration, live mutation,
default retuning, generic executor traits, or internally created pools/runtimes.

# Extracted protocol

Retain the arena, child deques, DFS reducer, J admission bound, K speculative
retention, and separate demand reservation. Expose opaque job IDs with relative
paths, `begin_epoch`, `pump`, `start`, `complete`, `finish_epoch`, `prune_here`,
and `into_summary`. The driver returns every reserved job exactly once, drains
before finishing an epoch, and never starts a new epoch until delivering its
pending output. Acknowledgment is epoch entry, not speculative completion.
Core owns no filesystem calls, locks, tasks, pool access, or wall-clock timing.
Runtime-neutral count diagnostics may remain in core; clocks/worker IDs belong
to adapters. Immediate classification and native-name ordering belong in core;
filesystem metadata and root checks belong only in fs's adapter-facing inspector.

Successful next calls across all drivers deliver only after draining all reserved
inspections. Divergent directories remain sealed until next-call acknowledgment;
pruning permits zero descendant inspections. Public summaries include only the
ordered committed prefix. J/K bound counts, not wide-directory bytes.

# Tokio ownership and cancellation

`async next(&mut self)` uses whole-directory `spawn_blocking` jobs on the current
caller runtime. The walk owns its task set and epoch state, not the temporary
future. Dropping a pending next future loses neither completions nor reserved
capacity. Re-entry continues the same epoch without acknowledging an undelivered
record. Output is removed only after draining, in the same poll that returns it.
No await follows output removal.

An async `close(self)` stops admission, drains already-reserved jobs, and returns
the committed-prefix summary without acknowledging or delivering pending output.
Dropping the adapter never blocks the event loop: started blocking closures can
continue read-only after drop, bounded by J and K+1. Quiescence after whole-walk
drop is explicitly NOT guaranteed. A stalled filesystem can stall next/close.

Blocking task panic/join failure becomes an explicit terminal driver failure
with preserved source; drain remaining work and never leave an outstanding job
unaccounted. Ordinary filesystem errors retain ordered DFS selection. No callback
or runtime-owned user work executes inside the reducer.

# Migration and verification

Move existing tests with their owners; retain pure simulated completion tests
in core and runtime gates in adapters. Preserve historical independent reports;
update current README/index and harness commands rather than rewriting evidence.
Retain the old per-frame control only as a benchmark facility outside core,
using the shared filesystem inspection implementation.

Check package dependency trees, current/all-feature nextest, doctests, formatting,
clippy, and actual Rust 1.88. Test real job overlap, seals/errors/prefixes, caller
pool/runtime ownership, current-thread responsiveness, cancellation/re-entry,
explicit close, drop, and task failure. Compare same-build release serial/Rayon/
Tokio on owned small directories and read-only supplied node_modules/whole roots,
five warm interleaved observations with P/J=4 and K=256. Initialization stays
outside timing; complete record and summary equality is checked outside timing.

This extends the [accepted concurrency design](/design/concurrency/design0.gpt6am.md)
without replacing its scheduling algorithm. The [previous measurements](/design/performance/parallel1.sol61x.md)
remain the extraction-regression reference, not promised Tokio performance.
