---
type: ArchitectureStudy
title: Registered workspace donors and source binding
description: Separate donor content, repository ownership, workspace identities and ignored payload when JW creates from a registered workspace at another path.
resource: /design/native/workspace-source0.gpt6am.md
status: independent-research
generated: { by: "model:openai/gpt-6-astra#medium", at: 2026-10-20 }
sources:
  - resource: "opencode://ses_ea7e30b42ffegrSPJE47yHB5SX"
    title: Independent source-binding analysis and bounded native registered-donor experiments
  - resource: /design/roadmap/brief0.gpt6a.md
    title: Scoped independent commission
  - resource: /design/native/metadata0.gpt6am.md
    title: Prerequisite metadata study read at commit 014db6e3
  - resource: "https://github.com/jj-vcs/jj/blob/803ef00a0a6ee4ddc275d3138d5f9e7326399906/cli/src/commands/workspace/add.rs"
    title: Native workspace creation binds the current repository and resolves revisions in the source workspace
  - resource: "file:///home/rektide/src/jj-waltz/src/lifecycle.rs"
    title: Creation policy and frozen target-base resolution at JW 3b99f1d70a64c1251ccae9f4ca1942751671fe8a
  - resource: "file:///tmp/opencode/workspace-source-clever-pixel/README.md"
    title: Retained native experiment provenance, script, logs and coordinates
extensions:
  ticket: diffdir-tree-native-workspace-source
verification:
  runtime: "jj 0.45.1-33d3a595429b7e3b54a8ea769560247913bc8a1b; Git 2.55.0; Linux"
  scope: "Native registered-donor source binding; JW source and test inspection, not execution"
---

# A registered workspace is a useful donor, not a new repository owner

**Yes: existing `jw add --from PATH` already selects a registered workspace
as its source context.** Its working files can differ from the archive, and
its current commit can supply the new target's parent. The target remains a
workspace of the donor's *resolved repository*, with a fresh native identity.
The physical archive continues to own shared storage and some JW defaults.

This is useful for a prepared dependency installation or a working branch
located away from the archive. It is not a request to copy the source's `.jj`
or `.git`, nor an import into the invoking directory's repository.

## Coordinates and evidence boundaries

JW source was inspected in `/home/rektide/src/jj-waltz` at Git HEAD
`3b99f1d70a64c1251ccae9f4ca1942751671fe8a`. The only working-tree diff reported
at inspection was an unrelated design document; inspected source/tests were
unchanged. Native source is `/home/rektide/archive/jj-vcs/jj`, Git HEAD
`803ef00a0a6ee4ddc275d3138d5f9e7326399906`, newer than the installed runtime.
Source-derived expectations and runtime observations below remain distinct.

The prerequisite [metadata study](/design/native/metadata0.gpt6am.md), read
at `014db6e3`, owns primary layouts, native metadata construction and the
common/private Git distinction. Here the source is a registered receiver at
another path. No parent architecture, full roadmap, concurrent colocation
study or parent transcript was consulted. No live repository was mutated by
the experiments, no JW binary was run or built, and no test suite was run.

## Source-binding model

Use these independently named coordinates in creation planning:

| Coordinate | Meaning and authority |
|---|---|
| `S`, `Ns` | Canonical donor checkout root and its registered native workspace name. Resolve from `--from`, otherwise the creation command's current workspace. |
| `R` | Resolved JJ repository directory, reached through `S/.jj/repo`. Owns shared operations, commits and workspace registry. |
| `A` | Physical primary checkout containing `R` at `.jj/repo`, if discoverable. Supplies current JW placement and local ignored-policy defaults; need not equal `S`. |
| `Cs` | Donor working-copy commit after any required snapshot; identifies tracked donor content, not installed payload. |
| `B` | Frozen target parent commit. Implicit JW selection is `Cs`; explicit `--at` can select another commit. |
| `T`, `Nt`, `Ct` | New root, new native workspace name and newly created working-copy commit, with parent `B`. |
| `G`, `Gs`, `Gt` | Git common store and optional source/target private worktree administration. Colocated source and target share `G`, never private administration. |
| `P` | Explicitly selected source-local ignored payload, copied from `S` under its own policy. It is not contained in `Cs` or implied by `B`. |

The fundamental relationship is `repository(S) = repository(T) = R`, not
`S = A`. The destination's `.jj/repo` points directly to `R`; it does not
chain through `S/.jj/repo`. A source `.git` gitfile is a reference to `Gs`,
not a template for `Gt`. Preserve native target metadata and exclude donor
root metadata throughout payload transfer, as specified by the prerequisite.

This model needs no arbitrary-directory equivalence. A valid donor has a
usable JJ workspace context and registration; a mere directory with files,
an orphaned copied `.jj`, or a Git-only worktree is not established as such a
source. A source in another repository changes which repository receives the
new workspace; existing `--from` is not cross-repository content injection.

