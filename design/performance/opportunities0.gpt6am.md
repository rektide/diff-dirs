---
type: OptimizationAssessment
title: Independent optimization opportunities — eliminate work before widening execution
description: Forty independently assessable optimization theories for directory comparison and optional synchronization, with source-verified Linux behavior, contract forks, and bounded experiment briefs.
resource: /design/performance/opportunities0.gpt6am.md
status: independent-proposal-for-comparison
generated: { by: "model:openai/gpt-6-astra#medium", at: 2026-10-18 }
sources:
  - resource: "opencode://ses_eb2ee3586ffeCmNiSndJnP80V0"
    title: Independently inspected current comparison, frontier, sync kernels and pinned dependencies; developed optimization inventory and discriminating experiments without reading the parallel same-wave assessment
  - resource: /README.md
    title: Current comparison, error precedence, ordered-prefix and mutable-consumer contract
  - resource: /src/pair.rs
    title: Combined sorted listings, eager paths and metadata, intermediate vectors
  - resource: /src/walk.rs
    title: Serial DFS frame ownership and summary assembly
  - resource: /src/frontier.rs
    title: Admission search, path copies, retained nodes, mutex and joined epochs
  - resource: /src/parallel.rs
    title: Borrowed pool, default admission limits and serial directory kernels
  - resource: /src/model.rs
    title: Names-only records and exact-prefix summary payload
  - resource: /sync/README.md
    title: Public validation, staging, timestamp, partial failure and copy-policy contract
  - resource: /sync/src/lib.rs
    title: Mirror loop and repeated root binding boundaries
  - resource: /sync/src/apply.rs
    title: Reinspection, per-leaf staging, metadata finalization, recursive batches and deletion
  - resource: /sync/src/paths.rs
    title: Repeated ancestor validation and BTreeMap category reconstruction
  - resource: /sync/src/platform.rs
    title: Platform permissions and leaf-operation differences
  - resource: /design/concurrency/design0.gpt6am.md
    title: Prior accepted frontier rationale and mutation boundaries
  - resource: /design/implementation/implementation1.sol61x.md
    title: Implementation invariants and correctness evidence
  - resource: /design/performance/parallel1.sol61x.md
    title: Existing live and topology measurements, admission saturation and resource limits
  - resource: /design/sync/brief0.gpt6a.md
    title: Selected first sync scope, rather than a permanent ban on contract reconsideration
  - resource: /design/sync/implementation0.sol61x.md
    title: Implemented optional applier and verification account
  - resource: /design/performance/sync0.sol61x.md
    title: Existing nonempty owned-fixture comparison, copy and clone observations
  - resource: /design/index.md
    title: Existing reading map consulted for surrounding evidence connections
  - resource: https://github.com/rust-lang/rust/blob/1.88.0/library/std/src/sys/fs/unix.rs
    title: Verified DirEntry metadata uses dirfd-relative statx/fstatat on Linux GNU and file_type uses d_type when available
  - resource: https://github.com/rust-lang/rust/blob/1.88.0/library/std/src/sys/pal/unix/kernel_copy.rs
    title: Verified File-to-File io::copy specialization, metadata probes and kernel-copy fallback behavior
  - resource: file:///home/rektide/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/reflink-copy-0.1.30/src/lib.rs
    title: Verified public clone/fallback distinctions
  - resource: file:///home/rektide/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/reflink-copy-0.1.30/src/sys/unix/linux.rs
    title: Verified open, source metadata, create-new, FICLONE and permission sequence
  - resource: file:///home/rektide/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/reflink-copy-0.1.30/src/sys/utility.rs
    title: Verified AutoRemovedFile creates the supplied destination, without a second random staging directory or rename
---

# Judgment first

The next useful work is **several small falsifiable experiments**, not an executor
rewrite. Three different cost centers need different answers:

1. **Clean/sparse reconciliation:** full paired traversal dominates. Remove
   unnecessary inspection and allocation; keep the measured P4/J4/K256 control.
2. **Fresh/dense application:** leaf preparation, metadata and directory namespace
   operations dominate tiny files. Reuse metadata, simplify staging and hold the
   output descriptor across clone and finalization.
3. **One donor, many workspaces:** amortize the donor's enumeration and keys under
   an explicit stable-generation lifetime. This is the most attractive large
   contract fork, because it removes repeated work rather than only overlapping it.

**Important corrections to tempting premises:**

- Rust 1.88 on Linux GNU already does `DirEntry::metadata()` through `dirfd` plus
  relative `statx` (fallback `fstatat64`), with no-follow flags. An FD backend
  does not newly remove full-path resolution from every existing comparison stat.
- `pair::inspect_entry` nevertheless constructs `entry.path()` eagerly for
  every entry. This allocation is separable from the kernel lookup above.
- Rust 1.88 `io::copy` for the actual `File` arguments already attempts
  `copy_file_range`, then suitable alternatives/fallbacks. On CoW filesystems
  Copy can itself obtain reflink-like kernel behavior; it simply promises none.
  Existing Copy-versus-Required timing is not proof of data-copy-versus-CoW cost.
- On Linux, reflink-copy 0.1.30's `AutoRemovedFile` is the **requested output
  file**, guarded for cleanup. It does not add another random temp directory or
  rename inside ours. The redundancy is open/stat/chmod/close/reopen/finalize,
  not a verified double-staging directory scheme.

This is research/design only. No production edits, live scans, new timings or
mutation experiments were performed. Existing reports are attributed evidence,
not this session's measurements. Same-wave Sol optimization material and scratch
were not read. Default serial/std-only and optional Rayon are useful baselines;
names-only records, one mode and strict ordering are revisitable design choices.

## Priorities

| Priority | Candidates | Why |
| --- | --- | --- |
| First small trials | A01, A02, A03, A06; S01, S02 | Concrete repeated work visible in source; low conceptual cost; useful controls for bigger designs |
| Throughput policy trial | Q01, Q02 | Existing evidence directly identifies K saturation; no inference from worker count alone |
| Potentially large backend wins | S03, S04, F02 | Fresh/dense application repeats many namespace operations and descriptor transitions |
| Topology-specific win | Q04 | Current whole-directory granularity demonstrably serializes a root-wide workload |
| Large contract bets | C01, C02, C03 | Donor reuse, trusted operation flow and a read-only execution contract can remove structural costs |
| Measure before complexity | A04–A05, F03–F04, Q03, Q05–Q07, S05–S10 | Plausible, but dominance or broad benefit not established |
| Specialized redesigns | C04–C06 | May beat generic walking decisively, with new ownership/invalidation or filesystem commitments |

