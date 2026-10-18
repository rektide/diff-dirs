---
type: Specification
title: diff-dirs-sync
description: Root-bound record application, explicit copy policies, staged replacement, partial progress, and optional borrowed-pool filesystem synchronization.
resource: /sync/README.md
status: implemented-awaiting-parent-review
generated: { by: "model:openai/gpt-6.1-sol#xhigh", at: 2026-10-18 }
sources:
  - resource: "opencode://ses_eaecf188affeV49Iu6kJOeHgsX"
    title: Implemented authority-based scoped reconciliation, deferred permission writes, local transition precheck, listed-leaf seeds and explicit container clearing with bounded semantic tests
  - resource: "file:///home/rektide/src/jj-waltz/design/sync-scope/design0.gpt6a.md"
    title: Selected authority-based generic synchronization direction, superseding ignored-root selection without moving repository meaning into this library
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
  - resource: /sync/src/apply.rs
    title: Shared operation kernel, target-side staging, permission finalization, and joined sibling failures
---

# diff-dirs-sync

Optional filesystem synchronization over `diff-dirs`: source presence updates
participating paths; **source absence deletes only with explicit authority**.
The default mirror grants that authority everywhere. The read-only core
never depends on this crate. Ordinary execution needs no Rayon; feature
`parallel` adds borrowed caller-pool comparison and application.

## Mirror

```rust,no_run
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use diff_dirs_sync::{mirror, CopyPolicy};
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

## Scope and absence authority

```rust,no_run
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use std::path::Path;
use diff_dirs::{NoCheck, Visit};
use diff_dirs_sync::{Applier, CopyPolicy, NeverRemove};
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

`Applier<C = NoCheck, S = NoScope, A = RemoveAll>` preserves existing unscoped
construction and mirror APIs. `.scope(S)` and `.absence(A)` consume the binding
and retain the same validated roots. `run()` / `run_in_pool(pool, limits)` honor
the selected policies; existing `.mirror()` / `.mirror_in_pool()` are aliases
for those operations (their default binding is still RemoveAll). The free
`update(source, target, copy_policy)` selects NeverRemove explicitly. `Absence`
is statically dispatched, as is `Scope`; closures implement both. Custom
implementations may borrow frozen caller indexes. There is no rule interpreter.

The [core scope contract](../README.md#pair-wide-scope) excludes union names
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
use diff_dirs::{NoCheck, Visit};
use diff_dirs_sync::{CopyPolicy, clear, seed_leaves};
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
Sequential list consumption retains no work catalog and returns partial Progress
on failure. This first installer is sequential, including inside a caller pool;
no new worker scheduler or parallel admission layer is introduced.

`clear(target, scope, check)` binds an existing real directory and deletes its
participating contents through the **same recursive remover** as sync. It never
removes the bound container, never follows a leaf link, and keeps preserved
children plus necessary ancestors. It returns Report without an observed diff.
It needs no source, fake record, automatic prune, or missing-donor interpretation.

## Caller-owned record loop

```rust,no_run
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use diff_dirs_sync::{Applier, CopyPolicy};
let applier = Applier::new("original", "target", CopyPolicy::ReflinkRequired)?;
// Both roots must exist for per-record application. If appropriate, initialize
// an absent target first with applier.initialize().
let mut walk = diff_dirs::walk(applier.original(), applier.target())?;
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
For a manual scoped loop use `diff_dirs::walk_scoped` (or its pool variant)
with `applier.check().borrowed()` and `applier.scope_policy().borrowed()`;
bring `diff_dirs::Scope` into scope for the latter. An unscoped manual comparison
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
use diff_dirs::{Check, CheckError, Entry};
use diff_dirs_sync::{Applier, CopyPolicy};
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
let mut walk = diff_dirs::walk_checked(applier.original(), applier.target(), applier.check().borrowed())?;
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

The [core check contract](../README.md#checked-inspection-and-read-only-audit)
defines borrowed facts, `Purpose`, purity/`Sync`, speculative invocation, and
literal-link availability. Checks accept or refuse only; no skipping/preservation
rules in Check itself; use the separate scope below. Private owned staging cleanup
intentionally does **not** run caller policy.
Binding itself performs structural path validation, not interior checks. Operations
check roots before their mutation; the checked walker also checks roots at construction.

Refusal is **streaming**, not whole-batch preflight: earlier permitted siblings or
records may already have changed. The offending entry is not copied, chmodded,
removed, or descended. Directory-to-leaf replacement additionally prechecks only
that affected subtree before dismantling it. Surviving accepted ancestors regain prior permissions.
Failed private preparations are discarded. Failures retain completed operations;
comparison failures keep a failed committed prefix, and apply failures keep an
incomplete observed prefix. There is no rollback or automatic retry. Resolve the
violation, then start a fresh comparison. A successful checked mirror establishes
full visited-policy coverage and stat-key convergence without a second walk;
its `observed` is still a pre-application comparison, not a verification scan.
A manual unchecked walk plus checked apply cannot validate unmentioned equal entries.

`Applier::audit()` inspects each existing side fully, original then target, without
comparison or writes. It is currently a sequential sorted DFS, independent of
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
let applier = diff_dirs_sync::Applier::new_replacing_checked(
    "original", "selected-target", diff_dirs_sync::CopyPolicy::ReflinkRequired,
    diff_dirs::NoCheck, // replace with the caller's payload guard
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
use diff_dirs_sync::{mirror_in_pool, CopyPolicy};
let pool = rayon::ThreadPoolBuilder::new().num_threads(4).build()?;
let limits = diff_dirs::ParallelLimits::for_pool(&pool);
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
whole-root leaf replacement.
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

Verification for this slice (session cited above): required-reflink core/sync
nextest passed **70 tests without default features** and **95 with all features**;
all-feature doctests passed 9 examples. Workspace/all-target/all-feature clippy
with `-D warnings` and `cargo fmt --all --check` passed. New semantic pairs are
tiny; older 1,000-sibling and oversized parallel regression fixtures were reduced
before final verification, retaining concurrency/admission coverage. Only tests
and compile-time checks ran, not performance examples, real-workspace imports,
or timing campaigns. Linux is verified; other platforms remain unverified.

Implementation/verification evidence: [builder account](/design/sync/implementation0.sol61x.md).
Owned-fixture method and observations: [measurements](/design/performance/sync0.sol61x.md).
Current leaf ownership/staging: [implementation](/design/sync/leaf0.sol61x.md) and
[controlled measurements](/design/performance/leaf0.sol61x.md).
Checked execution and JW handoff: [implementation](/design/sync/checked0.sol61x.md)
and [controlled measurements](/design/performance/checked0.sol61x.md).
