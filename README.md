---
type: Specification
title: diff-dirs
description: Generic deterministic directory comparison, consumer-driven reconciliation, and explicit scan coverage.
resource: /README.md
status: implemented
generated: { by: "model:openai/gpt-6.1-sol#xhigh", at: 2026-10-18 }
updated: { by: "model:openai/gpt-6.1-sol#xhigh", at: 2026-10-19 }
sources:
  - resource: opencode://ses_ea9c65f5fffeyf5EYvFs581in4
    title: Implemented immutable per-side early facts and the explicitly opt-in Linux native-dirent backend after the parent's architectural review
  - resource: opencode://ses_ead6a1d90ffe7BJn5ftJKW1h06
    title: Implemented and refined early/late visitors, metadata-aware special preservation, honest comparison/coverage states, private DirEntry metadata retention, finalized execution, preview and tiny semantic verification
  - resource: /design/performance/inspection0.sol61x.md#e1
    title: Previously verified std DirEntry parent-fd-relative metadata and shared directory-handle lifetime, restored by the visitor refinement without a new measurement
  - resource: opencode://ses_eb262a81effea1HgF0pPS3f1sy
    title: Commissioned and reviewed the visitor interface, then added dedicated source-grounded execution and context/donor-state reading paths
  - resource: opencode://ses_eaecf188affeV49Iu6kJOeHgsX
    title: Implemented pair-wide scope, scoped audit and honest coverage summaries; documented static check/scope specialization and disabled-policy obligations
  - resource: opencode://ses_eb24f98a3ffewdVqa9lao6tXQu
    title: Implemented static accept/refuse checks across comparison, recursive sync and read-only audit, retaining streaming records and frontier semantics
  - resource: opencode://ses_eb39c93e9ffeNju5a7XEsB3wGY
    title: Applied unpinned toolchain direction, commissioned leaf-pipeline optimization, and recorded the user's future coordinated multi-destination streaming direction
  - resource: opencode://ses_eb3033c79ffeRIkn14uSNhs7O6
    title: Implemented the optional filesystem synchronization crate and documented staged replacement, copy policy, partial progress, and borrowed-pool application without changing the read-only core
  - resource: opencode://ses_eb3948bedffe0T37Xc2wI2S5Jg
    title: Implemented generic comparison/CLI and measured baselines, then implemented accepted borrowed-pool directory-frontier concurrency with sealed decisions and committed-prefix summaries
  - resource: /design/fit/fit0.glm53m.md
    title: Initial fit research and contract contradictions, subsequently resolved by parent direction
  - resource: /design/concurrency/design0.gpt6am.md
    title: Parent-accepted directory frontier, ordered reducer, divergence seals, borrowed pool, and J/K demand reservation design
  - resource: /design/performance/opportunities0.gpt6am.md
    title: Independent optimization inventory and architecture-sensitive experiment ranking
  - resource: /design/performance/opportunities0.sol61x.md
    title: Source-grounded optimization inventory, multi-target alternatives, and experiment controls
---

# diff-dirs

A general tree-reconciliation library: compare an **original** tree (source of
truth) against a **target** tree (a copy of some earlier generation of it) and
let the caller drive the differences in their own loop. The library owns the
deterministic walk and the diff model; callers own what reconciliation means —
reflink-sync a payload, print a report, prune a mirror, or feed an audit.

This README is the specification. Implementation follows it; when the README
and code disagree, the README wins until deliberately revised.

For a numbered walkthrough, see [net execution flow](design/execution/flow0.gpt6a.md)
and [context payloads and donor state](design/execution/state0.gpt6a.md). Both keep
the generic library, an example consumer, and JW-specific behavior separate.

## Positioning

- A **generic library first**: embedded by tools that drive the walk in their
  own loop and apply per-entry policy. jj-waltz payload sync is the primary
  motivating use case, not a special case in the API.
- A **thin CLI** wraps the same walk for standalone/human use (JSONL or text).
- It does not reconcile by itself. It produces precise, ordered, deterministic
  differences; the caller's loop applies policy.
- Actual copy/sync procedures belong in a **separate optional crate**, not the
  diff core. [`diff-dirs-sync`](sync/README.md) now supplies root-bound record
  application and mirror convenience operations, with explicit ordinary-copy,
  required-reflink, or fallback policy. It is a consumer, never a core dependency.

Historical motivation inherited with the original spec (Linux/btrfs, warm
cache, 192k-entry `node_modules`): ~0.24s single-tree enumeration, ~1–2s
two-sided stat comparison, and 13.8s wall / 3.4s CPU for `rsync -n`. The method
and fixture behind those numbers were not recovered; **they are not results
measured by this implementation**. Avoiding blind delete-and-recopy is the
motivation, not a claim of measured end-to-end sync speedup.

