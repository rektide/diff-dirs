---
type: OpportunityInventory
title: Filesystem comparison and synchronization — performance opportunities and contract forks
description: Actionable, evidence-separated inventory of path, metadata, descriptor, scheduler, copy, staging, multi-target, and protocol alternatives, with candidate IDs, rejection tests, dependencies, qualitative complexity, and independently commissionable Sol high experiment briefs.
resource: /design/performance/opportunities0.sol61x.md
status: research-for-assessment-not-selected-for-implementation
generated: { by: "model:openai/gpt-6.1-sol#xhigh", at: 2026-10-18 }
sources:
  - resource: "opencode://ses_eb3033c79ffeRIkn14uSNhs7O6"
    title: Built sync, then followed the user's research-only commission to inventory broad optimization and contract alternatives; verified std source/syscalls, preserved original measurements, and adopted the unpinned configured-nightly direction without implementing optimizations
  - resource: /design/performance/inspection0.sol61x.md
    title: Newly verified historical/current std behavior, actual owned syscall probes, installed nightly directory API, and raw-data recomputation
  - resource: /README.md
    title: Existing comparison, ordering, opacity, mutation acknowledgment, coverage, and trust contracts considered as revisable design choices rather than permanent prohibitions
  - resource: /src/pair.rs
    title: Raw entry/name ownership, combined sort, eager diagnostic paths, all-entry metadata, and conversion/merge
  - resource: /src/walk.rs
    title: Serial frame paths, descent, yield/prune, and prefix summary
  - resource: /src/frontier.rs
    title: Job/node path copies, candidate scans, arena/mutex, J/K admission, clean/differing results, and epoch drains
  - resource: /src/parallel.rs
    title: Borrowed pool, default J/P and K/4P policy, and per-next installation
  - resource: /src/model.rs
    title: Public constructible names-only records, immediate counts, and committed-prefix summaries
  - resource: /src/error.rs
    title: Owned diagnostic paths, original causes, and terminal errors
  - resource: /sync/src/apply.rs
    title: Validated named operations, duplicate metadata, staged replacement, recursive copying/removal, permission finalization, result collection, and cleanup
  - resource: /sync/src/paths.rs
    title: Root/prefix checks and cloned-name BTreeMap validation
  - resource: /sync/src/lib.rs
    title: Applier/root scope and serial/borrowed-pool mirror loops
  - resource: /sync/src/model.rs
    title: Actual installed-operation progress, partial failure, and observed-versus-completed distinction
  - resource: /cli/src/render.rs
    title: JSON Value-tree/native-name allocation and text rendering
  - resource: /cli/src/main.rs
    title: Existing buffered output and terminal failed-summary handling
  - resource: /design/implementation/implementation0.sol61x.md
    title: Historical core design reasons and initial verification boundary
  - resource: /design/implementation/implementation1.sol61x.md
    title: Frontier invariants, seals, lifecycle, bounds, and verification experiments
  - resource: /design/concurrency/design0.gpt6am.md
    title: Original ordered-frontier choices, reserved-demand rationale, and explicitly deferred alternatives
  - resource: /design/fit/io0.glm53m.md
    title: Earlier IO/backend assessment, revisited by the verified fd-relative std baseline and actual sync implementation
  - resource: /design/fit/rayon0.glm53m.md
    title: Earlier pool/heads reasoning, distinguished from the measured implemented frontier
  - resource: /design/implementation/refactor0.sol61h.md
    title: Paused runtime/package separation assessment; distinguishes dependency topology from filesystem throughput
  - resource: /design/sync/implementation0.sol61x.md
    title: Sync architecture, metadata/alias safety, partial progress, and verification
  - resource: /design/performance/performance0.sol61x.md
    title: Historical syscall/metadata baseline and read-only live-root method
  - resource: /design/performance/parallel0.sol61x.md
    title: Historical per-frame scheduling control and its topology limitations
  - resource: /design/performance/parallel1.sol61x.md
    title: Frontier scaling, K/J saturation, wide/dense/skewed limitations, resource costs, and raw observations
  - resource: /design/performance/sync0.sol61x.md
    title: Existing owned nonempty-file copy/reflink/no-op/delta timings and actual work units
  - resource: /perf/results/frontier/confirmation.csv
    title: Consulted/recomputed historical live timing controls
  - resource: /perf/results/frontier/guided.csv
    title: Consulted/recomputed admission sweeps and independent repeat
  - resource: /perf/results/frontier/shapes.csv
    title: Consulted/recomputed small/dense/wide topology controls
  - resource: /perf/results/sync/observations.csv
    title: Consulted 60 original sync observations, retaining their actual Rust 1.88 compiler provenance
  - resource: /examples/perf.rs
    title: Existing read-only repeat/compare controls, gathering outside timing, and topology fixtures
  - resource: /sync/examples/sync_perf.rs
    title: Owned mutation scope, absent private targets, timing boundaries, and outside-timing verification
---

# Opportunity inventory, not a new implementation plan

The completed sync crate is the starting point, not the end of the design
space. The user explicitly invites **base-contract reconsideration**, richer
records, different read-only/report protocols, native backends, and tuned
primary-consumer variants. Existing README choices explain the baseline; they
do **not** automatically disqualify alternatives in this document.

No candidate below is authorized for implementation by its presence here.
This session changed no production code or defaults and launched no agents.
The purpose is to equip assessment and subsequent **Sol high** experiment
commissions, not silently undertake a rewrite.

**Toolchain override:** use ordinary configured `cargo`/`rustc`, currently
1.101.0-nightly (`822182db4`, LLVM 23.1.3); the project is unpinned. Do not impose
a Rust 1.88 API/dependency floor on new work. Historical measurements below
retain their actual 1.88 provenance. Any new control/candidate timing must use
the **same current compiler**, flags, dependency resolution, and fixture state,
recording `rustc -Vv`. The parent owns active manifest/root/perf instruction
updates; those files are not changed by this research.

## Reading map and evidence labels

