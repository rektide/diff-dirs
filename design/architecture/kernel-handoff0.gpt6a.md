---
type: ImplementationHandoff
title: Contextual effect-kernel seams
description: Narrow implementation contract for separating effect primitives from contextual recursion while preserving grouped opaque and desired-owned execution.
resource: /design/architecture/kernel-handoff0.gpt6a.md
status: proposed-implementation-handoff
generated: { by: "model:openai/gpt-6-astra#xhigh", at: 2026-10-29 }
sources:
  - resource: "opencode://ses_e79da1717ffeKJmKkEcTiGUEZa"
    title: Inspects the delivered foundation and existing effects, resolving private kernel ownership, lifecycle, fact-demand and authority seams for the next migration
  - resource: "opencode://ses_e7a2771c3ffez3sv1oPgD5gPH4"
    title: Parent commissions this targeted handoff before contextual execution migration and retains architectural review
  - resource: /design/architecture/execution0.gpt6a.md
    title: Accepted provider scope and semantics; this handoff specifies its private effect seams
  - resource: /design/architecture/execution0.sol61x.md
    title: Foundation implementation account at fb4719cc and 48d50af0
  - resource: /transfer/sync/src/apply.rs
    revision: 48d50af0
    title: Existing recursive executor, staged installation, conditional ancestry, permissions and returned-progress implementation
  - resource: /transfer/sync/src/coordinated/mod.rs
    revision: 48d50af0
    title: Current binding, pending refresh, route checking, application and lane disposition
  - resource: /transfer/sync/src/visitor.rs
    title: Compatibility steering order and overrideable late presets
  - resource: /transfer/sync/src/model.rs
    title: Existing progress, original-cause failures and ordered merging
  - resource: /transfer/sync/src/platform.rs
    title: Platform-dependent symlink creation/removal fact requirements
  - resource: /tree/inspect/src/prepared.rs
    title: Runtime preparation, logical binding and narrow full-fact adapter
  - resource: /tree/inspect/src/selection.rs
    title: Existing semantic authority, declared input, interior and coverage contracts
  - resource: /tree/inspect/src/policy.rs
    title: Contextual callbacks and pure ExecutionDemand interface
  - resource: /tree/inspect/src/acquire.rs
    title: Acquisition-cell ownership and masked sparse facts
  - resource: /tree/inspect/src/shared.rs
    title: Actual inspection pipeline, source session and presently empty route epoch
  - resource: /tree/compare/src/coordinated/mod.rs
    title: Paired recursion ownership and current call-local route cache
  - resource: "file:///home/rektide/src/jj-waltz/design/coordinated-refill/execution0.gpt6a.md#43-replacement-ownership-program"
    title: Explicitly authorized same-wave native ownership, complete desired coverage and intact-collapse obligations
verification:
  method: Source inspection and explicit Markdown/provenance checking
  scope: Design handoff against the foundation checkpoint; no production edits, runtime tests or performance evidence
---

# Contextual effect-kernel seams

**Extract the physical primitives while migrating contextual execution now.**
Leave grouped scheduling and manifest enumeration to their subsequent slices.
The existing selection model is sufficient; the necessary addition is a private
runtime boundary between inspection/recursive control and physical effects.

This supplements the accepted [provider execution design](execution0.gpt6a.md),
not its scope or selected behavior. The [foundation account](execution0.sol61x.md)
correctly identifies the remaining gap: `apply.rs::Context` still combines full
facts, policy callbacks, recursion, stopping, permissions and reporting.

## 1. Keep the delivered contracts

Use these directly:

- `PreparedDirectory<C>::selection()/facts(SlotId)` and the same selected Arc in
  group/barrier/preparation. `DirectoryPlan` and `PreparedOperation` are unit-context
  full-visitor projections, not the contextual kernel's input.
- `EntrySelection::{action,install,removal,interior,desired}` and the common
  validator. No new action enum, copied decision body or sync legality table.
- `Interior<C>` for selected child context/input; `SourceSession` for source
  generation/resources; `ExecutionDemand` for finite pre-effect acquisition.
- Existing `Progress`, `Failure`, original causes and ordered error merging.
  Manifest coverage remains an additional listed-runner result, not a replacement
  for actual effects.

