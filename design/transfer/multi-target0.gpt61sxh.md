---
type: CompositionResearch
title: Bounded source cohorts with independent destination continuations
description: Directory-local source-fact reuse, optional one-leaf byte reuse, explicit detach-and-reobserve lag policy, and eight tiny counted tests without a catalog or global scheduler.
resource: /design/transfer/multi-target0.gpt61sxh.md
status: proposed-and-bounded-model-tested
generated: { by: "model:openai/gpt-6.1-sol#xhigh", at: 2026-10-20 }
sources:
  - resource: "opencode://ses_ea79000c7ffeLCfxCyCYX34qWT"
    title: Independently designed bounded source-to-many composition, tested eight model/filesystem discriminators, and specified destination-owned continuation and incremental landing
  - resource: /design/roadmap/brief0.gpt6a.md
    title: Independent commission, reading boundary and good-source/no-history assumptions
  - resource: /design/architecture/contracts0.gpt61sxh.md
    title: Completed observation, authority, child ownership and component continuation contracts
  - resource: /design/execution/contracts0.gpt61sxh.md
    revision: 5d6c003a
    title: Ordered target-owned progression, typed goals, claims, resource-closed effects and saved component data
  - resource: /design/transfer/modes0.gpt6am.md
    revision: 053ecfe0
    title: Completed inherited modes, known leaves, residual obligations and separate reduction evidence
  - resource: /design/actions/dispatch0.gpt61sxh.md
    revision: 7a325e0c
    title: Completed finite dispatch, stable input/target separation, retained claims and request acknowledgment
  - resource: /design/observation/facts0.gpt6am.md
    revision: 5423a71f
    title: Explicit acquisition, physical presence, full-demand compatibility and immediate fact lifetimes
  - resource: /design/observation/names0.gpt61sxh.md
    title: Completed native-name evidence and sparse coordinate promotion instead of pinning whole batches
  - resource: /design/observation/codec0.gpt6am.md
    revision: 8b709ce0
    title: Completed owned Snapshot transport, compatibility and allocation qualifications
  - resource: /design/transfer/kernels0.gpt6am.md
    title: Completed kernel/lease study distinguishing delivered descriptor reuse from avoidable observation
  - resource: /design/execution/walk0.gpt61sxh.md
    title: Delivered walking specimen and attributed focused process/failure checks, not rerun here
  - resource: /examples/walk/src/driver.rs
    title: Inspected actual inspector/executor types, per-target pending owner and acknowledgment boundary
  - resource: /examples/walk/src/io.rs
    title: Inspected full public inspection, exact-byte source reads, independent target checks and staging closure
  - resource: /examples/walk/src/program.rs
    title: Inspected concrete lowering and its intentionally fixed single-target semantics
  - resource: /examples/walk/src/state.rs
    title: Inspected owned native coordinates, active frames and disjoint root binding
  - resource: /src/visitor.rs
    title: Inspected private two-side listing/acquisition/finalization seam and full-batch validation
  - resource: /src/listing.rs
    title: Inspected private source handles, no-probe hints and platform-specific acquisition
  - resource: /src/pair.rs
    title: Inspected child-coordinate lowering before release of clean facts
  - resource: /sync/src/apply.rs
    title: Inspected current per-install source open, FICLONE fallback and output finalization
  - resource: "file:///home/rektide/.rustup/toolchains/nightly-2026-10-14-x86_64-unknown-linux-gnu/share/doc/rust/html/src/std/fs.rs.html#1153"
    title: Configured std documents shared read/write/seek state of File::try_clone
  - resource: /.test-agent/multi-target/README.md
    title: Ignored owned probe instructions; discriminating construction and results preserved in this document
extensions:
  ticket: diffdir-tree-transfer-multi-target
  reading_boundary: Independent brief, scoped ticket, named completed prerequisites and targeted implementation only; withheld parent statement, full roadmap waves, parent transcript and concurrent native-integration output were not read, searched, cited or evaluated; no subagents
