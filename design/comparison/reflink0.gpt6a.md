---
type: Design
title: Optional donor-sharing observation and proof-qualified retention
description: Conservative Linux Btrfs extent comparison, process-local evidence, and an opt-in ensure-sharing policy for ordinary reconciliation and jj-waltz tracked manifests.
resource: /design/comparison/reflink0.gpt6a.md
status: proposed-design-only
generated: { by: "model:openai/gpt-6-astra#medium", at: "2026-10-30" }
sources:
  - resource: "opencode://ses_e74ed67f6ffeB43O854roApY4C"
    title: Astra medium researches FIEMAP and current provider/native boundaries and designs optional proof-qualified retention without implementation or benchmarking
  - resource: "opencode://ses_e74f4db73ffebUUrvE3MbpCHWY"
    title: Parent commissions an optional already-reflinked comparison design to reduce inode churn, allowing increased comparison cost
  - resource: "file:///home/rektide/src/jj-waltz/design/coordinated-refill/policy-scale0.gpt6a.md"
    revision: 5f7b7e00
    title: Current full tracked renewal, incremental ordinary comparison and representative read-only workload census
  - resource: "file:///home/rektide/src/jj-waltz/design/coordinated-refill/evidence0.sol61x.md"
    title: Tiny final route measurements and exact installed-manifest adoption contract
  - resource: /doc/architecture/README.md
    title: Physical observation, pure selection, shared-source lifetime and effect ownership
  - resource: /design/architecture/execution0.gpt6a.md
    title: Explicit comparator identities, sparse facts, selected work and declared manifest contracts
  - resource: /design/architecture/annealing0.gpt6ax.md
    title: Prior donor-specific sharing question and unconditional renewal baseline
  - resource: /design/performance/opportunity-also.md
    title: Related filesystem-checksum research, compressed coverage gap and non-fencing generation values
  - resource: /tree/inspect/src/comparison.rs
    title: Actual Comparator, Comparison and StatKey implementation
  - resource: /tree/inspect/src/policy.rs
    title: Pure policy stages and separate assessment/action fields
  - resource: /transfer/sync/src/listed.rs
    title: CreateAbsent and RenewOwned intents, refusal-only guards and ManifestCoverage
  - resource: /transfer/sync/src/listed/selection.rs
    title: Exact owned authority and unconditional desired-leaf selection
  - resource: /transfer/sync/src/listed/runner.rs
    title: Complete requires installed equals desired and complete owned retirement
  - resource: /transfer/sync/src/kernel.rs
    title: Fresh private staging, required FICLONE and exact mtime/permissions preservation
  - resource: "file:///home/rektide/src/jj-waltz/refill/engine/src/engine/command/native/listed.rs"
    title: Tracked source expectations, required reflinks and complete-success adoption gate
  - resource: "file:///home/rektide/src/jj-waltz/refill/engine/src/engine/command/native.rs"
    title: Same-tree child, S installation, donor state adoption, optional D restore and final native verification
  - resource: "file:///home/rektide/src/jj-waltz/transfer/workspace/src/streaming.rs"
    title: Current ordinary source-qualified StatKey cohort and device guard
  - resource: "file:///home/rektide/archive/kdave/btrfs-progs/cmds/filesystem-du.c"
    revision: 6797ce7600556138081382441bbc6104f35736e2
    title: Local-first source inspection; FIEMAP-based space accounting skips unknown, delalloc and inline extents
  - resource: "https://docs.kernel.org/filesystems/fiemap.html"
    title: FIEMAP pagination, synchronization and extent flag ABI documentation, consulted October 30
  - resource: "https://github.com/torvalds/linux/blob/v7.1/fs/btrfs/fiemap.c"
    title: Versioned Btrfs mapping, compressed-offset omission, inline physical zero, backreference work and synchronization
  - resource: "https://github.com/torvalds/linux/blob/v6.17/fs/btrfs/extent_io.c"
    title: btrfs_do_readpage explains different logical ranges referencing different offsets of one compressed extent
  - resource: "file:///usr/src/linux-headers-7.1.0-debplus.1/include/uapi/linux/btrfs.h"
    title: Local FS_INFO and GET_SUBVOL_INFO identity fields; not a live content-generation fence
  - resource: "https://man7.org/linux/man-pages/man2/ioctl_fideduperange.2.html"
    title: Deduplication mutates sharing, compares bytes and has per-range completion and alignment limits
