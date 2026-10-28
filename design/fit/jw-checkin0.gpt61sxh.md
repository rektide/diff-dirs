---
type: ArchitecturalAssessment
title: Frequent JW reseed — library fit, costs and alternative shapes
description: Independent source-based assessment of diff-dirs 0.2 coordinated execution and jj-waltz adoption, separating shared acquisitions from route, policy, opaque-effect and control costs.
resource: /design/fit/jw-checkin0.gpt61sxh.md
status: independent-review
generated: { by: "model:openai/gpt-6.1-sol#xhigh", at: 2026-10-28 }
sources:
  - resource: opencode://ses_e7b37dd77ffeIu8mHUKf204c5B
    title: Independent library-side inspection of current acquisition, coordinated traversal, effect and JW consumer seams; architectural alternatives without implementation or runtime measurement
  - resource: opencode://ses_e7eb31c84ffev4GUeYjdmZO7mY
    title: Commission, supplied baselines and read-only host identity evidence; binary source generation deliberately unassigned
  - resource: /tree/observe/src/listing.rs
    title: Deferred no-follow metadata and backend handle ownership
  - resource: /tree/observe/src/facts.rs
    title: Cached early facts and portable Unknown behavior
  - resource: /tree/inspect/src/visitor.rs
    title: Actual admission, full-fact actions, conditional containers and plan validation
  - resource: /tree/inspect/src/shared.rs
    title: Source-once immediate acquisition, target unions and lowering
  - resource: /tree/inspect/src/shared_tests.rs
    title: Existing counted acquisition, identity, release and error-order assertions, inspected but not run
  - resource: /tree/compare/src/coordinated/mod.rs
    title: Group progression, policy checks and repeated route validation
  - resource: /tree/compare/src/coordinated/state.rs
    title: Owned partial frontier, membership and summary lifetimes
  - resource: /tree/compare/src/walk.rs
    title: Pair walker comparison, clean-batch treatment and separate parallel frontier
  - resource: /transfer/sync/src/coordinated/mod.rs
    title: Live plan reuse, applications barrier, binding, refresh and effect routes
  - resource: /transfer/sync/src/coordinated/decision.rs
    title: Metadata-free decisions and their equality contract
  - resource: /transfer/sync/src/coordinated/state.rs
    title: Saved applications and cumulative reporting contracts
  - resource: /transfer/sync/src/coordinated/tests.rs
    title: Existing ownership, restoration and opaque-effect assertions, inspected but not run
  - resource: /transfer/sync/src/apply.rs
    title: Opaque recursion, deferred routing, staging and per-install source opens
  - resource: /transfer/sync/src/lib.rs
    title: Direct streaming execution versus public application boundaries
  - resource: /transfer/sync/src/paths.rs
    title: Absolute ancestor checks and canonicalization
  - resource: /transfer/sync/tests/conditional.rs
    title: Existing conditional-action tests and restored opaque ownership assertion, inspected but not run
  - resource: file:///home/rektide/src/jj-waltz/transfer/workspace/src/streaming.rs
    title: Actual source-qualified visitor, pure policy queries and sequential one-lane coordinated driver
  - resource: file:///home/rektide/src/jj-waltz/transfer/workspace/src/ignored.rs
    title: Pattern replay and existing incremental selector
  - resource: file:///home/rektide/src/jj-waltz/transfer/workspace/src/frozen.rs
    title: Immutable ignore matching and separate legacy native oracle
  - resource: file:///home/rektide/src/jj-waltz/transfer/workspace/src/transfer.rs
    title: Existing pair-adapter execution and caller-owned optional pool
  - resource: file:///home/rektide/src/jj-waltz/transfer/workspace/Cargo.toml
    title: Exact 0.2 path dependencies and optional parallel integration
  - resource: file:///home/rektide/src/jj-waltz/refill/engine/src/engine/replacement.rs
    title: Callable strict N1 refill and native-to-ordinary boundary
  - resource: file:///home/rektide/src/jj-waltz/refill/engine/src/engine.rs
    title: Production reseed still invokes the legacy auxiliary pair adapter
  - resource: file:///home/rektide/src/jj-waltz/refill/engine/src/lib.rs
    title: Separate production entrypoint and hidden callable N1 export
  - resource: file:///home/rektide/src/jj-waltz/app/cli/src/cli.rs
    title: CLI run_reseed to run_with_diagnostics dispatch
  - resource: /design/architecture/rebuild0.gpt6a.md
    title: Earlier delivered rebuild account and deliberate opaque-operation checkpoint scope
  - resource: /design/transfer/multi-target0.gpt61sxh.md
    title: Earlier bounded-cohort alternative; not the delivered scheduler or current throughput evidence