verification:
  method: Eight isolated std-only Rust tests, two tiny real-filesystem cases, owned-data rebind, warnings-denied clippy, formatting and document checks
  result: Eight tests passed with none skipped; clippy and formatting passed; application call counts are not syscall or runtime measurements; existing suites, native JJ and workloads were not run
---

# Source cohorts, not a multi-target tree catalog

## Recommendation

**Share one bounded, immutable immediate-source batch among targets already ready
for that source region. Keep every destination's policy, claims, goals, pending
request, acceptance and finish state independent. Detach a failed, paused or
divergent receiver and reobserve its remaining inputs later.** No receiver may pin
earlier source batches indefinitely. Do not build a persistent full-tree catalog
or introduce a global scheduler to obtain this local reuse.

Start **application-level orchestration over explicit known inputs**. Directory
reuse belongs at the narrow acquisition/inspection seam when a real consumer needs
walking; today's sealed `DirectoryPlan` does not accept injected source facts.
This study does not warrant an immediate generic multi-target API, modification of
either delivered example, or a change to sync or JW defaults.

The counted probe distinguishes three independent propositions:

| Proposition | Result on the tiny two-target fixture |
| --- | --- |
| Share source discovery and attributes | Source directory acquisitions **4 → 2**, yielded names **8 → 4**, metadata acquisitions **7 → 4**. |
| Additionally share a tiny immutable leaf body | Source body reads **5 → 3**; facts-only sharing still reads five times. |
| Avoid required destination effects | **No such saving:** both shared variants accept five deliveries and perform three destination file installations, totaling six installed bytes. |

These are application/model counts, with one real-filesystem anchor—not production
syscall totals, allocator measurements, a speedup claim, or a workload benchmark.

## 1. Useful sharing boundaries

The shared object contains **physical source evidence**, not the first receiver's
assessment result:

```rust
// Schematic application/internal seam, not delivered public Rust types.
struct SourceKey { binding: SourceBindingId, relative: NativePath }
struct SourceBatch {
    key: SourceKey,
    complete_names: OwnedNames,
    facts: FrozenFacts, // known / not requested / failed, with physical origin
}
struct Delivery {
    input: SourceKey,
    target: TargetCoordinate,
    policy: TargetPolicy,
    goal: Goal,
    claims: TargetClaims,
}
```

An observer privately builds/acquires the cache, then freezes it for consumers.
Accessors do not perform I/O. Additional demands use an explicit acquisition phase;
they do not mutate an object concurrently borrowed as frozen. No `Action`, target
absence, satisfaction result, exclusion mask or permission lease becomes a common
source fact. Acquiring source kind by stat can cache its attributes for later
demands; several receivers need not perform that acquisition again.

| Sharing scope | Retained data and useful return | Recommendation |
| --- | --- | --- |
| One source coordinate already supplied | Stable input binding/route; attributes acquired once for currently participating deliveries. No directory discovery. | Smallest first consumer slice. |
| One immediate source directory | Complete ordered native names, hints, admitted physical attributes/link text needed by the union of current demands. Each target independently merges its own names and finalizes policy. | Useful walking scope; cap and retire it. |
| One current leaf body | Immutable bytes, only while their contemporaneous deliveries are attempted. | Optional separate optimization, with a byte cap; not necessary for fact sharing. |
| One current open source file | Runtime descriptor for explicitly safe transfer operations. No desired-target result. | Defer by default; offset/fallback rules below are required. |
| Persistent source tree catalog or all bodies/history | All discovered regions survive unrelated target progress. | Not required; refuse this accidental retention policy. |

For directory inspection, collect the current receivers' **own** name admission and
fact demands before acquiring their union. Receiver A may exclude `b`; receiver B
can still demand its attributes and copy it. If all participating receivers exclude
that name, it remains physically present with unrequested facts, not absent. A
terminally preserved region needs no interior source scan when nobody else needs
it. A scoped satisfaction proof belongs to the particular target/domain/result
contract; it does not satisfy another destination or manufacture its leaf count.

