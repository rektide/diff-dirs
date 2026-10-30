---
type: ExampleGuide
title: Production reconciliation across fresh processes
description: Runnable source-to-many synchronization with application-owned versioned JSON, native path spelling, bounded calls and production-kernel effects.
resource: /examples/reconcile/README.md
status: implemented
generated: { by: "model:openai/gpt-6.1-sol#xhigh", at: 2026-10-28 }
sources:
  - resource: "opencode://ses_e744de129ffeJFNaoEhNajEXMV"
    title: Adopts provider0.8 and version4 postcheck-bound continuation with explicit codec vocabulary
  - resource: opencode://ses_e7a2771c3ffez3sv1oPgD5gPH4
    title: Promotes contextual selected semantics and resumed-effect evidence, then records the explicit schema3 retention-accounting correction
  - resource: /design/architecture/execution0.sol61x.md
    title: Provider 0.3 adoption, common semantic validation and 15 passing process cases
  - resource: /design/architecture/execution1.sol61x.md
    title: Generic v2 transport, fresh-process contextual comparison and 18 passing reconcile cases
  - resource: /design/architecture/execution2.sol61x.md
    title: Actual contextual A-accepted/B-unstarted process continuation and 20 passing reconcile cases
  - resource: /design/architecture/execution3.sol61x.md
    title: Verified schema3 transport and separate deviation/retention totals
  - resource: "opencode://ses_e7be6b24fffeDISpWdV1xNbqhD"
    title: Adopts exact 0.2 library pins and locks and extends the example's action vocabulary for source-only conditional routing
  - resource: "opencode://ses_e7e6c1941ffeFqyy1fEbFTKVmf"
    title: Reviews the delivered process consumer, fixes explicit-subset failure publication and the complete-file transport limit, and verifies wide diagnostic counters
  - resource: "opencode://ses_e7e8964e7ffe0nSda1UFEvX5PL"
    title: Implemented production sync continuation and this independent transport/CLI consumer, then verified partial lane heads and accepted decisions through new processes
  - resource: /transfer/sync/README.md
    title: Production cursor, application barrier, error accounting and opaque-operation contract
  - resource: /tree/compare/src/coordinated/mod.rs
    title: Comparison-owned traversal, lent policy inspection, controls and pure return settlement
  - resource: /examples/walk/src/codec.rs
    title: Earlier example precedent for separate cursor publication outside tree roots; its local codec dependency is not used here
---

# Production reconciliation across fresh processes

