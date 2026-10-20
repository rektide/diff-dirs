---
type: OpportunityInventory
title: Additional opportunities — native dirents and Btrfs checksum evidence
description: Post-native-dirent tuning candidates and optional filesystem checksum/change-token research, including the compressed-extent blocker, current-kernel limits and decision experiments.
resource: /design/performance/opportunity-also.md
status: research-follow-ups-open-not-implemented
generated: { by: "model:openai/gpt-6.1-sol#high", at: 2026-10-20 }
sources:
  - resource: "opencode://ses_ea82ab643ffel2vtvMoX2vqoZa"
    title: Human requested tuning without replacing size+mtime, commissioned FX and Luna xhigh, reviewed kernel/code evidence, and requested these tickets and opportunity notes
  - resource: "opencode://ses_ea82a8a5fffeuawvnWEKbf4HSd"
    title: FX located native-dirent implementation and tuning candidates; parent checked the current code and corrected the historical wide-directory parallelism claim
  - resource: "opencode://ses_ea82a6464ffeQxkz3CYiEaFfTO"
    title: Luna xhigh recovered hidden cross-project Btrfs research, reported an unsupported-kernel ioctl probe, and verified generation/writeback caveats on follow-up
  - resource: /src/listing.rs
    title: Native cached type/inode, shared directory handle, deferred metadata and bounded 8 KiB enumeration buffer
  - resource: /src/visitor.rs
    title: Eager paths, link observation, size+mtime comparison and existing per-entry parallel observation
  - resource: /src/lib.rs
    title: Current metadata-only comparison contract and nightly native feature
  - resource: /cli/Cargo.toml
    title: CLI does not enable native-dirent automatically
  - resource: /perf/operation/Cargo.toml
    title: Operation harness enables parallel dependencies, not native-dirent automatically
  - resource: /design/execution/state0.gpt6a.md
    title: Delivered early-hint capabilities, bounded buffer correction and normalized listing-only syscall evidence
  - resource: /design/performance/opportunities0.sol61x.md
    title: Existing U01, F01, F02, M01 and scheduling opportunity catalog; historical results are not current measurements
  - resource: /design/observation/facts0.gpt6am.md
    revision: 5423a71f
    title: Completed fact-demand research separates kind-only inspection from source attributes still required by copying
  - resource: file:///home/rektide/src/btr-churn/.design/churn/recon0.linux.gpt56t.md
    title: Prior hidden research distinguishes snapshots, root checkpoints, inode identity and incomplete tree-search/find-new hints
  - resource: https://github.com/torvalds/linux/blob/c03c1a14507dfaf8b856c2608f9ac216021fc222/include/uapi/linux/btrfs.h
    title: Locally inspected GET_CSUMS UAPI and range result flags in installed Linux 7.3 source
  - resource: https://github.com/torvalds/linux/blob/c03c1a14507dfaf8b856c2608f9ac216021fc222/fs/btrfs/ioctl.c
    title: Locally inspected checksum payload eligibility, aligned requests, ordered-range waits and checkpoint generation access
  - resource: https://github.com/torvalds/linux/blob/c03c1a14507dfaf8b856c2608f9ac216021fc222/fs/btrfs/ordered-data.c
    title: Locally inspected ordered-range wait starts writeback before waiting for completion
  - resource: https://github.com/kdave/btrfs-progs/blob/6797ce7600556138081382441bbc6104f35736e2/Documentation/Feature-by-version.rst
    title: Local archive lists unprivileged GET_CSUMS under Linux 7.2
  - resource: https://github.com/kdave/btrfs-progs/blob/6797ce7600556138081382441bbc6104f35736e2/Documentation/btrfs-ioctl.rst
    title: Local archive describes GET_CSUMS as raw checksums for a file range
extensions:
  tickets: [diffdir-tree-observe-dirent-tuning, diffdir-tree-observe-btrfs-csums]
verification:
  scope: Source review and host filesystem inspection; Luna-reported unsupported-kernel probe; no production changes or new performance measurements
  limitation: Installed kernel source is newer than the running kernel; source capability is not runtime capability
