---
type: DesignDecision
title: Generic comparison and optional synchronization contract
description: Parent decisions for the generic library, including comparison direction, native paths, partial summaries, and separately packaged synchronization; jj-waltz is the first consumer.
resource: /design/fit/decisions0.gpt6a.md
status: implementation-direction
generated: { by: "model:openai/gpt-6-astra", at: 2026-10-18 }
sources:
  - resource: "opencode://ses_eb39c93e9ffeNju5a7XEsB3wGY"
    title: Coordinated G research and Sol implementation, then commissioned and selected Astra medium's bounded directory-frontier redesign after weak live-tree scaling
  - resource: /README.md
    title: Initial tree-comparison specification and conflicting application examples
  - resource: /design/fit/fit0.glm53m.md
    title: G assessment of dependencies, native filesystem requirements, and contract ambiguities
  - resource: /design/performance/parallel0.sol61x.md
    title: Same-build live measurements and the limitation of current-directory-only parallelism
  - resource: /design/concurrency/design0.gpt6am.md
    title: Astra medium proposal for bounded directory jobs and an ordered commitment cursor
  - resource: "file:///home/rektide/src/jj-waltz/Cargo.toml"
    title: Rust 1.88, edition 2024, dependency set, and distribution platforms
  - resource: "file:///home/rektide/src/jj-waltz/src/reseed/engine.rs"
    title: Inspected execute body around lines 1440–1515, including both payload removal points
---

# Implementation contract