“High-confidence” means confidence that a redundant operation exists, **not a
predicted percentage speedup**. A small optimization may be lost under kernel
cost; retaining it still requires demonstrated resource or performance value.

# Evidence and experiment rules

Existing [frontier measurements](/design/performance/parallel1.sol61x.md) report
clean node_modules with 191,805 entries per side and 22,828 committed directory
pairs: final serial 722.817 ms versus P4/J4/K256 247.010 ms. The default K16 at P4
was materially weaker. Root-wide 30k files favored the predecessor per-metadata
backend (~40 versus ~80 ms). Dense differences had many epochs and low active
directory overlap. These establish directions, not future wins.

[Sync measurements](/design/performance/sync0.sol61x.md) report owned 30k 4-KiB
files: fresh Copy 3450.268 ms serial versus 1668.983 ms P4; clean 75.532 versus
21.715 ms. Sparse was close to clean. No-op optimization is principally core
optimization; faster mutation cannot erase its full scan.

Every candidate below inherits these gates unless explicitly changing contract:

- Compare serial baseline and changed implementation in the same release build,
  interleave repeated observations, retain outliers, report census, filesystem,
  cache conditions, wall/user/system time and resource high-water marks.
- Separate tracing/instrumented attribution from timing. Count *actual* syscalls,
  allocation bytes, FD high-water and physical jobs where the hypothesis requires
  them. Existing logical progress counts cannot substitute for these.
- Preserve native names, literal links, pre-epoch/exact mtimes, unsupported-kind
  behavior, sorted error selection and committed-prefix/prune semantics unless a
  candidate explicitly declares the changed guarantee. Exercise failures, not
  only successful equality.
- For sync: clean repeat zero writes; existing destination hardlink aliases
  untouched; old leaf survives failed preparation; restrictive modes and cleanup
  failures retain their meaning; joined work and truthful partial progress.
- A **win** is a repeatable relevant-workload gain beyond observed run variability
  or a meaningful measured resource reduction with acceptable latency. **Reject**
  a candidate with no such signal, a shifted cost that negates end-to-end benefit,
  or an unmet correctness/budget obligation. No numerical gain is assumed.
- Mutation fixtures belong to a newly owned `.test-agent/optimization-<builder>/`
  directory with a README. Both supplied live paths remain strictly read-only.
  Builders record a fresh census if they use live read-only inputs.

Each entry gives mechanism/evidence, applicability, contract/dependencies,
smallest experiment, and its additional success/rejection discriminator.
Complexity is qualitative: **small**, **medium**, **large**.

# A — Inspection and representation (six candidates)

## A01 — Construct error paths only on failure

**Source/evidence:** [`pair.rs`](/src/pair.rs), `inspect_entry` lines 104–118,
unconditionally computes `raw.entry.path()` before a successful relative stat.
**Mechanism:** construct it inside metadata/mtime error closures; create it for
readlink and unsupported-kind cases only. Native error paths remain identical.
**Workload:** all successful regular files/directories, especially long paths.
**Contract/dependencies:** none; portable std, independent of FD work. **Small.**
**Experiment:** isolated patch, allocation-counted clean shallow/deep trees and
normal serial/P4 scans. **Win/reject:** fewer path allocations with unchanged
syscalls/errors and measurable CPU/resource value; reject elaborate ownership
machinery if simple lazy construction captures the benefit.

## A02 — Classify with file_type before fetching irrelevant attributes

**Source/evidence:** `pair::inspect_entry` stats directories even though `Kind`
stores no directory metadata. Rust's Linux `file_type` uses known `d_type`, with
metadata fallback for unknown types. **Mechanism:** classify directory/symlink/
special first; fetch size/mtime only for regular files, read links as required.
**Workload:** directory-heavy and symlink-heavy clean trees. **Contract:** it may
stop discovering errors from redundant stats; specify whether observable errors
are “required inspections fail” rather than “every entry stat must succeed.”
Unknown d_type must work, and a portable fallback must avoid unnecessarily
statting a regular file twice. Existing stable-tree assumption is relevant but
does not imply stat permission errors never differ. **Small–medium.**
**Experiment:** clean directory-heavy fixture, known/unknown-type backend tests,
unreadable/special entries. **Win/reject:** syscall reduction with declared error
semantics and no regression on all-regular/unknown-type cases. Independent A01.

## A03 — Pair names before loading keys; skip irrelevant readlinks and stats

**Source/evidence:** `entries` inspects all raw entries before `compare` decides
one-sided/type-change categories. One-sided link contents and file size/mtime
cannot affect classification. **Mechanism:** name-group the sorted raw stream,
classify both sides, then obtain detailed keys only for same-kind paired leaves.
Regular↔symlink and dir↔leaf need type validation, not key comparison.
**Workload:** fresh/dense deltas and type churn; less on clean regular files.
**Contract/dependencies:** shares A02's required-inspection error decision;
unsupported special kinds must still fail. Preserve original-before-target
error precedence within each name; do not allow target classification to outrank
an earlier required original inspection. **Medium.**
**Experiment:** paired/unpaired links, dangling links, type transitions, inaccessible
entries; syscall-count a 90%-one-sided fixture. **Win/reject:** keys only read when
they can affect results, with explicit failure parity/deviation. Combine with A06
only after standalone attribution.

## A04 — Name storage without per-entry duplicate ownership

**Source/evidence:** `RawEntry` stores both `DirEntry` and `file_name()`; Rust Linux
DirEntry already owns a CString. **Mechanism:** an optional native listing backend
uses one name allocation (possibly a packed arena with offsets) shared by sorting,
metadata lookup and output extraction. std does not expose a borrowed basename,
so sorting std entries by repeatedly calling `file_name()` would worsen matters.
**Workload:** wide directories; allocation-heavy small entries. **Contract:** raw
native bytes, NUL termination and platform order must survive; lifetime/arena
complexity buys nothing if kernel work dominates. Depends on F01 or a custom
listing layer. **Large.** **Experiment:** listing/sort-only allocation profile and
full compare on one wide fixture. **Win/reject:** lower bytes/CPU end-to-end, not
just a faster synthetic string sort; reject a second parallel name representation.