- [Ground truth](inspection0.sol61x.md): source/syscall evidence and its limits.
- [Ranked starting points](#ranking): likely work removal, payoff hypotheses,
  and consequential architectural forks are separated.
- [Candidate catalog](#catalog): U = userspace representation; F = filesystem
  handles/syscalls; M = metadata/state; S = scheduling; A = application;
  C = contract alternatives; R = reporting sink.
- [Interaction map](#interactions): overlapping benefits and incompatible choices.
- [Experiment briefs](#commissions): bounded, independently commissionable work.
- [Verification and measurement](#gates): common gates and immutable live scope.

Evidence labels used throughout:

- **Observed:** actual trace, retained numeric observation, or checked API/type
  availability. Observed work is not automatically a measured bottleneck.
- **Inspected:** a concrete operation/data lifetime is present in current source.
- **Hypothesis:** the proposed replacement may improve a named workload; it has
  **not** been benchmarked here. No invented percentage or universal win follows.
- **Fork:** a semantic, resource-bound, ownership, or API change. It deserves a
  deliberate selection, not a label such as “small tuning.”

Qualitative complexity is Low / Moderate / High / Very high. These describe
implementation/reasoning burden, **not time estimates**. Scenario abbreviations:
clean, sparse, fresh, dense, wide, deep, and multi-target have their ordinary
fixture meanings; “fresh” includes absent-target initialization.

## Baseline that future experiments must beat honestly

Historical controls, recomputed or checked against retained raw observations:

| Shape | Established result | What it does not establish |
| --- | --- | --- |
| Actual node_modules, P4/J4/K256 | 722.817 → 247.010 ms serial/frontier (~2.9×) | A current-nightly timing or unlimited parallel scalability |
| Whole read-only live roots | 808.672 → 282.329 ms | End-to-end consumer/reseed speedup |
| Root-wide 30k entries | ~80 ms frontier/serial; ~40 ms predecessor entry parallelism | That a second unbudgeted nested Rayon layer is acceptable |
| Dense 4000 changed leaves in separate directories | 4001 epochs; diagnostic ~26.8 ms cumulative drain | That raising K removes decision barriers |
| Owned 30k × 4 KiB fresh Copy | 3450.268 → 1668.983 ms serial/P4 | That required reflinks are inherently faster on small files |
| Same fixture clean repeat | Zero application writes; ~walk-only cost | A cache that skips the paired traversal |

The [evidence account](inspection0.sol61x.md#e1) rules out several false premises:

1. **Current std DirEntry metadata is already fd-relative on this target.**
   Both actual 1.88 and current-nightly executables issue basename-relative
   statx. Manual fstatat does not newly eliminate those parent-prefix walks.
2. **Userspace path allocation is still separate work.** The core eagerly calls
   `entry.path()` even when metadata itself needs no child path.
3. **Current Copy already calls copy_file_range.** “Introduce kernel copying”
   is not a new win. Known length, repeated descriptor metadata, specialization,
   and fallback selection remain legitimate questions.
4. **Linux reflink-copy has no second random staging directory.** Its guard
   owns the supplied output. The outer applier creates the per-leaf directory;
   the library/applier do impose a real close/chmod/reopen lifecycle.
5. **Current nightly has experimental std::fs::Dir.** It is another backend
   building block, with incomplete public operation coverage—not a reason to
   assume all handle work needs a dependency or is already solved by std.

<a id="ranking"></a>

## Ranked starting points

### High confidence that work exists; small semantic surface

Start with **U01** lazy diagnostic paths, **U04** borrowed applier validation
names, **M02** internal metadata handoff, and **A01** known-empty staging
cleanup. These remove inspected/observed work under the existing stable-tree
model. Their **latency payoff remains unmeasured** and may be small once syscalls
dominate. Avoid spending architectural complexity to save an optimized-away or
insignificant allocation.

### High-payoff hypotheses, not small default adjustments

- **M01:** reduce metadata/readlink *count*, rather than only cost per lookup.
- **A03/A02:** bypass or amortize staging for owned absent leaves, where most
  fresh-copy operations have no predecessor to protect.
- **A04/A08:** hold usable clone/copy descriptors and exploit known state
  without losing std's existing offload or fallback correctness.
- **S04:** recover root-wide parallelism under a coherent physical-I/O budget.
- **S02/S03:** improve cursor-near admission or compress certified clean work;
  both need new proofs/bounds, not just a larger K.
- **M04:** reuse one stable donor observation across multiple destinations.

### Consequential forks worth comparing explicitly

**F01/F03** handle-native addressing and resource ownership; **M03** richer
root-bound inspection results; **C01** a genuinely read-only batch/unordered
protocol; **C02** an integrated mirror driver with a different acknowledgment
boundary; **C03/C04** comparison/opacity choices; **C06** a named tuned profile
beside a generic baseline. These may outperform cleanup-only work, but should
not be smuggled into the current API under “internal optimization.”

No global ranking says all clean workloads should precede fresh-copy work:
the operator must choose the payoff target. A clean donor/mirror fleet and a
fresh-copy-heavy application have different ceilings.

### Scenario map and required negative controls

| Candidate | Most plausible affected scenarios | Weak/unaffected or negative control |
| --- | --- | --- |
| U01 | Clean/sparse/wide leaf inspection, long prefixes | Metadata-dominated short paths; link paths still needed without F01 |
| U02 | Deep/long paths, many small clean directory jobs | Shallow/short paths where indirection costs more |
| U03 | Wide listings, many short or repeated names, dense records | Tiny directories; packed-name conversion overhead |
| U04 | Dense/wide supplied apply records | Clean sync emits no records to validate |
| F01 | Deep trees, links, fresh/apply/removal path resolution | Paired regular metadata already fd-relative |
| F02 | Metadata-heavy comparison; possibly remote metadata | Local btrfs may fill the same inode fields at the same cost |
| F03 | Deep/dense handle retention, many roots, multi-target | FD limits and small scopes whose reopening is cheaper |
| M01 | Directory/link-heavy clean trees, one-sided entries, transitions | Paired regular files still need keys; DT_UNKNOWN can regress |
| M02 | Fresh/dense copy and removals | Sparse delta is mostly comparison; new metadata vectors can hurt memory |
| M03 | Dense changed entries/links and compare-then-apply consumers | Clean scans; fresh opaque descendants are not inspected by core first |
| M04 | One donor to several destinations | N=1 must retain an inexpensive path unless its extra cost is selected |
| S01 | Many-small-directory clean/sparse walks | Dense seals, wide single directories, multi-walk retention |
| S02 | Skewed/deep cursor branch behind ready later results | Broad balanced fanout can waste reserved near capacity |
| S03 | Fully examined clean fanout/subtrees | Dense differences/errors provide little compressible work |
| S04 | Wide and mixed wide/small trees | Tiny directories and already-saturated directory concurrency |
| S05 | Many tiny cached directories, high K, fast inspection | Long filesystem stalls hide reducer CPU |
| S06 | Tiny tasks, skew, mixed file sizes, shared-pool contention | Any locality bias can hurt balancing or another pool user |
| S07 | Wide roots/deep wrappers before independent fanout | Many ready directory jobs already occupy the pool |
| A01 | Every successful staged write, fresh/dense copies | No-op has no staging cleanup |
| A02 | Many writes sharing a target parent | Sparse tiny batches and cross-mount staging choices |
| A03 | Fresh roots/added opaque subtrees/new links | Changed leaves cannot lose predecessor protection |
| A04 | Required/fallback cloning, writable and restrictive modes | Clean scans and platforms without a usable descriptor clone API |
| A05 | Copy metadata, restrictive directories, coarse timestamp targets | No-op; capability assumptions can break convergence |
| A06 | Large stale opaque trees, deep/wide removals | Clean/fresh trees; exact descendant progress may require the walk |
| A07 | Repeated unsupported filesystem-pair cloning | Supported btrfs: discovery/cache overhead with few avoided attempts |
| A08 | Fresh/dense Copy/fallback with varied file sizes | Std already offloads; virtual/short/growing files challenge length policy |
| A09 | Wide apply-result retention | Single-leaf or I/O-dominated batches |
| C01 | Dense read-only audit/reporting | Mutable apply/prune consumers need another contract |
| C02 | Dense/sparse mirror acknowledgment and state reuse | Clean scans have no apply gaps to overlap |
| C03 | Mode/precision/content-identity requirements and validated caches | Weak keys can produce faster but false clean results |
| C04 | Dense paired replacement, explicit expanded/coarse subtree use | Sparse trees suffer unnecessary copying/identity changes |
| C05 | Many tiny selected roots, leaf deployment, caller confinement | One large ordinary root has little root-setup amortization |
| C06 | Stable prevalidated donor/payload fleets | General mutable filesystems lack the selected facts |
| C07 | Dense/deep divergence or report-heavy consumers | Clean scans already have little summary-path output |
| R01 | Dense/wide CLI or structured report output | Existing core timers exclude the sink |

<a id="catalog"></a>

## U — userspace paths, names, lists, and allocations

<a id="u01"></a>

### U01 — construct leaf diagnostic paths only when needed

- **Problem / evidence:** `src/pair.rs:104–119`, `inspect_entry`, eagerly
  materializes `raw.entry.path()` before metadata. [E1](inspection0.sol61x.md#e1)
  proves metadata is fd-relative on the current Linux target. `map_err` path
  clones are already inside error closures; do not misidentify those as normal
  eager work.
- **Proposed change:** materialize a full path on metadata/mtime error,
  unsupported entry, or the existing path-based readlink branch. Keep an
  independently valid owned error path after the entry drops. Generic baseline
  can do this with std; a native backend can avoid the readlink path too.
- **Semantics / links:** no intended output change; independent of F01 and
  additive only where F01 does not already remove the same construction.
  Platforms whose std metadata internally builds a path still need that
  internal path. Not every PathBuf can become a borrowed temporary.
- **Experiment / rejection:** current-compiler allocation counts and CPU samples
  on clean wide/deep/live scopes, plus competing error and literal-link tests.
  Accept only if normal-path allocations actually fall with identical
  diagnostics/order. Reject performance claims if allocator optimization or
  metadata cost makes the difference indistinguishable. **Low complexity.**

<a id="u02"></a>

### U02 — distinguish cursor scratch from concurrent job path identity

- **Problem / evidence:** `walk.rs::advance` joins each frame path; `Roots::inspect`
  joins both roots; `parallel.rs:101` clones the relative path; `frontier.rs::reserve`
  duplicates it between Node and Job. Even a clean ComparedPair owns a DirDiff
  directory path that is never emitted.
- **Proposed change:** compare serial push/pop cursor scratch, a borrowed inspect
  directory argument with late owned-diff creation, and shared path identity
  (`Arc<Path>`, path arena ID, or parent/name chain) for concurrent jobs. These
  are separate variants, not one mutable global buffer.
- **Semantics / links:** serial scratch cannot outlive a yielded owned record;
  independently running jobs cannot borrow a buffer that another job mutates.
  Shared parent chains trade byte copying for indirection/refcounting and may
  retain ancestors. F01/F03 can replace I/O paths with handles, but diagnostics,
  DFS positions, and summaries still require recoverable names.
- **Experiment / rejection:** deep long-name chains plus small/shallow controls;
  measure allocations, copied name bytes, peak retention, and wall/CPU. Force
  sibling completion after cursor movement and inspect early summaries. Reject
  unsafe scratch lifetimes or an Arc/arena whose short-path overhead outweighs
  savings. **Low–Moderate for borrowed clean paths; High for a path arena.**

<a id="u03"></a>

### U03 — fuse listing conversion/merge and reconsider name storage/sorting

- **Problem / evidence:** `pair.rs::{list,entries,split_entries,compare}` keeps
  both std DirEntry (internally owned C name + shared directory) and exported
  OsString, combined-sorts by encoded name/side, allocates separate Entry
  vectors, then merge-compares them. Cached names avoid allocating inside each
  sort comparison: blindly deleting that cache would be worse.
- **Proposed change:** first compare fusing ordered conversion/classification
  without two materialized typed vectors; separately compare two sorted side
  lists with an ordered merge, paired-name groups, and native-backend packed
  name arenas/offsets with reusable buffers. Keep public owned OsStrings only
  for emitted differences/pending child names. A specialized UTF-8/ASCII or
  filesystem-order variant is possible, not an invisible replacement.
  Direct buffered getdents/packed conversion is a further native variant:
  current std/libc already buffers enumeration (32768-byte requests in the
  trace), so this is not removing a syscall per entry. Preserve entry boundaries
  and owned names before reusing the read buffer; test wider buffers separately
  from packed-name allocation savings.
- **Semantics / links:** both listings must finish before metadata error
  selection under today's contract; first error remains global name then side,
  not “finish all source metadata first.” M01 can drive selective observation
  after merge; S04 must not split a paired name incorrectly. Native bytes,
  Windows encoding, comparison order, arena lifetimes, and wide-list memory
  need explicit treatment. Independent per-side sorts are not free merging.
- **Experiment / rejection:** wide equal/mismatched names, many short versus
  long names, native invalid encodings, equal-name competing failures; measure
  allocation/peak bytes and sort CPU separately from stat time. Reject if
  duplicate buffers or comparison overhead grow, or if encoded ordering is
  silently replaced with Unicode/native-Ord semantics. **Moderate; High for
  platform-specific packed enumeration.**

<a id="u04"></a>

### U04 — borrow names while validating public records

- **Problem / evidence:** `sync/src/paths.rs::names` clones every category name
  into a BTreeMap, moves them into a Vec, and byte-sorts again. `apply(&DirDiff)`
  retains the original record until all application joins; these clones are
  not needed to survive a return or detached job.
- **Proposed change:** borrowed OsStr keys/references and masks, preserving
  duplicate/contradiction detection for arbitrary unsorted public records.
  Another variant merge-validates already sorted trusted category streams but
  must not assume all public callers sorted them.
- **Semantics / links:** structured Rayon borrowing fits the record lifetime;
  no `'static` names or new runtime is required. Preserve all-category validation
  before mutation and both legitimate transition masks. M03 can remove more
  revalidation in a distinct typed path, not by weakening this public one.
- **Experiment / rejection:** dense wide synthetic DirDiff allocation census,
  then real owned apply, including unsorted/duplicate/non-UTF-8 inputs. Reject
  any before-validation write, reference escape, or new ordering mismatch.
  Latency improvement can be negligible despite fewer bytes. **Low complexity.**

## F — directory handles, kernel resolution, and syscall policy

<a id="f01"></a>

### F01 — a handle-native addressing backend, not a manual-stat placebo

- **Problem / evidence:** [E1/E4](inspection0.sol61x.md#e1) distinguish already
  relative DirEntry stats from path-based child directory opens, readlink, and
  most sync operations. `sync::children` drops entry handles and later stats
  root-joined paths. Long prefixes can still be resolved repeatedly in kernel.
- **Proposed change:** compare three building blocks: current experimental
  std::fs::Dir (`dirfd`), rustix *at APIs, and cap-std. Open/pin selected roots;
  operate on parent handle + basename using openat, statat/fstatat or statx,
  readlinkat, mkdirat, symlinkat, unlinkat, and cross-parent renameat. Hold File
  descriptors for metadata restore. Std Dir already covers part of this on the
  selected nightly, but lacks public listing/readlink/symlink methods in the
  inspected API. Complement it rather than pretending it is complete.
- **Semantics / links:** rustix AsFd and scoped borrowing/Arc ownership are
  Rayon-compatible; per-task OwnedFd is also possible. Never use process cwd as
  a cursor. Raw relative APIs do not enforce beneath-root resolution, no-follow
  through multi-component paths, mount exclusion, or a tree snapshot. Cap-std
  adds policy and possible cost; use its literal-link contents variants. Root
  handles change behavior after namespace renames: identity-pinned access is
  not necessarily the same as re-resolving a named root. See F03/M03/A04.
- **Experiment / rejection:** small read-only backend spike, then deep-long-path
  and link-heavy controls against already-relative std metadata. Count calls,
  allocations, descriptor high-water, CPU, and actual lookup operands. Add
  symlink/bind-alias/renamed-root semantic probes only on owned fixtures. Reject
  a broad rewrite justified solely by replacing the same statx operation, or
  portability/security claims unsupported by the chosen library. **High.**

<a id="f02"></a>

### F02 — minimal metadata requests and genuine batching

- **Problem / evidence:** std requests BASIC_STATS|BTIME although comparison
  uses type, size, mtime; the current trace prints STATX_ALL. This is observed
  request volume, not proof that btime retrieval dominates. M01 may avoid more
  work by not issuing the lookup at all.
- **Proposed change:** native statx TYPE|SIZE|MTIME masks for comparison and a
  different mask for copied permissions/mount identity. Check returned masks
  and exact timestamp conversion, with fallback where needed. Separately,
  compare syscall submission/completion batching only if profiling justifies
  it: ordinary statx/fstatat is **not** a multi-path batch API. io_uring statx
  would be a new Linux execution/lifetime backend, not a Rayon tuning knob.
- **Semantics / links:** fewer requested fields may still return BASIC_STATS
  at the same cost on local btrfs. DONT_SYNC changes freshness semantics on
  remote filesystems and cannot be silently adopted. Batch completion still
  needs deterministic errors/seals/join boundaries unless C01 changes them.
  F01/S04 and descriptor budgets interact; no Tokio prerequisite follows.
- **Experiment / rejection:** fixed-count same-fd masked versus std calls on
  owned local data; add remote/FUSE only if later available and authorized.
  Preserve pre-epoch/nanosecond keys and missing-mask behavior. Reject if local
  cost is unchanged, error/freshness weakens without a profile, or batching
  costs exceed saved submission overhead. **Moderate for masks; Very high for
  a separately budgeted asynchronous completion backend.**

<a id="f03"></a>

### F03 — make descriptor retention, root authority, and validation reuse explicit

- **Problem / evidence:** std entries already retain parent handles during
  conversion, then current ready results discard them. Handle-bearing jobs,
  prepared records, and multi-target caches can instead retain FDs across K
  results or caller yields. `paths::{root,relative}` rechecks prefixes by path.
- **Proposed change:** compare per-job parents, borrowed epoch roots, Arc-held
  parent pairs, and bounded cached directory handles. Offer an owner-validated
  root/capability object to reuse root checks, with a separate path constructor.
  Model independent directory streams: dup/try_clone can share offsets;
  rustix Dir::read_from reopens `.` for its own cursor.
- **Semantics / links:** parent metadata operations can safely share an
  immutable handle, not an unsynchronized enumeration cursor. Count descriptors
  separately from J/K, test low RLIMIT_NOFILE, and define release on yield/drop/
  error. A traversal-only handle (current std's open_for_traversal) is not
  guaranteed readable for enumeration; metadata mutation may need different
  descriptor rights too. Root FDs alone do not cheaply prove ancestry/non-overlap
  under mounts. Handle-native descent can also reach deeper paths than one
  full-path syscall accepts, making stack/depth limits newly consequential.
  Same-filesystem rename constraints remain: one global staging directory can
  fail EXDEV at nested mounts, unlike current parent-local staging. No-follow
  or openat2 resolve flags must reflect an explicit mount/alias policy.
- **Experiment / rejection:** simulate saturated K, deep trees, multiple walks
  on one pool, and descriptor exhaustion. Compare handle retention versus
  reopen work; require no leaks/deadlocks and documented root-rename behavior.
  Reject an unbounded “cache all FDs” strategy or implicit confinement claim.
  **High; depends on F01/M03 decisions rather than a mechanical PathBuf swap.**

## M — observe less, reuse state, or carry richer observations

<a id="m01"></a>

### M01 — d_type-first, paired-key-only, and lazy literal-link observation

- **Problem / evidence:** `pair.rs::inspect_entry` stats every child and reads
  every link before comparing side/name presence. [E2](inspection0.sol61x.md#e2)
  confirms cached d_type on this fixture. Directory size/mtime is unused;
  one-sided leaf attributes and links in file↔link transitions are discarded.
- **Proposed change:** sort/merge names and cheap types first; fetch size/mtime
  for paired regular files and link contents for paired links. Keep examined
  special-file rejection and DT_UNKNOWN fallback. A portable version can use
  std file_type; a native enumerator can retain an already-fetched unknown-type
  fallback metadata result rather than fetch it again.
- **Semantics / links:** stable-tree outputs can match while syscall/error
  observations differ. The current read-only API reads even a one-sided link;
  deferring it can defer its failure until apply or omit that observation in
  report-only mode. Explicitly decide whether that is acceptable. Naive std
  file_type then metadata can **double** stats on DT_UNKNOWN filesystems.
  M03 rich records may want attributes that names-only comparison can skip.
- **Experiment / rejection:** count directory/regular/link/unknown lookup
  classes on clean/live, one-sided, transitions, and sparse data; use a fake
  listing seam to force DT_UNKNOWN/errors. Require output and chosen error
  contract parity; reject “wins” obtained by following links, hiding specials,
  or silently weakening required coverage. **Moderate–High.**

<a id="m02"></a>

### M02 — hand validated metadata through the existing sync operation kernel

- **Problem / evidence:** `apply_with::validate_entry` inspects immediate source
  and target, then copy/remove inspects again. Recursive `copy_directory`
  dispatch stats a child, then copy_leaf/copy_directory immediately restats it.
  Consecutive duplicate path stats are visible in [E4](inspection0.sol61x.md#e4).
- **Proposed change:** first remove the direct recursive double inspection by
  passing its Metadata/known type to the next internal function. Separately
  retain minimal validated source/target state for a public record batch rather
  than throw it away and repeat it. Do not precompute two full absolute paths
  per wide entry merely to save a later join without measuring memory.
- **Semantics / links:** the existing stable-tree model permits short-lived
  reuse; public syntax/type/ancestor validation still completes before writes.
  Opened-file identity/fstat can remain a distinct correctness check. Less
  restatting is not hostile-race protection. Full Metadata per operation may
  enlarge a wide batch substantially; minimal state versus full attributes
  deserves a controlled comparison. M03 can extend reuse across the core seam.
- **Experiment / rejection:** traced Copy and strict-clone paths, readonly modes,
  transitions, failed preparation/cleanup, and retry. Require fewer duplicate
  lookups without changed progress or writes through aliases; measure batch RSS.
  Reject if retention outweighs reduced work or a real check is accidentally
  removed under the guise of a duplicate. **Low–Moderate internally; Moderate
  for full-record state retention.**

<a id="m03"></a>

### M03 — richer records, inspection leases, or root-bound prepared operations

- **Problem / evidence:** public DirDiff deliberately discards stats/link data
  and is freely constructible. Sync therefore cannot treat it as a trusted
  inspection result. Changed links may be read during comparison and again
  during installation. Names-only is a chosen boundary, not an immutable law.
- **Proposed change / fork:** compare an optional detailed projection, a
  producer-created root-bound inspected result, and an ephemeral prepared-op
  lease retaining relevant parent/file handles. Keep the existing names-only
  public projection as one generic use case. Decide whether captured values or
  freshly opened source objects are authoritative at apply time.
- **Semantics / links:** a Metadata value is not a snapshot; File pins an inode,
  not its immutable bytes. Tie reuse to explicit source stability/generation and
  root identity. Holding DirEntry retains an FD. Borrowed details cannot be
  stored beyond their epoch without ownership. New contracts can reduce
  validation but must not let arbitrary constructed records masquerade as
  validated operations. F03 budgets and C02 acknowledgment are prerequisite
  design choices, not just additional struct fields.
- **Experiment / rejection:** in-process names-only compare+apply versus an
  inspected path with identical stable data; count repeated stats/readlinks,
  allocation/FD retention, and failed-prefix coverage. Specify source replacement,
  early drop, prune, and stale-token behavior on owned tests. Reject an API that
  silently applies stale metadata or forces all report consumers to retain
  heavy handles. **High architectural complexity.**

<a id="m04"></a>

### M04 — one stable donor, many destinations

- **Problem / evidence:** each current walk independently lists/stats the same
  original. Existing measurements are one pair, not a multi-target throughput
  or cache study. Repeated donor work is inspected structure; payoff is a
  hypothesis relevant both to generic deployment and the motivating fleet.
- **Proposed change / fork:** compare bounded per-directory donor-result
  broadcast to N target walkers, an immutable donor catalog, and caller-supplied
  generation/manifest state. Reuse sorted native names, keys, literal links,
  and possibly source FDs. Keep target traversal and per-target reporting.
- **Semantics / links:** declare when a source generation is stable and how
  it expires; directory mtime alone cannot validate descendants. An operator's
  selected immutable donor can justify stronger reuse than generic live paths.
  File::try_clone/dup shares the file offset: concurrent ordinary copies need
  independent opens or explicitly positioned I/O; FICLONE does not consume a
  read cursor. Every target still needs independent private inodes/CoW behavior.
  Slow/failed targets must not retain an unbounded broadcast backlog or cancel
  unrelated mirrors accidentally. See F03/M03/C03/C06.
- **Experiment / rejection:** owned 2/4-target same-generation runs, counting
  donor inspections once versus N, peak catalog/FD bytes, aggregate throughput,
  and per-target tail/error progress. Reject a universal replacement if N=1
  regresses; a multi-target-only profile may explicitly accept setup cost.
  Reject stale generation reuse or a “cache” that conceals fresh target scans.
  **High; Very high for persistent invalidation/change-journal support.**

## S — admission, granularity, ordered commitment, and pool use

<a id="s01"></a>

### S01 — measured J/K policy and named workload profiles

- **Problem / evidence:** `ParallelLimits::for_pool` chooses J=P/K=4P. Historical
  live P4 K16 saturates much more than K256; J1/K0 removes useful overlap.
  K256 also increases retained state. These are real observations, not proof
  that every application should raise its default.
- **Proposed change:** controlled current-compiler retuning or named profiles
  for clean small-directory walks, dense interactive use, and constrained
  multi-walk memory. Report P/J/K explicitly; optionally let callers supply a
  measured policy rather than a second thread count. No defaults change here.
- **Semantics / links:** preserving count units makes this lower-risk than
  S02–S04. Multiple walkers share workers but add K/name/summary memory; tuned
  profiles need limits for caller concurrency too. K is not bytes or descriptors.
- **Experiment / rejection:** small targeted P2/P4 J/K sweeps on the read-only
  anchors plus dense/wide controls, with separate utilization/RSS runs. Prefer
  policies that improve the target without unacceptable tails/retention. Reject
  a universal-default claim based on one topology or cross-compiler controls.
  **Low implementation complexity; Moderate policy/measurement burden.**

<a id="s02"></a>

### S02 — cursor-local speculation budgets instead of indiscriminate lookahead

- **Problem / evidence:** later ready siblings can occupy all K while an early
  deep branch advances using only demand. `frontier.rs::{candidate,pump}` and
  reserved-demand tests expose this intentionally. Reservation prevents
  deadlock; it is not a throughput guarantee.
- **Proposed change / fork:** partition lookahead into cursor-near and farther
  sub-budgets, rank eligible work by distance/branch, or donate far capacity
  without blocking demand. Retain nonblocking admission and bounded outstanding
  units. Do not simply remove the extra demand slot to save a branch/counter.
- **Semantics / links:** new priority/accounting policies must preserve seals,
  serial-first errors, committed prefixes, and P1/K0 progress. Reserved near
  capacity can sit idle or harm broad balanced trees. Interacts strongly with
  S03 compression and S05 candidate indexing; compare separately first.
- **Experiment / rejection:** deterministic completion permutations, saturated
  later results plus unread first-branch descendants, and owned skew/deep shapes;
  then current live controls. Measure real idle-with-eligible-work intervals,
  not worker IDs alone. Reject deadlocks, uncharged speculative state, repeated
  rescanning, or gains only from quietly raising the effective budget. **High.**

<a id="s03"></a>

### S03 — compact certificates for fully examined clean subtrees

- **Problem / evidence:** current K counts every uncommitted node, including
  ready clean leaves and expanded clean descendants. Many such results can
  occupy admission while holding little decision-relevant information.
- **Proposed change / fork:** collapse a completely examined clean subtree or
  contiguous clean sibling run into an ordered coverage certificate: count,
  position, and enough structure to commit coverage later without reopening.
  A clean immediate directory alone is **not** a clean subtree certificate.
- **Semantics / links:** do not prematurely add physical work to public Summary;
  an earlier stop/error excludes later certificates. Releasing K while retaining
  certificates changes the advertised K bound: charge certificates/bytes under
  an explicit new budget, or prove bounded range compression. Never free nodes
  that still own live child handles/errors. No directory-mtime shortcut follows.
  F03/M03 heavy retained details may conflict with compact certificates.
- **Experiment / rejection:** pure-state adversarial completion schedules with
  early errors/prunes, huge clean fanout, and skew. Prove exact prefix/count
  parity, no additional I/O, and bounded certificate memory before timing.
  Reject “compression” that grows an unbounded whole-tree side table or hides
  errors/freshness assumptions. **High–Very high.**

<a id="s04"></a>

### S04 — wide-directory chunks sharing one physical-work budget

- **Problem / evidence:** the accepted frontier calls the sequential pair
  kernel for one whole immediate directory. Historical root-wide median stays
  ~80 ms versus ~40 ms in the predecessor metadata-parallel control; diagnostic
  active concurrency is one despite P4/J4/K256.
- **Proposed change / fork:** enumerate/order a directory, then expose bounded
  name-group metadata/readlink chunks while other directories share the same
  I/O admission. Adaptive quanta can favor entry chunks when a single directory
  monopolizes work and ordinary directory jobs otherwise.
- **Semantics / links:** define whether J still counts directories or is replaced
  by work units; do not advertise the old bound while multiplying J×inner jobs.
  A parent cannot consume a permit and block waiting for children when every
  permit is held by a waiting parent. Use cooperative/nonblocking coordination
  or one inline chunk with donated capacity. K directory results and chunk
  buffers are distinct resources. Preserve paired-name/error order and full
  immediate-listing completion before delivery. Interacts with U03/F02/S07.
- **Experiment / rejection:** gated actual metadata calls on wide-only and mixed
  wide/small trees; assert physical concurrency/admission, P1/K0/nested-pool
  progress, serial-first errors, and seals. Reject a wide win obtained through
  hidden oversubscription or a regression on small directories. **High.**

<a id="s05"></a>

### S05 — profile and shorten reducer/search/arena critical sections

- **Problem / evidence:** `frontier.rs::candidate` allocates a search Vec and
  walks open frames/admitted clean nodes for each reservation; `complete→pump`
  does this under the shared mutex. `pump` returns fresh Job vectors, while
  arena slot storage is already reused. CPU increase is measured historically,
  but its mutex/search/allocator fraction is **not isolated**.
- **Proposed change:** reusable search scratch or an incrementally maintained
  eligible-owner index/cursor; reduce path allocation under the lock; reserve
  a small work batch and dispatch outside it as today. Profile before replacing
  a working generational arena with a more complex lock-free structure.
- **Semantics / links:** DFS candidate priority is component position, not flat
  full-path ordering. Runnable caches must invalidate on promotion, seal, prune,
  frame removal, and arena generation reuse. S02/S03 alter that maintenance;
  large queues can cost more than rescanning a small K.
- **Experiment / rejection:** separate allocation/lock-hold/lock-wait profiling
  on many tiny directories and mocked fast I/O; compare actual filesystem
  controls afterward. Reuse completion-permutation tests. Reject optimizations
  whose bookkeeping dominates, holds the lock over I/O, or changes admitted
  work unfairly. **Moderate; High for an indexed priority structure.**

<a id="s06"></a>

### S06 — work quanta, pool locality, and filesystem contention

- **Problem / evidence:** every directory completion can spawn successors;
  copy recursively invokes structured parallel iterators even on tiny child
  lists. Rayon LIFO/work stealing balances skew, but may scatter nearby inode
  work, contend on allocator/filesystem metadata locks, or compete with other
  application pool users. Higher historical P was not uniformly better.
- **Proposed change:** bounded inline continuation quanta, sequential tiny-list
  cutoffs, and cost-aware chunks for metadata versus large copying. Compare
  breadth against directory locality; expose hints/profile choices rather than
  creating a private/global pool. Consider caller-shared resource admission
  only if multi-walk contention justifies its extra architecture.
- **Semantics / links:** a quantum must expose stealable siblings before
  monopolizing a deep branch and must not introduce unbounded Rust recursion.
  P bounds running blocking workers, not storage capacity or fairness. A shared
  semaphore that blocks Rayon workers waiting for queued releasers can deadlock.
  S04 budgets and A09 result aggregation affect granularity measurements.
- **Experiment / rejection:** current P1/P2/P4 and selected larger-pool controls,
  balanced/skewed shapes, mixed file sizes, and two competing walks; measure
  wall/CPU, tails, kernel calls, allocation, and occupancy. Reject tuning that
  only trades caller latency for total throughput without naming that tradeoff.
  **Moderate; High for shared cross-walk admission.**

<a id="s07"></a>

### S07 — schedule source and target listings independently where useful

- **Problem / evidence:** `pair.rs::entries` lists original then target serially;
  a wrapper chain or one huge root cannot exploit across-directory fanout until
  both lists finish. Listing overlap is different from entry metadata chunks.
- **Proposed change:** try a joined two-side listing stage with deterministic
  original-first error selection, possibly only when otherwise idle capacity is
  available. A native backend can overlap enumeration/cheap-type conversion,
  but sorting/comparison still waits for both sides under current semantics.
- **Semantics / links:** two simultaneous large listings increase transient
  buffers and active I/O; nesting join inside every directory job can undermine
  J's physical meaning. Charge both sides appropriately or choose a single
  coordinator work unit design with S04. Never assume racing errors preserves
  existing source-listing precedence. M04 may make source listing shared.
- **Experiment / rejection:** wide root, deep wrappers, and many-small-directory
  controls; force source/target listing failures and P1/nested-pool execution.
  Reject if extra I/O contention/dispatch costs outweigh overlap or if memory
  expands beyond the stated budget. **Moderate–High.**

## A — filesystem application and copy/reflink/removal costs

<a id="a01"></a>

### A01 — known-empty successful staging cleanup

- **Problem / evidence:** `apply.rs::cleanup` always invokes generic recursive
  removal after a successful rename emptied its private TempDir. [E4](inspection0.sol61x.md#e4)
  shows two path stats and a directory listing before rmdir, per successful leaf.
- **Proposed change:** direct owned empty-directory removal on the proven
  successful path; retain robust recursive cleanup for failed preparations,
  failed transitions, and partially populated staged trees. Another variant
  can attempt empty removal first with a narrowly handled not-empty fallback.
- **Semantics / links:** preserve meaningful cleanup errors and installed-leaf
  counts. Ownership is the created TempDir, never an inferred prefix matching
  arbitrary user content. No-op sync must not create staging to begin with.
  A02/A03 may remove most instances, so combined benefits are not additive.
- **Experiment / rejection:** successful file/link/whole-directory transition
  traces plus injected preparation/install/cleanup failures and readonly staged
  descendants. Require fewer empty cleanup observations without leaked artifacts
  or suppressed secondary errors. Reject if generic failures are incorrectly
  treated as expected not-empty. **Low complexity.**

<a id="a02"></a>

### A02 — amortize private staging across a record or parent batch

- **Problem / evidence:** fresh copying makes one temporary directory for every
  regular leaf and link, despite independent names sharing a target parent.
  The 30k fixture's actual initialization workload includes this pattern.
- **Proposed change:** lazily create one parent-local private staging directory
  with unique per-operation absent child names for a joined batch. Compare
  per-record, per-copy-directory, and limited worker-local variants independently.
  Keep each changed leaf's prepare-before-rename guarantee.
- **Semantics / links:** shared staging is not a license to share mutable file
  state; assign names collision-free and join before cleanup/final parent modes.
  Cleanup must distinguish installed and still-prepared children, preserving
  secondary errors. One root-wide staging area may cross nested mounts and make
  rename fail EXDEV. A readonly target parent's temporary permissions still
  need scoped ownership. F03/A01/A03 are coupled alternatives.
- **Experiment / rejection:** owned fresh/wide/dense and mixed-transition
  batches, force one failed sibling while others install, inspect leftover
  artifacts and progress, and add a mount-boundary test only in owned authorized
  infrastructure. Reject a win requiring global-name races or worse recovery.
  **Moderate–High.**

<a id="a03"></a>

### A03 — direct exclusive creation for genuinely absent owned leaves

- **Problem / evidence:** `copy_leaf` stages new entries just like replacements;
  a fresh target has no old leaf to preserve. New links also pay mkdir/rename/
  cleanup although their creation is a single native operation.
- **Proposed change:** separate absent-create from changed-leaf installation.
  Copy with create_new and retain the usable descriptor through successful
  metadata; clone to an absent final path using exclusive ownership; directly
  create a new literal link. On preparation failure remove only the file this
  operation owns and report any cleanup failure/partial artifact.
- **Semantics / links:** do not stamp failed payloads with source timestamps or
  count them as installed. Never truncate an unexpectedly existing destination,
  write through a hardlink, or apply this to changed files because it is faster.
  Source special rejection and strict-no-fallback remain. Existing transition
  preparation is a distinct case: preserve the old leaf before removing it.
  This changes failure visibility for a new partial leaf, not successful
  convergence. A02 competes for the same fresh-copy cost.
- **Experiment / rejection:** Copy, real nonempty clone, unsupported clone,
  partial-copy fault, exact mtime/mode, restrictive umask, existing symlink/
  hardlink conflicts, and retry. Reject leaked final partial files concealed
  as success or any loss of the old-leaf guarantee. **Moderate complexity.**

<a id="a04"></a>

### A04 — retain the cloned output descriptor through metadata and installation

- **Problem / evidence:** Linux reflink-copy opens/stat-checks the source,
  creates/clones/chmods/closes output; sync chmods and reopens that private
  output, then sets mtime/checks/final chmod. Three permission-setting calls are
  observed even for writable-mode regular files. This is descriptor churn,
  not two separate Linux staging trees.
- **Proposed change:** compare a held-File clone primitive, a narrowly justified
  dependency API extension, or a Linux rustix FICLONE adapter over owned source/
  output descriptors. Prepare metadata exactly once before the final restrictive
  chmod. A smaller variant can skip proven-unnecessary private-mode widening.
- **Semantics / links:** a path API's return type cannot magically keep its
  closed descriptor alive. Range-clone builder APIs have zero-length/platform/
  alignment restrictions; do not substitute them blindly. Preserve strict and
  fallback distinctions, source types, per-file independent inodes, and support
  for readable files whose final mode prohibits reopening. This can be studied
  without rewriting all traversal into F01; M02/A08 share the state boundary.
- **Experiment / rejection:** per-file syscall lifecycle across writable,
  readonly, mode-000-readable where supported, zero/unaligned/large files;
  preserve old leaf on every preparation failure and independently mutate each
  source/target copy. Reject portability claims beyond verified APIs or fallback
  that converts permissions/not-found into “unsupported.” **Moderate native
  adapter; High for a portable dependency/interface change.**

<a id="a05"></a>

### A05 — explicit metadata/finalization policies, not silent weakening

- **Problem / evidence:** files receive exact mtime readback before install;
  new directories receive postorder modes even when already equal. Existing
  target directories are checked/relaxed/restored per authorized batch. These
  preserve convergence/readonly behavior, but are real syscalls.
- **Proposed change:** first reuse already known directory permissions and
  avoid demonstrably redundant permission calls. Separately compare a declared
  known-exact-timestamp filesystem profile versus default per-file readback,
  and longer-lived directory permission guards inside a fused driver.
- **Semantics / links:** default readback detects silently rounded/clamped
  timestamps; one successful sample does not prove all SystemTimes representable.
  Filesystem-pair capability facts must be scoped/invalidation-aware (A07).
  Postorder modes cannot be applied before joined children. Never chmod source,
  a replaced symlink target, or an unrelated hardlinked old leaf. Mode-only
  repair and ownership/ACL behavior are C03 choices, not free optimizations.
- **Experiment / rejection:** readonly/execute-only/new-directory umasks,
  pre-epoch/nanosecond/coarse timestamp targets, failure restoration, and no-op
  identity/ctime observation. Reject a faster “success” that leaves repeated
  differences, hides representability loss, or relies on reopening after mode
  000. **Moderate; High for changed verification/guard contracts.**

<a id="a06"></a>

### A06 — removal traversal, descriptor locality, and report granularity

- **Problem / evidence:** `Context::remove` recursively stats children, lists/
  sorts directories, handles readonly modes, counts every successful descendant,
  and rejects encountered specials. Independent units can parallelize, but one
  enormous target-only subtree is serial. There is no existing extra preflight
  walk to remove.
- **Proposed change:** first combine d_type/retained-parent handling with exact
  counters; compare structured sibling removal only above useful granularity,
  with joined postorder rmdir and failure restoration. A separate coarse-unit
  API could use a platform remove-tree primitive and report removed subtree
  units instead of per-descendant counts.
- **Semantics / links:** std remove_dir_all may remove special leaves that
  today's kernel rejects and does not provide the same per-entry partial counts.
  Adding a whole validation/count sweep first can erase its advantage. Directory
  rename-to-trash changes when failure is visible and leaves cleanup obligations;
  it is a larger explicit policy, not permission to asynchronously delete caller
  state. Cross-mount/alias and readonly leaf limits remain. See F01/C04.
- **Experiment / rejection:** owned wide and deep stale trees, readonly
  directories, external literal links, socket/FIFO rejection, competing failures,
  and retry. Reject any traversal of link targets, unjoined deletion, or a
  coarse counter presented as exact descendant progress. **Moderate–High.**

<a id="a07"></a>

### A07 — scope fallback knowledge to actual filesystem pairs and operation class

- **Problem / evidence:** ReflinkOrCopy can attempt a predictably unsupported
  clone for every file. Strict cloning on the measured btrfs fixture succeeds;
  existing fallback tests show cross-filesystem copy. No repeated-failure timing
  campaign or blanket filesystem-type capability proof exists.
- **Proposed change:** caller-supplied capability facts or a bounded run-scoped
  filesystem-pair/operation-class cache learned from **real needed operations**.
  A held-FD fallback can reuse already opened source/output state before adding
  a negative capability cache. Probe explicitly only if selected, target-side
  where possible, never through hidden writes to the authoritative source.
- **Semantics / links:** st_dev or statfs type alone is not a complete capability
  identity; mount/volume, subvolume/options, encryption/compression and per-file
  restrictions matter. Do not globally cache EINVAL/EPERM as unsupported, or
  suppress NotFound/PermissionDenied/AlreadyExists. Strict Required must still
  guarantee each real clone, not pretend a cached Copy satisfied it. An early
  fallback preference can forgo possible later clones and needs explicit policy.
  No-op must not perform hidden probe writes. See A04/A05/F03/C06.
- **Experiment / rejection:** owned btrfs↔tmpfs and supported pair, inject a
  file-specific failure followed by a cloneable file; cap cache size and test
  invalidation. Reject misleading outcome counts or a cache whose discovery
  stats cost more than failed clone attempts. **Moderate–High.**

<a id="a08"></a>

### A08 — known-length offload, cached file facts, and tiny-file policy

- **Problem / evidence:** [E3](inspection0.sol61x.md#e3) shows current io::copy
  statting both descriptors for specialization and issuing a successful
  copy_file_range followed by an EOF call. fs::copy already uses offload too;
  its internal metadata wrapper is not a public supplied-metadata interface.
- **Proposed change:** compare File/Take known-length copying (std recognizes
  Take), native explicitly positioned/ranged copying using validated facts,
  and deliberately buffered tiny-file copying as separately named variants.
  Avoid custom Read/Write wrappers that accidentally lose std specialization.
- **Semantics / links:** source length limits are valid only under the declared
  source model; virtual regular files can advertise zero length yet yield data,
  so generic until-EOF behavior cannot silently disappear. Detect short copies
  and preserve partial-offset/fallback correctness. Explicit FICLONE provides a
  CoW guarantee that copy_file_range does not. Sparse allocation, file offsets,
  physical bytes, and ordinary Copy's permission semantics need separate
  observation. Sharing duplicate source FDs across targets shares offsets
  unless positioned APIs are used (M04).
- **Experiment / rejection:** trace calls and measure owned empty/4KiB/mid-size/
  larger/sparse/unaligned files; use same compiler/fixture for every method.
  Include cross-filesystem fallback and supported error classes. Reject a
  purported kernel-copy gain that the control already has, lost content,
  partial duplication, or a size threshold chosen from one sample. **Moderate;
  High for full native fallback/offload selection.**

<a id="a09"></a>

### A09 — aggregate application results without retaining a large success per leaf

- **Problem / evidence:** `Context::batch` parallel-collects one
  Result<Progress,Failure> per immediate item before merging. Progress has ten
  u64 counters; a wide record/subtree retains sizable result slots until all
  children join. This is inspected width-sensitive memory, not a measured
  dominant CPU cost.
- **Proposed change:** per-chunk fold/reduce of successful counters, retaining
  only errors plus stable hierarchical operation ordinals for deterministic
  primary/secondary ordering. Keep cancellation and joining structured.
- **Semantics / links:** completion-order reduction must not replace current
  native-name selection among encountered errors. Nested failed directories,
  discarded private preparations, cleanup/restoration errors, and visible
  partial installs have distinct accounting. There is no need for a custom
  detached queue or one mutex update for every successful syscall.
- **Experiment / rejection:** wide apply with tiny files and many deliberate
  failures; measure retained bytes/RSS and CPU, not just filesystem wall time.
  Gate an error while a sibling is active and compare exact progress/error
  bundles under completion permutations. Reject if ordinal bookkeeping costs
  more than the collected vector or error ordering becomes incidental.
  **Moderate; interacts with S06/A02/A03.**

## C — deliberately different contracts and generic/specialized variants

<a id="c01"></a>

### C01 — a truly read-only ordered-batch or unordered report path

- **Problem / evidence:** the current pull API pays for arbitrary immediate
  mutation/prune decisions even when the consumer only audits. Every differing
  directory seals children; dense input creates thousands of joined epochs.
  That is an API promise with a measured cost, not a fundamental filesystem law.
- **Proposed change / fork:** keep strict DFS apply-in-loop as one product;
  compare a read-only stream that may inspect below differing nodes, an ordered
  batch iterator, and unordered completed-directory reporting. Deterministic
  final output can be restored by an explicit canonicalization step, or callers
  can select a weaker ordering contract.
- **Semantics / links:** early error/stop coverage is no longer necessarily a
  committed DFS prefix. Name-order first-error, first-record latency, buffering,
  prune, and mutation acknowledgment need new definitions. Unordered physical
  observations must not be mislabeled as the current Summary. A read-only API
  is still not an atomic snapshot against external writers. Encoded byte order
  and filesystem-native iteration are different choices (U03/C07).
- **Experiment / rejection:** dense and report-heavy shapes, comparing canonical
  full results against the baseline outside timing; explicitly test early stop,
  multiple errors, and memory under a stalled first directory. Never apply while
  using a read-only relaxed protocol. Reject if canonicalization/buffering
  consumes the throughput benefit or the new coverage story is ambiguous.
  **High architectural complexity.**

<a id="c02"></a>

### C02 — an integrated mirror driver can own a different acknowledgment boundary

- **Problem / evidence:** `mirror_in_pool` alternates joined diff.next and joined
  apply_in_pool. The record producer discards observations; the applier validates
  them again. Dense small records repeat this boundary, while no application
  work exists to overlap on a clean scan.
- **Proposed change / fork:** a root-bound internal mirror driver can retain
  prepared state, batch independent operations, and possibly let disjoint
  comparison work overlap application. Keep the public manual loop for callers
  who need unrestricted per-record policy. A smaller experiment measures the
  pool-install/batch boundary without assuming its overhead is significant.
- **Semantics / links:** current external yields guarantee zero running jobs.
  An integrated internal protocol may change that without weakening the manual
  API, but must prove no reads below unacknowledged mutable ancestors, no
  writes through cached sibling aliases, and joined failure/cancellation/return.
  Background ownership can make Drop block; source/target tasks may contend
  instead of overlap productively. M03/F03 and permission-finalization ownership
  are design prerequisites, not reasons to add a generic executor framework.
- **Experiment / rejection:** compare an internal sealed scope/lease model on
  owned dense and sparse data; gate conflicting descendants and active error
  siblings. Measure full operations, latency, and total CPU. Reject a new driver
  that duplicates the whole kernel or depends on “namespace siblings are always
  independent.” **Very high; select the protocol before building it.**

<a id="c03"></a>

### C03 — comparison keys, precision, mode repair, and validated caches

- **Problem / evidence:** exact type/size/mtime and literal-link equality are
  baseline choices; mode-only drift is invisible. Dropping Metadata::modified()
  alone saves an in-memory conversion, not the already completed stat syscall.
  Directory mtime is not a descendant fingerprint.
- **Proposed change / fork:** explicit profiles for executable/mode correctness,
  timestamp tolerance on coarse targets, verified manifest/content identity,
  or caller-proven immutable generations/change journals. Consider the tradeoff
  as correctness/usefulness as well as throughput. A generic exact profile and
  a tuned stat-cache/profile can coexist.
- **Semantics / links:** size-only or tolerant mtime can miss content changes;
  hashing adds reads rather than a free speedup. Mode repair cannot chmod an
  old hardlinked target inode without affecting aliases—break sharing or obtain
  explicit alias-write authority. Cached directory cleanliness needs a real
  invalidation/snapshot/generation proof for **both** donor and target, not just
  unchanged directory mtime. A05/M04 capability/cache assumptions must be named.
- **Experiment / rejection:** mode-only changes, same-size edits, preserved
  mtimes, nested divergence under unchanged parents, coarse/pre-epoch timestamps,
  journal gaps, and stale generations. Require each profile to specify which
  cases it detects/misses and what “clean” now means. Reject silent false-clean
  behavior disguised as a faster equivalent scan. **High–Very high.**

<a id="c04"></a>

### C04 — opacity, flattened changes, and explicit whole-subtree replacement

- **Problem / evidence:** a one-sided directory counts as one opaque unit;
  the applier traverses it once. There is **not** an existing core descendant
  scan plus an extra full copy preflight to eliminate. Paired dense subtrees,
  however, pay comparison followed by per-entry application.
- **Proposed change / fork:** compare optional expanded one-sided records or a
  shared inspect-and-apply traversal for richer progress/resume; separately
  expose caller-selected whole-subtree replacement/cost policy for dense paired
  regions. Current prune can support an explicit consumer choice, but the
  convenience mirror has no automatic replacement heuristic.
- **Semantics / links:** expanded records change count units, error visibility,
  target-absent traversal, and scheduling dependencies. Wholesale replacement
  changes clean descendant identities/metadata and performs more mutation than
  exact classified children; it needs explicit scope authority and cannot race
  cached descendants. A manifest may justify subtree equality/replacement, but
  directory mtime cannot. A06 coarse removal and C07 reports must agree.
- **Experiment / rejection:** owned sparse-versus-dense crossover with actual
  bytes, clone/copy outcomes, destructive scope, identity observations, failures,
  and retries. Reject an automatic heuristic that deletes clean retained work
  without authorization, or an “optimization” that adds a needless extra walk.
  **High; Very high for shared expanded traversal semantics.**

<a id="c05"></a>

### C05 — leaf roots, selected-entry scopes, and caller-owned prevalidation

- **Problem / evidence:** Applier binds real directory roots; absent initialization
  requires an existing parent. Consumers selecting many leaf payloads/root
  fragments must wrap those scopes and may repeat their own confinement checks.
  The prospective primary consumer already has native authority/policy outside
  this generic library; that does not authorize changing it here.
- **Proposed change / fork:** generic selected-entry/leaf-root operations with
  an explicit parent binding, and a validated-root/selection object reusable
  across comparisons. A trusted typed path can accept owner-supplied validated
  scope without pretending arbitrary DirDiff is trusted. A specialized caller
  can supply known names/types/roots or immutable selection manifests.
- **Semantics / links:** define symlink root as a literal selected leaf versus
  a forbidden directory alias, root transitions, absent ancestors, overlap,
  and cleanup authority. “Prevalidated” is an explicit logical trust contract,
  not necessarily Rust unsafe or a sandbox guarantee. F03 root handles and
  M03 producer-created tokens can encode different strengths.
- **Experiment / rejection:** many tiny selected roots plus the large-tree
  control; count duplicated checks, test overlap/missing ancestors and external
  links, and prove unselected names untouched. Reject convenience wrappers that
  silently broaden deletion scope or remove public-record validation.
  **Moderate–High API complexity.**

<a id="c06"></a>

### C06 — a tuned primary-consumer profile beside a generic baseline

- **Problem / evidence:** one generic path pays for public constructible records,
  portable copy fallback, repeated root validation, and arbitrary mutable input.
  A stable prevalidated Linux/btrfs donor generation and a caller-owned pool can
  legitimately supply stronger facts. Existing timings do not select that
  profile's settings or establish fleet end-to-end speed.
- **Proposed change / fork:** compare a named explicit profile combining only
  validated assumptions: fixed donor generation, required clone support,
  parent-local staging/owned creates, prepared observations, exact supported
  timestamps, and measured admission. Offer analogous generic capability-input
  APIs so this is not hardcoded repository naming/ignore policy.
- **Semantics / links:** do not hardcode `.jj`/`.git`, integrate jj-waltz, or
  silently weaken no-op/alias/source rules. Hardlinking to the donor is **not**
  equivalent to private CoW. Optional preservation of existing within-target
  hardlink topology, or an immutable artifact-cache hardlink mode, is a separate
  contract with different independence/progress semantics—not this profile's
  default shortcut. F01/M04/C03 can be generic mechanisms with tuned selections.
- **Experiment / rejection:** owned fleet-shaped data and a second unrelated
  generic deployment scenario; report one-target and aggregate costs, assumptions,
  and maintenance/dependency surface. Reject a profile that only hides work
  outside its timing boundary or relies on live-tree mutation. **High.**

<a id="c07"></a>

### C07 — optional summary detail and distinct physical/committed reporting

- **Problem / evidence:** `model.rs::Summary` owns topmost, changed-directory,
  and prune path vectors. Frontier commitment updates these under the mutex.
  Clean scans already avoid divergent path vectors; most clean cost is not
  summary formatting. Physical diagnostics are deliberately a separate feature.
- **Proposed change / fork:** selectable counts-only/full summary, sink-driven
  aggregate paths, or arena-backed lazy path materialization. For C01/C02,
  distinguish observed/physical coverage from ordered committed coverage and
  from completed filesystem operations, rather than overloading one counter.
- **Semantics / links:** final owned full paths must eventually be materialized
  if the caller requests them; lazy storage is not free. into_summary currently
  does no further I/O. Removing changed_dirs or exact prunes changes useful
  reporting and cannot be a hidden speed optimization. Opaque units remain
  distinct from descendants installed/deleted by sync.
- **Experiment / rejection:** dense/deep/report-heavy consumers with minimal
  and full reporting, including early break/error/prune and speculation. Measure
  allocation/retention, mutex hold, and sink cost separately. Reject “clean” or
  “complete” answers based only on physical speculative work or skipped prunes.
  **Moderate; High with a new coverage model.**

## R — reporting sink as a separate end-to-end cost

<a id="r01"></a>

### R01 — borrowed streaming serialization, without inventing an unbuffered CLI

- **Problem / evidence:** `cli/src/render.rs` builds nested serde_json::Value
  trees, Vec<Value>, and Unicode String copies per record/summary. `main.rs`
  **already uses BufWriter** and does not flush every record. Core harness
  timings exclude rendering, so they cannot predict output-bound CLI latency.
- **Proposed change:** borrowed Serialize views/native-name adapters written
  directly to the existing buffered sink; optionally selected batch output for
  report-only consumers. Preserve lossless invalid-byte/UTF-16 tags and text
  escaping. Generic structured sinks can avoid format work entirely.
- **Semantics / links:** JSON object key order may matter to text consumers even
  if JSON semantics ignore it; decide compatibility explicitly. Backpressure,
  broken-pipe errors, terminal failed summaries, and first-record latency remain
  real behavior. C01/C07 change how much/when output arrives.
- **Experiment / rejection:** dense/wide native-name records to a null sink,
  file, and throttled pipe, separating scan and serialization timers. Compare
  exact machine-readable values and exit/failure behavior. Reject claims that
  attribute a sink-only win to filesystem traversal or assume buffering absent.
  **Low–Moderate complexity.**

<a id="interactions"></a>

## Dependencies, conflicts, and decisions not to conflate

| Combination | Relationship / assessment consequence |
| --- | --- |
| U01 vs F01 | Userspace leaf paths versus remaining kernel prefix walks; measure separately, never count the same allocation twice |
| U02/U03 + M03/F03 | Compact names/path IDs must remain alive for concurrent jobs, retained descriptors, errors, and caller-owned records |
| U04 vs M03 | Borrowing names preserves arbitrary public records; trusted prepared records are a different stronger input path |
| M01 vs F02 | Eliding a lookup can beat shaving requested fields; DT_UNKNOWN can invert the result. Measure count and per-call cost independently |
| M02 + A04/A08 | Share a source/target state boundary; decide which metadata is a validation check, a syscall-selection hint, or a copied attribute |
| A01/A02/A03 | Empty cleanup, pooled staging, and direct absent creation remove overlapping work. Use independent controls before stacking them |
| A02 + F03 | Parent/mount-local staging and descriptor lifetime must agree; a root-wide staging pool is not automatically rename-compatible |
| S01 vs S02/S03 | Retuning the same K is distinct from redefining what K charges. Report new units and memory explicitly |
| S02/S03 + S05 | New admission/certificates change candidate index invalidation. Do not optimize an old scan and redesign its semantics simultaneously |
| S04 + S07 + S06 | Wide chunks, paired listings, and quanta can multiply hidden concurrency. One physical-work accounting story must cover all of them |
| C01 vs current apply/prune | Read-only relaxed inspection can coexist as another API; it cannot be substituted into the mutable iterator unnoticed |
| C02 + M03/F03/A05 | A fused driver needs ownership of prepared values, descriptors, permissions, acknowledgments, errors, and final joins |
| M04 + C03/C06 | Donor reuse is safe only with a declared generation/invalidation story; target reuse requires its own proof |
| C04 + A06/C07 | Expanded/coarse operations change diff/progress units and failure scope. Avoid calling one subtree unit one removed descendant |
| R01 vs core benchmarks | Output cost was excluded from existing core timings; keep sink gains separate |

The broad forks are **alternatives**, not a requirement to refactor into more
crates, add Tokio, or unify every path behind an executor trait. Breaking changes
may be appropriate for this pre-1.0 library, but each proposed API still needs
its own useful contract. A small portable baseline plus a typed/native fast path
can be preferable to one configurable combinatorial engine; a sufficiently
useful revised generic contract may instead replace the old one. Assessment,
not the old README or this inventory, should select that shape.

<a id="commissions"></a>

## Independently commissionable Sol high experiment briefs

These are proposed commissions, **not dispatched work**. Each builder should
write its own model-suffixed evidence/design account, retain commands/raw results,
and stop at consequential semantic forks for assessment. Implementation in a
future experiment branch requires the later commission; none is performed now.

### SH-A — portable allocation/lifetime corrections (U01, U04; optional U02 clean path)

**Question:** do lazy diagnostic paths and borrowed validation names remove
material allocations without changing semantics? **Scope:** pair inspection and
public-record name preparation only; no syscall/backend/scheduler redesign.
**Controls:** current compiler, wide and long-path clean scopes plus invalid
native records. **Deliver:** allocation census, tiny patch variants, output/
error parity, peak bytes and wall/CPU ranges. **Stop/reject:** unclear ownership,
optimized-away work, or no meaningful benefit. This is independent of native
backend selection and should not require a new dependency.

### SH-B — selective metadata versus narrower metadata (M01, bounded F02)

**Question:** which unused observations can be omitted, and what failure contract
must change? **Scope:** immediate pair inspection, with separate d_type/lazy-link
and mask variants. **Controls:** exact current full-metadata kernel, DT_UNKNOWN
simulation, clean/live read-only and one-sided/transition fixtures. **Deliver:**
lookup/readlink counts by type/presence, error matrix, exact native record parity,
CPU/RSS and timing. **Stop:** before a changed observation/failure policy enters
the baseline; distinguish a selectable profile from a compatible optimization.

### SH-C — reuse sync state and cheap successful cleanup (M02, A01)

**Question:** how much duplicated inspection/empty cleanup can disappear under
the existing stable-tree/partial-progress contract? **Scope:** internal metadata
handoff and owned successful-temp cleanup as independently measured variants.
**Controls:** retained 30k source with fresh private targets; sparse/dense,
transitions, readonly and preparation failures. **Deliver:** per-leaf syscall
lifecycle before/after, exact progress/cleanup-error tests, clean retry and
zero-write repeat. **Stop/reject:** increased wide-batch retention, missing checks,
or failure artifacts misreported as completed installations.

### SH-D — owned absent creation versus staged batches (A03 versus A02)

**Question:** which installation policy best serves fresh versus changed leaves?
**Scope:** separate exclusive direct-new and parent-batch staging variants; keep
changed-leaf old-file preservation. **Controls:** Copy/Required/nonempty fallback,
same current build, owned fixtures only. **Deliver:** creation/rename/cleanup counts,
partial-copy/install fault matrix, clone independence, restrictive modes, and
fresh/sparse measurements. **Stop:** at cross-mount staging or public failure-
visibility changes; do not silently apply direct writes to replacements.

### SH-E — directory-handle backend feasibility (F01, F03)

**Question:** what remaining path resolution can current nightly std Dir,
rustix, or cap-std remove beyond today's fd-relative metadata? **Scope:** first
a read-only immediate/tree inspection spike and explicit handle/lifetime model;
do not rewrite sync at the same time. **Deliver:** API coverage matrix, exact
syscalls, deep/link-heavy and shallow controls, FD high-water/exhaustion tests,
root-rename/no-follow/mount semantics, and dependency/portability burden.
**Stop/reject:** if the only difference is spelling the same statx call or the
handle cache needs unbounded retention. Assess backend selection before expansion.

### SH-F — admission pathology and reducer costs (S02/S03/S05, separately)

**Question:** is cursor starvation best addressed by near-cursor admission,
clean completion certificates, or merely cheaper candidate search? **Scope:**
instrument/simulate first, then one independently controlled variant at a time.
**Deliver:** saturation/permutation proofs, explicit new budget units if any,
P1/K0 liveness, prefix/error/seal parity, allocation/lock profiles, then small
current live/skew controls. **Stop:** before replacing K's all-uncommitted-node
meaning without a new bound. A larger effective budget is not a compression win.

### SH-G — shared-budget wide-directory work (S04; optional S07 control)

**Question:** can wide-entry work scale without hidden nested J×P concurrency?
**Scope:** coherent work-unit/admission design followed by one prototype; preserve
manual-loop semantics. **Deliver:** physical-I/O-bound assertions, paired-name
error order, parent/child no-permit-deadlock tests, actual distinct metadata workers,
and wide/mixed/small timing/resource controls. **Stop:** at an unresolved J unit
or listing-memory contract; do not just enable the predecessor map inside every
frontier job. Paired side-listing overlap should be a distinct variant.

### SH-H — read-only protocol fork (C01, C07)

**Question:** what does an audit-only API gain by abandoning mutation seals and
per-record joined returns? **Scope:** explicit read-only contract, ordered-batch
and/or unordered prototype, not a replacement mirror. **Deliver:** canonical
full-output oracle comparisons, early-stop/error coverage model, dense first-
record/total latency, and bounded-buffer evidence under a stalled early branch.
**Stop/reject:** if the new Summary ambiguously mixes physical and committed
coverage or restores order with unacceptable whole-tree buffering.

### SH-I — one donor/many mirrors (M04, explicit C06 assumptions)

**Question:** does stable donor observation reuse amortize enough work to justify
its state/lifecycle contract? **Scope:** bounded in-memory generation, 1/2/4 owned
destinations; compare broadcast and catalog, not persistent journal invalidation
first. **Deliver:** donor stat/list counts, target correctness and independent
inodes, shared-offset hazards, slow/failed-target backpressure, aggregate and
per-target costs. **Stop:** before assuming mutable target reuse or integration
into jj-waltz. This can be useful to generic deployment tools without repository
rules, but needs its source-generation selection explicit.

### SH-J — descriptor-held clone and known-length copying (A04, A08)

**Question:** can a prepared-file interface remove close/reopen/permission and
EOF/probe work while retaining portability or a clearly named native variant?
**Scope:** owned per-file primitives, independent of a full traversal backend.
**Deliver:** syscall and metadata lifecycle by policy/file size, strict/fallback
error behavior, nonempty real CoW, independent later edits, exact metadata and
old-leaf preservation; compare std File/Take without wrapping away specialization.
**Stop/reject:** platform/range restrictions, partial-copy corruption, source-offset
races, or an unsupported “Copy means no CoW” assumption. Present the portable
baseline and native tradeoff rather than prematurely unifying them.

**Suggested assessment order:** SH-A/B/C give low-coupling information about
actual work removal. SH-D/J address fresh/application cost; SH-F/G address
admission/wide throughput. SH-E/H/I should be selected by desired contract and
consumer payoff, not treated as prerequisites for every smaller experiment.
Avoid parallel builders simultaneously modifying the same production kernel;
independent evidence branches and later integration are the safer experiment
structure. This paragraph commissions no agents.

<a id="gates"></a>

## Shared experiment gates and proportionate measurement

### Scope and fixture rules

- **Strictly read-only:** original `/home/rektide/a/a/opencode`, target
  `/home/rektide/src/opencode-export-qa`, full roots and top-level node_modules.
  Use existing `repeat`/`compare`/read-only diagnostics. Never pass these roots
  to sync, setup, mutating `run`, or a cleanup helper. Do not modify jj-waltz.
- **Mutation only in owned disposable fixtures:** retained `.test-agent/sync`
  sources or a newly created owned area. The 30k source was checked; only its
  marker and source remain, no paired target. Create fresh private targets;
  do not delete existing benchmark sources or assume their equality to a target.
- **Compiler:** ordinary configured toolchain, record `rustc -Vv`, same compiler/
  flags/dependencies for controls and candidates. Old 1.88 reports are historical
  context, not a control against a new-nightly executable. Nightly experiments
  may evaluate available APIs without changing baseline defaults by stealth.
- **Setup separate from timing:** scenario changes, target initialization when
  not the named scenario, restoration, output parity gathering, and independent
  verification belong outside the measured operation. Fresh includes its declared
  initialization. Do not hide required capability/validation work off the clock.

### Correctness anchors

Compatible candidates retain full native records, DFS/component order,
original-before-target local error priority, exact committed prefixes, no I/O
from into_summary, opacity, pruning acknowledgment, and joined returns. A fork
must specify exactly which anchors change and provide the replacement oracle;
“old test fails” is neither automatic rejection nor permission to drop coverage.

All mutation candidates need Copy as well as real nonempty Required/fallback
coverage; every file/directory/link transition; literal/dangling/non-UTF-8 and
external links; root/record validation; hardlink alias independence; readonly
files/directories/umasks; exact supported nanos/pre-epoch timestamps; preparation
failure preserving old leaves; owned artifact cleanup and secondary errors;
joined-on-error siblings; honest partial progress; retry convergence and a clean
second comparison. Observe no-op inode/mode/mtime/ctime and **zero mutating calls**,
not only counters. Ordinary read-triggered atime behavior is not a userland
metadata-repair sweep. A selected different key/alias/verification profile needs
its corresponding explicit tests rather than claiming the original guarantee.

Scheduler/handle candidates additionally prove P1, K0, saturated K, skew, nested/
shared pools, no deadlocking worker waits, no jobs below seals where retained,
no active work after required joins/drop, FD exhaustion/release, and resource
units including wide listings/results—not merely thread counts. Use finite
gates at the actual filesystem boundary to distinguish concurrency from migration.

### Minimal evidence before a larger campaign

1. Source/API ownership model and one discriminating syscall/allocation test.
2. Small semantic/failure fixture matrix that can reject the idea.
3. Same-build alternating control/candidate observations on **only the affected
   shapes**, with warmup, raw samples, ranges, operation/coverage parity, and
   an explicit negative control. Use more observations only when close/noisy
   results need discrimination; do not repeat the whole 495-run matrix by habit.
4. Separate instrumentation for CPU, allocations, FD/retained-memory high-water,
   physical work, budget stalls, and joins; do not substitute instrumented or
   straced durations for uninstrumented timing. RSS is not a strict byte bound.
5. Ordinary workspace/default/optional checks, doctests, clippy and formatting
   for any later implementation. Platform-specific behavior needs actual
   platform verification before portable performance or safety claims.

Existing production mirror does **not** perform an unconditional final whole-tree
verification scan; the benchmark/test clean checks are outside timing. There is
no production rescan sweep to remove and call a speedup. Likewise, a faster
microkernel is not fleet/reseed benefit until its caller avoids earlier payload
deletion and actually adopts the changed operation seam. Such integration remains
another explicitly selected undertaking.

## Assessment questions

Select a target workload and admissible contract before dispatch: clean generic
comparison, fresh private copying, dense mutable mirroring, read-only reporting,
or one donor/many destinations. Then choose whether richer observations, native
handles, relaxed order, changed key/opacity policy, or source-generation authority
are acceptable. The most consequential next choice may be one of those contracts,
not a smaller allocation. Preserve the portable/manual baseline as a control
while deciding—not as a reason to exclude more useful alternatives.

## Connections to earlier inquiry

- [Earlier IO assessment](../fit/io0.glm53m.md): useful inventory of blocking
  facades, native libraries, and consumer-owned scope. The new [syscall ground
  truth](inspection0.sol61x.md#e1) qualifies its suggestion that manual relative
  statx would newly shave prefix resolution per DirEntry: that particular work
  is already relative here. Its old Windows-reflink and unlink-before-copy
  assumptions are not the implemented sync contract. Preserve that independent
  historical assessment rather than silently treating it as today's producer.
- [Earlier Rayon/heads assessment](../fit/rayon0.glm53m.md) and the
  [accepted frontier design](../concurrency/design0.gpt6am.md): precedents for
  shared worker ownership, bounded observations, and mutation seals. The
  measurements replaced its optimistic per-frame expectation with actual
  across-directory work. S02–S07 and C01/C02 revisit the next limits without
  undoing those established lessons by assumption.
- [Paused runtime-seam assessment](../implementation/refactor0.sol61h.md):
  package/dependency separation is not itself a filesystem speedup. A selected
  native backend or new report protocol may make a pure-engine boundary useful,
  but that is an architectural consequence to assess, not a prerequisite imposed
  on every small experiment or authorization to resume the paused rewrite.
- [Sync implementation](../sync/implementation0.sol61x.md) and
  [sync measurements](sync0.sol61x.md): the concrete operation/lifetime baseline
  for A candidates. This inventory revisits choices such as all-leaf staging;
  it neither retroactively claims they were unverified nor calls them optimal.
