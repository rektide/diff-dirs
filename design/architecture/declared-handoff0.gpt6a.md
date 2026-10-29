---
type: ImplementationHandoff
title: Inspect-owned preparation of declared desired and owned work
description: Concrete declared-lookup, target-census, authority-lowering and manifest-binding seams for the listed runner over the existing contextual effect kernel.
resource: /design/architecture/declared-handoff0.gpt6a.md
status: proposed-implementation-handoff
generated: { by: "model:openai/gpt-6-astra#high", at: 2026-10-29 }
sources:
  - resource: "opencode://ses_e799075e5ffeLWmQqtFubr3wPE"
    title: Resolves the remaining declared acquisition and manifest-binding seam against committed provider source, including missing routes, coupled scheduling and sparse source-cache completeness
  - resource: "opencode://ses_e7a2771c3ffez3sv1oPgD5gPH4"
    title: Parent commissions this narrow design while a separate owner migrates contextual effects and route epochs
  - resource: /design/architecture/execution0.gpt6a.md
    title: Accepted desired-owned API, complete coverage, acquisition, source sharing and coupled transition requirements
  - resource: /design/architecture/kernel-handoff0.gpt6a.md
    title: Accepted kernel ownership, pre-effect readiness, actual manifest binding and enclosing desired-slice interpretation
  - resource: /design/architecture/execution1.sol61x.md
    title: Delivered 0.4 frontier checkpoint and explicit remaining declared acquisition gap
  - resource: /tree/inspect/src/selection.rs
    revision: 7ed693b6
    title: Actual selected fields, listed legality, interior roles and structural validation
  - resource: /tree/inspect/src/acquire.rs
    revision: 7ed693b6
    title: Actual present-entry cells, retained metadata and independently masked fact demands
  - resource: /tree/inspect/src/policy.rs
    revision: 7ed693b6
    title: Actual context validation, coordinates and pure execution-demand contract
  - resource: /tree/inspect/src/prepared.rs
    revision: 7ed693b6
    title: Inspect-private live constructor and logical root plus source-generation binding
  - resource: /tree/inspect/src/shared.rs
    revision: 7ed693b6
    title: Physical-union-only preparation, grouped outcomes and source-session ownership at the committed checkpoint
  - resource: /tree/inspect/src/visitor.rs
    revision: 7ed693b6
    title: Existing no-effect PreserveTree, container actions and legacy presence restrictions
  - resource: /doc/architecture/README.md
    title: Maintained component ownership and distinction between delivered frontier and prospective listed execution
verification:
  method: Committed-source inspection, authorized completed-design comparison and explicit Markdown/provenance checking
  scope: Design only; no production edits, runtime tests, benchmarks or inspection of the active effect implementation
---

# Declared preparation handoff

**Add one inspect-owned immediate preparation entry point with two closed name
sources: supplied declarations, and actual target children for coupled removal.**
Both use the existing acquisition cells, selection validator and execution demand.
Sync supplies pure decisions over its immutable S/O indexes and binds the result
back to those indexes before effects. No persisted selection field must change.