---

# Additional opportunities after native dirent support

The user accepts **size plus exact mtime** for ordinary regular-file comparison.
That is the baseline to optimize, not a defect requiring content hashes. Btrfs
checksums remain a worthwhile **separate optional research path**. These notes
record opportunities and their limits; they do not select an implementation,
change defaults or constrain later designs to the existing public fact shape.

Work anchors, both open under `diffdir-tree-observe`:

- `diffdir-tree-observe-dirent-tuning`: bounded tuning and metadata-work reduction.
- `diffdir-tree-observe-btrfs-csums`: checksum/identity feasibility, with compressed
  files as a primary discriminator rather than an afterthought.

## Native dirent tuning

### Current facts

[Native listing](/src/listing.rs) already obtains cached `d_type` and `d_ino`
without a hidden stat fallback. Unknown type remains Unknown. Entries share a
parent directory descriptor; admitted metadata is descriptor-relative. Linux
dirents do **not** contain size or mtime, so they cannot alone replace paired
regular-file metadata acquisition.

[Current inspection](/src/visitor.rs) still acquires full metadata for each
admitted present side. Regular-file equality remains size plus exact mtime;
symlinks compare literal native link text. A size mismatch short-circuits mtime
extraction, not the metadata syscall that already supplied both fields.

### Candidates and qualifications

| Candidate | Saved work / experiment | Constraints and limitations |
| --- | --- | --- |
| Explicit native feature activation | Make control/candidate executable features unambiguous. CLI and operation-harness manifests do not automatically enable native-dirent. | Native currently needs nightly `dirfd`; ordinary builds otherwise measure the std backend. Feature availability is not proof of a speedup. |
| Larger enumeration buffer | Compare current 8 KiB with 32/64 KiB on wide directories; investigate bounded per-worker reuse separately. | Current EINVAL growth stops at 128 KiB. Preserve termination, offsets and EINTR behavior. Larger/reused buffers trade allocation/syscall savings for retained memory; tiny-directory controls matter. |
| Lazy diagnostic paths | Avoid the unconditional successful-path `entry.path()` join in `listed_facts`. | Keep useful paths and original causes for errors. Full-path allocation and descriptor-relative kernel metadata lookup are different costs. |
| Descriptor-relative symlink reads | Use the retained parent with `readlinkat` instead of full-path `fs::read_link`. | Keep literal native link spelling, no-follow behavior, platform fallback and descriptor lifetime. Parent-renaming/race behavior must be stated if it changes. |
| Demand-shaped metadata | Use cached known kind for kind-only cases; keep paired-file size/mtime acquisition and explicit Unknown escalation. | Full-fact visitors/checks currently receive `Metadata`. Sparse facts change payload/error behavior unless compatibility explicitly requests full facts. Copying still requires source permissions/mtime/size; do not count a deferred required stat as eliminated. |
| Observation thresholds | Current per-entry parallel observation starts at 64 admitted entries with Rayon minimum length 8. Compare small/wide workloads under the actual shared budget. | Parallel observation already exists. The older inventory's root-wide serial result is historical, not evidence that today's code lacks it. Avoid another unbudgeted parallel layer. |
| Narrow `statx` requests | Explore TYPE/SIZE/MTIME only if a different fact representation is selected. | `std::fs::Metadata` has no public constructor. Fewer requested fields may cost the same on local Btrfs; returned masks and synchronization semantics matter. |

