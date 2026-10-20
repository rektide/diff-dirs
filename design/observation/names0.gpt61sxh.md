---
type: ResearchEvidence
title: Borrowed native names, packed batches and explicit promotion
description: Source-backed lifetime and ownership design, with a bounded allocation and syscall discriminator for native listing, union, exclusion, metadata and retained child names.
resource: /design/observation/names0.gpt61sxh.md
status: proposed-with-bounded-evidence
generated: { by: "model:openai/gpt-6.1-sol#xhigh", at: 2026-10-20 }
sources:
  - resource: "opencode://ses_ea7d44a59ffe0dMHN5QqJ3oVuh"
    title: Independently investigated name lifetimes and storage, built an isolated allocator/syscall probe and compared safe arena storage with retained backing and owned child promotion
  - resource: /design/roadmap/brief0.gpt6a.md
    title: Scoped independent research boundary and human constraints
  - resource: /design/observation/facts0.gpt6am.md
    revision: 5423a71f
    title: Completed prerequisite defining unchanged fact demand and required name retention boundaries
  - resource: /doc/architecture/README.md
    revision: a63595a9
    title: Prior-wave proposed inspection, ownership and continuation contracts
  - resource: /design/execution/state0.gpt6a.md
    revision: a63595a9
    title: Established listing, public-plan and pending-child lifetimes
  - resource: /design/performance/inspection0.sol61x.md#e1
    title: Historical distinction between eager userspace paths and already-fd-relative metadata
  - resource: /src/listing.rs
    revision: a63595a9
    title: Native per-entry owned names, clone accessor, shared parent and RawDir reuse
  - resource: /src/visitor.rs
    revision: a63595a9
    title: Union ordering, logical paths, preservation, eager diagnostic paths and child-name cloning
  - resource: /src/pair.rs
    revision: a63595a9
    title: Child lowering before release of clean plans
  - resource: /src/walk.rs
    revision: a63595a9
    title: Owned pending names and containing coordinates in traversal frames
  - resource: "file:///home/rektide/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rustix-1.1.5/src/fs/raw_dir.rs"
    title: Exact dependency's lending next, alignment, initialized-byte handling and kernel parsing assumptions
  - resource: "file:///home/rektide/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/linux-raw-sys-0.12.1/src/x86_64/general.rs#L955"
    title: Native linux_dirent64 field layout, not an owning path representation
  - resource: "file:///home/rektide/.rustup/toolchains/nightly-2026-10-14-x86_64-unknown-linux-gnu/share/doc/rust/html/src/std/sys/fs/unix.rs.html#1023"
    title: Configured std's owned public name operation and private borrowed accessor
  - resource: "file:///home/rektide/.rustup/toolchains/nightly-2026-10-14-x86_64-unknown-linux-gnu/share/doc/rust/html/src/std/sys/helpers/small_c_string.rs.html#22"
    title: Configured std's stack C-string conversion below 384 bytes
  - resource: "file:///home/rektide/src/diff-dirs/.test-agent/name-storage/README.md"
    title: Owned ignored probe, fixture, modes, counter definitions and reproduction instructions; discriminating results preserved below
extensions:
  ticket: diffdir-tree-observe-names
  reading_boundary: Scoped brief and ticket, completed fact-demand prerequisite, named prior architecture/state/performance evidence and implementation only; withheld statement, full roadmap, parent transcript and concurrent other-topic drafts not consulted
verification:
  method: Exact native core inspection plus isolated common-shape representation experiments, System allocator census, marker-bounded strace, safe-view boundary checks and expected E0499 lifetime rejection
  result: Names/order/presence/exclusions and one retained non-UTF8 child agreed; all working modes issued the same 72 statx calls and six getdents64 calls; scratch build, clippy and formatting passed; no production edits or full-suite/large/live-workspace experiments
---

# Borrow names safely; own the batch; promote the survivors

**Use safe native name views and batch-relative offsets, not an owning-path
transmute or a new native parser.** For the present public API, first remove the
duplicate native handle/key name ownership and unnecessary successful-path joins.
For a new narrow inspection surface, a packed owned name arena plus borrowed
component coordinates is a credible next representation. Promote sparse pending
children into their own owned storage when releasing the batch.

The bounded experiment does **not** support “an arena always saves memory.” With
eager paths unchanged, it reduced allocation events by 29% but increased cumulative
requested bytes by 6%. Adding split logical coordinates and lazy diagnostic paths
reduced events by 94%, requested bytes by 15% and peak live bytes by 19% in the
**reduced common-shape probe**, not in a production replacement. Keeping the arena
for one 200-byte child retained 31,744 bytes instead of 200.

