---
type: ResearchAddendum
title: Truthful source state does not grant local membership authority
description: One two-receiver native comparison separates valid source cache and accurate prefill from permission to absorb and remove a retained ignored local path.
resource: /design/native/graph1.gpt61sxh.md
status: bounded-native-finding
generated: { by: "model:openai/gpt-6.1-sol#xhigh", at: 2026-10-20 }
sources:
  - resource: "opencode://ses_ea7bd841dffewRL4f7qZYaunBL"
    title: Parent-commissioned truthful-cache membership follow-up; one tiny two-receiver comparison and source-backed integration condition
  - resource: /design/native/graph0.gpt61sxh.md
    revision: 204653e6
    title: Earlier source/desired/target distinctions and separate existing sparse-reset route, left unchanged
  - resource: /design/roadmap/brief0.gpt6a.md
    title: Retained independent-reading boundary and good-source/failure assumptions
  - resource: "file:///home/rektide/src/diff-dirs/.test-agent/native-graph/README.md"
    title: Own fresh membership0 fixture, private environment, exact commands, state captures and source/local attribute matches
  - resource: "https://github.com/jj-vcs/jj/blob/33d3a595429b7e3b54a8ea769560247913bc8a1b/lib/src/local_working_copy.rs"
    title: Exact installed-runtime source for borrowed tree/file-state membership, ignore/auto-track gates and tracked-path removal during checkout
  - resource: "https://github.com/jj-vcs/jj/blob/803ef00a0a6ee4ddc275d3138d5f9e7326399906/cli/src/commands/restore.rs"
    title: Newer inspected source: plain restore selects the current change's parent tree
  - resource: "file:///home/rektide/src/jj-waltz/src/reseed/engine.rs"
    title: Current equal-S/D-subset sparse-reset route explicitly does not adopt donor tree_state
  - resource: "file:///home/rektide/src/jj-waltz/src/cow.rs"
    title: Current listed prefill delegates to strict create-only seed_leaves, not acceptance of an occupied local leaf
extensions:
  ticket: diffdir-tree-native-graph
  reading_boundary: No withheld parent statement, full roadmap waves, parent transcript, concurrent studies or subagents; scoped parent follow-up and prior own/authorized context only
verification:
  runtime: "jj 0.45.1-33d3a595429b7e3b54a8ea769560247913bc8a1b; Git 2.55.0; Linux btrfs"
  scope: One initialized archive and two tiny colocated receivers; truthful full-source prefill versus no-adoption equal-subset/reset control; actual reflinks, native membership, local inode, final D and receiving identity recorded; no JW execution or production change
  checks: Own-session provenance, source-path/link-anchor existence, fences, whitespace, probe syntax and recorded membership/local/final-D comparison passed
---

# Truthful source state does not grant local membership authority

**Yes, on the tested native build.** A truthful source `tree_state` can cause
receiver `status` to absorb a previously untracked/ignored local file into the
receiving tracked tree; plain `restore` then removes it because desired parent
`D` omits it. The file matched source bytes and relevant attributes throughout
prefill, retaining its original inode until native removal. This was not a bad
source, stale-byte cache or auto-tracking mistake.

The no-adoption equal-`S`/`D`-subset sparse-reset control retained that local file
and inode. **Both finished with exactly the intended tracked `D` tree, Git HEAD
and index.** Tracked conformance and clean Git status alone do not distinguish
the preserved-local outcomes.

