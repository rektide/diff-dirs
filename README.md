# diff-dirs

A general tree-reconciliation library: compare an **original** tree (source of
truth) against a **target** tree (a copy of some earlier generation of it) and
let the caller drive the differences in their own loop. The library owns the
deterministic walk and the diff model; callers own what reconciliation means —
reflink-sync a payload, print a report, prune a mirror, or feed an audit.

This README is the specification. Implementation follows it; when the README
and code disagree, the README wins until deliberately revised.

## Positioning

- A **library first**: embedded by tools (first consumer: jj-waltz's payload
  sync) that drive the walk in their own loop and apply per-entry policy.
- A **thin CLI** wraps the same walk for standalone/human use (JSONL or text).
- It does not reconcile by itself. It produces precise, ordered, deterministic
  differences; the caller's loop applies policy.

Measured context (Linux/btrfs, warm cache, 192k-entry `node_modules`): raw
enumeration of one tree is ~0.24s; an in-process two-sided stat comparison is
~1–2s. `rsync -n` costs 13.8s wall locally (3.4s CPU — protocol overhead), so
an embedded walk beats shelling out. The expensive mistake this tool exists to
prevent is not detection cost but **blind delete-and-recopy reconciliation**.

## Model

- **Direction:** compare `original` against `target`. "Removed" = present in
  original, missing in target (target lacks it → a copier would put it back).
  "Added" = target-only (stale relative to original → a pruner would remove
  it). The library never mutates either tree.
- **Comparison keys:** files: `(type, size, mtime_ns)`; symlinks compare
  their readlink target. Directories are structure, never stat-compared —
  a both-sided subdir is always descended and its contents compared
  entry-by-entry.
- **Trust model (stated plainly):** file-level stat equality is the same
  grade as jj's working-copy cache; it does not catch an in-place byte edit
  that preserves size and mtime. No subtree-level shortcut is taken:
  directory mtimes are a name-change log (create/remove/rename of direct
  children), not a content fingerprint — pruning on them would silently
  miss divergences below unchanged dir mtimes, and the measured full walk
  (~1–2s for 192k entries) makes the shortcut pointless. Hashing is not
  offered (its cost is the thing modern consumers are deleting).

## The walk

One mode. The walk compares both trees together, deterministically:

- Entries compared in bytewise-sorted name order; the walk is **depth-first
  pre-order** (a directory's record is immediately followed by its
  subdirectories' records, so each divergent subtree streams as a contiguous
  block). Memory is bounded by tree depth, not level width.
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
    files: ChangedAddedRemoved<Vec<String>>,
    dirs:   AddedRemoved<Vec<String>>,
}
```

- Names and classification only — **no stats**. The walker performed the
  comparison; classification is the output. Consumers that reflink, unlink,
  report, or audit act on identity, not on metadata.
- A both-sided dir's "changed" status is an aggregate, known only bottom-up,
  so it lives in the post-walk **summary**, not in streamed records.
- Clean directories emit nothing; a fully clean comparison emits no records.

## Summary

After the walk, `into_summary()` returns an aggregate (computed naturally
during post-order unwinding):

- **topmost divergent subtrees**: the minimal set of both-sided directories
  whose subtree contains any difference (coarse answer to "what moved");
- per-category counts (files/dirs × changed/added/removed);
- the exit-code basis (clean vs differences vs error).

## Driving the walk

No callbacks, no traits: the walker is a pull iterator running inside the
consumer's loop.

```rust
let mut walk = diff_dirs::walk(original, target)?;
for diff in walk.by_ref() {
    // apply policy per record, in your own loop
    if take_whole_subtree(&diff) {
        walk.prune_here();          // do not descend into this directory
        continue;
    }
    // ...reflink/unlink/report per entry...
}
let summary = walk.into_summary();  // topmost divergent subtrees, counts
```

- Records arrive depth-first pre-order; each divergent subtree is a
  contiguous run of records.
- `prune_here()` — legal immediately after a record is yielded — abandons
  descent into that record's directory (wholesale-subtree handling by
  consumer choice, never by stat guess).
- `break` stops the walk entirely. Errors surface as `Result` items/`next()`
  and abort naming the offending path.
- `into_summary()` consumes the finished walk and returns the aggregate.

Both consumption styles below run over this same walk — the library makes no
distinction between them. See [Use cases](#use-cases).

## Use cases

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
payload should cost the walk (~1–2s measured at this scale) and nothing else;
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

This is also the CLI's own shape: collect records, render JSONL or text,
exit 0/1/2.

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

## Embedding

Rust crate; jj-waltz consumes it for payload sync, applying each record with
primitives it already owns — reflink per changed/added file, chmod + mtime
restore (keeping jj's stat cache valid), unlink per removed file, existing
subtree clone/remove for one-sided dirs, with `prune_here()` available for
wholesale-subtree replacement. This replaces unconditional rm+recopy of
selected roots with walk-compare-apply of the delta. Open: path dependency
vs vendoring vs publishing — decided at integration time, not by the library.

## Non-goals

- No mutation, no rename detection (a rename is add+remove), no ignore-rule
  handling (compare exactly the roots given), no content hashing.

Settled during spec review (recorded so they don't resurface as questions):
single always-descending walk — no stat pruning, no mode flags; the consumer
drives the walk as a pull iterator in its own loop (`prune_here()` for
wholesale subtrees, `break` to stop, `into_summary()` for the aggregate);
records carry names and classification only, never stats; `dirs.changed`
lives in the summary, not streamed records; clean comparisons are silent.