## Actual CLI meaning and operation sequence

`src/cli.rs:781–803` normalizes `--path` against the caller **before** changing
directory to `--from`. Everything subsequently loading current JJ context
uses that source. This also means source-relative `@`, registered-name lookup,
repository metadata and capability checks belong to `S`/`R`.

`src/lifecycle.rs:651–684` distinguishes these cases:

| Request | Current creation meaning |
|---|---|
| No `--at` | Run source `status`, freeze an operation, resolve source `@` to `B`. Donor edits included by snapshot can become target content. |
| `--at X` only | Resolve `X` at a captured operation without first forcing a snapshot in this resolver. Pin that target base before later native creation. |
| `--source-at Q` | Snapshot source, require its tracked tree to equal `Q`'s tree, and retain source commit identity for payload planning. Equal trees suffice; commit identities may differ. |
| Both assertions | `Q` checks donor content; `X` independently chooses target parent. Neither replaces the other. |

An explicit `--at @` is therefore not precisely the same snapshot contract as
omitting `--at`. Later native commands may snapshot the source, but do not
retroactively change the already frozen `B`. This matters when testing a
donor with initially unsnapshotted edits.

`workspace.rs:488–519` always passes native `--revision B`. This differs
from omitting `-r` on native `jj workspace add`, which uses source `parents(@)`
(`cli/src/commands/workspace/add.rs:224–252`). A probe using bare native add
would otherwise test the wrong JW semantics.

Native add uses the current workspace's resolved repository, independently
initializes the target and registers its name/path (`add.rs:165–175`;
`lib/src/workspace.rs:374–412`). Sparsity defaults come from the selected
source workspace, not the archive. A future interface should carry that
materialization choice alongside content identity; commit equality alone
does not say which files are physically present in a sparse donor.

For current CoW mode, `lifecycle.rs:690–710` chooses `inventory.current_root()`
as the donor. `workspace.rs:616–659` copies donor tracked files, adopts only
donor tree state, then restores target `@` to the creation base. Thus donor
bytes are a materialization input, not authority to override explicit `B`.
This study does not validate that private-state optimization or its
colocation composition; those belong to the separate study.

## What remains archive-specific today

There is **no blanket primary-only source rule in add**. There are several
specific primary/default roles which must not silently move to the donor:

- `WorkspaceInventory::load`, `workspace.rs:119–140`, derives `A` from real
  repository ownership, separately from the registered workspace named
  `default`. `context.rs:519–537` resolves directory or text-file `.jj/repo`.
- Default target placement uses `A`'s parent and basename
  (`workspace.rs:869–880`). A source at an unrelated path does not relocate
  automatically named siblings beside itself. Explicit `--path` overrides it.
- Local ignored-copy policy comes from `A/.jwreseed.local.toml`, verified by
  `primary_policy_root` (`workspace.rs:167–180`), not from the donor's file.
  Precedence is disable > CLI > primary-local > global > empty.
- Creation reads only that file's copy list. Its `source`, `source_at` and
  freshening fields do not select a creation donor or prepare it.
- Links use `inventory.default_name()`'s root (`lifecycle.rs:270`), which can
  be a different role again: a workspace literally named `default` wins.
  `--no-links` avoids this dependency. It must not be conflated with either
  physical owner policy or the selected payload donor.

If `A` cannot be verified, ordinary defaults can fail even with a usable
registered source: automatic placement and local ignored policy need `A`.
The explicit escape hatches are a literal name plus `--path`, CLI ignored
selection or `--no-copy-ignored`, and `--no-links` if no default root exists.
That composition is source-derived, not tested here; detached/custom repository
layouts should receive a focused test before being advertised.

## Installed payload is donor-local, selection policy is not

`src/lifecycle/ignored.rs:98–165` selects roots under `S`, snapshots it and
freezes its tracked entries and ignore rules. It also checks selected paths
against `B`: they must not overlap target tracked entries and must remain
ignored in the desired target tree. Native add alone does not transfer them.

`src/cow/payload.rs:145–165` constructs ignore rules from frozen **tracked**
`.gitignore` contents plus backend/global excludes. Untracked donor-local
ignore files are deliberately not discovered. Changing from archive to
registered donor therefore changes selected bytes and potentially tracked
ignore contents, without changing the primary-local copy list.

Actual private payload copying requires real CoW, independently of tracked
checkout mode (`ignored.rs:231–244`). `--no-cow` or `--colocate` does not give
ignored installations a byte-copy fallback. No selected matches is success;
selected roots need the existing topology, ignore and transfer conditions.
This study did not exercise that transfer or build a CoW matrix.

An installation valid for `Cs` need not be valid for a different `B`. Existing
ignore/overlap checks establish placement eligibility, not dependency-lockfile
compatibility. Keep this as an explicit caller/application responsibility or
a future payload compatibility policy, rather than calling all source-local
files revision-equivalent. Source quality and concurrent writers are outside
this commission; neither new locking nor automated preparation is proposed.

