---
type: Specification
title: diff-dirs-transfer-sync
description: Root-bound record application, explicit copy policies, staged replacement, partial progress, and optional borrowed-pool filesystem synchronization.
resource: /transfer/sync/README.md
status: implemented-awaiting-parent-review
generated: { by: "model:openai/gpt-6.1-sol#xhigh", at: 2026-10-18 }
updated: { by: "model:openai/gpt-6.1-sol#xhigh", at: 2026-10-22 }
sources:
  - resource: "opencode://ses_e7e8964e7ffe0nSda1UFEvX5PL"
    title: Implemented production coordinated reconciliation with a resumable application barrier, lent runtime policies, exact decision refresh and returned-work accounting
  - resource: opencode://ses_e9d314c29ffeboK450qrHM3qhF
    title: Relocated synchronization and its fixtures, replaced the facade dependency with direct inspection/comparison owners and updated current APIs and commands
  - resource: opencode://ses_ea9c65f5fffeyf5EYvFs581in4
    title: Verified immutable early facts and opt-in native Linux enumeration across required-reflink recursive, staged, audit, preview and root operations
  - resource: "opencode://ses_ead6a1d90ffe7BJn5ftJKW1h06"
    title: Implemented and refined visitor steering, metadata-aware special preservation, honest reporting, private directory-relative DirEntry metadata, recursive presets, preview and guarded public plans with bounded semantic tests
  - resource: "opencode://ses_eb262a81effea1HgF0pPS3f1sy"
    title: Parent commission conveying the human's revised visitor-steering implementation direction; JW integration remains paused for review
  - resource: "opencode://ses_eaecf188affeV49Iu6kJOeHgsX"
    title: Implemented authority-based scoped reconciliation and shared seed/clear kernels, refined parent-batched seeding, and documented static check specialization with bounded semantic verification
  - resource: "file:///home/rektide/src/jj-waltz/design/sync-scope/design0.gpt6a.md"
    title: Earlier authority-based proposal, retained for failure and type-preservation reasoning; its Scope/Absence organizing model and JW defaults were not human-selected constraints on the revised visitor
  - resource: "opencode://ses_eb24f98a3ffewdVqa9lao6tXQu"
    title: Added static checked execution through equal comparison, recursive copy/removal, absent roots and staged root transitions, plus explicit full read-only audit
  - resource: "opencode://ses_eb2c07f5cffe2uTLIC7b1rrD99"
    title: Optimized owned Linux leaf descriptors and parent-local batch staging, retaining metadata, validation, partial-failure and portable-backend contracts
  - resource: "opencode://ses_eb3033c79ffeRIkn14uSNhs7O6"
    title: Implemented and verified separate filesystem applier, borrowed-pool operations, native links, replacement safety, metadata and partial-progress semantics
  - resource: /design/sync/brief0.gpt6a.md
    title: Selected builder guidance and explicit filesystem synchronization scope
  - resource: /README.md
    title: Comparison keys, direction, opaque subtrees, and joined frontier application boundary
  - resource: /transfer/sync/src/apply.rs
    title: Shared operation kernel, target-side staging, permission finalization, and joined sibling failures
---

# diff-dirs-transfer-sync

Optional filesystem synchronization over the tree inspector's finalized **early/late
visitor batches**. Default proposals mirror the source: create absent names,
replace different collisions, prune target-only names, and descend directories.
The visitor can independently steer that work without editing observed facts.
The read-only core never depends on this crate. Ordinary execution needs no Rayon; feature
`parallel` adds borrowed caller-pool comparison and application.

This crate lives at `transfer/sync` and depends directly on
[`diff-dirs-tree-inspect`](/tree/inspect/README.md) and
[`diff-dirs-tree-compare`](/tree/compare/README.md), not the public facade. Rust
imports use `diff_dirs_transfer_sync`; public function names and shared type
identities are unchanged. Historical verification sections below retain their
original commands/package names as evidence, not current recipes.

## Mirror

```rust,no_run
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use diff_dirs_transfer_sync::{mirror, CopyPolicy};
let report = mirror("original", "target", CopyPolicy::Copy)?;
println!("copied {} files", report.completed.files_copied);
# Ok(()) }
```

Both roots must be real directories, except that an absent target is initialized
as a whole tree. Its immediate parent must already exist. Existing file/symlink
roots are rejected by the default binding; an explicit replacing binding is
available below. Construction and root checks perform no writes or source
capability probes. There are no `.git`, `.jj`, ignore, or mount exclusions.

Every regular file uses the explicitly selected policy (no default):

- `Copy`: ordinary `io::copy`, no physical CoW/allocation promise.
- `ReflinkRequired`: real clone or error; **never silently falls back**. Linux
  uses whole-file `FICLONE` on held descriptors; other platforms retain the
  `reflink-copy` 0.1.30 backend.
- `ReflinkOrCopy`: reports actual clone versus copy result. NotFound,
  PermissionDenied, and AlreadyExists do not fall back. Linux reuses its open
  descriptors for ordinary `io::copy` after other clone errors; other platforms
  retain the dependency's fallback policy.

Symlinks are native literal links, never followed or normalized. Windows source
link kind is retained, including dangling directory links where metadata
supports it. Symlink metadata is not mirrored. Unsupported special entries fail
with their path; copied opaque subtrees validate while visiting, without a
separate whole-tree preflight or content-reading pass.

## Visitor-steered synchronization

`Applier::visitor(V)` uses the same statically dispatched `Visitor` throughout:
paired comparison (including clean entries), recursive source-only copy,
target-only pruning, absent-target initialization, explicit leaf-root replacement,
both directory↔leaf transitions, and explicit audit/preview. Root binding/type
checks remain mandatory. There is no separate visitor mirror engine beside an
old Scope/Absence engine: compatibility policies project into the same early
`Names` and late `Operations`, then one staged operation kernel consumes the
validated final `Action` of each named entry.

