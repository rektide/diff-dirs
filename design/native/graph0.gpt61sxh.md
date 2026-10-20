---
type: ArchitectureStudy
title: Copying bytes and selecting native work tips
description: Separate ordinary copying, archive-backed creation and existing-workspace revision transitions; preserve receiving identity, reuse native work, and compose optional progress without an effects history.
resource: /design/native/graph0.gpt61sxh.md
status: independent-research
generated: { by: "model:openai/gpt-6.1-sol#xhigh", at: 2026-10-20 }
sources:
  - resource: "opencode://ses_ea7bd841dffewRL4f7qZYaunBL"
    title: Derived plain/native phase models from current JW and native source; ran one existing-colocated-target sparse/refill case with distinct source, desired and private work tips; specified optional intent/progress state
  - resource: /design/roadmap/brief0.gpt6a.md
    title: Scoped independent commission and failure assumptions
  - resource: /design/execution/contracts0.gpt61sxh.md
    revision: 5d6c003a
    title: Completed owned progression, intent and native readiness barriers
  - resource: /design/native/metadata0.gpt6am.md
    revision: 014db6e3
    title: Completed archive versus receiving administrative ownership study
  - resource: /design/native/workspace-source0.gpt6am.md
    revision: d6f19aeb
    title: Completed separation of donor, repository owner, policy root and desired revision
  - resource: /design/native/colocation0.gpt61sxh.md
    revision: 9b4fc6ec
    title: Reported ten successful fresh-receiver native colocation/CoW cases; not existing-target evidence
  - resource: /design/transfer/policy0.gpt61sxh.md
    revision: a63595a9
    title: Completed copy/removal/preservation authority and post-snapshot classification analysis
  - resource: /doc/architecture/README.md
    revision: 1193a1ee
    title: Prior proposed component ownership and optional continuation baseline
  - resource: "file:///home/rektide/src/jj-waltz/src/reseed/engine.rs"
    title: Current target snapshot, frozen source/base/old tips, guards, sparse/new/prefill/reset and verification sequence
  - resource: "file:///home/rektide/src/jj-waltz/src/reseed/bookmarks.rs"
    title: Post-snapshot work-tip coverage, without implicit bookmark creation or movement
  - resource: "file:///home/rektide/src/jj-waltz/src/reseed/prepare.rs"
    title: Separate source-only fetch/advancement and retry-from-start preparation
  - resource: "file:///home/rektide/src/jj-waltz/src/reseed/payload.rs"
    title: Existing outside-payload visitor also rejects receiving root Git administration
  - resource: "file:///home/rektide/src/jj-waltz/src/workspace.rs"
    title: Sparse-empty new identity, tracked copy, private donor state adoption and restore
  - resource: "file:///home/rektide/src/jj-waltz/src/lifecycle.rs"
    title: Explicit/implicit creation-base freezing and current exclusive creation modes
  - resource: "file:///home/rektide/src/jj-waltz/src/jj.rs"
    title: Snapshot-free pinned queries, tracked file lists and tree-state-only helpers
  - resource: "file:///home/rektide/src/jj-waltz/src/ownership.rs"
    title: Target-private versus common Git administration and ownership resolution
  - resource: "file:///home/rektide/src/jj-waltz/src/cow.rs"
    title: Listed create-only reflinks with attributes, not arbitrary donor discovery
  - resource: "file:///home/rektide/src/jj-waltz/src/cow/transfer.rs"
    title: Existing clear preserves only root JJ administration and refuses Git entries
  - resource: "file:///home/rektide/src/jj-waltz/docs/reseed.md"
    title: Maintained source-only preparation, fresh-child reseed, auto-tracking and separately selected feature layering
  - resource: "https://github.com/jj-vcs/jj/blob/803ef00a0a6ee4ddc275d3138d5f9e7326399906/lib/src/local_working_copy.rs"
    title: Native private tree/stat/sparse state, independent checkout identity, sparse expansion and placeholder state
  - resource: "https://github.com/jj-vcs/jj/blob/803ef00a0a6ee4ddc275d3138d5f9e7326399906/lib/src/git.rs"
    title: Native first-parent HEAD and merged-parent index semantics
  - resource: "https://github.com/jj-vcs/jj/blob/803ef00a0a6ee4ddc275d3138d5f9e7326399906/cli/src/cli_util.rs"
    title: Native snapshot rewrites and colocated intent-to-add/export behavior
  - resource: "https://github.com/jj-vcs/jj/blob/803ef00a0a6ee4ddc275d3138d5f9e7326399906/cli/src/commands/sparse/mod.rs"
    title: Native sparse mutation commits local state, independently of complete payload readiness
  - resource: "https://github.com/jj-vcs/jj/blob/803ef00a0a6ee4ddc275d3138d5f9e7326399906/cli/src/commands/restore.rs"
    title: Plain restore restores current content from its parents, not new parent selection
  - resource: "file:///home/rektide/src/diff-dirs/.test-agent/native-graph/README.md"
    title: Own initialized tiny fixture, explicit snapshot policies, exact commands and retained stage captures
