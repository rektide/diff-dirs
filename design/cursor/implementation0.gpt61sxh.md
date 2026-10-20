---
type: ImplementationReport
title: Bounded cursor example implementation and verification
description: Delivered an isolated assessment-only Rust crate using public inspection and actual kopacetic snapshots, with process-to-process continuation and generated logical summary bytes.
resource: /design/cursor/implementation0.gpt61sxh.md
status: implemented
generated: { by: "model:openai/gpt-6.1-sol#xhigh", at: 2026-10-20 }
sources:
  - resource: "opencode://ses_ea8162bcbffe20eNrM60NJtgVS"
    title: Prepared API integration, implemented the reviewed cursor design, corrected immediate incomplete ancestry, and verified library and real CLI-process behavior
  - resource: /design/cursor/cursor0.gpt6am.md
    title: Parent-reviewed implementation handoff
  - resource: /examples/cursor/src/cursor.rs
    title: Implemented DTO, native coordinates, immediate assessment driver and virtual summary generation
  - resource: /examples/cursor/src/codec.rs
    title: Implemented actual Snapshot codec, semantic validation and bounded file transport
  - resource: /examples/cursor/tests/continuation.rs
    title: Verified state-machine, codec, failure and skip properties
  - resource: /examples/cursor/tests/process.rs
    title: Verified separate executable invocations and file transport
  - resource: /design/execution/state0.gpt6a.md#21-library-state-ownership-ledger
    title: Existing ownership and donor-state account, read through the report-state distinctions
  - resource: "file:///home/rektide/src/kopacetic/crates/kopacetic-rkyv/src/lib.rs"
    title: Snapshot API used without modifying the kopacetic checkout
verification:
  method: Commands and small fixture executions recorded below
  result: 12 focused tests passed, Clippy warnings denied passed, formatting passed, default core graph remained dependency-free
---

# Delivered slice

The standalone [example crate](/examples/cursor/README.md) implements the reviewed
[bounded cursor design](/design/cursor/cursor0.gpt6am.md). All implementation and
dependency changes are inside `examples/cursor`. No core source, root manifest,
root lockfile, sync code or kopacetic source was changed. Cargo accepts its own
nested workspace without a root exclusion.

The driver calls public `inspect_directory`, with pure `NoVisitor`. N counts
attempted immediate directory assessments, including injected failures, binding
errors and actual inspector errors. It saves actual DFS frames, selected pending
tips and accumulators instead of replaying `PlanWalk`. Both one-sided interiors
and the sole directory side of collisions are inspected. Completion is postorder;
zero-assessment completion transitions drain even at budget exhaustion.

The persisted `directory-summary/v1` descriptor dispatches to a real compiled
handler parameterized by `label`. Each completion emits a logical `summary.txt`
and its manufactured bytes. No summary or mirror operation is installed into
the source or target. The cursor keeps the descriptor and necessary accumulators,
not output history or previously generated bodies. A final aggregate is retained
after the root frame is removed, so terminal cursors remain useful and require
no tree I/O.

