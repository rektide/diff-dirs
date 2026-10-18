---
type: ImplementationAccount
title: Separate filesystem synchronization with staged leaves and joined caller-pool work
description: Implemented root-bound record application, whole-root initialization, explicit copy outcomes, metadata and permission discipline, convergent partial failures, and proportionate Rayon integration.
resource: /design/sync/implementation0.sol61x.md
status: implemented-verified-awaiting-parent-review
generated: { by: "model:openai/gpt-6.1-sol#xhigh", at: 2026-10-18 }
sources:
  - resource: "opencode://ses_eb3033c79ffeRIkn14uSNhs7O6"
    title: Built and verified the optional sync crate, discriminating pool and filesystem tests, then measured owned nonempty btrfs workloads
  - resource: /design/sync/brief0.gpt6a.md
    title: Selected filesystem synchronization scope and copy/application contract
  - resource: /README.md
    title: Read-only comparison semantics and frontier application guarantees
  - resource: /src/model.rs
    title: Public DirDiff classification and opaque-subtree count units
  - resource: /src/pair.rs
    title: Native literal symlinks, exact available SystemTime, and paired type transitions
  - resource: /src/walk.rs
    title: Caller-owned serial iterator and committed-prefix summary
  - resource: /src/parallel.rs
    title: Joined borrowed-pool walker and explicit J/K limits
  - resource: /design/implementation/implementation1.sol61x.md
    title: Cached sibling stability, divergence seals, and no work between yields
  - resource: /design/performance/parallel1.sol61x.md
    title: Existing concurrency evidence and limits, not a sync measurement
  - resource: "file:///home/rektide/src/jj-waltz/src/cow.rs"
    title: Read-only reference for reflink, mode/mtime, postorder permissions, and scoped removal
  - resource: "file:///home/rektide/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/reflink-copy-0.1.30/src/lib.rs"
    title: Verified absent-destination clone, truthful fallback result, preserved errors, and Windows implementation
  - resource: /sync/src/apply.rs
    title: Implemented shared serial/Rayon filesystem operation kernel
  - resource: /sync/tests/sync.rs
    title: End-to-end fixture tests and independent post-sync comparisons
---

# Filesystem synchronization checkpoint

## Scope and API

Added `sync/`, package `diff-dirs-sync`, as a consumer of the existing core.
The core source and dependency edges are unchanged. No runtime split, custom
scheduler, Tokio driver, mutation CLI, jj-waltz change, ignore/admin policy,
or automatic prune/reconciliation heuristic was introduced.

The [maintained sync specification](/sync/README.md) owns executable examples:

- `Applier::new(source,target,CopyPolicy)` binds validated roots without writes.
- `apply(&DirDiff)` supports an existing caller-owned loop; `initialize()`
  explicitly copies a whole absent target whose immediate parent exists.
- `mirror(source,target,CopyPolicy)` drives comparison and fully applies each
  record before acknowledging it with another `next()`.
- Feature `parallel`: `apply_in_pool`, `initialize_in_pool`, and
  `mirror_in_pool(...,&ThreadPool,ParallelLimits)` borrow the application's pool.

Root binding checks real directories and existing ancestors, rejects symlinks,
canonicalizes the deepest existing target ancestor, and rejects equal/nested
roots even when the destination is absent. Per-record validation checks relative
directory components, single native names, duplicates, category masks, existing
real directory ancestors, and actual source/target immediate types **before
mutation**. The two intentional directory/leaf transition masks are accepted.
One-sided subtree descendants are validated during their actual traversal.

These are stable-tree/nonhostile-alias checks, not a capability sandbox or
bind-mount detector. Existing ancestors and pending sibling names must not be
concurrently replaced. Leaf hardlinks are safe because the old inode is neither
written nor chmodded. Source/target ownership, hardlink topology, mounted
directory aliases, repository policy, and quiescence remain caller concerns.

## Shared operation kernel

The execution choice is a small internal boolean, not an executor trait or new
task abstraction. Both entry paths call the same validated named operations,
recursive copier, removal, staging, and permission finalizers. A record's names
are sorted and combined so same-name type transitions cannot run independently.
Rayon's indexed `par_iter` lazily splits current siblings. Recursive copies
reuse that pool; directories precede children, final chmod follows joined work.
Removal recursively visits its unit serially, while independent record units
may run concurrently. No detached work runs across a record boundary.

An atomic stop flag skips not-yet-started siblings after failure where feasible.
Started operations drain; indexed results are merged in native-name order.
That deterministically selects among encountered errors, but cancellation means
the encountered set can vary. Finalizers and private cleanup still run after
failure. All operation/path/original causes are retained, including secondary
joined and cleanup errors. There is no exact serial-prefix mutation claim.

The pool bounds active workers, not bytes/pending work. Current sorted sibling
names and result slots, recursively retained parent listings, the diff's bounded
lookahead, and core summary all cost memory. No whole-tree work plan is built.
Core J/K remain separately configurable; no apply admission semaphore or extra
worker setting competes with the supplied pool.

## Leaf safety and metadata

Every regular leaf is copied/cloned at an absent child of a private temporary
directory in the target parent. `Copy` uses `io::copy`; strict `reflink-copy`
does not fall back; fallback policy retains that crate's error distinctions and
`None` versus copied-byte result. No source probe file or write occurs.