True source absence for target-only names requires a successful **complete** source
listing at that region, or a separately declared desired-domain contract. An
incomplete listing, capacity refusal, failed acquisition, exclusion, or supplied
partial leaf list grants no deletion authority. The probe is source-present-only,
retains extras, and never exercises removal.

### Per-target validation and errors

Each receiver retains its full immediate admission/validation boundary before
that batch's effects. Sharing acquisitions is not permission to interleave that
receiver's early effects with its still-fallible late observations. Full existing
Visitors still receive full attributes and the existing link payload contract;
unknown/unrequested facts cannot implement their unconditional metadata accessor.
Do not hoist one target's late Update/Seed/Prune decisions into universal source
filtering or omit its callbacks because another target made the same decision.

A source-fact failure can be cached as a failed acquisition **for this batch**.
Only a receiver demanding that fact fails; a receiver terminally excluding that
name may finish normally. Report cached failures at that receiver's own ordered
union boundary; an eagerly acquired source error for later `b` must not preempt
its earlier target error at `a`. Cache acquisition order is not error authority.
A source listing failure blocks receivers needing that
listing; it is never an empty listing. Target observation, container, staging,
permission and installation failures remain target-local. There is no all-target
transaction or promise that no peer has already performed effects.

## 2. Bound and deliberate lag policy

Use a **bounded ready cohort with detach-and-reobserve**, not an indefinitely
backpressured fanout log:

1. Take at most `C` already-eligible requests with exactly the same `SourceKey`
   and compatible physical observation interpretation. Do not scan future target
   work or wait indefinitely for another target to arrive.
2. Acquire one complete source batch. Each receiver accepts its own lowering into
   its existing driver, retaining owned future coordinates/goals rather than a
   borrow or `Arc` to the batch. Shared evidence is read-only; target evidence and
   policy decisions remain private.
3. Reuse current facts for immediately eligible effects in existing logical order.
   A cohort barrier can wait for the admitted bounded turn, but on returned failure,
   receiver budget exhaustion, explicit pause, or frontier divergence, detach that
   receiver. Its unresolved request/coverage remains its own; detachment is **not**
   skip, failure acceptance, or incomplete coverage by itself.
4. Retire the batch before acquiring another source directory. Detached work later
   reacquires the facts it needs. A not-yet-accepted directory inspection is retried
   as a directory; an already-accepted source-leaf goal can use direct leaf
   reobservation under the fixed desired-file program. A general Visitor may need
   its complete charged refresh, as the execution contract already specifies.
5. Drop all runtime caches at saved pause. A target can optionally rejoin a later
   ready cohort at the same current key; never rewind its accepted prefix to rejoin.

Choose explicit limits such as `Caps { cohort: C, source_entries: K,
source_name_bytes: L, source_link_bytes: R, cached_body_bytes: B }`. The live
**shared cache** holds at most one source batch: `O(K * fact_slot + L + R)` plus
its single region coordinate, one containing source handle if acquisition needs
it, at most one cached body
of `B` bytes, and `O(C)` membership. The first slice can have zero open source-file
handles between effects. This is a structural/payload bound, not an RSS bound.

Reject/return `TooWide` or `PayloadTooLarge` **before accepting the affected
directory batch**. An application may explicitly choose today's independent
inspection path instead, dropping the partial shared batch first and charging the
new attempt. The probe tests refusal; an automatic fallback implementation is not
claimed. Do not present arbitrary chunks of an incomplete directory as complete
Visitor batches. Known selected leaves can instead be chunked because their
contract makes no all-directory/absence claim.

These limits do **not** bound target-listing widths, per-target frame/suffix state,
candidate-state clones, body size of an uncached streaming transfer, bytes/syscalls
per effect, or elapsed time. A general target-specific whole-batch policy can still
require a large target batch. Copying source facts into owned finalized plans can
also cost `O(C*K)` private payload; release those plans at ordinary lowering rather
than count their copies as nonexistent. Capped shared storage is not a bound on
the entire program's memory.

