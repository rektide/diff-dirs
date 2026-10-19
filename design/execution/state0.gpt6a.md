---
type: StateReport
title: "Context payloads, donor state and lifetimes"
description: Exact payload and ownership of the visitor contexts, directory plans, traversal/application state and separately described JW donor/JJ state.
resource: /design/execution/state0.gpt6a.md
status: source-reviewed
generated: { by: "model:openai/gpt-6-astra#xhigh", at: 2026-10-19 }
updated: { by: "model:openai/gpt-6.1-sol#xhigh", at: 2026-10-19 }
sources:
  - resource: "opencode://ses_ea9c65f5fffeyf5EYvFs581in4"
    title: Implemented and verified per-side early facts, opt-in Linux RawDir plus std Dir, unknown completion, root reuse and recursive synchronization after the parent approved the backend boundary
  - resource: "opencode://ses_eb262a81effea1HgF0pPS3f1sy"
    title: Traced context fields, lifetimes and donor identities, then clarified enumeration-provided facts versus additional metadata I/O after the human's dirent question
  - resource: /design/performance/inspection0.sol61x.md#e2
    title: Earlier verification of cached d_type and the metadata fallback in DirEntry::file_type
  - resource: "file:///home/rektide/.rustup/toolchains/nightly-2026-10-14-x86_64-unknown-linux-gnu/share/doc/rust/html/src/std/sys/fs/unix.rs.html#1053"
    title: Rechecked configured std file_type cached-type dispatch, metadata fallback, and ino accessor
  - resource: /src/visitor.rs
    revision: 20c11498
    title: EarlyFacts, FileKind, FactOrigin and immutable per-side Name payloads alongside the existing late facts and plans
  - resource: /src/listing.rs
    title: Single-pass default/native enumeration, cached hints, shared parent lifetime and deferred metadata
  - resource: "file:///home/rektide/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rustix-1.1.5/src/fs/raw_dir.rs"
    title: Verified RawDir continuation and cached name, type and inode accessors
  - resource: "file:///home/rektide/.rustup/toolchains/nightly-2026-10-14-x86_64-unknown-linux-gnu/share/doc/rust/html/src/std/sys/fs/unix/dir.rs.html#157"
    title: Verified std Dir no-follow basename-relative statx/fstatat implementation
  - resource: "file:///home/rektide/.rustup/toolchains/nightly-2026-10-14-x86_64-unknown-linux-gnu/share/doc/rust/html/src/std/fs.rs.html#2844"
    title: Verified public Windows enumeration-metadata no-extra-syscall contract
  - resource: /tests/early_facts.rs
    title: Actual listing hints, absent versus present-unknown, independent side kinds, inode values and structural-root provenance
  - resource: /sync/tests/early_facts.rs
    title: Tiny project-filesystem required-reflink tests of direct, recursive, staged, removal, preview, audit and initialization paths
  - resource: /src/model.rs
    title: Comparison counts, coverage and planned/retained accounting
  - resource: /src/pair.rs
    title: Root facts, child lowering and release of clean batch metadata
  - resource: /src/walk.rs
    title: Active frames and yielded-plan lifetime
  - resource: /src/frontier.rs
    title: Private speculative state and committed prefix
  - resource: /sync/src/lib.rs
    title: Bound applier fields, report aggregation and API boundaries
  - resource: /sync/src/apply.rs
    title: Batch Context, held descriptors, staging, writable leases and grouped seed state
  - resource: /sync/src/model.rs
    title: Progress, Policy, Report and Failure payloads
  - resource: "file:///home/rektide/src/jj-waltz/src/reseed/engine.rs"
    revision: 24b58a67d0047e27c4a8479686e00d0ebffb8515
    title: Actual per-target Plan and donor/revision/configuration ownership
  - resource: "file:///home/rektide/src/jj-waltz/src/cow/transfer.rs"
    title: Compiled Auxiliary state and early/late use of its frozen boundaries
  - resource: "file:///home/rektide/src/jj-waltz/src/cow/payload.rs"
    title: FrozenRules, IgnoreRules and persistent native oracle fields
  - resource: "file:///home/rektide/src/jj-waltz/design/sync-scope/jj-state-review0.gpt6a.md"
    title: Reviewed JJ identity/cache distinctions and move/adoption conditions
  - resource: "file:///home/rektide/src/jj-waltz/tests/reseed.rs"
    title: Existing-workspace donor-state convergence test, read without execution
  - resource: "file:///home/rektide/src/jj-waltz/design/sync-scope/visitor0.sol61xh.md"
    title: Test provenance, runtime version and unimplemented production adoption boundary