## 1. Native views already exist; their lifetime is the obstacle

On Unix, this conversion is safe, lossless and allocation-free:

```rust
use std::{ffi::OsStr, os::unix::ffi::OsStrExt, path::Path};
let name: &OsStr = OsStr::from_bytes(entry.file_name().to_bytes());
let path: &Path = Path::new(name);
let component = path.components().next();
```

No UTF-8 conversion or layout reinterpretation is involved. The probe checked
identical backing addresses for the CStr bytes, OsStr and Path, including `ff 80`
inside actual names. Once `.` and `..` are removed, a real basename is a Normal
component. Untrusted/restored names must additionally reject empty names, NUL,
separators, absolute paths and dot components.

`RawDir::next(&mut self) -> Option<Result<RawDirEntry<'_>>>` lends from its own
mutable borrow, not for the full buffer lifetime. Even advancing to the **next
record in the same buffer** while retaining that name is rejected by the API.
The probe's intentional compile failure produced E0499 at the second `next()`
when the first borrowed OsStr was used afterwards. The buffer can be overwritten
at refill; dropping RawDir does not grant ownership of its borrowed names.

Thus safe borrowing is sufficient for inspecting a single record, copying a name
into an arena, and borrowing names from a **frozen owned arena** during sort and
policy. It is insufficient by itself for collect/sort across arbitrary directory
order, two-side union, whole-batch early callbacks, parallel admitted acquisition,
public owned plans or pending traversal after enumeration ends.

The default std backend has a different boundary: public
`DirEntry::file_name() -> OsString` allocates. The configured Unix implementation
has a private `file_name_os_str() -> &OsStr`, but it is not a public escape hatch.
Do not depend on std-private layout/accessors. Other-platform enumeration may
still benefit from internal arena/path changes; the Linux copy counts below are
not cross-platform measurements.

## 2. Where current ownership pays

Source coordinates refer to `a63595a9`; the inspected production files remained
identical during this study.

| Boundary | Current source-derived cost / reason |
| --- | --- |
| Native listing, `listing.rs:68–100` | One 8 KiB reusable RawDir buffer and one shared parent per side. Every listed name is copied into its Handle's OsString. Each entry clones an Arc, not a directory descriptor. |
| Union input, `visitor.rs:1004–1025` | `Entry::file_name()` clones that OsString again; sorting compares native encoded bytes and then side, using unstable in-place sorting. Sorting itself need not allocate. |
| Union identity, `visitor.rs:1031–1061` | One key survives per union name. Equal-side duplicate keys are dropped. `dir.join(name)` constructs a full logical relative PathBuf for every union name **before** exclusion. |
| Exclusion, `visitor.rs:1081–1100` | Excluded handles drop without child metadata. Preserved basenames and the first excluded relative coordinate remain as boundary evidence. This is already delivered work avoidance. |
| Acquisition, `visitor.rs:754–760` | `entry.path()` eagerly constructs a physical child path, although Linux metadata is already basename/parent-fd-relative. Non-links need it only on an error. Links may need a physical readlink path. |
| Finalization, `visitor.rs:838–900` | Validation eagerly forms a logical target path for every admitted entry; baseline differences clone names into public DirDiff vectors. These costs are not eliminated merely by changing RawDir. |
| Child lowering, `visitor.rs:569–575`; `pair.rs:92–103`; `walk.rs:109–114,181–182` | Children clone basenames into owned pending state; clean metadata/plans can then die. Frame advancement later joins a containing coordinate and child. |

For `N` present listing entries, `U` union names, `A` admitted present sides and
`C` retained children, current Linux native name storage has **N name copies + N
merge-key clones**, then **U logical joins + A physical joins**, plus finalization,
diff and child projection. These are source-derived operation counts, not a claim
that every join is exactly one allocator call. The probe found substantial join
reallocation too.

The fixture has `N=96`, `U=64`, `A=72`, `C=1`; listed name bytes total 17,906.
Removing the native handle/key duplication alone can remove **96 OsString clone
allocations requesting 17,906 bytes**, without narrowing any metadata demand.
Union names contain 11,506 bytes; a NUL-terminated arena for **both listings** uses
18,002 initialized bytes before capacity slack. It still contains both copies of
the 32 matched names until explicitly compacted.