verification:
  method: Targeted source inspection and existing document-checker binary only
  result: Document check passed (1 document, 28 links, 29 sources); no builds, tests, benchmarks, native operations or live reseeds performed; existing test assertions are attributed, not new test receipts
extensions:
  baselines:
    diff_dirs: "10361cf0; starting empty working copy 59f9466f; public packages 0.2.0"
    jj_waltz: "supplied code baseline 76fb0656; unfamiliar documentation changes in 4db9154e untouched"
  independence: Parent-supplied commission and host facts, targeted source and earlier research only; no peer same-wave reports read; no implementation, tickets or shared-index edits
---

# Frequent reseed: a useful substrate, not yet a demonstrated fast path

## Judgment and adoption boundary

**Keep the separation of physical facts, target policy and filesystem effects;
reassess the execution shape around it.** The library has genuine source sharing
and unusually useful preservation/replacement semantics. It does not yet establish
that the coordinated cursor is the cheapest frequent-reseed driver. Its strongest
reuse covers already-paired directory frontiers; its heaviest source-only work
remains destination-owned opaque recursion. That distinction can reverse which
architecture looks attractive on an actual reseed population.

At the inspected baselines, JW's **callable N1** `refill_one_from` performs native
work, then `streaming::sync_one`; that constructs one `SyncTarget` and uses
`Execution::Sequential`. It rejects multi-receiver requests, preparation and
dry-run. The production CLI instead reaches `engine::run`, whose ignored-payload
stage still uses `Auxiliary::legacy` and `Execution::run` over the pair adapter.
Exact 0.2 path dependencies do not make these the same execution path. Neither
establishes real N2 shared-source savings. [E8–E10]

Parent-supplied host evidence identifies installed `jw` 0.8.0, identical to
`target/release/jw` (SHA-256
`00ae8cb62630f1ccdfd9a27ee36f35708367daed44ced305908a31d2ec5a5652`),
installed from `app/cli` in release mode without optional features, using configured
Rust 1.101 nightly `822182db4`. Native JJ reports
`0.45.1-33d3a595429b7e3b54a8ea769560247913bc8a1b`. Installed metadata does not identify
the JW source commit: **checkout behavior is not asserted to be installed-binary
behavior**. Older pair-adapter timings cannot rank this coordinated route.

## What serves reseed well

The early/late split puts policy at the useful point: before child metadata when
names/hints suffice, after full observation when device/type or collision authority
matters. Preservation is pair-wide and cannot manufacture source absence. Equal
entries remain visible to late policy. These are not incidental abstractions: they
let JW exclude administration and local content without a parallel policy walker.

The 0.2 `Container::CreateIfNeeded` addition solves a real integration mismatch.
An absent routing directory can be searched without leaving unmatched skeletons;
selected empty directories still use eager creation. Pending routing ancestry
materializes on admitted descendant work and finalizes permissions postorder.
This is genuine single-pass execution, not a discovery preaudit. [E1, E5]

Comparison and effects have one recursive owner each: coordinated comparison owns
paired edges; production sync owns source-only, target-only and collision interiors.
Coupled replacements, lazy staging and permission leases protect important behavior
that a faster ad hoc copier would have to reproduce. Facts are reused internally
without routing every plan through the expensive public revalidation boundary.
Those properties deserve preservation even if the cursor changes.

## Where the current shape charges work

### Shared facts are real, but narrower than shared transfer

`SharedRoots::inspect_ready` lists/sorts the source once, builds each target's own
union and early admission, then acquires the union of demanded source facts once.
The same `Arc<Facts>` reaches participating policies. Excluded source handles are
dropped without metadata; a source error affects only demanding lanes. [E2]

It still lists every ready target, repeats union construction and late policy per
target, and creates target-specific operations. Sharing does not share source file
opens or bodies: `prepare_file` opens source and output for each installation, then
clones/copies, finalizes and installs each independent output. Required destination
effects remain N effects.

More importantly, `paired_children` excludes one-sided directories and collisions.
If N destinations lack the same large dependency subtree, each application calls
its own `Context::plan` recursively through that source subtree. If A lacks it but
B already has it, A's opaque source traversal and B's later paired traversal also
observe it separately. A newly created target directory deliberately does not join
the previously accepted paired frontier. **Shared root acquisition is therefore
not shared source acquisition throughout a cold refill.** [E3, E5, E11]

### Route admission is another observation budget