## Model

- **Direction:** compare `original` against `target`. "Added" = original-only
  (copy to target); "removed" = target-only (delete from target). The library
  never mutates either tree.
- **Comparison keys:** regular files: `(type, size, SystemTime mtime)` with
  exact available precision, including pre-epoch times; symlinks compare their
  literal native readlink target, not normalized path components. Regular-file
  ↔ symlink is `files.changed`. Directory ↔ leaf produces paired added/removed
  categories in one record. Directories are structure, never stat-compared —
  by default a both-sided subdir is descended and its contents compared
  entry-by-entry.
- **Trust model (stated plainly):** file-level stat equality is the same
  grade as jj's working-copy cache; it does not catch an in-place byte edit
  that preserves size and mtime. No subtree-level shortcut is taken:
  directory mtimes are a name-change log (create/remove/rename of direct
  children), not a content fingerprint — pruning on them would silently
  miss divergences below unchanged dir mtimes. Mode-only changes (including
  executable bits), ownership, ACLs, and other metadata are outside this
  comparison contract. Hashing is not offered.
- **Comparison roots:** both must exist as real directories; missing roots, files, and
  symlink roots are errors. A consumer handles absent or non-directory selected
  roots using whole-root policy before calling `walk`. Sync offers absent-root
  initialization and an explicitly authorized, staged leaf-root transition.
- **Safety:** symlink leaves are not followed. Unsupported special-file entries
  in examined listings fail with their path; one-sided directories are opaque
  units, so their descendants are not validated by comparison alone. Checked
  sync validates them during recursive application; explicit audit validates
  them without mutation. This is not a sandbox or an
  atomic snapshot: callers retain quiescence and confinement, including checks
  against ancestor symlinks and concurrent path replacement. No special `.git`,
  `.jj`, mount, or ignore-rule policy is built in.

## The walk

One comparison model, with sequential or optional concurrent execution:

- Entries compared in bytewise-sorted name order; the walk is **depth-first
  pre-order** (a directory's record is immediately followed by its
  subdirectories' records, so each divergent subtree streams as a contiguous
  block). Sorting uses `OsStr::as_encoded_bytes()` (raw bytes on Unix).
  Sequential memory includes current sorted listings and pending child names on
  the active descent, **not depth alone**. Parallel execution additionally holds
  bounded uncommitted directory results and running listings. Wide directories
  cost memory; J/K are count bounds, not byte bounds. No whole-tree record list
  is retained; the summary can grow.
- Each directory pair lists both sides (sorted) and diffs **immediate
  children only** — records never describe descendants of other records.
- Files present on both sides are stat-compared (`type, size, mtime_ns`;
  symlinks compare readlink targets) → changed. One-sided → added/removed.
- Subdir pairs present on both sides: **always descended** — no stat-based
  shortcut. One-sided subdirs → added/removed dir entries; never descended
  (wholesale units).

## Records

One record per examined directory that has at least one difference:

```rust
struct DirDiff {
    dir: PathBuf,                    // relative; "" is the root pair
    files: ChangedAddedRemoved<Vec<OsString>>,
    dirs:   AddedRemoved<Vec<OsString>>,
}
```

- Names and classification only — **no stats**. The walker performed the
  comparison; classification is the output. Consumers that reflink, unlink,
  report, or audit act on identity, not on metadata.
- A both-sided dir's "changed" status is an aggregate, known only bottom-up,
  so it lives in the post-walk **summary**, not in streamed records.
- Clean directories emit nothing; a fully clean comparison emits no records.
- Each category is independently bytewise sorted. Both immediate listings are
  fully materialized before a record is returned: consumers may safely replace
  its classified immediate children. Paired child directories are opened later;
  replacing one requires `prune_here()` or stopping, not arbitrary mutation of
  pending descent paths.
- In parallel execution, independent permitted branches may be inspected ahead
  of the ordered cursor. Every differing directory, even a speculative one,
  **seals all its paired children** until its record has been delivered and the
  caller advances. All work joins before returning a record/error/`None`.

## Summary

`into_summary()` consumes the walker **without doing further IO**. It describes
only the **committed DFS prefix**, even after a break or error. Private
speculative inspections/results are excluded, including completed clean ones:

- `counts.files.{changed,added,removed}` and `counts.dirs.{added,removed}` count
  immediate entries. A one-sided subtree counts as one, never all descendants.
- `topmost_divergent`: directly divergent directories with **no directly
  divergent ancestor**, in DFS pre-order. A clean root with changes below it
  does not collapse the answer to the root.
- `changed_dirs`: both-sided directories whose examined subtree contains a
  difference, including ancestors and the root, in DFS post-order. This is the
  aggregate directory-change answer, separate from immediate entry counts.
