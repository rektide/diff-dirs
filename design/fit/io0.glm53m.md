---
type: ResearchAssessment
title: "diff-dirs IO assessment: std vs async vs low-level backends, jj's patterns, generic sync crate"
description: General-IO research for the diff-dirs library — verified comparison of std blocking, Tokio fs, async-fs/smol, rustix, cap-std, traversal/parallelism options, and portable copy/reflink crates for a separate sync component; jj's working-copy IO patterns from archived source as structural reference; recommendation for architecture, backends, and the sync crate's mtime/mode/link/type/overwrite/error contract.
resource: /design/fit/io0.glm53m.md
tags: [diff-dirs, io, async, tokio, rustix, cap-std, reflink, sync, jj, performance]
status: draft
generated: { by: "model:zai-coding-plan/glm-5.3#max", at: 2026-10-18 }
sources:
  - resource: opencode://ses_eb39c53ccffeOncs3vs6bocqfU
    title: IO-research session — verified cached/archive crate APIs, inspected jj's working-copy source, compared IO models, wrote this assessment
  - resource: /README.md
    title: The diff-dirs specification — walk contract the IO layer must serve
  - resource: /design/fit/fit0.glm53m.md
    title: Sibling fit assessment — embedder seam (corrected), dependencies, README contradictions
  - resource: "jj-waltz:Cargo.toml"
    title: Embedder dependency set — no tokio/rayon/jj-lib; subprocess-only jj integration; MSRV 1.88, edition 2024
  - resource: "jj-waltz:src/jj.rs"
    title: JjClient — run/run_at subprocess adapter pinning --at-operation/--ignore-working-copy; proves CLI integration, not library embedding
  - resource: "archive:jj-vcs/jj@803ef00a0a6ee4ddc275d3138d5f9e7326399906 (2026-09-27)"
    title: Archived Jujutsu source — lib/src/local_working_copy.rs snapshot walk, FileState cache, rayon/mpsc structure; lib/Cargo.toml dependency allocation; lock/unix.rs rustix usage
  - resource: "crates.io cache:tokio-1.53.3 src/fs/{mod.rs,read_dir.rs,copy.rs}"
    title: Verified tokio::fs delegates every operation to spawn_blocking thread pools — ordinary blocking file operations behind the scenes
  - resource: "crates.io cache:async-fs-2.2.0 src/lib.rs"
    title: Verified async-fs offloads blocking I/O onto the `blocking` crate's thread pool
  - resource: "crates.io cache:rustix-1.1.4 src/fs/{at.rs,statx.rs,openat2.rs}"
    title: Verified rustix fs surface — openat, statx, openat2 raw syscall wrappers
  - resource: "crates.io cache:cap-std-3.4.5 src/fs/dir.rs"
    title: Verified cap-std Dir — openat-relative open/open_dir/read_dir/symlink_metadata/create_dir capability surface
  - resource: "crates.io cache:walkdir-2.5.0 src/lib.rs"
    title: Verified walkdir — WalkDir::new/min_depth/sort_by_file_name; single-sided, no parallelism
  - resource: "crates.io cache:ignore-0.4.32 src/walk.rs"
    title: Verified ignore crate — WalkBuilder::build_parallel/threads as parallel-walk precedent (gitignore semantics are a non-goal here)
  - resource: "crates.io cache:filetime-0.2.29 src/lib.rs"
    title: Verified filetime — set_file_times/set_file_mtime/set_symlink_file_times
  - resource: "crates.io cache:fs_extra-1.3.0 src/lib.rs"
    title: Verified fs_extra — copy_items/copy_items_with_progress; coarse-grained, not a fit
  - resource: "crates.io cache:rayon-1.12.0"
    title: Presence verified in cache; API usage read inside jj's snapshot code rather than in isolation
  - resource: "crates.io cache:reflink-copy-0.1.30 src/lib.rs"
    title: Verified reflink/reflink_or_copy and create_new destination semantics (also cited in fit0)
---

`jj-waltz:` paths denote the sibling checkout `/home/rektide/src/jj-waltz`
(read-only). `crates.io cache:` paths denote
`~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/<crate>-<version>`,
read in place. The jj archive was located per workspace convention at
`~/archive/jj-vcs/jj` (git revision in the sources above); `~/src/jj` exists
but is empty; `~/wiki/jj-vcs/jj` was not needed for this assessment.

# diff-dirs IO assessment

## Situation and ask

