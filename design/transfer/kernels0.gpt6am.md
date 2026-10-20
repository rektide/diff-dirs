---
type: ResearchProposal
title: Remaining transfer-kernel work after leaf ownership improvements
description: Current-source census and a tiny directory-replacement trace distinguish reusable directory observations from necessary copying and output verification, with ranked local implementation slices.
resource: /design/transfer/kernels0.gpt6am.md
status: proposed
generated: { by: "model:openai/gpt-6-astra#medium", at: 2026-10-20 }
sources:
  - resource: "opencode://ses_ea7d3fbf6ffeH4eyKrL2VgJe47"
    title: Independent transfer-kernel census, bounded replacement traces and retained-plan allocation model
  - resource: /design/roadmap/brief0.gpt6a.md
    title: Independent research scope and reading boundary
  - resource: /doc/architecture/README.md
    revision: a63595a9
    title: Prior proposed component ownership, coupled replacement and failure contracts
  - resource: /design/observation/facts0.gpt6am.md
    revision: 5423a71f
    title: Completed fact-demand study; working file verified identical to commissioned revision
  - resource: /design/performance/inspection0.sol61x.md
    title: Historical descriptor metadata, ordinary copy offload and staging anatomy
  - resource: /design/performance/leaf0.sol61x.md
    title: Historical controlled evidence for delivered leaf ownership improvements
  - resource: /sync/src/apply.rs
    revision: a63595a9
    title: Current application handoff, replacement, recursive removal, permission leases and copy kernels
  - resource: /sync/src/lib.rs
    revision: a63595a9
    title: Internal run and root dispatch
  - resource: /sync/src/paths.rs
    revision: a63595a9
    title: Structural checks and existence observations
  - resource: /sync/src/platform.rs
    revision: a63595a9
    title: Unix mode and Windows readonly/symlink distinctions
  - resource: /src/visitor.rs
    revision: a63595a9
    title: Immediate enumeration, admitted metadata and mismatch readlink acquisition
  - resource: /src/pair.rs
    revision: a63595a9
    title: Root observations and dropping clean plans after child lowering
  - resource: /src/walk.rs
    revision: a63595a9
    title: Name-only child progression and divergent batch handoff
  - resource: /src/listing.rs
    revision: a63595a9
    title: Backend-specific metadata acquisition versus cached Windows attributes
  - resource: /.test-agent/transfer-kernels/README.md
    title: Local scratch protocol, actual current-library probe and read-only retained-plan model
extensions:
  ticket: diffdir-tree-transfer-kernels
  reading_boundary: Independent brief, scoped ticket, completed facts, prior architecture and historical measurements; no withheld statement, full roadmap, parent transcript or concurrent topic drafts
verification:
  method: Offline isolated debug build; four tiny Linux/btrfs traces; assertions for content, mode, exact mtime and intact refusal; current-source census
  result: Actual removable and retained cases and both read-only model cases passed; no production changes, suite runs or timing claims
---

# Remaining transfer-kernel work

**Prioritize local directory fact/lease handoff, then retain the already-authorized
removal precheck for a coupled directory→leaf replacement.** Internal leaf reuse
is delivered. The remaining extra reads are not evidence that ordinary copying
needs a new transfer backend.

The current replacement probe lists each of two target directories twice and
observes their three descendants twice. A read-only retention model keeps the
two real `DirectoryPlan`s in **5,680 requested live heap bytes** on this fixture.
That can eliminate the second observation pass; it does not eliminate unlink,
rmdir, permission changes, staged installation or exact output-mtime verification.

## 1. Current ownership and costs

Coordinates refer to `a63595a9`; the only initial working-copy change was the
shared ticket snapshot. Historical reports are evidence for mechanisms, not
claims that their old baseline costs remain.

