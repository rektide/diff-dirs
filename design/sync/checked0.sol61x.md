---
type: ImplementationAccount
title: Static checked streaming execution and complete read-only audit
description: Accept/refuse inspection shared by the existing frontier, recursive sync and explicit root transitions, without a policy interpreter, whole-tree map or normal-execution prewalk.
resource: /design/sync/checked0.sol61x.md
status: implemented-awaiting-parent-review
generated: { by: "model:openai/gpt-6.1-sol#xhigh", at: 2026-10-18 }
sources:
  - resource: "opencode://ses_eb24f98a3ffewdVqa9lao6tXQu"
    title: Selected and implemented the authorized static checker seam, recursive guard coverage, separate audit and staged selected-root transition; verified correctness and measured owned controls
  - resource: /design/fit/policy0.gpt6a.md
    title: Read updated proposal for shared observations, recursive coverage, pre-admission checks, batch mount topology and operation-time reflink failure
  - resource: /design/fit/policy0.glm53m.md
    title: Read earlier control-policy inquiry; rule vectors and editable prefilters were not selected for this build
  - resource: "file:///home/rektide/src/jj-waltz/design/up-performance/inline-policy0.gpt6a.md"
    title: Read integration split, root-transition need and explicit streaming refusal timing
  - resource: "file:///home/rektide/src/jj-waltz/.test-agent/inline-policy/integration-handoff.md"
    title: Read parent integration edges, source preparation boundaries, native benchmark control and existing tracked-to-payload behavior
  - resource: /README.md
    title: Deliberately amended comparison contract, retained stat-free records, ordering and committed coverage
  - resource: /sync/README.md
    title: Current checked mirror, manual loop, audit and explicitly authorized root transition
  - resource: /src/frontier.rs
    title: Retained reducer, joined epochs, divergence seals and J/K demand reservation without scheduler changes
  - resource: /sync/src/apply.rs
    title: Shared generic operation kernel, immediate metadata handoff and policy-free owned cleanup
---

# Static checked streaming execution

## Selection and scope

The human authorized the diff-dirs rework first; a later GLM agent integrates
jj-waltz. This session changes **diff-dirs only**. Starting parent:
`fd277144c51700ca70d068fc88f3ff72a0d64293`, initially clean. The first implemented
checkpoint is `8658b6da`.

The selected architecture is narrower than either proposal: a generic concrete
checker or closure accepts/refuses a borrowed observation. No skip/preserve rule
engine, mutable diff framework, full payload map, retained operation plan,
watcher/cache, or speculative runtime layer was added. A checkpoint explaining
this choice preceded implementation. The source-root directory contract stays.

Linux mount topology and frozen repository/ignore facts belong to the caller's
front guard. No Linux mount dependency, per-entry device/mount query, mount-table
prefix search, or synthetic reflink probe was added. Real required-reflink errors
remain normal staged-operation failures, never implicit copy fallback.

## The seam

`Check: Sync` has `check(Entry<'_>) -> Result<(), CheckError>`, a blanket `Fn`
implementation, and an unconditional `NoCheck` whose `ENABLED = false` removes
observation construction/check dispatch. `Check::borrowed()` retains that static
specialization while lending one policy to comparison and application. Error
causes are boxed only on refusal; there is no per-entry trait object dispatch.

`Entry` borrows root, relative path (empty at root), side, purpose, already-needed
no-follow metadata, and optional literal link text. A root's native name can be
obtained from `entry.root.file_name()`. Comparison and copying already need
readlink; removal does not read it solely for policy. Audit deliberately reads
literal links so a dry-run can produce the same source-link diagnostics. Special
entry metadata reaches the checker, then the kernel refuses unsupported kinds
even if the checker accepts them. Public records remain names/classification only.

Checker **decisions must be pure and deterministic**. No filesystem mutation,
printing or subprocesses; no ordering/exactly-once assumption. Speculative calls
may overlap, be discarded from public progress, or repeat in application. A
caller collecting warnings must deduplicate thread-safely, keep decisions
independent of collection, and describe them as encountered observations, not
committed notifications. This permits absolute-link diagnostics without adding
a public diagnostics scheduling framework.

## Coverage and work actually removed

| Route | Inspection before affected entry's action |
| --- | --- |
| Equal or divergent paired directories | Core immediate inspection, including both present sides of equal entries |
| Source-only subtree | Recursive copier checks each source entry before its corresponding create/install |
| Target-only subtree | Recursive remover checks each target entry before chmod, descent or unlink |
| Absent selected target | Checked source root, then the same recursive copier |
| Leaf → directory | Checked complete private preparation before removal of old leaf |
| Directory → leaf | Checked leaf preparation, then checked recursive target removal before install |
| Explicit selected-root leaf → directory | Bound real parent, no-follow final leaf, checked staged whole-directory preparation |
| Dry-run | Separate full read-only audit of source then existing target, including opaque interiors |

Normal mirror does **not** run audit first. It compares each paired immediate
directory, yields its ordinary record, applies those names, and continues. One-
sided directories are copied/removed recursively in that operation. Audit has
no mutation/plan phase and is selected explicitly only for read-only readiness.

