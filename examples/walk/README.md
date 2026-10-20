---
type: ExampleGuide
title: Component-built walking and process continuation
description: A bounded Unix byte-copy and generated-manifest specimen with explicit component construction, ordered requests, two budgets and optional Snapshot transport.
resource: /examples/walk/README.md
status: implemented-bounded-specimen
generated: { by: "model:openai/gpt-6.1-sol#xhigh", at: 2026-10-20 }
sources:
  - resource: "opencode://ses_ea7b7c0a4ffe1KVm1nvPn7svzl"
    title: Implemented and tested component construction, actual walking, retry/skip and fresh-process continuation
  - resource: /design/execution/builder0.gpt6am.md
    revision: 91d2f7e3
    title: Reviewed component builder and concrete implementation handoff
  - resource: /design/execution/contracts0.gpt61sxh.md#4-next-commission-a-component-builder-and-real-walking-specimen
    revision: 5d6c003a
    title: Fixed demo semantics and four numeric process gates
  - resource: /design/execution/model0.gpt61sxh.md
    revision: 3113dfd2
    title: Ordered child ownership and failure semantics
  - resource: /examples/cursor/src/codec.rs
    title: Existing actual Snapshot and complete-file publication precedent, left unchanged
extensions:
  ticket: diffdir-tree-execution-walk-demo
verification:
  method: Example-local nextest, clippy, format and dependency checks; see implementation report for exact commands
  result: The tiny Unix fixture gates pass; no production sync, native JJ, crash recovery or performance claim
---

# Component-built walking

This standalone nested crate demonstrates **real byte copying, effectful directory
finishes and continuation through four processes**. It does not modify core, sync
or the original [assessment-only cursor](/examples/cursor/README.md).

## Run the four gates

From the repository root, with the sibling `kopacetic` and `facet-format-path`
checkouts used by the existing cursor example:

```sh
cargo build --offline --manifest-path examples/walk/Cargo.toml --features cli
fixture=$(mktemp -d /tmp/opencode/walk-guide.XXXXXX)
mkdir -p "$fixture/source/d" "$fixture/target" "$fixture/reports"
printf one > "$fixture/source/d/a"
printf two > "$fixture/source/d/b"

examples/walk/target/debug/walk start \
  --source "$fixture/source" --target "$fixture/target" --reports "$fixture/reports" \
  --label A --cursor "$fixture/cursor.bin" --assessments 2 --effects 2
examples/walk/target/debug/walk resume --cursor "$fixture/cursor.bin" --assessments 0 --effects 1
examples/walk/target/debug/walk resume --cursor "$fixture/cursor.bin" --assessments 0 --effects 1
examples/walk/target/debug/walk resume --cursor "$fixture/cursor.bin" --assessments 0 --effects 1
cat "$fixture/reports/dir-64.txt" "$fixture/reports/root.txt"
```

| Call | Accepted boundary | Saved ordinal |
| --- | --- | --- |
| Start, 2 assessments / 2 effects | Ensure `d`, settle `a`; `b` pending. `d` leaves 1, root leaves 0. | 1 |
| Resume, 0 / 1 | Settle `b`; `d` finish pending. `d` leaves 2, root leaves 0. | 1 |
| Resume, 0 / 1 | Install `dir-64.txt`, retire/fold `d`; root finish pending, root leaves 2. | 2 |
| Resume, 0 / 1 | Install `root.txt`; terminal leaves 2, incomplete false. | 3 |

Each command emits JSON **after** publishing its separate cursor file. JSON is
diagnostic output, not the continuation codec. A blocked attempt publishes its
state and exits 2; use `resume --retry` or `resume --skip` with explicit limits.
Other command/publication errors retain their error cause and exit unsuccessfully.
Do not infer rollback, recoverability or an effects ledger from a cursor file.

## Library builder and manual stepping

Default features are empty: no codec, Facet, Clap or Serde is needed. The runnable
[builder example](examples/builder.rs) uses direct saved-parts reconstruction and
the same request/ack loop as `drive`:

```sh
mkdir "$fixture/library-target" "$fixture/library-reports"
cargo run --offline --manifest-path examples/walk/Cargo.toml --no-default-features \
  --example builder -- "$fixture/source" "$fixture/library-target" "$fixture/library-reports"
```

The construction seam is a real Bon builder:

```rust
let mut run = cursor()
    .parts(StartParts::Fresh(Bindings::new(source, target, reports)?))
    .inspector(FullInspector)
    .executor(ByteExecutor)
    .program(DesiredCopy::v1())
    .manifest(Manifest::builder().label("A".to_owned()).build())
    .build()?;
run.bind_filesystem()?;
let progress = run.drive(Limits { assessments: 2, effects: 2 })?;
let saved = run.into_saved()?;
let manifest = Manifest::from_config(saved.manifest_config().clone())?;
// Rebuild with .parts(StartParts::Restored(saved)), new inspector/executor,
// DesiredCopy::v1() and this manifest; explicitly bind before dispatching I/O.
```