## A05 — Compact relative path identity, materialize output paths late

**Source/evidence:** `frontier::reserve` clones job paths, `ParallelWalk::next`
copies them into inspection, and `DirDiff.dir` repeats node/frame paths; serial
`examine` clones the diff path for its frame. **Mechanism:** move owned paths where
possible first; then compare `Arc<Path>` or parent/name handles, materializing
only at OS boundaries and output/summary. Serial descent can use push/pop scratch.
**Workload:** deep trees and tiny directories. **Contract:** task scratch cannot be
shared mutably; borrowed scratch cannot outlive a joined ownership boundary.
Keep summary paths stable after arena reclamation. **Medium**, large for path DAG.
**Experiment:** depth sweep with allocation/path-byte counters, A01 held fixed.
**Win/reject:** fewer bytes/copied components without increased per-operation
path reconstruction or long-lived parent chains. F01 may make a path DAG useful;
do not build it just to avoid short PathBuf copies.

## A06 — Compare the sorted union directly, without split-and-remerge

**Source/evidence:** `entries` sorts `(name,side)`, constructs two `Vec<Entry>` in
`split_entries`, then `compare` merge-walks them. **Mechanism:** adjacent-name
groups in the sorted union directly produce children/categories; transfer names
only when retained. Retain a fallible ordered inspection phase if needed for
parallel metadata results. **Workload:** every directory; wide ones most clearly.
**Contract:** no need to change output/error order; preserve all-listings-before-
metadata precedence. **Small–medium**, composes with A03.
**Experiment:** count allocations/capacity and compare exact records/errors across
wide/mostly-clean trees. **Win/reject:** intermediate buffers actually disappear;
reject if a replacement group buffer has comparable cost or complicates error order.

# F — Descriptor-native filesystem backend (four candidates)

## F01 — Carry directory handles for descent and link reads

**Source/evidence:** `Roots::inspect` full-path opens both directories; readlink
uses `fs::read_link(path)`. Existing entry stats are already relative on this
Linux GNU toolchain. **Mechanism:** open children with parent dirfd, enumerate
through owned handles, use `readlinkat`; retain relative printable identities
separately. Scope parent handles only as long as jobs/results need them.
**Workload:** deep paths, symlink-heavy trees, long root prefixes, repeated descent.
**Contract:** optional Unix/Linux backend; portability fallback remains. Avoid
claiming that anchoring alone proves confinement—no-follow descent, `..`, mounts,
and root policy are separate. **Large.**
**Experiment:** fixed entry count, independently sweep depth and root-prefix length;
compare ordinary stats, lazy paths A01 and handle backend, trace lookup forms.
**Win/reject:** added relative opens/readlinks yield real gains after accounting for
FD lifetime; reject if saving only A01's allocations explains the result.

## F02 — FD-native sync chain from parent to installed leaf

**Source/evidence:** `apply_with`, `copy_leaf`, `prepare_file`, `remove`, and
`paths::relative` repeatedly join and resolve the same ancestors. **Mechanism:**
bind parent capabilities once, use relative open/stat/readlink/mkdir/unlink/rename;
keep output fd for timestamps/mode, source fd for cloning. Source metadata can
be obtained from the opened object. **Workload:** deep dense sync and recursive
copy/delete; more promising than merely rewriting diff's existing relative stats.
**Contract/dependencies:** platform backend, explicit ownership/FD budget; S03
is its smallest useful slice. Public record validation remains. **Large.**
**Experiment:** one regular replacement operation first, shallow/deep variants,
then siblings; verify hardlink and failure guarantees. **Win/reject:** fewer path
lookups/syscalls and lower operation latency; reject broad backend construction
before the leaf chain demonstrates value.

## F03 — Minimal statx mask, without silently weakening freshness

**Evidence:** pinned std requests `STATX_BASIC_STATS | STATX_BTIME`, more than
comparison needs. **Mechanism:** native backend requests type/size/mtime only;
check returned mask and retain exact SystemTime range/precision. **Workload:**
filesystems where extra attributes are costly, potentially remote/FUSE; local
btrfs benefit unproven. **Contract:** `AT_STATX_DONT_SYNC` is a distinct weaker-
freshness policy, not part of this same-semantics optimization. **Medium**, F01
can supply shared names/handles but is not required for a microbenchmark.
**Experiment:** identical fd-relative calls with full/minimal masks and existing
sync flags. **Win/reject:** kernel-visible reduction on the target filesystem;
reject if masks return identical cost, and never infer a win from smaller Rust structs.

## F04 — Larger/batched directory enumeration and scratch reuse

**Evidence:** std wraps libc readdir, owns each returned name and Arc reference;
current code creates fresh vectors per pair. **Mechanism:** first reuse modest
per-job listing capacity; separately test a native getdents buffer/packed names.
Do not assume one readdir call means one syscall: libc already buffers.
**Workload:** huge flat directories; repeated tiny directories may benefit from
capacity reuse but suffer retained worst-case buffers. **Contract:** Linux ABI
parsing, malformed-buffer checks, native names and deterministic listing errors;
worker-local scratch must tolerate Rayon reentrancy. **Medium–large.**
**Experiment:** syscall and allocator counts across 8/128/30k-entry directories.
**Win/reject:** fewer getdents calls or allocations without retaining each worker's
largest-ever buffer. Prefer explicit job scratch leases over implicit thread-local
mutable state held across nested Rayon work.

## Rayon compatibility and backend choice

Directory-relative syscalls are compatible with Rayon. They use explicit fd/name
pairs, not thread-local working directories. Never use process-wide `chdir` as a
shortcut. Owned `File`/`OwnedFd` can move into jobs; shared directory handles can
be reference-owned, with a borrow kept alive until each syscall finishes. Avoid
sharing an enumeration cursor concurrently; give an enumerator one owner, and
share only the underlying lookup handle with metadata tasks. A duplicated fd can
share an open-file-description offset, so `dup` alone is not independent directory
enumeration state.