- `examined_directories`: successfully compared and committed directory pairs.
- `state`: `Incomplete` before normal exhaustion, `Complete` after `next()`
  returns `None`, or `Failed` after the terminal error.
- `pruned`: paths explicitly skipped by the caller. Normal iterator completion
  with pruning is **not** full coverage. `is_complete()` requires normal
  exhaustion and no pruning; `is_clean()` additionally requires zero differences.

After the last record, call `next()` until `None` to establish completion.
Consuming an unstarted walk or breaking early never claims a clean scan.

## Driving the walk

The walker remains a pull iterator running inside the consumer's loop. Compiled
early/late visitors supersede the old "no callbacks, no traits" restriction.
Legacy checks and scopes are thin visitor projections; plain `DirDiff` loops
remain usable with their default comparison meanings.

```rust,no_run
fn compare(original: &std::path::Path, target: &std::path::Path)
    -> Result<diff_dirs::Summary, diff_dirs::Error>
{
let mut walk = diff_dirs::walk(original, target)?;
while let Some(diff) = walk.next() {
    let diff = diff?;
    // apply policy per record, in your own loop
    let take_whole_subtree = false; // consumer policy, not a library heuristic
    if take_whole_subtree {
        assert!(walk.prune_here()); // do not descend into this directory
        continue;
    }
    // ...reflink/unlink/report per entry...
}
Ok(walk.into_summary())             // topmost divergent subtrees, counts
}
```

- Records arrive depth-first pre-order; each divergent subtree is a
  contiguous run of records.
- `prune_here()` — legal immediately after an `Ok` record is yielded — abandons
  descent into that record's directory (wholesale-subtree handling by
  consumer choice, never by stat guess). It returns `true` once; before a record,
  after another `next()`, after an error, or when repeated it returns `false`
  without action. Even an explicit leaf prune is recorded conservatively.
- `break` stops the walk entirely. Errors surface as `Result` items/`next()`
  and abort naming the offending path. The terminal `Err` is followed by fused
  `None`; summaries retain `Failed`, never `Complete`.
- Listing failures are selected original-side first, before metadata work.
  Metadata errors are deterministic by native name, then original before target
  for equal names. Parallel workers never race to select the reported error.
- A directory failing midway emits no partial record and contributes no counts.
- `into_summary()` consumes any walk and returns the examined aggregate.

