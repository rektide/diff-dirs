---
type: ArchitectureStudy
title: Native colocation and CoW are independent creation axes
description: Real-JJ convergence evidence and a bounded integration contract for archive-backed Git-colocated receiving workspaces, retaining useful reflink prefill without copying donor identity.
resource: /design/native/colocation0.gpt61sxh.md
status: independent-research
generated: { by: "model:openai/gpt-6.1-sol#xhigh", at: 2026-10-20 }
sources:
  - resource: "opencode://ses_ea7e35793ffenYg0DnsiUG8p3e"
    title: Native creation, real-reflink and tree-state convergence experiments; source inspection; default and reseed compatibility boundary
  - resource: /design/roadmap/brief0.gpt6a.md
    title: Independent commission, preferred default and reading boundary
  - resource: /design/native/metadata0.gpt6am.md
    title: Prerequisite archive and receiver metadata study, read from the artifact introduced at 014db6e3
  - resource: "https://github.com/jj-vcs/jj/blob/803ef00a0a6ee4ddc275d3138d5f9e7326399906/cli/src/commands/workspace/add.rs"
    title: Native workspace-add sequencing, colocation policy and parent selection
  - resource: "https://github.com/jj-vcs/jj/blob/803ef00a0a6ee4ddc275d3138d5f9e7326399906/lib/src/git.rs"
    title: Native Git worktree creation, first-parent HEAD, merged-parent index and unlink behavior
  - resource: "https://github.com/jj-vcs/jj/blob/803ef00a0a6ee4ddc275d3138d5f9e7326399906/lib/src/git_subprocess.rs"
    title: Native orphan worktree creation command and configured Git executable
  - resource: "https://github.com/jj-vcs/jj/blob/803ef00a0a6ee4ddc275d3138d5f9e7326399906/lib/src/local_working_copy.rs"
    title: Private tree state, sparse patterns, Watchman clock and stat-based snapshot trust
  - resource: "https://github.com/jj-vcs/jj/blob/803ef00a0a6ee4ddc275d3138d5f9e7326399906/CHANGELOG.md"
    title: Unreleased workspace-colocation feature and Git 2.42 orphan-worktree requirement
  - resource: "file:///home/rektide/src/jj-waltz/src/workspace.rs"
    title: Current sparse-empty creation, tracked clone, state adoption and rollback integration
  - resource: "file:///home/rektide/src/jj-waltz/src/lifecycle.rs"
    title: Current exclusive creation modes and shared add/switch creation policy
  - resource: "file:///home/rektide/src/jj-waltz/src/ownership.rs"
    title: Creation-minted Git ownership and precisely bounded cleanup
  - resource: "file:///home/rektide/src/jj-waltz/src/reseed/engine.rs"
    title: Current colocation and owned-worktree rejection at the reseed boundary
  - resource: "file:///home/rektide/src/diff-dirs/.test-agent/native-colocation/README.md"
    title: Owned tiny fixtures, exact commands, raw output, state captures and cross-case checks
extensions:
  ticket: diffdir-tree-native-colocation
verification:
  runtime: "jj 0.45.1-33d3a595429b7e3b54a8ea769560247913bc8a1b; Git 2.55.0; Linux btrfs"
  scope: "Ten successful native receiving-workspace cases: eight core/root cases and two clean-merge control/composition cases; no production code or defaults changed"
---

# Native colocation and CoW are independent creation axes

**Yes: actual native JJ can create a Git-colocated receiver from the JJ archive,
then converge a tracked reflink prefill plus donor `tree_state` to the requested
tree without replacing an unchanged prefilled file.** This worked with both a
non-colocated archive's internal bare Git store and a colocated archive's root
Git store. No hand-built `.git` metadata or Git reset was necessary.

JW's current CoW/colocation exclusion is **local policy**, not a restriction
demonstrated by this native path. Removing that exclusion alone is nevertheless
not a coherent implementation: CoW error/fallback paths presently drop the
new Git ownership record, and **current reseed rejects colocated targets**.
The latter is a product/lifecycle compatibility boundary, not a failed native
materialization experiment.

## Coordinates and evidence boundary

