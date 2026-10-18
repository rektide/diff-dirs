---
type: MaintenanceHandoff
title: "diff-dirs integration handoff: assessment, jj-waltz wiring plan, and integration-level follow-ups"
description: Parent assessment of the built workspace (core walker, sync applier, frontier, perf corpus) from the integrating consumer's seat — verified green with warm walk measurements and cold-cache variance recorded, direction semantics confirmed already-correct, the recursive guard composition documented as load-bearing, and the concrete Option-B integration plan with integration-level suggestions beyond the existing opportunity inventories.
resource: /design/fit/handoff0.glm53m.md
tags: [diff-dirs, jj-waltz, integration, handoff, reseed, payload]
status: draft
generated:
  by: "model:zai-coding-plan/glm-5.3#max"
  at: 2026-10-18
sources:
  - resource: "opencode://ses_eb8a7487affeE2smndZ914OQMS"
    title: Parent session — commissioned the spec, measured the pool costs that motivated it, assessed the built workspace, and wrote this integration handoff
  - resource: /README.md
    title: Root specification — walk, records, summary, driving model, use cases
  - resource: /sync/README.md
    title: Implemented sync contract — mirror, Applier, copy policies, staged replacement, metadata and safety boundary
  - resource: /design/index.md
    title: Design and evidence reading map (what this handoff deliberately does not re-cover)
  - resource: /design/fit/fit0.glm53m.md
    title: Prior fit assessment — corrected seam analysis (Option B) and contract contradictions C1–C8
  - resource: /design/performance/performance0.sol61x.md
    title: Controlled live-tree sequential measurements (node_modules pair median 1,033 ms warm)
  - resource: "jj-waltz:src/reseed/engine.rs"
    title: Current payload seam — early removal at the else branch, final remove+clone loop, validation boundaries
  - resource: "jj-waltz:src/cow.rs"
    title: Admin-entry, mount-boundary, and confinement guards whose composition with sync is load-bearing
  - resource: "jj-waltz:design/up-performance/measurement1.gpt6a.md"
    title: End-to-end reseed timings the integration is measured against
---

# diff-dirs integration handoff

## What was assessed

The built workspace on 2026-10-18: std-only core (`walk`, `pair`, `model`,
`error`; optional Rayon `frontier`/`parallel`), `cli/` (binary `diff-dirs`,
JSONL/text, exit 0/1/2), and `sync/` (`mirror`, `Applier`, `CopyPolicy`
including `ReflinkRequired`, staged target-side replacement, exact
permissions/mtime with readback, absent-target initialization, borrowed-pool
variants). Verification run for this handoff: `cargo nextest --workspace`
**51/51 green**; release CLI built clean; one read-only live walk executed
(see expectations below). The [design index](../index.md) owns the full
evidence corpus — parallel scheduler, leaf-pipeline measurements, and the
opportunity inventories are deliberately **not** re-covered here.

## Assessment from the integrating consumer's seat

The implementation exceeds the integration assumption in
[fit0](fit0.glm53m.md): its contract-to-seam mapping expected jj-waltz to
hand-roll the apply loop from its own `cow` primitives. `diff-dirs-sync`
now supplies that entire half — including the two details fit0 called out as
jj-waltz's job: unlink-before-reflink (solved better, via staged temp +
rename, which also never writes through destination hardlinks) and mtime
discipline (exact `SystemTime` with pre-installation readback, stricter than
`cow::preserve_modified_time`). `CopyPolicy::ReflinkRequired` failing rather
than falling back matches jj-waltz's CoW-required payload contract exactly.
The pull-iterator + `Applier` composition is precisely the "caller-owned
loop" shape the engine needs.

## Findings requiring decisions or adjustment

1. **README direction semantics — verified already correct (corrects this
   handoff's first draft).** The first draft claimed fit0's C1 inversion was
   still live; re-reading the current Model section shows the builders landed
   the amendment: "Added" = original-only (copy to target), "removed" =
   target-only (delete from target), agreeing with code, use cases, and
   sync. No action required.
2. **Warm versus cold walk expectations.** Controlled live measurement
   ([performance0](../performance/performance0.sol61x.md)): paired
   `node_modules` walk, 22,828 directory pairs, **median 1,033 ms** warm,
   n=5. This handoff's runs of the release CLI on the same scope (donor vs
   the `perfprobe` scratch workspace, clean, exit 0): first run after cache
   eviction **11.4–13.9 s wall at ~20% CPU** (IO-bound), followed
   immediately by **0.71 s and 0.68 s** — warm steady-state beats even the
   corpus median. Acceptable for integration: the cold case is a read-only
   outlier that still undercuts the ~15–18 s destructive churn it replaces,
   and in the integrated flow jj-waltz's own pre-validation walks
   (`validate_contents` over the target, donor payload `validate_tree`)
   touch the same directories immediately before the diff walk, pre-warming
   it. The Positioning section's "~1–2 s" should still be labeled as the
   warm figure. The frontier remains the option if cold latency ever
   matters, with its measured J/K caveats.