In particular, DFS must not pin a parent's source batch while descending. If source
names are `a-directory` then `z-file`, retire parent facts before `a-directory`'s
interior; later `z-file` uses its retained coordinate and reacquires execution
inputs. Reordering all files before directories merely to improve reuse would
change the driver's ordering contract. The probe deliberately has files before
its child directory and represents an advantageous, not universal, sharing case.
Distinct descendant policies can split cohorts; independence wins over reuse.

An alternative strict whole-cohort barrier is smaller but lets one blocked target
stop every peer. A `B`-batch sliding window still needs eviction/detachment and is
unnecessary here. Detachment pays extra observation to keep old-source retention
bounded. It promises peer **logical** progress after a failed attempt returns,
not interruption of a stuck syscall/native command: running work must first join
and close its resources at the existing boundary. No global fairness or latency
guarantee is smuggled into an attempt-count quantum.

## 3. Binding, claims and continuation ownership

### Stable input and nonaliased destinations

`SourceBinding` fixes the caller's donor identity/root mapping and any supplied
revision/input interpretation. Reuse is keyed by that binding plus a lossless
native relative coordinate and fact interpretation—not just a display path,
mtime, inode hint, or equal source bytes. Changing the bound donor while restoring
a run refuses. Good source and stable caller-controlled topology are premises;
this adds no recurring donor-quality scan, Merkle catalog or proof service.

Resolve destination roots under a caller-owned binding contract **before effects**:

- Distinct target IDs or strings do not establish distinct physical writers. Check
  normalized/component ancestry and resolved root aliases; use available root
  object identity where needed. Source/target overlap is also a conflict.
- First slice: reject duplicate/aliased roots and ancestor/descendant target roots,
  even if current leaf names happen to differ. Revalidate binding compatibility
  and actual routing on rebind; decoding IDs alone is not a filesystem check.
- If deliberate coalescing is later supported, require one explicitly selected
  physical target identity, compatible policy/goals and **one** continuation/claim
  owner. Two UI aliases then describe one delivery, not two successful targets.
  Never silently coalesce incompatible preservation or overwrite rules.
- More general mappings require cross-target claims in a common physical namespace:
  exact leaf duplicates, leaf/ancestor conflicts, container dependencies and
  exclusive-region/native footprints all matter. Equal source input does not
  make overlapping destination claims independent. Reservation lifetime extends
  through the relevant competing-discovery domain, not only until first success.

Canonical path checking is useful but not a universal alias resolver. The probe
detects ordinary equal/`/.` and nested paths, not bind-mount aliases or adversarial
topology. Applications retaining those possibilities must supply stronger binding
knowledge or refuse them. Staged leaf replacement, rather than writing through
an existing inode, also matters when target files are hardlinked. The good-source
premise is not permission to overwrite it through an unnoticed destination alias.

### One semantic owner per destination

```rust
struct MultiSaved {
    schema: Version,
    source: SourceBinding,
    targets: Vec<TargetSaved>,
}
struct TargetSaved {
    binding: TargetBinding,
    program: ProgramDescriptor, // policy, mapping, satisfaction, desired domain
    claims: OwnedReservations,
    walk: OwnedFrames,          // phases, remaining suffix, aggregates, coverage
    pending: Option<Pending>,  // goal/recipe, ticket, attempt/block, dependency
    components: ComponentsState,
}
struct MultiRuntime {
    source_observer: SourceObserver,
    targets: BoundTargetRuntimes,
    batch: Option<SourceBatch>,
    body: Option<CappedBody>,
}
```