Both consumption styles below run over this same walk — the library makes no
distinction between them. See [Use cases](#use-cases).

## Visitor-steered planning

`Visitor: Sync` has **one early name batch and one late operation batch per
directory**. It is statically dispatched Rust, not an interpreted rule list or a
shared mutable scheduler. Every name initially participates; supported entries
initially propose normal mirror work. Equal leaves remain no-ops.

* `early(&mut Names)` sees the native-byte-sorted union of immediate basenames
  and immutable per-side [`EarlyFacts`](src/visitor.rs), plus
  `Presence::{SourceOnly,Both,TargetOnly}` from listings. Available cached kind
  and inode facts require no additional child observation I/O. It can
  `preserve_tree()` pair-wide or return a refusal, **before child metadata,
  readlink, checks, or entry into that subtree**. Presence is not erased to make
  a hidden source look absent. The containing listing and structural root checks
  are still necessary. A structural-root singleton is explicitly marked and its
  metadata necessarily precedes steering.
* `late(&mut Operations)` sees actual no-follow metadata, already-needed literal
  links, immutable baseline comparison and **all** admitted proposals, including
  clean leaves and matched-directory descent. It edits existing `Action` slots,
  not names, facts, types, or injected paths. `retain()` keeps an existing target
  directory while allowing child work; `preserve_tree()` also stops descent.
  Named `refuse(cause)` methods attach an actual slot's final logical coordinate.
* A whole immediate batch is validated before application or child publication.
  `Directory { container: Keep/Create/Remove, descend }` separates container work
  from descent. Keeping an absent source-only container while admitting child
  writes is invalid. Directory↔leaf replacements are **coupled actions**, never
  independently editable delete/create halves. Equal-leaf force installation is
  deliberately not supported. The exact authority table and staging contract
  are in [`Action`'s Rustdoc](src/visitor.rs).

```rust,no_run
use diff_dirs::{CheckError, Names, Operations, Presence, Visitor};
struct SeedExceptAdministrative;
impl Visitor for SeedExceptAdministrative {
    fn early(&self, names: &mut Names) -> Result<(), CheckError> {
        for name in names.entries_mut() {
            if name.name() == ".jj" { name.preserve_tree(); }
        }
        Ok(())
    }
    fn late(&self, batch: &mut Operations) -> Result<(), CheckError> {
        for entry in batch.entries_mut() {
            if entry.presence() != Presence::SourceOnly {
                let paired_dirs = entry.source().is_some_and(|f| f.metadata().is_dir())
                    && entry.target().is_some_and(|f| f.metadata().is_dir());
                if paired_dirs { entry.retain(); } else { entry.preserve_tree(); }
            }
        }
        Ok(())
    }
}
# fn main() -> Result<(), diff_dirs::Error> {
let mut walk = diff_dirs::walk_with_visitor("source", "target", SeedExceptAdministrative)?;
for batch in walk.by_ref() {
    let batch = batch?; // immutable DirectoryPlan; core never applies it
    println!("{:?}: {} planned actions", batch.operations().dir(), batch.work());
}
let observed = walk.into_summary();
assert!(!observed.is_clean()); // selected coverage is not full-root equality
# Ok(()) }
```

`walk_with_options_visitor` and `walk_in_pool_visitor` share the existing DFS,
joined epochs, J/K demand reservation and divergence seals. The **finalized**
batch determines child admission; a clean directory whose descent was stopped
cannot expose descendant jobs. Callbacks run outside frontier locks. A rich walk
still yields only observed divergent directory batches and leaves one-sided
interiors opaque; clean batches nevertheless run late and contribute steering
coverage. [`diff-dirs-sync`](sync/README.md#visitor-steered-synchronization)
recursively inspects those interiors through the same contexts and consumes
finalized actions, not a filtered report.

Phase failure order is explicit: listing, early steering, admitted metadata in
native-name/source-before-target order, late steering, then action validation.
Late never receives an incomplete metadata batch. This deliberately changes old
per-entry Check/metadata interleaving while retaining deterministic committed DFS
errors and zero counts for a failed directory. Only early can avoid an unwanted
metadata/readlink failure; late preservation is not a premetadata filter.

Listings retain private shared-parent handles through union/admission instead
of reconstructing child metadata lookups from full paths. Default admitted entries
use `DirEntry::metadata()` (no-follow); native Linux uses
`Dir::symlink_metadata(basename)`. Excluded entries are dropped uninspected.
Handles are released after needed metadata/literal links are collected, before
late/publication, and never enter frontier plans. This preserves std's existing
[Linux directory-relative metadata path](design/performance/inspection0.sol61x.md#e1),
also in the [opt-in native backend](#optional-cached-linux-dirents), without a
measured speedup claim. Enumeration-cached full metadata (Windows) is reused;
otherwise admitted sides obtain full metadata once, even when kind was Unknown.
Readlink remains literal and path-based; stable caller-owned trees are required.

`DirectoryPlan.diff()` and `Summary.counts` retain baseline observed comparison;
`DirectoryPlan.work()` and `Summary.planned` count immediate proposed namespace
actions, with a collision counted once. Conditional directory removal may later
retain a necessary ancestor. `retained` counts changed action slots, including
stopped clean descent. These are neither recursive payload catalogs nor actual
completed operations. Exclusions/retention never certify full-root cleanliness.

An unsupported observed kind (socket, FIFO, etc.) has `comparison() == None`,
`baseline() == None`, and initially `action() == None`: no normal file comparison
or mutation is invented. Late can inspect its real metadata and explicitly
`preserve_tree()` for the **whole pair**. Other actions/defaults still refuse;
structural root requirements remain mandatory. Such names are omitted from
`DirDiff` categories, contribute one retained slot and zero planned work, and
prevent a full-root complete/clean claim even if `has_differences()` is false.
`is_different()` means a known supported difference, not proof of equality when
false. Successfully finalized entries always have `Some(action)`; `action()` and
`baseline()` are optional in this revised rich-batch API. No special contents are
opened or read. Explicit audit separately counts them as `retained_specials`,
never as regular files. Early preservation still avoids their metadata entirely.

Visitors must use frozen caller facts and supplied observations: no filesystem
I/O/mutation, processes, printing or order/count-dependent decisions. Parallel
calls may be speculative, repeated or uncommitted, not ordered notifications.
Refusal prevents this batch's writes/admission; prior work or started siblings
can remain. There is no snapshot, rollback, or finite stalled-call latency bound.
`NoVisitor::ENABLED == false` skips hook dispatch/error mapping and callback-only
root construction, not ordinary observations or directory-plan construction.
Declaring false promises both callbacks are semantic no-ops and may never run;
no measured performance improvement is claimed.

`audit_with_visitor(root, side, &visitor)` provides explicit read-only one-sided
coverage, using the same final descent decisions and literal links. Its counts
describe successfully finalized directory batches, not deletion predictions.
Use sync's joint `preview()` when collision and create/delete decisions matter.

This extends the earlier [constrained inspection-gate proposal](design/fit/policy0.gpt6a.md#3-the-gate-a-constrained-prefilter):
that proposal deliberately deferred zero-stat exclusion and limited edits to
preservation/refusal. The revised human-directed interface adds the early phase
and actual typed work steering without accepting its independent JW policies.
The [frontier design](design/concurrency/design0.gpt6am.md) remains the ordering,
joining and admission precedent, not a reason to expose children before late.

## Checked inspection and read-only audit

Compatibility surface: `Check` and `Scope` project into the visitor batches
below; they are not separate decision or filesystem engines.

```rust,no_run
use diff_dirs::{CheckError, Entry};
fn check(entry: Entry<'_>) -> Result<(), CheckError> {
    if entry.name == ".git" || entry.name == ".jj" {
        return Err("administrative entry in payload".into());
    }
    Ok(())
}
# fn main() -> Result<(), Box<dyn std::error::Error>> {
let mut walk = diff_dirs::walk_checked("original", "target", check)?;
for item in walk.by_ref() { let diff = item?; /* caller applies */ }
let compared = walk.into_summary();
// Separate, explicitly requested full validation; NEVER implicit before sync:
let inspected = diff_dirs::audit("original", diff_dirs::Side::Original, &check)?;
# Ok(()) }
```

`Check: Sync` accepts or refuses each observed **present side**, including roots,
equal leaves, paired directories, and unsupported entry types (which the library
still rejects if the checker accepts them). `Entry` borrows the bound root,
already-built full path, native basename, side, `Purpose`, and already-needed
no-follow metadata. `relative_path()` borrows the root-relative path (empty for
root) **lazily**; name/type checks need not parse root prefixes or allocate.
`link_target` is literal text where comparison/copy needs it;
removal does not add `readlink` solely for a hook. Audit deliberately observes
literal links too. No mount facts, syscall-request interface, rule vectors,
or retained inspection catalog are added. Preservation is a separate name-stage
scope, not a successful check or a refusal.

Closures with `Fn(Entry<'_>) -> Result<(), CheckError> + Sync` implement `Check`.
Annotate a closure's argument as `Entry<'_>` for lifetime inference. A concrete
implementation can borrow command context. `Check::borrowed()` lends any checker
to another operation without changing its static dispatch or no-op specialization.
Existing APIs use `NoCheck`; its disabled associated constant removes hook work.
Public `DirDiff` stays stat-free. Metadata is not a reusable validation certificate.

Checks must make **pure, deterministic decisions**: no filesystem mutation,
printing, processes, or reliance on invocation order. Parallel speculative checks
run outside the frontier mutex, before results can expose children. Calls can
overlap, repeat across comparison/application, and concern an uncommitted suffix.
They are not ordered/exactly-once notifications. If a caller collects diagnostics,
use an idempotent thread-safe collector and label them as encountered observations,
not committed progress; do not let collection affect decisions. `Error::Refused`
retains the caller's cause and the observed path. Existing ordered-prefix error,
join, divergence-seal, pruning, and J/K contracts are unchanged.

`walk_with_options_checked` and `walk_in_pool_checked` are the execution variants.
A checked comparison still leaves one-sided interiors opaque: successful iterator
exhaustion certifies **comparison** coverage, not full recursive policy coverage.
Use [checked sync](sync/README.md#checked-execution-and-audit) or explicit audit
for that. `audit(root, side, &check)` is a read-only sorted DFS of every entry,
including a leaf root without following it. It returns side-local counts including
the root; `AuditFailure` carries the successfully checked prefix. It neither
compares roots nor builds a plan. Sync's audit binds roots and combines both sides.

## Use cases

### Pair-wide scope

`walk_scoped(original, target, check, scope)`, `walk_with_options_scoped`, and
`walk_in_pool_scoped` use the same traversal and scheduler as the plain APIs.
`Scope: Sync` is a compatibility early visitor returning `Visit::Reconcile` or `Visit::PreserveTree` for a logical
root-relative path. Closures implement the trait; `NoScope` compiles away the
selection. `Scope::borrowed()` preserves static specialization.

Scope is pure and stable, like Check. It selects each sorted union **name before
metadata, readlink, checks, or child admission**, symmetrically on both sides.
Preserved source names never become apparent source absence. Preserved ancestors
cannot be reopened. Root identity/type checks still run; preserving the empty
relative root prevents listing its contents. No repository-specific names exist
in the library. Parallel scope runs in the inspection worker, outside scheduler
locks, before the result can expose children; joined epochs, divergence seals,
J/K budgets, and committed-prefix reporting are unchanged.

`Summary.scoped` identifies scoped coverage; `excluded` counts committed union
boundaries, not their hidden contents. `state == Complete` means the scoped walk
exhausted. `is_complete()` and `is_clean()` remain **full-root** certificates and
are false for scoped walks, even when no excluded name was encountered.
`audit_scoped(root, side, &check, &scope)` also skips before observation, including
one-sided interiors. Its `Audit.excluded` counts side-local excluded boundaries.
Public records remain names/classification only, not policy certificates. The
[sync specification](sync/README.md#scope-and-absence-authority) covers recursive
scope enforcement, retained ancestors, and source-absence authority.

### 1. Apply-in-loop — jj-waltz payload sync

The first embedded consumer, and the shape the record model was designed
around.

**Context.** jj-waltz maintains feature workspaces as copies of a prepared
donor (`~/a/a/opencode` in the operator's pool): the tracked tree through jj,
the ignored payload — `node_modules`, `**/node_modules`, `**/dist` — as
private reflink CoW copies, roughly 192k entries / 8.1 GB per workspace. Its
payload phase historically deleted and re-copied every selected root on every
re-sync: ~15–18s of pure syscall churn even when nothing diverged, inside
re-seeds measured at 38s wall.

**Desire.** Re-sync cost proportional to actual divergence. An unchanged
payload should cost the walk and nothing else;
a rebuilt payload should cost the delta, not the whole tree.

**Application.** jw drives the walk over each selected root pair and applies
each record with primitives it already owns:

- `files.changed` / `files.added` → reflink the donor file over the target
  path, then chmod and **restore mtime** — restoring mtimes is what keeps
  jj's working-copy stat cache valid across copies, so the workspace stays
  snapshot-cheap after every sync;
- `files.removed` → unlink;
- `dirs.added` → existing subtree clone; `dirs.removed` → existing subtree
  remove;
- `prune_here()` when a divergence is cheaper to replace wholesale than to
  enumerate — a consumer choice, never a stat guess.

**Why in-loop.** Records are small and contiguous per divergent subtree, so a
failure names the exact record; no intermediate candidate lists sit in
memory; and the applier composes with jw's existing confinement and safety
checks rather than replacing them. After applying, every touched file carries
the donor's `(type, size, mtime_ns)`, so the next reconciliation compares
clean and again costs only the walk — steady-state cheapness is a property of
the apply discipline, not of a cache the library maintains.

### 2. Gather — audit, report, batch reconcile

**Context.** Not every consumer wants to act incrementally. Drift audits
("what did that agent do to this workspace's payload?"), backup-vs-live
comparisons, mirror verification, and scripted repairs all want the complete
difference first and policy second — often rendered, logged, or reviewed
before anything is applied.

**Desire.** Deterministic, diffable output; exact deletions; a coarse
human-scale view (the summary's topmost divergent subtrees) alongside the
per-entry detail; and batch application with familiar tools rather than a
bespoke applier.

**Application.** Run the walk, accumulate each record's entries into
candidate lists, then act:

- `cp -a --reflink=always --parents f1 f2 … target/` for scattered
  changed/added files — `-a` preserves the mtimes the next comparison relies
  on, `--reflink=always` keeps the copies CoW-shared;
- one-shot subtree copies (`cp -a --reflink=always`) for each `dirs.added`;
- exact deletions of everything in the `removed` lists, `rm -rf` for dirs.

The thin CLI streams records as JSONL or text, then emits the summary, and
exits 0/1/2. Batch consumers may collect them instead.

**Standing rules for this style.** Deletions run before copies within a
record, so file→dir type transitions apply cleanly. Deletions are exactly
the reported `removed` entries — never a sweep of "everything unmentioned,"
which the library cannot express because clean subtrees emit nothing by
design. Copy tools must preserve mtimes, or the next reconciliation loses
its cheap keys.

## CLI (thin wrapper)

`diff-dirs ORIGINAL TARGET [--format json|text]`
Exit codes: `0` no differences, `1` differences found, `2` usage/IO error
(never silently truncate; unreadable directories abort naming the path).

Build/run with `cargo run -p diff-dirs-cli -- ORIGINAL TARGET --format json`,
or `cargo build --release -p diff-dirs-cli` and use `target/release/diff-dirs`.
Default format is text. `--` terminates options for dash-prefixed paths.

Each output record has `type: "diff"`, `dir`, `files`, and `dirs`. A final
`type: "summary"` line includes counts, directory aggregates, scan state,
`complete`, and `clean`. An IO failure after construction emits a failed-prefix
summary and diagnostic on stderr, then exits 2; invalid roots/arguments fail
before records. Streamed records before an error are **not** a complete scan.
Output write failures also exit 2.

Text uses Rust debug escaping for all native paths/names, including control
characters and invalid bytes. JSONL uses strings for valid Unicode and tagged
native representations otherwise; it never substitutes replacement characters:

```json
{"encoding":"unix-bytes","bytes":[120,255]}
```

On Windows the tag is `windows-wide` with a `units` array of native UTF-16
code units. Other platforms use `rust-native` and encoded `bytes`, meaningful
only on the same platform/toolchain. Tags may occur anywhere a path or name
appears, including summary lists; clients must handle string-or-tagged values.
Root-relative `dir` is the empty string for the root pair.

## Embedding

The root `diff-dirs` package is std-only **by default**, edition 2024;
embedding does not require a runtime, CLI dependencies, or sync
component. Optional `parallel` adds Rayon; `walk()` remains sequential even
when the feature is compiled in.
The separate `cli/` workspace member owns standalone rendering dependencies.
The separate `sync/` member owns optional filesystem mutation and its dependencies.
Path dependency vs vendoring vs publishing remains an integration decision.

### Optional cached Linux dirents

`native-dirent` enables rustix's single-pass raw Linux listing and the experimental
`std::fs::Dir` API (`dirfd`): **Linux builds with this feature require nightly**.
It exposes cached type/inode hints to early visitors without hidden stat fallback,
and retains one shared parent descriptor for admitted relative metadata. Enable
with `cargo test -p diff-dirs --features native-dirent`, or add
`features = ["native-dirent"]` to the embedding `diff-dirs` dependency. To exercise
sync with it: `cargo test -p diff-dirs-sync --features diff-dirs/native-dirent`.
There is no toolchain pin or default compiler-baseline change. On non-Linux the
feature leaves the std backend in use.

`Name::source()` / `target()` return `Option<&EarlyFacts>`: None is absent;
Some with `FileKind::Unknown` is present but its type is unavailable. Facts expose
`kind()`, optional `inode()`, and `origin()`. Default Unix listings expose inode
but conservatively leave kind Unknown; std's `file_type()` can stat on DT_UNKNOWN
and is never used to fill early hints. Native Linux exposes actual cached d_type,
including Unknown. Windows std uses cached enumeration metadata, reused late.
Structural roots reuse mandatory metadata (`FactOrigin::RootMetadata`), not
listing facts. These hints are neither full stat nor equality/replacement proof.

jj-waltz is a prospective consumer, not changed by this library implementation.
Its integration must preserve selected payload roots through earlier cleanup
before an incremental final payload loop can help; changing only that final
loop does not establish end-to-end benefit. The optional sync crate handles
absent-target initialization, exact deletions, type transitions, staged
leaf replacement, copied permissions/mtime, and caller-pool application. It adds
no jj-waltz integration or CLI mutation mode. Confinement, alias stability,
repository-specific policy, and final verification remain consumer concerns.

### Optional filesystem synchronization

```rust,ignore
let report = diff_dirs_sync::mirror(original, target,
                                    diff_dirs_sync::CopyPolicy::ReflinkRequired)?;
// Or bind Applier once and apply each DirDiff in the caller's existing loop.
```

[`sync/README.md`](sync/README.md) owns the API and safety contract, including
borrowed Rayon pools, whole-subtree operations, read-only-directory handling,
exact supported timestamps, and joined partial failures. Mirror includes
target-only deletion; it is not an unconditional metadata-repair sweep or a
whole-tree transaction. Clean repeat synchronization performs zero writes.
Observed diff counts remain distinct from completed recursive filesystem work.

### Optional shared-pool parallel execution

Enable `features = ["parallel"]`. Prefer the explicit borrowed-pool API:

```rust,ignore
let limits = diff_dirs::ParallelLimits::for_pool(&app_pool); // initially J=P, K=4P
let mut walk = diff_dirs::walk_in_pool(original, target, &app_pool, limits)?;
while let Some(item) = walk.next() {
    let diff = item?;
    if replace_whole(&diff) {
        assert!(walk.prune_here()); // seal before replacing this directory
        replace_target_subtree(&diff)?;
    } else {
        apply_classified_children(&diff)?;
    }
}
let summary = walk.into_summary();
```

`ParallelWalk<'pool>` stores `&rayon::ThreadPool` and installs every epoch in
that pool; callers do not have to surround each `next()` with `install`.
Applications may own an `Arc<ThreadPool>` and lend `&*pool`. No internal pool,
global reconfiguration, detached task, or async runtime is introduced.

One job performs a **whole immediate directory comparison**, with sequential
enumeration/metadata internally. Clean completions dynamically expose new
directory jobs; there is no lockstep batch or permanently assigned subtree
worker. Only an ordered reducer updates records/errors/summaries. Each `next()`
joins a Rayon scope before returning. Drop, break, and `into_summary()` do no IO
and have no running jobs to wait for. A stalled filesystem call can still stall
that epoch; joining does not impose a finite latency bound.

`ParallelLimits` uses `NonZeroUsize max_directory_jobs` (J) and
`usize max_speculative_directories` (K). J counts queued + running inspections.
K counts **all** uncommitted speculative nodes, including clean, differing, and
failed ready results. One extra demand slot cannot be consumed by speculation;
K=0 and one-worker pools still progress. Actual directory-job concurrency is at
most `min(P, J, K+1)` for P pool workers. Defaults J=P, K=4P are a starting
policy, not universal tuning. Multiple walkers share workers but their J/K and
memory budgets add. Retained memory includes committed active-path child names,
up to K+1 private results/listings, and the summary, not merely depth or K bytes.

Between yields, apply only the returned record's classified immediate children,
or prune-before-replace that directory. The next call acknowledges that work
is finished. Pending siblings, their ancestors, and previously acknowledged
paths stay stable. Namespace siblings can alias storage through hard links or
mounts: do not make in-place mutations that alter comparison keys in another
pending path. Unlink-and-create avoids hard-link writes through another alias.
This is still not a snapshot or security sandbox.

The compatible ambient path remains
`walk_with_options(..., WalkOptions { execution: Execution::Parallel })` and now
uses the same frontier. It selects default limits at the first `next()` and
uses the current pool on each call (normal global fallback outside a pool).
Only constructing it inside `install` does not pin the pool. Nested/cross-pool
`install` may execute unrelated work on the calling pool; applications must
not hold reentrancy-sensitive locks across these calls.

The [accepted design](/design/concurrency/design0.gpt6am.md) and
[frontier implementation account](/design/implementation/implementation1.sol61x.md)
explain the ordering, demand reservation, seals, and test evidence. The old
[per-frame measurements](/design/performance/parallel0.sol61x.md) are historical
controls, not measurements of this scheduler. `parallel-bench` exposes a hidden
temporary predecessor control; `parallel-diagnostics` instruments physical
jobs separately from committed coverage. Neither is needed for embedding.
The [frontier measurements](/design/performance/parallel1.sol61x.md) show a
repeatable live single-root win with explicit P4/J4/K256, but also saturation
of the conservative default K, root-wide serialization, and CPU/memory/latency
costs. Limits are caller policy; enabling Rayon alone promises no speedup.

## Future enhancements

These are prospective capabilities, not current API promises.

### Coordinated multiple destinations

Plan a **streaming walk of one original and several destinations together**,
advancing their comparisons through the same ordered directory/name sequence.
Inspect the current source directory once and share that observation across the
participating destinations, rather than independently walking the full source
for each target. Keep per-destination differences, progress, and errors distinct.

This should retain live, directory-at-a-time processing: a persistent cache or
whole-source catalog is not required. The design needs bounded lookahead and a
deliberate policy for slow, stopped, or failed destinations, with source stability
defined for the shared observations. Test one, two, and several destinations to
establish where the extra coordination pays off. Multiple destinations are a
future extension; the current API compares one pair.

The [optimization comparison](/design/performance/comparison0.gpt6a.md) connects
the independent inventories and their experiment briefs. Other ideas remain
research candidates, including cross-boundary metadata reuse whose end-to-end
benefit has not yet been measured.

## Non-goals

- No mutation **in the diff core**, no rename detection (a rename is add+remove),
  no built-in ignore language/repository policy, no content hashing. Callers can
  compile their own early/late policy in a visitor.

Settled during spec review (recorded so they don't resurface as questions):
single default always-descending walk — no stat pruning, no mode flags; the consumer
drives the walk as a pull iterator in its own loop (`prune_here()` for
wholesale subtrees, `break` to stop, `into_summary()` for the aggregate);
records carry names and classification only, never stats; aggregate directory
changes live in summary `changed_dirs`, not streamed records; clean comparisons
are silent in the library (the CLI still prints its final summary).

## Development and evidence

Use the environment's configured Rust toolchain with ordinary `cargo` commands.
The project does not pin a compiler or retain a Rust 1.88 compatibility floor;
Rust 1.101 nightly is the currently configured and permitted toolchain. Record
`rustc --version --verbose` for new performance results and build comparison
controls with the same compiler. Historical reports retain their actual compiler.

```sh
cargo nextest run --workspace
cargo nextest run --workspace --all-targets --all-features
cargo test --workspace --doc
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all --check
cargo test --workspace --all-targets --all-features
```

The [implementation account](/design/implementation/implementation0.sol61x.md)
explains the state machine, summary distinction, and verified trust boundaries.
The [checked-execution account](/design/sync/checked0.sol61x.md) explains the
subsequent static checker, recursive coverage, root transitions and JW handoff.
The [performance method](/perf/README.md) and
[measured run](/design/performance/performance0.sol61x.md) separate synthetic
metadata fixtures from the supplied live checkout and node_modules tests.