Today Rust's retained DirEntries share a directory resource, so comparison does
not open one fd per entry. A new backend could nevertheless retain handles for
K completed results plus active ancestors rather than only running listings.
Specify FD high-water independently of J/K and test low `RLIMIT_NOFILE`; closing
and reopening everything may erase the proposed savings. Holding a descriptor
does not make file contents immutable or eliminate permitted caller mutations.

For the smallest Linux experiment, rustix is the natural syscall interface:
reflink-copy already uses its `ioctl_ficlone`, verified in local source. Adding it
directly still requires an explicit feature/dependency decision. cap-std is a
reasonable capability-oriented design candidate, but its particular version,
resolution behavior and syscall count have **not** been inspected here. Do not
assume its higher-level confinement behavior is free or required for this stable-
tree API. Compare abstractions after the useful operation sequence is established.

# Q — Scheduling and retention (seven candidates)

## Q01 — A measured throughput limits preset

**Evidence:** `ParallelLimits::for_threads` sets K=4P; measured P4/K16 saturates
while K256 improves the motivating tree. **Mechanism:** expose/document an opt-in
throughput preset or select a better default after breadth/memory measurements.
**Workload:** many-small-directory clean trees. **Contract:** no semantic change;
budgets add across walkers and K is not bytes. **Small**, independent.
**Experiment:** K16/64/256 on clean, dense, wide, multi-walker shared pool with RSS
and first-record latency. **Win/reject:** a defensible Pareto point; reject a
universal K256 claim based solely on one live topology.

## Q02 — Reserve a cursor-local lookahead window, not only one demand node

**Evidence:** existing demand reserve guarantees progress but can run one job at
a time while far-ahead results consume K. **Mechanism:** partition K into local
and global quotas; keep several soon-needed children admissible, reclassifying
slots as the cursor advances. No extra hidden slots. **Workload:** skew and
depth under small K. **Contract:** same total K/J and seals; avoids raising memory
just to improve occupancy. **Medium.**
**Experiment:** forced early slow branch/later fast branches at K8/16/64; count
idle-J intervals and compare to Q01. **Win/reject:** greater occupancy and speed
at fixed memory; reject if rigid reservation wastes parallelism on broad trees.

## Q03 — Compress only fully resolved clean speculative subtrees

**Evidence:** ready clean nodes consume K until commitment. **Mechanism:** collapse
a proven entirely clean closed subtree into a receipt carrying exact examined
count and DFS placement; commit that count only when cursor reaches it. Partial
clean nodes with pending children are not compressible. **Workload:** many clean
leaves/small subtrees ahead of cursor. **Contract:** K currently means directories,
not receipts: this changes its accounting unless represented-directory count
still charges K. State a replacement byte/work bound explicitly. **Large.**
**Experiment:** pure reducer model with early error/break plus filesystem control.
**Win/reject:** reduced retained memory or improved occupancy while exact prefix
counts survive; reject “one receipt means one directory” semantic accounting.

## Q04 — Shared-budget chunks inside exceptionally wide directories

**Evidence:** public frontier always calls sequential `Roots::inspect`; wide-root
control favors predecessor metadata parallelism. **Mechanism:** list and sort,
then admit indexed metadata chunks using available global work slots, returning
ordered results to that directory. No nested unaccounted fanout. **Workload:**
root-wide files, or one giant directory on critical path. **Contract:** specify
whether J counts all inspection tasks rather than only directories; K still
retains directory results. Keep list/error ordering. **Large.**
**Experiment:** one 30k-file directory plus a mixed tree, P1/2/4 and unrelated
pool work. **Win/reject:** recover wide speedup without regressing small-directory
throughput or exceeding the declared budget. Never block pool workers waiting
for permits held by their own children.

## Q05 — Reusable or indexed candidate search

**Evidence:** `Frontier::candidate` allocates a search Vec and traverses frames
and admitted nodes; `pump` may call it repeatedly while holding the mutex.
**Mechanism:** reuse scratch first; if proven hot, maintain a ready-owner index
ordered by DFS position, updating it on completion/commit/seal. **Workload:** very
small directories, large K and deep frames. **Contract:** frontier selection,
arena generations and seals must stay correct. **Small** scratch, **large** index.
**Experiment:** mocked zero-cost inspector measures scheduler overhead, then actual
tiny directories and lock-held duration. **Win/reject:** end-to-end CPU gain; reject
queue invariants if reusable scratch solves most of it. Q02/Q03 alter the index.

## Q06 — Execute short demand continuations inline

**Evidence:** every admitted directory uses scope.spawn and separate start/complete
lock acquisitions, even for J1/K0 or clean wrapper chains. **Mechanism:** execute
the preferred demand continuation on the completing worker, spawning siblings;
batch a bounded number of reducer steps without monopolizing the pool.
**Workload:** tiny/deep trees and demand-only use. **Contract:** joined return,
accurate running counts, stack safety and fairness; not an unbounded recursive
call chain. **Medium.** **Experiment:** J1/K0 and deep wrappers, plus a second
walker competing on P2. **Win/reject:** removes measured scheduling cost without
starving the sibling or growing stack with depth. Q04 needs compatible task units.

## Q07 — Byte-aware admission as a complementary limit

**Evidence:** K counts directories; wide running listings dominate memory beyond
retained-name diagnostics. **Mechanism:** account capacities/results by bytes and
pause new admissions under pressure, retain a demand reserve; bound active wide
work separately. This is a throughput-under-pressure optimization, not a promise
that K was incorrectly implemented. **Workload:** concurrent walks or huge listings.
**Contract:** a single required directory may exceed the target unless external
sorting/streaming is added; expose soft versus hard bounds honestly. **Large.**
**Experiment:** several walkers each hit wide dirs under a process memory limit.
**Win/reject:** avoids memory-induced collapse without deadlock or demand starvation;
reject if byte tracking materially penalizes ordinary trees without useful control.

# S — Synchronization kernel (ten candidates)

## S01 — Carry already obtained metadata through recursive operations