**Later toolchain direction:** the user removed the initial Rust 1.88 baseline.
Current work uses the environment's configured, unpinned toolchain (Rust 1.101
nightly is permitted). The initial 1.88 choices below record build history,
not an ongoing compatibility requirement; active commands live in the root
[development instructions](../../README.md#development-and-evidence).

The user's outcome is a fast generic library, first embedded in jj-waltz reseed.
[G's assessment](fit0.glm53m.md) precedes the Sol xhigh build. These are parent
implementation decisions resolving the initial specification's ambiguities;
the maintained [README](../../README.md) must reflect them. They are not a
claim of user acceptance or an end-to-end reseed performance result.

The user subsequently clarified that the I/O investigation should consider
general libraries such as Tokio, and authorized copy/sync procedures provided
they remain separate from comparison. The selected packaging is a `sync/`
crate depending on the core, with no reverse dependency. G is investigating
general filesystem backends before that component is implemented. The core
remains usable by callers with their own apply loops or reporting policy.

## Consumer-facing model

**Direction follows the application examples:** original-only entries are
`added`, target-only entries are `removed`. Applying the record makes the
target resemble the original. This deliberately corrects the initial Model
paragraph, whose labels contradicted both application examples.

Names are `OsString`, not `String`; relative directories are `PathBuf`.
Arbitrary native names must survive the library boundary. The CLI must encode
unusual names explicitly rather than silently replace bytes. Sorting uses
native encoded bytes, deterministic on the host platform.

Regular files compare type, size, and exact modification time; symlinks compare
link contents without following them. Mode-only changes remain outside the
initial comparison contract. Use `SystemTime` directly, including pre-epoch
values, instead of introducing an epoch-conversion restriction. A regular
file/symlink transition is a changed leaf; a directory/leaf transition is a
paired removal and addition. Unsupported special entries produce a named
error. One-sided directories remain wholesale units, without inspecting their
descendants.

Both roots must already be real directories. Missing roots, symlink roots, and
non-directory roots are errors. An embedder handles absent roots and selected
leaf roots through its existing whole-root operations. Existence is a caller
dispatch condition, not an assumption about every reseed target.

## Traversal and summary

Use the specified pull iterator, paired depth-first walk, and pre-order
directory records. Materialize and compare a directory's immediate children
before yielding its record; this permits the caller to apply those immediate
changes before descent. The example uses `while let Some(item) = walk.next()`
so the caller can invoke `prune_here()` without a conflicting loop borrow.

`prune_here()` skips pending descent from the most recently yielded record.
It is a caller policy hook, not an automatic divergence heuristic. An error
terminates the iterator, which thereafter yields `None`.

Summary roots are **topmost directly divergent directories**: emitted record
directories without an emitted ancestor. Otherwise every divergence would
collapse trivially to the root pair. Counts describe examined immediate
differences; a one-sided subtree counts once. Partial iteration, caller
pruning, and errors must be visible in the summary, so incomplete inspection
cannot assert a complete clean comparison. Consuming the summary must not
silently perform more filesystem work.

Memory includes sorted pending child entries on the active descent path and
the accumulated summary. Neither a depth-only bound nor an unconditional
sublinear-to-tree-size claim is accurate for arbitrarily wide trees.

## Dependencies and ownership

Use a zero-runtime-dependency default core, Rust 1.88 and edition 2024. The root package
is the library; a separate `cli/` workspace member owns CLI dependencies. This
keeps embedding inexpensive without duplicating a general traversal engine or
pulling terminal/serialization dependencies into jj-waltz.

The core compares only. A separate optional sync crate may provide copy and
reconciliation procedures without introducing jj concepts or a mandatory async
runtime. Its backend choice follows the additional I/O assessment; the initial
std-only comparison implementation does not establish that all possible sync
backends must be std-only. Applications own root selection and their policy
requirements; jj-waltz additionally owns its workspace confinement and
quiescence checks. The core is not a filesystem security boundary under
concurrent mutation.

### Initial shared Rayon execution

The user subsequently requested Rayon and investigation of sharing its runtime.
The implementation direction is an optional `parallel` feature and an explicit
execution choice, preserving a dependency-free default and allowing sequential
and parallel runs in the same benchmark binary. Parallel operations should use
Rayon's current pool: an embedding caller can run the **entire consumption
loop** inside `ThreadPool::install`; otherwise Rayon supplies its shared global
pool. The library must not configure the process-global pool or construct a
private pool for every walk. G is verifying the concrete integration recipe.

Only parallelize work for the current directory comparison, and join it before
returning from `next()`. Speculative subtree work across a yield would break
the caller's ability to replace/prune that subtree. Sort and retain ordered
results so failure selection, as well as successful records, stays
deterministic. The right batch threshold and thread count are measurement
questions: compare sequential and shared-pool execution on fixtures and the
user-selected live source/destination trees. Tokio's use of blocking filesystem
operations establishes neither that parallelism is useless nor that Rayon
necessarily improves this workload.

### Selected redesign: directory frontier and ordered commitment

The initial per-frame implementation is now a measured baseline. On the supplied
live `node_modules`, its best retained median improved roughly 690 to 636 ms;
the many small directory comparisons still ran serially. The user requested
Astra medium to redesign traversal after Sol completed that work.

The parent selects [Astra's directory-frontier proposal](../concurrency/design0.gpt6am.md)
for the next implementation. This supersedes the initial restriction to one
current directory pair; it does not claim the new scheduler has been verified
or accepted by the human. The selected properties are:

- Whole directory-pair jobs execute concurrently on a borrowed caller-owned
  Rayon pool. Completion supplies further jobs dynamically; workers do not
  permanently own subtrees.
- A separate DFS cursor commits records, errors, and summary effects in the
  established order. Speculatively inspected siblings do not enter an early
  summary. The public wording becomes **ordered committed prefix**.
- A differing directory seals its descendants until the caller advances past
  its record. Pruning therefore discards unadmitted children rather than racing
  already-running descendant work.
- One global per-walk limit bounds queued/running directory jobs, another
  bounds retained speculative nodes, and an additional demand slot ensures
  progress when later siblings fill the speculative budget. Pool workers,
  jobs, retained results, and width-sensitive memory are distinct bounds.
- Every `next()` epoch joins before returning. No pool construction, global
  pool configuration, blocked receiver worker, or background task lifecycle is
  introduced. Multiple walks share workers but retain separate memory budgets.
- An explicit `walk_in_pool` / borrowed-pool walker makes pool ownership clear;
  the existing ambient entry point remains usable. The old per-frame path is
  retained as a measurement control while the new path is verified.

The mutation contract must also state that applying one record cannot modify
other pending paths through hard-link or mount aliases. Unlink-and-create
replacement fits that contract. The separate sync crate remains the subsequent
implementation slice; diff workers have joined before the applier runs.

Verification requires genuine concurrent small-directory jobs, adversarial
demand-budget and error-order cases, prefix parity, prune/drop quiescence, and
repeatable read-only measurements of the same live trees. Architecture alone
does not establish the expected speedup.

The parent found **two payload deletion points** in the inspected reseed
execution: the cleanup before tracked materialization and the final payload
remove/clone loop. Replacing only the latter cannot retain the old payload for
a delta comparison. G is extending its assessment with the conditions for
preserving payload across the earlier step. Integration is a follow-on change
in the consumer, not a library benchmark result.

## Evidence required from the build

Tests must exercise the real embedding contract: deterministic output,
directory/leaf transitions, native names, no-follow links, pruning and partial
summaries, terminal errors, and applying records in-loop followed by a clean
second comparison. Compilation, clippy, formatting, CLI behavior, and doctests
complete the implementation checks.

Release benchmarks should separate fixture construction from timed walks,
repeat warm runs, and record filesystem, entry counts, and scenarios. Synthetic
30k and roughly 192k fixtures test traversal scaling; they do not reproduce an
8 GB dependency installation merely by matching its entry count. The inherited
README timings lack a recovered method here and must remain distinguishable
from new measurements. No end-to-end jj-waltz speedup is established until its
own integration and application costs are measured.