This is a semantic data contract, not a second implemented cursor schema. Reuse
the existing target driver's frames and pending owner; do not introduce another
source-coordinator sequence that independently pops its children. The coordinator
chooses currently eligible reuse opportunities and supplies inputs; it does not
acknowledge every destination when a source read succeeds. Requests/acks identify
`(TargetId, ticket, pending unit)`, so target-local ticket `1` is not globally unique.

Before/children/finish dependencies, accepted aggregate, incomplete ancestors,
manifest ordinal and required finish values remain per target. A source-directory
enumeration count is not a target's settled-leaf count. Generated input may share
an immutable chosen body when its producer/input/parameter binding really matches;
target-dependent reports/ordinals cannot share recipes merely because they visited
the same source. Finish and native readiness must wait on that target's outcomes.

At a joined pause save only future obligations and their required component data.
No source `Metadata`, borrowed name, shared-batch ID requiring a lost cache, live
handle, permission lease or transfer job belongs in the DTO. Promote remaining
native coordinates before release; sparse survivors must not pin a large arena.
Codec/version/native-byte validation and executable rebinding follow the completed
codec and walking contracts. A scheduler tie-break/cap normally affects efficiency,
not desired output; it need not become frozen output parameters unless the
application actually assigns it semantic meaning.

A visible but unacknowledged destination remains unresolved. Retry reobserves that
destination and the required source input and may settle with no new installation;
it does not infer acceptance from another target's success. A blocked target may
be retried, left detached, or explicitly skipped under the existing incomplete
coverage rules. Accepted peers are neither rolled back nor recollected. Forked or
older saved states may repeat work. No effects log, receipts, recovery guarantee,
atomic cursor/effect publication, locking or exactly-once delivery is proposed.

## 4. Source observation, bytes, handles and writes are different

With `N` independent destinations, one source listing/stat acquisition can feed
`N` policies. This does not remove target listing, satisfaction, output inode,
staging, final metadata/readback, rename or clone work. If all `N` need different
physical outputs, there are still `N` installations. A satisfied, preserved or
unselected target can avoid its own installation; that follows from its goal and
state, not sharing. Reflinking saves data movement, not target ownership or the
number of destination inodes. Hardlinking targets together is a different semantic
choice, not a free implementation of independent copy.

Facts-only reuse does not share file bytes. An exact-byte satisfaction program
still needs desired bytes for each attempt unless they are supplied once as an
immutable input. OS page cache may already service repeated reads; fewer API reads
does not prove less device I/O. A one-leaf body cache can cap retention to `B`,
discard on the next leaf/batch/pause, and let a detached target reread. For larger
files use independent ordinary transfer, or separately commission a bounded chunk
fanout with explicit lag/cancellation and satisfaction rules. Do not retain every
body for eventual laggards. Whole-file exact-byte comparisons and kernel-offloaded
copies are not automatically compatible with a chunk broadcaster.

### If live source handles are later shared

**`File::try_clone` is not an independent read cursor.** Configured std explicitly
documents that reads/writes/seeks affect both instances. In the real six-byte
probe, copying handle A consumes six bytes; copying its duplicate next consumes
**zero**, since the shared offset is already EOF. `Arc<File>` does not fix this.

Choose one explicit strategy rather than letting the installer guess:

| Strategy | Required offset/fallback contract |
| --- | --- |
| Independent `File::open` per target | Independent ordinary copy cursors; preserves current installer shape. Saves no opens but may still share directory facts. Recommended default. |
| One serialized source descriptor | Exclusive ownership through each complete transfer; seek source to zero before every copy attempt. Never seek while another reader/clone fallback uses it. Output truncation and output position reset are separate obligations. |
| Explicit-offset reads/copy API | Each transfer owns its offset and short-read/retry loop; positioned reads do not advance the shared cursor. Do not wrap this and assume std `io::copy` retains its current offload behavior. |
| Whole-file clone from a shared source descriptor | FICLONE itself does not consume the source offset; **per-target fallback may**. Required-clone versus permitted-copy remains per-target policy. Serialize fallback/reset, use independent opens, or an explicit-offset copier. |