---

# Context payloads, donor state and lifetimes

This is the state companion to [the execution walkthrough](flow0.gpt6a.md).
“Context payload” means the data supplied to a visitor or held by the execution
engine. **It is not the contents of the files being transferred.**

The visitor baseline was diffdirs `0ce979c26a323ac2172215c4b32a09b3e9a86b5d`;
early-facts implementation is `20c11498`, with bounded verification in `09ed238a`.
JW source remains `24b58a67d0047e27c4a8479686e00d0ebffb8515`. This describes
implemented library behavior, not an accepted new JW copying default.

## 1. Library: context payloads

### 1.1 Library: `Names` is the early context

| Field / slot data | Meaning | Visitor mutation |
| --- | --- | --- |
| `dir: PathBuf` | Logical containing directory relative to bound roots | None |
| `root: bool` | Singleton structural-root context, rather than a child listing | None |
| `entries: Vec<Name>` | Sorted union of immediate names | Existing slots only; no injection/reordering |
| `Name.name` | Native basename | None |
| `Name.relative` | Logical final root-relative name | None |
| `Name.presence` | `SourceOnly`, `Both`, `TargetOnly` from listings | None |
| `Name.source()`, `target()` | `Option<&EarlyFacts>` per side: kind hint, optional inode, origin | None |
| `Name.admission` | Initially `Observe`; can become `PreserveTree` | Yes |

No full `Metadata`, contents, JJ identity or mount table is exposed here. Private
paired listing handles are held beside the context, not exposed as visitor
authority. Preservation drops them without additional child observation.

The early callback receives `&mut Names`, but public methods permit editing
admission, not inventing a new source absence. A named refusal carries the actual
slot coordinate plus the caller's cause. Reordered opaque slots are rejected.

#### Enumeration can already provide facts

**Implemented:** the early/late boundary is before **additional observation I/O**,
not names-only. Linux directory records commonly include an inode number and a `d_type`
classification (directory, regular file, symlink, or another kind). They do not
provide ordinary file size, mtime or full permissions. Some filesystems return
`DT_UNKNOWN`; other platforms' enumeration interfaces can provide different or
richer facts.

`Name::source()` / `target()` distinguish absent (`None`) from present with
unknown kind (`Some` with `FileKind::Unknown`). `EarlyFacts` exposes immutable
`kind()`, optional `inode()` and `origin()`; kind includes regular, directory,
symlink, FIFO, socket, block/character device, Other and Unknown.

| Backend / context | Early facts | Later observation |
| --- | --- | --- |
| Default std Unix listing | Cached inode; kind Unknown | One admitted `DirEntry::metadata()` |
| Linux `native-dirent` | Raw cached d_type and inode; DT_UNKNOWN stays Unknown | One admitted `Dir::symlink_metadata(basename)` |
| Windows std listing | Kind derived from enumeration-cached metadata; no inode promised | Reuses that cached full metadata |
| Other std listing | Kind Unknown; inode unavailable | Normal admitted std metadata |
| Structural root singleton | Kind and Unix inode derived from mandatory root metadata; `RootMetadata` origin | Reuses those same full facts |

Linux native is explicitly opt-in and requires nightly for std `dirfd`;
`cargo test -p diff-dirs --features native-dirent` enables it. The dependency is
optional, Linux-specific rustix 1.1.5, already used by sync. Default remains
std-only and stable-compatible. No JW Cargo or production code was changed.

`RawDir` owns parsing and enumeration; no hand-written getdents parser, duplicate
listing or std-private-layout access exists. One shared `Dir` per listed side
supplies relative no-follow late metadata; no per-entry open+fstat is introduced.
Unknown hints do not invoke std's hidden `file_type()` metadata fallback. Full
facts are fetched once only for admitted entries when not already cached. The
Windows cache branch is source/API-grounded, not exercised platform evidence.