The parent clarified the library is **generic**, not jj-waltz-specific: the IO
research must weigh the full option space (std blocking, Tokio fs, async-fs /
smol, rustix, cap-std, traversal libraries, parallelism) rather than only
bespoke-walker dependencies. A **copy/sync component is now authorized but must
live in a separate crate or feature** — never integral to the diff core — and
must stay generic (no jj concepts), with consumers free to retain custom
policy. The user also asked whether jj-waltz *embeds* jj and whether diff-dirs
should be structurally sympathetic to jj's IO patterns. That question is
answered first, then the option space, then the recommendation.

## Is jj embedded, and what does "sympathy" mean here?

**jj-waltz does not embed jj.** `jj-waltz:Cargo.toml` carries no `jujutsu`/
`jj-lib` dependency; `jj-waltz:src/jj.rs` wraps a `jj` **CLI child process**
(`run`/`run_at` build argument vectors; `run_at` pins `--at-operation` plus
`--ignore-working-copy` so queries never snapshot a working copy). The
performance history in jj-waltz's design docs is literally a census of jj child
processes. So there is no shared runtime to be compatible with — "sympathy"
can only mean **structural alignment with jj's source-level IO patterns**
(readable in the archive), not runtime coexistence. That distinction shapes
everything below: diff-dirs is free to choose its own backend, and jj's code is
a reference implementation to learn from, not a constraint.

## jj's working-copy IO patterns (archived source, verified)

From `lib/src/local_working_copy.rs` at the archived revision:

- **Traversal is synchronous std**, parallelized with rayon: `visit_directory`
  does `fs::read_dir` → collect to `Vec` → `into_par_iter()` with
  `.with_min_len(100)` ("Don't split into too many small jobs"), and
  subdirectories spawn into the same `rayon::scope`, used explicitly "as a
  queue of per-directory jobs".
- **Results flow through std mpsc channels** (tree entries, file states,
  untracked paths, deleted files), with **first-error-wins** via an
  `OnceLock<SnapshotError>` checked before each job — the walk short-circuits
  without unwinding.
- **Determinism is imposed at the merge boundary**: collected file states are
  `sorted_unstable_by(path)` before merging into the tree state. Parallelism
  lives *inside* the walk; the output order is still total.
- **Stat cache**: `FileState { file_type, mtime: MillisSinceEpoch, size }`
  persisted in `tree_state`; cleanliness is
  `new_file_state.is_clean(current) && current.mtime < tree_state.own_mtime` —
  the classic guard against recording a state file whose entries may have
  changed after it was written. Note the cache is **millisecond** resolution.
- **`async fn` only at the store layer**: `snapshot()` is async because tree/
  object writes are async; the filesystem traversal itself runs inside a
  synchronous closure scope, bridging with `pollster`'s `.block_on()`.
- **Dependency allocation** (`lib/Cargo.toml`): `rayon` (walk), `rustix` — used
  only for `flock` in `lock/unix.rs`, not traversal — `tokio` **optional,
  behind the `watchman` feature only**, `gix` optional (git backend). No
  walkdir, no jwalk, no `ignore` crate; gitignore semantics come from
  `gix-ignore` through jj's own `GitIgnoreFile` chaining.