The [completed fact-demand study](/design/observation/facts0.gpt6am.md#2-practical-demand-table)
is the more precise continuation of **M01**: matched directories and known-kind
target leaves can be cheaper, but installing source-only regular files still
requires source attributes. Its private acquisition cache avoids paying twice
when Unknown kind was already resolved by metadata. The older
[opportunity inventory](opportunities0.sol61x.md) supplies **U01/F01/F02/M01**
experiment context; it is not a second implementation backlog to duplicate.

First discriminating controls: wide versus tiny directories; paired regular-file
trees versus directory/link-heavy and one-sided shapes; deep prefixes; forced
Unknown kinds; full visitor/check demand; selected sync work needing attributes.
Record syscall/allocation counts separately from latency and compiler/features
with every executable. The earlier listing-only trace proves omitted child I/O
under early preservation, **not a measured scan-time win**.

## Btrfs checksums and change tokens

### The promising interface is real

`BTRFS_IOC_GET_CSUMS` reads raw checksums for logical file ranges. The local
btrfs-progs feature table lists it under **Linux 7.2** and describes it as
unprivileged. The inspected Linux **7.3** source declares
`btrfs_ioctl_get_csums_args` and tagged range results in
`include/uapi/linux/btrfs.h:1103–1133`; implementation is at
`fs/btrfs/ioctl.c:5252–5535`.

The actual host runs **7.1.0-debplus.1**. Luna's read-only query returned
**ENOTTY**: installed newer headers/source do not make the ioctl available in the
running kernel. Runtime capability detection remains necessary even after a
version check. The host filesystem covering `~/src` is Btrfs with
`compress=zstd:3`, **CRC32C** checksums and 4096-byte sectors. This describes the
mount/filesystem, not proof that every individual extent is compressed.

### Compressed extents are the major blocker here

**The inspected implementation returns a COMPRESSED range tag but no checksum
payload for compressed extents.** Inline extents also provide no checksum payload,
including compressed-inline data. This is explicit branching in
`fs/btrfs/ioctl.c:5439–5465`, not a failure to discover the correct flag.

That substantially weakens this ioctl for the actual Zstd workload. A future
kernel upgrade alone does not fix the inspected API's coverage. Do not suggest
disabling compression as an assumed acceptable workaround, or quietly turn a
checksum path into byte-hashing most small/compressed files and call that a win.

The reason the research remains open: Btrfs does checksum stored compressed
data, but those storage checksums are not automatically a canonical hash of
decompressed file bytes. A newer API, careful encoded-extent comparison or a
filesystem-specific proof might help. Each needs an actual supported retrieval
path and a proof of its meaning; **none was established here**. Different
compression, extent boundaries, offsets or rewrites can represent the same
logical bytes differently. A representation mismatch must not automatically be
reported as a content mismatch in a byte-equality profile.

### Capability and cost matrix

| Surface / case | What is available | Limitation / required handling |
| --- | --- | --- |
| Ordinary checksummed uncompressed extent | HAS_CSUMS plus per-sector values | Not a single file hash. Bind algorithm, sector size, file length, ordered coverage and the declared comparison meaning. CRC32C is collision-prone, not a cryptographic identity. |
| Compressed or encrypted extent | Range classification, no checksum payload in inspected code | Unknown for checksum comparison; investigate another sound path or account for byte-reading fallback. |
| Inline extent | INLINE tag, possibly with compression/encryption tags | No checksum payload; significant for small-file workloads. |
| NODATASUM | Explicit NODATASUM result | No data checksum evidence. Never manufacture equality from absent checksums. |
| Hole or preallocated range | ZEROED classification | No stored data-checksum vector. Normalize logical zeros correctly if claiming byte equality; a hole and allocated zeros need not have the same representation. |
| Partial final sector / empty file | API requires a nonzero, sector-aligned range | Define rounded coverage and exact logical length; explicitly handle empty files. A checksum of padding is not by itself a canonical hash of the logical tail. |
| Large ranges / output limits | Bounded result buffer with updated offset/remaining length | Parse tags, validate contiguous requested coverage and paginate. One ioctl is not guaranteed to cover a whole file. |
| Dirty file data | Ioctl calls `btrfs_wait_ordered_range` twice | That helper starts writeback and waits. It can flush dirty pages and block; not passive, free metadata access. No concurrent-mutation consistency promise is established here. |
| Access | Inspected ioctl requires a readable regular-file descriptor, without a CAP_SYS_ADMIN gate | Adds file-open/ioctl/close and tree work. Directory listing/stat permissions do not imply file-read access. Compare fallback/error behavior deliberately. |
| Reflinks/shared extents | Potentially shared stored data | Shared physical identity is not a portable file hash; account for filesystem, exact ranges, length, mutable state and reuse. FIEMAP flags/extent changes need a separate soundness study. |

Possible profiles must stay distinct:

1. **Existing metadata comparison:** size + mtime, no file-data reads. Keep it.
2. **Filesystem-checksum comparison:** explicit algorithm/collision and eligible-
   range assumptions; incomplete coverage returns Unknown, not Equal.
3. **Canonical logical-byte digest:** handles every logical byte and file length;
   filesystem checksums do not magically supply such a digest. Byte hashing can
   be the fallback, but its full I/O/cost must be visible.

For a logical-content profile, checksum/representation disagreement may require
fallback rather than justify Different; matching checksums are also only as
strong as their collision model. A separately named representation-sensitive
predicate can make a different promise. Neither should silently replace today's
meaning of `different`, which also cares about mtime.

### Identity and transaction values are not live change counters

An inode number/generation identifies an object or reuse epoch, not unchanged
bytes. Btrfs identity additionally needs filesystem/subvolume scope. Extent
addresses can change on defragmentation or CoW and can eventually be reused;
address equality without a stable lifetime/range proof is not sufficient.

`BTRFS_IOC_GET_SUBVOL_INFO.generation` is a **root checkpoint** value. Unchanged
generation cannot skip a scan of a live writable subvolume: pending writes may
not have advanced it. `ctransid` was not established as a fence for all pending
changes either. Reusing results for the same immutable read-only snapshot is a
different, potentially valid optimization keyed by identity and checkpoint.
Syncing and then allowing writers to continue is not an immutable checkpoint.

`TREE_SEARCH` requires CAP_SYS_ADMIN and its reported leaf transaction ID is not
each item's modification sequence. `find-new` is incomplete for deletions and
metadata-only changes. The recovered
[Btrfs change-stream study](file:///home/rektide/src/btr-churn/.design/churn/recon0.linux.gpt56t.md)
explains these limits and the distinct read-only-snapshot/incremental-send path.

### What would make the checksum idea worth selecting?

- Verify the running/new candidate kernel's real API, not only source/header
  availability. Record kernel/source revision and filesystem checksum settings.
- Make **compressed and compressed-inline fixtures first-class**, then ordinary
  inline/uncompressed, mixed extents, NODATASUM, holes/preallocation, reflinks,
  empty files and partial tails. Do not extrapolate a large-uncompressed-file win.
- Establish retrieval and semantics for compressed data if possible, including
  different compression/extent layouts with identical logical content. If no
  supported path exists, state that blocker plainly and identify what upstream
  change would be required; this ticket does not authorize kernel development.
- Define Equal/Different/Unknown and fallback before integration. Include same-
  size/preserved-mtime edits to distinguish an optional content profile from the
  accepted metadata baseline. Unsupported evidence must never become equality.
- Measure eligibility/fallback rate as well as ioctl, metadata, bytes read,
  writeback and latency. Compare with the current zero-content-read scan and an
  explicit byte-hash control under the same stable/owned fixture conditions.
- Accept a negative feasibility result for this workload. A useful future
  capability is worth preserving without claiming the current kernel/API solves it.

## Connections and standing

- [Execution/state evidence](/design/execution/state0.gpt6a.md#enumeration-can-already-provide-facts)
  establishes the delivered native seam; this note adds follow-up choices, not a
  replacement implementation report.
- [Demand-shaped facts](/design/observation/facts0.gpt6am.md) is the existing
  acquisition/retention design. A future optional checksum is another explicitly
  requested observation, not an excuse to hash every entry.
- Earlier hidden research was recovered across `~/src`, including
  `watchment/.design/file-watching0.gpt5.md`; Luna also found the speculative
  `facet-caminus/doc/btrfs.md`. The later Btrfs study above supplies the checked
  limitations; the older find-new idea is not complete change detection.
- [Design index](/design/index.md#performance-opportunities--research-not-implementation-selection)
  connects the broader inventories. Tickets hold lifecycle/selection; this file
  holds the reasoning and constraints. No production tuning, hash backend or
  new default was implemented in this session.