For a copy fallback, reset/discard the private destination appropriately and
establish **both** source and output offsets for the chosen copier. `set_len(0)`
does not reset an output cursor; truncating output cannot rewind an EOF source.
The tiny probe forces the reset path and verifies the correct six bytes after
explicit source/output seeks. It also checks Unix `read_at(..., 0)` leaves the
shared cursor unchanged. It does **not** perform or fault a real FICLONE.

Current `sync/src/apply.rs:846–884` opens a fresh source/output for **one** install,
keeps them through FICLONE/fallback and finalization, and truncates before fallback.
Its fresh cursors plus the documented non-consuming whole-file clone make that
current path coherent. Do not diagnose it as a `try_clone` bug or transplant
source sharing without changing its assumptions. Clone refusal/fallback, receiving
filesystem capabilities and permissions remain target-specific; one destination's
failure is not a globally cached prohibition for all others. Path-based non-Linux
helpers require their own API contract; no cross-platform handle reuse is proved.

## 5. Actual bounded probe

Ignored scratch: `.test-agent/multi-target/{Cargo.toml,model.rs,README.md}`, nested
workspace and isolated `target/`, no dependencies or production imports. Eight
tests passed. Six are deterministic model/owned-data tests, one anchors shared
copy counts on a real tiny tree, and one anchors alias/offset/reset behavior on
real files. The model's small in-memory source map is its test environment, **not**
a catalog built or retained by the proposed coordinator.

### Fixture and counter definitions

```text
source/a   = aa       target A/a = aa       target B/a   = zz
source/b   = bb       target A/b = KEEP     target B/b   absent
source/d/c = cc       target A/d/c absent   target B/d/c = cc
                     A/extra = x           B/extra = y
```

All roots and `d` containers exist. A terminally preserves `b`; B selects it.
Other source-present regular files use exact-byte settlement, attributes
unspecified. Both preserve extras and refuse unsupported output shapes. There
are **three distinct source leaves**, five selected target deliveries, two
equal-byte no-write deliveries, and three required file installations.

`lists` = calls acquiring one immediate source-directory listing; `entries` =
source names yielded; `facts` = explicit no-follow source metadata acquisitions,
directory entries included; `reads`/`read B` = full desired-source body reads and
returned bytes. `deliveries` = accepted target leaf goals, not attempts or sources.
`installs`/`installed B` = successful destination file writes in this adapter.
Binding/ancestor checks, fixture setup/assertion reads, target metadata/read calls,
staging and kernel internals are outside the source counters. Target satisfaction
checks are separately asserted: A two and B three on ordinary completion.

| Run | lists | entries | facts | reads | read B | deliveries A/B | installs A/B | installed B |
| --- | ---: | ---: | ---: | ---: | ---: | --- | --- | ---: |
| Independent desired-copy walks | 4 | 8 | 7 | 5 | 10 | 2 / 3 | 1 / 2 | 6 |
| Shared source facts, independent bodies | 2 | 4 | 4 | 5 | 10 | 2 / 3 | 1 / 2 | 6 |
| Shared facts + one-leaf body cache | 2 | 4 | 4 | 3 | 6 | 2 / 3 | 1 / 2 | 6 |
| Supplied leaves `[a,b]`, then `[d/c]`, body cache | 0 | 0 | 3 | 3 | 6 | 2 / 3 | 1 / 2 | 6 |
| B blocks at `a`, detaches, later rebind/retry; no body cache | 3 | 5 | 7 | 5 | 10 | 2 / 3 | 1 / 2 | 6 |

The **real shared-copy case** obtains the third row's same application counts
`2/4/4/3`, five target checks and `[1,2]` installations, verifies all final bytes,
and leaves A's `b` and both extras unchanged. It calls actual `read_dir`,
`symlink_metadata`, `read` and `write`, but these counters are not traced syscalls.
Its simple adapter directly writes a checked regular/absent destination and closes
the call. There is no staging, atomicity, metadata preservation, durability or
permission-management claim; the delivered walk/sync installer remains unchanged.