3. **Load-bearing composition with jj-waltz's guards — recursive, not
   root-only.** `sync` has **no** `.git`/`.jj`/mount exclusions by design.
   Root-only checking is not safe: administrative entries appear at any
   depth (a `.git` inside a git-dependency checkout under `node_modules`,
   test fixtures), and mount points can be nested. jj-waltz already recurses
   on both sides — selection-time donor payload `validate_tree` (admin
   entries at any depth) and execute-time `validate_contents` over the whole
   target tree (admin/special plus `validate_same_mount` with statx mount
   IDs at every level). That composition is the safety story: if jj-waltz
   ever relaxed those recursive validations, sync would sweep a nested
   `.git` in target payload as a target-only deletion. Record the
   dependency in the integration's comments and docs; do not "simplify" the
   validations away as redundant with the walk.

## jj-waltz integration plan (Option B, now concrete)

Follows fit0's corrected requirements, updated for the sync crate:

1. **Dependencies**: path dependencies on `diff-dirs` and the sync package
   from jj-waltz's workspace; do **not** enable `parallel` initially
   (sequential walk is ~1 s warm at pool scale; Rayon buys nothing for one
   root). Verify the crates build under jj-waltz's pinned stable toolchain
   (this repo currently builds under the operator's unpinned nightly
   override) before wiring.
2. **`execute()` changes** (jj-waltz `src/reseed/engine.rs`):
   - delete the early else-branch payload removal (`for path in
     &plan.ignored { remove_path }`) so the previous generation survives the
     tracked phase — this is the change that makes any delta possible;
   - keep `--replace-content` wholesale (wipe contract, unchanged loop);
   - replace the final per-root `remove_path` + `create_dir_all` +
     `clone_tree` loop with, per selected root: `confined_parents` (keep),
     then `Applier::new(donor_root, target_root, ReflinkRequired)` (with
     `initialize()` for an absent target root) driving `diff_dirs::walk` in
     the caller-owned loop; the defensive second `remove_path` disappears;
   - `validate_source`/`validate_identities` pinning around the loop stays;
     the window is no wider than `clone_tree`'s today.
3. **Failure semantics**: convergent, not transactional — already blessed in
   fit0 and consistent with jj-waltz's journal-free, out-of-band recovery
   story (bookmark + re-run; a retry re-walks and finishes the delta).
4. **Optional but recommended — post-sync convergence proof**: after the
   apply loop, run a second walk over each root pair and require
   `Summary::is_complete()` with no differences. At ~1 s warm per root this
   restores a bounded payload verification the journal deletion removed,
   without resurrecting content hashing. If the cold-cache variance in
   finding 2 matters, this is the place it would be felt — measure before
   making it unconditional.
5. **Tests** (jj-waltz side): clean re-sync performs **zero writes** (assert
   target inode/mtime stability across a run); sparse delta converges
   exactly (one changed file restored with donor mtime; stale file pruned;
   symlink retargeted; file↔dir transition applied); absent-root
   initialization path; `--replace-content` behavior unchanged; existing
   process-count/perf gates stay green.
6. **Expected end-to-end effect**: clean-payload re-seed drops the ~15–18 s
   rm+recopy churn to a ~1 s walk (warm) plus zero writes, leaving jw's
   ~8–10 s planning/snapshot overhead dominant (≈ 10–13 s total versus the
   38 s measured baseline); changed payload costs proportionally to the
   delta. Do not promise sub-5 s end-to-end without first attacking the
   planning overhead — that is a separate jj-waltz conversation.

## Integration-level suggestions not in the existing inventories

Checked against the [design index](../index.md) and the opportunity
inventories; these are compositions, not new library capabilities:

- **Batch waves on one borrowed pool**: `--group all` reseeds N workspaces;
  each payload sync is an independent root pair. The existing
  `mirror_in_pool`/`walk_in_pool` API is exactly the shape for sharing one
  caller pool across workers — measure later under the frontier's documented
  J/K caveats before adopting.
- **CLI as the pool's drift-audit tool**: `diff-dirs donor/node_modules
  ws/node_modules` answers "what did that agent do to this workspace's
  payload" read-only before any destructive reseed. Worth one line in the
  operator procedures (~/ado/patches.md Phase 1) and jw docs.
- **Toolchain pin**: this repo rides the operator's nightly override; add a
  `rust-toolchain.toml` or explicitly verify stable in CI once jj-waltz
  depends on it, so the dependency's build doesn't drift with the
  environment.
- **License**: no LICENSE file exists. jj-waltz is MIT; decide before any
  vendoring or publishing so the dependency's provenance is unambiguous.
- **Scratch note**: `~/src/opencode-perfprobe` (registered in the pool as
  `perfprobe`) remains a live-shaped, disposable target used read-only for
  the walk verification above; keep it for integration testing or remove it
  with `jw remove perfprobe` when done.

## Verification performed for this handoff

- `cargo nextest run --workspace` (this checkout): **51/51 passed**.
- `cargo build --release -p diff-dirs-cli`: clean.
- Release CLI, live pair `~/a/a/opencode/node_modules` vs
  `~/src/opencode-perfprobe/node_modules`: complete, clean, exit 0,
  22,828 directories examined. Cold first run 13.9 s (prior day) / 11.4 s
  (this session); immediate warm repeats **0.71 s / 0.68 s**. Controlled
  warm median 1,033 ms in performance0. No live tree was mutated.
