---
type: ResearchAssessment
title: "diff-dirs IO assessment: std vs async vs low-level backends, jj's patterns, generic sync crate"
description: General-IO research for the diff-dirs library — verified comparison of std blocking, Tokio fs, async-fs/smol, rustix, cap-std, traversal/parallelism options, and portable copy/reflink crates for a separate sync component; jj's working-copy IO patterns from archived source as structural reference; recommendation for architecture, backends, and the sync crate's mtime/mode/link/type/overwrite/error contract.
resource: /design/fit/io0.glm53m.md
tags: [diff-dirs, io, async, tokio, rustix, cap-std, reflink, sync, jj, performance]
status: draft
generated: { by: "model:zai-coding-plan/glm-5.3#max", at: 2026-10-18 }
updated:
  - by: "model:zai-coding-plan/glm-5.3#max"
    at: 2026-10-18
    note: Parent review corrected overstatements — tokio ReadDir chunks 32 entries per dispatch (not per next), performance comparisons reframed as unmeasured rather than "not faster", rustix claim narrowed to inspected code, own_mtime/cache-grade disclaimers added, parallel-seam claim made conditional; waltz execution map added with source references.
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
    title: Verified tokio::fs spawn_blocking model and ReadDir's 32-entry next_chunk batching (CHUNK_SIZE const)
  - resource: "jj-waltz:src/cli.rs"
    title: Reseed CLI dispatch — run_reseed/run_refill into reseed::run_with_diagnostics, all synchronous
  - resource: "jj-waltz:src/metadata.rs, jj-waltz:src/manager/ui.rs"
    title: The only std::thread::spawn sites in src/ — cfg(test) metadata races and the interactive manager UI, neither on the reseed path
  - resource: "jj-waltz:Cargo.lock"
    title: Transitive check — crossterm/ratatui linked via manager UI only; no rayon/tokio in the graph; reflink-copy/fs2/tempfile spawn nothing
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
- **Dependency allocation** (`lib/Cargo.toml`): `rayon` (walk), `rustix` — in
  the code inspected for this assessment it appears only for `flock`
  (`lib/src/lock/unix.rs`); other modules were not audited and no broader
  claim is made — `tokio` **optional, behind the `watchman` feature only**,
  `gix` optional (git backend). No walkdir, no jwalk, no `ignore` crate;
  gitignore semantics come from `gix-ignore` through jj's own `GitIgnoreFile`
  chaining.

Applicability to a generic paired diff: the *per-directory job granularity*,
*sorted-merge determinism boundary*, *first-error-wins channel design*, and
*(type, mtime, size) cleanliness key* all transfer. The async surface does not
(jj needs it for object-store writes; a two-sided stat walk has no async
boundary worth bridging). The `own_mtime` guard also has no transferable
analog, and jj's cache discipline is a *different grade* of assurance, not a
coarser version of ours: jj compares disk state against a persisted cache
record and uses the cache file's own mtime to bound when its entries could
have changed; diff-dirs compares two live trees with no persisted state. The
two solve different problems; nothing here broadens one into the other.

## IO model comparison for the walk core

The workload: per directory pair, `getdents64` on both sides plus one
`stat`-class call per entry per side, `readlink` per both-sided symlink, zero
content reads. 192k entries ≈ 400k syscalls; measured 1–2s two-sided warm on
btrfs (spec; consistent with the recorded 0.24s single-tree enumeration).