`Coordinated::validate_route` checks source/target roots and every relative
ancestor for each group. Its `Routes` map shares repeated source checks within that
group, not across groups. `SyncCursor::apply` additionally calls `route` for each
target: absolute root ancestors, canonicalization and relative directory ancestors
on both sides. The shared source is checked again for each application, including
ordinary clean groups. Binding adds its own root/identity checks. [E3, E4, E6]

Consequently source-acquisition counts omit important source-side work. The
`route_metadata_checks` diagnostic also explicitly omits binding observations and
does not count sync's separate application routes. Deep, narrow, warm trees can
make this repeated ancestry work important; its actual time remains unmeasured.
Removing checks requires an explicit stability/authority contract, not treating a
previous stat as permanent confinement. These APIs are not hostile-writer sandboxes.

### Full-fact policy is expressive, not demand-minimal

After early admission, the normal visitor receives complete admitted metadata and
required link text, including equal entries. Early policy can preserve a whole
pair or request ordinary observation; it cannot ask merely for authoritative kind,
then conditionally request attributes or target facts. Cheap source hints help,
but default std Unix listing supplies `Unknown` kind; JW's inspected manifests do
not enable `native-dirent`. Unknown routing candidates must reach late facts.

This is a coherent compatibility contract, not a bug. A selective interface could
save work for source-qualified discovery or explicit known inputs, but cannot
silently weaken existing full-fact callbacks or infer deletion authority from an
unrequested/incomplete source listing. [E1, E2]

### No filesystem I/O in policy does not mean cheap policy

JW's frozen in-process ignore matcher avoids native queries during callbacks.
However, `SourceQualified::candidate` rechecks successive path prefixes, each
`Patterns::matches` restarts the component automaton, and `FrozenIgnore::ignored`
searches ancestors and applicable rule directories. Early and late repeat
disposition work. On deep unselected routes this repeats prefix processing; source
fact sharing does not share these target-specific computations. [E8]

Start improvement here in JW: immutable query indexes, directory-scoped memoization
or carried compiled prefixes, without resurrecting a payload census. The existing
incremental `Selector` demonstrates a different query shape, not a drop-in policy
replacement. A library context hook earns its place only if consumer-local query
improvements cannot express the required reuse safely under repeated callbacks.

### Group and state lifetime are substantial choices

Comparison releases immediate facts after reduction unless lowering retains them.
Sync deliberately retains full plans for all current successful nonstructural
lanes, plus metadata-free `DirectoryDecision`s containing **every** admitted entry
and excluded name. Decisions are also returned in the public group. Clean normal
groups still become unstarted applications; unlike the pair walker, they are not
silently drained. [E3, E4, E7]

This makes precise pause/rebind behavior available, but gives a process-local N1
consumer decision projection, cloning, application routing and dispatch even when
no writes are needed. Exported state has no metadata or handles; runtime plans do.
`parts()` clones frontier/decisions, whereas `into_parts()` moves them. There is no
serialization cost unless a consumer serializes, but semantic snapshot preparation
already occurs during ordinary execution.

Memory is directory-local, **not bounded independently of width or target count**:
all prepared target unions coexist before shared acquisition; source facts are
shared but target operations/decisions scale with admitted names per lane. Active
DFS frames retain whole child suffixes and memberships. Changed-directory summaries
can grow with the traversed tree. Opaque recursion retains ancestor plans while
descending. Group/application limits bound none of these bytes. [E2–E5, E7]

### Concurrency and backpressure are not interchangeable

Coordinated traversal has sequential inter-group progression. Shared inspection
parallelizes sufficiently wide metadata vectors (threshold 64), but target listing
and early preparation are serial, followed by source acquisition and then each
target's metadata/finalization phase. `drive` applies targets serially; `apply`
borrows the cursor mutably. Effect batches can use Rayon internally, but conditional
routing chains and target-only reconciliation have sequential recursive interiors.
This is not pair-frontier J/K scheduling. JW's callable N1 selects sequential
execution regardless. [E2–E5, E9]

The current barrier prevents another comparison group until all current
applications are disposed. Thus an opaque copy for A delays B's otherwise-ready
paired descent. A returned failure can terminate A and allow healthy lanes onward;
a stalled call cannot be detached mid-flight. Lane heads are independently
represented, not independently scheduled. Wide-group memory retention and
head-of-line blocking are the price of this simple joined boundary.

Large cohorts also encounter repeated linear identity searches: runtime/policy
validation scans bindings, and application refresh revalidates runtime identity
even when plans are cached. Their nested scans warrant indexing if large N becomes
real; small N may make them irrelevant. No current timing ranks these CPU costs.