- Runtime: real `/usr/local/bin/jj`, version
  `0.45.1-33d3a595429b7e3b54a8ea769560247913bc8a1b`; `/usr/bin/git` 2.55.0.
  The mise launcher was not used.
- Native checkout inspected: `/home/rektide/archive/jj-vcs/jj`, Git HEAD
  `803ef00a0a6ee4ddc275d3138d5f9e7326399906`, newer than the runtime commit.
  The runtime commit object is also present locally. A retained targeted
  `git diff 33d3a595... 803ef00a... --` of `workspace/add.rs`, `git.rs`,
  `git_subprocess.rs`, and `local_working_copy.rs` establishes that the relevant
  create-worktree/reset-index implementation is unchanged apart from surrounding
  refactoring; workspace-add's only delta is enum visibility. Sparse-ignore
  snapshot behavior **does** differ elsewhere in `local_working_copy.rs`.
  This is not a claim that all behavior of the two revisions is identical.
- JW source: `/home/rektide/src/jj-waltz`, Git HEAD
  `3b99f1d70a64c1251ccae9f4ca1942751671fe8a`. Its starting unrelated research
  document change was left alone. No JW source was changed or suite run.
- Prerequisite: [metadata study](/design/native/metadata0.gpt6am.md), introduced
  at `014db6e3`. Its sparse-empty Git-index observation was treated as an input,
  not as proof of successful composition. Its fixture stayed read-only.
- Commission: [independent brief](/design/roadmap/brief0.gpt6a.md) and the scoped
  ticket. No withheld architecture, full roadmap waves, parent transcript, or
  same-wave workspace-source research was consulted. The parent's later pointer
  to reseed's rejection was independently checked in JW source.

## Composition: native identity, optional payload, native convergence

Use five boundaries rather than a single exclusive creation mode:

1. **Native creator** allocates the receiving JJ workspace and, if selected,
   its Git linked worktree. For prefill, use `--sparse-patterns empty` to avoid
   native tracked-file materialization. Pass the colocation decision explicitly.
2. **Ownership recorder** captures the newly allocated Git administration and
   mints JW's ownership token before any optional materialization work can fail.
3. **Tracked prefill** installs only donor-tracked regular files and symlinks,
   using real reflinks and preserving relevant file state. It must preserve
   receiver root `.jj` and `.git` from both source copying and target deletion.
4. **Private state adapter**, when supported, copies only donor
   `.jj/working_copy/tree_state`, not `checkout`, the repository pointer or Git
   metadata. Donor sparsity/file state describes the prefill, not its identity.
5. **Native converger** runs receiver `jj status`, then plain `jj restore`.
   Status snapshots the supplied donor tree into the receiver's own working-copy
   change; restore reconciles that tree to the receiver's requested parent tree.
   Verify the resulting identity, parents and tree, plus native Git semantics.

For one desired parent `D`, the final receiver is a **new** working-copy change
whose parent is `D` and whose tree is `D`'s tree. It is not the donor's working-copy
commit. Git HEAD is `D`, not that new JJ commit; the index represents its parent
tree. The donor may contain a different tree `S`. Native restore removes
source-only entries, creates desired-only entries, rewrites differing entries,
and leaves identical entries alone.

These are ordered mutations, **not an atomic transaction**. `status` can record
an intermediate donor-shaped tree and operation. A command error does not prove
absence of effects. Restoring a saved `tree_state` blob alone is not repository
rollback. New-workspace cleanup must use the actual receiving ownership record;
an uncertain outcome should retain the checkout and explain what remains.

### Native metadata stays target-owned

For archive `A` and receiver `W`:

```text
A/.jj/repo/                         shared JJ repository
W/.jj/repo                         native text pointer to A/.jj/repo
W/.jj/working_copy/checkout         W's workspace name and operation binding
W/.jj/working_copy/tree_state       optional, private state-adoption seam
W/.git                             native gitfile, when colocated
G/worktrees/<native-registration>/  W's HEAD/index/gitdir/commondir
```