## Bounded native observations

Fixture: [script and retained evidence](file:///tmp/opencode/workspace-source-clever-pixel/README.md).
One internal-Git owner, four registered receivers, 221 entries and 30,565
regular-file bytes including native metadata. Isolated configuration and
explicit `/usr/local/bin/jj` were used. All assertions passed.

| Operation | Observed result |
|---|---|
| Create JJ-only donor at unrelated sibling path; change tracked file from `archive` to `registered donor`; add ignored cache | Donor `@` differs from owner; owner working file remains `archive`. |
| From that registered donor, add explicit colocated target at frozen donor `@` (JW implicit-base equivalent) | Target receives donor tracked bytes; parent is donor commit; no ignored cache arrives. |
| Modify that colocated receiver to `colocated donor`, then use it as source for JJ-only target at owner's frozen base | Target receives `archive`, has no `.git`, and parent is the explicit base. Both donor working files/commit identities remain intact. |
| From first donor, omit native `-r` | Target receives `archive`, parent equals donor's parent, demonstrating the native/JW omission difference. |

All four receiver `.jj/repo` files contain `../../owner/.jj/repo`, directly
referencing the owner. The colocated receiver's private Git administration is
`owner/.jj/repo/store/git/worktrees/col-target`; its common store is
`owner/.jj/repo/store/git`. Using it as donor for another workspace did not
turn it into a repository owner or pass its private administration onward.

These observations establish native source binding for JJ-only → colocated
and colocated → JJ-only creation with differing source/target content. They
do not establish JW execution, arbitrary native versions, sparse donors,
source tree-state transplantation, or default-on CoW+colocation.

## Supported paths, gaps, and smallest next changes

**Already supported in inspected JW:** registered source selected by `--from`
or current directory; implicit donor `@` or independent explicit base; caller-
relative explicit destination; primary-owned ignored policy applied to donor-
local payload. Existing tests directly encode these contracts:

- `tests/cli.rs:160–188`: registered alternate donor revision and caller-relative
  destination, using explicit `--at @` after snapshotting donor content.
- `tests/creation_ignored.rs:242–318`: equal-tree source assertion and independent
  explicit target base, including dirty donor snapshot behavior.
- `tests/creation_ignored.rs:363–409`: physical primary policy versus a donor
  named `default`, dirty donor bytes and donor-local cache. This test requires
  CoW and may skip on unsupported filesystems.
- `tests/colocate.rs:216–237`: creation from both JJ-only and colocated siblings
  currently remains JJ-only by default. Tests were read, not run here.

**Not supported as interchangeable sources:** arbitrary ordinary directories,
Git-only worktrees without JJ identity, source installations inferred solely
from a revision, source private Git/JJ metadata copied as target identity, or
cross-repository imports under the existing `--from` meaning.

**Preferred default-on colocation:** native registered-source bindings show
no new owner-versus-donor obstacle for the tested ordinary Git-backed cases.
Make target colocation an explicit target policy, not donor `.git` inheritance.
Current `CreationMode::resolve` defaults to CoW/JJ-only and rejects positive
CoW+colocation (`lifecycle.rs:126–151`). Changing that combination requires
the separate colocation work; this return does not claim it solved. JJ's
native default itself depends on source colocation and `git.colocate`, so it
cannot implement an unconditional JW default by simply omitting both flags.

The smallest discriminating additions are:

1. Add one JW integration test selecting a registered JJ-only donor at another
   path with unsnapshotted tracked edits: omit `--at`, request `--colocate`,
   disable ignored copying and links, and assert donor parent/content, direct
   owner pointer and fresh target Git administration. Then use that target as
   a colocated donor with `--no-colocate --at <owner-base>`. This tests actual
   JW composition of the two native observations without CoW.
2. Add an explicit-`--at @` versus omitted-`--at` dirty-donor regression to
   preserve the existing freezing distinction, or deliberately change/document
   that contract if product intent wants equivalence.
3. Extend the existing primary-policy test with distinct source and target
   tracked ignore rules: selected donor cache accepted when target ignores it,
   rejected when target tracks/unignores it. Keep CoW execution conditional;
   eligibility checks should be independently assertable.
4. Carry a resolved source binding (`S`, `Ns`, `R`, `Cs`) and target intent
   (`T`, `Nt`, `B`, colocation, sparsity) through a future creation API rather
   than reconstructing donor meaning from ambient cwd. Keep primary policy
   and link roots as separately resolved fields. Existing CLI flags suffice
   for ordinary registered donors; no new user-facing source switch is needed.

Follow-up for source-local data should verify sparse materialization and
monitor/tree-state assumptions using the selected donor, not its archive.
The metadata prerequisite supplies the protected-root boundary; the separate
colocation study supplies state-adoption feasibility. Neither requires making
every registered checkout equivalent to the archive in policy or payload.