**Evidence:** `copy_directory` stats a child then `copy_leaf`/`copy_directory` stats
it again; `validate_entry` throws away kinds/metadata then operations re-read them.
**Mechanism:** internal prepared operation carries Metadata and kind into the
kernel. First optimize recursive private copy, then public validation output.
**Workload:** fresh and dense trees. **Contract:** existing stable-tree assumption;
retain a separate public validation boundary. Descriptor metadata can later
supersede stale path metadata in F02. **Small–medium.**
**Experiment:** count stats per copied file/directory and validate source mismatch
failure behavior. **Win/reject:** eliminate repeated same-operation inspections
without removing the pre-mutation whole-record validation guarantee.

## S02 — Fast successful staging cleanup

**Evidence:** `cleanup` always calls general recursive `Context::remove`; after a
successful rename the private staging directory is known empty. That path stats,
relaxes, enumerates, then removes it. **Mechanism:** directly `remove_dir` on known
empty success; retain general owned cleanup after preparation failure. Propagate
cleanup failure as before. **Workload:** every installed leaf. **Contract:** no
path namespace shared with user entries, no silent cleanup, no automatic external
pruning. **Small**, independent of copy policy.
**Experiment:** syscall trace a successful one-file and 1k-file copy, then inject
cleanup failure. **Win/reject:** empty-dir traversal disappears while errors and
installed progress remain truthful. This is a useful baseline before S04.

## S03 — Clone into an already owned output fd

**Evidence:** reflink dependency opens/stats source, creates output, FICLONEs,
chmods and closes; our code chmods it writable, reopens, sets time/readback/mode.
**Mechanism:** Linux clone backend opens source/output once, retains output fd,
calls FICLONE, finalizes metadata once. No chmod-to-reopen dance. **Workload:**
strict-CoW tiny files and restrictive modes. **Contract:** preserve required/fallback
policy and cleanup behavior; optional Linux fast path, portable dependency fallback.
**Medium**, useful without F02 or S04.
**Experiment:** isolate existing private staging strategy, compare syscall sequence
for 000/444/644 source modes and clone failure. **Win/reject:** fewer opens/chmods
and lower fresh/dense latency, same hardlink-safe replacement and exact readback.

## S04 — One private staging arena per touched target directory

**Evidence:** each leaf currently makes and removes its own temp directory.
**Mechanism:** create one private directory lazily per batch/parent; independent
names prepare unique children, finalize and rename them, then join and remove the
arena. Track ownership at the batch so cleanup failures preserve installed counts.
**Workload:** many changes in one directory, recursive initialization. **Contract:**
target-side/same-filesystem placement; no arena on clean scan; permissions restored
after cleanup; avoid source-name collision with a staging name in live listings.
**Medium**, combines S03, supersedes much of S02's benefit.
**Experiment:** 1/8/1000 changed siblings, including one failed preparation and
restrictive parent. **Win/reject:** mkdir/rmdir scale with touched parents, not
leaves, without broad cleanup of names not owned by the operation. Reject root-wide
arena reuse across mount boundaries or excessive synchronization contention.

## S05 — Direct preparation in private newly created subtrees

**Evidence:** recursive copy creates absent directories yet still stages every
leaf under them. **Mechanism:** when an entire subtree is itself private and
staged, its children can be created directly: the subtree is already the isolation
boundary. For visible absent-target initialization, direct leaf creation is a
separate weaker partial-failure choice; failed incomplete leaves need deliberate
cleanup and must not later masquerade as equal. **Workload:** leaf→directory
transitions or explicit whole-root staging. **Contract:** retain existing replacement
safety; private-subtree mode is narrower than “new means safe.” **Medium.**
**Experiment:** staged directory transition with nested copy failure and readonly
children. **Win/reject:** per-leaf staging eliminated while old leaf survives failed
preparation. Do not silently apply it to visible added directories without defining
their failure semantics. Complements S04 rather than requiring it.

## S06 — Reuse validated ancestors inside a trusted mirror session

**Evidence:** `paths::relative` stats root and every directory component on both
sides for every diff. Dense deep records repeat this ancestry work.
**Mechanism:** private mirror path uses a validated DFS ancestor stack/handles;
manual `Applier::apply` retains full checks. Invalidate stack after authorized
replacement/prune and scope its lifetime to the mirror call. **Workload:** many
records at depth. **Contract:** stable trees already required, but public arbitrary
record validation cannot simply disappear. **Medium**, C02/F02 can carry proofs.
**Experiment:** deep dense tree, compare ancestor stat counts; malformed manual
record and ancestor-link tests still fail. **Win/reject:** cost approaches newly
entered components rather than sum of full depths, without leaking trust to manual API.

## S07 — Vector category validation instead of BTreeMap then sort

**Evidence:** `paths::names` clones names into a BTreeMap, then sorts collected
entries by encoded bytes. Public records need not be sorted; valid core records
are. **Mechanism:** flatten borrowed `(name,mask)` entries, native-byte sort once,
coalesce and reject duplicate masks; or linear merge for trusted core categories.
**Workload:** wide divergent records. **Contract:** preserve malformed-record
rejection before mutation; cross-platform native ordering cannot be assumed to
equal BTreeMap's order. **Small–medium.**
**Experiment:** wide generated records, adversarial duplicates and type transitions.
**Win/reject:** lower allocations/CPU with identical validation, without adding
expensive sortedness checks that consume the fast path's gain.

## S08 — Reduce indexed per-item result storage

**Evidence:** `Context::batch` collects one `Result<Progress,Failure>` per sibling,
then merges; successful Progress values are additive. **Mechanism:** chunk ordered
siblings, accumulate progress locally, retain only failures with their order keys,
merge chunks in order. Tune grain to avoid scheduling tiny operations excessively.
**Workload:** extremely wide copies or many cheap removals. **Contract:** stop flag,
encountered-error ordering and truthful joined progress; no serial-prefix claim.
**Medium**, independent of core scheduling.
**Experiment:** high-width batch peak RSS and wall time with a forced early/late
failure. **Win/reject:** memory scales with chunks rather than each successful
result while error identity/progress stay valid. Reject oversized chunks that
destroy useful balance on mixed-size files.

## S09 — Parallel deletion below an opaque removed root