The relevant implementations are [prepared](../../tree/inspect/src/prepared.rs),
[selection](../../tree/inspect/src/selection.rs),
[policy](../../tree/inspect/src/policy.rs) and
[acquisition](../../tree/inspect/src/acquire.rs). Full compatibility continues
through `FullVisitor<Steering<...>>`, preserving [Steering's order](/transfer/sync/src/visitor.rs).

## 2. Runtime ownership and coordinates

The following are **private signatures/shapes**, not new persisted/public models.
Names can vary while these ownership distinctions remain:

```rust,ignore
struct EffectCall<'a> {
    source: &'a SourceSession,
    epoch: InspectionEpoch,             // one externally returned call
    execution: Execution,
}
struct LaneCall<'a, P: Policy, M: Comparator> {
    binding: &'a EffectBinding,          // physical roots + CopyPolicy
    policy: &'a P,
    comparator: &'a M,
    stopped: AtomicBool,                 // this lane only
    // encountered-policy counters; actual effects merge through local results
}
struct BatchPlace<'a> {
    relative: &'a Path,                  // final logical coordinate
    source_directory: Option<&'a Path>,
    observed_target_directory: Option<&'a Path>,
    output_directory: &'a Path,          // may be pending or privately staged
}
enum RecursionOwner { PairedFrontier, ThisCall }

fn apply_ready<P: Policy, M: Comparator>(
    call: &EffectCall<'_>,
    lane: &LaneCall<'_, P, M>,
    prepared: PreparedDirectory<P::Context>,
    place: BatchPlace<'_>,
    owner: RecursionOwner,
) -> Result<Progress, Failure>;
```

`apply_ready` belongs to recursive orchestration (`opaque.rs` or its initial
private equivalent). `kernel.rs` has no `Policy`/`Comparator` callbacks and does
not enumerate user trees. Its only recursive removal is cleanup of privately
owned staging. The public `apply.rs` boundary binds, prepares/refreshes, invokes
orchestration and forms reports.

The source session outlives individual calls, including paired work and later
opaque applications. `EffectCall` does not construct another source session per
batch. A whole convenience `run` owns one source session/resource handle; a cursor
retains the corresponding runtime handle across returned calls, never in Parts.

**Three paths must remain distinct.** Policies/comparison diagnostics use final
roots plus `relative`; source I/O uses the admitted source route; writes use the
physical output path. A replacement preparation keeps its final logical target
root even when its output is an arena slot. Its source-only child inspection has
`observed_target_directory: None`: the old final leaf is not a target directory,
and already-created private children do not become a paired target input. Never
recover logical coordinates with `strip_prefix(target_root)` on a staging path.

`BatchPlace` is constructed alongside the internal inspection request. Current
`PreparedDirectory::LiveBinding` records logical roots/generation, **not** the
actual request routes. Consequently a publicly obtained preparation is not enough
to authorize an arbitrary path mapping. Public application freshly prepares under
its own binding; only internal orchestration can supply the mapping above. No
new public constructor accepting a selection plus metadata is needed.

At the external boundary, match policy/context-schema/comparator/CopyPolicy and
validate saved contexts against rebound inputs before route I/O. Refresh all
unstarted saved applications before any selected subset resumes effects, comparing
entire selections by value, including grants and child contexts. Accepted lanes
and ancestors are not refreshed. The private effect gate does not replace this
existing coordinated barrier contract.

### One recursion owner

| Input/interior | Owner |
| --- | --- |
| Physical `Paired`, entered from comparison | Compare promotes basename plus selected context; application does not enter it. |
| Physical source-only, target-only or either replacement | This opaque call owns its entire interior, including any later paired shapes. |
| Listed routing, including two real directories | Listed call owns all declared descendants. No compare edge. |
| Preview | Read-only recursion under the same selected context/input rules; no effect scope is opened. |

Use `InteriorRole`, not `source.metadata().is_dir()` or a newly created directory,
to decide this. `ApplicationNeed` controls whether application is needed; zero
`planned` is not a skip condition. Structural-root entry handling uses its
selected interior context; admitted listings use their saved input context without
replaying root admission.

Promote only child basenames, `Interior<C>`, physical location, needed source
permissions and finish obligations before dropping an immediate batch. A runtime
child/finish frame may own these values; it must not pin the whole selection/fact
arena. This also gives a later cohort scheduler movable jobs grouped by source
generation, logical source coordinate and compatible input slice. It can share
inspection while retaining different lane contexts and destinations.

## 3. Sparse execution gate

Implement one `PhysicalExecutionDemand: ExecutionDemand` in sync. The current
trait signature in `policy.rs` already suffices. Feed it to every effectful
immediate inspection, including fresh pending refresh and opaque recursion:

| Selected effect | Required additional entry facts |
| --- | --- |
| Install regular leaf, including collapse | Source attributes for copying, exact mtime, permissions and logical byte count. |
| Install symlink | Source literal link; source attributes only where platform subtype needs them (Windows). |
| Create/CreateIfNeeded/prepare directory | Source attributes; retain only donor permissions for postorder finish. |
| Remove regular leaf | Resolved target kind. |
| Remove symlink, including old leaf during expansion | Resolved target kind; target attributes on Windows for directory-link removal. |
| Keep/preserve/ConfirmAbsent; retained or removable directory container | No extra entry attributes merely for the action. Actual write-lease acquisition is separate and lazy. |

Policy/guard demands can require more. Target link text is not an unlink input.
Change the small platform helpers to accept resolved kind plus their real
platform-required facts; Unix should not require `Metadata` just to satisfy an
old helper signature.

Before opening **any** effect scope, validate the entire prepared batch's required
facts, using the same demand table:

```rust,ignore
fn require_execution_inputs<C>(batch: &PreparedDirectory<C>) -> Result<(), Failure>;
```

The private binding gate also checks roots/generation/relative coordinate and
domain authority. `PreparedDirectory` can legitimately have been produced with
`NoExecutionDemand`; its existence alone is not proof of sync readiness. An absent
required fact is an integration refusal, never a fallback `stat`/`read_link` in
`copy_leaf_with`. All acquisition finishes before the immediate batch's first
effect, preserving the later-sibling-acquisition-failure boundary.

Thread the demand through **initial compare publication**, not just `apply`:
today `SharedRoots::inspect_lowered_using` hardcodes `NoExecutionDemand`, and its
lowering callback runs after acquisition cells have done their work. Give the new
contextual traversal seam an E parameter; comparison-only wrappers supply the
empty demand. The readiness check also performs fallible required-field extraction
such as regular-source `modified()`, without creating another facts model.

Operate on borrowed `EntrySelectionRef<C>` and `EntryFacts`. Small helpers extracting
required metadata/link references suffice. Do not build synthetic full `Facts` or
a parallel lowered operation vector. Runtime finish frames copy permissions;
source metadata already acquired by inspection is reused by leaf preparation.
Output exact-mtime verification remains a genuine output operation.

Current `shared.rs::assess_admitted` already finalizes selections then acquires
execution inputs. Current `AcquisitionCell` retains metadata obtained during kind
resolution. Preserve those properties while wiring the table through traversal.
Inspection-only preview/removability uses its declared policy/guard needs; its
second, effectful removal pass obtains selected execution inputs normally.

## 4. Kernel primitives and directory lifetime

Extract/reuse the bodies below from [apply.rs](/transfer/sync/src/apply.rs):

| Primitive | Existing body / required separation |
| --- | --- |
| Prepare private leaf | `prepare_file`, `preserve_file`, symlink portion of `copy_leaf_with`; retain source/output descriptors through publication. |
| Publish prepared node | Rename portion of `copy_leaf_with` / `ReplaceWithDirectory`; takes a privately bound destination, never arbitrary overwrite permission. |
| Remove one supported leaf | `remove_leaf` + platform helper, using required sparse facts. |
| Open/finish containing directory | `Writable`, `relax_with`, `finish_permissions`, lazy `Staging`; combine their ownership below. |
| Create/finish owned container | Creation/postorder portions of `copy_directory_selected` and `PendingDirectory`; no recursive policy calls. |
| Remove empty container | Tail of `reconcile_directory`, after its runner establishes no retained descendants. |
| Clean private arena | `cleanup` and private `remove`; no policy recursion or target-effect credit. |

One containing physical directory owns **one write lease** until all its started
children and staging cleanup join. An arena borrows that lease; it must not open
an independent lease that remembers temporarily relaxed permissions as the original
mode. A concrete private interface is:

```rust,ignore
struct DirectoryScope { /* physical path, lazy write lease, lazy arena */ }
impl DirectoryScope {
    fn ensure_writable(&self, epoch: &InspectionEpoch) -> Result<(), Failure>;
    fn stage_slot(&self, slot: SlotId, epoch: &InspectionEpoch)
        -> Result<StageSlot, Failure>;
    fn finish(self, result: Result<Progress, Failure>, end: DirectoryEnd,
              epoch: &InspectionEpoch)
        -> Result<Progress, Failure>;
}
enum DirectoryEnd {
    Retained,                    // restore original mode if relaxed
    Removed,                     // only after successful remove_dir
    Created { donor: Permissions }, // postorder donor mode, even on child failure
}
```

Constructing the scope or reserving candidate names performs no I/O. Stage slots
are exclusive, parent-local and protect all participating names as today's arena
does. Their paths are private runtime values. Cleanup finishes before restoration.
An unsuccessful directory removal uses `Retained`, not `Removed`. Finish errors
merge with the earlier result and never hide successful operations.

For new containers, record creation ownership/progress **immediately after mkdir**,
before fallible post-umask metadata or relaxation. Observe actual created permissions;
donor permissions cannot stand in for those facts. Ensure failure during scope setup
still reaches postorder finalization and reports that creation. Existing containing
permissions can enter the lease from a current route observation; clean/retained
batches never call `ensure_writable`.

Conditional ancestry remains a depth-sized lane-local stack. Push pending output
path, donor finish permissions and selected context without mkdir/chmod. A selected
ordinary effect, including an explicitly selected empty directory, marks the active
chain needed and materializes it top-down. Count each actually created directory
once while unwinding; finish every created ancestor on failure. An unused conditional
chain leaves no skeleton. Nested source-only inspection works while output is absent.

## 5. Publication, progress and failures

The minimum leaf seam can retain these concrete forms:

```rust,ignore
fn prepare_leaf(source: &Path, facts: SideFacts<'_>, slot: &StageSlot,
                policy: CopyPolicy) -> Result<PreparedLeaf, Failure>;
fn publish_leaf(prepared: PreparedLeaf, destination: &BoundDestination,
                epoch: &InspectionEpoch) -> Result<Progress, Failure>;
```

`BoundDestination` is a **private borrowed/runtime check**, derived from the original
selected entry, effect binding and actual manifest binding when relevant. It records
the selected destination shape/grant and whether this very coupled transition has
successfully cleared the old node. It is not another saved action model or public
capability. Publication enforces its placement condition: `AbsentOnly` cannot
overwrite a newly encountered collision. A fresh no-follow placement check may
refuse unexpected change; it must not reclassify/retry under a wider grant. This
retains the stable-tree contract, not a hostile-concurrency transaction promise.

Directory publication uses the same rename/placement primitive after its owned
tree is fully prepared and finished. Keep current transition order:

- Ordinary leaf renewal: prepare fresh private inode/link, then replace by rename.
  No old-leaf removal counter for an overwrite-by-rename.
- Leaf→directory: complete private selected tree; remove old leaf; rename tree.
  Failure after unlink reports the real removal.
- Directory→leaf: read-only removability preflight; prepare leaf; scoped recursive
  removal; rename. Failure after removal reports all actual removals.

Use existing `Result<Progress, Failure>` **locally**. A private tree's accumulator
is provisional until its outer rename succeeds. On discarded preparation, clear
only that private result's completed effects, retaining its primary/secondary
errors and encountered policy counters. Merge visible siblings and containing-parent
lease effects outside that boundary. Never zero a whole lane's already-aggregated
failure. Nested staging promotes into its enclosing accumulator, which may itself
still be provisional.

The listed runner similarly holds staged desired coverage provisionally. It credits
desired installation only on final publication. Actual old-owned removals/confirmed
absence can retire obligations even when a later installation fails; coverage must
remain incomplete. Kernel returns effect outcomes; the listed runner knows which
exact declared obligation they fulfill and owns the once-only accounting.

Keep `Failure`'s original cause and deterministic native-name error merging.
Cleanup/restoration become secondary errors after an existing failure, primary
when they alone fail. Each lane owns its stop flag and accumulators. A grouped
runner joins started work, closes failed-lane scopes and continues healthy lanes;
the kernel does not decide cohort policy. Replace `Staging::finish`'s dependency
on generic `Context` with explicit arena completion/cancellation state.

## 6. Intact refusal and listed binding

Move `removable`, `reconcile_directory` and recursive action dispatch into the
runner. A successful read-only preflight may yield a private operation-local
`RemovalReady` value tied to this coordinate, selected `Interior<C>` and bound
authority. It is neither persisted nor an unconditional recursive-delete permit.

For **both** passes, start with the exact `RemoveForReplacement` interior context;
each recursive step uses its newly selected child's context. Keep final logical
coordinates during preparation, preview and removal. A preserved/excluded child
vetoes the whole coupled replacement before its staging allocation, permission
relaxation or descendant dismantling. Ordinary target-only removal remains
different: removable siblings may complete while retained descendants keep their
necessary ancestors. Preserve the accepted bounded-memory two-pass protocol.

The future listed runner lends the same kernel an actual bound desired/owned input:

1. Match manifest IDs to supplied immutable indexes, then validate each leaf/node
   reference, exact coordinate/expectation, selected action and authority relation.
   Digest equality or generic `validate_entry` alone is insufficient.
2. `CreateAbsent` has no occupied-name/removal authority even if an O set was
   supplied. `RenewOwned` requires exact O leaf ownership for leaf overwrites and
   expansion; an old O leaf now holding a directory refuses.
3. Collapse owns all subordinate O retirement before any subordinate work is
   dispatched. Enumerate its **actual target** read-only: exact O leaves and O
   structural directories only, with current guards. An unowned empty directory
   vetoes too. Then perform the scoped second pass under the same O subtree.
4. Source preparation follows `InteriorInput::Listed`, enumerating only declared
   desired descendants. Never call physical `copy_directory_selected` as a shortcut.
   Obsolete O-only work keeps `SourceObservation::NotRequested`.

`InteriorInput::Listed` identifies an enclosing desired-input slice, not proof
that every target-only descendant has a desired node. Below a collapse, or through
O-only routing, retain the enclosing slice and resolve the current relative
coordinate against it (possibly no desired input). Every prepared manifest has
a root node, including an empty manifest. Desired expansion narrows to its actual
desired node. O authority comes from the bound owned index/removal grant,
not that desired node ID. This accommodates target-only recursion without inventing
desired entries, source absence, or another persisted input enum.

**One real future acquisition addition remains:** today's public `inspect_group`
only builds physical-union candidates, and sync cannot construct `PreparedDirectory`.
The listed slice must add an inspect-owned direct-declared-lookup/finalization path
over the same acquisition cells, common validator and execution-demand completion.
Sync supplies declared obligations/authority and guard requests; inspect acquires
actual facts. Do not expose `from_selection_and_metadata`, force NotRequested
through legacy `Presence`, or build a JW copier. This entry point need not be
implemented in the contextual slice; the kernel's direct prepared/fact seam already
accommodates its output.

## 7. Route epoch and next dependency order

`InspectionEpoch` is presently empty; compare's `Routes` and sync's `route` still
perform separate checks. Give the epoch a private no-follow route/metadata cache
and checked lookup plus invalidation methods callable across the existing crates.
For example: `directory(path) -> Result<Arc<Metadata>, Arc<inspect::Error>>`,
`invalidate_subtree(path)` and `invalidate_attributes(path)`. Directory lookup
admits the entire no-follow route, not merely the last path component.
For parallel effects, use shared-reference access with short internal locks;
inspection may change its current `&mut InspectionEpoch` argument to `&InspectionEpoch`.
Never hold a route/cache index lock across policy callbacks, recursion or file I/O.
Per-key coordination can prevent duplicate concurrent acquisition without a
whole-cache lock. This is runtime plumbing, not a route-certificate constructor.

- Admit frozen roots and all real ancestors, including absolute ancestors. Reuse
  source prefixes once within a call and target prefixes per physical lane.
- On successful mkdir/unlink/rmdir/rename invalidate affected physical paths and
  descendants, including negative entries and both renamed endpoints. On chmod,
  invalidate cached attributes for that directory. Existing owned leases retain
  their original restoration value; newly acquired leases cannot reuse stale modes.
- Newly created directories need current post-umask facts. Already acquired directory
  metadata may populate the epoch within that same live scope; policy views stay
  demand-masked. Do not treat old prepared facts as a route certificate.
- `apply` creates one epoch for internal refresh plus application; standalone
  `refresh` discards its epoch on return. All returned calls discard routes even
  when prepared facts or the bounded source cache survive. Fresh route checks do
  not replay accepted policy admission.

Recommended commit/dependency order for the next worker:

1. Propagate C/P/M, exact binding descriptors, saved input/child contexts and
   `ApplicationNeed` through paired publication/barrier refresh as already planned.
2. Wire `PhysicalExecutionDemand`, pure whole-batch fact checks and direct
   `PreparedDirectory<C>` execution. Extract policy-free leaf/directory primitives
   and lane-local call state concurrently with contextual opaque recursion.
3. Close conditional/preview/two-pass context propagation, finish/provisional error
   boundaries and shared session/epoch plumbing. Preserve fresh-process continuation.
4. Later grouped work changes scheduling/inspection grouping around these primitives;
   later listed work adds declared acquisition/binding/coverage around them.

Review discriminators are the accepted design's existing gates: a sparse equal
renewal; later sibling execution-fact failure with zero immediate effects; unused
conditional ancestry; selected empty-directory materialization; target-only inherited
removal; intact collapse veto; failed private expansion retaining the old leaf;
failure after visible removal; permission/cleanup failure retaining progress; and
A-accepted/B-unstarted restore. Grouped and manifest-specific gates belong to their
subsequent slices. No new product fork is required by this handoff. Parent should
review the private seam and the enclosing-slice interpretation above before it is
treated as an implementation contract.
