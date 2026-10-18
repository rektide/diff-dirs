---
type: ImplementationDesign
title: Runtime-neutral frontier and separate blocking runtime drivers
description: Package graph, extracted job protocol, cancellation boundary, and migration for serial, Rayon, and Tokio directory comparison.
resource: /design/implementation/refactor0.sol61h.md
status: implementation-in-progress
generated: { by: "model:openai/gpt-6.1-sol#high", at: 2026-10-18 }
sources:
  - resource: opencode://ses_eb335933bffeUeaPkxqOtKDOWi
    title: Selected runtime-neutral package boundaries and cancellation protocol; implementing and verifying the extraction and adapters
  - resource: /design/concurrency/design0.gpt6am.md
    title: Existing ordered reducer, demand reserve, seals, and completion-driven admission design
  - resource: /design/implementation/implementation1.sol61x.md
    title: Tested generational arena and joined Rayon implementation being extracted
  - resource: /design/performance/parallel1.sol61x.md
    title: Baseline single-root performance, explicit P4/J4/K256, conservative defaults, and width costs
  - resource: /README.md
    title: Existing comparison, native-name, prefix-summary, and mutation contracts
---

# Selected package graph

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