#### Verification of the early seam

Backend-unit instrumentation covers Known/Unknown early preservation (zero
metadata calls), name-based `.jj` preservation even when both kinds are Unknown,
and an admitted Unknown symlink completed exactly once without following it.
Root-binding tests finalize already-owned metadata after moving the tiny owned
root paths, proving no root restat solely for early/late payloads. Actual Linux
tests check cached kinds, independent side types and enumeration inode values.

A two-entry owned Btrfs fixture additionally passed a narrow syscall check on
configured Rust `1.101.0-nightly (822182db4 2026-10-13)`. Between markers outside
callbacks, preserving the actual cached directory and symlink produced exactly:

```text
openat(AT_FDCWD, ".../original", O_RDONLY|O_CLOEXEC|O_DIRECTORY) = 3
getdents64(3, ... /* 4 entries including . and .. */, 8192) = 96
getdents64(3, ... /* 0 entries */, 8192) = 0
close(3) = 0
```

No child stat, readlink, open or descent occurred. Reproduction: build the core
unit-test executable with `cargo test -p diff-dirs --lib --features native-dirent
--no-run --message-format=json`, then run its reported executable through
`strace -f -e trace=openat,newfstatat,statx,readlink,getdents64,close,write` with
`--exact visitor::listing_tests::real_cached_type_filter_performs_only_the_containing_directory_listing
--nocapture`. The ignored scratch trace is `.test-agent/native-dirent/native.trace`;
the normalized excerpt above is the maintained evidence. This is not a timing
campaign or proof that every filesystem supplies a recognized d_type.

Affected core/sync nextest passed **128 all-feature** and **101 default** tests,
none skipped, with `DIFF_DIRS_SYNC_REQUIRE_REFLINK=1`. Core/sync doctests passed
16 all-feature and 15 default examples. Workspace/all-target/all-feature clippy
and default core/sync clippy passed with warnings denied; formatting and Rustdoc
generation also passed. Installed stable Rust 1.99 checked default core/sync;
the default core dependency tree remains just `diff-dirs`. New fixtures stay well
below 256 entries/1 MiB; no live workspace, import, benchmark or JW operation ran.

### 1.2 Library: `Operations` is the late context

`Operations` owns logical source/target root paths, the containing relative `dir`,
a root/singleton bit, a `Purpose`, and the admitted `Vec<Operation>`.
Physical temporary paths are not substituted for these logical coordinates.

Each `Operation` contains:

| Payload | What it says |
| --- | --- |
| `name`, `relative` | Immutable slot identity |
| `source: Option<Facts>` | Actual source-side facts, or genuine absence |
| `target: Option<Facts>` | Actual target-side facts, or genuine absence |
| `different: Option<bool>` | Supported baseline comparison, or unknown for unsupported kinds |
| `baseline: Option<Action>` | Normal mirror proposal before late steering |
| `action: Option<Action>` | Current proposed work, editable through methods |

`Facts` contains owned no-follow `std::fs::Metadata` and optional literal
`PathBuf` link text. The callback borrows those values. Metadata can expose size,
mtime, type and platform fields such as device/mode; it does not carry file bytes
or a content hash. Link text is not the referent's contents and is not followed.

The `Option`s have different meanings:

- Missing `source` or `target` means that side was absent, **not excluded**.
- Early-excluded entries are absent from the late slot vector altogether.
- `comparison() == None` means an observed unsupported kind has no normal
  comparison. It is not equality.
- For such a slot, baseline/action initially are `None`. Late can explicitly
  select `PreserveTree`; otherwise finalization refuses it.
- Every successfully finalized entry has `Some(action)`.

The action vocabulary separates local container work from descent:

```text
KeepLeaf / InstallLeaf / RemoveLeaf
Directory { container: Keep | Create | Remove, descend: bool }
ReplaceWithLeaf
ReplaceWithDirectory { descend: bool }
PreserveTree
```

Presence and type constrain legal combinations. Keeping an existing container
while visiting children is different from preserving its subtree. A missing
container cannot be “kept” while permitting child creation. Type replacements
are coupled operations. The current API does not force-install an equal leaf.