Copied mode and exact supported mtime are finalized before rename. Ordinary
copy retains its descriptor throughout metadata application. The reflink crate
has already applied source permissions when it returns, so its newly owned
staging inode is temporarily made usable **before** opening it, then the open
descriptor survives restrictive final chmod. Mtime is read back before
installation; an unrepresentable timestamp is an explicit failure.

Staged symlinks preserve literal native contents. Changed leaf preparation
failure preserves the old leaf. A directory-to-leaf transition prepares before
removing the old directory. Leaf-to-directory transitions prepare the entire
new subtree before unlinking the old leaf, so a later strict clone failure does
not destroy that old leaf either. Once removal begins these sequences are not
transactions; failure/retry can leave a different delta. Cleanup only owns its
private temporary directory and retains cleanup errors.

On Unix, target directories gain owner `rwx` only when needed; surviving
existing directories regain their previous modes after joined work, including
failure. New directories get source modes postorder, even after a child error.
Only source-readable/searchable trees are copyable. Paired target ancestors must
be searchable for comparison/validation. No existing leaf is chmodded to force
removal, which avoids mutating hardlinked aliases and means some Windows
readonly-leaf operations fail. Windows link kind and directory-readonly handling
are cfg-based best effort, not exercised platform evidence. The dependency does
have a Windows reflink implementation; its own testing caveat is retained.

The comparison contract excludes mode-only drift, directory mtimes, ACLs,
ownership, xattrs, and hashing. No metadata sweep repairs those omissions.
This preserves convergence under core keys, not `rsync -a`, crash durability,
tree transactions, or content correctness for stat-preserving edits.

## Progress and verification

`Progress` counts visible completed installations/removals (including recursive
children), logical copied/cloned bytes, and directory permission calls. Private
preparations discarded before installation are excluded. A failed visible
subtree can retain some completed children. `Report.observed` is the original
diff summary, absent for whole-root initialization, not a verification scan.
Failure includes completed work and the committed comparison prefix when one
exists. Retry starts a fresh comparison, not a replay of stale public records.

Tests exercise independent post-sync comparisons, not a second implementation
of the copier:

- delta direction, exact deletions, recursive opaque counts versus diff counts,
  initialization, manual application, and zero-write repeated synchronization;
- all six file/directory/symlink transitions, literal/dangling links, native
  non-UTF-8 names, external link targets, and ordinary `.git`/`.jj` payloads;
- invalid paths, duplicate/all contradictory category masks, stale actual
  types, root overlap, missing overlapping descendants, and ancestor links;
- unchanged hardlink aliases (bytes/mode/mtime) after leaf replacement; no-op
  target inode/mode/mtime/ctime observation, not merely self-reported counters;
- readonly files/directories/removals, preserved target directory modes, exact
  nanosecond/pre-epoch mtime, and deliberate no-repair of mode-only drift;
- an isolated child process with umask `0777` and an unreadable source listing:
  failure still reports visible directory creation plus temporary relaxation
  and finalization, without changing the shared test runner's umask;
- required nonempty real btrfs reflinks and independent later edits, tmpfs→btrfs
  strict failure preserving old leaves, truthful fallback copy/clone outcomes,
  no staging residue, failed transition preparation, partial opaque copy/retry;
- P1/P2/P4 and K0/K1/K16 progress, calls inside the supplied pool, and actual
  distinct named workers rendezvousing before both record-sibling and nested
  subtree file operations. A deleted-source error must not return while an
  already-started sibling is held at a finite-time test gate; its completed
  copy is reported after joining, and retry converges.

CoW tests may report unavailable capability on other test filesystems; setting
`DIFF_DIRS_SYNC_REQUIRE_REFLINK=1` makes real clone verification mandatory.
That setting was used for actual all-feature Rust 1.88 verification here.
The final checkpoint passed **43 default workspace tests** and **68
all-feature/all-target tests**, with zero skipped tests, under actual Rust 1.88.
Default/all-feature workspace clippy passed with `-D warnings`, formatting
passed, and actual Rust 1.88 compiled all four default / five all-feature
doctests (including the three sync API examples). The benchmark example is
uniquely named `sync_perf`, without workspace output collisions. Core-alone
default dependency tree still contains only `diff-dirs`; default sync has
reflink/tempfile dependencies but no Rayon. Seven maintained/evidence documents
were checked for session provenance and local file-link existence; the 60
numeric CSV observations were checked unchanged against the original output.
The renamed final benchmark also passed a small owned-fixture smoke run across
all four policy/execution configurations and five scenarios (20 observations,
not included in the 30k performance table).

Code checkpoints: `6480fd6c` (applier and integration tests), `276c4794`
(explicit real-reflink enforcement and nested worker evidence), `c691d6be`
(early-listing failure permission progress), and `9a8e35ca` (isolated-umask
public-API regression).

## Measurements and review limits

[Owned-fixture measurements](/design/performance/sync0.sol61x.md) retain 60 timed
observations on 30,000 nonempty 4 KiB files, separate from fixture setup. Clean
repeat sync costs the walk with zero completed writes; sparse application copies
two leaves, not the entire tree. Pool fresh copying overlaps real filesystem
work; tiny-file strict reflinking is not intrinsically faster than ordinary copy.

The source live roots and jj-waltz remained read-only. No end-to-end consumer
speedup, Windows/macOS execution, content hashing, physical extent accounting,
strict byte-memory bound, or finite stalled-I/O latency is claimed.
