---
type: ResearchAssessment
title: "diff-dirs fit: embedder seam, dependencies, contract contradictions, benchmark plan"
description: Research wave assessing design/dependency/integration fit for the diff-dirs spec against jj-waltz's live reseed payload phase; records the exact replacement seam, std-only dependency recommendation, README contradictions with minimal resolutions, a credible benchmark plan, and the choices that need the parent before a builder starts.
resource: /design/fit/fit0.glm53m.md
tags: [diff-dirs, jj-waltz, reseed, payload, reflink, integration, performance]
status: draft
generated: { by: "model:zai-coding-plan/glm-5.3#max", at: 2026-10-18 }
sources:
  - resource: opencode://ses_eb39c53ccffeOncs3vs6bocqfU
    title: Fit-research session — read the spec and the embedder's native source/docs/measurements, assessed libraries vs std, and wrote this assessment
  - resource: /README.md
    title: The diff-dirs specification under assessment (the README wins over code until deliberately revised)
  - resource: "jj-waltz:src/reseed/engine.rs"
    title: Reseed engine — the unconditional payload remove+clone loop this library replaces, plus preflight/verification boundaries around it
  - resource: "jj-waltz:src/cow.rs"
    title: CoW adapter — reflink, clone_tree, clone_files, remove_path, validate_tree, mount-boundary and mtime/mode preservation primitives
  - resource: "jj-waltz:src/cow/ignored.rs"
    title: Payload root selection walk — one compiled pattern set, outermost-root pruning, confinement policy
  - resource: "jj-waltz:src/cow/payload.rs"
    title: Frozen ignore rules, tracked-entry listing, directory identity, relative-path confinement, confined_parents
  - resource: "jj-waltz:Cargo.toml"
    title: Embedder dependency set, edition 2024, rust-version 1.88, and the dist target matrix including Windows and musl
  - resource: "jj-waltz:design/up-performance/handoff0.glm53m.md"
    title: Hours-scale up incident handoff — payload digest/stamp cost history and the delta-sync motivation
  - resource: "jj-waltz:design/up-performance/measurement0.gpt6a.md"
    title: 0.325s native reflink copy of 8,700 files; instrumented scaling baseline
  - resource: "jj-waltz:design/up-performance/measurement1.gpt6a.md"
    title: Native end-to-end timings after batching and journal work (prepare 3.17s, reseed 6.64s fixtures)
  - resource: "jj-waltz:commit 96eb99d6"
    title: Payload-walk removal measurements — 30,000-file/123MB add 3.56s→1.53s; two-worker reseed 5.93s→5.08s
  - resource: "jj-waltz:docs/reseed.md"
    title: Maintained reseed operator guide — payload replacement contract and measurement history
  - resource: "jj-waltz:.test-agent/speed/demo.py"
    title: The 30,000-file synthetic payload fixture generator shape worth mirroring for benchmarks
  - resource: "crates.io:reflink-copy 0.1.30 src/lib.rs"
    title: reflink() destination semantics — OpenOptions::create_new, so destinations must not exist
---

`jj-waltz:` source URIs above denote the sibling first-party checkout
`/home/rektide/src/jj-waltz` (read-only during this research; its uncommitted
working-copy changes were not touched).

# diff-dirs fit assessment

## Situation and ask

The [spec](/README.md) is committed; no code exists yet. The parent commissioned
this research wave before handing a builder (Sol xhigh) the implementation:
confirm the integration seam against jj-waltz's *current* reseed payload phase,
choose dependencies, pressure-test the walk/record/summary contract, surface the
README's internal contradictions with minimal resolutions, and leave a credible
benchmark plan. Scope held: no library implementation, no jj-waltz changes.

## The embedder seam today (evidence)

The payload phase lives in `jj-waltz:src/reseed/engine.rs`, `execute()`. After
sparse reset and tracked-tree materialization, for every selected ignored root
(lines ~1484–1490):

```rust
for path in &plan.ignored {
    confined_parents(&plan.root, path)?;
    cow::remove_path(&plan.root.join(path))?;          // rm -rf, validated
    fs::create_dir_all(plan.root.join(path).parent()...)?;
    confined_parents(&plan.root, path)?;
    cow::clone_tree(&plan.source.join(path), &plan.root.join(path))?; // full reflink copy
}
```