### 1.3 Library: `DirectoryPlan` is a frozen immediate batch

After late and validation, the plan owns:

- the immutable `Operations` and their metadata/link facts;
- a baseline `DirDiff` for admitted supported entries;
- excluded count, first excluded example and immediate preserved names;
- scoped/steered standing, planned-work count and retained-action count.

The immediate preserved-name vector supports safe public-record validation. It
is not an inventory of the excluded interiors. Public plan accessors do not
offer mutable actions after finalization.

`DirDiff` contains names/classifications only:

```text
dir
files.changed / files.added / files.removed
dirs.added / dirs.removed
```

Matched-directory descent comes from finalized actions, not a `dirs.changed`
field. Aggregate changed directories live in `Summary.changed_dirs`.
Late-retained differences remain in the baseline diff. Late-preserved specials
have no normal file delta; retained coverage keeps the result from claiming
full-root cleanliness.

### 1.4 Library: what is mutable, and when?

| Stage | Mutable data | Immutable data |
| --- | --- | --- |
| Early callback | Admission of existing union slots | Names, presence, per-side early facts, containing coordinate |
| Observation | Inspector constructs facts | Early suppression has already stopped those names |
| Late callback | Current action on existing observed slots | Facts, names, presence, baseline comparison/proposal |
| Finalized plan | No public mutation | Whole immediate plan |
| Application | Filesystem state and progress counters | Plan's input observations under caller's stability contract |

The visitor is shared as `&self`; each invocation receives its own mutable batch.
This permits parallel inspection without a shared mutable policy scheduler.
Callbacks must use frozen caller facts and supplied observations. Speculation
can repeat/discard invocations, so a callback is not an ordered progress event.

`Visitor::ENABLED = false` promises both callbacks are unconditional no-ops.
It permits skipping callback dispatch and root callback-only construction, **not**
all normal directory observation/plan construction. `Check::ENABLED` similarly
controls compatibility checking. The seed installer's construction-site guard
avoids guard-only work; it does not suppress metadata needed to copy the file.

## 2. Library: execution-state lifetimes and actual file data

### 2.1 Library: state ownership ledger

| Owner / lifetime | Retained state | Released or transferred when |
| --- | --- | --- |
| Bound `Applier`, caller-chosen lifetime | Canonical source/target paths, copy policy, visitor and compatibility adapter values, root-replacement permission | Binding is dropped |
| Root binding / paired `Roots` | Actual root metadata for structural validation and root callback reuse | Walk ends |
| Immediate inspector | Raw entries, union slots, private std/native shared-parent handles | Excluded handles drop early; admitted handles drop after facts are collected, before late/publication |
| Immediate `DirectoryPlan` | Metadata/link facts, final actions and baseline diff | Consumed/dropped after application; clean plans are reduced to children/counters and released in the worker |
| Active walk frames | Pending paired child names, relative directory and summary propagation flags | Subtree finishes or traversal stops |
| Parallel frontier | Bounded queued/running/ready directory nodes; ready divergent plans until commitment | Ordered commit/discard; all started jobs join before return |
| Apply `Context` | Root references, visitor/adapter references, copy policy, parallel/stopped state and atomic planned/retained/excluded counters | Immediate apply/root operation ends |
| Recursive application | Current child batches, staging arenas and writable leases | Joined child operation completes/fails |
| Final `Report` / `Failure` | Counts, coverage, paths/causes and partial progress | Caller decides retention |

There is no persistent source filesystem catalog or full payload operation map.
This is **not constant-memory or depth-only**: wide listings, active child names,
divergent lookahead plans and summaries consume memory. J/K are directory-count
bounds, not byte bounds. Named-seed installation additionally holds the explicitly
supplied path list; that list comes from the caller, not filesystem discovery.

### 2.2 Library: the donor is a live directory, not a snapshot object

The library stores a source path and observations made while visiting it. It has
no donor revision, repository lock, receipt, generation identifier or persistent
content digest. Root binding does not freeze bytes. A public plan is not a durable
capability to apply after writers resume.