| Area | Current source and verdict |
| --- | --- |
| Internal source leaf | `lib.rs:309–322`, `apply.rs:149–170,504–517`: the walker hands the actual plan into execution, which passes source metadata and literal link text to installation. **No second explicit source-leaf inspection here.** |
| Recursive new directory | `apply.rs:589–610`: source-only inspection, then the same batch installer. Source metadata is reused for final directory permissions. No target interior enumeration is invented. |
| Known leaf list | `1152–1269`: grouped parent-local staging, verified routing-prefix stack and permission handoff already exist. Per-leaf source metadata and destination absence checks enforce this API's contract. They are not leftover per-leaf ancestor walks. |
| Successful staging | `1302–1388,1452–1466`: one lazy arena per sibling batch; successful empty cleanup is a direct rmdir. Per-leaf tempdir creation and empty recursive cleanup from the historical baseline are gone. |
| Linux clone | `846–884`: source/output stay open across FICLONE, final metadata and rename. Fallback truncates output and reuses the descriptors. No source reopen or path-helper permission roundtrip to remove. |
| Ordinary copy | `857–858`: `io::copy(File, File)`. Its descriptor inspection/offload rules remain separate from the inspector's source attributes. Current trace sees the two descriptor metadata calls identified historically. |
| Output verification | `1283–1299`: set output mtime, read output metadata, require exact equality, then set source permissions. The read answers what the receiving filesystem represented, which source metadata cannot answer. Keep it. |
| Directory→leaf | `504–506,835,943–991`: read-only recursive precheck is discarded; after preparation, removal stats the root and inspects the subtree again. Confirmed below. |
| Directory leases | `457,569,624,948,1407–1426`: ordinary execution uses lazy but unverified leases, despite often having just observed the target directory. Replacement also creates a second lease for an already leased parent. |
| Public plan application | `34–146`: intentionally re-observes under its own binding/policy. This is not internal-run overhead. Preserve this boundary. |