## Viable futures, not one compulsory framework

**1. A lighter live execution view is the smallest credible library evolution.**
Keep full inspection/validation, paired traversal and the production effect kernel;
avoid eagerly producing portable decisions for consumers that never checkpoint.
Use direct current-plan authority, with explicit checkpoint export or a separate
nonpersistent driver. Auto-dispose genuinely effect-free groups, not merely
`coverage.planned == 0`: an opaque kept target-only directory can still remove
descendants, as an existing test explicitly demonstrates. Preserve useful control
windows where requested. Validate with JW's real sequential N1 warm/no-op route.

**2. Share source facts lazily at the acquisition owner.** An internal or narrow
source-epoch interface could separate complete source names, demand acquisition
and target finalization, avoiding repeated acquisition without requiring a global
catalog. Sharing scope must expire; target policy/errors stay independent. A
source-led selected-region request might avoid irrelevant target listing, but must
declare preservation outside its domain rather than pretend it performed a full
mirror assessment. Validate with JW's sparse wildcard selection and differing
desired ignore/native boundaries, not `NoVisitor` alone.

**3. Source-to-many opaque work is the biggest structural sharing opportunity.**
Let ready source-only/replacement deliveries consume common source batches while
retaining independent target container, staging and postorder obligations. This
belongs primarily in transfer execution, not by redefining read-only paired
comparison. It changes mutation/failure grouping and retained state; do not label
it a trivial extension of `SharedRoots`. Required-clone installations remain
separate. Shared source descriptors are optional, with explicit fallback/offset
semantics—not a prerequisite for shared facts. Validate two actual receivers
missing the same selected subtree, then the mixed missing/paired case.

**4. Match only common ready frontiers if barriers prove costly.** Independent
destination progression with bounded opportunistic cohorts can trade reacquisition
for progress and memory release. The earlier [source-cohort study](/design/transfer/multi-target0.gpt61sxh.md)
explores this alternative; it is not implemented scheduling evidence. Retain one
authoritative continuation per destination, not a second walker hidden in a cache.
Existing `stop_lanes` abandons future obligations; it is not suspension. A slow
receiver must not pin arbitrarily many source batches. First validate an actual
heterogeneous N2 consumer; identical synchronous receivers may favor today's
simpler grouping.

For route overhead, a runtime-bound effect capability could reuse shared source
route admission within a joined epoch while checking each target at consumption.
On restore, executable policy binding and fresh routes remain mandatory;
unstarted applications refresh decisions, while accepted parents are not replayed.
Current `DirectoryDecision` equality checks semantic decisions, not old metadata,
inode identity or exact bytes. Do not turn it into a historical content certificate.
The public `Applier::apply_plan` intentionally reobserves under its own binding;
its cost should not be imposed on an internally authorized live plan.

JW should continue owning native S/D membership, source qualification, ignore
inputs, workspace readiness and sequencing. Known tracked-leaf installation need
not discover a directory union just to resemble ordinary transfer. Conversely,
the library should own safe source-batch reuse, container effects and portable
continuation semantics. Changing JW to make all receivers ready together is an
architectural option, not an automatic prerequisite: it must justify its native
sequencing and failure consequences independently. Neither side is finished.

## Next discriminating evidence

Measure matched contracts, separating physical acquisition, route validation,
policy CPU, decision allocation and effect work. Record group widths, live plan
bytes/high-water RSS, source-only interior size and receiver waiting—not just
source-list counts.

| Real consumer case | Choice it discriminates |
| --- | --- |
| Current N1, repeated clean payload and deep narrow routes | Lightweight execution and route/policy costs versus useful checkpoint/control machinery |
| Two ready receivers, same populated paired payload | Actual coordinated source reuse versus independent pair execution |
| Both missing a large selected subtree; then one missing/one paired | Whether shared opaque work matters more than shared comparison |
| Sparse wildcard selection through absent unselected routes | Deferred containers, demand shape and query-index costs |
| One slow receiver, wide directory and explicit pause | Barrier simplicity versus independent cohorts; memory and refresh charge |

No build/test/benchmark was run here. Existing tests supply source-visible
assertions, not new passing receipts or speed evidence: `shared_tests` asserts
single acquisitions, shared pointer identity, release and ordered failures;
coordinated sync tests assert barrier restoration and exactly one opaque recursive
owner; conditional tests cover no-skeleton behavior and saved conditional actions.
These establish what must survive a redesign, not which redesign will win.

## Evidence map

