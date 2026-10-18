---
type: ImplementationBrief
title: Separate filesystem synchronization crate
description: Proposed scope, application contract, copy policies, replacement discipline, concurrency boundary, and verification for the optional sync crate.
resource: /design/sync/brief0.gpt6a.md
status: proposed-for-builder
generated: { by: "model:openai/gpt-6-astra", at: 2026-10-18 }
sources:
  - resource: "opencode://ses_eb39c93e9ffeNju5a7XEsB3wGY"
    title: User requested the separate sync crate and good guidance before dispatching a Sol xhigh builder; parent inspected current APIs and copy primitives
  - resource: /README.md
    title: Comparison direction, opacity, metadata trust, and apply/prune contract
  - resource: /src/model.rs
    title: Public native-name records and committed-prefix comparison summary
  - resource: /src/parallel.rs
    title: Borrowed caller pool and joined inspection epochs
  - resource: /design/implementation/implementation1.sol61x.md
    title: Verified scheduler bounds, divergence seals, and application boundary
  - resource: /design/implementation/refactor0.sol61h.md
    title: Runtime refactor remains unselected; the sync build uses the existing layout
  - resource: "file:///home/rektide/src/jj-waltz/src/cow.rs"
    title: Inspected reflink, mode/mtime, delayed directory permissions, and existing removal procedures
  - resource: "file:///home/rektide/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/reflink-copy-0.1.30/src/lib.rs"
    title: Verified create-new destination semantics, fallback result, and platform support
---

# Builder scope

Add **`sync/`**, package **`diff-dirs-sync`**, as an optional consumer of the
existing library. The core must not depend on it. Use the existing package
layout and APIs; the proposed runtime/multi-crate refactor is not selected.
This is a filesystem synchronization library, with jj-waltz as a prospective
consumer, not a repository-specific operation.

The first useful surface comprises:

- A root-bound applier for `DirDiff` records, enabling existing consumer loops.
- A mirror convenience operation that drives comparison and applies each record
  completely before requesting the next one.
- Whole-root initialization when the source is a real directory and the target
  is absent. Existing non-directory or symlink roots are errors.
- Explicit copy policy and truthful operation/progress reporting.

Avoid a CLI mutation command, Tokio driver, new task framework, automatic
whole-subtree replacement heuristic, or jj-waltz integration in this slice.
The existing read-only diff CLI remains useful for previewing changes.

## Direction and application

`original` is authoritative; `target` is brought into agreement under the diff
contract. Original-only entries are added, target-only entries removed, and
changed leaves replaced. The mirror operation includes removal of target-only
entries; the low-level applier performs only the supplied record's operations.
Never delete by sweeping everything absent from the record: clean entries are
intentionally unmentioned.

Directory/leaf transitions remove the conflicting target entry before installing
its replacement. A same-name transition is one logical operation, not two
independent concurrently scheduled tasks. One-sided subtrees are recursively
copied or removed as explicit whole units; the walker does not enumerate them
for the applier. Recursive operations treat symlinks as leaves and never follow
their targets. Native names and literal link contents survive unchanged.

Records are public data. A public applier must validate relative directories,
single-component basenames, and contradictory category combinations before
acting. Reject absolute paths, parent traversal, and unexpected ancestor links;
allow the intentional cross-category directory/leaf transition. Root binding
does not turn arbitrary caller-constructed records into trusted paths.

Before mutation, reject identical/overlapping source and destination roots,
including a nonexistent destination reached through an existing ancestor.
Canonical path checks are not proof against bind-mounted aliases or hostile
concurrent replacement. Retain the documented stable-tree/alias contract rather
than claiming a capability filesystem sandbox. Generic synchronization adds no
special `.jj`, `.git`, ignore-rule, or repository metadata exclusions.

## Explicit copy policy

Make the caller select one of:

1. **Copy:** ordinary portable copying; no CoW promise.
2. **Reflink required:** failure to obtain a real reflink is an error; never
   silently fall back. This fits jj-waltz's current requirement.
3. **Reflink or copy:** try reflink and allow conventional copy fallback.

Use `reflink-copy` compatible with jj-waltz's 0.1.30 dependency, verifying actual
APIs rather than reconstructing platform ioctls. Its `reflink` requires an absent
destination. `reflink_or_copy` reports reflink versus copied-byte outcome and
does not fall back for NotFound, PermissionDenied, or AlreadyExists. Preserve
those distinctions; avoid an extra source-tree probe or content-reading pass.

The inspected crate supports Linux/Android, Apple platforms, and a Windows
implementation (which its own docs label insufficiently tested). Earlier
research's blanket assertion of no Windows reflink implementation was wrong.
Do not claim platform verification that has not been performed. Platform-specific
symlink creation must preserve link kind where the platform requires it.