verification:
  method: Targeted code and kernel-source reading, read-only host mount/kernel identification, and explicit Markdown/OKF checking
  scope: Design only; no implemented probe, runtime FIEMAP fixture results, production changes or new timing evidence
---

# Recommendation

Add an **opt-in donor-sharing observation**, then use it in a distinct
**ensure-private-donor-sharing** policy. A supported positive observation can keep
an existing private receiver inode; inconclusive evidence selects the existing
required-reflink installer. Keep **always renew** as its own unchanged intent.

The useful first consumer slice is **tracked replacement with proof-qualified
retention**, built on a small independently testable observation/comparison seam.
An ordinary comparator alone cannot remove today's tracked churn. Clean ordinary
payload already stays in place under StatKey; strengthening that predicate can
actually cause more renewals. Read-only sharing diagnostics are a useful initial
provider checkpoint, not the completed consumer feature.

This buys avoided inode replacement, staging, rename and attribute writes on
eligible leaves even if the probe is slower overall. No speedup is claimed.
Compressed and inline files will initially fall back, materially limiting usefulness
on this machine. Review that tradeoff before authorizing implementation.

## Current facts and evidence boundary

- The [parent investigation](file:///home/rektide/src/jj-waltz/design/coordinated-refill/policy-scale0.gpt6a.md)
  records 8,935 tracked paths (~143 MB), and 201,221 ordinary regular files
  (~11.61 GB), 26,263 directories and 10,383 symlinks across 70 candidate roots.
  Candidate roots are not frozen production admission. Its 185,910 multiply-linked
  donor regular paths are neither receiver-private evidence nor sharing evidence.
- Both tracked routes currently install all S: `CreateAbsent` after clear, or
  `RenewOwned` with exact O authority. Both require reflinks; replacement preserves
  compatible directories but stages a new inode for each regular leaf.
- Ordinary node_modules/dist use size+mtime `StatKey`, independently of native
  route. Metadata equality is the declared heuristic, not byte equality or a
  storage certificate. Current equal ordinary leaves may be copies or hardlinks.
- In this session, `findmnt -T` identifies the checkout on Btrfs, root subvolume
  260, `compress=zstd:3`; `uname -r` returns `7.1.0-debplus.1`. These observations
  do not determine individual extent compression or prove FIEMAP behavior.
- Local archive discovery found btrfs-progs, but no full kernel checkout in the
  searched conventional locations. Its `filesystem-du.c` was read first. Upstream
  v7.1 FIEMAP code supplies the kernel evidence below; it is not a source attestation
  for the distribution's running build. No FIEMAP experiment was performed here.

The [tiny route evidence](file:///home/rektide/src/jj-waltz/design/coordinated-refill/evidence0.sol61x.md)
has seven tracked regulars totaling 104 bytes and 0–1 rounded ms listed installation.
It cannot predict this policy's latency or eligible fraction. No new full-scale
route timings exist in this commission.

## What the filesystem can establish

There is no portable read-only “was this cloned from that file?” API. Clone history
is unnecessary: deduplication or a snapshot may produce the same current sharing.
The desired observation is **same data backing at corresponding logical offsets**,
under a named stable-files assumption, with a separate private-inode check.

`FIEMAP_EXTENT_SHARED` alone is inadequate: it names no donor and no matching
logical offset. In Btrfs, the kernel computes it using backreferences. The current
kernel documentation's SHARED wording is awkward; the implementation makes clear
that it is returned extent information, not an instruction to clone. Two files
each shared with unrelated third files are a necessary negative fixture.

### Conservative initial Btrfs profile

Use `FS_IOC_FIEMAP` on opened regular files, with bounded paginated buffers. Limit
the positive backend initially to **the same mounted Btrfs instance and same
subvolume/device scope**, established from descriptor-associated mount identity,
`fstatfs` and `st_dev`, with checked no-follow routing. Do not compare bare addresses
across devices, mount namespaces or filesystems. A matching filesystem UUID alone
does not distinguish separately mounted cloned images.

Requiring the same mount ID and device scope is deliberately narrower than Btrfs's
ability to share across subvolumes. Cross-subvolume support needs explicit live
filesystem identity plus `(subvolume ID, inode)` object identity; inode numbers
alone can coincide across subvolumes. Local FS_INFO/GET_SUBVOL_INFO headers expose
useful fields, not a ready-made safe cross-mount proof. Do not weaken JW's existing
device-boundary guard as a side effect of this feature.

Within that accepted scope, ordinary uncompressed Btrfs FIEMAP `fe_physical` is
derived from `disk_bytenr + file_extent_offset`: a **Btrfs filesystem address**, not
a raw-device sector usable without chunk/RAID translation. Matching normalized
addresses at each logical offset is useful pairwise backing evidence even on a
multi-device Btrfs filesystem, without reading any underlying block device.

For equal lengths, merge the two ordered interval streams over `[0, size)`:

1. Validate lengths, checked additions, monotonic progress, supported flags and
   complete query coverage. A full buffer is not end-of-file. Advance by the last
   returned extent end; handle LAST, empty replies and the requested range correctly.
   A count-then-fetch allocation is neither necessary nor atomic.
2. Normalize segmentation: records need not have identical boundaries or counts.
   For each common data interval at logical offset `x`, compare
   `physical + (x - logical)` from both sides. Reject unsupported encodings before
   doing this arithmetic. Require SHARED on both data intervals as an additional
   conservative check, never as the pairwise proof by itself.
3. Account explicitly for holes. Equal holes at the same offsets satisfy the
   zero-data portion; they do not contribute “shared allocated bytes.” Hole versus
   allocated zero data fails this representation-sensitive predicate even if byte
   contents match. Partial sharing fails full satisfaction; report it diagnostically
   if collected, but do not patch individual ranges in this first slice.
4. Clip proof coverage to exact EOF. FIEMAP can report allocation past EOF; neither
   padding nor preallocation beyond EOF is part of this file-content promise.
   A normal uncompressed partial final block is acceptable when its mapping is
   known and covers the logical tail. Do not demand block-aligned file sizes.

Subject to quiescence, writeback and supported mappings, equal normalized backing
plus equal holes and length establishes the same logical data and full data-range
sharing. Different backing **does not establish different bytes**. This is a
sharing predicate, not a universal byte comparator, and not proof of historical
FICLONE use or exclusive physical ownership.

| Case | Initial result / reason |
| --- | --- |
| Uncompressed aligned mappings, all corresponding addresses equal, SHARED present | Pairwise data-sharing observation, conditional on the lifetime rules below. |
| Known differing mapping or hole layout | Not fully shared; bytes may still be identical. |
| ENCODED, DATA_ENCRYPTED | Unsupported for positive proof. No linear offset interpretation. |
| DATA_INLINE, DATA_TAIL, NOT_ALIGNED | Unsupported; metadata/tail packing is not ordinary data backing. |
| UNWRITTEN | Unsupported initially, even though reads give zeros; equal read semantics alone do not establish the chosen storage promise. |
| UNKNOWN, DELALLOC | Unresolved. Never interpret physical zero as shared data or silently skip the interval. |
| MERGED or unknown future flags | Unsupported in the initial allowlisted profile; only LAST and SHARED are accepted extent flags. |
| Empty files / matching all-hole files | Vacuous data satisfaction, separately labeled; zero shared allocated bytes, still require independent private inode and metadata. |
| Different size, same inode, receiver nlink > 1 | Unsatisfied before expensive extent probing. |
| Non-Btrfs, incompatible mount scope, absent capability | Unsupported; no generic physical-address fallback. |

**Compression is a concrete soundness gap, not merely caution.** In upstream v7.1
`extent_fiemap`, `extent_offset` is assigned only when compression is NONE. Encoded
records can therefore share `fe_physical`, logical position and length while
referencing different decompressed portions of the same extent. The v6.17
`btrfs_do_readpage` comments describe precisely such offset/length arrangements.
Inline FIEMAP emits physical zero. Comparing those tuples would admit false
positives. Do not infer compressed equality from having previously run FICLONE,
matching timestamps, or matching SHARED bits. A later filesystem-specific encoded
proof requires a supported offset-aware retrieval path; none is established here.

### Independent inode and metadata are separate obligations

For the narrow same-scope profile, require distinct `(st_dev, st_ino)` and receiver
`st_nlink == 1`. A donor may have many hardlinks. A destination hardlinked to a third
file is still unsafe to retain as a private receiver even when its donor differs.
Snapshot sharing is not hardlink aliasing; `nlink == 1` does not mean unshared data.
Nor does it prevent writes through already-open descriptors; that is quiescence.

For the first ensure-sharing slice require receiver length, exact supported mtime
and permissions matching the install contract, plus the expected tracked kind and
executable state. [The kernel](../../transfer/sync/src/kernel.rs) currently preserves
mtime and source permissions; it does not promise owner/xattrs/ACL identity.
JW's native content semantics require executable state, not every Unix mode bit,
but quietly accepting extra mode/mtime differences would weaken the current
installer-equivalence promise and complicate donor cache adoption.

Thus metadata mismatch initially renews even if sharing is proven. In-place chmod
or timestamp repair could avoid more churn later, but needs a distinct effect,
failure accounting and authority decision. It is not hidden inside comparison.
Symlinks have literal-target semantics, not data reflinks; retain them only under
a separately declared link-satisfaction rule. The minimal tracked slice continues
installing symlinks, so all proof-qualified retention counters initially concern
regular files only.

## Lifetime, dirty data and source-to-many reuse

**A positive result is an observation within a quiescent operation, not a durable
certificate.** Require the donor and receivers, their path bindings and relevant
storage mappings to stay stable from observation through retained acceptance and
native handoff. This includes external writers through donor hardlinks, mmap,
truncate, hole-punch, dedupe, defrag and relocation. This proposal adds no lock
manager; callers unable to supply that premise must renew or refuse the mode.

Open descriptors pin file objects, not contents or the names that reach them.
Descriptor-associated pre/post stats and final no-follow name-to-object checks
catch some races; compare identity, size, mtime, ctime, nlink and required mode.
These stamps must be freshly acquired from the open objects, not supplied from
the existing source-attribute cache as if they were rechecks.
Detected changes are failures, not a request to keep or blindly overwrite a newly
arrived object. Unchanged stats are not a concurrent-proof fence: pending data,
timestamp granularity and mapping relocation can escape them. Subvolume generation,
inode generation and native commit identity are not live per-write counters.

For positive live-data proof, select **FIEMAP_FLAG_SYNC** explicitly on both sides.
Unsynchronized mapping can describe old regular extents while newer file bytes are
dirty; lack of DELALLOC flags is not enough to rule that out. The Btrfs implementation
waits for ordered writes, including compression, and repeats the wait after acquiring
the inode lock. This helps within a call, not across the two files or pagination.
There is still no atomic pair observation and no protection after return.

SYNC is logically non-editing but can cause writeback, block and fail with I/O
errors. Expose this cost/behavior in the selected profile; do not present it as
zero-I/O metadata inspection. A strictly non-flushing diagnostic profile can show
raw maps but must not produce retention-eligible proof without an independently
established flushed, stable source/target premise. Preview should either explicitly
allow this observational writeback or leave retention unknown; it must not promise
an exact retain count from unsynced data.

Share donor extent observations once across immediate source-to-many lanes under
the existing `SourceSession` generation and bounded resource ownership. Charge map
capacity; stream or decline over-budget retention. Cache no target proof as a source
fact. Revalidate each receiver independently. Donor hardlink aliases may eventually
reuse a descriptor-bound map within the same scope, but path strings, inode numbers
without scope and timestamps alone are not cache keys. Start with per-entry reuse.
Our own clone effects can change SHARED flags even without changing donor bytes;
cached negative sharing flags may cause conservative extra renewal, never a new
positive conclusion. Invalidate mapping evidence if an owned operation can change
its backing, and do not broaden the source cache into a persistent address index.

Discard positive evidence at externally returned pause/resume or source-phase
boundaries; refresh any unstarted retention decision. Persist only policy identity,
requested work and historical result summaries, never a reusable proof token.
Already completed retention remains historical coverage under the caller's stable-
tree contract, not a guarantee against subsequent writes. Native restore from S to
D may replace or remove files and ends the relevant sharing claim.

## Fit into the existing library

This is a **storage observation + pure comparator + selected-work policy**:

- `tree/observe` owns descriptor acquisition, identity and the Linux ioctl backend.
  `tree/inspect` extends finite typed demands/facts with storage maps and performs
  shared source acquisition. Unsupported observation is a typed outcome; actual
  permission/I/O failures retain their causes. Do not perform ioctls from a pure
  `Comparator::compare` merely because it receives paths.
- `tree/inspect` owns normalized pair assessment, distinct from mutable work.
  Extend the current attributes/literal-link-only demand family deliberately.
  Ineligible shapes/metadata can short-circuit before mapping; any extra staged
  demand is a bounded explicit phase, not an arbitrary request-more-facts loop.
- `tree/compare` retains its existing recursion, membership and continuation.
  `transfer/sync` owns proof-qualified retained acceptance and effects. Declared
  inspection must expose the same observation to listed selection; it currently
  omits physical comparison. No second JW walker or copier is needed.

Illustrative API additions, **not implemented signatures**:

```rust,ignore
enum SharingObservation {
    FullDataSharing { shared_logical_bytes: u64, hole_bytes: u64 },
    VacuousData { hole_bytes: u64 },
    NotFull { reason: SharingMismatch },
    Unsupported { reason: SharingUnsupported },
    Unresolved { reason: SharingUnresolved },
}
struct PrivateDonorSharing { /* backend, SYNC, metadata and unknown policy IDs */ }
enum ListedIntent {
    CreateAbsent,
    RenewOwned,                 // always fresh regular inode, unchanged
    EnsurePrivateDonorSharing, // exact O authority + fresh proof or install
}
enum DesiredCompletion {
    Installed,
    RetainedVerified,          // accepted live witness, not a serialized capability
}
```

The storage result alone does not include inode privacy or metadata satisfaction.
`PrivateDonorSharing` composes those requirements for compatible regulars:
`Equal` means **satisfies this exact predicate**; `Different` means a known unmet
requirement, not necessarily different bytes. `Unsupported` and `Unresolved` map
to the existing explicit comparison variants. Omission stays `NotAssessed`.
For directories only local shape is compared; descendants still require traversal.
For symlinks select and document literal comparison separately.

All parameters belong in the versioned `SemanticId`: backend scope, SYNC behavior,
hole/vacuous semantics, metadata obligations and unknown fallback. Preserve the
storage reason in diagnostics rather than flattening every failed proof to false.
No `StatKey OR sharing` rule may report “sharing ensured”: a StatKey fallback would
retain ordinary copies. A separately named opportunistic stat profile could do
that, but would make only its weaker claim and is not selected here.

### Decision table and operation flow

| Intent / existing state | Decision |
| --- | --- |
| Existing StatKey ordinary mode | Current behavior unchanged; no sharing claim. |
| RenewOwned, even fully shared | Install a fresh private regular inode. |
| CreateAbsent, occupied leaf | Refuse as today, even if perfectly shared. |
| Ensure policy, eligible owned/private/metadata-equal/full or vacuous proof | Retain with fresh verified completion. |
| Ensure policy, known mismatch or unsupported/unresolved observation | Required staged reflink, but only under existing replacement authority. |
| Ensure policy, unowned collision | Refuse; sharing never grants ownership or native membership import. |
| Observation I/O error, detected race, guard or cleanup failure | Fail with truthful partial progress; no adoption. |

For ordinary work, this requires an explicit policy choosing `Renew` for unknown
sharing: today's default mirror has no baseline action for unresolved comparison.
Do not make the comparator silently authorize replacement. Required reflink failure
remains failure; no byte-copy fallback is inferred.

For tracked work, run after the existing native same-tree child transition:

```text
bind S/O + native guards
  -> exact desired/owned lookup and route/kind validation
  -> assess eligible regular pairs with shared donor acquisition
  -> validate whole immediate selection, ownership and live evidence
  -> retain verified OR existing private-stage/clone/publish kernel
  -> finish all S obligations and O-only retirement; settle cleanup
  -> native adoption gate using explicit satisfaction coverage
  -> optional restore D, reparent, verify native state
```

Clear-first has deliberately removed the candidates, and explicit replace-content
also clears them: neither benefits. Initially expose the optional intent only with
ReplaceAuthorized without explicit clearing, rejecting incompatible CLI combinations
rather than silently switching native route. CLI spelling belongs to JW review.

### Complete manifest and native adoption contract change

Today `ManifestCoverage.complete` requires `installed == desired` and complete
owned retirement. `ListedGuard` can only refuse; it cannot skip obligations. A
custom ordinary comparator therefore cannot implement this feature.

Add exact per-desired-leaf completion accounting, with disjoint `installed` and
`retained_verified` totals; complete requires their sum to equal desired **and**
all ownership/retirement/cleanup obligations to succeed. A preserved, excluded,
unassessed or unknown leaf never contributes verified retention. Bind accepted
retention to the same exact manifest/LeafId and current effect binding as installed
work. Common selection validation must reject fabricated retention outside the
ensure intent; public decoded values cannot manufacture live witnesses.

Keep actual effects honest: retained files add zero files_reflinked, bytes_reflinked,
rename or metadata-write effects. Count probe attempts, supported/unknown reasons,
retained regulars/logical bytes and vacuous outcomes separately. Full sharing bytes
are logical covered bytes, not exclusive physical disk savings or compressed size.

JW must revise its provider gate, outcome model and public schema/parser deliberately
to distinguish **complete S satisfaction** from **complete S installation**. Existing
schema-one consumers requiring every S leaf installed must not silently receive a
different meaning. A provider pre-1.0 API bump and consumer wire-version decision
belong to implementation review. Keep failure coverage even when all leaf work
finished but permission cleanup or owned retirement failed.

Donor `tree_state` adoption remains native authority. Retention must prove donor
data, expected kind/executable state and the stat/metadata preconditions of that
adoption profile before the gate. Add a bounded native test where retained S files
have older receiver inode/ctime identities; verify real cache adoption and subsequent
snapshot/restore do not hide a mismatch or lose an intended edit. If native cache
semantics demand additional normalization, design it explicitly or refuse retention
there. Do not weaken native verification because provider coverage succeeded.

The promise is S satisfaction at handoff. D≠S restore and later reparent/finish
must not be reported as universal final-D donor sharing. “Ensure private donor
sharing” here means retain proven eligible data or obtain a successful required
clone into a private inode; it does not newly promise physical sharing of inline
metadata that the filesystem may copy even during a successful clone.

## Cost and bounded verification before implementation acceptance

For F candidate files, E returned extents and N receivers, stat comparison is
roughly O(F) metadata operations. The new profile adds opens/closes, pre/post stats,
mount/backend checks and paginated FIEMAP work: O(E) normalization plus kernel tree
and backreference work, with source acquisition amortizable across N. Btrfs's
`fiemap_search_slot` explicitly calls sharing checks expensive. SYNC adds potentially
dominant writeback. Clone itself is extent-sensitive, but probing both sides can
cost more than cloning once; fragmented files can exhaust a configured probe budget.
Budget exhaustion means unresolved then authorized renewal, not successful proof.

Expected benefit is effect avoidance on the proven subset. It does not avoid
manifest lookup, route/device checks, native JJ calls, obsolete retirement or
unsupported-file installation. The compressed fraction and per-file extent counts
are unmeasured; 11.61 GB logical size alone predicts neither cost nor payoff.

Use one tiny private Btrfs fixture, tens of files and a few MiB, reused between
cases; no imported OpenCode corpus or large benchmark. Proposed acceptance:

1. Real required reflink pair, independent inode/nlink=1, flushed uncompressed
   extents: positive observation and retained inode. Equal-byte ordinary copy:
   not a positive proof (verify fixture did not become deduplicated). Source and
   receiver each shared with different files: never positive merely from SHARED.
2. Same-inode hardlink and receiver hardlink to a third file: renew, preserving
   donor/third-file bytes. Donor nlink>1 with independent receiver: eligible.
3. Modify one middle block after cloning, including a same-size/restored-mtime
   edit; partial sharing must not retain. Different extent segmentation with the
   same normalized mapping should be accepted. Include range-cloned shifted data
   to discriminate corresponding logical offsets.
4. Empty, all-hole, sparse-with-data, trailing hole, unequal hole layout, partial
   final block and preallocation beyond EOF. Check exact size, shared versus hole
   accounting and zero false “shared bytes” for empty files.
5. Real compressed and inline fixtures (mount option alone is not evidence):
   unsupported fallback. Where feasible, two range-cloned windows of one compressed
   extent demonstrate why physical-tuple equality is insufficient. Exercise
   unwritten/delalloc and unknown flags when available; otherwise mark unavailable
   and cover parser/decision handling with constructed observations.
6. Buffer-boundary pagination, invalid/overflow/non-progress streams, budget
   exhaustion, unsupported ioctl/backend and genuine I/O failure. Dirty overwrite
   followed by the SYNC profile must see current backing; deterministic injected
   pre/post identity changes must fail. Passing race tests is not a concurrency proof.
7. Metadata-only mode/mtime and executable mismatches renew under the first profile.
   Ordinary default and RenewOwned still behave exactly as before. Ensure mode
   refuses unowned collisions, performs O-only retirement and preserves coupled
   directory-collapse refusal. No keep counter substitutes for completed cleanup.
8. One tiny native D=S case with mixed installed/retained files, then D≠S restore,
   including source membership additions and type changes. Verify native identity,
   exact bytes/executable state, truthful coverage/effects and failure-before-adoption.
   A receiver-write isolation check follows fixture acceptance, outside probing.

Only after these gates, optionally measure this same small fixture: metadata-only,
always-renew and ensure-sharing on clean and partly diverged states. Report probe
counts/extents, fallback reasons, retained inode count and wall/CPU separately;
do not call such a sample representative speed evidence. A negative latency result
can coexist with useful reduced churn.

## Review forks and related work

1. **Select the benefit:** ensure-sharing retention versus mandatory fresh inode.
   These are distinct intents; accept the new opt-in semantics explicitly.
2. **Select the initial coverage:** narrow same-mount/subvolume, uncompressed proof
   is soundly bounded but may help little here. Compressed support remains research;
   do not disable compression to manufacture eligibility.
3. **Select SYNC and preview semantics:** permit documented observational writeback
   or leave proof unavailable. No unsynchronized positive shortcut.
4. **Select native coverage/wire evolution:** verified-retained completion needs
   provider and JW agreement before cache adoption, not a local comparator patch.

`FIDEDUPERANGE` is a separate possible operation: the kernel compares bytes and
shares equal ranges atomically relative to writes, with alignment/length and
per-destination status/bytes-deduped rules. It **mutates storage**, may read all
data, does not prove that sharing existed beforehand, and does not remove receiver
hardlink aliases. It belongs in an expressly authorized executor design, not a
read-only comparator or this first retention slice.

The [earlier Btrfs study](../performance/opportunity-also.md#btrfs-checksums-and-change-tokens)
is a related content-evidence inquiry: checksums cannot certify pairwise sharing.
The [annealing assessment](../architecture/annealing0.gpt6ax.md#why-the-btrfs-inquiry-does-not-gate-this-baseline)
keeps unconditional renewal useful independently of this optional optimization.
The [consumer account](../../doc/consumer/jj-waltz.md) is the maintained home for
JW's needs; this document proposes a new capability, not accepted implementation.