This runnable consumer uses [`SyncCursor` and `SyncRuntime`](/transfer/sync/README.md#coordinated-reconciliation-and-continuation),
not the older walking specimen's byte-copy executor. Every application uses sync's
actual staged copy/reflink, literal-link, removal, collision, metadata and permission
kernel. Comparison alone owns admitted paired-directory traversal.

The package is excluded from the root workspace: building it is explicit, and its
Serde/JSON/Clap transport dependencies do not enter the library's default dependency
graph. It needs no local codec project or universal continuation framework.

## Run

Source and both destinations must already be real directories, under stable,
nonhostile caller ownership. Destination roots must be disjoint from each other
and from the source. The cursor's existing parent must lie outside every tree.

From the repository root:

```sh
cargo run --manifest-path examples/reconcile/Cargo.toml -- start \
  --source /data/source --target A=/data/target-a --target B=/data/target-b \
  --copy-policy copy --cursor /data/checkpoints/reconcile.json \
  --groups 1 --applications 1

cargo run --manifest-path examples/reconcile/Cargo.toml -- resume \
  --cursor /data/checkpoints/reconcile.json --groups 2 --applications 3
```

The first command can stop with A's root application accepted, B's application
unstarted and both root traversals unfinished. The second is an independent
process: callbacks/resources are reconstructed, not restored from serialized code.

`--copy-policy` is required: `copy`, `reflink-required` or `reflink-or-copy`.
Required reflinks never silently fall back. `--groups` bounds new joined comparison
groups; `--applications` bounds application requests, including a refused resumed
refresh. They are not byte, memory, syscall or latency bounds. One application may
recursively copy/remove an entire one-sided subtree. Zero budgets start no new
work; ready pure returns can drain.

Optional explicit example policies on `start`:

```sh
# B preserves this name and subtree before child metadata acquisition.
--preserve B=a

# A refuses this actual observed slot, including inside an opaque copy.
--refuse A=opaque/bad
```

There are no built-in foreign-directory names or ignore defaults. Relative rule
paths, their native spelling and their executable interpretation are frozen in
the snapshot. Only one refusal path per target is supported by this example.

## Open controls and exact application boundaries

`resume` can use the restored current window:

```sh
# Apply exactly B's pending batch, publish, and retain the open control window.
cargo run --manifest-path examples/reconcile/Cargo.toml -- resume \
  --cursor /data/checkpoints/reconcile.json --groups 0 --applications 1 \
  --apply-only B

# Close ready returns without inspecting the next queued sibling.
cargo run --manifest-path examples/reconcile/Cargo.toml -- resume \
  --cursor /data/checkpoints/reconcile.json --groups 0 --applications 0
```

`--apply-only ID` is repeatable, requires `--groups 0` and sufficient application
budget, and deliberately does not settle the control window. It makes a saved
state before root closure directly observable. A shared refresh may fail other
selected peers before their turn; those peers are not dispatched again, and the
reported application count includes only attempted requests. Healthy batches left
unstarted after a refused request remain available on resume. `--prune ID` removes that lane's
current paired-child memberships without canceling its immediate application;
`--stop ID` abandons the lane's remaining obligations. Controls are explicit, not
automatic pruning or recovery.

Stdout is one JSON diagnostic object: invocation groups/application requests,
`Budget`/`Done`, diagnostic structural-root hook count, failures and per-lane
observed-work reports. Restored accepted root policy produces zero root hooks.
Exit status **0** means a successful pause or successful completion; **2** means
returned lane failures or terminal incomplete reconciliation; **1** means a command,
schema, structural-state, binding, publication or other global error. Healthy lanes
can complete even when another lane failed, so `Done` alone does not mean success.

## Transport and effect boundaries

[`transport.rs`](src/transport.rs) defines explicit application DTOs under
`schema = "diff-dirs.reconcile-example", version = 4`. Field names and closed enum
spellings belong to this example, not to runtime Rust layout. Unknown fields,
vocabulary, versions, incompatible policy descriptors and invalid saved state are
refused before execution. Actual error causes are returned live; saved terminal
markers are data, not executable error objects or an operation recovery log.

The 0.3 adoption explicitly stores comparison outcomes, source observation,
installation/removal authority, interior role/input and policy/context-schema/
comparator descriptors. Reconstruction uses inspection's common selected-work
validator. Version 1 refuses rather than receiving invented context or authority.
The 0.4 transport is generic over context, including initial bindings, active frames,
queued members and selected inputs/interiors. `Comparison<C>` supports comparison-only
checkpoints; `Snapshot<C>` supports contextual application data. Fresh-process tests
verify deep independent heads and pre-I/O interpretation refusal. The production
effectful CLI still uses unit context; additional production-runtime subprocess
tests now verify actual contextual A-accepted/B-unstarted execution, second-comparator
effects and rejected changed opaque residuals. Version3 additionally carries true
selected retention separately from action deviation and explicit opaque/necessary
additions. All version2 documents refuse: their old mixed aggregates cannot recover
the corrected meaning. No silent migration is provided.
Version4 additionally stores one postcheck (`none` or `mtime`) per lane; runtime
rebinding must match it. Version3 is refused rather than assigning an unstated
installation-verification policy. Comparison-only parts retain their own version.

The 0.2 library adoption adds `directory/create-if-needed/descend` to this
vocabulary. Older readers reject unknown spellings; it is not a serialization
of pending routing ancestors or a partial-copy checkpoint. Shape/descent legality
is still checked by the decoded production decision validator. The example's
stock preserve/refuse CLI policy does not itself select this new routing action.

[`NativeString`](src/native.rs) stores Unix bytes or Windows UTF-16 units. Roots,
basenames and rule paths never pass through lossy Unicode conversion. Foreign
platform units refuse reconstruction. Windows code is present but not runtime
verified; Linux is verified. The 32 MiB document limit includes the published
newline and bounds transport bytes, not total parsing/frontier memory.

On resume only unstarted current-group decisions are refreshed, together, through
shared admitted inspection. Every refreshed signature is checked before refreshed
effects. Accepted application/ancestor/root decisions are not replayed. A failed
opaque call is terminal for its lane; healthy lanes continue. There is no partial
copy cursor, retry interpreter, whole-root initialization or cleanup reconciliation.

Cursor publication uses a separate staged complete-file replacement outside the
trees. Its parent is resolved and checked before effects. Publication failure does
not roll back tree effects. Old/forked snapshots can repeat work; no crash recovery,
directory-fsync or exactly-once publication guarantee is made. Terminal resume
does not reconstruct filesystem-bound appliers or access removed tree roots.

## Verification

```sh
cargo nextest run --manifest-path examples/reconcile/Cargo.toml
cargo nextest run --manifest-path examples/reconcile/Cargo.toml --all-features
cargo clippy --manifest-path examples/reconcile/Cargo.toml --all-targets --all-features -- -D warnings
```

The process suite exercises separate executable invocations, including:

- A accepted before B; A-only deep descent while B waits above it.
- An accepted deep batch before return, return before shared `b`, and accepted
  shared batches before root closure.
- Unchanged accepted-file inode, no root-policy replay, and closed staging and
  restrictive directory-permission scopes at returned boundaries.
- Changed kinds or equal-count early-excluded name sets refusing before effects.
- Honest partial opaque failure, no retry, and healthy completion after the failed
  destination root is removed.
- Restored prune/stop controls, terminal resume after all tree roots are deleted,
  non-UTF8 roots/rules/names, wide counters and malformed schema/state/bindings.
- Shared-refresh failures during explicit application subsets, wide diagnostic
  output, and complete-file size refusal without replacing the previous checkpoint.

These are correctness fixtures and a transport-boundary case, not a benchmark or
crash-durability campaign. All **14 cases passed** in both default and all-feature
builds on Linux, together with strict example Clippy and formatting. The production
workspace separately passed 214 default / 217 all-feature nextest tests and 20
doctests at this checkpoint.