**Evidence:** `Context::remove` is serial within each named subtree even in pool
mode; one large removed node cannot use independent sibling workers.
**Mechanism:** recurse with structured sibling batches, remove parent after joins;
limit grain and preserve restoration after failures. **Workload:** one large removed
subtree, not just many separate removals. **Contract:** failure encountered-set may
change, as already in parallel copy; all started work joins. **Medium.**
**Experiment:** one wide and one many-small-dir removal, serial/P2/P4; track btrfs
contention and permissions. **Win/reject:** lower wall time without material CPU/
tail collapse; reject automatic parallel deletion if filesystem serialization
negates it. Exact removal paths still come from records, never a cleanup sweep.

## S10 — Explicit trusted timestamp-representability policy

**Evidence:** `preserve_file` reads metadata after each set_modified to reject
unrepresentable mtimes. **Mechanism:** retain exact readback by default; specialized
caller asserts a known source/target filesystem and timestamp domain, allowing
setter success to stand without per-file readback. A one-file capability probe
cannot establish support for every possible timestamp. **Workload:** tiny files on
controlled local FS. **Contract:** deliberately weaker per-leaf verification, not
a semantics-preserving removal. **Small** implementation, significant policy choice.
**Experiment:** known-range timestamps plus extreme/pre-epoch values, injected
rounding backend; compare saved syscall cost after S03. **Win/reject:** useful
residual cost and enforceable caller restriction. Reject generic “probe once,
therefore all mtimes exact” reasoning.

# C — Contract and workload forks (six candidates)

## C01 — Immutable donor catalog shared across destinations

**Evidence:** each `Roots::inspect` re-enumerates and re-keys original, and each
`mirror` is independent. jj-waltz's prepared donor can serve multiple workspaces.
**Mechanism:** a generation-scoped in-memory catalog of native sorted names, kinds,
file keys and literal links. Build once, compare each destination against it;
optionally retain changed-file metadata for application. Lazy per-directory
memoization reduces cold setup when targets have opaque missing subtrees.
**Workload:** one donor→N targets; small benefit or loss for N=1. **Contract:** donor
generation is immutable for the catalog lifetime; neither directory mtime nor
matching root stat validates that promise. Open handles pin objects, not contents.
**Large**, generic reusable API with an explicit snapshot/stability authority.
**Experiment:** N=1/2/4 destinations, count donor inspections and catalog bytes;
include build/free cost and many missing targets. **Win/reject:** amortized scan
reduction justifies memory; reject an unversioned process-global path cache.

## C02 — Prepared records / fused compare-apply session

**Evidence:** names-only DirDiff discards keys, root validation and source link
targets, while manual applier reconstructs and validates them. **Mechanism:** keep
DirDiff for reporting; offer an owned or lending prepared record with opaque root/
generation binding, metadata and optional handles. Or private `mirror` integrates
the kernels without publishing a new record type first. Caller decisions still
control prune; explicit ownership prevents using a prepared record with other roots.
**Workload:** dense sync; negligible clean application benefit. **Contract:** size,
lifetimes, stale record behavior and trust authority are now part of the API.
**Large** public API, **medium** internal prototype. Builds on S01/S06, competes
with simply doing cheap repeated stats.
**Experiment:** private mirror path only, exact observed-summary parity and syscall
counts for dense sync. **Win/reject:** material avoided work before exposing public
prepared types; reject forcing every reporting consumer to retain rich metadata.

## C03 — Read-only completion-oriented scan, ordered output optional

**Evidence:** divergence seals and joined epochs principally support mutable
consumer/prune decisions. A report-only scan does not need those decisions.
**Mechanism:** explicit read-only scanner can continue below differences and
process completions freely, optionally sort records at the end. A callback/fold
can avoid storing all records when order is not required. No async runtime needed.
**Workload:** dense audits, counting/reporting, gather-before-apply. **Contract:**
early break/error coverage is no longer the same committed DFS prefix unless an
ordered reducer is retained; deterministic final ordering requires storage and
deterministic error choice may require additional completion. **Large.**
**Experiment:** dense fixture with 4k differences, compare epoch count and complete
records; explicitly test partial coverage reporting. **Win/reject:** removes real
seal cost with acceptable memory/lifecycle. Reject changing default iterator
behavior merely because report mode is faster.

## C04 — Fused mirror with branch-local compare/apply continuation

**Evidence:** mirror alternates global joined comparison epochs and apply epochs;
one divergent record can stop useful comparison elsewhere.
**Mechanism:** fixed mirror policy permits branch-local compare→apply→descend,
parallel among disjoint branches; build final observed summary deterministically
after joins. Keep leaf-safe writes. **Workload:** dense differences spread across
many directories. **Contract:** no arbitrary mutable caller loop; failure is a
set of completed operations, not the old comparison DFS prefix. Root/ancestor
permission changes must be coordinated with descendants, and aliases excluded.
**Large**, alternative to C02 retaining present ordering; no generic task framework.
**Experiment:** narrow fixed-policy mirror on a two-level tree, failure in one
branch, repeat convergence. **Win/reject:** substantial dense gain over S01–S04;
reject if alternate failure/report semantics overwhelm the consumer benefit.

## C05 — Certified generation/change-set comparison

**Mechanism:** caller supplies a trustworthy changed-path set for a donor/target
generation; compare only affected branches, with full scan after lost history.
Could be a write-mediated workspace service or a loss-detecting journal/watch
protocol. Plain mtimes, watcher silence or a cached manifest are not certificates.
**Evidence:** current README correctly rejects directory-mtime subtree shortcuts;
there is no such authority in the implementation. **Workload:** repeated clean
reseed where full scanning dominates. **Contract:** new external completeness,
rename/delete, overflow, restart, hardlink and external-writer obligations.
**Large**, strongest asymptotic opportunity and strongest invalidation burden.
**Experiment:** an owned write-mediated fixture with explicit generation ledger,
compare selective versus full scans after writes/renames/deletes and lost events.
**Win/reject:** identical full-scan results plus safe fallback on lost authority;
reject a benchmark that excludes maintaining the certificate.

## C06 — Filesystem-level or immutable-package replacement profile