### Failure, retirement, pause and refusal checks

- **B blocks before its `a` effect:** A accepts `a`, preserves `b`, installs `d/c`
  and finishes with two accepted leaves while B has zero. Both shared batches are
  retired; B holds only pending `a` plus `b,d` future coordinates. After owned-data
  rebind it retries `a`, then `b`, then inspects `d`; no root relisting or A recount.
  Initial source costs `lists=2,facts=4`; resume adds `lists=1,facts=3` (direct `a,b`
  reacquisition plus `d/c`). Six total target attempts yield five accepted goals;
  old failed acknowledgment cannot advance the retry.
- **Source acquisition fails for `b`:** B's whole root lowering blocks before any
  B leaf effect. A excludes `b`, finishes both its leaves and retires its batches.
  Source failure is not absence, nor a reason to block an unrelated admitted target.
- **Pause after only A's `a` acknowledgment:** dropping the active source batch
  leaves A at `d` and B at `a`. Newly constructed independent runtimes consume their
  saved suffixes, eventually reach `[2,3]`, and never recount A's accepted `a`.
  Frontiers need not be forced back together to reconstruct a shared cache.
- **Capacity and identity:** the fourth root name exceeds the three-entry cap,
  publishes no batch, performs no installations and leaves both directory requests
  unresolved. Changed source binding or target preservation policy refuses rebind.
  A's exclusion is explicitly checked not to remove physical `b` from B's schedule.
- **Real alias and handle case:** ordinary separate roots bind; duplicate/`/.`,
  nested target and source-as-target routes refuse. Shared-offset EOF, explicit
  fallback resets, unchanged positional-read offset and both output byte strings
  are asserted as described above.

At ordinary completion the two shared directory batches are retired, current
batch/body are empty, and measured logical high-water counters are **three source
slots and two cached body bytes**. The collector's limits are `K=3`, name bytes
`L=64`, and cached body `B=2`. The scratch clones a body into the active attempt,
so cache plus delivery body can hold four bytes; two cached bytes is not its total
allocation peak. No allocator/RSS measurement or hostile-input decoder is supplied.

Each real case owns its uniquely named fixture in this scratch directory, uses
only source plus two targets, and stays below **24 workload entries / 128 file
bytes**, well below 256 entries / 1 MiB. Cleanup removes only its exact owned
fixture. Build output is separate in the scratch target directory. No other
scratch material was removed, imported or executed.

Executed from repository root:

```sh
CARGO_TARGET_DIR=.test-agent/multi-target/target cargo nextest run --offline --manifest-path .test-agent/multi-target/Cargo.toml --no-capture
CARGO_TARGET_DIR=.test-agent/multi-target/target cargo clippy --offline --manifest-path .test-agent/multi-target/Cargo.toml --all-targets -- -D warnings
cargo fmt --manifest-path .test-agent/multi-target/Cargo.toml --check
```

**8 passed, 0 skipped; strict clippy and formatting passed.** The probe has a fixed
tiny ordered queue, not a replacement walking driver: no generated finish,
permission lease, native-byte codec, parallel transfer, process restart,
after-visible-effect injection, generalized arbitrary-tree cohort splitting or
public-plan compatibility proof is claimed. Its saved data is moved into fresh
runtime construction without a codec. The completed walk report's 9 codec-free /
11 all-feature tests and actual process/after-visible-effect checks are reused as
attributed prerequisite evidence, not rerun or relabeled as multi-target tests.

## 6. Smallest landing and promotion gate

**First application slice:** accept a caller-supplied bounded batch/stream of known
physical input leaves, stable source binding and explicit disjoint target bindings.
Build target-owned typed deliveries and use the existing request/ack idea. Share
input attributes in one invocation; keep source-body caching optional and capped,
with independent ordinary opens/transfer as default. Target satisfaction and
overwrite/removal choices remain explicit. This needs neither a new provider
framework nor a generic multi-target `Action`, and makes no discovery or authority
claim about unlisted target names. An application reading a supplied full list is
not authorization to construct a persistent full-tree list for every ordinary walk.