| Option | What it really is (verified) | Fit |
| --- | --- | --- |
| **std blocking** (`read_dir`, `DirEntry::metadata`, `read_link`) | Direct syscalls; `ReadDir` iterates `getdents64` with internal buffering; sequential | **Recommended core.** Deterministic by construction, zero deps, embeds anywhere. The syscall census is the workload — nothing to offload. |
| **tokio::fs** | `read_dir`/`copy`/`metadata` dispatch to the `spawn_blocking` thread pool ("ordinary blocking file operations behind the scenes" per module docs); `ReadDir` dispatches one blocking job per *chunk* of 32 entries (`next_chunk`, `CHUNK_SIZE = 32`), not per `next()` | Same syscalls, on a pool. Whether pool concurrency improves wall time for this stat-bound walk is **unmeasured here** — plausible overlap exists (kernel dentry/inode cache lookups and IO submission can overlap across cores) and we make no "not faster" claim. The verified costs are structural: a mandatory runtime plus `Send` plumbing through a pull-iterator/`prune_here` design, inherited by a sync embedder that has no other async work. |
| **async-fs 2.2.0 (smol ecosystem)** | Offloads to the `blocking` crate's thread pool (crate docs, line 7) | Same model as tokio with a smaller crate; still blocking-under-the-hood, still a runtime dependency. Relative performance unmeasured. |
| **rustix 1.1.4** | Raw syscall wrappers: `openat`, `statx`, `openat2` (fs features verified) | A *targeted* optimization, not a backend: dirfd-relative `statx` batches could shave path-resolution cost per entry and request reduced statx masks. Linux-centric API. Worth a `backend` seam, not v1 — std is already inside the measured budget. |
| **cap-std 3.4.5** | Capability `Dir` (openat-relative `open`, `open_dir`, `read_dir`, `symlink_metadata`, `create_dir`, `canonicalize` — verified) | Confinement-oriented: paths resolved relative to a held directory fd, no `..` escape. jj-waltz already owns confinement (`relative()`, `confined_parents`, mount checks); the generic library deliberately doesn't. Interesting later for a sandboxed consumer; not v1. |
| io_uring-based crates | Not present in the local registry cache; not assessed | Linux-only, thread-per-core models, young APIs. Out of scope for a portable v1; noted for completeness. |