## Replacement and metadata

**Recommended replacement policy:** prepare a changed regular file at a private
temporary sibling location, restore required metadata, then rename it over the
target leaf. This avoids writing through destination hardlinks and preserves the
old leaf if preparation fails. Reflink creation needs a nonexistent staging path;
a private temporary directory with an absent child is one available pattern.
Use an established temporary-file library when it avoids inventing name races.
Clean up only artifacts this operation created.

This is not whole-tree transactionality or crash durability. Directory/leaf
transitions and target-only removals remain partial mutations on later failure.
Specify platform replacement limitations honestly. No fsync/rollback journal is
required. Newly added paths can use a simpler owned-creation path if failed
outputs are handled correctly; do not conceal partially written final files as
success or stamp failed copies with source timestamps.

Preserve file permissions and exact available `SystemTime` mtime for successful
copies. Metadata operations must still work for read-only/mode-000 source files
where reading is permitted: do not close the only usable descriptor and then
assume reopening after restrictive permissions will work. Finalize newly created
directory permissions after populating descendants. Define how existing
read-only target directories are handled; do not accidentally chmod source
entries or unrelated hardlinked target files while making removal possible.

The core deliberately ignores mode-only drift, directory mtimes, ownership,
ACLs, and xattrs. Therefore “mirror” means convergence under the stated diff
contract, with mode/mtime preservation on copies, not `rsync -a` equivalence.
Do not add an unconditional metadata-repair sweep or content hashing. A second
completed diff must be clean after successful sync on compatible filesystems.
If a destination cannot represent a source timestamp exactly, document or
report that limitation rather than promising exact convergence universally.

## Concurrency scope to settle before dispatch

Use the existing caller-owned Rayon pool for optional parallel comparison.
The diff's epoch has joined before application starts. Every application batch
must join before `next()` acknowledges the record; no apply operation may race
with descendant inspection or mutate a cached sibling through an alias.

The proportionate first parallel application scope is **independent named
operations within a record**, sharing that same pool and an explicit bounded
apply budget. Complete each same-name removal/replacement sequence together.
Do not create a pool per operation or queue an unbounded detached task tree.
Whole-subtree operations may initially execute serially inside their task;
that limitation must be explicit and measured, not described as fully parallel
recursive synchronization. A single newly initialized root will not gain
whole-tree copy parallelism from this limited policy.

Sequential application with parallel comparison is an even smaller first slice.
The user's scope choice should settle this before the builder implements it.
Neither choice requires changing the diff scheduler or introducing async I/O.

## Errors and reports

Return operation and path context with the original error cause. Stop further
admission on failure; join already-started apply operations before returning.
Parallel siblings already applied remain applied, so error reporting must not
claim only an exact serial prefix changed. Choose deterministic reported error
order where feasible, without suppressing meaningful secondary cleanup errors.

Keep **observed differences** separate from **completed filesystem operations**.
Comparison counts are not copied-file counts inside opaque subtrees. Report
reflinks versus copies, relevant deletions, and partial progress with explicit
count units. A read-only no-op sync must perform zero writes. Retrying after a
partial failure must converge; it need not leave the exact original delta.

## Verification and measurement

Meaningful owned-fixture tests should cover:

- Clean no-op; changed/added/removed entries; absent-target initialization;
  all directory/file/symlink transitions; clean second comparison.
- Native non-UTF-8 names, literal/dangling/directory links; external symlink
  targets untouched; destination hardlink aliases unchanged on replacement.
- Permissions and high-resolution/pre-epoch timestamps where supported;
  read-only leaves and directories; no mode-only repair promise.
- Invalid record paths/combinations and overlapping roots rejected before
  mutation; unsupported specials rejected when encountered.
- Strict reflink failure without fallback; real reflink success on the available
  btrfs fixture and independent post-copy mutation; explicit fallback outcome.
- Preparation failure preserving an existing leaf; no leaked owned temporary
  artifacts; honest partial apply plus retry; pool/budget/join behavior if built.

Do not mutate `/home/rektide/a/a/opencode` or
`/home/rektide/src/opencode-export-qa`. Those remain read-only comparison inputs.
Benchmark writes on owned disposable btrfs fixtures, with setup separate from
timing: initial copy, clean repeat, sparse delta, and selected subtree removal
or replacement. Compare no-op cost with the existing walk and report actual
copy/reflink work; no end-to-end jj-waltz speedup claim.

Run appropriate workspace/default/optional-feature tests, clippy, formatting,
doctests, and actual Rust 1.88 verification. Confirm that embedding `diff-dirs`
alone still pulls no sync/reflink/temporary-file dependencies. Commit coherent
stages and preserve earlier implementation/performance reports.