**Walking reuse seam, when needed:** factor private source enumeration/acquisition
from target union/assessment, approximately `observe_source_dir` →
`inspect_target_with_source`. Source batch construction remains an actual-observer
responsibility, with no caller-forged `Metadata`/presence. Reuse merge/comparison/
validation internals and the earlier explicit demand design. The current public
`inspect_directory` remains the full-demand independent path.

This is not a drop-in cache around today's public inspector: `visitor.rs:1004–1012`
lists both sides, and `inspect_listed:1016–1130` privately owns merge, early steering,
admitted acquisition and finalization. Nor can one target's sealed plan be rebound
as another's. `examples/walk`'s `Inspector` returns that concrete `DirectoryPlan`,
and its concrete program checks source-present baseline actions and complete
inspection; arbitrary per-target exclusion is not automatically supported by
swapping its inspector. Leave that specimen and generic sync untouched rather
than claim a wrapper already provides shared directory observations.

The smallest eventual generic surface is **immutable physical input acquisition
plus explicit per-target assessment**, not a library-wide multi-target scheduler
or saved shared catalog. Promote it only after a consumer shows:

1. Same per-target full-Visitor facts, admission, error ordering and selected work
   as independent inspection; differing exclusions, target extras/shapes and an
   acquisition failure do not contaminate peers.
2. Existing per-target ownership/finish/coverage survives real pause/rebind and
   after-visible-effect retry, including native byte names; no source batch is
   required to decode a target continuation.
3. Wide-directory refusal/explicit fallback and parent-to-child retirement respect
   the declared live-source cap; interleaved DFS and divergent subtrees detach
   rather than pin ancestors or change ordering.
4. Target alias/overlap/claim validation runs before affected effects, and every
   runtime staging/permission/native footprint has a real owner.

The eight tests settle the local composition choice, not those production gates.
They support **bounded orchestration first** and preserve a useful private
acquisition seam for later evidence, rather than justify an immediate public API.

### Connections and scope

- [Execution contracts, ownership and failure](/design/execution/contracts0.gpt61sxh.md#2-progress-acceptance-and-failure): reused separately for each target; source retirement is not another child owner or acceptance channel.
- [Dispatch handoff](/design/actions/dispatch0.gpt61sxh.md#6-remaining-forks-and-next-reuse-point): resolves its multi-target question with stable input keys, target-specific acknowledgments, explicit aliases and separate delivery/write counters.
- [Inherited modes](/design/transfer/modes0.gpt6am.md#4-reduction-and-saved-mode-state): terminal preservation and sufficient proof/reduction remain target-specific; known leaves remove discovery, not satisfaction or target effects.
- [Demanded facts](/design/observation/facts0.gpt6am.md#3-smallest-demandacquisition-contract): union demands belong to acquisition, not accessor I/O or a shared target policy.
- [Name promotion](/design/observation/names0.gpt61sxh.md#3-bounded-experiment-and-actual-results): promote sparse future coordinates before retiring shared backing instead of pinning the batch through lagging targets.
- [Kernel study](/design/transfer/kernels0.gpt6am.md#1-current-ownership-and-costs): current per-install descriptor reuse is already delivered; this composition does not eliminate output verification or select a replacement copy backend.

Ordinary refresh, mirror, strict seed, desired-byte settlement and explicit removal
remain different policies. Sharing source observation selects **none** of their
product defaults. Native metadata, colocation, JJ command boundaries/readiness and
application-wide lifecycle handling remain outside this return; no native JJ was
run and no concurrent native-integration result was consulted. Only this document
is committed. The scoped ticket gets a pointer/findings/session note; parent owns
closure, exports, indexes, integration and comparison against withheld early stakes.
