---
type: ArchitectureStudy
title: Archive and receiving-workspace metadata topology
description: Native JJ ownership boundaries for archive-to-workspace copying, with exact-runtime probes of colocated and internal Git backing stores.
resource: /design/native/metadata0.gpt6am.md
status: independent-research
generated: { by: "model:openai/gpt-6-astra#medium", at: 2026-10-20 }
sources:
  - resource: "opencode://ses_ea7e8f705ffe474lRe5TDujfPe"
    title: Independent native source reading and isolated real-JJ metadata probes
  - resource: /design/roadmap/brief0.gpt6a.md
    title: Scoped independent research commission and reading boundary
  - resource: "https://github.com/jj-vcs/jj/blob/803ef00a0a6ee4ddc275d3138d5f9e7326399906/cli/src/commands/workspace/add.rs"
    title: Native workspace creation control flow
  - resource: "file:///tmp/opencode/native-metadata-shiny-rocket/README.md"
    title: Local reproducible fixture provenance and retained output
extensions:
  ticket: diffdir-tree-native-metadata
verification:
  runtime: "jj 0.45.1-33d3a595429b7e3b54a8ea769560247913bc8a1b; Git 2.55.0; Linux"
  scope: "Two initialized one-file primary repositories; six successful native workspace creations; existing-target rejection; metadata and Git index inspection"
---

# Native metadata is target identity, not donor payload

The ordinary archive owns the JJ repository. Receiving workspaces share that
repository through a pointer, but have independent local working-copy identity
and state. Colocated receivers additionally have independent Git worktree
administration pointing to the archive's Git common store. **Create these links
through JJ; preserve them during filling or reseeding; exclude source root `.jj`
and `.git` from generic copying.**

Native creation works on the installed binary from both tested archive layouts,
including explicit `--colocate` from an archive with an internal bare Git store.
The archive itself need not become colocated. This establishes the metadata
construction path, not the correctness or cost of later CoW/state adoption.

## Coordinates and evidence strength

Native source inspected at Git HEAD
`803ef00a0a6ee4ddc275d3138d5f9e7326399906` in
`/home/rektide/archive/jj-vcs/jj`; installed runtime is separately
`/usr/local/bin/jj`, version
`0.45.1-33d3a595429b7e3b54a8ea769560247913bc8a1b`.
The source is newer; source behavior below is not silently attributed to the
installed build. Runtime facts come from the fixture described at the end.

The term archive here means the primary JJ checkout which contains the real
`.jj/repo` directory. It is not a tarball or necessarily a bare repository.
Its backing Git repository may be bare while the JJ archive has working files.

## Actual layouts

Let `A` be the archive root, `W` a receiving workspace, `R=A/.jj/repo`,
and `G` the Git common directory. Arrows below denote resolved paths, not symlinks.

```text
Primary A, colocated                     Primary A, non-colocated
  .jj/repo/ = R                            .jj/repo/ = R
    store/type = git                         store/type = git
    store/git_target = ../../../.git         store/git_target = git
    ...JJ shared stores...                   store/git/ = G (bare)
  .jj/working_copy/                        .jj/working_copy/
  .git/ = G (non-bare)                     no root .git
  payload                                 payload

Receiver W, either source layout
  .jj/repo [text file] -----------------> R
  .jj/working_copy/
    checkout       own operation ID and workspace name
    tree_state     own tree/file-stat/sparsity state
    type = local
  payload

Additional links when W is colocated
  W/.git [gitfile] ---------------------> G/worktrees/<registration>/
                                           HEAD
                                           index
                                           gitdir ---------> W/.git
                                           commondir ------> G
                                           logs/, refs/ as applicable
```

`R` contains JJ's `index`, `op_store` (operations/views), `op_heads`,
`store` (Git backend plus JJ `extra` metadata), `workspace_store`, and
`submodule_store` in these fixtures. These are shared repository state, not a
second copy beneath every workspace. Repository-level configuration or JW's
repository metadata, when present, also belongs to this shared owner.

`G` contains Git objects, common refs and configuration. A primary colocated
archive's `G` also contains its own primary `HEAD` and `index`; linked receivers
use their own `G/worktrees/<registration>/HEAD` and `index`. Do not confuse that
per-worktree Git directory with the common directory. `jj git root` returned
`G` even from receivers; it does not identify their private Git administration.
Use Git's `rev-parse --absolute-git-dir` and `--git-common-dir` to distinguish them.

Observed pointer bytes for sibling roots:

| Entry | Colocated archive | Internal-Git archive |
|---|---|---|
| `W/.jj/repo` | `../../col/.jj/repo` | `../../internal/.jj/repo` |
| `W/.git` | `gitdir: ../col/.git/worktrees/col--colocate` | `gitdir: ../internal/.jj/repo/store/git/worktrees/internal--colocate` |
| admin `commondir` | `../..` | `../..` |
| admin `gitdir` | `../../../../col--colocate/.git` | `../../../../../../../internal--colocate/.git` |

These are regular text files, not directory aliases. The `.jj/repo` path is
relative to `.jj`; gitfile paths resolve relative to the gitfile's containing
directory; admin backlinks resolve relative to the admin directory. Other
versions/layouts may use absolute paths. The observed Git config acquired
`extensions.relativeWorktrees = true`. Do not manufacture pointers by assuming
these exact relative depths or that a registration name equals the JJ name.

## What native creation initializes

Source [`workspace/add.rs`](https://github.com/jj-vcs/jj/blob/803ef00a0a6ee4ddc275d3138d5f9e7326399906/cli/src/commands/workspace/add.rs)
lines 132–176, 193–277 establishes this sequence:

1. Check the workspace name and absent-or-empty destination.
2. If colocation is selected, create Git worktree administration.
3. Initialize the receiver against the existing JJ repository, create local
   working-copy state, and register its name/path in the shared workspace store.
4. Add `.jj/.gitignore` for a colocated receiver; apply requested sparsity.
5. Create a **new** working-copy commit on the requested parent revision(s),
   record the workspace's commit in the repository view and check out its tree.

`-r X` makes `X` a parent of a newly created working-copy commit. It does not
adopt the source's working-copy identity. With no revision, the source uses the
current workspace's parents. Sparse defaults copy the current workspace's
patterns; `full` and `empty` override that. Native source defaults colocation
only when the current workspace is colocated **and** `git.colocate` is true;
explicit flags override this. A JW default-on policy must therefore make its
choice explicit rather than relying on native inheritance from an archive.

Source [`lib/src/workspace.rs`](https://github.com/jj-vcs/jj/blob/803ef00a0a6ee4ddc275d3138d5f9e7326399906/lib/src/workspace.rs)
374–412 writes the relative `.jj/repo` pointer, initializes the local state and
adds the workspace registry entry. Source `local_working_copy.rs` 2676–2724
shows why copying `working_copy/checkout` is wrong: it contains the donor's
workspace name and operation ID. `tree_state` is a separate private format.

Source [`lib/src/git.rs`](https://github.com/jj-vcs/jj/blob/803ef00a0a6ee4ddc275d3138d5f9e7326399906/lib/src/git.rs)
1853–1935 creates a Git worktree without Git checking out payload, initially
with unborn HEAD; JJ's subsequent HEAD export supplies the working-copy parent.
Its populated-directory helper creates temporary administration and repairs
paths. **That helper does not mean `jj workspace add` accepts an already
populated destination**: the CLI rejects it earlier. After-the-fact colocation
is a different operation, to be studied separately against the installed binary.

## Copy boundary: new versus existing receiver

| Material | New receiver | Existing receiver |
|---|---|---|
| Source root `.jj`, including its real repository | Suppress entire subtree | Suppress; never replace destination `.jj` |
| Source root `.git`, directory or gitfile | Suppress | Suppress; preserve target `.git` |
| Destination `.jj/repo`, local `checkout`, registration | Native JJ creation supplies them | Preserve existing identity and repository relationship |
| Destination Git worktree admin and common store | Native creation supplies/links them if selected | Preserve; generic copying must not mutate or delete them |
| Working files and requested ignored content | Materialize under chosen content policy | Replace/update under chosen content policy |
| Donor `tree_state` | Only an explicit, version-sensitive adoption mechanism | Only an explicit adoption mechanism, never wholesale `.jj` replacement |

Exclusion must protect the **destination** as well as omit source entries.
An asymmetric walker that hides source `.jj`/`.git` but deletes target-only
entries can destroy a perfectly valid receiver. A non-colocated archive has no
root `.git`; that absence is not an instruction to delete a colocated receiver's
gitfile. Likewise, a colocated archive's `.git` directory must not overwrite a
receiver's gitfile merely because their entry kinds differ.

For a new workspace, create metadata first and fill around it. Sparse-empty
creation is a demonstrated way to obtain native identity without payload
checkout. For an existing workspace, do not call `workspace add` as a reseed
primitive: it rejects populated destinations. Keep its native metadata and
perform deliberate revision/materialization transitions through the existing
workspace. Copying alone neither changes the workspace's intended revision nor
establishes that JJ's state agrees with the filled bytes.

## Installed-runtime results and the composition boundary

The exact installed JJ and Git 2.55.0 produced these outcomes, with a committed
six-byte `hello` file and explicit `-r @-`:

| Archive primary | Receiver request | Result |
|---|---|---|
| colocated | `--no-colocate` | success; `.jj/repo` pointer, no `.git`, payload present |
| internal bare Git | `--no-colocate` | same |
| colocated | `--colocate` | success; linked Git admin, own `.git`, payload present |
| internal bare Git | `--colocate` | same, common store remains inside archive `.jj` |
| either | `--colocate --sparse-patterns empty` | success; native metadata present, `hello` absent |
| either | add into existing populated receiver with fresh name | rejected: destination not empty |

Both normal colocated receivers had clean Git status and HEAD at the seed
parent. **Both sparse-empty colocated receivers still had `hello` in the Git
index**, and `git status --porcelain` reported ` D hello`. Sparse-empty is a
JJ materialization setting, not proof of a correspondingly empty Git index or
an immediately clean Git working tree. This is a concrete input to the next
composition study: filling and JJ tree-state adoption must converge both JJ's
view and Git's index/HEAD, rather than assuming metadata initialization alone
has finished that job.

No evidence here establishes CoW+colocation composition, initial-root-parent
behavior, merge/conflict behavior, after-the-fact conversion on the installed
binary, or minimum supported Git/JJ versions. Tiny targeted probes of those
conditions are preferable to extrapolating from flags or the source version.

## Current JW integration points

Read-only inspection of `/home/rektide/src/jj-waltz` found:

- `src/workspace.rs:488–549`: native creation with explicit revision;
  sparse-empty when cloning; explicit colocation choice; records created Git
  administration after native success. `670–675` requires an absent path,
  stricter than native JJ's acceptance of an empty directory.
- `src/workspace.rs:616–659` and `src/jj.rs:643–667`: current clone path copies
  tracked files, separately adopts only `tree_state`, validates through JJ,
  then runs `restore`. Watchman is excluded. This is not copying donor identity,
  but its private-state assumptions merit the separate donor-state study.
- `src/lifecycle.rs:125–151`: current policy defaults to CoW/JJ-only and rejects
  explicitly combined CoW and colocation. The desired default-on outcome is a
  policy/composition change, not a missing native metadata primitive in the
  tested ordinary cases.
- `src/ownership.rs:89–118`: distinguishes private Git directory from common
  directory using Git resolution; `276–290` checks admin placement/backlinks.
  Preserve this topology distinction even if the copy engine changes.
- `src/jj.rs:362–389`: probes flag support and Git backend. Necessary checks,
  but not proof that every source/target combination works.
- `tests/colocate.rs:178–220`: assertions cover explicit colocation/config,
  gitfile creation and preserving an existing non-colocated workspace when
  switching with `--colocate`. Read as precedent; this study did not run JW's
  test suite or modify its code.

For a registered workspace as donor, recover `R` through `.jj/repo` rather
than treating the donor as repository owner; recover `G` and private admin
separately. Whether its payload/state is appropriate for another receiver is
the workspace-source study's question. Copying its gitfile would point at the
donor's private administration, not create new target ownership.

## Evidence and handoff

Local fixture: `file:///tmp/opencode/native-metadata-shiny-rocket/` contains
`probe.py`, `output.txt`, `details.txt`, `README.md`, two tiny primary repos and
six receivers. Creation used an allowlisted environment with isolated HOME,
JJ_CONFIG and Git configuration, real `/usr/local/bin/jj`, and no enclosing
live repository. The script and logs preserve the exact commands and outputs;
the directory is ephemeral evidence, not a distributed test dependency.

Native documentation read: `docs/git-compatibility.md:74–129` explains primary
colocation/internal backing and automatic import/export;
`docs/config.md:1791–1803` describes primary init/clone defaults. Workspace-add
default behavior above comes from the command source, not an extrapolation
from that configuration paragraph.

The [independent brief](/design/roadmap/brief0.gpt6a.md) supplies scope and
reading constraints. Follow-on tickets `diffdir-tree-native-colocation`,
`diffdir-tree-native-graph`, and `diffdir-tree-native-workspace-source` receive,
respectively, the sparse-empty Git-index observation, the distinction between
native identity and donor tree state, and the distinction between repository
owner and workspace donor. Same-wave drafts and the withheld architecture
statement were not consulted. Shared indexes remain untouched for integration.