`Inspector` and `Executor` are supplied executable objects, not IDs selecting
hidden defaults. Tests provide a counting inspector and a failing executor,
restore to fresh wrappers, and prove no ancestor relisting. `DesiredCopy` and
`Manifest` are deliberately concrete. Manifest config/version/label and its
mutable accepted-finish ordinal are saved separately from rendering code.
Changing the label on restoration refuses; changing it on a **new run** is valid.
Custom inspector/executor implementations must obey the full-inspection/three-work
semantics and carry no additional behaviorally necessary cross-pause state; mutable
diagnostic counters and fault injection are invocation-local, not new saved owners.

`next_request()` performs no I/O and returns an owned `Inspect`, `Effect` or
`Done`. Repeated reads keep the pending ticket. `dispatch(&request)` calls the
supplied component; `acknowledge(ack)` checks identity/kind and accepts the result.
Hosts may supply external acknowledgments, which are trusted execution evidence.
Wrong, duplicate, consumed and blocked acknowledgments do not change the state.
An issued but unacknowledged saved request remains unresolved on restoration.

## Semantics and counters

- One DFS driver owns every recursive edge: **before → assess → children → finish**.
  Active children remain at their parent's front until closure; folding occurs
  once in this advancing lineage. There is no recursive `apply_plan` child step.
- Each attempted immediate inspection consumes one assessment, including errors
  and empty/clean batches. Full facts come from public sequential inspection.
  A narrow pure visitor preserves target-only slots, including special kinds
  which `NoVisitor` would otherwise reject. No source-present proposal is steered;
  pure lowering validates the entire batch before admitting its children.
- Each resource-closed ensure/file/finish attempt consumes one effect, including
  failure or an exact-byte satisfaction no-op. Both limits allow zero. Stop at the
  next exhausted class; do not jump past it. `0/0` drains only pure transitions.
- `Progress` separates attempted assessments/effects, accepted container/leaf/
  finish goals, accepted no-write goals, and invocation-local observed target
  **file installations / created directories**. These are not staging syscall
  counts. Failed after-visible-effect attempts report writes but no accepted goal.
- A failed unit blocks the unchanged goal. Retry issues a fresh ticket and checks
  current satisfaction; an already-installed file/report need not be rewritten.
  Skip abandons the blocked leaf, before/assessment subtree, or finish, marks all
  active ancestors incomplete, and leaves visible artifacts alone. Partial finishes
  are permitted. Skipped finishes consume no ordinal; ordinal reuse is possible.
- Reports contain accepted settled-leaf count, incomplete flag, frozen label and
  ordinal, **not a fresh inventory or proof of source/target equality**. Supported
  physical difference counts and retained boundaries remain separate aggregates.

## Boundaries

Unix native coordinates are byte vectors, never lossy display strings. Library
roots must be absolute and lexically normalized; CLI normalizes against cwd.
Construction, restored validation and codec decoding are data-only. Explicit
binding checks existing real, resolved disjoint roots without listing descendants;
initial binding checks are outside numeric budgets. Active attempts recheck relevant
routes/kinds. Terminal binding/drive do no tree I/O, even after removing the roots.

Source-present regular files and directories are the domain. Retain target extras
without descending their interiors; refuse source links/specials, admitted target
links/specials and file/directory collisions. Exact regular-file bytes determine
satisfaction, even if core's size/mtime comparison was equal. Desired attributes
are unspecified: no mode/mtime preservation or chmod adaptation, CoW, native JJ,
type replacement or deletion. Directories use ordinary creation/umask behavior.

Copied/generated files use closed **same-parent private staging then rename**;
failed preparation/publication explicitly attempts and reports owned cleanup.
Staging files use tempfile's ordinary private-file permissions. There is no
durability transaction or crash guarantee. Callers own good stable source and
nonhostile topology/concurrency; route checks are not a race-resistant sandbox.

Reports occupy a separate exclusively reserved root: `root.txt` or
`dir-<lowercase hex of native relative bytes>.txt`. The mapping is injective and
all reports are direct children. Directory-relative bytes are capped at 123 so
report basenames fit the **255-byte demo cap**; reject before that entity's effects.
This is not a portable `NAME_MAX` assertion; smaller filesystems can fail normally.

Feature `codec` uses actual Kopacetic Snapshot schema `diff-dirs.walk-example`,
revision 1, identity `WalkingCursor` / `example`. `cli` adds this codec plus
Clap/Serde JSON output. Archives are capped at 8 MiB before decode/after encode;
projection/reconstruction allocations, listing width and individual file size
are not bounded by that limit. Cursor publication is outside all three roots and
separate from effects/stdout. Old/forked snapshots can repeat work; there is no
cross-process exactly-once, history, recovery, locking or cleanup-reconciliation
promise. Optional local path dependencies remain example-only.

Exact checks, deviations from proposed spellings, and limitations are in the
[implementation report](/design/execution/walk0.gpt61sxh.md).