## 3. Bounded experiment and actual results

### Setup, instrumentation and equivalence

The separate crate is `.test-agent/name-storage/`, with its own `[workspace]` and
target directory. It does not replace an existing demo or change core code.
The fixture contains 48 entries per side, 32 matched names and 16 names unique to
each side: **96 empty entries / zero file-content bytes**, plus the two containing
directories. One matched entry is a directory; the other entries are empty files.
There are lengths 1, 3, 7, 8, 15, 16, 200 and 255, and two non-UTF8 union names.
The retained child is a 200-byte non-UTF8 name. Sixteen union names are preserved
early; 36 entries on each side remain for the same full no-follow metadata demand.

All working modes assert exact native names and order against a preconstructed
golden union, true source/target presence masks and exclusion decisions. The reduced
variants also check listing inode against acquired metadata and expected file kind.
Every mode verifies the retained child after all batch state is dropped. Core
late checks compare admitted names/presence and the public child projection yields
that same single child. Successful full drops restore the allocator's live baseline.

The global allocator delegates to System. **Events = successful alloc/alloc_zeroed
+ realloc calls**, including in-place realloc. **Requested** sums every newly
requested allocation/reallocation size, not bytes copied. **Peak/live** count live
requested Layout sizes, not RSS, allocator usable size or physical memory. During
realloc, accounting replaces the old layout with the new one; allocator-internal
temporary overlap is not measured. Startup, fixture/golden construction and output
formatting are outside the window; stack snapshots and successful checks allocate
nothing. Single-threaded native core inspection is used. The no-work control
recorded **zero events, bytes, live bytes and filesystem syscalls**.

Two evidence classes must not be subtracted from one another:

1. **Actual core** calls native `inspect_directory` with an early preserving visitor,
   retains full facts and public reporting, clones the source-equivalent child
   projection through public accessors, then drops the plan.
2. **Reduced common-shape variants** share their own RawDir → sorted union → early
   → same full metadata → child pipeline. They omit baseline comparison,
   finalization, public reporting, parallelism and sync. Metadata slots are inline
   in this reduced layout even before acquisition, unlike production. These isolate
   storage changes, not an implemented production optimization.

### Total allocation census

All bytes below are exact counter results for the recorded run.

| Mode | alloc | realloc | Events | Requested bytes | Peak live bytes | Retained after batch drop/promotion |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| No work | 0 | 0 | 0 | 0 | 0 | 0 |
| **Actual native core** | 424 | 380 | 804 | 280,734 | 117,030 | 200 |
| Reduced `owned` | 339 | 275 | 614 | 170,234 | 100,462 | 200 |
| Reduced `arena-compat` | 148 | 286 | 434 | 181,142 | 94,090 | 200 |
| Reduced `arena-view` | 12 | 24 | 36 | 143,916 | 81,828 | 200 |
| Reduced `arena-pin` | 11 | 24 | 35 | 143,716 | 81,828 | 31,744 |

`arena-compat` changes only the reduced name storage/key type: copied names become
one growable Vec of NUL-terminated bytes and checked `{start:u32,len:u32}` IDs.
It retains eager logical and diagnostic paths and the same per-entry parent Arc.
Against reduced owned: **180 fewer events (29.3%), 6,372 fewer peak bytes (6.3%),
but 10,908 more requested bytes (6.4%)**. Arena growth requests and unused retained
capacity matter. An arena-only allocation-count win is not a byte-volume win.

`arena-view` additionally represents logical coordinates as borrowed
`{containing:&Path, name:&OsStr}`, iterating their chained components, and creates
physical diagnostic paths only on error. It does **not** pretend those discontiguous
parts are a `&Path`. Against reduced owned: **578 fewer events (94.1%), 26,318 fewer
requested bytes (15.5%), and 18,634 fewer peak bytes (18.5%)**. Against arena-compat,
these path changes remove 398 events and 37,226 requested bytes. The larger event
gain therefore cannot be credited to arena layout alone.

### Lifetime checkpoints

| Checkpoint: live bytes | Reduced owned | Arena with existing paths | Arena with views |
| --- | ---: | ---: | ---: |
| Both sides collected, before sort | 46,216 | 38,052 | 38,052 |
| Union constructed | 90,222 | 87,946 | 75,684 |
| Early exclusions release handles/non-first excluded paths | 82,839 | 84,972 | 75,684 |
| Full admitted metadata retained; listing handles released | 69,178 | 84,808 | 75,520 |
| All slots dropped; arena still owned if applicable | 200 | 31,744 | 31,744 |
| Child name promoted; arena released | 200 | 200 | 200 |