All library rows refer to **10361cf0**. JW rows refer to supplied code baseline
**76fb0656**, inspected in its checkout; unrelated uncommitted documentation was
not read or changed. Paths/symbols, rather than old README descriptions, ground
the assessment.

| ID | Exact source and symbols |
| --- | --- |
| E1 | [observe/listing.rs](/tree/observe/src/listing.rs): `Entry`, backend `read`, `metadata`; [observe/facts.rs](/tree/observe/src/facts.rs): `EarlyFacts`; [inspect/visitor.rs](/tree/inspect/src/visitor.rs): `Admission`, `Container`, `Operation::validate`, `paired_children`, `inspect_listed` |
| E2 | [inspect/shared.rs](/tree/inspect/src/shared.rs): `SharedRoots::inspect_ready`, `union`, `indexed_map`; [shared_tests.rs](/tree/inspect/src/shared_tests.rs): `source_once_facts_are_shared_and_released_after_reduced_publication`, `caller_selected_full_plans_share_source_facts_until_the_caller_lowers_them` |
| E3 | [compare/coordinated/mod.rs](/tree/compare/src/coordinated/mod.rs): `next_group_with`, `validate_route`, `Routes`, `settle_returns`, `stop_lanes`; [state.rs](/tree/compare/src/coordinated/state.rs): `Parts`, `GroupFrame`, `ChildEdge` |
| E4 | [sync/coordinated/mod.rs](/transfer/sync/src/coordinated/mod.rs): `SyncCursor::next_group`, `apply`, `refresh`, `route`, `drive`, `validate_runtime`; `SyncRuntime::with_policies` |
| E5 | [sync/apply.rs](/transfer/sync/src/apply.rs): `Context::plan`, `execute_batch`, `copy_directory_pending`, `reconcile_directory`, `prepare_file`, `Staging`, `Writable` |
| E6 | [sync/paths.rs](/transfer/sync/src/paths.rs): `root_with_leaf`, `directory`; [sync/apply.rs](/transfer/sync/src/apply.rs): `apply_public`, public `apply_plan`, internal `apply_plan_with` |
| E7 | [coordinated/decision.rs](/transfer/sync/src/coordinated/decision.rs): `DirectoryDecision::of`; [coordinated/state.rs](/transfer/sync/src/coordinated/state.rs): `Barrier`, `SyncParts`; [compare/walk.rs](/tree/compare/src/walk.rs): `PlanWalk::advance`; [sync/lib.rs](/transfer/sync/src/lib.rs): `Applier::run`, `run_in_pool` |
| E8 | JW [streaming.rs](file:///home/rektide/src/jj-waltz/transfer/workspace/src/streaming.rs): `SourceQualified::{candidate,disposition,early,late}`; [ignored.rs](file:///home/rektide/src/jj-waltz/transfer/workspace/src/ignored.rs): `Patterns::path_prefix`, `Selector`; [frozen.rs](file:///home/rektide/src/jj-waltz/transfer/workspace/src/frozen.rs): `FrozenIgnore::{ignored,exact}`, `FrozenIgnores` |
| E9 | JW [replacement.rs](file:///home/rektide/src/jj-waltz/refill/engine/src/engine/replacement.rs): `refill_one_from`, lines 47–55, 95–97, 200–218, 262–267; [streaming.rs](file:///home/rektide/src/jj-waltz/transfer/workspace/src/streaming.rs): `sync_one`, lines 181–217 |
| E10 | JW [cli.rs](file:///home/rektide/src/jj-waltz/app/cli/src/cli.rs): `run_reseed`, `run_refill`; [engine/lib.rs](file:///home/rektide/src/jj-waltz/refill/engine/src/lib.rs): `run_internal`, N1 export; [engine.rs](file:///home/rektide/src/jj-waltz/refill/engine/src/engine.rs): lines 1575–1591; [transfer.rs](file:///home/rektide/src/jj-waltz/transfer/workspace/src/transfer.rs): `Execution::run`; [Cargo.toml](file:///home/rektide/src/jj-waltz/transfer/workspace/Cargo.toml): exact dependencies/features |
| E11 | [coordinated/tests.rs](/transfer/sync/src/coordinated/tests.rs): `opaque_replacement_does_not_join_later_paired_traversal_after_becoming_a_directory`, `zero_work_count_still_reconciles_opaque_kept_target_container`, `barrier_saves_a_before_b_without_reapplying_or_replaying_root_policy`; [conditional.rs](/transfer/sync/tests/conditional.rs): `saved_current_group_refresh_validates_conditional_action_without_new_paired_membership` |