This addendum refines the ownership boundary around
[graph0's reusable cache](/design/native/graph0.gpt61sxh.md#6-native-cache-reuse-is-saved-work-not-a-recurring-donor-trial).
It does not amend graph0 or allege a bug in current JW: current reseed does not
adopt source state, and fresh creation has no such preexisting retained locals.

## One bounded comparison

Fresh evidence home:
`file:///home/rektide/src/diff-dirs/.test-agent/native-graph/membership0/`.
Executable `membership.py` refuses an existing run and checks its owned root.
One initialized internal-Git archive, two native colocated receivers; three
source payload files totaling **1,847 bytes**. Same installed JJ/Git as graph0,
private HOME/XDG/JJ/Git configuration, fsmonitor none, stale updating disabled.
No previous fixture or live repository was mutated; no cleanup was run.

The tracked trees are deliberately this simple:

```text
S: .gitignore = "extra.txt\n", common.bin, extra.txt
D: .gitignore = "extra.txt\n", common.bin
```

Source `extra.txt` was tracked before adding the ignore rule; native snapshots
keep an already-tracked ignored path tracked. Construct `D` by removing it, then
return the archive to a source-shaped child. Thus the **same ignore rule is
present in both trees**; no ignore-file difference explains the result.

```text
S = 573a3fe1e3450fd4f955cdb8cf2d30f3e1ab24fa
D = 1a19740a18f5cbce1cfdaed4870858bb79f98b6a
```

Both receivers begin as full empty children of `D`. Create ignored local
`extra.txt` independently in each. Match its bytes, regular-file kind, mode
`0644`, size `45`, and mtime `1700000000000000000 ns` to source **without replacing
its original inode**. An initial explicit `snapshot.auto-track=all()` status in
each receiver leaves it untracked/ignored. Every later command uses `none()`.

For each receiver, sparse-clear only tracked disk contents and select a new
empty child of `D`. The local file survives. Reflink `.gitignore` and `common.bin`
from source with real `FICLONE` and attributes; leave local `extra.txt` alone.
All three `S`-tracked paths now accurately match source bytes/mode/size/mtime.
This establishes a **truthful full-source prefill**, not a partially filled or
mismatched-cache counterexample.

| Route | Subsequent work |
| --- | --- |
| `adopt` | Write source `tree_state` bytes only; preserve receiver `checkout`; run receiver `status`, then plain `restore`, then final status. |
| `control` | Do not adopt source state; run `sparse reset` and final status. Its reflinked leaves are precisely the equal-`S`/`D` subset. |

The experiment deliberately accepts the matching existing local as truthful
prefill. It does not claim current JW's strict create-only `clone_files`/
`seed_leaves` would accept that occupied destination (`src/cow.rs:156–176`).

## Observed membership and physical outcomes

All setup/final-intent assertions passed on the first run. Raw stage JSON,
source/prior state blobs and `commands.jsonl` retain exact observations.

| Stage | `adopt` | `control` |
| --- | --- | --- |
| Initial receiver | `extra.txt` absent from repository tracked list and local native file-state map; present, ignored, inode **298147415**. | Same membership; present, ignored, inode **298147489**. |
| Truthful prefill | Same local inode/bytes/attrs; receiving commit still omits extra. | Same local inode/bytes/attrs; receiving commit still omits extra. |
| State-only adoption | Borrowed local native tree/file-state map includes extra; repository working-copy commit still omits it. `checkout` bytes untouched. | Not performed. |
| After status / reset | Status rewrites receiver's own change to the `S` tree: extra is now **tracked**, same original inode. Git shows intent-to-add ` A extra.txt`. | Reset leaves extra **untracked**, present with original inode. |
| After final native finish | Plain restore returns tracked tree to `D`; extra is **absent from disk**. | Tracked tree is `D`; extra remains **present/untracked/ignored**, same original inode. |

Both keep their root, `.jj`, repository pointer, `.git` gitfile and private Git
administration identity. Both end empty single-parent children of exact `D`,
with the same complete `D` index, HEAD=`D` and clean Git porcelain status.
Source tracked files, current `S` and its `tree_state` bytes remain unchanged.

The diagnostic distinction is visible **before** any repository rewrite:
immediately after adoption, `jj --ignore-working-copy debug local-working-copy`
shows borrowed source tree `c66235d3bd30bf45d0497515b13b655d2bd52f2c` and the extra
file state, while snapshot-free `file list -r @` still reports only `D`'s two
paths. After status, repository membership agrees with the borrowed source
tree. The source-state blob's hash remains unchanged through that status.
Therefore the cache was accurately reusable; its semantic effect was not merely
avoiding hashing.

## Source explanation: accuracy and membership are different facts

Exact runtime source `33d3a595…:lib/src/local_working_copy.rs` establishes:

1. **`1141–1177`:** reading `tree_state` supplies the tree, file-state map,
   sparse patterns and monitor clock. It is not only a bag of stat hints attached
   to the receiver's unchanged tracked membership.
2. **`1691–1704`:** ignore and new-file auto-tracking gates apply when
   `maybe_current_file_state.is_none()`. After adoption, the extra has such a
   state: neither ignoredness nor `snapshot.auto-track=none()` removes it from
   the borrowed tracked tree.
3. **`2193–2199,2325–2331`:** native checkout diffs the old native tree against
   the new one and removes a formerly present tracked leaf. Plain restore selects
   the parent tree (newer inspected `restore.rs:129–136`); the tested parent is
   `D`, which omits the extra. Native removal is consequently ordinary tracked
   `S→D` materialization, not generic copy ignoring a preserve directive.

By contrast, current JW `src/reseed/engine.rs:486–491,1427–1435,1500–1519`
selects equal source/desired leaves, explicitly **does not adopt donor state**,
and expands receiving `D` state. The extra belongs to neither old tracked target
contents nor desired tracked contents, so that route never imports its source
membership. The control demonstrates this native distinction; it does not run JW
or bypass/alter its production colocation guards.

## Integration condition, not another donor validation system

A good source suffices for **source validity**. Truthful prefill suffices for the
cache's representation of tracked disk content. Neither fact grants authority
to reinterpret a retained receiving local as native-owned, then remove it.

Let `K(S)` and `K(D)` be the source/desired tracked path sets, and `R` the
receiving local region promised preservation by the consumer. In this exact-path,
ordinary-file example, the removal hazard is:

```text
R ∩ (K(S) \ K(D)) ≠ ∅
```

For a future **full-`S`-state existing-target adapter**, native integration must
explicitly handle that overlap before adoption: decline the optimization/use the
no-adoption route, refuse incompatible intent, or require deliberate permission
to absorb/remove those locals. Byte equality is not that permission. Ancestor/
descendant shape overlap needs the same namespace-ownership treatment, not only
an exact filename set; this one-leaf probe is not runtime coverage of those shapes.

If preservation includes **remaining untracked**, not just surviving this restore,
the stronger concern is `R ∩ K(S)`: borrowing membership itself can violate that
promise even if a later native operation would leave the bytes present.

- **`S=D`, or equal tracked trees:** `K(S)\K(D)` is empty, so this source-only
  restore-deletion mechanism disappears. This is not permission for desired `D`
  itself to overwrite a retained local obstruction; normal desired/local
  compatibility still applies.
- **No retained overlap:** if all `S`-tracked materialization belongs to the native
  phase, or retained locals lie outside its affected namespace, this particular
  ownership conflict disappears even when `S≠D`. Fresh creation's source-only
  prefill files are supplied by the operation, not preexisting retained locals.
- **Current existing-target route:** keep equal-subset/reset and no borrowed
  membership; no new source-state condition is needed merely to retain that route.

This is a compatibility condition over already meaningful source/desired native
membership and consumer retention intent—not a demand for repeated source-byte
verification, a new full donor walk, locks, a metadata framework, recovery or an
effects ledger. Choosing the existing no-adoption route avoids having to admit
this additional ownership transition at all.

Scope remains one conflict-free/full/single-parent comparison on the named build.
No other versions, shapes, failures, filters, sparse sources or monitor modes
were probed. Only this addendum is committed; graph0, production code/defaults,
shared indexes and prior scratch remain unchanged. Parent owns integration and
ticket closure.