**Honest syscall model (corrected)**: the async *facades* inspected — tokio::fs
and async-fs — execute blocking syscalls on thread pools; neither eliminates or
transforms the syscalls themselves, and this document previously overclaimed
that as "no async filesystem in mainstream Rust" and "async buys nothing".
Both statements are withdrawn: io_uring-based crates exist outside the local
registry cache (unassessed), and thread-pool concurrency may help even warm
syscall-bound work — that is a measurement question, not a structural one. What
the verified evidence *does* support: entry enumeration is chunk-buffered on
both paths (libc `readdir` buffering under std; tokio's 32-entry chunks), so
the dominant per-entry cost in either model is the stat-class call; and the
async options structurally require a runtime the sync embedder would carry for
no other use. Deterministic record order must be re-imposed at a sorted merge
under any pool concurrency. Parallelism, if adopted, is better added the way jj
did it (rayon over bounded per-frame jobs + ordered merge) than by making the
API async — see [the Rayon assessment](/design/fit/rayon0.glm53m.md).

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

## The waltz execution map (source-referenced)

Follow-up requested by the parent; every claim carries a file/line reference
from the inspected checkout. This is the mechanical-sympathy baseline the Rayon
assessment builds on.

**CLI → engine.** `main.rs` parses via clap; `cli.rs:477` dispatches
`Commands::Reseed(cmd)` → `run_reseed` (`cli.rs:551`) → `run_refill`
(`cli.rs:586`) → `reseed::run_with_diagnostics` (`reseed.rs:101`) →
`engine::run` (`engine.rs:111`) → `run_inner` (`engine.rs:173`). All one
thread, one process, synchronous.

**Multi-target scheduling.** Planning walks the selected names sequentially
(`engine.rs:362–531`), one `Plan` per target; execution is a strictly
sequential loop `for mut plan in plans { execute(&default, &mut plan) }`
(`engine.rs:561–589`) with per-target timing. No target overlap, no batching
across targets.

**Process adapter.** Every jj interaction is a synchronous child:
`JjClient::execute` (`jj.rs:624–632`) is `Command::new("jj")…output()` — one
process at a time, stdout/stderr piped and fully buffered. `run_at`
(`jj.rs:333–355`) prepends `--at-operation` + `--ignore-working-copy`.
`IgnoreRules::is_ignored` (`cow/payload.rs:84–110`) likewise spawns one
`git check-ignore` child **per queried path** (bounded by candidate/root
count, not tree size).

**Walker/file-copy loops (payload path).** All recursive, single-threaded,
std-only: pattern-selection walk (`cow/ignored.rs:276–318`); donor payload
validation `validate_payload_root`/`validate_directory` (`cow.rs:246–284`);
`clone_files` per-file loop — reflink + `preserve_mode` +
`preserve_modified_time` per entry (`cow.rs:166–197`); `clone_tree` recursion
— `read_dir` + `symlink_metadata` + reflink/symlink per entry
(`cow.rs:349–379`); `remove_path` = `validate_tree` walk + `ensure_removable`
walk + `fs::remove_dir_all` (`cow.rs:381–434`) — **three traversals of the
payload subtree per removal**, on top of `validate_contents` at execute start
(`engine.rs:1387`) and the planning-time donor walk: today's re-seed crosses
the payload trees roughly five to six times before the wholesale copy.

**Reflink backend.** reflink-copy 0.1.30 issues the FICLONE/clonefile ioctl
directly; it spawns no threads. fs2 (locks) and tempfile spawn nothing.

**Threads/concurrency anywhere?** In `src/`, `std::thread::spawn` appears
only in `#[cfg(test)]` metadata tests (`metadata.rs:830, 864`) and the
interactive manager UI (`manager/ui.rs:292`) — neither is on the reseed path.
ratatui/crossterm are linked only through `manager/ui.rs`. The dependency
graph contains no rayon, no tokio, no thread-spawning transitive on the reseed
path. **The entire reseed execution is strictly sequential: one thread, one
child process at a time, one directory entry at a time.**

**Existing batching.** Tree queries are batched into single jj invocations
(`file list` with a template emitting every path, one `diff` for changed
paths, `engine.rs:187–219, 808–838`); symlink capability proofs are chunked at
128 paths per child (`engine.rs:866`); directory enumeration is buffered in
libc `readdir` under std (and in 32-entry chunks under tokio). Per-entry stat
calls and per-file reflink/chmod/utimensat are never batched — syscalls per
entry, serially.

**Dominant recorded costs.** Child-process census dominates the tracked-tree
phases (measurement0: 428 jj + 246 git children per two-worker reseed at the
8,555-file fixture; ~20–60 ms each); the payload phase is dominated by its
~5–6 full traversals plus wholesale reflink+inode creation (0.325 s per 8,700
files recorded → ≈7 s extrapolated at 192k) — the churn diff-dirs exists to
remove. The delta walk itself (1–2 s @192k two-sided) is small against these,
which frames the open performance question: parallelism inside the walk only
matters once the surrounding churn is gone.

## Embedder runtime fit

jj-waltz today: sync, single-threaded, no tokio/rayon, unix-only payload path
(`cfg(unix)` cow module), per the execution map above. A zero-dep std diff
core embeds without any runtime or dependency consequence. The sync crate adds
only `reflink-copy` on unix — already in jj-waltz's dependency tree — and
nothing on Windows beyond std. A tokio- or async-fs-based core, by contrast,
would make a runtime a *mandatory* dependency of every embedder — that is a
verified structural cost, deliberately not dressed up as a performance claim;
the relative speed of pool-backed versus inline execution for this workload is
unmeasured and is exactly what the optional rayon work will measure.

## Structural alignment with jj — adopt / decline

**Adopt** (cheap, proven in jj's tree, already mostly in the spec): the
`(type, mtime, size)` cleanliness *key shape* (jj: `FileState`); first-error
-wins with path context; per-directory job granularity with a sorted-merge
determinism boundary as the parallel seam; mtime restoration on the apply side
so the next compare is clean. Two precise disclaimers: jj's cached-state
discipline (including the `own_mtime` guard and its millisecond-resolution
`MillisSinceEpoch`) is a different assurance grade from a two-live-tree
compare, not a worse version of it — adopting the key shape does not adopt
that discipline; and restoring mtimes is a *consumer-side* apply discipline,
not an analog of jj's snapshot cache.

**Decline**: jj's async `snapshot()` surface (object-store driven; we have no
async boundary); `RepoPath`/tree/protobuf domain types (jj-shaped, not
generic); fsmonitor/watchman shortcut hooks (spec: no subtree shortcuts — and
jj-waltz *refuses* watchman workspaces precisely because such caches lie);
rustix-for-traversal (jj itself doesn't use it for walking either — only
flock).

## Recommendation

1. **Core (`diff-dirs`)**: std-only, zero dependencies, sequential
   materialize-sort-merge paired DFS behind the committed contract. No tokio,
   no async-fs, no rustix/cap-std, no walkdir/jwalk/ignore. Optional rayon is
   designed in [the Rayon assessment](/design/fit/rayon0.glm53m.md) — feature-
   gated, ambient-pool by default. Whether a parallel backend is even an
   *internal-only* change is **conditional**, not automatic: it holds only if
   parallel work is confined to gathering a frame's two listings/stats inside
   `next()` (before that frame's record is yielded) and never speculates below
   records the consumer may still prune or wholesale-replace, since a
   contract-abiding consumer mutates only yielded records' immediate children.
   Within that confinement the public contract (record order, prune, error
   semantics) is unchanged; outside it, it is not.
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