Before slot release the owned mode has already cloned its child. Arena modes can
keep just a child ID while the arena lives, release slots, then copy the child.
`arena-pin` instead moves the whole arena to the pending child: **158.7 times the
child's byte length**. This is a lower bound for Arc-sharing that same buffer,
because it does not add an Arc control allocation or retain the slot tables.
Copying the sparse survivor is the useful optimization, not a failure of borrowing.
Many surviving siblings can instead justify one **compact child-only** arena;
that option was not benchmarked and needs no universal threshold in this design.

### Real syscall measurements: storage saved no observation I/O

Between explicit stderr markers, **actual core and every reduced working mode**
each issued exactly:

```text
2 openat(directory), 6 getdents64, 72 statx(no-follow basename relative), 2 close
0 newfstatat, 0 readlink/readlinkat, 0 child opens
```

Each side exercised refill: source reads returned **8,104 + 1,344 + 0 bytes**;
target returned **8,168 + 2,688 + 0 bytes**, each with an 8,192-byte buffer. These
are actual traced syscall byte counts, including dot entries, not guessed record
sizes. The 72 stats agree with the independent source census: 96 listed sides less
24 excluded sides. No sparse-fact design, d_type stat-elision or symlink-demand
change was smuggled into the representation comparison.

The executable records one instrumented elapsed sample per mode. Those samples
are not speed evidence; no percentage timing claim, large tree, benchmark matrix,
live JJ/workspace operation or production/core patch was performed.

## 4. Retained raw batches and zerocopy: possible, not the next dependency

The exact rustix 1.1.5 source (`raw_dir.rs:128–140,194–239`) trims its mutable
MaybeUninit buffer for native alignment, tracks the initialized byte count from
getdents, advances by `d_reclen`, and borrows the NUL-terminated name. Its unsafe
implementation assumes the kernel supplies complete, aligned records and a NUL.
Its cookie accessor exposes a seek cookie; it does **not** promise a durable
continuation coordinate.

On the inspected x86_64 Linux ABI, the fields are inode at byte 0, offset/cookie
at 8, `u16 d_reclen` at 16, type at 18 and variable name at 19. Records advance by
`d_reclen`, not by `size_of::<header>()`; padding is not name data. This layout is
source-derived and not used by the probe to parse a record. RawDir and std already
perform the native parsing we need.

A retained-batch backend could stop **after draining each buffer and before its
next refill**, freeze that backing allocation, and enumerate into a fresh one.
Entries would store `{chunk, name_range}` and copied hints, never owning raw
pointers. Borrowed views are then tied to an immutable chunk owner. RawDir's
current API does not export a frozen initialized buffer/range abstraction; a
local adapter would need a carefully justified projection of validated initialized
name ranges from its MaybeUninit backing (or a separately owned initialized input
and bounded parser). It must not expose padding/uninitialized tails as initialized
bytes merely because some names were returned. Dropping an iterator with unread
records would lose those records when resuming from the descriptor's already
advanced offset.

The tradeoff here is unpromising enough not to manufacture that unsafe adapter:
the four observed data batches would retain **32,768 buffer bytes** at the current
capacity, versus an 18,002-byte packed payload / 31,744-byte actual arena capacity.
Freezing buffers avoids the 17,906 name-byte copy, but adds chunk indexing, keeps
native headers/tails, and must still sort, merge and promote sparse children.
Retaining one ordinary full 8 KiB chunk for the child is already 41 times its
name bytes. These are source/geometry calculations, **not measured raw-retention
allocations or a proof it cannot win on other workloads**. A parser is not needed
to demonstrate the safe arena's current allocation advantage.

If a later copy-bandwidth profile justifies raw retention, prefer an upstream
owned/frozen-batch API or a narrow adapter over changing the traversal API into
raw structs. A custom parser must check initialized prefix, header bounds,
nonzero/in-range reclen, overflow/progress, a NUL **within that record**, name
component validity and platform ABI; it must not form aligned native references
from arbitrary serialized bytes. Fixed-width fields can be decoded safely from
byte slices; a zerocopy header helper would not solve the variable-length tail,
kernel lifetime, arena ownership or cursor validation. No zerocopy dependency or
custom native parser is recommended on this evidence.

## 5. Smallest useful representation and public change surface

### Compatibility-first: private cleanup, no public borrowed lifetime