This closes [execution §9](execution0.gpt6a.md#9-reusable-prepared-leaf-lists-and-proactive-inputs)
and the explicit gap in [kernel handoff §6](kernel-handoff0.gpt6a.md#6-intact-refusal-and-listed-binding).
It targets the committed `7ed693b6` implementation described by the
[0.4 account](execution1.sol61x.md), not the concurrent kernel/epoch edits.
Signatures below specify ownership and information flow; private helper names may
follow that migration. Parent retains promotion/index integration.

## 1. Owner boundary and public seam

| Owner | Responsibility |
| --- | --- |
| Native caller | Declares desired S, previously owned O and `ListedIntent`; owns the truth of native authority. |
| Sync | Builds immutable tries/ManifestIds; chooses the next immediate declarations; lowers observed shapes to exact authority; binds references and coverage ownership; schedules the existing kernel. |
| Inspect | Obtains actual presence/kinds/facts, calls pure demand/check/selection hooks, validates and alone constructs `PreparedDirectory<C>`. |
| Observe / epoch | Native I/O and no-follow route admission; no S/O interpretation. |

Keep `reconcile_listed`, `ListedTarget` and `ListedGuard` from execution §9.1.
The following lower seam belongs to **inspect** (suggested `declared.rs`), so it
imports no sync types, tries, `ListedIntent` or native ownership logic:

```rust,ignore
pub struct DeclaredName<'a> {
    pub name: &'a OsStr,
    pub desired: Option<DesiredInput>,
}
pub enum DeclaredNames<'a> {
    Exact(&'a [DeclaredName<'a>]),
    TargetCensus, // actual immediate target names; source always NotRequested
}
pub struct DeclaredRequest<'a> {
    pub relative: &'a Path,
    pub purpose: Purpose,
}
pub struct DeclaredInspection<'a, L: DeclaredSelection> {
    pub target: usize,
    pub logical_root: &'a Path,
    pub physical_directory: Option<&'a Path>,
    pub desired: ManifestId,
    pub owned: ManifestId,
    pub context: &'a L::Context,
    pub names: DeclaredNames<'a>,
    pub selection: &'a L,
}
pub fn inspect_declared_group<L: DeclaredSelection, E: ExecutionDemand>(
    source: &SourceSession,
    at: DeclaredRequest<'_>,
    lanes: &[DeclaredInspection<'_, L>],
    execution_demand: &E,
    epoch: &InspectionEpoch,
    execution: Execution,
) -> Result<Vec<TargetComparison<PreparedDirectory<L::Context>>>, SharedInputError>;
```

`Exact` is a complete immediate **declared** batch, not a physical union. Names
are unique ordered native basenames. `None` desired means do not query source,
not that source is absent. Inspect never derives candidates by source `read_dir`.
An S leaf missing on both sides still has its supplied slot and fails acquisition.
Each lane can supply different O-only names; shared source keys include only the
union of participating desired coordinates.

`TargetCensus` is used only inside a sync-bound coupled removal scope. Inspect
enumerates the actual target directory, retains its listed handles, and submits
**every** immediate child with `desired: None`. Sync's hook looks up each name in
the bound O subtree and refuses unowned children. No source directory is checked
or enumerated. This closes the unowned-empty-directory hole that a declarations-only
preflight would leave. An absent/non-directory census route is a changed-input
failure, not an empty census or permission to reclassify the ancestor transition.

Source paths are always `source.root()/relative/name`; this API needs no arbitrary
source-route override. `physical_directory: Some(path)` names the actual containing
target route to inspect, including one found absent. `None` means a privately
constructed empty input, as in replacement preparation; only sync's private
placement binding may authorize executing such a preparation. Logical roots and
relative coordinates remain final even when output is staged elsewhere. Public
application must freshly prepare under its own binding, as already required.

## 2. Candidates, guards and immutable finalization

Use immutable inspect-owned candidates and a narrow pure selector. The selector
can choose work but cannot replace observed shape, name, desired declaration or
facts. There is no public selection-plus-metadata constructor.

```rust,ignore
pub struct DeclaredEntryCandidate<'a> {
    pub info: BatchInfo<'a>,
    pub domain: SelectionDomain,         // always Listed
    pub coordinate: Coordinate<'a>,
    pub desired: Option<DesiredInput>,
    pub shape: Shape,                    // inspect-owned actual observation
}
pub struct DeclaredRouteCandidate<'a> {
    pub info: BatchInfo<'a>,
    pub side: Side,
    pub relative: &'a Path,              // empty for the bound root
    pub kind: Kind,                      // actual admitted directory
}
pub enum DeclaredCandidate<'a> {
    Route(DeclaredRouteCandidate<'a>),
    Entry(DeclaredEntryCandidate<'a>),
}
pub struct DeclaredChoice<C> {
    pub action: Action,
    pub install: Option<InstallAuthority>,
    pub removal: Option<RemovalAuthority>,
    pub interior: Option<Interior<C>>,
}
pub trait DeclaredSelection: Sync {
    type Context: Clone + Eq + Debug + Send + Sync;
    fn identity(&self) -> &SemanticId;
    fn context_schema(&self) -> &SemanticId;
    fn validate_context(&self, at: ContextSite<'_>, context: &Self::Context)
        -> Result<(), CheckError>;
    fn demand(&self, context: &Self::Context, at: DeclaredCandidate<'_>)
        -> SideDemands;
    fn check(&self, context: &Self::Context, at: DeclaredCandidate<'_>,
             facts: EntryFacts<'_>) -> Result<(), CheckError>;
    fn select(&self, context: &Self::Context, at: DeclaredEntryCandidate<'_>)
        -> Result<DeclaredChoice<Self::Context>, CheckError>;
}
```

All callbacks are pure. Sync's private `ListedSelection<'a, G>` implements this
trait using the prepared S/O loans, intent and guard. Its `select` performs the
table in §4; its `demand/check` adapt the accepted public guard unchanged:

```rust,ignore
pub trait ListedGuard: Sync {
    fn demand(&self, candidate: ListedCandidate<'_>) -> SideDemands;
    fn check(&self, candidate: ListedCandidate<'_>, facts: EntryFacts<'_>)
        -> Result<(), CheckError>;
}
```

Define the previously unspecified `ListedCandidate` as a read-only view of the
route/entry candidate plus this lane's intent and **resolved declaration relation**
(desired leaf/directory, exact owned leaf, owned structural node, or neither).
It identifies ordinary work, private desired preparation or coupled removal from
the input context. The guard returns refusal, never replacement action, exclusion,
ownership or alternate source input. It sees only its own requested facts.

Routes need this same guard interface: a device/admin check must see actual roots
and intermediate directories, not only leaf entries. Inspect admits absolute
ancestors without following links, then presents actual root-to-containing-route
directories to the lane guard in root-to-leaf order. Source route candidates exist
only where a desired lookup requires them. A route fact view contains only its
identified side; route demand is restricted to that side. No `Presence` projection
is used. Epoch facts can satisfy demand without becoming opportunistically visible.
Guard success is lane/context-local even if route metadata is shared.

The route hook also checks the containing directory of a census, including its
initial coupled root. It does not create a listed structural-root singleton:
listed selections retain `structural_root = false`, including `relative = ""`.
Guard/root checks do not run ordinary `Policy::names/shape/select` or derive mirror
authority. The initial listed context is supplied by sync, not ordinary root gating.

### Finite preparation order

1. Validate request syntax/order, domain IDs and input context before lookup.
2. Admit and guard needed routes. Generate `Exact` slots or target census slots;
   acquire actual source/target presence and kinds, source first per native name.
3. Collect guard demands for all surviving lanes. Add inspection's expectation
   demand (regular executable-bit expectation needs source attributes). Acquire
   the source-demand union and lane-local target demands.
4. Validate desired actual kind/executable expectation; run each guard with only
   its mask; call `select`; validate each proposed interior context at its final
   child coordinate and role. Guard refusal fails the lane's whole immediate batch.
5. Build the existing `SelectionParts<C>` with actual observations, supplied
   desired refs, `Comparison::NotAssessed`, `baseline: None`, no exclusions,
   `scoped: true`, and the selector's policy/schema IDs. Use one inspect-owned
   versioned semantic ID for omitted listed comparison; invoke no comparator.
   Call the existing `DirectorySelection::from_parts` for the entire lane.
6. Union `ExecutionDemand::for_entry` across successful lanes, acquire outstanding
   inputs using the same cells, and return prepared lanes only after this stage
   completes. Prepared fact masks include that lane's guard, expectation and
   execution demands, never unrelated peer demands.

The internal finalizer can be factored with physical inspection:

```rust,ignore
// crate-private; ObservedSlots contains inspect-owned cells and absence outcomes.
fn finalize_declared<C: Clone + Eq>(
    selected: Arc<DirectorySelection<C>>,
    observed: ObservedSlots,
    lane_demands: Vec<SideDemands>,
    binding: LiveBinding,
) -> Result<PreparedDirectory<C>, Arc<Error>>;
```

Execute the grouped execution-demand stage **before** this finalizer. It must
reject any unresolved requested fact rather than publish an incomplete runtime
view. `NoExecutionDemand` remains valid for read-only preflight; sync separately
checks execution readiness before effects. Reuse the selection validator and cell
machinery, not the current `PairSlot` assumption that two missing sides cannot
exist. No new serialized facts or parallel legality table is introduced.

## 3. Actual lookup, required facts and shared failures

Add a crate-private no-follow child-lookup adapter around the current cell:

```rust,ignore
enum LookupOutcome { Absent, Present(Arc<AcquisitionCell>) }
fn lookup_child(parent: &CheckedDirectory, name: &OsStr, acquire: &impl Acquire)
    -> Result<LookupOutcome, Arc<Error>>;
// Only used inside inspect after actual successful no-follow lookup:
fn cell_from_lookup(path: PathBuf, metadata: Arc<Metadata>) -> AcquisitionCell;
```

`CheckedDirectory` stands for the epoch's private admitted route state, not a new
public certificate. A direct lookup uses actual `symlink_metadata`, retains the
whole result, and reports acquired-metadata origin. Do not label a child lookup
as `RootMetadata` merely to reuse today's `AcquisitionCell::bound`. A later
attributes demand reuses that same result. Literal links remain separately lazy.
When that lookup establishes a real directory below an admitted parent, make its
metadata available to this epoch's route cache too. Entering it as the next
containing directory must not stat it again solely because its role changed.
This is current-epoch observation reuse, never a retained cross-call route proof.

Only a genuine missing component under an admitted real directory establishes
absence. A missing target ancestor establishes absence of all declared descendants;
an intervening symlink, regular file, permission error, `ENOTDIR`, unsupported kind
or failed listed-entry acquisition does not. The route resolver returns the
missing-prefix observation to inspect internally; callers cannot inject it as
metadata. For a desired source, missing leaf **or route** fails the obligation.
For O-only work, inspect makes no source lookup, route check, `read_link` or source
guard call, even if another lane queries that physical path for desired work.

An O-only selector/guard/execution demand asking for source facts is an integration
error: fail without I/O. An absent target has no `SideFacts`; a demand for attributes
of present target sides does not fabricate a target. A requested literal link is
required only on an actual symlink, as with the existing cell. A needed fact whose
acquisition fails retains its original error; it never becomes NotRequested or
absence. A sync readiness check also performs fallible field extraction, such as
`Metadata::modified()`, before the first effect.

For one group, intern source outcomes by `(source generation, relative child)`.
Collect the union at each finite demand stage, acquire each fact at most once,
then evaluate each lane's own requested outcomes. Do not collapse the union into
one fallible `acquire(FULL)` result: a peer's failed link request cannot poison a
kind-only view. Direct lookup metadata is essential to actual presence/kind, so
its failure affects every lane needing that source path. Shape failure, guard
failure and expectation mismatch remain independently attributable per lane.

Use deterministic stage order above and native-name/source-before-target ordering
inside each stage; asynchronous completion never chooses the error. Malformed
group requests return `SharedInputError`; filesystem, context, expectation, guard
and authority failures return the existing ordered per-target outcomes. Healthy
lanes continue. Inspect returns the whole group before sync starts any member's
effects; an error in lane A is not a group-wide effect veto for healthy lane B.

## 4. Per-lane lowering and concrete context roles

Sync computes authority from actual candidate shape plus its bound S/O relation.
Every installation below has `InstallIntent::Renew`, even CreateAbsent: the work
is proactive, not an inferred physical difference. No removal grant accompanies
an absent destination. Checks for an exact O leaf now occupying a directory or
special node take precedence in RenewOwned; this is an obstruction even when S
would otherwise use that directory as a route.

| Candidate / actual target | Action, grants and recursive input |
| --- | --- |
| S leaf / absent | `InstallLeaf`, `AbsentOnly`, no removal. |
| S leaf / supported leaf | RenewOwned plus exact O leaf only: `InstallLeaf`, `ReplaceLeaf`, `OwnedLeaf`. CreateAbsent or unowned occupancy refuses. |
| S leaf / directory | RenewOwned plus O structural node only: `ReplaceWithLeaf`, `ReplaceTree`, `CoupledOwned(node)`; `RemoveForReplacement`, enclosing desired slice. |
| S directory / absent | `Directory(Create, true)`, no install/removal grant; `SourceOnly`, exact S child-node slice. Source must actually be a directory. |
| S directory / real directory | `Directory(Keep, true)`, no grants; `Paired`, exact S child-node slice. Listed runner owns descent; no compare edge. |
| S directory / supported leaf | RenewOwned plus exact O leaf only: `ReplaceWithDirectory(true)`, `ReplaceLeaf`, `OwnedLeaf`; `PrepareReplacement`, exact S child-node slice. |
| Obsolete O leaf / absent | `ConfirmAbsent`, `OwnedLeaf`, no interior; source NotRequested. |
| Obsolete O leaf / supported leaf | `RemoveLeaf`, `OwnedLeaf`, no interior; source NotRequested. |
| O-only structural node / directory | `Directory(Keep, true)`, no grants; `TargetOnly`, enclosing desired slice. Retain container after descendants. |
| O-only structural node / absent | `PreserveTree`, no grants/interior; source NotRequested. Bound missing-route observation settles its obsolete O subtree; see below. |
| Coupled census exact O leaf / supported leaf | `RemoveLeaf`, `OwnedLeaf`; source NotRequested. |
| Coupled census O structural node / directory | `Directory(Remove, true)`, `CoupledOwned(child node)`; `TargetOnly`, enclosing desired slice. |

All other relevant obstructions refuse: unsupported desired/target kinds, obsolete
O leaf now directory, an O-only structural route now a leaf/link, unowned census
leaf or directory, or a census directory at an exact O leaf. Guard refusal is
also failure, not `PreserveTree` fulfillment. Ordinary target contents not named
by declarations are untouched and need not be observed.

CreateAbsent schedules S only: its effective ownership and obsolete obligations
are empty even if the supplied O index is nonempty. It can retain real routing
directories but cannot replace anything. Binding checks this independently of
whatever choice the selector produced.

**Absent structural O routes need no new action.** At `7ed693b6`, `PreserveTree`
is a legal no-effect entry with NotRequested source, absent target, no desired
input and no interior. Here it records an observed missing routing prefix, not
preservation of desired work. Sync binds its coordinate to an actual O structural
node and settles precisely the obsolete leaves beneath that absent prefix. It
does not recurse or fabricate one `ConfirmAbsent` per descendant. The same rule
applies to an absent S leaf/directory coordinate with subordinate obsolete O:
settle those old obligations before creating the new node and suppress their
separate scheduling. An exact obsolete O leaf at an absent S directory coordinate
is similarly already absent. The bind result owns these once-only coverage credits;
the kernel merely sees the selected effect.

This uses the accepted missing-ancestor absence rule. `ApplicationNeed::None`
must not cause the listed runner to skip this binding/coverage step; it means no
kernel application, not no declared obligation. A missing route's coverage is
credited only after the whole immediate batch has passed preparation and binding.

Use a sync-owned semantic `ListedContext` in the existing C slot. It carries the
listed intent/interpretation and the current operation scope: ordinary declared
work, desired preparation, or coupled removal rooted at an exact O node. IDs and
coordinates are owned semantic values, not trie pointers. The adapter lends the
actual tries separately. Its `validate_context` checks these relations at the
given coordinate and interior role.

`InteriorInput::Listed` retains its accepted **enclosing desired slice** meaning.
For S-directory descent, narrow to the actual child S node. For O-only routing
and collapse, retain the enclosing slice, even when the current coordinate resolves
to no S node. In coupled removal, an S leaf at the collapse root is not a desired
source for its children; census candidates have no desired input. O grants and
the bound coupled scope authorize removal, not that enclosing desired node.
Every manifest, including empty S, has a root node.

Preflight and the effectful removal pass start with the same selected
`RemoveForReplacement` context. Newly selected nested directory roles are
`TargetOnly`; propagate their exact child contexts in both passes. Preview versus
execution is a demand/call mode, not permission to change the context or scope.
Private desired preparation starts with `PrepareReplacement` and follows only S
children under its selected node. The public guard needs no new Context type.

## 5. Actual manifest binding is the effect gate

Generic `validate_selection` proves structural legality: ordered slots, actual
shape/action compatibility, listed comparison omission, grant-ID agreement and
required interior role/input. It cannot prove a LeafId exists, a NodeId contains
this coordinate, a declared batch is complete, or native O authority is true.
Do not strengthen that validator by importing sync's indexes.

Sync adds a **private** bind against its immutable inputs and current owned scope:

```rust,ignore
fn bind_listed_batch<'a, C>(
    prepared: &'a PreparedDirectory<C>,
    desired: &'a PreparedLeafList,
    owned: &'a OwnedLeafSet,
    scope: &'a ListedScope,             // intent, input slice, coupling, placement
) -> Result<BoundListedBatch<'a, C>, Failure>;
```

The returned value borrows the original preparation/indexes; it is not a second
selected operation array or persisted capability. It ties slots to their exact
obligations and defines which absent prefixes/coupled transitions own retirement.
Before opening a permission lease, staging arena, mkdir, unlink or publication:

1. Match actual S/O ManifestIds, platform/version and selector/context interpretation;
   verify roots, source generation, final relative coordinate and private placement.
2. Resolve every desired LeafId/NodeId. Require exact full coordinate, exact copied
   expectation and S node type; validate the enclosing input and child-slice relation.
   Direct inspection has already checked the expectation against actual facts.
3. Resolve every `OwnedLeaf` to the exact same coordinate, and every `CoupledOwned`
   to the exact O structural node at that coordinate inside the active coupling.
   Reject exact O-leaf-as-directory authority. Verify every action/grant against
   intent and declared relation, including the weakest destination grant in §4.
4. Match `Exact` slots to the complete next immediate S/O declaration set for this
   scope, accounting for explicitly consumed/absent-owned scopes. In a census,
   bind every actual returned slot and require O membership; census completeness
   comes from fresh inspect enumeration, never from decoded semantic Parts.
5. Bind no-effect absent-route markers by coordinate to their O subtree and check
   there is no omitted desired input. Assign retirement to one owner; do not infer
   coverage from `PreserveTree` or `ConfirmAbsent` action names in isolation.
6. Run the kernel's pure whole-batch `require_execution_inputs` and required-field
   extraction. Only then dispatch the original prepared selection/facts.

A mismatched ID, out-of-range reference, wrong leaf coordinate/expectation,
overbroad subtree, omitted declared child or forged CreateAbsent grant fails with
zero effects for this immediate batch. Hash equality alone is not this check.
`BoundListedBatch` never authorizes a decoded historical selection to execute:
inspection's live acquisition and sync's private placement are both required.

The boundary is immediate-batch atomic admission, not whole-manifest rollback.
Earlier completed siblings/ancestors remain reported. Every recursively entered
batch repeats this gate. Coupled collapse adds its full read-only subtree veto
before **that transition's** staging, chmod or dismantling; source preparation and
actual target removals retain the kernel handoff's provisional/progress rules.

## 6. Exact candidate schedule and grouped requests

Use an ordered component-trie walk of declarations, with one immediate batch per
containing coordinate. In RenewOwned, its names are the immediate child union of
S and still-active O; in CreateAbsent they are S children only. Entries are ordered
using the existing native-name ordering. Do not flatten O-minus-S into a preliminary
removal queue, and do not enqueue children before their parent slot is bound.

After all immediate candidates are prepared/bound, visit slots in native order.
For each lane, a slot chooses exactly one continuation:

1. **S leaf:** install, or own the entire coupled collapse at this coordinate.
   O descendants are consumed by that transition or by actual prefix absence;
   they never become independent obsolete jobs. With S=`q`, O=`q/a,q/b`, actual
   unowned `q/local` vetoes before either owned child is dismantled.
2. **S directory:** recurse only through its S node and applicable O declarations
   if retaining/creating a route. If replacing an exact O leaf, stage an **S-only**
   subtree, then replace that leaf as one owner. Source `p/local` is never listed
   when only `p/a,p/b` were declared. Existing target-only locals in retained
   routing directories are not candidates.
3. **O-only directory:** retain it and recurse through O declarations, or settle
its subtree from the missing-prefix observation. A runtime frame records that
   already-settled O work must not be rediscovered against newly created S output.
4. **O-only leaf:** remove/confirm absence once. No source query.

Within collapse, the recursive preflight uses TargetCensus at **every actual
directory**, binding each child to O. After successful full preflight and donor
preparation, the second pass uses TargetCensus again with the same coupled scope
and execution demand. It is not an Exact/O-only pass: actual unexpected children
must not escape the scoped removal check. Removing the coupled directory settles
remaining subordinate obsolete O absence; credit actual removal even if subsequent
desired publication fails. On second-pass failure retain only known completed
retirements/progress; no post-failure census is required.

For multiple lanes, group the compatible S-owning continuations at the same
source coordinate/input slice **before** their child inspection. A newly created
target directory, a retained directory and a privately prepared directory can all
share one Exact source acquisition while keeping different target routes, O sets,
contexts and errors. Call `inspect_declared_group` with their independently formed
name slices; its source map ignores O-only names and mask-leaks no facts. Do not
run entire lane A then lane B and call a bounded-cache hit source-once grouping.

TargetCensus work can remain lane-local; it has no shared source demand. Grouping
does not violate parent-before-child/coupled ownership or native order within a
lane. Unrelated earlier siblings may finish before a later collapse veto. This
specifies request dependencies, not a new generic scheduler or saved listed cursor.

For coverage, obsolete O means O leaf coordinates that are not S leaf coordinates
(empty in CreateAbsent). An owned leaf renewed at the same desired coordinate
does not add an obsolete-retirement credit. Expansion/collapse and missing-prefix
owners settle their exact share of that set once, separately from actual unlink
counts and desired installation credits. Private desired coverage stays provisional
until final publication; a later publication failure does not erase already-visible
O retirement.

## 7. Source generation, route epoch and the bounded-cache addition

The current checkpoint only shares cells within physical calls. Declared grouping
must already share live lookup cells without cache retention. Reuse the same
SourceSession across listed/paired/opaque calls; never allocate a session per
directory. Fresh route admission belongs to the current InspectionEpoch, including
when a retained source fact can satisfy a demand. No source route is admitted for
an O-only or target-census batch. Target changes invalidate epoch paths/attributes
as specified by the kernel owner; target observations never enter the source cache.

The future bounded cache needs one small representational addition: **sparse
known entries are not a complete listing**. Keep the accepted generation/directory
key and capacity limits, but distinguish lookup-populated cells from complete
enumeration, for example privately:

```rust,ignore
struct SourceBatch {
    complete_names: Option<OriginalListing>, // original enumeration hints
    cells: NativeNameMap<Arc<AcquisitionCell>>,
}
```

An Exact lookup may populate only named positive cells; it never sets
`complete_names`, treats all other names as absent, or triggers `read_dir` to fill
the cache. PhysicalUnion still requires a complete enumeration. Conversely, an
available complete listing may answer named lookup presence and supply the same
cells after fresh route admission. Failure of a fact on an enumerated name is
failure, not evidence of absence. Original physical-listing hints remain attached
to the enumeration view; direct-lookup metadata must not strengthen later early
policy views merely through cache history.

Coordinate per cell and charge names, map/arena capacity and retained fact/link
storage to the existing batch budget. No manifest IDs, target authority, guard
decisions or coverage enter cache identity. Different manifests may reuse the
same physical positive facts, then validate different expectations independently.
Keep missing/failed lookup outcomes only in the active group/epoch; retain only
successful source facts across calls. Close listed handles at call boundaries.
Eviction/disabled retention changes cost only; it cannot affect declared coverage.
A source-phase change requires a fresh generation; neither manifest identity nor
matching path strings establishes source-content continuity.

## 8. Decisions and implementation review gates

**No unavoidable change to the accepted persisted fields or public listed runner/
guard signatures.** Add the inspect request/candidate/selector types and entry
point, private direct-lookup support, and sparse-vs-complete source cache state.
Use the existing C field for sync's listed scope; use existing no-effect
`PreserveTree` only for the tightly bound absent O-structural marker described
above. Actual manifest relation binding remains in sync. Generic structural
validation alone never claims it.

Implementation ownership: the later listed owner adds the inspect seam and sync
adapter/manifest binder after the contextual kernel stabilizes. Coordinate the
small sparse-cache record addition with its cache owner; use the kernel owner's
actual epoch API for route admission rather than introducing another route cache.

Required discriminators for that implementation, not tests run by this design:

- Both-missing S leaf fails; unrelated physical source children are never copied.
- An O-only removal succeeds without touching an inaccessible/unrelated source
  child; another lane's desired query at that name does not change its observation.
- A later immediate sibling's guard/execution-fact/bind failure yields zero effects
  for that lane's batch; a healthy lane remains independent.
- A missing O-only structural prefix settles exact obsolete coverage; a symlink or
  non-directory prefix refuses; later S materialization does not redispatch O.
- Collapse rejects an unowned leaf **and an unowned empty directory** before any
  subordinate O change. Expansion stages manifest children only and retains its
  old leaf when preparation fails.
- Corrupt leaf IDs, node relations, expectations, omitted declaration slots and
  CreateAbsent grants refuse before effects even if generic validation succeeds.
- Grouped absent/retained/staged target lanes acquire each needed desired source
  fact once; peer-only failed facts stay masked. Sparse cache population never
  makes a subsequent physical listing incomplete.

The [maintained architecture](/doc/architecture/README.md#components-and-ownership)
remains the ownership home. This handoff extends the two accepted execution
documents and resolves the gap explicitly retained by execution1; native
replacement proof and broad scheduling remain with their existing owners.