`G` is either `A/.git` or `A/.jj/repo/store/git` in the tested layouts.
The archive does **not** need a root `.git` to create a colocated receiver.
Never clone the archive's `.jj` or `.git` into `W`. In particular, an absent
donor root `.git` must not become an instruction to delete `W/.git`.
The [metadata study's copy boundary](/design/native/metadata0.gpt6am.md#copy-boundary-new-versus-existing-receiver)
is a required input to a generic installer, not an optional performance detail.

## Installed-runtime experiments

### Fixture and procedure

All writable repositories were fresh, explicitly initialized, tiny repositories
under `file:///home/rektide/src/diff-dirs/.test-agent/native-colocation/`.
The script validates that exact root and refuses an existing run directory.
There is no deletion, reuse of another session's fixture, live archive mutation,
maintenance, sweep or benchmark. This filesystem is btrfs; `/tmp/opencode` is
tmpfs, so it was not used for reflink claims.

Each source tree contains approximately 8 KiB plus short text files: unchanged
`common.bin`, changed `change.txt`, executable `run.sh`, symlink `alias`, and a
donor-only or desired-only file. The merge edge adds one short file. Workload
payloads remain far below 256 entries / 1 MiB per case; generated native
repository administration and command logs are evidence, not imported payload.

The runner builds sibling revisions `S` and `D` from base `B`, returns the
archive to `S`, gives tracked files old timestamps, and snapshots them with
native JJ. For each prefilled receiver it:

1. Runs `jj workspace add --name NAME -r D --colocate
   --sparse-patterns empty DEST` (or `S`, root, or the stated control flags).
2. Records native JJ/Git identity, parent/tree entries, sparse patterns,
   Git HEAD/index/status/registration and both local working-copy state files.
3. Copies only the native `file list -r @` donor leaves. Regular files use Linux
   `FICLONE` ioctl `0x40049409`, with **no byte-copy fallback**; symlinks are
   recreated. `copystat` preserves mode/size/mtime. It checks separate source and
   target inodes, correct attributes, and the payload bound.
4. Writes the donor's **tree_state bytes only**, verifies receiver `checkout`
   bytes were untouched by that write, and retains both before/after blobs.
5. Runs receiver `jj status`, records the intermediate JJ/Git observations,
   then runs plain `jj restore` and another `jj status`.
6. Checks final receiver name and native registry root, unchanged receiving
   change ID, requested parent IDs, expected native tree entries, empty
   `jj diff -r @ --summary`, preserved repository/gitfile relationships, Git
   registration, and retained `common.bin` inode where it belongs in the result.

Every subprocess uses an allowlisted environment, isolated HOME/XDG/JJ config,
`GIT_CONFIG_NOSYSTEM=1`, `GIT_CONFIG_GLOBAL=/dev/null`, and `fsmonitor.backend=none`.
Git inspections use `--no-optional-locks`. Native help for templates/revsets was
read before the corrected experiment's queries. No Git repair/reset or manual
Git administration was used to force convergence.

### Results

| Archive / creation | Requested parent(s) | Result after native convergence |
|---|---|---|
| Internal Git; ordinary `--colocate` | `D` | Correct new JJ identity/tree; detached Git HEAD=`D`; clean Git |
| Internal Git; ordinary `--no-colocate` | `D` | Correct JJ identity/tree; no receiving `.git` |
| Internal Git; reflink + state; colocated | `S`, equal donor/desired tree | Correct tree; clean Git; unchanged prefill inode retained |
| Internal Git; reflink + state; colocated | `D`, different tree | Correct tree; source-only removed, desired-only created, changed bytes replaced; clean Git; unchanged inode retained |
| Internal Git; reflink + state; opt-out | `D` | Same desired JJ tree/content/exec/link result; no `.git`; unchanged inode retained |
| Root-colocated Git; reflink + state; colocated | `S` | Same successful equal-tree composition |
| Root-colocated Git; reflink + state; colocated | `D` | Same successful different-tree composition |
| Internal Git; reflink + state; colocated | JJ root | Empty desired tree/index; clean Git; HEAD legitimately unborn at `refs/jj/root` |
| Internal Git; ordinary colocated control | `D`, `M` clean merge | Correct merged tree; HEAD=`D`; index=merged parents; staged `A  merge-only.txt` |
| Internal Git; reflink + state; colocated | Same `D`, `M` | Same native tree/index/status as ordinary control; unchanged inode retained |

All eight core/root cases passed their assertions. Both merge cases and their
cross-case assertions passed. The final read-only check verified all ten JJ
workspace registry paths and receiving repository pointers.

For different-tree internal-Git composition, retained evidence includes:

```text
S = 1fda9e73a5d871721d08f73f33267034ceaafa68
D = ef9839f91beee56e1033af3e697d45f1a8c56b3b
receiver = cow-different
common.bin inode after prefill = 297250735
common.bin inode after status/restore/status = 297250735
final Git HEAD = D; final Git status and cached diff = empty
```

The final internal-Git `D` case's complete index entries match the ordinary
colocated `D` control, not merely an empty status display. Native tree entries,
file content hashes, executable state and symlink target also match the ordinary
and opt-out controls.

**The sparse-empty intermediate state is not clean Git.** Initially the Git
index contains desired-parent files while the payload is absent, so status shows
deletions. After adopting the different donor tree and running JJ status, Git
reported modified `change.txt`, deleted `desired-only.txt`, and intent-to-add
`donor-only.txt`. Restore eliminated these differences. For an equal tree the
prefill/status stage was already clean, and restore did not replace common files.

**Full POSIX permissions are not a JJ/Git tree property.** In this fixture the
donor's unchanged `common.bin` had mode `0600`; prefill preserved it, whereas
ordinary native materialization produced `0644`. Comparing all mode bits to an
ordinary checkout is therefore a false conformance criterion. Contents,
executable state and links agreed. This route preserves donor attributes on
unchanged leaves; it does not promise a full permissions-normalization checkout.

### Root and clean-merge interpretation

A root-parent receiver has no Git parent commit. `git rev-parse --verify HEAD`
fails by design while `git symbolic-ref HEAD` gives `refs/jj/root`. This is a
valid clean native result, not a reason to invent a commit or branch.

For multiple parent revisions, native JJ uses the **first** parent as Git HEAD
but the **merged parent tree** as Git index. Thus even an unchanged clean JJ
merge working copy can show staged changes relative to Git HEAD. Both ordinary
and reflink routes showed exactly `A  merge-only.txt`; Git's unstaged diff and
JJ's working-copy diff were empty. Do not "repair" that with `git reset`.

Current JW resolves one creation-base commit and passes one `-r`; it does not
expose this multi-parent workspace-add edge. Creating on a merge **commit** is
still one parent, not this two-parent fixture. The edge establishes the native
verification boundary; it is not evidence that ordinary current JW additions
will all appear Git-dirty. Conflicted trees were not tested.

## Why native composition works, and what it costs

At native source `803ef00a`:

- `cli/src/commands/workspace/add.rs:132–176,193–277` creates native metadata,
  applies sparse inheritance and creates a new JJ working-copy commit with the
  selected parent(s). It rejects populated destinations before its lower-level
  Git helper is reached. Metadata must precede prefill; do not prefill first
  and assume workspace-add will adopt that directory.
- `lib/src/git.rs:1853–1935` and `git_subprocess.rs:297–321` create an orphan
  Git worktree without payload checkout; Git owns its gitfile/administration.
  The native helper's populated-directory branch is not permission for CLI
  `workspace add` into a prefilled destination.
- `lib/src/git.rs:2000–2052,2087–2139` resets HEAD to the first parent
  (or the unborn placeholder when that parent is root) and builds an index from the merged parent tree,
  adding intent-to-add entries for working-copy additions. That explains the
  intermediate Git state and successful native convergence.
- `lib/src/local_working_copy.rs:1142–1178` loads tree IDs, file states, sparse
  patterns and Watchman clock from private `tree_state`. `checkout` is separate
  identity/operation state. `1936–1956` trusts matching file state only when the
  recorded file mtime precedes the state file's own mtime. Preserving useful
  donor attributes can save JJ hashing; simply reflinking bytes is insufficient.

These mechanisms show **useful retained prefill**, not a benchmark or a
zero-I/O guarantee. Git index construction/import/export still does work;
native reset-index enumerates index entries and can write the index even when
the tracked payload survives. Git tools may hash files to refresh their own
stat knowledge. Different desired trees require real native file changes.
State adoption and restore may add JJ operations, even for creation. No evidence
here quantifies whether Git colocation's cost dominates a large workload.

The private adapter is version-sensitive. Successful decoding/status is not a
portable public API guarantee, nor sufficient final conformance by itself.
Keep a native ordinary-materialization path. Watchman clocks describe their
recording checkout: JW already excludes Watchman for tree-state reuse, and this
study does not weaken that exclusion. Sparse-ignore differences between source
revisions, conflicted materialization, submodules, other platforms/filesystems,
filters and unusual configurations remain outside these tiny tests.

## JW integration and lifecycle compatibility

### Current behavior is deliberately exclusive

`src/lifecycle.rs:126–152` currently resolves exactly one of `CopyOnWrite`,
`JjOnly`, and `Colocated`:

- Unset config/flags means **CoW and JJ-only**, independent of native inheritance.
- Positive CLI `--colocate` selects ordinary colocated materialization even if
  config requests CoW. Positive CLI `--cow` selects non-colocated CoW even if
  config requests colocation. Both positive CLI flags together error.
- Otherwise per-field CLI opt-outs overlay config. Configured colocation and
  explicitly enabled effective CoW conflict; colocation with unset CoW selects
  ordinary colocated creation.
- `--no-colocate` remains a useful CoW/JJ-only route; `--no-cow` alone means
  ordinary non-colocated tracked checkout when colocation is not requested.

`src/workspace.rs:488–549` already accepts independent `clone_from` and
`colocate` fields and issues the necessary native flags. `616–659` already
implements tracked prefill, tree-state-only adoption, status, and restore.
`src/cow.rs:37–44,156–177` preserves mode/mtime and delegates listed leaves to
the strict reflink-required installer. The exclusive lifecycle policy prevents
these pieces from being exercised jointly today; no native ban was found.

### Two currently unreachable cleanup paths must change together

In `workspace.rs:558–573`, the clone fallback calls
`rollback_workspace_parts(..., None)` instead of passing `owned_git_worktree`.
The clone-error helper at `662–665` also passes no owner. If joint mode is merely
enabled, these paths encounter an unowned linked Git checkout and can fail
cleanup rather than doing the intended fallback. They are not evidence that
native convergence fails; they are a concrete integration blocker to the
one-line policy change.

Retain `ownership.rs`'s creation-minted token, registration snapshot, common-dir
and backlink validation. Thread that owner through **all** pre-metadata-record
failure and recreate paths. Ordinary fallback must retain the selected
colocation choice, not silently turn a default-on receiver JJ-only.

Native `git.rs:1947–1978` unlinks the gitfile then globally prunes worktree
administration. JW's `ownership.rs:303–429` avoids that broad prune by detaching
only its validated gitfile and deleting only its exact validated administration.
Do not replace this with native forget directly or relax ownership checks to
make a new combination succeed. No removal/prune experiment was run here.

### Reseed is a separate compatibility boundary

Direct source confirmation: `src/reseed/engine.rs:790–795` rejects **either**
root `.git` **or** `owned_git_worktree`/`external_owner` metadata. Consequently
default-created colocated workspaces will be ineligible for **current reseed**,
despite successful native creation and CoW convergence.

A production default decision must explicitly choose one of:

1. **Creation-only landing with acknowledged boundary:** default-on for new
   supported receivers; reseed workflows deliberately use `--no-colocate`.
   Update the usage contract/examples so this is not a surprise loss of reuse.
2. **Preserve reseed availability for the new default:** commission a separate
   colocated-reseed adaptation before enabling the default. It must preserve
   the existing receiver's `.jj`/`.git` identity, Git HEAD/index and ownership
   throughout its own transitions and failures. Creation evidence does not
   discharge that work, and deleting the guards is not an adaptation.

This study does not authorize or implement either production change. Native
restrictions do not defeat the preferred creation default; the existing reseed
contract may legitimately determine its landing order or require retaining the
opt-out for those workflows.

## Proposed capability/default/flag contract

The feasible policy is two independent axes, with **CLI > JW config > product
default** for each. Recommended product defaults on supported repositories are
`colocate=true`, `copy_on_write=true`; CoW remains a best-effort tracked-prefill
optimization under its existing capability/fallback policy, not a prerequisite
for colocation. This is a proposed contract, not current behavior:

| New workspace request, absent contrary config | Native topology | Tracked materialization |
|---|---|---|
| No flags | Git-colocated | CoW/state route where supported; otherwise ordinary native checkout with warning |
| `--cow --colocate` | Git-colocated | Same CoW request; no cross-axis conflict |
| `--no-cow` | Git-colocated | Ordinary native checkout |
| `--no-colocate` | JJ-only | CoW/state route where supported |
| `--no-cow --no-colocate` | JJ-only | Ordinary native checkout |

Same-axis positive/negative flags still conflict. `--cow` must no longer mean
"override config colocation"; help, precedence tests and configured-both-true
tests must change with the new contract. Explicit `workspace.colocate=false`
is the durable opt-out. JW should continue passing `--colocate`/`--no-colocate`
rather than making source workspace topology or native `git.colocate` choose
its product default.

Native defaults are different: at `workspace/add.rs:149–156` an unspecified
choice colocates only when the invoking workspace is colocated **and** native
`git.colocate` is true. Explicit native flags override this. Relying on native
inheritance would therefore fail the desired archive-backed default for an
internal-bare-Git archive or a JJ-only invoking sibling.

Capability boundaries:

- Probe the **actual JJ executable's** `workspace add --help` for the flag, as
  `src/jj.rs:362–389` already does; require a Git backend. This installed
  `0.45.1-<hash>` build works although JW help/error text says "0.46+".
  The newer source changelog lists the feature as unreleased; a release-number
  guess is not a substitute for probing actual builds.
- The inspected native path requires Git's `worktree add --orphan`; native
  `CHANGELOG.md:15–17` gives Git **2.42.0** as the minimum for that operation.
  Only Git 2.55.0 was run here. Check the Git executable selected by native
  `git.executable-path`, not an unrelated executable found first in a shell.
  JW's current flag/backend probe does not itself check that executable's
  version; an honest before-mutation capability gate must cover it as well.
- Flag presence/backend checks establish ordinary capability, not the private
  state adapter's compatibility. Keep native conformance coverage for supported
  JJ builds and a controlled ordinary checkout fallback. Preserve evidence on
  unverified errors rather than silently overwriting metadata.
- Recommended default failure behavior when colocation is unavailable: fail
  preflight with an actionable `--no-colocate`/config opt-out **before mutation**.
  That makes the new default honest rather than quietly varying topology.
  Opt-out bypasses colocation-only capability requirements and preserves
  compatibility with older JJ or non-Git repositories. An automatic downgrade
  policy would be a distinct product decision, not inferred here.

### Exact add versus switch scope

`src/cli.rs:699–750` constructs the same `CreationPolicy` for `add` and `switch`.
`lifecycle.rs:244–307` creates every requested add target. `319–424` creates
only missing switch names, including intermediate names in a multi-name switch,
using the same preflight, checkout plan and `create_workspace` path.

Therefore changing the shared creation default applies to:

- all new `jw add` receivers, including exact destinations;
- missing final and intermediate receivers made by `jw switch`/its shorthands;
- creation from an explicit source, subject to its own source-validation
  contract, not conversion of that source.

It does **not** convert existing workspaces when switching. If no target needs
creation, `lifecycle.rs:358–375` bypasses creation capability/base resolution;
existing switch should still work without native colocation support. The
manager's `switch_existing_workspace` explicitly disallows creation. This
proposal does not implicitly enable colocated reseed, conversion, adoption,
automatic cleanup, or mutation of the primary archive's topology.

If the authorized product change is strictly **add-only**, parameterize that
default at the command/policy boundary. Do not describe a shared resolver edit
as add-only while silently changing missing-name switch behavior. A deliberate
shared **new-workspace** policy is the smaller consistent implementation, but
its scope must be explicit at acceptance.

## Smallest coherent landing

1. Decide the reseed compatibility/landing-order choice above and the explicit
   add-only versus shared-new-creation scope. No native obstacle requires
   discarding CoW to obtain Git colocation.
2. Separate topology from materialization in policy/preflight; retain both
   opt-outs. Keep native metadata construction, not custom `.git` generation.
3. Make the existing tracked-prefill route accept the topology independently;
   thread receiving Git ownership through clone fallback/error cleanup. Preserve
   both administrative roots in the installer. Do not widen arbitrary payload
   copying or remove private-state/Watchman guards in this slice.
4. Add real-native integration coverage for internal and root-colocated Git
   archives, equal/different desired trees, no-CoW and no-colocation controls,
   retained prefill inode, source/desired file deltas, receiving JJ/Git identity,
   and fallback/error cleanup without touching unrelated registrations. Root
   HEAD is an explicit expected edge. Conflicted/sparse/version cases need
   separate supported-scope decisions, not invented universal success.
5. Change the accepted defaults/help/config tests only after those contracts
   hold. Reseed stays a separate adaptation unless explicitly commissioned.

## Reproduction and retained raw provenance

Local evidence home:
`file:///home/rektide/src/diff-dirs/.test-agent/native-colocation/`.
It contains executable-source `probe.py`, `merge.py`, `analyze.py`, the ownership
README, `run1/` and `run3/` initialized repositories, JSON summaries and exact
argv/stdout/stderr logs, captured `checkout`/`tree_state` blobs, prefill stats,
`registry.json`, `analysis.txt`, and `native-source-delta.txt`.

From diff-dirs, reproduce into **fresh** run names on this btrfs filesystem:

```sh
python3 .test-agent/native-colocation/probe.py run4
python3 .test-agent/native-colocation/merge.py run5
```

The commands refuse existing run roots; no cleanup is needed or requested.
`analyze.py` checks the retained original `run1`/`run3` outputs. To reproduce
its cross-run comparisons against new outputs, select the corresponding new
summary paths rather than overwriting old evidence.

The essential command sequence, independent of the multi-case runner, is:

```text
# In a fresh isolated, explicitly owned tiny JJ Git repository A:
# make B, sibling source S and desired D; return A to S; snapshot known-good S.
/usr/local/bin/jj workspace add --name receiver -r <D-id> \
  --colocate --sparse-patterns empty <absent-receiver-path>
# Copy only A's `jj --ignore-working-copy file list -r @` leaves into receiver.
# FICLONE each regular file; recreate symlinks; preserve mode/size/mtime.
# Record receiver metadata/inodes before adopting any state.
# Write A/.jj/working_copy/tree_state bytes to receiver's tree_state only.
/usr/local/bin/jj -R <receiver> status
/usr/local/bin/jj -R <receiver> restore
/usr/local/bin/jj -R <receiver> status
/usr/local/bin/jj -R <receiver> --ignore-working-copy diff -r @ --summary
/usr/local/bin/jj -R <receiver> --ignore-working-copy debug tree -r @
/usr/bin/git --no-optional-locks -C <receiver> rev-parse --verify HEAD
/usr/bin/git --no-optional-locks -C <receiver> ls-files --stage
/usr/bin/git --no-optional-locks -C <receiver> status --porcelain=v1
```

The runner preserves the exact initializers, sibling-revision construction,
allowlisted environment, actual ioctl, attributes, native identity/tree
queries and assertions needed to run this from a fresh tiny fixture. File
names are deliberately simple; its line-based file-list parsing is **not** a
proposed production path codec. JW's existing JSON-string file-list handling
must not be replaced by that research convenience.

Two harness failures are retained, not hidden native failures: `run0` attempted
the nonexistent installed-JJ template keyword `tree_id` before case creation;
the corrected runner compares complete native `debug tree` entries. `run2`'s
merge control reached correct native creation, then its test tried singular
`diff --from @-` with two parents. The corrected assertion is `jj diff -r @`,
relative to the merged parents; `run3` passed both cases. A later over-strict
cross-case POSIX-mode comparison exposed the documented `0600`/`0644` difference;
the final checker tests the actual JJ/Git executable/content/link contract.

Raw fixtures are local, ignored evidence, not a portable production dependency.
Only this model-suffixed study is committed; shared indexes and production
defaults/code remain unchanged.