Applicability to a generic paired diff: the *per-directory job granularity*,
*sorted-merge determinism boundary*, *first-error-wins channel design*, and
*(type, mtime, size) cleanliness key* all transfer. The async surface does not
(jj needs it for object-store writes; a two-sided stat walk has no async
boundary worth bridging). The `own_mtime` guard has no analog — jj compares
disk against a recorded cache file; diff-dirs compares two live trees, so the
race it guards cannot occur (donor pinning is the embedder's concern).

## IO model comparison for the walk core

The workload: per directory pair, `getdents64` on both sides plus one
`stat`-class call per entry per side, `readlink` per both-sided symlink, zero
content reads. 192k entries ≈ 400k syscalls; measured 1–2s two-sided warm on
btrfs (spec; consistent with the recorded 0.24s single-tree enumeration).

| Option | What it really is (verified) | Fit |
| --- | --- | --- |
| **std blocking** (`read_dir`, `DirEntry::metadata`, `read_link`) | Direct syscalls; `ReadDir` iterates `getdents64` with internal buffering; sequential | **Recommended core.** Deterministic by construction, zero deps, embeds anywhere. The syscall census is the workload — nothing to offload. |
| **tokio::fs** | Every operation (`read_dir`, `next`, `copy`, `metadata`) is dispatched to the `spawn_blocking` thread pool; module docs state plainly "Tokio will use ordinary blocking file operations behind the scenes" | No syscall advantage for a stat-bound walk; costs a runtime, `Send`/lifetimes plumbing through the iterator/`prune_here` design, and a dependency the embedder (a sync CLI with no tokio) would inherit. Only pays when composing with other genuinely-async work (sockets, timers), which neither diff-dirs nor jj-waltz has. **Not async IO — honestly not faster.** |
| **async-fs 2.2.0 (smol ecosystem)** | Offloads to the `blocking` crate's thread pool (crate docs, line 7) | Same story as tokio with a smaller crate; still blocking-under-the-hood, still a runtime dependency. No advantage. |
| **rustix 1.1.4** | Raw syscall wrappers: `openat`, `statx`, `openat2` (fs features verified) | A *targeted* optimization, not a backend: dirfd-relative `statx` batches could shave path-resolution cost per entry and request reduced statx masks. Linux-centric API. Worth a `backend` seam, not v1 — std is already inside the measured budget. |
| **cap-std 3.4.5** | Capability `Dir` (openat-relative `open`, `open_dir`, `read_dir`, `symlink_metadata`, `create_dir`, `canonicalize` — verified) | Confinement-oriented: paths resolved relative to a held directory fd, no `..` escape. jj-waltz already owns confinement (`relative()`, `confined_parents`, mount checks); the generic library deliberately doesn't. Interesting later for a sandboxed consumer; not v1. |
| io_uring-based crates | Not present in the local registry cache; not assessed | Linux-only, thread-per-core models, young APIs. Out of scope for a portable v1; noted for completeness. |

**Honest syscall model**: in mainstream Rust there is no async filesystem —
tokio::fs and async-fs both execute blocking syscalls on thread pools. The
walk's cost is syscall count × latency, which thread pools only help if the
kernel work actually parallelizes (cold cache, network filesystems) — and
deterministic record order must then be re-imposed at a sorted merge anyway.
Parallelism, if ever wanted, is better added the way jj did it (rayon over
directory jobs + sorted merge) than by making the API async.

## Traversal libraries and parallelism

- **walkdir 2.5.0** (verified: `WalkDir::new`, `min_depth`,
  `sort_by_file_name`): single-sided sorted traversal, portable, battle-tested.
  A paired two-sided merge-join is not what it does; adopting it would still
  require both listings materialized and sorted — which std does in three
  lines. No value for the core.
- **jwalk**: not in the local registry cache (verified absent); parallel
  one-sided walker with load-balanced streaming. Parallelism is redundant while
  the core is sequential, and its streaming order is not contractual for a
  paired diff. Not recommended; API would need fetching before any future
  reassessment.
- **ignore 0.4.32** (verified: `WalkBuilder::build_parallel`, `threads`):
  parallel-walk + gitignore machinery. Ignore semantics are a spec non-goal;
  its `WalkParallel` design (job per directory, visitor callbacks, unordered
  completion) is useful *precedent* for a future parallel backend — jj's
  rayon-scope version of the same idea is closer to what we'd copy.
- **rayon 1.12.0** (in cache; usage read inside jj): the pragmatic parallelism
  layer if benchmarks ever justify one — `scope`-spawned per-directory jobs,
  `with_min_len` batching, results sorted before emission. Design the core so
  this slots in behind the existing sort/merge boundary without API change.

**Recommendation**: v1 sequential std; keep the per-frame
materialize-sort-merge structure (it *is* the parallel seam); document that a
rayon backend is an internal change, not an API change.

## The generic sync crate (separate crate, not a feature of the core)

Authorized scope: turn records into filesystem changes, generically — no jj
concepts, no ignore rules, no policy the consumer didn't choose. Consumers may
retain custom policy (jj-waltz keeps its confinement/probes and can ignore this
crate entirely).

**Backends** (a small trait, `clone_file(from, to)` + `copy(from, to)`):

| Backend | Crate / API (verified) | Notes |
| --- | --- | --- |
| Reflink-only | `reflink-copy::reflink` 0.1.30 | FICLONE (Linux) / clonefile (macOS). Destination is opened `create_new` — **must not exist**. No mode/mtime preservation. Unix only. jw's current strictness. |
| Reflink-or-copy | `reflink-copy::reflink_or_copy` | Generic filesystem fallback; must be **explicit opt-in** (jw refuses fallback; other consumers may want it). |
| Portable copy | `std::fs::copy` | Truncating overwrite-in-place, copies permission bits, **not** mtime; everywhere std is. |
| mtime restore | `std::fs::File::set_modified` (1.75+) | ns precision on Linux, portable. `filetime` 0.2.29 (`set_file_mtime`, `set_symlink_file_times`) only if atime or symlink-mtime ever matter — not needed now. |

`fs_extra` 1.3.0 (`copy_items`) was verified and rejected: coarse directory
copy with option structs, no reflink, no record-level control.

**Overwrite strategy**: for changed files, **unlink-then-create** (+ reflink or
copy, then `set_permissions`, then `set_modified`) — matches `create_new`
semantics, matches the spec's deletions-before-copies standing rule, and keeps
type transitions (file→dir, dir→file, symlink↔file) expressible as ordered
primitive steps within one record. Temp-file+rename buys per-file crash
atomicity the payload use case doesn't need (no fsync discipline exists there
today) at the cost of double path churn; leave it as a documented alternative,
not the default.