The [historical leaf report](/design/performance/leaf0.sol61x.md#syscall-attribution-not-traced-wall-time-claims)
attributes the earlier wins to shared staging, held descriptors, successful-empty
cleanup and metadata handoff. All four have current code counterparts. Its large
fixture timings are neither reproduced nor extrapolated here.

### Useful source and descriptor observations

For the probe's one regular replacement file:

1. One listed source metadata acquisition supplies kind, source mode, mtime and
   byte accounting. A stat-key comparison against another regular file would also
   need target size/mtime; this target is a directory.
2. `io::copy` performs source and output descriptor metadata queries for its own
   transfer selection. These are real syscalls, but not an executor accidentally
   forgetting its `Metadata` input. Removing them requires owning the std copy
   backend's offload/fallback behavior, not merely passing a field further.
3. One output descriptor metadata query follows `set_modified`. It verifies the
   exact-mtime contract. Its removal would weaken semantics.

Successful Linux FICLONE does not use the `io::copy` pair of descriptor queries;
it still needs source attributes and output readback. Permitted fallback reaches
`io::copy` after clone failure and truncation, without opening a second source.
These clone/fallback statements are **source-derived**, not new runtime tests.
No known-length custom copy loop or clone-capability cache is selected.

## 2. Tiny replacement discriminator

Scratch: `.test-agent/transfer-kernels/`, with README, standalone manifest,
`probe.rs`, isolated `target/`, four traces and stdout results. Build:

```sh
CARGO_TARGET_DIR=.test-agent/transfer-kernels/target cargo build --offline \
  --manifest-path .test-agent/transfer-kernels/Cargo.toml
```

Configured compiler: `rustc 1.101.0-nightly (822182db4 2026-10-13)`; Linux,
btrfs, debug build, default listing backend, serial execution. Each case owns
fresh scratch roots, with far fewer than 256 entries and 1 MiB:

```text
source/victim                 regular file, 11 bytes, mode 0640
target/victim/                mode 0500
  a                           1 byte
  nested/                     mode 0500
    b                         1 byte
```

The second variant preserves `victim/nested/b` through pure early scope.
Actual runs call unchanged `Applier::run(Copy)` after binding. BEGIN/END stderr
markers delimit API activity in `strace -yy -e trace=%file,getdents64,write,fchmod`.
Setup, binding, verification and fixture retention are outside these counts.

| API-marker count | Actual removable | Actual retained | Retention model removable | Retention model retained |
| --- | ---: | ---: | ---: | ---: |
| `statx`, all | 20 | 7 | 3 | 2 |
| `openat`, all | 8 | 4 | 2 | 2 |
| `getdents64`, including EOF calls | 12 | 8 | 4 | 4 |
| staging mkdir | 1 | 0 | 0 | 0 |
| directory chmod | 2 | 0 | 0 | 0 |
| file fchmod | 1 | 0 | 0 | 0 |
| unlink / rmdir / rename | 2 / 3 / 1 | 0 / 0 / 0 | 0 / 0 / 0 | 0 / 0 / 0 |

**Model columns are not optimized sync runs.** They call the real public
target-only inspector, recursively reject retained descendants, and retain its
plans in a local vector. They do no preparation or removal. Their counts show
the precheck observation and retention cost only; proposed savings below follow
from matching those plans to the second actual call path.

### Attribution of the 20 actual metadata calls

| Purpose | Calls |
| --- | ---: |
| Run root dispatch + paired root binding | 3 |
| Paired containing-directory observation: source file, target victim directory | 2 |
| Precheck descendants: `a`, `nested`, `nested/b` | 3 |
| Removal re-observes those descendants | 3 |
| `remove_present` reclassifies victim root | 1 |
| Permission leases: outer target, victim, nested, second target-parent lease | 4 |
| Newly created private arena's actual permissions | 1 |
| `io::copy` source/output descriptor metadata | 2 |
| Exact output-mtime readback | 1 |
| **Total** | **20** |

Within the victim subtree, actual precheck and removal each open two directories
and issue four getdents calls (data plus EOF for each tiny directory), with three
admitted descendant stats each. Whole API enumeration additionally includes the
source and target roots. This distinction prevents attributing all twelve
getdents calls to replacement duplication.

The success assertion checks replacement contents, source-equal mode and exact
mtime. Progress reports two removed files and two removed directories. Both
restrictive directories are relaxed once and then removed, with no unnecessary
restoration chmod after successful removal. The retained case refuses before
staging, chmod or deletion, with zero completed effects. Both original leaf bytes
and both 0500 directory modes are asserted intact. Early preservation suppresses
the stat of `b`, explaining two precheck descendant stats rather than three.

### Retained memory and scope

The successful model keeps two actual plans with three operation slots. A
single-threaded counting `GlobalAlloc` around System measures live requested heap
bytes immediately after construction and before printing: **5,680 bytes** above
the precheck baseline; dropping the vector returns the delta to zero.
`size_of::<DirectoryPlan>() = 328`, `size_of::<Operation>() = 456` on this build.
The heap result includes vector capacity, real plan payloads and owned paths;
it excludes stack, allocator bookkeeping, RSS and transient allocation peaks.
It depends on these absolute scratch path lengths and current representation.
The retained variant returns no held plans and a zero final delta, not a claim
of zero transient allocations.

Keeping full plans is a useful first implementation, not a compact format. It
duplicates roots/paths and retains source-empty fact slots, diff projections and
vector spare capacity. Later lowering could retain just authorized removal kind,
name/coordinate and directory permissions, but must preserve ordering and report
semantics. Measure that only if full-plan retention is materially costly.

For `N` admitted descendants and `D` directories including the replaced root,
successful reuse avoids **D second listings and N second metadata acquisitions**,
plus the root classification read when the parent observation is handed in.
On this fixture: **2 listings, 3 descendant stats + 1 root stat**. It retains
O(N) local state until installation/removal finishes. Current DFS holds plans
along the active path (sum of active directory widths), also O(N) in the worst
case; the difference is retaining completed sibling subtrees until the whole
precheck succeeds, not a universal O(depth)→O(N) claim. Parallel sibling
replacements can multiply the live local retention. No whole-tree catalog is
needed. If a memory cap is later required, an explicit fallback to current
rewalking can bound retention, with separately tested deterministic behavior.

## 3. Ranked minimal implementation slices

### 1 — Thread directory permissions and reuse the owning lease

Start inside sync's recursive calls, where `Operation::target()` already carries
real target attributes. Pass those permissions into `reconcile_directory` and
construct `Writable::verified`; the existing seed implementation is precedent.
Pass the outer arena's parent lease into directory replacement removal instead
of `remove_present` constructing an independent `Writable::new(parent)`.

The tiny case has **two avoidable directory permission stats** (`victim`,
`nested`) and **one avoidable second parent-lease stat**. Sharing the lease also
avoids unnecessary lock/state construction. Keep one owner for restoration:
the inner operation borrows it and must not finish it. If the parent was already
temporarily relaxed, treating its observed effective mode as the original final
mode would be wrong; the original outer lease must outlive all joined children.

New-directory copying has another local opportunity: `copy_directory_selected`
calls `relax(to)` immediately after creation, then its batch's unverified lease
can stat `to` again on first mutation. Hand off the **effective relaxed target
permissions/lease**, while retaining source permissions separately for final
completion. Do not substitute source mode for post-umask target mode, and do not
skip acquisition of the newly created directory's actual initial permissions.
The private arena's own post-umask acquisition similarly remains justified.

Matched-directory walking needs a larger seam: `pair.rs:96–103` lowers children
to names and drops clean plans; `walk.rs:181–207` carries names rather than target
permissions. Full coverage therefore needs an execution-oriented child context
carrying optional target permissions through the walker/frontier, or another
explicit handoff. Do not retain every clean plan just for one mode value. This
is a follow-on to the sync-local slice, not a one-line universal fix. Savings are
one acquisition per activated lease with available attributes, zero for a lease
that remains dormant. Grouped seed already receives this benefit.

Verification: writable and 0500 parents with two sibling writes; retained/clean
directories with zero mutation; restrictive umask for newly created directories;
successful removal versus retained directory restoration; joined sibling error
with surviving parent restored. Preserve existing progress accounting and the
failure contract; failures promise no rollback or recovery.

### 2 — Consume the local replacement precheck

Change `removable` from `Result<()>` to a private owned precheck product whose
complete successful construction precedes staging. Execute those validated
target-only plans after preparing the replacement. Carry the already-observed
victim directory attributes and borrow the owning parent lease. Keep ordinary
target-only deletion streaming; it has no required preliminary whole-subtree
check and should not acquire one.

Do not replace precheck with “remove as you discover”: a retained descendant must
refuse **before any part of this coupled replacement is dismantled**. Source
preparation still precedes target removal, and staged rename remains necessary.
Holding plans supplies no new authority over preserved names or other regions.

On the successful fixture this slice alone saves four metadata calls and two
listings; permission slice 1 adds three nonoverlapping saved metadata calls.
Combined prediction: 20→13 metadata calls, still including root duplication,
outer permission reacquisition, actual private-arena mode, descriptor-copy work
and output verification. This is a **source-derived prediction**, not a measured
candidate sync result. Additional general root/permission handoffs could reduce
more, but must not be counted twice.

Ensure `planned`, `excluded`, `retained` and completed removal counters fold once.
The current precheck increments exclusion through `Context::plan`, but does not
fold ordinary planned work; execution does. Preserve current failure boundaries
and deterministic child order. Pure policy need not run twice, but compatibility
must not turn an omitted callback into lost authority or missing report evidence.
Preview (`465–485`) has the same precheck/reinspection shape and can share the
read-only product after the executor seam is established.

Verification: the exact two cases above, late retention and stop-descent blockers,
empty directories, early excluded unsupported entries, readonly modes, nested
retained ancestors, literal links and final exact mtime. A tiny instrumented
candidate must demonstrate the predicted second-pass disappearance and no new
mutation in the blocked case. No broad campaign is needed to settle this slice.

### 3 — Demand-driven mismatch link reads, owned by observation

`visitor.rs:1105–1109` still reads every source link and every Both target link.
Target link text on a regular-source/link-target mismatch serves neither baseline
comparison nor removal; one readlink per such slot is avoidable when policy does
not demand it. Source link text is needed for selected installation, but not for
a kind-only decision subsequently retained by policy. Full late visitor/Check
compatibility currently exposes those facts, so silently removing them is not a
kernel-local optimization. Use the completed
[fact-demand contract](/design/observation/facts0.gpt6am.md#3-smallest-demandacquisition-contract).
Target-only non-Audit readlink omission is already implemented.

### 4 — Small path and root handoffs, after the above

`execute_batch:459` constructs source/target child paths before action dispatch;
`reconcile_directory:629–630` even builds a synthetic source path for target-only
removal. This fixture constructs three such unused removal-source paths. Moving
source joins into actions that actually consume them removes these joins without
changing native name storage. Preserve logical paths for callbacks and errors,
and physical private paths for preparation. No allocator or timing savings for
this proposed microchange were measured. A wholesale fd-relative executor has
far more surface area; core listing metadata is already parent-relative here.

`whole_root` observes target and paired `Roots::new_checked` observes it again:
the trace confirms **one redundant dispatch/binding target stat per ordinary
run**. Internal bound-root handoff is plausible; public binding/plan validation
must remain explicit. Initial binding's ancestor checks/canonicalization and
initialize/replace checks are outside this probe's marked run. Do not advertise
a per-file win or drop these checks without an equivalent structural contract.

Staging still scans reserved basenames and creates a prefix/mutex even for some
noninstalling batches. That is userspace work, not extra tempdir syscalls; the
arena is already lazy. No measured evidence here justifies refactoring it.

## 4. Platforms and limits

Linux/default Unix admitted entry metadata normally queries the filesystem;
Linux native-dirent would still query full admitted attributes under today's
contract. On Windows, listing already caches metadata: removing the second
inspection saves enumeration and construction, but **N metadata calls is not N
additional stat syscalls saved there**. Directory `paths::directory` queries are
separate from listing cache reuse. Unix permission handling adds owner rwx and
restores mode; non-Unix handling clears/restores readonly, not Unix mode/ACL
equivalence. Windows link creation/removal also needs file-versus-directory-link
subtype. Retained metadata or a future compact removal record must preserve it.

Non-Linux clone uses the path helper (`apply.rs:886–904`), then adjusts private
output permissions, opens it and finalizes exact mtime/mode. Source shows a
remaining descriptor lifecycle difference; this study supplies no new native
Apple/Windows measurements and does not transplant Linux syscall savings there.

All results assume a good source and stable caller-controlled topology, with
concurrency/locking out of band. No repeated donor proofs, automatic pruning,
global catalogs, receipts, recovery guarantees or cleanup redesign are selected.
Only the model-suffixed research document is committed; scratch is local evidence.
Ticket notes carry the findings and session; parent owns integration and closure.

## Cross-references

- [Prior architecture: immediate inspection and authority](/doc/architecture/README.md#immediate-inspection-and-decision-authority): retaining the precheck preserves its whole-batch validation and coupled-replacement rule.
- [Completed fact study: re-queries and delivered savings](/design/observation/facts0.gpt6am.md#5-re-queries-and-already-delivered-savings): confirms its candidate double traversal and refines permission handoff into local leases versus walker-carried facts.
- [Historical inspection: ordinary-copy offload](/design/performance/inspection0.sol61x.md#e3): explains why descriptor queries are not forgotten leaf observations.
- [Historical leaf ownership results](/design/performance/leaf0.sol61x.md): establishes the predecessor measurements; current code determines what remains.