extensions:
  ticket: diffdir-tree-native-graph
  reading_boundary: Withheld parent statement, full roadmap waves, parent transcript and concurrent drafts were not read, searched, cited or evaluated; only explicitly authorized completed studies and targeted factual sources were used; no subagents
verification:
  runtime: "jj 0.45.1-33d3a595429b7e3b54a8ea769560247913bc8a1b; Git 2.55.0; Linux btrfs"
  scope: One initialized internal-Git archive and one existing colocated receiver; actual reflinks, private tracked work, initial auto-tracking, retained ignored file, exact native identity/tree/HEAD/index/status assertions; no JW execution, production changes, failure probes, suites or benchmarks
  document_checks: Own-session provenance, source-path existence, six linked files/anchors, balanced Markdown fences and trailing whitespace checked; probe syntax checked; all passed
---

# Copying bytes and selecting native work tips

**Ordinary copying needs no JJ/Git operations. Native identity creation, snapshot
classification and revision selection are separately requested JW work.** Copying
files into a workspace does not select its parent, preserve its old feature line
by itself, or establish agreement between disk, native tree state and Git index.

Fresh-receiver evidence is already strong. The single new **existing-colocated**
case below also succeeds through native sparse clear → new child of desired `D`
→ equal-source/desired reflinks → sparse reset, without donor state adoption or
Git repair. This is a feasible native mechanism, **not current `jw reseed`
support**. Its root `.git`/owned-worktree rejection, repeated identity guard,
outside-payload validation and explicit-clear policy remain incompatible.

Assume a good source checkout. Concurrency/locking are out of band. Failure
promises no automatic recovery, rollback or effects history. Native JJ recording
old work is useful ordinary native history, not a new transfer transaction.

## 1. Source S, desired D and existing work tip T

Use a small JW-owned binding, not a global metadata framework:

| Coordinate | Meaning |
| --- | --- |
| `A`, `R`, `G` | Archive/physical primary, its real `A/.jj/repo`, and Git common store: `A/.jj/repo/store/git` or `A/.git` in the established layouts. |
| `P`, `S` | Selected donor checkout path and its pinned, snapshotted tracked commit. Normally `P=A`. Source disk supplies bytes; `S` identifies the tracked source tree. |
| `D` | Frozen desired **parent commit**, from source current tip by default or independent `--at`. Its tree is the intended tracked answer. |
| `W`, `T` | Existing receiver root and its old tracked working-copy commit **after the initial snapshot**. Feature work is here, not in `S` or `D`. |
| `N` | Receiving new working-copy commit. Successful reseed makes an empty child of `D`: `parents(N)=[D]`, `tree(N)=tree(D)`. |
| `L` | Independently selected auxiliary donor-local payload, not represented by `S` or implied by `D`. |

`S`, `D` and `T` are commit coordinates, not paths or interchangeable tree IDs.
Equal trees can have different commits/parents. `D` may itself be a merge commit;
current JW still supplies it as **one** parent. Frozen operation coordinates make
revision interpretation precise; an operation ID is not a copy receipt.

Actual source resolution is already separate from desired selection:

- **Creation:** `src/lifecycle.rs:651–683` snapshots for an implicit base or
  `--source-at`, then freezes `D`; explicit target-only `--at` resolves without
  forcing that initial snapshot. `--source-at` asserts tree equality to the donor;
  it does not select `D`. `workspace.rs:498–505` always passes the frozen revision.
  Later native commands do not change the already selected `D` retroactively.
- **Reseed:** `src/reseed/engine.rs:362–412` snapshots source and targets, pins
  `S`/source current commit and old `T`, then selects `D`: donor current unless
  explicitly overridden. `source_commit` can be an equal-tree asserted revision
  distinct from `source_wc_commit`. `old_commit=T`, `base_commit=D`, and the mutable
  field `desired_commit` starts at `T`, becoming the allocated `N` after `jj new`.
  That last field is **not** another spelling of `D`.
- `--from` selects the actual checkout supplying bytes. A registered workspace
  points directly to resolved `R`; it does not acquire archive ownership. The
  completed [source-binding study](/design/native/workspace-source0.gpt6am.md#source-binding-model)
  owns that extension and the separate primary policy/link roots.

### Initial auto-tracking changes classification

Current reseed's first target `status` uses native auto-tracking policy, not
`none()` (`engine.rs:380–397,833–837`). An eligible nonignored `notes.txt` absent
from the pre-command tree can therefore enter `T`. It is then **tracked old
work**, authorized to leave disk when reseed selects `D`, and saved in native
history. A still-untracked ignored local file outside selected/clear/link regions
stays. Initial untrackedness is not a perpetual preservation right.

Later verification and transition commands explicitly disable **new** tracking
(`engine.rs:1030–1056,1452–1467,1480–1486,1510–1516`). This neither untracks files
already recorded in `T` nor retroactively changes the first snapshot's policy.

`src/reseed/bookmarks.rs:94–100,187–215` guards the post-snapshot work tip:
an ordinary undescribed empty single-parent child delegates to its parent;
otherwise the work tip is the current commit itself. A named ref must cover
that tip or a descendant, unless explicitly bypassed. JW does not create/move
the ref for this guard. An older ancestor bookmark need not cover new work.
The check can block **after** snapshot effects but before refill.

This is the existing contract, not a recommendation to add pre-command inventories.
Invocation-time untracked preservation would need an explicit new product promise
across native clearing, native materialization and auxiliary transfer; the completed
[transfer study](/design/transfer/policy0.gpt61sxh.md#5-initial-untracked--snapshot-tracked-a-temporal-classification-not-a-loophole)
details that fork. No generic copy path needs this classification.

## 2. Exactly when native operations belong

| Requested outcome | Native work needed? |
| --- | --- |
| Copy/update/seed/mirror ordinary directories under declared preservation/removal policy | **No.** Enumerate/assess/install according to that policy. No `status`, fetch, snapshot, `new`, restore, bookmark or Git-index maintenance. |
| Copy selected ignored data around an already correct native workspace | Filesystem transfer alone suffices for the copy. A consumer may separately request native conformance; it is not implicit revision advancement. |
| Copy tracked bytes into an existing native workspace without changing its parent | Bytes can be copied normally under explicit authority; they are working edits. A separately requested snapshot records them. Neither action makes `D` the parent. |
| Create a registered JJ receiver, with optional Git colocation | Native creation must allocate fresh receiver administration and a new working-copy commit. Payload copy alone cannot manufacture this identity. |
| Reseed existing receiver onto `D`, renewing tracked CoW | Snapshot/classify old work as declared; preserve native identity; select new child `N` of `D`; materialize it and converge native/Git state. Even equal trees do not erase the requested graph operation. |
| Fetch/build the archive for later use | Explicit source preparation, separate from copying/refilling receivers. |
| Layer private feature work onto a new base | Explicit later JJ duplicate/rebase/etc., not a side effect of copy or reseed. |

Plain copy does not require repository detection. Generic libraries have no
universal `.jj`/`.git` rule; JW supplies its administrative reservations. A later
human/native command may snapshot a generic copy's results, but that is not the
copy engine implicitly acquiring graph authority.

`jj restore --from D` changes the current change's **content**, not its parents;
plain `jj restore` takes content from its existing parents. Neither substitutes
for `jj new D` when the request is a fresh child of `D`. The creation optimization
uses plain restore only because native creation already established the correct
parent. No direct Git reset or index replacement is needed in the demonstrated
routes; native JJ owns those transitions.

## 3. Target-owned state stays; some values legitimately change

At a JW receiving root, reserve **both sides** of root `.jj` and `.git` against
generic writers/removers. An absent source `.git` cannot delete a receiving
gitfile; a source `.git` directory cannot replace that gitfile.

| State | Required treatment |
| --- | --- |
| `W`, native workspace name/path registration and repository relationship | Preserve on an existing transition; native creator allocates on new creation. No root swap or re-registration as donor. |
| `W/.jj/repo` | Preserve receiver's pointer to shared `R`; never copy archive repository or donor pointer wholesale. |
| `W/.jj/working_copy/checkout` | Keep receiver-owned name/operation binding. Native commands may update its operation bytes. Never transplant donor `checkout`. |
| `W/.jj/working_copy/tree_state` | Receiver-owned mutable native tree/stat/sparse cache. One explicitly supported private adapter may adopt donor state; generic transfer cannot. Current reseed does **not** adopt it. |
| `W/.git`, its private `G/worktrees/<registration>/`, backlinks and common-store relationship | Preserve existing topology/ownership. Native JJ legitimately changes receiving HEAD/index; do not insist their old values stay. Do not copy donor-private Git administration. |
| Shared JJ/Git repository and refs | Native-owned. Requested native commands can record operations/commits and export refs; not generic payload. Old work coverage remains an explicit policy. |
| Unrelated target ignored/untracked contents | Preserve unless an authorized selected mirror, link operation or explicit clear affects them; refuse retained obstructions to desired tracked paths. |

The [metadata study](/design/native/metadata0.gpt6am.md#copy-boundary-new-versus-existing-receiver)
owns the topology. `local_working_copy.rs:1142–1178,2676–2724` independently
confirms that `tree_state` and `checkout` have different contents and purposes.
`src/ownership.rs:89–118` resolves target-private versus common Git directories;
do not use `jj git root` as the private-admin coordinate.

Preserving identity is **not** preserving the old current commit/change ID:
reseed intentionally selects new `N`; private `T` stays as old native work.
Source-state adoption during fresh creation instead rewrites the already-created
receiving change's content before restore, keeping its fresh change identity.

## 4. Direct phase models

### Plain directory operation

```text
bind ordinary roots + declared transfer/preserve/absence policy
  → inspect/settle containers and leaves
  → complete selected filesystem obligations
```

Optional cursor progress wraps those obligations only. It adds no source
snapshot, graph intent, repository metadata, native finish or required codec.
Mirror/update/seed select different overwrite/removal authority; this account
does not choose a broader JW transfer default.

### New archive-backed native receiver

```text
resolve donor P/S + desired parent D + requested native topology
  → native workspace add -r D [--colocate|--no-colocate]
  → native metadata/receiving ownership recorded
  → tracked materialization
  → native convergence complete
  → selected auxiliary payload/links
  → ready in the command's declared domain
```

For ordinary native materialization, workspace-add supplies tracked files.
For the existing creation optimization (`workspace.rs:616–659`):

1. Native add uses sparse-empty and allocates the receiver on `D` first.
2. Copy donor tracked leaves with real CoW/attributes. Do not copy `.jj`/`.git`.
3. **After complete donor tracked prefill**, adopt donor `tree_state` only.
4. Receiver `status` snapshots source-shaped content into its own new change;
   plain `restore` converges that content to the already-selected parent `D`.
5. Native finish establishes the intended tree/parents and native Git semantics.

When `S != D`, source-only paths disappear, desired-only paths materialize, changed
paths reconcile, and equal tracked files can retain the prefill. The completed
[colocation study](/design/native/colocation0.gpt61sxh.md#results) reports ten
successful tiny cases, including internal/root-colocated archives, opt-out,
different trees, root and clean multi-parent controls. They are reused as reported,
not rerun or converted into existing-target evidence here.

Current lifecycle still excludes joint CoW+colocation (`lifecycle.rs:126–152`),
and clone fallback/error cleanup drops receiving Git ownership in
`workspace.rs:558–572,662–665`. Native feasibility does not enable that combination
or settle a product default.

### Existing native transition: current reseed mechanism and proposed colocation adaptation

```text
bind existing W + native identity + source S + desired D
  → initial snapshot policy → freeze old T + work-tip coverage
  → declare retained local/auxiliary/clear regions; refuse obstructions
  → sparse clear, retaining receiver administration and unrelated local data
  → new empty child N of D, still sparse-empty
  → prefill only leaves equal between S and D
  → sparse reset / full native materialization
  → native tree/parent/sparsity conformance
  → selected auxiliary mirrors and configured links
  → final native conformance → ready
```

This is not the creation state-adoption path. Current
`reseed/engine.rs:455–491,1427–1568` computes `clones` as desired paths present
in source and not changed between `S` and `D`; clears tracked disk content;
creates `N`; reflinks that subset; expands sparsity; then verifies.
It never copies donor `tree_state`. `remove_recorded_file` is leaf-oriented,
not recursive removal of unrelated children (`1574–1595`).

An optional explicit clear currently runs after selecting `N` and before prefill.
The new experiment does **not** exercise it. Any colocated adaptation must reserve
root `.git` as well as `.jj` throughout clear, payload validation and native
identity checks. Current blocking sites include:

- `engine.rs:790–795`: root `.git` and external/owned-Git records rejected.
- `engine.rs:1372–1375`: repeated rejection of acquired Git metadata.
- `reseed/payload.rs:44–46,78–83`: receiving outside-payload validation called
  with `allow_git=false`, so it also refuses root `.git`.
- `cow/transfer.rs:165–185`: `WorkspaceClear` preserves root `.jj` only and
  refuses `.git`, including root `.git`.

Deleting guards is not the adaptation. Existing target Git ownership must remain
precise; an owned native linked worktree is not permission to admit every external
app-owned target. No removal/recreation/prune is part of this transition.

## 5. One existing-colocated-target experiment

### Scope and exact intent

Own fixture:
`file:///home/rektide/src/diff-dirs/.test-agent/native-graph/run0/`.
One fresh initialized internal-Git archive and **one** colocated receiver, on
Linux/btrfs. Source payload: four regular leaves, **7,198 bytes**; every stage
remains far below 256 payload entries / 1 MiB. No imported workload, existing live
checkout mutation, wholesale `.jj` copy, donor state adoption or automatic cleanup.

Private HOME/XDG/JJ/Git configuration; actual `/usr/local/bin/jj` version
`0.45.1-33d3a595429b7e3b54a8ea769560247913bc8a1b`, `/usr/bin/git` 2.55.0;
fsmonitor none, stale auto-update disabled. Initial source/target captures
explicitly use `snapshot.auto-track=all()`; subsequent commands use private
config `none()`. The initial target snapshot is deliberately allowed to classify
new nonignored `notes.txt` as tracked. `keep.local` is ignored by the tracked
`.gitignore` in base, source and desired trees.

Create sibling source `S` and desired `D` from `B`. Source has different
`change.txt` and `source-only.txt`; desired has `desired-only.txt` instead.
Create full colocated receiver on `B`, then edit its tracked `change.txt` privately,
add initially untracked `notes.txt` and ignored `keep.local`. Snapshot the receiver
to `T`, explicitly create a research bookmark `private-feature` at `T`, and
request a **new empty child of `D`**, not rebase/preservation of feature contents
at the same working path. That explicit bookmark is fixture setup, not behavior
attributed to the JW guard.

```text
B = 060d9753fb70b19121d676179d46ef9c25d7d2fb
S = e1fa3b33ba025ab9913857b9e5e36f47550b0a8b
D = f7d4d310638b34775f18206a22d9d6bede4d1714
T = 8389d59bd913d25da8f46d2530ce4f0303077d63
N = 0b2a34a70eec6e3f7a2c6af6351048450ef43ee9
```

Execute native `sparse set --clear`, `new <D>`, strict real `FICLONE` of only
`.gitignore` and `common.bin` (the equal `S`/`D` leaves), `sparse reset`, then
auto-track-disabled tracked-tree verification. No direct Git mutation. Unlike
current JW, this research sequence explicitly leaves receiving `.git` alone.
Native commands, not hand-built metadata, retain and update Git administration.

### Observed result

**All assertions passed on the first run.** Exact commands/stdout/stderr are
`commands.jsonl`; stage JSON captures and `summary.json` are retained with the
script/README. Git reads use `--no-optional-locks`.

| Stage | Native tracked meaning | Git observation |
| --- | --- | --- |
| Before snapshot | Existing tip does not yet include `notes.txt`; private disk edit present. | Working changes/untracked candidate visible. |
| Saved `T` | Tracked private edit and newly auto-tracked notes; parent `B`. | HEAD=`B`; index is parent plus intent-to-add notes; ` M change.txt`, ` A notes.txt`. |
| Sparse-empty `T` | Same `T`; tracked disk leaves removed, ignored local retained. | Intermediate deletions, not readiness. |
| New sparse-empty `N` | Empty child of **`D`** already exists logically, but payload absent. | HEAD=`D`; index exactly `D`; four tracked deletions. `keep.local` temporarily appears `??` because disk `.gitignore` is absent. |
| After equal-leaf reflinks and sparse reset | Full desired tree; unchanged `N`, parent `D`, empty diff. | HEAD=`D`; exact complete `D` index; clean porcelain status. |

Before/after identity values agree:

| Coordinate | Preserved value |
| --- | --- |
| Receiver root `(device,inode)` | `(31,297940725)` |
| Root `.jj` `(device,inode)` | `(31,297940797)` |
| `.jj/repo` bytes | `../../archive/.jj/repo` |
| `.git` bytes | `gitdir: ../archive/.jj/repo/store/git/worktrees/receiver` plus newline |
| Private Git admin `(device,inode)` | `(31,297940736)`; same absolute path, `commondir` and `gitdir` backlink |
| Native receiver name/registered path | `receiver`, same root |
| Ignored `keep.local` | Same bytes and inode `297941128` |

The target's old and new **change IDs differ**, as intended. Its `checkout`
hash changes through native `new`; that is an operation-binding update, not lost
identity. `private-feature` still points at exact `T`; querying `T` recovers both
private file bodies. They are absent from the new desired disk tree. Source
current `S`, tracked bytes and its `tree_state` bytes are unchanged.

Both real reflinks have separate source/target inodes and preserved attributes;
target `common.bin` inode `297943992` survives sparse reset and verification.
Desired-only content materializes; different `change.txt` has desired bytes;
source-only and old notes are absent. Final complete index entries, not merely
status text, match `D`'s Git tree. No CoW performance or hash-count claim follows.

### What this resolves, and what it does not

This directly resolves a **native existing-target** uncertainty for the tested
ordinary single-parent/full/conflict-free case: target colocation does not by
itself prevent the current sparse/new/equal-prefill/reset mechanism from converging
while preserving ignored local data and target administration. No private donor
cache transplant was needed. Creation evidence alone would not establish that.

It does **not** execute JW, prove broad explicit clear or selected-payload/link
composition, exercise JW ownership records, establish failure/retry behavior,
or cover root/multiple-parent/conflicted/sparse/Watchman targets, submodules,
filters, external owners or other versions/platforms. Those are separate supported-
scope decisions and focused integration work, not grounds to remove guards now.
The older ten-case creation report remains creation evidence only.

## 6. Native cache reuse is saved work, not a recurring donor trial

There are three distinct savings:

1. **Shared native repository:** receiver metadata points to existing `R`/`G`.
   Objects/operations are not recursively copied with every workspace.
2. **Tracked prefill:** listed known leaves avoid unrelated payload discovery and
   native rewriting of accepted prefilled bytes. Real reflink shares extents with
   private inodes; preserved mode/size/mtime are useful execution attributes.
3. **Donor `tree_state`, on the supported creation route:** native tree identity,
   stat map and sparsity are already-computed work. Matching file state with a
   suitably earlier mtime lets JJ avoid rereading/hashing unchanged tracked bytes
   (`local_working_copy.rs:1936–1956`). This is a private, version-sensitive cache
   adapter, not ownership and not proof of auxiliary payload or whole disk equality.

`S != D` does not invalidate a good cache for the **source prefill**. That cache
describes `S`; native restore does the genuine `S→D` work. It neither skips desired
changes nor converts `S` into `D` by naming. Never attach the entire source cache
to a partially copied source tree or only the equal-`S`/`D` subset and pretend its
full source stats/tree describe the target.

Current reseed selects the equal subset, does not adopt donor stats, and relies
on native sparse expansion plus verification. Native expansion encountering
already-present prefills can record placeholder state (`local_working_copy.rs:
2430–2449`); verification may read/hash them. The new experiment proves retained
prefill and exact outcome, **not** the creation cache's hash savings on reseed.
A future donor-state reseed optimization needs its own supported adapter/evidence.

No whole-checkout validity proof loop is required by this design. Current
`validate_source`/`verify_disk` repetition is observable current JW behavior, not
a proposed recurring source-quality system. Under the good-source/out-of-band-
writers assumption, bind `P/S`, compatible native cache and explicit desired `D`
once; preserve those inputs through the run. Execution still needs file attributes,
routing and native format/capability compatibility. Rebinding an incompatible
cache must decline that optimization, not silently invent equality.

Native finish still costs work: stat/discovery as required by JJ, genuine differing
payload materialization, operation/commit updates and Git index/export. Native
`git.rs:2000–2052,2087–2139` defines HEAD as the first parent and index as the
merged-parent tree, with intent-to-add entries for working additions. For root
HEAD can legitimately be unborn; for multiple parents clean JJ need not mean
empty Git staged diff. Do not "repair" native semantics to an invented universal
clean-Git criterion. The new one-parent result correctly requires clean Git.

## 7. Optional cursor: future intent and native barriers

Use the [completed execution contract](/design/execution/contracts0.gpt61sxh.md#2-progress-acceptance-and-failure):
one owned pending unit, ordered acceptance, quiescent effects and explicit incomplete
coverage. JW can bind a native component beside a tracked/auxiliary transfer driver.
This does not require native types in core, a command journal, an action registry,
a universal metadata store or serialization for ordinary in-process use.

### Minimal owned data

| Saved owner | Data needed to continue the chosen intent |
| --- | --- |
| JW binding/descriptor | Source `P/S` (and asserted equal-tree revision if meaningful), repository relationship, receiver root/name/topology, desired exact `D`, declared snapshot/retention/auxiliary/clear policy and adapter compatibility. Do not resolve moving `@`/bookmarks anew as changed intent. |
| Existing transition component | Old `T`/work-tip coverage where still required; phase; allocated receiving `N`/change identity after native selection; expected parent `D`; required native materialization/conformance obligation. |
| Transfer driver | Owned pending tracked/auxiliary coordinates or remaining selected list, phases, accepted coverage/aggregate and unresolved finish; not completed-file mutation records. |
| Private cache adapter, if selected | Compatible source-state input/reference or owned bytes bound to `S` and the prefill it describes. Keep native blob outside a generic metadata schema. The new existing-target route has no such input. |

`T`, `D`, `N` and component phase are **intent/tips**, not an effects history.
Native operation bindings may be retained where a later pinned query needs them;
they are not a proof that each file was installed. Retire obsolete planning data
when no future obligation needs it. Do not accumulate every operation, removed
child or native command stdout as the cursor grows. Research logs are evidence,
not the runtime data model.

### Supported voluntary pauses

Pause between joined native commands or closed leaf/container transfer units.
Close files/staging and settle temporary permission scopes first; no JJ/Git
process, native lock, borrowed dirent or live permission lease is saved.
Native sparse reset, status and restore remain **opaque native effect units**;
counting one dispatch does not make their tree work bounded by a directory or
numeric byte/time budget.

- With **sparse-empty/equal-subset prefill**, the cursor may carry remaining clones
  before expansion. Logical `N` already contains `D`, while the disk/Git status is
  intentionally partial. Snapshot-free queries can inspect intent; neither an
  empty JJ diff under sparse-empty nor a known desired tree proves readiness.
- With **full source prefill/state adoption**, finish must wait until the whole
  promised donor tracked materialization is complete before the full source state
  is adopted and snapshot/restore runs. A missing copied leaf can otherwise become
  a recorded deletion; running finish mid-copy is not a harmless status refresh.
- A tracked **optimization** leaf may be intentionally abandoned only if another
  selected native materializer supplies the required content. That is declining
  prefill, not skipping the desired tracked-file obligation. A genuinely skipped
  required tracked region propagates incomplete coverage and forbids readiness.
- Auxiliary transfer can pause independently after tracked native conformance.
  Final whole-command readiness waits for every required auxiliary/link obligation
  and the declared final native check. Unrelated preserved regions are outside the
  claim; completion never means an unrestricted donor/target disk mirror.

Native selection is not made idempotent by storing `phase=New`: blindly retrying
`jj new D` can allocate another change. Save allocated `N` after an acknowledged
successful command; an unacknowledged/failed native mutation requires native-specific
current-tip examination, explicit refusal, or out-of-band disposition—not guessed
success or automatic replay. If the binding cannot safely support a pause around
that obligation, expose it as one non-pausable native segment. No complete native
failure/continuation adapter is claimed implemented or tested here.

Codec reconstruction/binding checks representation, routes, descriptor and phase
compatibility, not source quality through a new full donor walk. Actual unresolved
effects observe what they need. A cursor-file write failure after native/transfer
effects does not undo those effects; stale cursor reuse can repeat them. No
cross-process exactly-once, crash recovery, history or rollback promise is added.

## 8. Remaining human/product choices and landing boundary

1. **Colocation scope/default:** creation-only with an explicit JJ-only reseed
   boundary, or a separately implemented colocated-reseed slice first? The new
   result favors the feasibility of the latter, not its current availability.
   Shared add/switch creation default scope and creation ownership/fallback repair
   remain the separate colocation study's decisions.
2. **Native route:** keep current existing-target equal-prefill/sparse reset, or
   commission donor-state reuse there too? The first now has this bounded native
   colocated evidence. The second may save hashing but is not established here.
3. **Snapshot contract:** preserve current post-snapshot classification, or select
   a new invocation-time untracked preservation promise? Explicitly decide rather
   than letting a generic visitor try to repair earlier native classification.
4. **Transfer breadth/clear:** keep selected ignored-root mirrors and retained
   unrelated data, or select a broader policy? If explicit clear is supported for
   colocated receivers, reserve both native roots and test it deliberately. The
   retained-local experiment does not prove broad-clear behavior.
5. **Progress granularity:** plain forward native segment plus pausable auxiliary
   walking is already coherent. Fine-grained tracked prefill continuation needs
   the saved native intent/barriers above and explicit uncertain-native-command
   behavior; a codec alone does not implement it.

Source `prepare` remains independently requested source-only work
(`prepare.rs:115–201`): fetch, select source child, execute trusted preparation
programs sequentially, report exact source goal. Current retry reruns preparation;
there is no durable installer health/command history. With a good prepared archive,
receiver operations need not fetch/build or repeat that preparation. Feature
layering remains explicitly later ordinary JJ work, as the
[maintained guide](file:///home/rektide/src/jj-waltz/docs/reseed.md#reseed-an-existing-workspace)
specifies.

JW source inspected at `3b99f1d70a64c1251ccae9f4ca1942751671fe8a`; the named current
JW files have no working-copy delta. Native source inspected at
`803ef00a0a6ee4ddc275d3138d5f9e7326399906`, newer than runtime. Targeted runtime-to-
source diff shows sparse command code unchanged and `git.rs` HEAD/index routines
unchanged; broader snapshot/sparse-ignore behavior is not assumed identical.
Actual behavior claims above come from this runtime case or attributed completed
evidence, not version-number guesses. No other existing-target cases were run.

Only this model-suffixed document is committed. The scoped ticket receives its
pointer/findings/session; parent owns closure, export, indexes and integration.
Production code/defaults/guards and prior scratch remain untouched.