**Portability matrix for the sync crate**: unix — full (modes, mtime ns,
symlinks, reflink); Windows — `std::fs::copy` + `set_permissions` (readonly
bit only; mode bits have no analog), `set_modified` (100ns), symlinks via std
with privilege caveats documented, no reflink (backend falls back to copy only
under the opt-in policy); the **diff core remains fully portable regardless** —
it never mutates.

**Error and partial-apply semantics** (generic contract, mirroring the core's):
first error aborts the apply with record + path context; already-applied
entries stay applied; **apply is convergent, not transactional** — the
documented acceptance property is *clean second diff*: after a complete apply,
re-walking the same pair yields no records; after an aborted apply, re-walking
yields exactly the unapplied remainder. That property is the sync crate's
primary test surface and is what makes the jj-waltz retry story (fit0,
corrected requirements) sound.

## Embedder runtime fit

jj-waltz today: sync, single-threaded, no tokio/rayon, unix-only payload path
(`cfg(unix)` cow module). A zero-dep std diff core embeds without any runtime
or dependency consequence. The sync crate adds only `reflink-copy` on unix —
already in jj-waltz's dependency tree — and nothing on Windows beyond std. A
tokio-based core, by contrast, would force a runtime into every embedder for
zero measured benefit; an async-fs core would do the same with a smaller crate.
This is the concrete content of "no claim async inherently faster" for this
workload: the async options are thread pools around the same syscalls.

## Structural alignment with jj — adopt / decline

**Adopt** (cheap, proven in jj's tree, already mostly in the spec): the
`(type, mtime, size)` cleanliness key (jj: `FileState`; spec: comparison key —
note jj's cache is ms-resolution where our key is ns, which is finer and fine);
first-error-wins with path context; per-directory job granularity with a
sorted-merge determinism boundary as the future parallel seam; mtime-based
steadiness discipline on the *apply* side (restore donor mtimes so the next
compare is clean — jj's snapshot-cheapness analog).

**Decline**: jj's async `snapshot()` surface (object-store driven; we have no
async boundary); `RepoPath`/tree/protobuf domain types (jj-shaped, not
generic); fsmonitor/watchman shortcut hooks (spec: no subtree shortcuts — and
jj-waltz *refuses* watchman workspaces precisely because such caches lie);
rustix-for-traversal (jj itself doesn't use it for walking either — only
flock).

## Recommendation

1. **Core (`diff-dirs`)**: std-only, zero dependencies, sequential
   materialize-sort-merge paired DFS behind the committed contract. No tokio,
   no async-fs, no rustix/cap-std, no walkdir/jwalk/ignore. Document the
   rayon-behind-the-sort-seam future explicitly.
2. **Sync (`diff-dirs-sync`, separate crate)**: generic record applier; backend
   trait with reflink / reflink-or-copy (opt-in) / std-copy; unlink-then-create
   overwrite discipline; `set_permissions` + `set_modified` after every file;
   deletions-before-copies within a record for type transitions; convergent
   partial-apply semantics with *clean second diff* as the acceptance test; no
   jj concepts, consumer policy hooks only.
3. **CLI**: unchanged from fit0 (own serde_json dependency; JSONL/text, exit
   0/1/2) and may use `diff-dirs-sync` for a future `--apply` mode without
   touching the core.
4. **jj-waltz**: may adopt `diff-dirs-sync` or keep its cow primitives — its
   confinement, probes, and replace-content semantics stay sovereign either way
   (fit0's corrected integration requirements stand independently).