The caller supplies stable source/target roots, path topology and the permitted
mutation discipline. In a normal run the loop mutates classified entries only
after their batch was observed, while pending paired paths remain stable. Staged
replacement avoids writing through an existing hard-linked target leaf, but the
library is not a protection system against arbitrary concurrent replacement.

Repeated target runs inspect source metadata again. Current multi-target use does
not share one global source scan across all destinations; that remains a separate
future direction.

### 2.3 Library: where bytes and descriptors live

File bytes stay in the source/target files and storage layer until an operation
needs to copy. Visitors and `DirectoryPlan` carry no file-content buffers.

For a regular installation, `PreparedLeaf` retains useful source/output `File`
handles through clone/copy, metadata finalization and installation. On Linux,
required reflinking is descriptor-to-descriptor `FICLONE`; ordinary copying uses
`io::copy`. Source extents can be shared, while the output inode is private.
Permissions and exact mtime are installed before the staged name is renamed.

Each relevant parent batch owns a lazy `Staging` arena. Its reservation set avoids
collisions with intended sibling names. Writable leases save and restore original
permissions when actual mutation required relaxing them. Successful clean or
retained-only work creates neither an arena nor permission changes.

Cleanup of failed private preparations is library-owned lifecycle work; caller
preservation decisions do not strand implementation temporaries. Completed-work
counters exclude failed private installations. None of this constitutes a whole-
tree transaction or crash-durable journal.

### 2.4 Library: reports describe different state

- `Summary`: committed paired-comparison coverage and its observed differences,
  exclusions, planned/retained counters, prunes and completion/failure state.
- `Report.observed`: optional pre-application comparison summary; absent for
  operations without that paired walk, including joint preview and root init.
- `Report.planned`: planned namespace slots encountered through the operation,
  including recursive application; not necessarily the same count as Summary's
  paired-directory prefix.
- `Report.completed`: actual copied/reflinked bytes/leaves, links/directories,
  removals and permission operations.
- `Report.policy`: satisfaction plus encountered retention/exclusion counts.
  These are not unique-path catalogs or proof of full-root equality.
- `Failure`: cause/path, completed and planned progress, policy state, optional
  comparison prefix, plus sibling/cleanup/restoration issues.

`is_complete()`/`is_clean()` are conservative about scoped coverage. A visitor
that can steer defaults to scoped standing even if no excluded name occurred.
Physical speculative work is not retroactively counted as committed coverage.

## 3. Example consumer: the state it supplies