**Mechanism:** if payload roots become dedicated snapshot-capable subvolumes, use
snapshot/clone at root granularity; alternatively immutable package identities
permit replacing changed package directories instead of comparing every leaf.
Content hashes help only if they are already authoritative package identities
or amortized across versions/targets; computing all bytes every reseed may cost
more than today's metadata scan. **Evidence:** generic library has no subvolume
layout or immutable package authority; ordinary FICLONE is per-file, not a tree
snapshot. **Workload:** controlled jj-waltz provisioning, dense package replacement.
**Contract:** different storage layout, deletion/mount behavior, retained target
edits and reported granularity; explicit whole-root policy, not an automatic prune.
**Large**, outside generic backend implementation.
**Experiment:** owned representative payload organized under the proposed storage
unit, include provisioning and post-reseed consumer checks; compare incremental
scan+delta versus root/package replacement. **Win/reject:** end-to-end reseed benefit
with private CoW and intentional target preservation semantics. Reject “snapshot
is O(1)” as a substitute for measuring actual layout and cleanup costs.

# X — Cross-cutting discriminators (seven candidates)

## X01 — Lightweight summary policy

**Evidence:** `Summary` always retains full `changed_dirs` and `topmost_divergent`
paths. **Mechanism:** optional counts/coverage-only accumulation for embedders that
do not need path aggregates; retain existing default. **Workload:** dense large
trees, many changed ancestors. **Contract:** explicitly different result type or
available fields, not silently empty lists. **Small–medium.** **Experiment:** dense
deep tree output memory; win if retained path cost matters, reject on clean-only
evidence. Complements A05/C03 but independent of execution backend.

## X02 — Overlap the two directory listings

**Evidence:** `entries` lists original then target before any metadata. **Mechanism:**
under shared global budget list both sides concurrently and retain errors until
original-side precedence can be decided. **Workload:** high-latency directories,
wide roots; local tiny directories may lose to scheduling. **Contract:** joining
still required; target may be inspected despite original error, as speculative
physical work. **Medium.** **Experiment:** latency-injected enumeration and local
control; win only when overlap beats overhead, reject unbudgeted nested Rayon.
Q04 may provide shared task accounting.

## X03 — Use platform enumeration metadata where it is already returned

**Mechanism:** backend policy can use metadata-rich enumeration on platforms
that provide required keys, rather than forcing a Linux-shaped stat sequence.
**Evidence:** only Linux GNU behavior was verified here; Windows/macOS API and
precision must be researched before implementation. **Workload:** those platforms,
especially remote directories. **Contract:** no-follow kind, exact mtime and field
availability must match; never infer Linux benefit. **Large research/backend.**
**Experiment:** syscall/API trace and precision parity on one actually available
target platform; reject any untested “portable fast backend” claim.

## X04 — Avoid repeated known-unsupported reflink attempts in fallback mode

**Evidence:** `reflink_or_copy` tries clone per file. **Mechanism:** within a stable
source/target filesystem pair, remember only narrowly justified unsupported-clone
outcomes and copy later files; never cache permissions, ENOSPC or per-file failures
as global capability. Required mode must still fail honestly. **Workload:** many
files on non-clone FS, not the current strict btrfs use. **Contract:** mount/file
attributes can affect support; coarse device-ID caching may be wrong. **Medium.**
**Experiment:** owned unsupported FS plus mixed supported/unsupported cases;
win if failed ioctl/setup costs disappear with correct per-pair scoping; reject
heuristics that suppress a required clone or hide a genuine error. Independent S03.

## X05 — Size-aware copy strategy using already known regular-file length

**Evidence:** std io::copy probes source/output metadata and loops until EOF;
kernel-copy code itself notes small-file overhead. **Mechanism:** experimental
trusted regular-file backend uses known size, tries direct bounded kernel copy
or a small buffered copy, handling short reads/writes and fallback offsets.
Do not assume BufferedReader automatically disables all std specialization.
**Workload:** tiny files versus large/sparse files. **Contract:** a stable source
length requirement, kernel bug workarounds, actual logical byte accounting;
Copy still makes no physical-allocation promise. **Medium–large.**
**Experiment:** zero/4KiB/large/sparse matrix, trace io::copy first, compare exact
bytes and time; reject replacing robust std fallback behavior for a marginal
tiny-file gain. Strict reflink is a separate policy, not a size heuristic.

## X06 — Release-build tuning as an orthogonal control

**Mechanism:** compare existing release with thin LTO/codegen-unit settings;
optionally host-specific target CPU for local embedding, never silently for
portable distribution. **Evidence:** existing measurements used default release,
no profile override. **Workload:** CPU-heavy sort/reducer portions. **Contract:**
build portability and size; no algorithmic change. **Small.** **Experiment:** same
corpus, binary size/build settings recorded, user/system split. **Win/reject:**
repeatable end-to-end benefit, not only a microbenchmark. Run after attribution
so compiler flags do not conceal which algorithm eliminated work.

## X07 — Pool-locality and multi-walker admission policy

**Evidence:** all walkers share P workers but independently own J/K; prior data
measures primarily one walk. **Mechanism:** consumer coordinates concurrent root/
workspace walks, using a shared budget or limiting outer concurrency; compare
per-target traversal to C01 batched donor fanout. **Workload:** jj-waltz reseeding
several workspaces while the pool serves other work. **Contract:** application
policy, not a mandatory new global scheduler. **Medium experiment**, potentially
small integration. **Experiment:** one/two/four concurrent target scans, include
unrelated short pool tasks and FD/RSS peaks. **Win/reject:** aggregate throughput
and fairness improve; reject isolated speedups that hurt actual group completion.

# Independently commissionable Sol HIGH briefs

These are bounded builder experiments, not authorization to merge every theory.
Each builder should own a separate fixture and report, keep baseline callable,
record exact changes and preserved/changed semantics, and return negative results
as useful evidence. No production live-tree mutations. Architectural forks need
selection after their experimental return.

## E1 — Pure inspection waste, portable first

**Scope:** A01 and A06 as independently switchable changes; A02/A03 in a separate
variant that explicitly states required-inspection error semantics. **Inputs:**
existing serial/frontier, native-name/error tests; small/deep/wide and one-sided
fixtures. **Return:** allocation and syscall deltas, exact parity or documented
error-contract delta, release timings, recommendation per change. **Gate:** no
scheduler/API redesign and no conflating fewer keys with weakened special-kind
validation. Establishes baseline before any FD backend.

## E2 — Strict-CoW leaf operation anatomy