```rust,no_run
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use diff_dirs_tree_inspect::{CheckError, Names, Operations, Visitor};
use diff_dirs_transfer_sync::{Applier, CopyPolicy, Seed};
struct Payload;
impl Visitor for Payload {
    fn early(&self, names: &mut Names) -> Result<(), CheckError> {
        for name in names.entries_mut() {
            if name.name() == ".jj" { name.preserve_tree(); }
        }
        Ok(())
    }
    fn late(&self, batch: &mut Operations) -> Result<(), CheckError> {
        Seed.late(batch) // create absent names; keep existing leaves/collisions
    }
}
let report = Applier::new("source", "target", CopyPolicy::ReflinkRequired)?
    .visitor(Payload).run()?; // no automatic audit/preview first
assert!(report.policy.satisfied);
# Ok(()) }
```

The [core visitor contract](/README.md#visitor-steered-planning) and
[`Action` Rustdoc](/tree/inspect/src/visitor.rs) define the authority table. Early starts
with every sorted union name admitted, immutable listing presence, and per-side
`EarlyFacts`, **before additional child stat/readlink/open**. `source()` / `target()`
distinguish absence from present Unknown; available kind/inode hints and their
listing/root-metadata provenance are read-only. A suppressed name cannot become
fake source absence.
Late starts with actual comparison facts and normal work, including unchanged
leaves and matched-directory descent. Only existing action slots are mutable.
`name.refuse(cause)` / `entry.refuse(cause)` report an actual logical slot; a
generic callback error reports its batch, not an arbitrary injected path.

Late may preserve a socket/FIFO or another unsupported kind based on its actual
metadata. Its `comparison()` and `baseline()` are None, and its initial `action()`
is None rather than a fictitious file operation. Explicit PreserveTree retains
**both sides and descendants**, including a supported peer at that name. Default
visitors and non-preservation actions still refuse; structural root binding is
unchanged. These names have no normal `DirDiff` category, zero planned/completed
work, and retained coverage that prevents a full-root clean claim. `Report.policy`
includes committed retained slots even when no normal diff batch was yielded.
Audit reports special observations separately as `retained_specials`. No special
file is opened/read; early filtering remains the zero-child-metadata choice.
`AuditFailure.inspected` boxes its cold prefix counters; ordinary field access
still dereferences transparently.

The shared inspector keeps listing handles private through early admission and
obtains admitted no-follow metadata relative to their shared parent on Linux.
Excluded children are not inspected; handles drop after fact collection and never
enter plans. Default std Unix exposes inode but unknown kind, avoiding
`DirEntry::file_type()`'s hidden metadata fallback. Optional Linux `native-dirent`
exposes actual cached d_type/inode and uses `std::fs::Dir` for relative late stats;
it requires nightly (`dirfd`). Enable for sync with
`cargo test -p diff-dirs-transfer-sync --features native-dirent`. This feature
forwards to both inspection and comparison. Default remains unchanged; non-Linux uses std.
Enumeration-cached full metadata (Windows) and mandatory root facts are reused;
other admitted sides obtain required metadata once, including Unknown types.
Literal readlink remains path-based. No speedup or concurrent-replacement safety
is inferred from hints. [Core backend details](/README.md#optional-cached-linux-dirents).

Useful compiled presets, not additional mode flags:

| Visitor | Work |
|---|---|
| `diff_dirs_tree_inspect::NoVisitor` (default) | Normal mirror; equal leaves are no-ops. |
| `Seed` | Recursive tree-level create-if-absent. Keep matched-directory descent; preserve existing leaves, target-only trees and type collisions. |
| `Update` | Create/update source-present names; preserve target-only trees. A nonempty directory-to-leaf collision is blocked intact. |
| `Prune` | Delete target-only names, including below matched directories; retain source-present leaves/collisions and never create source-only containers. |

For custom decisions, `entry.retain()` keeps an existing target directory
container **and continues descent**. `entry.preserve_tree()` also stops descent.
This permits pruning selected children below a retained target-only parent.
Surviving descendants keep every necessary ancestor. `Directory { container:
Create, descend: false }` explicitly creates an empty source-only container;
Keep on an absent container with child work is invalid, never silently overridden.
Whole immediate batches validate before their writes or child admission. A later
child-batch refusal can leave its already-authorized newly created ancestor.

Collisions remain `Presence::Both`: `ReplaceWithLeaf` and
`ReplaceWithDirectory` are indivisible named-entry actions. A policy cannot
authorize only the deletion half as though the source were absent. A
directory-to-leaf replacement locally prechecks **only its affected subtree**
through the same visitor, refusing any static retained descendant before staging,
chmod or dismantling that subtree. A leaf-to-directory replacement prepares the
admitted source subtree before removing the old leaf. Callbacks always receive
final logical paths, even inside private preparation. Owned staging cleanup is
separate from user policy and cannot be blocked by exclusions.

`run_in_pool` uses the core's same joined epochs, deterministic committed DFS,
J/K bounds, demand reserve and divergence seals. Finalized descent alone exposes
children; there are no jobs beneath discarded clean descent. Callbacks run
outside scheduler locks. They must be pure decisions over supplied/frozen facts:
no filesystem I/O/mutation, subprocesses, printing or order-dependent state.
They may repeat, overlap or be speculative, not ordered notifications. Already
started application siblings or prior batches can remain on failure; there is no
rollback, snapshot or finite cancellation-latency promise for stalled I/O.

### Observation, final plan, preview, and progress

* `DirectoryPlan.diff()` / `Report.observed` describe baseline observation, not
  an action-filtered diff or post-sync verification. `Summary.planned` counts
  only immediate actions in the committed paired-comparison prefix.
* `Report.planned` / `Failure.planned` count finalized named namespace proposals
  encountered by application, including recursive batches. A coupled collision
  counts once, plus its descendant work. Directory removal is conditional on no
  surviving descendants, so planned counts are not success counters.
* `Report.completed` / `Failure.completed` count actual visible installations,
  removals and permission work. Discarded private preparation is not completed
  installation. Retained-only/unchanged branches do **zero namespace, staging or
  chmod writes**. Equal-leaf force installation is not supported.

`preview()` is an explicit **read-only joint traversal** using the same early/
late actions, recursive coverage and local collision blockers. It returns
planned/policy counts with zero completed work and no legacy `observed` summary.
It never stages, chmods, probes clone capability or reads file contents. A preview
is not durable authorization after writers resume and does not establish that
real required clones or later namespace operations will succeed. Normal `run`
does not invoke it. `audit()` instead inspects each side separately using
`Purpose::Audit`; it honors visitor descent, but is not a joint collision plan.
Compatibility Absence does not prune explicit audit coverage.

The new rich `DirectoryPlan` is sealed against public fact/action fabrication.
`apply_plan(&plan)` is still a public trust boundary: it freshly validates all
real routing ancestors and re-finalizes under this binding's visitor. Its fresh
signature compares the legacy diff, names, actions, presence and supported
comparison classification, refusing a mismatch rather than ignoring steering.
It does **not** compare every supplied metadata value or file bytes: actual work
uses the fresh facts, and stable trees remain required.
Only `run`'s just-inspected internal seam reuses its batch without that public
revalidation. Public `DirDiff` application also cannot reopen a preserved or
stopped ancestor. Stable, nonhostile caller-owned trees remain required.
Public record application now observes/finalizes its complete immediate directory
batch before selecting its requested names for writes; unchanged siblings can
therefore reach late/Check. Normal `run` reuses the just-observed batch instead
of restatting every selected leaf. Exact old per-entry callback interleaving and
purpose labeling are not an additional compatibility promise; the core specifies
listing → early → metadata → late → validation failure order.

`Applier<C = NoCheck, S = NoScope, A = RemoveAll, V = NoVisitor>` retains old
construction/checked mirror APIs. New `Summary` and `Report`/`Failure` count fields
require updating exhaustive literals; native Rust performance consumers compile
with this change, but no performance campaign was run. This generic library
work does not select JW transfer defaults, migrate its configuration, move `.jj`
metadata, or settle its paused working-copy protocol.

## Coordinated reconciliation and continuation

`coordinated::SyncCursor` composes the comparison frontier with the **same
production application kernel**. `SyncRuntime` separately owns the appliers and
their executable policies. Temporary composite visitor loans last only for each
inspection call: policies need neither cloning nor serialization, and the objects
are not self-referential.

```rust,no_run
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use diff_dirs_tree_compare::Execution;
use diff_dirs_transfer_sync::{Applier, CopyPolicy, SyncCursor, SyncRuntime, SyncTarget};
use diff_dirs_transfer_sync::coordinated::Limits;
let runtime = SyncRuntime::new(vec![
    SyncTarget { id: "A".into(), descriptor: "mirror/v1".into(),
        applier: Applier::new("source", "target-a", CopyPolicy::Copy)? },
    SyncTarget { id: "B".into(), descriptor: "mirror/v1".into(),
        applier: Applier::new("source", "target-b", CopyPolicy::Copy)? },
]);
let mut run = SyncCursor::new(&runtime, Execution::Sequential)?;
let progress = run.drive(&runtime, Limits { groups: 1, applications: 1 })?;
// A may be applied while B is still pending and the root traversal is partial.
let saved = run.into_parts(); // releases all cached full plans/runtime facts
let mut resumed = SyncCursor::from_parts(saved, Execution::Sequential)?;
if !resumed.is_terminal() {
    // A new process reconstructs compatible runtime policies, not saved callbacks.
    resumed.bind(&runtime)?;
    resumed.drive(&runtime, Limits { groups: 100, applications: 100 })?;
}
for lane in resumed.reports() {
    println!("{}: satisfied={}, copied={}", lane.target.0,
        lane.satisfied, lane.totals.completed.files_copied);
}
# let _ = progress;
# Ok(()) }
```

All roots must already be real directories. Source/target and cross-target
equality or nesting are rejected before effects, including resolved name aliases;
Unix additionally rejects identical root device/inode identities. Existing root
ancestors cannot be symlinks. This is still the stable, nonhostile-tree contract,
not protection against hidden mount aliases or concurrent replacement. Whole-root
initialization/replacement remains available through the existing applier APIs,
not through this first coordinated cursor.

### Traversal and application are different acceptance states

`next_group(&runtime)` publishes clean outcomes as well as differences and
lane-local comparison failures. Each successful ordinary directory receives an
unstarted application. `apply(&runtime, &token, &target)` consumes the cursor's own
pending plan, accepts its returned report once, and leaves other lanes pending.
Comparison cannot advance until every current application is accepted, failed or
abandoned. A structural-root preservation/stopped-descent outcome is an accepted
no-op, with no interior application.

The comparison frontier alone advances paired children. One-sided interiors and
replacements remain opaque production calls, including recursive visitor policy,
staging, metadata and permission finalization. A zero immediate `work()` count
does **not** justify skipping application: a retained target-only container may
still select recursive descendant work.

`prune_here` changes selected paired-child memberships, not the immediate batch;
`stop_lanes` abandons selected live lanes. Control tokens are runtime-local and
reissued after restoration. `settle()` closes a disposed group's control window
and folds ready returns without another inspection. Thus a checkpoint can retain
deep partial ancestors, B waiting while A descends alone, a return before a later
shared sibling, or accepted applications before root closure. `heads()` and
`reports()` are nonmutating views, not destructive summary extraction.

### Saved decisions and fresh execution

`SyncParts` is public versioned semantic data, validated by `from_parts` without
filesystem access. It contains the comparison frontier, current application
dispositions, frozen copy policies, callback descriptors and per-lane cumulative
counters. It contains no Metadata, handles, full plans, error causes or resources.
The caller must make each descriptor identify all behavior-affecting callback
inputs; the library cannot introspect arbitrary callback code.

An outstanding `DirectoryDecision` records the complete selected name union:
early-excluded names plus admitted clean/preserved/different entries, acquired
kinds, presence, comparison, baseline and final actions, coordinate, purpose and
coverage. It saves neither old metadata nor bytes. On restoration **all unstarted
lanes are refreshed together** with shared source inspection; every refreshed
signature is checked before any refreshed effect. A mismatch/refusal terminates
its lane while validated healthy peers remain runnable. The application uses
fresh facts. Matching decisions are not a historical content or inode certificate.

Accepted ancestor/root policy is not replayed, accepted applications are not
refreshed, and observations are not committed twice. Physical no-follow routing
is revalidated before effects and before queued paired edges are inspected. Old
whole-ancestor signatures are not required to match their own accepted writes.

A returned opaque application failure is **terminal for that lane**. Its actual
partial progress is counted, remaining lane memberships are stopped, and healthy
lanes continue. There is no same-continuation retry, subdirectory-internal cursor,
partial-copy recovery log or automatic cleanup/reconciliation pass. Start a fresh
run to reconcile the remaining actual filesystem state. Staging/permission scopes
are closed before each returned application; cleanup/restoration failures retain
their existing production error semantics.

### Budgets and reports

`drive` counts joined comparison groups and application requests, including a
refused restore refresh, not bytes/syscalls/memory/latency. One application can
copy a large opaque subtree. Zero budgets do not start new work; ready pure
returns may drain. Lane failures appear in `Drive.failures`; healthy lanes can
still finish, so `Stop::Done` does **not** mean every lane succeeded.

`LaneReport.observed` is the committed **pre-application** comparison projection.
`totals` separately accumulate actual returned application results, including
failures. Cumulative work counters use `u128`; individual kernel reports retain
their existing `u64` types. `retained` includes immediate comparison retention
once plus additional recursive retention. `satisfied` is true only after successful
lane exhaustion, never after failure, explicit stop or traversal prune. Visitor
preservation is part of the declared policy and does not itself imply failure.

Terminal restoration needs no appliers or access to removed tree roots. Transport
and checkpoint publication are application responsibilities. Parts validation
checks structure, not the truth of forged accepted results. Forked/old snapshots
can repeat effects; publication is not transactional or exactly-once.

## Scope and absence authority

Compatibility adapters, not the organizing model: Scope modifies early admission,
Check projects immutable late facts into side-local refusals, and Absence edits
late target-only actions. No filesystem engine consults these policies separately.
The caller's `.visitor(...)` runs after these compiled projections; early excluded
slots never reach late and cannot be reopened there.

```rust,no_run
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use std::path::Path;
use diff_dirs_tree_inspect::{NoCheck, Visit};
use diff_dirs_transfer_sync::{Applier, CopyPolicy, NeverRemove};
let scope = |path: &Path| {
    if path == Path::new("local") { Visit::PreserveTree }
    else { Visit::Reconcile }
};
let absence = |path: &Path| path.starts_with("cache");
let transfer = Applier::new_checked("source", "target", CopyPolicy::ReflinkRequired, NoCheck)?
    .scope(scope).absence(absence);
let report = transfer.run()?; // one streamed comparison/application, no audit first
assert!(report.policy.satisfied);
// Explicit update uses constant NeverRemove; default mirror uses RemoveAll.
let update = Applier::new("source", "other-target", CopyPolicy::Copy)?
    .scope(scope).absence(NeverRemove);
let report = update.run()?;
# Ok(()) }
```

`Applier<C = NoCheck, S = NoScope, A = RemoveAll, V = NoVisitor>` preserves existing unscoped
construction and mirror APIs. `.scope(S)` and `.absence(A)` consume the binding
and retain the same validated roots. `run()` / `run_in_pool(pool, limits)` honor
the selected policies; existing `.mirror()` / `.mirror_in_pool()` are aliases
for those operations (their default binding is still RemoveAll). The free
`update(source, target, copy_policy)` selects NeverRemove explicitly. `Absence`
is statically dispatched, as is `Scope`; closures implement both. Custom
implementations may borrow frozen caller indexes. There is no rule interpreter.

The [core scope contract](/README.md#pair-wide-scope) excludes union names
symmetrically **before observation**, including equal entries. Checks accept or
refuse only participating entries. Scope reaches recursive source-only copy,
target-only removal, absent-root initialization, explicit root replacement, and
both type transitions. Excluded source children stay absent on new outputs.
Preserved target children keep their required real ancestor directories. Private
staging always selects against logical final relative paths. Root identity/type
checks remain mandatory even when the empty relative root is preserved.

Public records are not trusted policy certificates: scope is checked on their
directory, **every ancestor**, and named children before entry metadata lookup.
An opaque removed directory means source absence to reconcile, not unconditional
recursive deletion. Exclusion never turns hidden source presence into deletion.

Source presence replaces unpreserved matching leaves regardless of absence
authority. Target-only leaves are removed only when `Absence::remove(relative)`
permits. Target-only directories are traversed for arbitrary mixed policies:
a retained `old` can contain a removable `old/cache`. Their own names disappear
only if authorized and no retained descendant remains. Constant NeverRemove
prunes the entire target-only branch; NoScope and constant absence decisions
monomorphize without policy dispatch. No filesystem prewalk or full-tree plan
is added. Missing source roots always fail, never mean an authoritative empty
tree. Retained-only/clean/no-op regions perform zero namespace, staging, or chmod
writes, even when raw comparison keeps reporting retained extras.

Directory-to-leaf replacement would erase target-only descendants. A local,
read-only precheck of **only that affected target subtree** refuses the first
preserved or absence-retained descendant (or Check refusal), with its path,
before changing any of its contents. NeverRemove therefore blocks replacement
of a nonempty directory, but permits an empty one. After a successful precheck,
prepare the leaf, remove with the shared recursive kernel, and install. Later
I/O failure remains honest partial progress, not rollback. Leaf-to-directory
replacement prepares the complete scoped subtree before removing the old leaf.

`audit()` uses the same bound scope and check on both existing sides, including
one-sided interiors; excluded boundaries are counted without inspecting them.
It is read-only entry validation, **not an action plan or prediction of deletion
counts**, and absence authority does not suppress participating observations.
Execution never audits first. Neither audit nor a manually fabricated record
certifies whole-root convergence; successful `run` reports policy satisfaction,
while scoped comparison cannot claim full-root clean coverage.

### Listed leaves and explicit container clearing

```rust,no_run
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use diff_dirs_tree_inspect::{NoCheck, Visit};
use diff_dirs_transfer_sync::{CopyPolicy, clear, seed_leaves};
use std::path::Path;
// Existing native path list: no enumeration of unlisted source directories.
let copied = seed_leaves("source", "target", ["src/main.rs", "bin/tool"],
                         CopyPolicy::ReflinkRequired, NoCheck)?;
// A distinct, explicit deletion request. Target container remains bound/alive.
let preserve = |p: &Path| if p == Path::new("local") {
    Visit::PreserveTree
} else { Visit::Reconcile };
let cleared = clear("target", preserve, NoCheck)?;
# Ok(()) }
```

`seed_leaves(source, target, leaves, copy_policy, check)` requires existing real
directory roots and nonempty normal relative leaf paths. It creates only missing
real target parents, refuses any existing target leaf (including dangling links),
and uses the same staged file/link/permissions/exact-mtime kernel as sync. It
inspects current source metadata rather than accepting cached metadata as proof.
Missing source, existing destination, and directory-as-leaf errors have specific
`seed source missing`, `seed destination exists`, and `seed source type`
operations; ancestor errors retain their native path/cause. Existing source and
target ancestors pass Check; new output parents follow recursive copy semantics.
The known list is collected and sorted by parent native component order, then
basename encoded bytes, not input order or flat full-path byte order. Normal
components normalize redundant separators/interior dots; duplicate aliases and
invalid paths fail before writes with zero progress (`seed duplicate path` or
`seed relative path`). Runtime failures preserve the actual sorted installation
prefix, including created parents; parent preparation may precede the first leaf
failure. Same-parent leaves share one lazy staging arena and permission lease,
reserving every batch basename against arena collisions. A depth-bounded active
ancestor stack reuses verified real source/target directories across nested
groups. Roots/ancestors are not re-observed per listed leaf, nor is a parent
restatted just to open its write lease. Source leaf metadata/link text and absent
destinations are still checked individually at installation time. This removes
repeated filesystem work structurally; no new performance measurement is claimed.

Only O(listed names) and the active routing stack are retained, not a discovered
payload/metadata/operation catalog. This first installer is sequential, including inside a caller pool;
no new worker scheduler or parallel admission layer is introduced.

This explicit known-path interface retains its existing Check-only contract and
parent-batched installer; it does not expose early/late steering or silently list
the whole payload to manufacture it. General recursive tree seed is the `Seed`
visitor above. A caller-listed operation's absence refusal/sorted-prefix failure
semantics remain distinct from general mirror and from a whole-directory late
batch. Both still share the actual staged leaf/clone/metadata kernel.

`clear(target, scope, check)` binds an existing real directory and deletes its
participating contents through the **same recursive remover** as sync. It never
removes the bound container, never follows a leaf link, and keeps preserved
children plus necessary ancestors. It returns Report without an observed diff.
It needs no source, fake record, automatic prune, or missing-donor interpretation.

## Caller-owned record loop

```rust,no_run
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use diff_dirs_transfer_sync::{Applier, CopyPolicy};
let applier = Applier::new("original", "target", CopyPolicy::ReflinkRequired)?;
// Both roots must exist for per-record application. If appropriate, initialize
// an absent target first with applier.initialize().
let mut walk = diff_dirs_tree_compare::walk(applier.original(), applier.target())?;
for record in walk.by_ref() {
    let completed = applier.apply(&record?)?;
    // completed counts this application batch, including recursive children.
    println!("reflinked {}", completed.files_reflinked);
}
assert!(walk.into_summary().is_complete());
# Ok(()) }
```

`apply` changes exactly the classified entries, not clean/unmentioned entries.
Same-name directory/leaf transitions are one operation. One-sided directories
are recursive policy units; paired directories remain for the existing walker.
The applier neither advances nor prunes the iterator. Finish applying before
requesting the next record, preserving the core's cached-sibling guarantee.
For a manual scoped loop use `diff_dirs_tree_compare::walk_scoped` (or its pool variant)
with `applier.check().borrowed()` and `applier.scope_policy().borrowed()`;
bring `diff_dirs_tree_inspect::Scope` into scope for the latter. An unscoped manual comparison
would still inspect preserved entries even though application refuses to change
them.

Publicly constructed records are validated **before mutation**: normal relative
directory components, exactly one normal native component per name, no duplicate
or contradictory categories. The paired add-directory/remove-leaf and
add-leaf/remove-directory transitions are allowed. Actual immediate entry types
must match the record. Supplied directory ancestors on both sides must be real
directories, never links. Replaying a stale record is not the retry API;
recompare with `mirror` after partial application.

## Checked execution and audit

```rust,no_run
use diff_dirs_tree_inspect::{Check, CheckError, Entry};
use diff_dirs_transfer_sync::{Applier, CopyPolicy};
fn guard(entry: Entry<'_>) -> Result<(), CheckError> {
    if entry.name == ".git" || entry.name == ".jj" {
        return Err("administrative name in selected payload".into());
    }
    Ok(())
}
# fn main() -> Result<(), Box<dyn std::error::Error>> {
let applier = Applier::new_checked("original", "target", CopyPolicy::ReflinkRequired, guard)?;
// Dry-run ONLY: full read-only coverage, including one-sided interiors.
let coverage = applier.audit()?;
// Actual execution: call mirror directly, NOT audit then mirror.
let report = applier.mirror()?;
// Or, with both roots existing, drive the checked pull loop yourself:
let mut walk = diff_dirs_tree_compare::walk_checked(applier.original(), applier.target(), applier.check().borrowed())?;
for item in walk.by_ref() { let completed = applier.apply(&item?)?; }
assert!(walk.into_summary().is_complete());
# Ok(()) }
```

`mirror_checked` is the free-function convenience. For borrowed-pool execution,
use `applier.mirror_in_pool(&pool, limits)` or the manual checked pool walker plus
`apply_in_pool`. No intermediate whole-tree work map or validation pass is built.
Comparison inspects paired directories; recursive application checks source-only
copies and target-only removals as it goes. The **same checker** covers equal
entries, roots, recursive initialization, transitions, and public-record validation.
Already-needed immediate metadata is retained only for the application batch and
reused by its operations; recursive copy/removal similarly reuse their observations.
The default `Applier<NoCheck>` specializes away checking.

The [core check contract](/README.md#checked-inspection-and-read-only-audit)
defines borrowed facts, `Purpose`, purity/`Sync`, speculative invocation, and
literal-link availability. Checks accept or refuse only; no skipping/preservation
rules in Check itself; use the separate scope below. Private owned staging cleanup
intentionally does **not** run caller policy.
Binding itself performs structural path validation, not interior checks. Operations
check roots before their mutation; the checked walker also checks roots at construction.

Refusal is **directory-batched**, not whole-tree preflight: every immediate batch
finalizes before its writes, but earlier batches, legal new ancestors, or already
started parallel siblings may have changed. The offending entry is not copied, chmodded,
removed, or descended. Directory-to-leaf replacement additionally prechecks only
that affected subtree before dismantling it. Surviving accepted ancestors regain prior permissions.
Failed private preparations are discarded. Failures retain completed operations;
comparison failures keep a failed committed prefix, and apply failures keep an
incomplete observed prefix. There is no rollback or automatic retry. Resolve the
violation, then start a fresh comparison. A successful checked mirror establishes
full visited-policy coverage and stat-key convergence without a second walk;
its `observed` is still a pre-application comparison, not a verification scan.
A manual unchecked walk plus checked apply still cannot validate equal entries
in clean directories that never yield a record.

`Applier::audit()` inspects each existing side's visitor-admitted coverage,
original then target, without joint comparison or writes. With only a Check and
no exclusions/stopped descent this is full side coverage. It is currently a
sequential sorted DFS, independent of
the optional execution pool. `AuditReport` has side-local entry counts including roots;
`AuditFailure` retains its checked prefix and refusal/I/O cause. An absent target
has zero target observations. A dry-run is a current observation, not a durable
authorization after writers resume. Audit reads literal symlink text but follows
neither symlink leaves nor a selected target-root link.
It does not probe clone capability or prove that later writes/metadata operations
will succeed; real staged operation failures remain normal runtime errors.

### Explicit selected-root transition

```rust,no_run
# fn main() -> Result<(), Box<dyn std::error::Error>> {
let applier = diff_dirs_transfer_sync::Applier::new_replacing_checked(
    "original", "selected-target", diff_dirs_transfer_sync::CopyPolicy::ReflinkRequired,
    diff_dirs_tree_inspect::NoCheck, // replace with the caller's payload guard
)?;
let report = applier.mirror()?; // existing dir, absent root, or existing leaf
# Ok(()) }
```

Only this explicit binding permits an existing file/link selected root. Its real
parent is bound; the final leaf is never canonicalized/followed. `replace_root`
(or `replace_root_in_pool`) requires an existing leaf and stages the complete new
directory before removing it. Scope uses final logical names during preparation,
not private staging names. Refusal or reflink preparation failure leaves the
old leaf intact. Once old-leaf removal succeeds, installation failure is partial
progress, not a whole-root transaction. Parent permissions are restored. Ordinary
bindings and `mirror` continue rejecting leaf roots rather than silently following
destination links. Original roots must remain real directories.

## Borrowed caller pool

```rust,no_run
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# #[cfg(feature = "parallel")] {
use diff_dirs_transfer_sync::{mirror_in_pool, CopyPolicy};
let pool = rayon::ThreadPoolBuilder::new().num_threads(4).build()?;
let limits = diff_dirs_tree_compare::ParallelLimits::for_pool(&pool);
let report = mirror_in_pool("original", "target", CopyPolicy::ReflinkOrCopy,
                            &pool, limits)?;
println!("clones {}", report.completed.files_reflinked);
// For manual loops: applier.apply_in_pool(&record, &pool).
// For absent roots: applier.initialize_in_pool(&pool).
# }
# Ok(()) }
```

Comparison joins before a record is returned. Application uses the same operation
kernel as ordinary execution: structured parallel iterators over independent
record names and copied subtree children, then postorder directory permissions.
All started application work joins before the next diff or any return. Recursive
removal within a named unit is serial; independent named removals can overlap.
No internal/global pool configuration, detached task list, extra scheduler,
admission semaphore, async runtime, or separate apply-worker setting is added.

Pool size bounds executing filesystem workers, **not pending bytes**. Current
sorted listings and indexed results cost width-sensitive memory; nested sibling
work and core summaries also consume memory. No whole-tree operation list is
precollected. Comparison J/K still govern speculative directory units, separately
from application. More workers need not improve a saturated filesystem.

## Replacement, permissions, and metadata

Regular leaves are prepared at an absent child of a private **target-side**
temporary directory, metadata finalized, and renamed into place. One arena is
created lazily per touched destination sibling batch, not per leaf or at the
root for an entire tree. Unique slots and a prefix excluding planned sibling
names prevent collisions with legitimate internal-looking source names. Symlink
replacements are staged too. This does not write through destination hardlinks,
chmod an existing leaf, or follow a replaced link. Preparation failure leaves
the old leaf intact. Leaf-to-directory transitions prepare the whole new subtree
before removing the old leaf; directory-to-leaf transitions first run the local
read-only removability precheck, then prepare the leaf before removal. Once
authorized removal starts these transitions are not transactions.

Successful regular copies preserve permissions and exact supported `SystemTime`
mtime. Copy and Linux clone/fallback retain usable source/output descriptors
through metadata finalization and installation, avoiding clone chmod/reopen churn.
Other platforms make privately owned clone/fallback outputs usable before
reopening and retain that descriptor through final restrictive chmod. Exact timestamp
readback is checked **before installation**: unrepresentable target timestamps
fail rather than silently promise convergence. Directory permissions are applied
after all child work joins and arena cleanup finishes, including on failure.
Successful empty arenas use direct removal; failed/stopped preparations retain
robust recursive owned cleanup. Directory mtimes are not copied.

On Unix, authorized existing target directories temporarily gain owner `rwx`
only when an actual namespace write needs it, and regain their prior mode after joined work (or on failed
removal). Copied directories get source modes postorder. Source directories must
be readable/searchable by the caller; existing paired target ancestors must be
searchable/readable for validation/comparison and policy inspection. Inspecting
a subtree that is entirely retained does not relax permissions. Privately owned
failed preparations are different: cleanup can relax them to enumerate/remove
them. Permission/ACL failures are reported.
Copied modes must likewise permit later comparisons under the caller's identity;
source access granted only by ownership/ACLs need not survive copying mode bits
without those ownership/ACLs.
No source or unrelated hardlinked leaf is chmodded. On Windows the analogous
directory readonly flag is relaxed/restored, but readonly **leaves are never
chmodded** to force replacement/removal: OS restrictions can return an error.
Renaming over existing leaves has platform-specific limitations; this is not a
cross-platform atomicity promise. Owned staging cleanup failures are retained.

Mirror convergence is the core's `(type,size,mtime)` / literal-link contract,
**not `rsync -a`**. No mode-only repair, directory metadata sweep, hashing,
ownership/ACL/xattr repair, hardlink topology preservation, fsync, rollback
journal, snapshot, or automatic subtree replacement heuristic is provided.

## Reports, failures, and safety boundary

`Report.observed` is the diff's complete **pre-application observed delta**, not
a post-sync verification. It is `None` for absent-root initialization or explicit
whole-root leaf replacement or explicit joint preview.
`Report.completed` counts installed copied/reflinked files and logical bytes,
created links/directories, recursive removals, and directory permission calls.
Replacing a leaf counts its installation, not a fictitious separate unlink.
One opaque added directory can yield thousands of completed copied files.
Failed private preparations/cleanup are not successful installations. Counters
are not syscall, disk-byte, or crash-durability accounting. Clean sync does zero
writes, including no staging or temporary chmod.

`Report.policy` separately records `satisfied`, `retained`, and `excluded`.
Success means the **declared operation's policy** was exhausted, not that full
roots are equal. Retained counters cover retained leaves, constant-NeverRemove
pruned branches, and required structural containers; excluded counters count
application boundaries, not hidden entries. Comparison exclusions remain in
`observed.excluded`; these counts are not additive unique-path inventories.
`Failure.policy` retains encountered counters but never claims satisfaction.
No retained/processed per-file report catalog is built. `apply_report` offers
these results for just one public record, not for the whole bound tree; existing
`apply` continues returning only completed `Progress`.

`Failure` exposes `primary` operation/path/original cause, `secondary` sibling,
cleanup, and permission-restoration issues, `completed` partial operations, and
the committed comparison prefix in `observed` when available. Stop admission on
failure where feasible; already-started siblings may finish. Select errors in
native-name order among those encountered, not arbitrary completion order;
cancellation means the set encountered can vary. No transaction or serial-prefix
mutation guarantee. Retry via fresh comparison converges without recopying
successful identical files. A stalled filesystem call can stall the joined return.

Root binding rejects identical/nested roots, including missing target paths
through existing ancestors. Existing root ancestors cannot be symlinks.
**Stable trees and nonhostile aliasing are required**: path checks are not a
capability sandbox, protection against concurrent replacement, bind-mounted
aliases, or arbitrary mount topology. Callers keep pending siblings/ancestors
stable and exclude hidden directory aliases. Ordinary hardlinked leaf aliases
are safe because installation replaces names instead of writing old inodes.

Linux/btrfs is actually verified; macOS and Windows are cfg-based best effort,
not tested here. `reflink-copy` has a Windows implementation, which its own docs
call insufficiently tested; it is not Linux/macOS-only.

### Source compatibility and implementation boundary

Plain `walk`, `mirror`, `Applier<C>`, `apply`, and borrowed-pool APIs remain
usable with their previous default meanings. New generic parameters have
defaults. Public `Summary`/`Audit` and sync `Report`/`Failure` gained coverage or
policy fields; exhaustive literals/patterns need updating. `Failure.policy` and
its cold `secondary` vector are boxed to keep streaming Result values compact;
normal field access/iteration is unchanged, but code taking the owned vector
uses `*failure.secondary`. Directory-to-leaf refusal deliberately changed from
partial sibling deletion to the local intact-subtree precheck. Deferred chmod
also means fewer permission operations on partial failures.

The implementation extends the existing pair/frontier and staged operation
kernels. Source copying derives scope coordinates from unchanged bound source
paths, never physical output staging. Recursive removal returns retention and
keeps structural containers; deferred per-directory permission leases join
before restoration. Listed seeds reuse the same staged leaf installer. No
repository parser, filesystem operation catalog, per-entry mount/clone probe,
new scheduler, or automatic reconciliation/prune pass is introduced. This is
the generic library slice only; caller revision/configuration integration stays
outside this repository. No cancelled performance campaign was resumed.

The `Context::observe` Rustdoc explains its construction-site `C::ENABLED` gate.
It is a concrete generic associated constant, not a runtime policy switch:
NoCheck skips guard-only root/basename selection, Entry construction and checker
dispatch, not metadata/readlink needed by the actual operation. Entry::check also
gates direct observation consumers; optimized inlining could make the outer gate
redundant, but keeping it avoids relying on that optimization for field
construction (including less-optimized builds). Declaring false promises
unconditional acceptance and no reliance on invocation. This rationale is not a
measured speedup claim.

### Early-facts verification

The opt-in native-dirent extension passed affected core/sync nextest with
`DIFF_DIRS_SYNC_REQUIRE_REFLINK=1`: **128 all-feature** tests and **101 default**
tests, none skipped. New tiny Btrfs fixtures require actual reflinks and cover
early facts through direct/recursive copying, removal/clear, staged children,
absent/replaced roots, audit, preview and borrowed-pool execution. Backend seams
prove zero metadata calls for early-preserved Known/Unknown names and one
completion for admitted Unknown; an actual cached-type filter's syscall trace
has only the containing-directory open/list/close. Root facts are reused.

Core/sync doctests passed 16 all-feature and 15 default examples; workspace
all-target/all-feature and default core/sync clippy passed with warnings denied.
Formatting, Rustdoc generation and installed stable 1.99 default core/sync checks
passed. Native Linux remains opt-in/nightly; Windows is API-grounded but not
runtime-tested. No benchmark, live workspace, JW production/Cargo change, or
default compiler-baseline switch occurred. [Exact backend/proof account](/design/execution/state0.gpt6a.md#enumeration-can-already-provide-facts).

### Earlier parent-review refinement verification

The late-special and private-DirEntry corrections passed focused semantic cases
first, then the affected core/sync suites with real reflinks required: **121
all-feature tests** and **93 std-only tests**, none skipped. The six new cases
cover socket/FIFO preservation and intact peers/collisions, honest unknown
comparison/retention accounting, held-parent metadata lookup, no-follow metadata
and literal links. The existing parallel metadata case now also proves early
exclusion does not admit a metadata job. A mixed normal/special batch checks that
comparison/application retention is not counted twice.

All **15 doctests**, workspace/all-target/all-feature clippy, std-only core/sync
all-target clippy, warnings-denied Rustdoc, and formatting passed. `apply_plan`
documentation/test now explicitly distinguish projection checking from exact
metadata/byte equality and verify that installation uses fresh bound facts.
Only tiny owned fixtures and compile-time checks ran: no benchmarks, live
workspace fixtures, new backend/tooling, extra operation flags or JW edits.

### Initial visitor checkpoint verification

The 2026-10-19 visitor revision (session
`opencode://ses_ead6a1d90ffe7BJn5ftJKW1h06`) passed:

- `DIFF_DIRS_SYNC_REQUIRE_REFLINK=1 cargo nextest run -p diff-dirs -p diff-dirs-sync --all-features --lib --tests`: **115 passed**, none skipped.
- The same command with `--no-default-features` instead of `--all-features`: **89 passed**, none skipped.
- All-feature core/sync doctests: **15 passed**.
- Workspace/all-target/all-feature clippy, plus std-only core/sync all-target clippy, both with `-D warnings`.
- Core/sync all-feature Rustdoc generation with `RUSTDOCFLAGS='-D warnings'`, and `cargo fmt --all --check`.

Direct visitor cases cover premetadata exclusion, clean late callbacks, independent
creation/collision/prune decisions, legal containers versus stopped descent,
recursive/staged/root paths, coupled collision blockers, public-plan/ancestor
validation, equal-leaf force refusal, zero-write inode/mode/time retention,
deterministic parallel admission/joining, and observed/planned/completed counters.
Existing required-reflink failure, metadata, permission, staging and parent-batched
seed regressions run through the retained suite. New fixtures are tiny; no live
workspace import, benchmark, performance run, new tooling or JW production change
was performed. Linux is verified; other platforms remain unverified.

### Earlier scope and seed checkpoints

The preceding scope/absence checkpoint's required-reflink core/sync
nextest passed **70 tests without default features** and **95 with all features**;
all-feature doctests passed 9 examples. Workspace/all-target/all-feature clippy
with `-D warnings` and `cargo fmt --all --check` passed. New semantic pairs are
tiny; older 1,000-sibling and oversized parallel regression fixtures were reduced
before final verification, retaining concurrency/admission coverage. Only tests
and compile-time checks ran, not performance examples, real-workspace imports,
or timing campaigns. Linux is verified; other platforms remain unverified.

The parent-batched seed/Rustdoc refinement subsequently passed 3 focused seed
tests with real reflinks required, both without default features and with all
features; the all-feature targeted seed/staging/permissions/strict-failure subset
passed 14 tests. Expanded doctests passed 13 examples, targeted core/sync
all-target/all-feature clippy passed with `-D warnings`, and Rustdoc generation
passed with warnings denied. The refinement did not rerun unrelated full suites
or performance cases. Its tiny fixtures check shared directory observations,
mixed file/link batches, nested parents, sorted-prefix failure progress, and
up-front duplicate/invalid-name refusal. No measured performance claim is made.

Implementation/verification evidence: [builder account](/design/sync/implementation0.sol61x.md).
Owned-fixture method and observations: [measurements](/design/performance/sync0.sol61x.md).
Current leaf ownership/staging: [implementation](/design/sync/leaf0.sol61x.md) and
[controlled measurements](/design/performance/leaf0.sol61x.md).
Checked execution and JW handoff: [implementation](/design/sync/checked0.sol61x.md)
and [controlled measurements](/design/performance/checked0.sol61x.md).