The [generic example](flow0.gpt6a.md#2-example-consumer-a-small-generic-mirror)
stores a zero-sized `KeepLocal` visitor and chooses `ReflinkRequired`.

- Before metadata, its decision uses the supplied native basename.
- After metadata, its decision uses source/target regular-file type and mtime.
- It neither owns a donor cache nor creates an entry index.
- It does not mutate observations or copy file bytes through the context.
- Its local `.local` data remains in the target; its metadata is not a visitor
  payload and is never opened by the transfer merely to establish preservation.

This is a consumer-specific policy example, not a dependency on JJ or an implied
JW default. Another compiled visitor can choose different legal operations over
the same contexts without changing the filesystem kernel.

## 4. JW specifically: command state and donor state

### 4.1 JW specifically: coordinates are not interchangeable

| JW coordinate | Meaning |
| --- | --- |
| `source_operation` | Pinned repository operation used for native source queries |
| `source_commit` | The asserted donor tracked-tree coordinate (`--source-at` or the recorded source selection) |
| `source_wc_commit` | The donor workspace's actual captured working-copy commit; it may have a different ID but equal tracked tree |
| `base_commit` / D | Desired target base selected by `--at`, otherwise the donor's captured current commit |
| `old_commit` / O | Target working-copy commit captured after the initial snapshot, before refill |
| `desired_commit` | Initially the old target commit in the plan, then the new empty-change child of D created during execution |
| `target_operation` | Target query coordinate, updated with that execution transition |

The source/desired tracked lists and equality proofs determine which source leaves
can prefill the target. The target's old list determines which recorded leaves
may need clearing. **An empty change at D still contains D's complete file tree.**

### 4.2 JW specifically: the per-target `Plan`

The actual engine `Plan` stores:

- target name/root, target root and `.jj` device/inode identities;
- source root, source root and `.jj` identities, shared repository location;
- the native operation/commit coordinates above;
- `clones`: known equal tracked leaves to install;
- `entries`: desired tracked entries; `old_paths`: old target tracked leaves;
- `ignored`: selected auxiliary-root boundaries, not all descendant entries;
- shared `Rc` source entries and source/target `IgnoreRules` contexts;
- replace-content mode, copied link-config bytes, config root, bookmark guard;
- selection/warning and execution diagnostics.

This is an in-memory **command/target plan**, not the library's `DirectoryPlan`.
It is not serialized as a recovery journal and does not contain a complete map
of payload files or copies of their contents.

JW's advisory `operation.lock` serializes cooperating JW prepare/reseed commands.
It is not JJ's native mutation lock or a lock on arbitrary editors, builds, or
filesystem writers. Root device/inode identities catch root substitution, not
every mutation beneath the root.

### 4.3 JW specifically: what the visitor captures

`cow::transfer::Auxiliary` contains:

| Field | How it is used |
| --- | --- |
| `roots: BTreeSet<PathBuf>` | The compatibility-selected auxiliary regions |
| `routes: BTreeSet<PathBuf>` | Their ancestor directories, including the root routing context |
| `devices: (u64, u64)` | Source/target device facts; late checks supplied metadata, without a fresh mount query |
| `new_targets_only: bool` | Creation-mode refusal of an occupied selected root |

Construction checks selected roots against the source/desired tracked inventory
in both prefix directions. That inventory is not retained inside the visitor.
Within a selected region, default mirror actions remain; routing containers are
kept/created and descended; outside regions are early-preserved.

The separate `payload::Guard` compatibility check owns a mutex-protected set of
absolute-source-link warning strings. Ordinary entries do not format paths or
lock that set. Diagnostics are encountered observations, not ordered commit
notifications. The general visitor receives no `JjClient`, Git child process,
repository object database or file-state protobuf.

### 4.4 JW specifically: frozen ignore context

`FrozenRules` is command-owned:

- `files`: extracted tracked `.gitignore` bytes keyed by proven tree coordinate;
- `equivalent`: aliases backed by native whole-tracked-tree equality;
- `rules`: `Rc<IgnoreRules>` keyed by tree coordinate **and effective exclude bytes**.

Each `IgnoreRules` holds a temporary isolated Git rule directory, memoized answers
keyed by `(relative path, directory/file query kind)`, and an optional native oracle.
That directory contains rule inputs, not a copied dependency tree. The lazy oracle
owns stdin/stdout handles and a bounded-retention stderr-draining thread; queries
use framed NUL-separated `check-ignore --verbose --non-matching --no-index` output.
The child and reader are stopped/joined when discarded.

These structures are intentionally on the caller thread (`Rc`/`RefCell`), outside
parallel visitor execution. They freeze rule inputs and answers, **not ignored
file bytes**. Git ignoredness also does not establish JJ tracked membership; the
native tracked lists are a separate proof. This remains compatibility machinery
in the current production path, not a generic diffdirs requirement.

### 4.5 JW specifically: what is and is not frozen about the donor

**Captured:** native coordinates, tracked membership/tree checks, rule inputs,
root identities, relevant configuration, selected roots and upfront topology.

**Still live:** ordinary donor files, ignored payload, and path namespace. There
is no immutable donor content snapshot, read-only subvolume, global content hash,
or source-to-many metadata cache created by this command. Native tracked-tree
verification does not verify all auxiliary bytes.

Production rechecks the donor at multiple execution barriers. Those checks and
the paused-writer/topology assumptions define the confidence of this operation;
holding an `Rc<IgnoreRules>` or a commit ID is not a substitute for them.

## 5. JW specifically: native JJ state and the adoption test

This is a different state layer from library callback contexts and payload files.

| Native state | Ownership / meaning |
| --- | --- |
| Workspace `.jj/repo` pointer | Links that workspace to its shared repository; relative placement can matter |
| Workspace `checkout` | Its own workspace name and operation binding; not donor identity to copy wholesale |
| Workspace `tree_state` | Tracked-tree binding, file-state cache, sparse patterns and monitor-related state |
| Shared repository views/store | Native commits/trees/objects and workspace-to-commit relationships |
| Shared workspace registration | Native workspace naming/path relationship |
| Colocated Git linkage, where applicable | Additional native Git path/registration state |

The [reviewed JJ research](file:///home/rektide/src/jj-waltz/design/sync-scope/jj-state-review0.gpt6a.md)
documents exact versioned qualifications. Advancing the shared operation head
alone does not necessarily make a working copy stale; tree identity and operation
ancestry participate. A stat cache can avoid reading unchanged content only when
its recorded facts actually match disk under JJ's race rules.

### 5.1 JW specifically: production reseed today

The target keeps its own `.jj`, root and native identity. Sparse clearing removes
tracked materialization/file-state entries; `jj new D` selects a new empty change
with D's tree. Known prefills are installed; sparse expansion and verification
reconcile the file states. The engine does not transplant a donor cache in this
production path and does not move `.jj` aside.

### 5.2 JW specifically: upstream creation and the bounded experiment

Upstream creation has a different, private-layout optimization: copy donor-tracked
files preserving their stats, copy the donor's `tree_state`, validate through JJ,
then restore the requested base. The target retains its own `checkout` binding.
The donor's copied tree S can differ from final desired tree D.

The current native test
`existing_workspace_can_validate_donor_tree_state_then_restore_a_different_desired_tree`
demonstrates that sequence on a tiny **existing** workspace:

1. Save target work, disable new-file auto-tracking, create disjoint local names.
2. Sparse-empty, create the target's new change at D, and seed all S-tracked leaves.
3. Copy only S's `tree_state`; assert target checkout bytes stayed unchanged.
4. Native `status` validates/snapshots the copied S; native `restore --from D`
   converges to D.
5. Assert desired tree/parent, target metadata/root identity, saved work, local
   data and the unchanged cloned file inode.

This is reported passing runtime evidence for installed JJ `0.45.1-33d3a595`, not
production dispatch, a hash-free timing result, or a complete failure/concurrency
protocol. It does not prove arbitrary nonignored locals survive normal initial
snapshots—the test disables new-file tracking before creating those locals.
Future production adoption still needs explicit overlap, locking and failure
handling. Merely restoring old `.jj` metadata over changed contents establishes
neither the desired revision nor truthful file-state caches.

## 6. Reading the state boundaries correctly

The library owns observations and short-lived proposed work; it does not know
repository identity. A generic consumer supplies steering, not donor bytes hidden
inside its callback. JW supplies native command state and a compatibility visitor;
its donor payload remains live filesystem data. JJ's persistent state is a third
layer with its own ownership and validation rules.

These distinctions explain why a visitor can reduce inspection/copying work
without itself solving revision adoption, and why a valid commit coordinate or
cached rule context cannot be treated as a frozen copy of all donor contents.

## 7. Source navigation

### Library definitions

- [Visitor context and plan types](../../src/visitor.rs): `Names`, `Name`, `Facts`,
  `Operation`, `Operations`, `DirectoryPlan`, `Visitor`.
- [Bound consumer](../../sync/src/lib.rs): `Applier`, root dispatch and aggregation.
- [Private application state](../../sync/src/apply.rs): `Context`, `PreparedLeaf`,
  `Staging`, `Writable`, grouped known-leaf installation.
- [Traversal state](../../src/walk.rs) and [frontier state](../../src/frontier.rs):
  active paired frames, bounded private results and commitment.
- [Comparison summary](../../src/model.rs) and [sync reports](../../sync/src/model.rs).

### JW definitions and evidence

- [Per-target Plan and live checks](file:///home/rektide/src/jj-waltz/src/reseed/engine.rs).
- [Auxiliary visitor and shared pool](file:///home/rektide/src/jj-waltz/src/cow/transfer.rs).
- [Frozen rule/cache/process payload](file:///home/rektide/src/jj-waltz/src/cow/payload.rs).
- [Native adoption test](file:///home/rektide/src/jj-waltz/tests/reseed.rs) and
  [its scoped verification account](file:///home/rektide/src/jj-waltz/design/sync-scope/visitor0.sol61xh.md).