1. **One owner per native listed name.** Separate the metadata parent/hints token
   from name storage. Sort/merge by a borrowed name, move the chosen name into the
   union, and pass that borrowed union name to basename-relative acquisition.
   An Entry/Handle's `metadata` must no longer require its own cloned OsString.
   Std can retain its normal DirEntry token and one public file_name result.
2. **Lazy physical diagnostic paths.** Borrow the entry/token through metadata;
   construct its physical child path in the error branch or when a link read
   actually needs it. Keep logical final and physical staging coordinates separate.
   Likewise move success-only validation path construction behind a real refusal.
   Error order and blame coordinates remain unchanged; do not drop the errors.
3. **Keep owned public names/relative paths initially.** `Name`, `Operation`,
   DirectoryPlan and DirDiff can retain today's interfaces and ownership. Child
   lowering still produces owned names; do not pin a whole listing to save that
   necessary sparse-survivor copy.

The private change surface is `src/listing.rs` and union/acquisition helpers in
`src/visitor.rs`. Child lowering is already a sensible ownership boundary; no
`walk.rs`, cursor codec, sync fact contract or public `Facts` change is required
for these initial cleanup opportunities. This report implements none of them.

### Packed narrow inspector: one growing owner, offset records, borrowed views

For the [demand-shaped inspector proposal](facts0.gpt6am.md#3-smallest-demandacquisition-contract),
the useful internal batch shape is:

```text
Batch owns native name bytes + entry/union records + per-side parent handles
Record = name range + true side presence + hints/admission/fact references
View<'batch> borrows Batch and a Record; name() → &OsStr
logical coordinate = containing coordinate + normal native component
lowering → selected owned work / compact pending-child names / boundary evidence
```

Mutate/grow only while retaining offsets, never references across a Vec growth.
Freeze before constructing borrowed views or dispatching parallel readers.
Sort offsets/indices by the corresponding name bytes and side, using the current
duplicate-on-one-side refusal and source-first acquisition order. Presence and
excluded coverage are separate fields. Do not deduplicate by inode or Unicode,
normalize names, or use a name's Path ordering instead of native byte ordering.

The probe uses checked u32 offsets solely as a compact bounded experiment. A
production batch must choose `usize` or chunked IDs, or report overflow; never
silently impose a 4 GiB directory-name ceiling or truncate. Safe slice range
checks and `CStr::from_bytes_with_nul` suffice for its NUL-terminated arena.
NUL storage is convenient for direct native APIs but not necessary for OsStr.
The unchanged std Dir metadata path copies these short names to a stack C-string
buffer on this toolchain (threshold 384), not a heap allocation; a CStr-backed
syscall adapter would remove a stack copy, **not** the 72 stat syscalls.

**Public implications are real.** Today's `Names::entries() -> &[Name]` and
`Name::relative_path() -> &Path` cannot be implemented by simply putting a byte
Vec beside offset-only Name records: a standalone `&Name` has no access to its
owner, and split parent/name storage is not a contiguous Path. Options are:

- Keep owned compatibility payloads at early/late/publication boundaries. This
  preserves APIs but pays their materialization; do not claim arena-view totals.
- Add batch-borrowed `NameView<'_>` / operation views and an explicit component
  coordinate interface. Callers may inspect during callbacks but must own/promote
  anything retained. Entries/entries_mut are view iterators rather than slices of
  owner-independent Name values; wrappers can still edit only admission/action.
- Put an immutable shared arena handle plus ranges inside each public record.
  This can keep self-borrowing accessors without public lifetime parameters.
  Store complete relative spellings in a packed path arena if `&Path` must stay;
  a basename can then be a subrange. Account for Arc/freeze construction and batch
  retention, especially when just one record escapes. This was not implemented or
  measured by the external-store arena probe.

Do not make a self-referential Vec of `&OsStr`, invent a `'static` lifetime or
transmute variable dirent records into PathBuf. PathBuf/OsString own allocator
storage; a native record has a header, borrowed name/tail and no compatible
ownership/capacity/drop contract.

## 6. Lossless continuation, not native stream continuation

At a supported pause, drop runtime dirents, parent handles and refill buffers.
Retain **owned native names and logical coordinates**, either individually or as
owned compact bytes with validated offsets, plus only the driver's required
pending phase/presence/finish data. Reconstruct runtime resources on resume.
Do not serialize borrowed addresses, Arc internals, inode-as-name identity,
RawDir offsets or `d_off` seek cookies as durable progress.

For Unix, encode raw OsStr bytes; for Windows, use an explicitly tagged native
code-unit representation. `to_string_lossy` and an unspecified portable encoding
of `as_encoded_bytes()` do not meet the lossless contract. A native-format cursor
can instead declare its platform/encoding compatibility requirement. Validate
arena ownership, offset arithmetic/ranges, names, ordering/duplicates, side masks
and parent routing before reconstituting components under rebound roots. IDs are
batch-local coordinates, not globally meaningful names. Buffer validity grants
neither fresh observations nor mutation authority.

This preserves the [prior architecture's pause boundary](/doc/architecture/README.md#component-built-continuation-and-pause)
and the [prerequisite's fact-retention distinction](facts0.gpt6am.md#4-ownership-and-retention):
borrow while inspecting; own only what must survive; reacquire facts only at an
explicitly declared boundary. No persisted pointer or durable-cookie assumption
is needed to retain sortable traversal state.

## 7. Provenance and reproduction

Recorded 2026-10-20, Linux `7.1.0-debplus.1`, x86_64 GNU, Btrfs; strace 6.13.
Configured rustc `1.101.0-nightly`, commit
`822182db4c4a1827256881d8c7aa685f0bcf2b11` (2026-10-13), LLVM 23.1.3;
Cargo `1.101.0-nightly (29c5daa1a 2026-10-07)`. The isolated lock selected rustix
1.1.5, linux-raw-sys 0.12.1 and bitflags 2.13.2. Native std Dir needs the existing
`dirfd` nightly feature. No toolchain pin was added. The archive checkout of
rustix was also checked for location/version (1.1.4); consequential claims use
the **actual registry dependency 1.1.5**, not that archive's moving checkout.

Run from `/home/rektide/src/diff-dirs`; relative fixture path length is part of the
allocation census:

```sh
cargo build --offline --release --manifest-path .test-agent/name-storage/Cargo.toml
# Fresh fixture only; skip setup when reusing the recorded fixture.
.test-agent/name-storage/target/release/name-storage-probe setup .test-agent/name-storage/fixture
.test-agent/name-storage/target/release/name-storage-probe boundaries
.test-agent/name-storage/target/release/name-storage-probe arena-compat .test-agent/name-storage/fixture
strace -qq -o .test-agent/name-storage/recheck.trace \
  -e trace=openat,getdents64,statx,newfstatat,readlink,readlinkat,close,write \
  .test-agent/name-storage/target/release/name-storage-probe arena-view .test-agent/name-storage/fixture
```

Setup refuses any existing root. Use `control`, `core`, `owned`, `arena-compat`,
`arena-view`, `arena-pin` for
the table's modes. Build output remains within the scratch crate's target.
The scratch README defines the windows and `lifetime-fail.rs` intentionally fails
against that build's rustix artifacts with **E0499**, not a runtime crash.
`results.txt`, per-mode traces, `trace-summary.txt`, `provenance.txt` and
`SHA256SUMS` preserve the local run; this document preserves the durable findings.

Exact SHA-256 identities of the discriminating probe artifacts:

```text
a6b55dac3835ed326cefb869501a4f0b24e20e508909e0bc75b0e5556b99abc4  src/main.rs
9338efac50b46a23079773b0ac119fc8d9943cbc073e0d9ab34f3793b353add9  Cargo.toml
4cc14746ee7e18d32a43e36862e37666e1d93df883df4146a62b7067208aaad8  Cargo.lock
a05b7f124450d35ba29b8826af3660e68fab89a8640f9c62816f42b9c83525c0  target/release/name-storage-probe
9eaeff016b58fa06600dac37bd9fb515afd3e8345ec9465292b42ded59b652d0  results.txt
631ce989453011766a702dc8eda07c233386d7ca8bf7f278eb7436bc0bd937e6  trace-summary.txt
```

Scratch release build, `cargo clippy --offline --manifest-path
.test-agent/name-storage/Cargo.toml --all-targets -- -D warnings`, and scratch
format check passed. Boundary checks covered malformed/missing NUL and ranges,
u32/usize overflow, invalid components, non-UTF8 names, lengths adjacent to native
alignment boundaries and child ownership after drop. No custom parser was added,
so these are **safe storage/view boundary checks**, not fuzzing of Linux dirents.
Production file identity, document source/link/whitespace checks and owned-path
commit are separate from functional optimization verification. Parent owns any
integration, index changes, ticket closure or export.