That is exactly the "blind delete-and-recopy" the spec's Positioning section
exists to prevent: cost is proportional to tree size (192k entries ≈ 8.1 GB per
workspace, per the spec) even at zero divergence. Everything around the seam is
already the right shape for an in-loop applier and stays untouched:

- **Preflight, once per run, shared by all targets**: pattern selection
  (`cow/ignored.rs`, outermost-root pruning), donor-side payload validation
  `cow::validate_payload_root` (inline admin-entry/special-file/mount checks plus
  absolute-symlink warnings), and the real-CoW proof `cow::probe_between`.
- **Identity and quiescence**: `validate_source`/`validate_identities`
  (device+inode pinning) re-checked around the payload phase; ignored roots
  re-asserted under frozen ignore rules after refill.
- **Post-conditions**: `verify_disk` tracked-tree proof and `require_full`
  unchanged — payload is ignored, so jj never sees it.

The apply primitives the spec's use case 1 names all exist as private functions
in `jj-waltz:src/cow.rs`:

| Spec phrase | jj-waltz primitive |
| --- | --- |
| reflink the donor file | `reflink_copy::reflink` + `preserve_mode` + `preserve_modified_time` (`File::set_modified`; no-op on macOS where `clonefile` preserves timestamps) |
| unlink | `fs::remove_file` / `cow::remove_path` (validated, fails closed) |
| subtree clone | `cow::clone_tree` (destination must not exist; validates inline) |
| subtree remove | `cow::remove_path` |
| confinement | `confined_parents`, `relative()`, `validate_same_mount` (dev + Linux statx mount ID) |

One concrete new need: a single-entry clone helper. `cow::clone_files` has the
right shape (relative path list, reflink+mode+mtime) but its skip semantics
(NotFound → continue, non-file → continue) are for fresh creation, not for a
delta where the donor entry is asserted. And `reflink-copy` 0.1.30 opens the
destination with `create_new` — **a changed target file must be unlinked before
reflink** (or reflink-to-temp + rename). That is an applier discipline detail in
jj-waltz, not a library concern, but the spec's standing rules should not
contradict it (see C1/C6 below — deletions-before-copies already covers it).

Measured context, cross-checked:

- The spec's 192k-entry numbers (0.24s single-tree enumeration; 1–2s two-sided
  stat compare; `rsync -n` 13.8s wall / 3.4s CPU; 15–18s payload churn inside
  38s re-seeds) are **not recorded anywhere in either repository** — they are
  the parent session's own measurements. They are plausible against what *is*
  recorded in jj-waltz: 0.325s for `cp -a --reflink=always` of 8,700 files
  (≈37µs/file → ≈7s for 192k copy, plus removal, lands in the 15–18s churn
  band), and 3.56s→1.53s for a 30,000-file/123MB `jw add --copy-ignored` after
  digest/stamp walks were removed. Recommendation: land the 192k measurement
  method (command, fixture identity, environment) into this repository before
  the builder starts, so the spec's Positioning claims are auditable.
- Two-sided stat compare at 1–2s for 192k entries ≈ 2×(readdir+stat) at a few
  µs each, warm btrfs: credible for a std-only sequential walker. No evidence
  motivates parallelism (determinism is contractual; the workload is
  syscall-bound, not CPU-bound).

## Dependencies, platforms, MSRV

**Recommendation: std-only core, zero runtime dependencies.**

- No existing crate does paired two-sided diff: `walkdir`/`jwalk` are one-sided
  walkers; `ignore` is gitignore semantics (explicit non-goal); `similar` and
  friends are content diffs. The needed loop is small: `read_dir` both sides,
  `DirEntry::metadata()` (fstatat, one stat per entry per side), sort each
  side's names by encoded bytes, merge-join, explicit DFS stack.
- mtime_ns: use `std::fs::Metadata::modified()` → `SystemTime` → nanos since
  epoch. Portable (Linux ns via statx/timespec; Windows 100ns units; macOS ns),
  which matters because the embedder's dist matrix builds `x86_64-pc-windows-msvc`
  and musl alongside the unix targets. Avoid `std::os::unix::fs::MetadataExt`
  in the core so the library compiles and behaves everywhere; jj-waltz's *apply*
  side remains unix-only by its own `cfg(unix)` cow module — unchanged.