This extends the [execution-state ledger](../execution/state0.gpt6a.md#21-library-state-ownership-ledger)
with a caller-owned durable representation of active traversal frames. It does
not archive `DirectoryPlan` metadata or turn the live donor into a snapshot. Its
completion counter measures generated assessment summaries, not the mutation
engine's `Report.completed` effects described in that account.

## Codec, invariants and small adaptations

- Uses actual `kopacetic_rkyv::Snapshot::from_value/from_bytes/reconstruct` with
  exact application schema and fixed expected identity. No second codec.
- Facet-only DTOs use explicit Unix native bytes; paths are not `PathBuf` opaque
  projections and never pass through lossy UTF-8. JSON byte arrays are output
  presentation, not the snapshot format.
- Additional semantic validation covers platform, normalized safe relative
  paths, absolute disjoint roots, coincident coordinates, child adjacency/order,
  active-frame consistency, phases, indices, descriptor and basic counters.
  Counter arithmetic errors are reported rather than overflowing/panicking.
- Bounds archive input reads and output bytes at 8 MiB. This is not a total
  allocation/work bound. Snapshot projection uses generic typed path rows and
  owned reconstruction, not a native zero-copy cursor archive.
- Complete-file cursor replacement precedes JSONL delivery, and input/output may
  name the same file. This order reduces avoidable output on publication failure
  but intentionally provides no joint transaction, receipt or replay log.
- Skip marks every active ancestor incomplete immediately. Waiting for postorder
  folding would eventually yield the right final answer but leave stale coverage
  flags in an intermediate saved position; this was corrected separately.
- Invocation events are collected in memory, not streamed one at a time. They
  are not retained in the continuation; N is not a byte/event-count bound.

## Actual verification

Run from `/home/rektide/src/diff-dirs` on 2026-10-20:

| Command | Result |
| --- | --- |
| `cargo check --manifest-path examples/cursor/Cargo.toml --all-targets` | Passed initial build. |
| `cargo nextest run --manifest-path examples/cursor/Cargo.toml` | Final run: **12 passed, 0 skipped**, across four test binaries. |
| `cargo clippy --manifest-path examples/cursor/Cargo.toml --all-targets -- -D warnings` | Passed after the correction. |
| `cargo fmt --manifest-path examples/cursor/Cargo.toml --check` | Passed after the correction. |
| `cargo tree -p diff-dirs --edges normal` | Exactly `diff-dirs v0.1.0 (/home/rektide/src/diff-dirs)`; no Facet/kopacetic/rkyv dependencies in the default core. |

Tests cover:

1. N=1 snapshots versus uninterrupted traversal: identical assessment coordinates,
   completion order, sequence numbers, generated bytes, final totals and cursor.
2. Ten separate N=1 executable invocations versus one uninterrupted mixed-tree
   process. Mixed fixture includes clean/empty directories, source-only and
   target-only nested interiors, equal/changed leaves, a literal symlink change,
   a type collision and a non-UTF8 native directory name.
3. Assessed ancestor retention, mutation adding a new root sibling after pause
   (not relisted), multi-frame snapshot roundtrips, and completion chains after
   the final real assessment.
4. Blocked injection, retry, skipped subtree, deep ancestry before completion,
   blocked-root skip, real missing/replaced directory binding errors and an actual
   unsupported Unix socket inspector failure without partial batch counts.
5. Terminal resume after both roots are removed; no assessment/event replay.
6. Backend magic, truncation/corruption, wrong schema/identity, hostile paths,
   child index/active-frame mismatch, unknown descriptor/platform and checked
   arithmetic on malicious but well-formed reflected data.
7. Zero-unit command rejection, skip-without-block, corrupt/oversized input,
   same-file input/output and failed output publication without fabricated JSONL
   success or overwriting the old cursor on decode errors.

No broad core/sync suite, benchmark campaign, JJ subprocess or reflink operation
was run. This is isolated example verification, not a new performance claim or
universal correctness proof.

### Tiny runnable observation

Created `/tmp/opencode/cursor-demo-s2xc3v1h` with paired empty `branch/`, a
source-only `new.txt` and a target-only `old.txt`. Invoked
`examples/cursor/target/debug/cursor` as two real processes:

```sh
examples/cursor/target/debug/cursor start \
  --source /tmp/opencode/cursor-demo-s2xc3v1h/source \
  --target /tmp/opencode/cursor-demo-s2xc3v1h/target \
  --units 1 --label tiny-fixture \
  --cursor-out /tmp/opencode/cursor-demo-s2xc3v1h/state.rkyv

examples/cursor/target/debug/cursor resume \
  --cursor-in /tmp/opencode/cursor-demo-s2xc3v1h/state.rkyv \
  --units 1 --cursor-out /tmp/opencode/cursor-demo-s2xc3v1h/state.rkyv
```

First process: attempted 1, completed 0, done false; paused archive **3,618 bytes**.
Second: attempted 1, completed 2, done true, no block. Two logical summary files
were generated. Final observed totals: source-only 1, target-only 1, changed 0,
equal 1, incomplete false. Absolute root spelling and cursor shape affect archive
size; no speed/size comparison was performed. `start.jsonl` and `resume.jsonl`
were left alongside that scratch fixture. The maintained README supplies a
fresh-fixture version of these commands.

## Checkpoints and boundaries

- `442a00dc`: initial isolated crate, CLI, codec and ten passing tests.
- `bbd7247a`: immediate incomplete ancestry correction, plus deep-skip and real
  unsupported-entry failure coverage; final twelve-test suite passes.

This demonstrates a saved continuation, not effects history. Concurrent changes,
donor validity and routing-ancestor stability remain caller responsibilities.
Outputs can be lost after cursor publication or repeated from an older cursor;
there is no rollback, recovery guarantee, materialization engine or authentication.
No general action-dispatch/mapping framework, generated filesystem writes,
cross-platform native-path codec or JW policy was added. The parent owns corpus
integration links and the broader action/generation design.