**Scope:** S02 then S03, separately measured; keep current staging and readback.
**Inputs:** pinned reflink-copy source, sync hardlink/restrictive-mode behavior;
owned nonempty tiny/large files. **Return:** syscall sequence for one successful
copy/clone, failed clone, mode000 output; fresh/dense timings. **Gate:** no fallback
under Required, no destination hardlink writes, no old-leaf loss before preparation,
same progress/cleanup guarantees. **Decision:** whether direct descriptor cloning
earns an optional Linux implementation independent of a full FD traversal backend.

## E3 — Staging granularity and metadata reuse

**Scope:** S01/S04, control current clone/copy backend so E2 is independent.
**Inputs:** 1/8/1000 changed siblings; nested added tree and one forced failure.
**Return:** mkdir/rmdir/stat/open counts, cleanup correctness, batch memory, timing
by files-per-parent. **Gate:** lazy arena, zero writes on no-op, same filesystem,
every temporary name owned, permission restoration after joined cleanup.
**Decision:** per-parent arena versus current per-leaf privacy.

## E4 — K occupancy without bigger hidden budgets

**Scope:** Q02 versus Q01 controls; Q05 scratch reuse may be a separately reported
micro-change, no clean-subtree compression yet. **Inputs:** pure frontier mock
with forced completion orders plus small/skew/deep filesystem shapes.
**Return:** same total K/J, occupancy time, first yield, peak nodes/bytes, parity
across early break/error/prune. **Gate:** demand progresses at K0/P1 and no worker
waits for unavailable permits. **Decision:** default/preset tuning versus a more
local admission rule; preserve measured K256 as a control.

## E5 — Wide-dir shared task accounting prototype

**Scope:** Q04 only, with a clear revised J definition; no unbounded nested
par_iter beneath directory jobs. **Inputs:** wide root and mixed topology,
P1/2/4, a competing walker. **Return:** actual task/IO concurrency, ordering/error
tests, user/system/wall and memory. **Gate:** deterministically ordered errors,
bounded total admissions, no deadlock when a parent needs child work.
**Decision:** whether a hybrid earns complexity versus keeping a specialized
wide-directory backend option.

## E6 — Donor-generation prototype, not a persistent cache

**Scope:** C01 in-memory read-only catalog, one explicitly stable owned donor,
N1/2/4 targets; no watchers, disk persistence or invalidation guessing.
**Inputs:** source repeated-work counts, small dirs and missing subtrees.
**Return:** build-inclusive amortization curve, exact per-target records, bytes
retained and source inspections eliminated. **Gate:** object lifetime explicitly
binds the donor generation; no claim that root metadata detects donor changes.
**Decision:** is a multi-destination API a worthwhile primary-consumer profile?

## E7 — Descriptor lookup hypothesis isolation

**Scope:** F01 read-only mini-backend, after reproducing pinned std behavior with
a tiny trace; no production applier rewrite. **Inputs:** equal entry counts across
depth/prefix-length and link-density sweeps. **Return:** statx/readlink/open forms,
FD peak and low-limit behavior, timings with A01 baseline, portability boundary.
**Gate:** no chdir, no concurrent directory cursor, native links and deterministic
errors. **Decision:** full handle descent versus just readlinkat and lazy paths.

# Dependencies, conflicts and recommended sequence

E1, E2, E3, E4 and E6 can proceed independently in isolated branches; they touch
overlapping conceptual areas but their baselines permit attribution. E7 should
use E1's A01 result as a control before claiming path-resolution benefits. E5's
task accounting deserves a separate review before combining with Q02.

- A03/A06 are naturally composable, but compare individual patches first.
- S03 and S04 multiply useful savings; S04 reduces the importance of S02.
- S01 is a cheap precursor to C02; F02 is not a prerequisite to proving S03.
- Q02/Q03/Q05 all change frontier structure; combining them in one build would
  obscure both performance attribution and invariant review.
- C03 and C04 deliberately relax different ordering/mutation boundaries. They
  should not be disguised as internal optimizations of the existing iterator.
- C01 and C05 have different trust scopes: one freezes the donor for reuse;
  the other certifies changes across generations, including target changes.
- C06 can dominate a controlled storage layout but cannot be generalized to
  arbitrary directory roots. Measure consumer obligations, not just copy calls.

First select small portable wins and syscall anatomy. Then choose whether the
next larger payoff is wide-directory support, FD-native application or reusable
donor knowledge. The evidence currently favors avoiding repeated work over
introducing a new async/runtime abstraction.

# Cross-references and return to parent

- [Accepted frontier design](/design/concurrency/design0.gpt6am.md): predecessor
  rationale. Q02–Q07 refine its performance tradeoffs; C03/C04 intentionally
  reconsider parts of its public lifecycle instead of silently violating them.
- [Implemented frontier](/design/implementation/implementation1.sol61x.md): the
  invariant baseline for all scheduler experiments, particularly exact prefix,
  seals and count budgets.
- [Frontier measurements](/design/performance/parallel1.sol61x.md): positive
  single-root evidence and negative root-wide/default-K evidence. No new speedup
  claimed by this inventory.
- [Sync brief](/design/sync/brief0.gpt6a.md) and
  [sync implementation](/design/sync/implementation0.sol61x.md): first-build
  selection and operation guarantees. New trusted/fused APIs are new proposals,
  not claims that the existing public manual applier should stop validating.
- [Sync measurements](/design/performance/sync0.sol61x.md): fresh versus sparse
  cost distinction. Pinned std source above adds an interpretation constraint:
  ordinary Copy may already use kernel CoW, so policy labels are not physical-I/O
  measurements.

Suggested index entry (shared index deliberately left for parent integration):

> [Independent Astra optimization inventory](performance/opportunities0.gpt6am.md):
> forty candidates spanning inspection work, descriptor-native operations,
> scheduling, staging and donor-generation contracts; verified Rust 1.88/reflink
> implementation details, explicit tradeoffs and seven bounded builder experiments.

**Verification of this document:** source inspection and version-pinned upstream
reads only; no benchmark campaign or implementation checks claimed. External Rust
source was fetched after checking the installed toolchain/archive for local
source; reflink source came from the local Cargo cache. Shared index and sibling
same-wave documents were not edited.