Public-record validation remains before batch mutation. It now retains just the
current batch's already-observed source/target metadata and source link text for
use by operations, eliminating repeat immediate-leaf metadata lookups. Removal
also reuses its directory metadata for permission relaxation rather than statting
again. This is directory-local state, not a whole-tree map. Copy/removal check
construction reuses paths/metadata; `NoCheck` does not pay root-prefix extraction
per entry. Comparison/application still have a real public-record boundary and
therefore some duplicate observations; no unsafe "checked record" certificate
was invented to hide it.

This library creates the seam that lets JW remove its eager payload validation
passes. It does not itself remove JW's calls or establish an end-to-end reseed
speedup. Native Git/JJ selection work, mount assessment, payload preservation
through tracked refill and source preparation remain integration responsibilities.

## Frontier and failures

The scheduler itself is unchanged. The worker invokes checked inspection outside
its mutex; only a successfully checked directory result can publish paired
children. Refusal yields no children. Divergent results remain sealed until the
caller advances. The same ordered reducer chooses the terminal error and commits
only the DFS prefix. Every epoch joins before a record/error/None is returned;
existing J/K reservation/liveness and cancellation semantics remain.

Mutation is deliberately streaming. A deep refusal can follow successful siblings
or earlier records. The offending entry is neither acted on nor descended; an
accepted ancestor may already have been created/temporarily relaxed. Surviving
existing ancestor modes are restored after joined work. Failed private preparation
counts no installation and is cleaned without invoking user policy on owned
temporary artifacts. Joined sibling/cleanup/restoration issues and completed
operations are retained. No whole-tree rollback or serial-prefix mutation promise.

`Summary` remains a **comparison** summary: successful checked walk exhaustion
does not validate opaque interiors. A successful checked mirror covers those
through its recursive operations. Failed comparison retains `Failed`; a failed
apply retains the incomplete committed comparison prefix and completed physical
work. Whole-root initialization/replacement has `observed: None`. Audit reports
side-local checked counts including roots; errors retain its checked prefix.

## JW integration

```rust,ignore
use diff_dirs::{Check, CheckError, Entry};
fn payload_guard(e: Entry<'_>) -> Result<(), CheckError> {
    let name = e.relative.file_name().or_else(|| e.root.file_name());
    if name.is_some_and(|n| n == ".git" || n == ".jj") {
        return Err("nested administrative name".into());
    }
    Ok(()) // unsupported special types still fail in the kernel
}
let apply = diff_dirs_sync::Applier::new_replacing_checked(
    source, target, diff_dirs_sync::CopyPolicy::ReflinkRequired, payload_guard,
)?;
if dry_run {
    apply.audit()?; // no writes; source and all existing target interiors
} else {
    apply.mirror_in_pool(&pool, limits)?; // OR apply.mirror() without Rayon
}
```

`new_replacing_checked` explicitly authorizes a selected target file/link root;
default construction still rejects it. Target-root links are not canonicalized or
followed. The real parent and root separation remain bound; a failed complete
preparation preserves the old leaf. `replace_root` is available directly if the
caller has already classified that case. There is no directory-root-to-file API
because original roots still must be directories.

For custom loops with **both roots existing as directories**:

```rust,ignore
let mut walk = diff_dirs::walk_in_pool_checked(
    apply.original(), apply.target(), &pool, limits, apply.check().borrowed(),
)?;
while let Some(item) = walk.next() {
    let diff = item?;
    let completed_batch = apply.apply_in_pool(&diff, &pool)?;
}
let observed = walk.into_summary();
```

The manual loop must retain its own earlier completed progress and attach
`walk.into_summary()` on error; `Applier::mirror*` already handles that bookkeeping.
Do not substitute an unchecked walker: it cannot guard equal entries. Do not
pre-audit real execution. Do not use `prune_here` without taking responsibility
for the deliberately omitted paired descendants.

### Equality and confinement limits — keep in JW documentation

Incremental equality is still regular-file `(type,size,exact mtime)` and literal
symlink text. **Mode-only edits and same-size/same-mtime content edits are not
repaired.** Paired directory modes are not swept either. When a leaf really needs
copying, its permissions/mtime converge; that does not strengthen the equality
predicate. There is no hashing, ACL/xattr/ownership repair, fsync or snapshot.

Callers maintain stable nonhostile roots, ancestors, hardlink semantics and mount
topology. Binding and no-follow leaves are not a capability sandbox against
concurrent rename/mount changes. JW must establish mount topology once over the
actual operated scopes before traversal, and re-establish it across external
boundaries that can invalidate it. No per-entry mount/probe tax is needed here.
Linux/btrfs is verified; other platform cfg paths remain unexecuted best effort.

## Verification and measurements

The first checkpoint passed 92 all-feature tests, 64 default-feature tests and
7 all-feature doctests. Tests include equal `.git`, deep one-sided blockers in
both directions, special-file facts, partial sibling work and surviving modes,
absent roots, six existing ordinary type transitions plus guarded transitions,
selected-root symlink replacement, clean stable inode/ctime/mtime and zero
progress, unchanged stat-key limitations, checked parallel admission/errors,
real checker concurrency outside the frontier mutex, and divergence seals.
Existing real strict clone/cross-filesystem failure tests remain green.

The [controlled performance account](../performance/checked0.sol61x.md) records
same-compiler before/after/no-op/concrete-check controls, exact commands, raw
observations, syscall attribution and limitations. Results must not be read as
JW integration measurements.