- Symlinks: `file_type().is_symlink()` + `fs::read_link` are portable; targets
  compare as `OsStr` bytes.
- Error type: hand-rolled enum (`Io { path, source }`) with manual `Display` +
  `From<io::Error>` (~20 lines) keeps the zero-dep property; `thiserror` is the
  conventional alternative and acceptable if the parent prefers it — it costs a
  proc-macro at build time only. Either way errors must name the offending path
  (spec requirement, and jj-waltz's error style).
- **Crate layout**: workspace with `diff-dirs` (lib only, zero deps) plus a
  thin `diff-dirs-cli` crate (serde/serde_json for JSONL, clap or plain argv
  parsing). A single crate with a bin would drag CLI deps into jj-waltz's build
  via path dependency unless feature-gated; two crates is simpler to reason
  about. The spec's "path dependency vs vendoring vs publishing" stays open as
  stated; path dependency is the obvious v0 choice (both repos are local and
  co-versioned by the same operator).
- **MSRV/edition**: match the embedder — edition 2024, `rust-version = "1.88"`.
  Trivially satisfied by std-only code (`File::set_modified` 1.75,
  `OsStr::as_encoded_bytes` 1.74 are the newest APIs nearby, and those are
  jj-waltz-side anyway).

## Walk design sketch (for the builder)

Iterative paired DFS, pre-order records, post-order summary accumulation:

1. Frame per both-sided directory pair: read both `read_dir`s fully, capture
   `(OsString, file_type, metadata-if-not-dir)` per entry via
   `DirEntry::metadata()` (skips a second stat pass), sort each side bytewise
   (Unix raw bytes; `as_encoded_bytes` elsewhere — deterministic per platform,
   which is all a single-host compare needs).
2. Merge-join the sorted child lists by name; per name: both-sided
   file/symlink → compare key (`type ∈ {file, symlink}`, `size`, `mtime_ns`;
   symlinks compare `read_link` targets) → changed; one-sided → added/removed
   per the resolved direction (C1); dir-vs-non-dir → paired classification
   (C6); both-sided dirs → push child frame in sorted order.
3. Emit `DirDiff` for the directory before descending (pre-order, divergent
   subtree contiguous). `prune_here()` pops the pending descent. Errors yield
   `Err` naming the path, then the iterator is fused-None; `break` supported.
4. Summary accumulates on unwind: per-category counts, topmost-divergent set
   per the resolved definition (C4), clean/differences/error basis.

In-loop mutation safety (why this contract works for jj-waltz's applier): the
apply loop only mutates immediate children of the directory whose record was
just yielded — those child lists are already materialized in the frame, and
directories are never stat-compared, so parent-dir mtime churn from
unlink/create never affects comparison. This property is worth an explicit
sentence in the README because it is the reason "records describe immediate
children only" is load-bearing for use case 1.

## README contradictions and minimal resolutions

**C1 — direction labels are inverted between Model and Use cases (blocking).**
[Model](/README.md#model) defines *Removed = present in original, missing in
target* and *Added = target-only*. Both use cases apply the opposite:
[use case 1](/README.md#1-apply-in-loop--jj-waltz-payload-sync) reflinks donor
files for `files.changed`/`files.added` and unlinks for `files.removed`; use
case 2 copies changed/added and deletes removed. As written, `files.removed` =
"missing in target" makes its own `→ unlink` a no-op error. *Minimal
resolution*: swap two words in the Model bullet — **Added = present in
original, missing in target (a copier would put it back); Removed = target-only
(stale relative to original → a pruner would remove it)**. This keeps the
consumer-facing sections untouched and matches every consumer's actual
reconciliation direction (make target equal original). Parent should ratify
because naming preference is a judgment; the use-case reading is the only one
that composes.

**C2 — the driving example does not borrow-check (blocking).**
`for diff in walk.by_ref() { … walk.prune_here() … }` holds the `&mut` for the
loop body — E0499. Also items are `Result` per the error contract, but the
example binds `diff` bare. *Minimal resolution*: replace with
`while let Some(diff) = walk.next() { let diff = diff?; … }` — the `next()`
borrow ends at the statement, so `prune_here()` is legal, and the `?` shows the
error contract. Semantics unchanged.

**C3 — "Memory is bounded by tree depth, not level width" is false as stated.**
Each frame must materialize and sort both sides' child names (readdir order is
arbitrary), so peak memory is the sum of current-path directory widths, not
depth alone. For payload trees (depth ≈ 10, width ≤ ~10k names) this is a few
MB — fine — but the claim should be honest. *Minimal resolution*: reword to
"bounded by the sorted child listings of the directories on the current descent
path — never by total tree size; sibling widths never accumulate."

**C4 — the summary's "topmost divergent subtrees" is degenerate as defined.**
"The minimal set of both-sided directories whose subtree contains any
difference" is satisfied by `{root}` whenever anything differs (every ancestor
contains a descendant's difference), and "topmost" pulls against "minimal".
Candidate precise rules: (a) *topmost directly-divergent* — directories whose
own record has ≥1 difference and no proper ancestor's record does — computable
in post-order, coarse only when divergence starts high; (b) *wholesale
collapse* — directories whose subtree is (near-)fully divergent with no such
parent, the `git status --dir` intuition and the best "what moved" answer, but
it needs a divergence-density threshold; (c) drop it from v0 and add later.
*Recommendation*: v0 implements (a); record (b) as the candidate upgrade once
real payload audits show whether (a) is coarse enough. Parent decision.

**C5 — root-pair validation and missing-root policy are unspecified.**
`walk(original, target)?` returns `Result` but the README never says what is
checked. *Minimal resolution*: one sentence — both roots must exist as real
directories (`symlink_metadata().is_dir()`), symlinked or missing roots are
errors naming the path. A missing target root is an error, not
"whole-tree-added": jj-waltz's creation flow uses `clone_tree` for fresh
payload; reseed always has a previous generation.

**C6 — type transitions are implied but never specified.**
The standing rules ("deletions before copies … so file→dir type transitions
apply cleanly") presuppose that a file-in-original/dir-in-target name yields
paired classifications (`files.*` + `dirs.*`) rather than a descent or a
mysterious "changed". *Minimal resolution*: two sentences in Model — the
comparison key's `type` ranges over {file, symlink} (directories are handled
structurally); a name whose kinds differ across sides produces the two
one-sided classifications (one in `files`, one in `dirs`) within the same
record. Plus the C1 fix, the standing rules then read correctly.

**C7 — mode/exec-bit is absent from the comparison key (record, don't fix).**
`(type, size, mtime_ns)` misses chmod-only drift. jj-waltz preserves donor
modes on every copy, and the spec's trust model already grades stat equality
like jj's working-copy cache, so this is consistent — but it is the one cheap
addition (mode rides the same stat) if payload exec-bit drift ever matters.
Record it in the trust-model paragraph as deliberate; parent may override.

**C8 — error/termination edge behavior (one sentence each).** After an `Err`
item the iterator yields `None` forever; `into_summary()` after an error or
early `break` describes the examined portion (counts reflect examined entries;
error basis = exit 2). Also state that the walk itself never special-cases
`.jj`/`.git` or mount boundaries — "compare exactly the roots given" — because
jj-waltz's preflight (`validate_payload_root`, `validate_tree`) already
rejects those inside selected payload trees, and `remove_path` fails closed on
the target side.

## Contract-to-seam mapping (integration test surface)

- `files.changed/added` → unlink target leaf if present → `reflink` →
  `preserve_mode` → `preserve_modified_time`. Steady state closes: applied
  leaves carry donor `(type, size, mtime_ns)`, so the next walk is clean — the
  spec's "property of the apply discipline" claim checks out mechanically.
- `files.removed` → `fs::remove_file` (exact leaf; `remove_path` also works
  and fails closed on specials).
- `dirs.added` → `confined_parents` + `create_dir_all(parent)` +
  `cow::clone_tree` (wholesale primitive unchanged).
- `dirs.removed` → `cow::remove_path`.
- `prune_here()` → jj-waltz policy hook: when a record's divergence is dense,
  fall back to `remove_path`+`clone_tree` for that directory — the current
  unconditional path becomes the divergence-driven fallback. Threshold is an
  integration-time decision, not a library constant.
- Walk roots are per selected payload root (donor `source.join(p)` vs
  `root.join(p)`), not the workspace root, matching the spec's "per root pair".

## Benchmark plan

Fixture generator mirroring `jj-waltz:.test-agent/speed/demo.py`'s shape
(fanout directories × files, configurable symlink count and sizes), three
scenarios at 30k entries (matching the recorded 30k evidence) and one
192k-shaped run:

1. **Clean** (zero divergence) — walk-only wall/CPU. Target: the spec's ~1–2s
   @192k claim; this is the steady-state cost every re-seed pays.
2. **Sparse delta** (k changed/added/removed for k ∈ {10, 1k}) — walk + apply
   wall vs the current unconditional remove+clone. Target: proportional-to-
   delta behavior; sub-second at k=10.
3. **Wholesale subtree replaced** — validates the `prune_here` fallback path
   against plain remove+clone (expect parity, not regression).

Method discipline copied from jj-waltz's measurement practice: warm cache,
report filesystem/kernel/host, executable hash, n ≥ 3 with variance,
`strace -c` syscall census as the regression guard (stat count = 2×entries,
readlink count = symlinks, **zero `read()` of file contents** — that invariant
is the spec's core performance promise). `rsync -n` / `diff -r` / `cp -a
--reflink=always` as external anchors on the same fixture. Criterion only if a
micro-scale question appears (sort/merge-join); primary metrics are
end-to-end.

## Choices requiring the parent

1. C1 ratification (recommended: fix the Model bullet, two-word swap).
2. C4 summary definition (recommended: rule (a) for v0).
3. Record names: spec says `Vec<String>`; jj-waltz's clone/remove path handles
   arbitrary `OsString` names today, so a `String` record model is *stricter*
   than the status quo and would error on non-UTF-8 payload names.
   Recommended: `OsString` in records, CLI escapes lossily with a marker;
   alternatively keep `String` and specify the error. Parent call.
4. Crate layout: two-crate workspace, lib zero-dep (recommended) vs single
   crate with feature-gated bin.
5. Error type: hand-rolled (recommended for zero-dep) vs `thiserror`.
6. Integration mode for jj-waltz: path dependency (recommended v0) vs vendor
   vs publish — the spec already defers this to integration time.
7. Whether to record the 192k measurement method in-repo first (recommended;
   makes Positioning auditable) and whether mode joins the comparison key (C7).

## Builder brief (Sol xhigh)

Implement to the README as amended by C1–C8 minimal resolutions, in this order:

1. **Library core, std-only, edition 2024, rust-version 1.88**: paired DFS
   walker per the design sketch; `DirDiff` records (names +
   classification, no stats); pull iterator with `next()` returning
   `Result`, fused-None after error; `prune_here()`; `into_summary()`
   (counts + resolved topmost-divergent rule (a) + clean/diff/error basis);
   root-pair validation per C5; error enum naming paths; zero dependencies.
2. **Determinism and confinement tests**: byte-order record sequences
   (including non-ASCII and — if C3 resolves to `OsString` — non-UTF-8
   names), symlink/file/dir type transitions per C6, prune/break/error
   terminal behavior, clean-tree silence, summary accounting.
3. **Thin CLI crate**: `diff-dirs ORIGINAL TARGET [--format json|text]`,
   exit 0/1/2, JSONL via its own serde_json dependency.
4. **Benchmark fixture + harness** per the plan above; record the 192k
   measurement method alongside the results.
5. Do **not** touch jj-waltz. Integration (the seam mapped above, the
   single-entry clone helper with unlink-before-reflink, and the prune
   threshold) is a separate, parent-reviewed change on the jj-waltz side.

Verification bar for the builder: `cargo nextest` green, clippy clean,
`strace -c` shows no content `read()` during a clean-tree walk, and the
clean-walk 30k fixture lands well under a second warm (extrapolating to ~1–2s
@192k).
