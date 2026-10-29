---
type: ImplementationDesign
title: Contextual selections and shared acquisition for production execution
description: Provider-side Rust contracts, persisted-schema transition, recursive ownership and build gates for the October 29 coordinated execution commission.
resource: /design/architecture/execution0.gpt6a.md
status: proposed-for-implementation
generated: { by: "model:openai/gpt-6-astra#xhigh", at: 2026-10-29 }
sources:
  - resource: "opencode://ses_e7a2675f8ffe76VJT5Lt1Lwe5l"
    title: Designs and reviews provider contracts, then integrates the authorized native handoff into complete desired/owned manifest renewal with coupled transition ownership
  - resource: "opencode://ses_e7a2771c3ffez3sv1oPgD5gPH4"
    title: Parent commissions detailed Astra design and Sol implementation with broader future architecture, then corrects generated model provenance from the completed session's measured xhigh execution
  - resource: /design/architecture/annealing0.gpt6ax.md
    title: Selected integration assessment and proactive renewal requirements
  - resource: /design/architecture/rebuild0.gpt6a.md
    title: Delivered partial-frontier ownership and coarse opaque-operation checkpoint contract
  - resource: /doc/architecture/README.md
    title: Current component responsibilities and delivered versus prospective capabilities
  - resource: /design/architecture/draft0.gpt6a.md#resource-control-and-dirent-buffers
    title: Previously selected caller-owned 64 KiB buffers, lazy idle pool and resource-lifetime contracts
  - resource: /design/observation/facts0.gpt6am.md
    title: Finite demand stages, full visitor compatibility and execution handoff
  - resource: /design/observation/names0.gpt61sxh.md
    title: Native-name allocation evidence and sparse child promotion
  - resource: /design/transfer/modes0.gpt6am.md
    title: Inherited region meaning, residual exceptions and explicit construction authority
  - resource: /design/actions/dispatch0.gpt61sxh.md
    title: Typed desired work and dispatch separated from physical differences
  - resource: /tree/inspect/src/visitor.rs
    title: Current full visitor, baseline, action legality and combined fact/selection ownership
  - resource: /tree/inspect/src/shared.rs
    title: Source-once inspection, lane-local error order and validated-plan lowering
  - resource: /tree/observe/src/listing.rs
    title: Current consumed acquisition handles, native name ownership and buffer allocation
  - resource: /tree/compare/src/coordinated/mod.rs
    title: Current group acceptance, route checks, reductions and lowering
  - resource: /tree/compare/src/coordinated/state.rs
    title: Exact partial ancestors, child memberships and saved control state
  - resource: /tree/compare/src/pair.rs
    title: Current clean-plan release predicate
  - resource: /tree/compare/src/walk.rs
    title: Current divergence-only plan publication
  - resource: /tree/compare/src/frontier.rs
    title: Parallel publication and descendant seals
  - resource: /transfer/sync/src/coordinated/mod.rs
    title: Current saved decisions, deep cloning, refresh, runtime binding and application
  - resource: /transfer/sync/src/coordinated/decision.rs
    title: Duplicated semantic validator and metadata-free saved shape
  - resource: /transfer/sync/src/coordinated/state.rs
    title: Barrier consistency and returned-work accounting
  - resource: /transfer/sync/src/apply.rs
    title: Opaque copy/removal/replacement, prepared leaf kernel, permission scopes and listed seed preparation
  - resource: /transfer/sync/src/visitor.rs
    title: Existing adapter order and overrideable late presets
  - resource: "file:///home/rektide/src/jj-waltz/transfer/workspace/src/streaming.rs"
    title: Current source-qualified consumer requirements, conditional routing and retained device guards
  - resource: "file:///home/rektide/src/jj-waltz/design/coordinated-refill/execution0.gpt6a.md#43-replacement-ownership-program"
    revision: 2d2dfa2b
    title: Parent-authorized same-wave native handoff requiring complete desired coverage, O-scoped ownership and intact coupled-collapse refusal
verification:
  method: Source and prior-design inspection starting at diff-dirs 6e01c7da; explicit Markdown and provenance checking
  scope: Implementable design, not production implementation or new runtime/performance evidence; native design was read only after explicit parent authorization and its proof is attributed rather than repeated
extensions:
  related_work: [diffdir-tree-coordinated, diffdir-tree-coordinated-light-state, diffdir-tree-coordinated-region-context, diffdir-tree-coordinated-comparison-policy, diffdir-tree-observe-fact-reuse]
---

# Contextual selections and shared acquisition

**Build one contextual inspection engine, one metadata-free selection contract,
and one production effect kernel.** Use these for ordinary pairs, coordinated
lanes, opaque interiors and prepared leaf lists. Pull compact names, caller-owned
resources, bounded source reuse and explicit proactive installation into this
delivery because each changes ownership at those seams.

The [annealing assessment](annealing0.gpt6ax.md) is the direction; this document
chooses provider contracts sufficient to build it. Parent review selects these
choices before public-surface implementation. Native revision/cache authority and
the clear-first/replacement protocol remain with JW's independently commissioned
design. Parent owns shared indexes, maintained-document integration and tickets.

## 1. Concrete scope and invariants

Deliver all of the following, in the sequence in section 12:

1. A shared, immutable `DirectorySelection<C>` owned by inspection, with one
   common semantic validator and a separate live `PreparedDirectory<C>`.
2. A typed policy context carried in paired children, active lane frames,
   unstarted applications and every opaque recursive call, including preview and
   replacement removability checks.
3. A supplied comparator with a frozen semantic identity, explicit missing
   assessment states and two working built-in predicates.
4. Explicit proactive leaf renewal and destination grants; full work publication
   even when observed differences are empty.
5. Finite staged acquisition, source demand union and retained acquisition cells;
   unchanged full-fact visitor behavior through a small compatibility projection.
6. Compact batch coordinates/names, owned child promotion, caller-owned observation
   resources and within-call route reuse.
7. A production `apply_many` opaque cohort, plus bounded source observation reuse
   between opaque and subsequent paired work in the same source session.
8. A reusable prepared desired/owned manifest and listed execution API with
   strict-create, exact owned-leaf renewal/removal, coupled transitions and complete
   desired-leaf coverage independent of physical name-union discovery.

These are concrete physical-source capabilities. A generic input-provider registry,
arbitrary executable work payloads and general generated/mapped tree enumeration
are unnecessary for this build. The selected-work and explicit listed-input seams
give those later consumers a place to attach without changing physical observations
or the paired frontier's meaning.

Preserve these invariants throughout the migration:

- Physical presence, omitted assessment, unsupported kinds, preservation and
  observation failure are different states. A listed entry failing acquisition
  never becomes absent.
- Each `(lane, recursive region)` has exactly one owner. Paired descent belongs
  to compare; one-sided/replacement descent belongs to sync for the entire call.
- Entire immediate selections validate before dependent children or effects become
  available. Selected execution-fact acquisition completes before the first effect
  of that immediate batch.
- A pause preserves partial ancestors and exact decisions, with no raw facts,
  handles, buffers, leases, jobs or opaque-operation interior position.
- A returned opaque failure terminates that lane. Healthy lanes retain their own
  progress. There is no automatic in-flight retry/reclassification.
- Physical comparison counts, selected work, actual effects and policy coverage
  remain independently reportable.

## 2. Ownership and public Rust shape

Keep the existing crate dependency direction. New modules are within existing
owners; an extra universal "protocol" crate would obscure rather than simplify it.

| Owner / suggested paths | Owns |
| --- | --- |
| `tree/observe/src/{listing,resources,names}.rs` | Native enumeration, name storage, cached hints, no-follow acquisition, raw-buffer leases. |
| `tree/inspect/src/{selection,policy,comparison,acquire,shared}.rs` | Typed assessment, common semantic validation, finite fact demand, source session and immutable selections. |
| `tree/compare/src/{pair,walk,frontier,coordinated/*}.rs` | Ordered paired publication, contextual child memberships, partial frames, comparison reductions and controls. |
| `transfer/sync/src/{apply,kernel,listed,opaque}.rs` | Production effects, placement grants, prepared lists, opaque cohort recursion and per-lane effect results. |
| `transfer/sync/src/coordinated/{mod,state}.rs` | Exact application barrier, fresh preparation, accepted/failed disposition and effect-binding validation. |
| `examples/reconcile/src/transport.rs` | Versioned application-owned codec, including a concrete context payload. |

Names below are intended Rust contract names. Private field packing and helper
names may vary; semantic ownership and signatures should not drift silently.

```rust,ignore
// tree/inspect: generic over owned semantic context, not over runtime machinery.
pub struct DirectorySelection<C = ()> {
    // Private immutable storage. Read-only accessors are the public view.
    /* relative, purpose, structural_root, input_context,
       names, entries, exclusions, comparison coverage, work coverage */
}

pub struct PreparedDirectory<C = ()> {
    selected: Arc<DirectorySelection<C>>,
    facts: BatchFacts,              // runtime-only; indexed by selected slot
    binding: LiveBinding,           // physical roots / source session identity
}

impl<C> PreparedDirectory<C> {
    pub fn selection(&self) -> &Arc<DirectorySelection<C>>;
    pub fn facts(&self, slot: SlotId) -> EntryFacts<'_>;
}

pub struct SelectionParts<C> { /* owned semantic values for transport */ }
impl<C: Clone + Eq> DirectorySelection<C> {
    pub fn from_parts(parts: SelectionParts<C>) -> Result<Self, SelectionError>;
    pub fn to_parts(&self) -> SelectionParts<C>;
}
```

Only inspection can construct a **live prepared** batch. Public semantic parts can
construct a validated selection, but that is not permission to execute historical
facts. External application and restore acquire fresh facts and compare the entire
new selection before execution. No public `from_selection_and_metadata` shortcut.

Selection fields per admitted slot are: native name, actual side presence and
resolved kinds, `Comparison`, baseline proposal if available, final `Action`,
installation authority, and optional interior context. Exclusions retain names
without invented kinds. One containing coordinate replaces per-entry `PathBuf`s.
The batch retains its input context so refresh has the exact policy input.
Section 9.2 adds the declared-input domain, explicit NotRequested source observation,
removal authority and interior-input selector needed by native manifest renewal.
Those are part of this same contract, not a later extension after its API is fixed.

Use explicit entry points over the same engine, with no generic lowering payload
mistaken for saved context:

```rust,ignore
pub struct TargetInspection<'a, P: Policy, M: Comparator> {
    pub target: usize,
    pub logical_root: &'a Path,
    pub physical_directory: Option<&'a Path>,
    pub context: &'a P::Context,
    pub policy: &'a P,
    pub comparator: &'a M,
}

pub fn inspect_group<P: Policy, M: Comparator, E: ExecutionDemand>(
    source: &SourceSession,
    at: DirectoryRequest<'_>,
    lanes: &[TargetInspection<'_, P, M>],
    execution_demand: &E,
    epoch: &mut InspectionEpoch,
    execution: Execution,
) -> Result<Vec<TargetComparison<PreparedDirectory<P::Context>>>, SharedInputError>;
```

`DirectoryRequest` identifies the logical relative directory, actual present/absent
source-directory route, purpose and `RootGated` versus `AlreadyAdmitted` entry.
All lanes in one group share that source coordinate. Physical route omission is
allowed only by the calling composition's genuine absence, explicit single-tree
operation or private construction invariant. It is never a policy filter. Public
external application goes through its checked binding; it cannot inject an
arbitrary admitted request to execute a forged selection. Sync replacement
preparation uses final logical roots plus its private output location separately.

`PreparedDirectory`, the barrier and returned group share the **same**
`Arc<DirectorySelection<C>>`. Sync's current `DirectoryDecision::of` projection
and semantic validator disappear. A transitional decision-sharing commit is useful,
but the destination is this ownership split, not permanent duplicate decision types.
Comparison diff projection may be computed/cached once from selection; it is not a
second owner of actions. Caller-retained views can prolong the selection's life.

### Common semantic validation

Move current baseline/action tables from `visitor.rs` and sync `decision.rs` into
`inspect::selection`. Use the same functions for live finalization and decoded
selection validation:

```rust,ignore
fn baseline(shape: Shape, compared: &Comparison) -> Option<Action>;
fn interior_role(shape: Shape, action: Action) -> Option<InteriorRole>;
fn validate_entry(entry: EntrySelectionRef<'_, impl Eq>) -> Result<(), SelectionError>;
fn validate_selection(parts: &SelectionParts<impl Eq>) -> Result<(), SelectionError>;
```

The validator checks ordered native names, exclusion disjointness, side/kind
agreement, comparator-result structural restrictions, action/grant legality,
interior role/context presence and derived counters. It derives rather than trusts
`planned`, `deviated`, comparison-state counts and `needs_application`.

Sync adds only its own restrictions: correct source/target binding, two real roots
for coordinated use, disjoint effect roots, matching barrier/window, lane state,
copy policy, application totals and exact paired-child/context projection. Compare
validates frontier membership/order/reduction, not JW's context internals.
Listed selection additionally binds its manifest references to the supplied
prepared values before effects. Structural decoding can check reference shapes,
but cannot verify an external manifest's contents without that bind step.

## 3. Typed policy context and assessment stages

Use a new `Policy` trait for contextual/sparse work. Keep today's `Visitor` as the
full-fact compatibility interface, implemented through the same engine rather
than maintained as another inspector.

```rust,ignore
pub trait Policy: Sync {
    type Context: Clone + Eq + std::fmt::Debug + Send + Sync;

    fn identity(&self) -> &SemanticId;
    fn initial_context(&self) -> Self::Context;
    fn scoped(&self) -> bool { true } // conservative comparison-coverage promise
    fn validate_context(&self, at: ContextSite<'_>, value: &Self::Context)
        -> Result<(), CheckError>;
    fn observation_profile(&self) -> ObservationProfile {
        ObservationProfile::Staged
    }

    fn names(&self, context: &Self::Context, names: &mut Names<'_>)
        -> Result<(), CheckError>;
    fn shape(&self, context: &Self::Context, batch: &mut ShapeChoices<'_, Self::Context>)
        -> Result<(), CheckError>;
    fn select(&self, context: &Self::Context, batch: &mut Choices<'_, Self::Context>)
        -> Result<(), CheckError>;
}

pub struct Interior<C> {
    pub role: InteriorRole,
    pub context: C,
}
pub enum InteriorRole {
    Paired,
    SourceOnly,
    TargetOnly,
    PrepareReplacement,    // source-directory / target-leaf
    RemoveForReplacement,  // source-leaf / target-directory
}
```

Default method bodies can be no-ops where appropriate. `ShapeChoices` presents
resolved kinds and immutable name/presence; mutable choices request policy facts,
select comparison versus omission, and declare provisional action/grant/context.
`Choices` lends the acquired sparse facts and comparison result, then permits
final action/grant/context selection. Neither permits changing identity or facts.
The final action determines the required interior role. No context is published
until the whole batch passes validation.

The engine sequence is finite:

```text
enumerate → names admission → acquire kinds of surviving sides
→ shape choices → acquire union(policy demand, comparator demand)
→ compare → final policy selection → validate all selections
→ acquire missing selected-execution inputs → publish prepared batch
```

`shape` can explicitly terminal-preserve a resolved kind without asking for full
attributes. `names` can preserve without kind acquisition. Final policy cannot
reopen either terminal preservation. Existing overrideable behavior uses the
compatibility route below and does not acquire this terminal meaning accidentally.

`ObservationProfile::LegacyFull` is used by the full adapter. Its exact legacy
demands are known immediately after names admission, so it acquires that payload
as a single full observation stage, before its no-op shape projection. This is
essential for error precedence: a source readlink failure at an earlier name must
not lose to a later name's kind/metadata failure merely because new sparse policies
resolve kinds first. In a mixed group, acquired outcomes may be shared, but each
lane evaluates only its own profile's ordered demands. This is one acquisition
engine with two explicit schedules, not a second full-fact implementation.
For LegacyFull, rank the complete per-name outcome (source facts, target facts,
then default comparison) before the next name's outcome. A comparison-time timestamp
extraction error at name a must likewise precede name b's acquisition failure.
Compute earlier valid-slot comparisons before selecting a later fact error; do
not blindly concatenate global fact and comparison error lists.

Comparison-only callers supply an empty execution-demand provider. Sync supplies
its closed physical action demand table. This is a small inspection interface:
`ExecutionDemand::for_entry(EntrySelectionRef) -> SideDemands`; implementations are
pure and cannot perform I/O. It prevents inspect depending upward on sync while
keeping the pre-effect acquisition boundary in the inspector.

### Context meaning and recursive propagation

The context describes the policy applying **inside** the selected name. For an
admitted JW Q it can be `SelectedMirror { admitted_root, residual }`; compare does
not know that enum. A target-only child inside Q inherits this meaning even though
it has no source. A vanished Q encountered while searching inherits `Search`,
therefore preserves the target. Physical recursion role never grants region policy.

The initial context is bound and saved per lane. Structural-root assessment may
derive a different context for the root's contents. Store that derived context on
the accepted root frame even though current root gating otherwise hides the root
singleton. A restored root-directory application uses this stored input; it does
not call root admission again.

Replace `ChildEdge.members: Vec<usize>` with ordered contextual memberships:

```rust,ignore
pub struct ChildMember<C> { pub target: usize, pub context: C }
pub struct ChildEdge<C> { pub name: OsString, pub members: Vec<ChildMember<C>> }
pub struct LaneFrame<C> {
    pub target: usize,
    pub context: C, // policy input for this directory's immediate listing
    pub direct_changed: bool,
    pub direct_ancestor: bool,
    pub subtree_changed: bool,
}
```

An active child's member context must equal its lane frame's input context.
Accepted paired children are promoted into independently owned names and contexts;
they never retain a full selection arena. Termination/prune removes the lane's
context with its membership. Existing return-chain and empty-active-edge rules stay.

Sync's entry kernel takes the selected `Interior<C>` into **all** of
`copy_directory_selected`, `copy_directory_pending`, `reconcile_directory`,
`removable`, removal after preflight, replacement staging and preview. Their child
inspection calls use that context. A `RemoveForReplacement` preflight and the
subsequent removal use the same context and final logical coordinate; a temporary
staging path never becomes a policy coordinate. Conditional containers carry the
context along their runtime depth stack without creating paired edges.

`validate_context` is pure bind-time interpretation checking: known context variant,
valid residual/index references, admitted-root ancestry and correct semantic input
version. It must not rescan ancestors or repeat admission. Frozen shared indexes
may be rebound from descriptors; a process-local pointer/address is not a saved
context. Context bytes are data to validate, not proof that a historical caller
obtained authority honestly.
Live finalization calls the same policy check for its input and proposed interior
contexts before publication. Structural selection validation remains generic over
context values; policy interpretation validation remains with that policy. An
unchanged child may inherit a cloned context by default, but role and coordinate
validation still applies.

### Full visitors and API migration

`FullVisitor<V>` implements `Policy<Context = ()>` and uses the default stat-key
comparator. It projects the current `Names` and full `Operations` payloads, retaining
full metadata, source-link text, Both/Audit target-link text, root handling,
source-first error order and early/late precedence. Old `Check`, `Scope`, `Absence`
and Seed/Update/Prune composition stays in this adapter in its present order:
compatibility check/absence first, caller late visitor last.

Do not run a custom comparator through the old `comparison() -> Option<bool>`
contract. New policy views expose the explicit enum. A new policy can request full
facts if wanted; that does not require the legacy interpretation.

Use `Applier<P = Mirror, M = StatKey>` and `SyncRuntime<P, M>` for the new surface,
with `SyncCursor<P::Context>` holding only data. Existing convenience constructors
can construct a `FullVisitor<Steering<...>>`; they need not keep the old four-generic
ladder as the architecture of the new executor. Break source APIs where needed,
update all in-repo consumers, and publish coordinated pre-1.0 minor versions and
exact internal pins under the project's version policy. Behavior compatibility
for full visitors is required; universal source-compatibility adapters are not.
The `EffectBinding` extraction in section 9 belongs in
`transfer/sync/src/binding.rs`: validated physical roots and CopyPolicy can be lent
to a listed program without borrowing an unrelated ordinary visitor/comparator.

## 4. Comparison identity and truthful result states

```rust,ignore
pub struct SemanticId {
    pub family: String,
    pub version: u32,
    pub parameters: Vec<u8>, // canonical caller encoding; exact equality on bind
}

pub enum Comparison {
    NotAssessed,
    Equal,
    Different,
    Unsupported { code: u32 },
    Unresolved { code: u32 },
}

pub trait Comparator: Sync {
    fn identity(&self) -> &SemanticId;
    fn demand(&self, shape: Shape) -> SideDemands;
    fn compare(&self, input: ComparisonInput<'_>) -> Result<Comparison, CheckError>;
}
```

The engine supplies `NotAssessed` only when the selected assessment mode omits
comparison. A comparator cannot return it to disguise an unanswered request.
Codes are stable within comparator family/version, not saved error objects.

The shared structural wrapper establishes requested one-sided/supported-kind
differences and local equality of two directories. Leaf predicates compare only
compatible leaf shapes. No predicate can mark missing sides or a type collision
equal. Directory `Equal` means equal local container shape, never subtree proof.
Unsupported special physical kinds still allow only explicit preservation; they
do not become executable because a predicate knows something about them.

Implement two production predicates:

- `StatKey` retains current regular size/exact-mtime and literal-link behavior.
- `SizeAndLink` compares regular size and literal links, deliberately ignoring
  regular mtime. Its descriptor and documentation state the weaker property. It
  provides an independently meaningful test of predicate identity and selection.

`Unsupported` means the supplied rule cannot assess this supported input/domain;
`Unresolved` means requested satisfaction is not established with the available
evidence. Acquisition I/O failure remains an error, not either result. No endless
request-more-facts loop: a rule declares the finite current physical demand, then
returns its result. Future content/storage acquisition can extend the typed fact
families deliberately without changing the distinction among these states.

Baseline proposals are derived once. Assessed supported shape differences use the
current mirror proposal. Paired equal leaves propose KeepLeaf.
NotAssessed/Unsupported/Unresolved has **no baseline**; policy must explicitly
retain/preserve or authorize work. Structural directory descent can still be
explicitly selected without comparing leaf content. A skipped one-sided slot does
not contribute an observed difference merely because its work is obvious.

Store policy ID, context-schema meaning and comparator ID separately in each
`TargetBinding<C>`. Runtime binding compares all of them before filesystem access,
then validates saved contexts. Matching actions alone never establishes comparator
compatibility. A consumer with heterogeneous policies/predicates can supply one
closed Rust enum for each; no trait-object registry is needed.

### Coverage and result publication

Add per-state comparison counts to `DirectoryCoverage` and `Summary`:
`equal`, `different`, `not_assessed`, `unsupported`, `unresolved`. Excluded names
remain excluded rather than being counted as attempted comparisons. Rename the
misleading action-deviation `retained` field to `deviated` on the new surface;
renewal is a deviation but is not retention. Actual preservation/retention diagnostics
may be counted separately by the effect/policy report.

Record stopped **paired** descent explicitly instead of treating every action
deviation as a coverage hole. The new `Policy::scoped() == false` promises unscoped
comparison coverage, not that selected work equals baseline work. `Mirror` returns
false; the full adapter forwards the legacy visitor's conservative declaration.
Actual exclusion, stopped paired descent and explicit prune still limit coverage
regardless of that declaration. Equal-leaf renewal alone does not.

`Summary::is_clean()` requires complete relevant coverage, no comparison gaps and
no differences under its **bound comparator**. Expose that comparator in the report.
An empty legacy `DirDiff` alone says nothing about omitted or unresolved assessment.
The old full visitor/StatKey projection keeps its established meaning.
Distinguish traversal exhaustion from comparison completeness with explicit helpers;
remove `deviated == 0` as a universal completeness requirement. Thus a fully assessed
unscoped equal renewal can truthfully be comparison-clean and still require work.

`LaneReport::satisfied` continues to mean that the **selected policy work** completed,
not a post-effect rerun of the comparator. Preserve unresolved observed results in
the report even if proactive installation later succeeds. The default mirror
refuses an unresolved leaf rather than silently choosing KeepLeaf; an explicit
policy retention is reported as such. A later strict satisfaction consumer can
require a suitable predicate or constructive work result.

## 5. Installation authority and selected work

Keep the finite physical `Action` family. Add orthogonal selected installation
authority; do not manufacture a Difference to make InstallLeaf legal.

```rust,ignore
pub enum InstallIntent { ReconcileDifference, Renew }
pub enum DestinationGrant { AbsentOnly, ReplaceLeaf, ReplaceTree }
pub struct InstallAuthority {
    pub intent: InstallIntent,
    pub destination: DestinationGrant,
}
```

Grants are nested destination-shape permissions: AbsentOnly permits absence;
ReplaceLeaf permits absence or a supported leaf; ReplaceTree additionally permits
a supported directory, subject to coupled descendant constraints. None permits
replacing unsupported special kinds.

`ReconcileDifference` requires `Comparison::Different`. `Renew` permits Equal or
NotAssessed as well as Different/Unresolved/unsupported predicate outcomes, provided
the physical source kind is supported and the destination grant covers its actual
shape. `ReplaceTree` still requires coupled removability and inherited descendant
policy; it is not rm-rf permission. `AbsentOnly` never changes into replacement on
an unexpected collision. Regular renewal always prepares a new private inode and
renames it into place; never write through the old inode or hardlink it to donor.

An authority is required on InstallLeaf and coupled installing replacements,
including source-directory preparation. Other actions carry no leaf-install grant;
directory Create/CreateIfNeeded retain their inherent absent-container constraint.
`Renew` on a type-changing source-directory preparation means proactive selected
source installation, with the selected interior defining what contents are wanted.
The validator rejects unused/inconsistent authority fields instead of allowing
inert-looking grants to become latent permission.

Ordinary mirror derives its existing difference-based placement grants. A policy
selecting renewal must explicitly choose the grant. Source-only copy, target-only
removal and container authority keep their current shape restrictions. Native
obsolete-name removal requires a native-authorized operation; a prepared desired
list does not turn every unlisted target into source absence.

Add a derived `ApplicationNeed` with `None` or `LocalOrOpaque`. It is true for any
local namespace work **or** sync-owned interior, including target-only
`Directory { Keep, descend: true }` with zero immediate planned count. It is false
for a pure paired `Keep`/descent and retained leaves. Compute it from selected
actions/interiors, not `diff.is_empty()`, `planned == 0` or coverage flags.

Change plan publication deliberately:

- `PlanWalk` publishes when `has_observed_difference || needs_application`.
- Plain `Walk` remains a divergence-only comparison projection and filters work-only
  publications internally.
- The parallel frontier seals children at **every plan publication requiring caller
  control**, including clean proactive work. It retains facts until that publication.
- Coordinated groups already publish clean groups. Sync may mark proven
  `ApplicationNeed::None` selections accepted without invoking the kernel; it still
  preserves group/control/coverage semantics. This is not a zero-planned shortcut.

Paired child work is still assessed later. This avoids moving all transfer to a
new nonpersistent driver merely to handle proactive actions.

## 6. Fact acquisition and execution reuse

The sparse view is not `Option<Metadata>` standing in for presence:

```rust,ignore
pub struct SideFacts<'a> { /* present side, resolved kind + origin,
                             independently available attributes/link */ }
pub enum Available<T> { NotRequested, Known(T) }
pub struct Demand { pub attributes: bool, pub literal_link: bool }
pub struct SideDemands { pub source: Demand, pub target: Demand }
```

Kind resolution is its own first stage. Source and target presence surround these
views, and attributes remain actual `std::fs::Metadata`. No sparse fake Metadata.
Kinds can come from listing, binding-root metadata or admitted acquisition; the
view reports that origin. Literal links use native `OsStr` comparison.

Use one private acquisition cell per actual present entry. It owns an optional
unconsumed listed handle and cached outcomes for metadata and readlink. Unknown
kind resolving through metadata stores the **entire** acquired Metadata. Later
attributes demand reuses it, including for installation. Known kind demand does
not stat. A Windows listing's already-cached attributes populate the same cell.
Errors use shared original causes and are associated with the fact requested.

Shared inspection collects per-lane demands, acquires their source union once,
and exposes each lane only its declared view. A source readlink error needed by A
does not fail B which requested kind only. An attributes failure does not poison
a known-kind-only lane. If kind itself required that failed metadata, every lane
requiring kind is affected. Policy must not inspect an opportunistic stronger fact
merely because another lane happened to request it; otherwise cohort composition
would change semantics.

Deterministic errors remain lane-local: listing, names admission, kind/shape stage,
native-name/source-first requested-fact acquisition, comparison, final policy,
validation, execution-input acquisition. Within each stage collect indexed outcomes
before choosing the first error; thread completion order does not choose it.
The full adapter requests its old payload before its late callback and preserves
its existing observation-versus-late ordering. No errors from unused peer demands
are injected into that sequence.

Sync execution-demand examples:

| Selected work | Additional input demand |
| --- | --- |
| Regular install | Source attributes, retained through timestamp/permissions/byte accounting. |
| Link install | Source literal text and platform-required subtype; Unix does not need target text. |
| Directory creation | Source permissions through postorder finish. |
| Unix leaf removal | Resolved target kind; no target size/mtime/link text solely for unlink. |
| Windows link removal | Actual platform-required metadata/subtype. |
| Existing directory write lease | Current containing-directory permissions, acquired lazily only if work needs the lease. |

Hand already-acquired containing-directory permissions to `Writable::verified`
within their live route/effect scope. Newly created directories need actual
post-umask permissions; donor permissions do not substitute for that observation.
Output exact-mtime verification remains its own required operation.

## 7. Runtime resources, compact names and route lifetimes

### Names and promotion

Provide a private `NameTable` and stable `NameId` for each immutable listing/union.
On Unix, pack native bytes with checked ranges and expose `OsStrExt::from_bytes`;
on other platforms an owned native-string table is an acceptable first backend.
Do not make a wire format from process-local encoded bytes. Existing transport
continues using its platform-tagged native encoding.

The native observer owns the parent descriptor once per listing and references
names by index, eliminating the handle-name/merge-key clone. Sort indices by native
bytes. The finalized union owns its compact names and contains no per-entry joined
path. Policy views expose `Coordinate { parent: &Path, name: &OsStr }`, component
iteration and explicit `to_path_buf()` for consumers that need a full path. The
legacy full adapter may materialize its existing paths. Build diagnostics lazily.

Before discarding a batch, copy only admitted paired-child basenames into the
frontier's child-only storage. A remaining single child must not pin the full union
or source acquisition batch. Long/non-UTF8 names and exclusion order remain exact.
Keep rustix/std parsing; no raw-dirent parser or layout transmute is needed.

### Resource handle

Implement the previously selected `ObservationResources` contract from
[draft0](draft0.gpt6a.md#resource-control-and-dirent-buffers): configurable 64 KiB
native read size, lazily allocated maximum 16 idle buffers, default 1 MiB retained
capacity ceiling, finite EINVAL growth, no waiting for idle slots, coherent live
configuration updates and return under current settings. Expose backend activation;
explicit unsupported backend controls refuse. No environment lookup per entry.

Create one resource owner per outer convenience operation. New constructors accept
a shared handle. Pass it through immediate inspection, compare, sync recursion and
prepared lists. A raw-buffer lease ends after enumeration; it is independent of
name arenas, source fact cache and semantic state. Caller Rayon ownership stays.

### Route admission

Introduce a private `RouteEpoch` inside inspection's public opaque
`InspectionEpoch`, used for one joined API call. Compare and sync can borrow that
epoch across crate boundaries without constructing route certificates. It caches no-follow
route results and containing-directory metadata by physical root/path, including
shared source prefixes. It is lent to inspection and effect preparation, rather
than `refresh` and `apply` each performing the same full check within one call.

At each externally returned call boundary, discard route certificates. Thus a
standalone `refresh()` followed by `apply()` checks routes again, while `apply()`'s
internal refresh shares its epoch. `apply_many()` checks the shared source route
once and each selected target route independently. Restored bind checks active
routes before fresh observations. No context or source cache entry is a route
certificate. Own namespace changes invalidate affected target suffixes inside the
epoch; held parent permission/descriptor scopes retain their existing guarantees.

This is useful route reuse now without changing the existing stable-tree contract
or silently making an old saved admission valid against a newly replaced ancestor.
Count bind, route, listing/fact, output-verification and permission acquisitions
separately.

## 8. Both-missing and mixed cohorts

### Source session and bounded retention

Place `SourceSession` in inspection. It binds one physical source generation,
shares observation resources and owns a bounded cache of successful source listings
and acquired source facts. Compare and sync receive the same handle. It contains
no policy decisions, target observations, target grants or traversal progress.

Cache key is `(runtime source-generation identity, relative directory)`. Never
reuse merely because two source path strings match. Binding/restoring a new run
creates a new generation and empty cache; an explicit source-phase transition also
starts a new generation. This fits the caller's good-source/quiescence premise.
Cached early views retain the original enumeration hints. A previous lane's acquired
metadata must not silently strengthen a subsequent `names` callback's input; explicit
kind resolution and declared fact stages are where cached stronger facts become
available.

Use explicit default cache limits of 32 batches and 8 MiB **charged owned capacity**,
with zero disabling cross-call retention. Charge names, links and indexed fact
storage; distinguish this from RSS or the memory of a currently inspected wide
directory. Over-limit batches serve active consumers then are not retained. Evict
least-recently-used unpinned batches; do not block on cache space. Do not retain
arbitrary executable error causes across calls. Active-epoch failures still share
their original Arc cause with all demanding lanes.

Cached cells retain metadata obtained during kind resolution. Listing handles close
at the inspection boundary; a later demand for a fact never acquired may use the
bound path after fresh route admission. It must not reacquire attributes already
in the cell. Concurrent consumers coordinate per-cell acquisition, never hold the
whole-cache lock across filesystem I/O. Retention limits constrain the reusable
cache, not live width/depth already required by an active batch.

### One opaque cohort call

Add:

```rust,ignore
pub fn apply_many<P: Policy, M: Comparator>(
    &mut self,
    runtime: &SyncRuntime<P, M>,
    token: &GroupToken,
    targets: &[TargetId],
) -> Result<ApplicationsReport, Error>;
```

`ApplicationsReport` has ordered per-lane `Result<Report, Failure>` outcomes.
Invalid/duplicate/disposed subsets refuse atomically before effects. Fresh restored
decisions for **all pending lanes** still refresh and compare before any resumed
effect; this preserves the existing barrier promise. Return refresh failures with
their actual dispositions, and do not try dispatching newly terminated lanes.

For the selected application subset, sync owns one runtime `OpaqueCohort` that
groups compatible source-owning interiors by logical source coordinate. Each lane
retains physical target/staging path, role, context, permissions, needed/retained
state, partial actual totals and local failure. At an immediate opaque level:

1. Enumerate the source once for participating source lanes, and each existing
   target independently. An absent/newly-created target contributes known empty
   interior by the current construction invariant, not a global empty-root claim.
2. Perform lane-local policy/demand/whole-batch validation through the shared
   inspector. Union source demands only across otherwise-ready lanes.
3. Execute direct work, recurse into compatible source children together, and
   finish each lane's container/permission obligations in postorder.
4. Remove failed lanes from subsequent opaque work; join already-started work,
   restore their scopes and retain original causes/actual progress. A sibling lane
   does not inherit their stop flag or target failure.

Private replacement preparation has provisional progress until installation into
the final namespace. Preserve the current kernel's accounting: discarded staging
does not count as completed target installation; a failed rename after visible
target removal still reports that removal. Each lane owns its provisional totals
and restoration/cleanup errors independently, even when source observations share.

Extract primitive leaf preparation/install, directory setup/finish and permission
leases from `apply.rs`; single-lane recursive execution and the cohort use those
same primitives. `apply(target)` becomes a one-member call, preserving the ability
to pause with A accepted and B unstarted. Implement same-coordinate shared opaque
copy for source-only and source-directory replacement preparation. Target-only
removal remains lane-local because there is no shared source input. Coupled
directory-to-leaf preflight/removal retains its intact-refusal semantics.

Do not first recursively copy each lane and call the resulting cache hit a cohort.
The both-missing case must perform one source listing/acquisition per shared
immediate level even if the total subtree is larger than the retained cache limit.
One active grouped batch plus the runtime depth stack achieves that without a
whole-subtree source catalog. Immediate entry/fact arrays can be dropped after
lowering unstarted child descriptors and required finish values.

### Mixed ownership discriminator

For A missing Q and B already containing Q:

| Region | Owner |
| --- | --- |
| A/Q population | Sync's opaque call, through completion/failure. |
| B/Q comparison and its paired descendants | Compare's saved frontier, after the current application barrier. |
| Source observations under Q | Inspection's source session, shared when retained; never a recursion owner. |

A newly created Q never gains a paired edge. The opaque runner does not inspect B's
future target or accept B's future policy in advance. B can reuse retained source
observations after fresh route checks and its own target/policy work. If cache
eviction forces source reobservation, semantics stay identical; report a miss and
do not promise whole-tree source-once behavior for mixed scheduling.

This deliberately combines **guaranteed immediate sharing for a joint opaque
cohort** with **bounded opportunistic reuse across different recursive owners**.
It avoids both an unbounded source catalog and a second saved walker. All runtime
opaque state closes before `apply_many` returns; it introduces no interior pause.
JW's native design selects fail-fast ordered apply-one for its ordinary command.
That remains valid over the same cursor/source session. The generic apply-many
helper's healthy-peer continuation is not silently substituted for that consumer
policy; parent can select a different cohort failure contract explicitly if wanted.

## 9. Reusable prepared leaf lists and proactive inputs

The parent-authorized [native handoff](file:///home/rektide/src/jj-waltz/design/coordinated-refill/execution0.gpt6a.md#43-replacement-ownership-program)
makes this a **desired/owned program**, not just a faster `seed_leaves` loop.
Desired S and previously owned O are independent immutable inputs. Implement it
in `transfer/sync/src/listed.rs`, sharing validation/acquisition/effect primitives.
Do not implement another copier in JW.

### 9.1 Prepared inputs and exact provider API

```rust,ignore
pub enum LeafKind { RegularFile, Symlink }
pub enum LeafExpectation {
    SupportedLeaf, // compatibility seed: actual supported leaf kind
    Exact { kind: LeafKind, executable: Option<bool> },
}
pub struct DesiredLeaf {
    pub relative: PathBuf,
    pub expected: LeafExpectation,
}
pub struct PreparedLeafList {
    /* immutable desired leaves, parent groups, component trie, semantic identity */
}
pub struct OwnedLeafSet {
    /* immutable exact leaf paths and their structural prefixes; no physical facts */
}
impl PreparedLeafList {
    pub fn new<I, P>(paths: I) -> Result<Self, Failure>
    where I: IntoIterator<Item = P>, P: AsRef<Path>;
    pub fn from_specs(leaves: impl IntoIterator<Item = DesiredLeaf>)
        -> Result<Self, Failure>;
}

pub enum ListedIntent { CreateAbsent, RenewOwned }
pub struct ListedTarget<'a, G: ListedGuard> {
    pub id: &'a TargetId,
    pub binding: &'a EffectBinding,
    pub owned: &'a OwnedLeafSet,
    pub intent: ListedIntent,
    pub guard: &'a G,
}
pub trait ListedGuard: Sync {
    fn demand(&self, candidate: ListedCandidate<'_>) -> SideDemands;
    fn check(&self, candidate: ListedCandidate<'_>, facts: EntryFacts<'_>)
        -> Result<(), CheckError>;
}
pub fn reconcile_listed<G: ListedGuard>(
    source: &SourceSession,
    desired: &PreparedLeafList,
    targets: &[ListedTarget<'_, G>],
) -> Result<ListedReport, Error>;
```

`EffectBinding` is the validated physical-root/copy-policy portion extracted from
Applier; `Applier::binding()` lends it. Runtime policy/comparator are separate.
The listed runner installs proactive desired work, so its physical comparison is
NotAssessed. `ListedGuard` adds pure path/device/administration/source-attribute
refusal and demand; it cannot broaden ownership or silently drop required work.
Native code supplies the frozen O paths and declares RenewOwned. The provider
does not discover whether those paths were native-owned.

Constructors are filesystem-free. Normalize accepted normal-component spellings
as existing seed does; reject empty/absolute/escaping/NUL paths, duplicate aliases
and leaf/ancestor conflicts such as `a` plus `a/b` **within each leaf set**. Overlap
between desired and owned sets is expected and encodes transitions. `Exact` requires
the actual donor kind; `executable` is an optional regular-file requirement under
the supported platform contract. Refuse an unsupported executable-bit requirement
rather than guessing a cross-platform meaning. JW's initial native profile is Unix.

Prepare desired input once per S, owned input once per distinct O, share with Arc,
and reuse immutable indexes without building a fresh S trie per receiver. Bind each
lane's owned set independently. A semantic manifest identity describes its declared
names/expectations, not a certificate of donor contents. Native tree/commit identity
and cache bytes stay native-owned.

Define `ManifestId` in inspection as a versioned platform-tagged 32-byte digest.
Prepared constructors compute BLAKE3 once over an unambiguous length-prefixed
encoding of ordered native components and expectations, with separate desired/owned
domain tags (Unix bytes; Windows UTF-16 units). The small hashing dependency belongs
to sync's preparation, not observation of file contents. Rebinding a manifest
recomputes/checks this declared-data identity; it is neither a source-content hash
nor native authority. This avoids process-address identity and avoids storing a
second copy of the full manifest inside every selected entry.

Every desired coordinate is looked up directly, including a leaf missing on **both
physical sides**. Missing source, wrong kind or expected executable-bit mismatch
fails that obligation; it cannot vanish because `read_dir` did not return it.
Source-only physical children absent from the manifest never become candidates.
This route does not enumerate source directories to discover desired content.

### 9.2 Explicit desired domain in the common selection contract

Pull this distinction into the new contract **before stage 1 fixes its fields**:

```rust,ignore
pub enum SelectionDomain {
    PhysicalUnion,
    Listed { desired: ManifestId, owned: ManifestId },
}
pub enum SourceObservation {
    Present(Kind),
    Absent,       // actual union/lookup or established structural absence
    NotRequested,
}
pub enum RemovalAuthority {
    SourceAbsent,
    OwnedLeaf { owned: ManifestId, leaf: LeafId },
    CoupledOwned { owned: ManifestId, subtree: NodeId },
}
pub enum InteriorInput {
    PhysicalDirectory,
    Listed { desired: ManifestId, subtree: NodeId },
}
```

`DirectorySelection<C>` carries the domain and entry source observation; `Presence`
is a read-only projection for physical-union entries, not a forced classification
of a listed obligation. Listed batches are still grouped at a real containing
coordinate, but do not claim a complete physical union. Desired IDs, expected kinds,
owned authority and the interior input are semantic fields. Raw facts remain live.
Manifest IDs bind to actual prepared values, not arbitrary application assertions
accepted as sufficient at an external execution boundary.

Extend the one lower-level work validator accordingly. Physical-union deletion
still requires true source absence and SourceAbsent authority. Listed obsolete
deletion uses `SourceObservation::NotRequested` plus an exact OwnedLeaf grant; it
must **not** pretend an unrelated, unlisted physical source name is absent. Common
kind/action/coupling legality remains shared. Domain validation additionally proves
each manifest reference, expected desired slot, covered ownership prefix and
input-boundary relation against its supplied immutable manifests.

Physical full visitors keep their honest full-union projection. For the existing
listed seed Check interface, use a small `CheckedListedGuard` over the same actual
facts; do not project NotRequested as absent through the legacy visitor.

Add one semantic no-effect action, `Action::ConfirmAbsent`, for a listed obsolete
owned leaf whose target was actually found absent. It requires a declared owned
removal obligation, no desired leaf at/below that coordinate, NotRequested source
and absent target. It is invalid in PhysicalUnion. Do not manufacture a present
side or call an absent leaf KeepLeaf just to fit the old action table. A missing
no-follow target ancestor can establish absence of its declared descendants;
an intervening symlink cannot.
Interior roles describe supplied recursive inputs, not independent absence evidence:
a listed target-only removal input still carries NotRequested source and its
explicit owned grant. SourceAbsent may not be inferred from that role name.

`InteriorInput::Listed` is load-bearing: source-directory expansion must use the
manifest subtree slice. Calling today's `copy_directory_selected` unchanged would
enumerate and copy unselected donor children. Make recursive candidate acquisition
an explicit closed choice between physical directory and prepared manifest input,
while sharing the entry/permission/staging kernel. Compare accepts only
PhysicalUnion selections; the listed call owns its entire runtime recursion,
including matched routing directories. This is two concrete input domains, not an
open provider/handler framework or a second saved traversal model.
Its `ApplicationNeed` derives from the input domain too: a listed routing directory
can own unstarted manifest descendants despite having no direct namespace effect.
Source acquisition cells support either a real listed handle or a declared-path
lookup; both preserve the same caching and requested-fact error contracts.

### 9.3 Ownership trie and coupled transition ordering

Walk the component-wise union of desired/owned **declarations**, with actual target
shape checks at each relevant node. The source is queried only for desired leaves
and their required real directory routes. Use one shared source group across lanes;
targets, ownership decisions and failures remain independent.

| Declared node / observed target | Selected obligation and authority |
| --- | --- |
| Desired leaf; target absent | Install that exact source leaf. Missing source is an error even if target is also absent. |
| Desired leaf; target supported leaf | CreateAbsent refuses. RenewOwned requires an exact O leaf at that name; prepare fresh then replace even when equal. An S-only occupied local refuses. |
| Desired leaf; target directory | RenewOwned may collapse only an O-derived structural directory, with the complete coupled preflight below. An exact former O leaf unexpectedly now holding a directory refuses. |
| Desired descendants; target real directory | Retain container and traverse only the declared desired/owned children. Keep unrelated target contents. |
| Desired descendants; target absent | Create needed route(s), checking actual source directory attributes. No unrelated source descendants enter the desired domain. |
| Desired descendants; target supported leaf | RenewOwned requires exact O ownership of the blocking leaf. Prepare only the selected S subtree privately, then replace the owned leaf as one coupled transition. |
| Obsolete exact O leaf, outside a coupled transition | If absent, settle owned absence without an effect. If a supported leaf, remove under OwnedLeaf authority. If now a directory/special kind, refuse. |
| O-only structural directory | Retain it, including when it becomes empty. It is not an automatic prune obligation. |

CreateAbsent has an empty effect ownership set and permits no overwrite/removal;
passing a nonempty O index cannot broaden it. RenewOwned intersects O-derived
permission with the guard's requirements. Declarations for `.jj`/`.git` and device
boundaries are checked by the supplied guard at actual routes/entries.

**Collapse is planned at its ancestor before subordinate obsolete work.** If desired
S has leaf `q` and O has `q/a`, `q/b`, all removal beneath q belongs to the selected
q replacement. Do not enqueue a global O-minus-S removal pass first. Before staging,
chmod or dismantling q, enumerate its entire actual target subtree read-only:

- Every removable actual leaf must be an exact O member and pass kind/device/admin
  guards. An unowned local vetoes the entire collapse even if byte/stat-equal.
- A directory must be in O's structural prefix closure. An unrelated empty local
  directory also vetoes; no implicit authority arises merely from emptiness.
- An exact O leaf encountered as a directory is an obstruction, not permission
  for recursive deletion. Unsupported kinds and guarded descendants veto.
- Missing O descendants are already absent; they do not justify ignoring other
  actual children. The desired source leaf must also satisfy its source expectation.

On veto, both the local and all subordinate O leaves remain intact. Unrelated earlier
siblings may already have completed, as in today's directory-local failure contract.
On success, prepare the source leaf, remove the coupled subtree under the same
ownership context, and install. Keep the existing two-pass bounded-memory preflight
if retaining all plans is too large; the second pass must use the same scoped grant.

Expansion is similarly one owner: O leaf `p`, desired `p/a` and `p/b`. Stage only
those desired descendants and required containers. If donor also has `p/local`, it
is neither listed nor copied. Complete source/guard validation of each selected
batch precedes its effects; the old target leaf survives failure of private
preparation. Provisional staging progress becomes target progress only on install.

### 9.4 Full manifest coverage and native adoption

Return `ManifestCoverage` separately from effect counts and ordinary comparison:

```rust,ignore
pub struct ManifestCoverage {
    pub desired: u64,
    pub installed: u64,
    pub owned_obsolete: u64,
    pub owned_retired: u64, // old leaf removed/replaced/already absent, not unlink count
    pub complete: bool,
}
```

Only successful final-namespace installations count as installed. Exact native
desired lists require every regular/link leaf installed; keep/preserve is not
successful fulfillment. Every obsolete owned leaf must also be retired, including
those consumed by coupled collapse/expansion, counted once by ownership of that
transition. A retired O leaf's coordinate may now be a required S directory, so
retirement does not claim that its path is absent. Already-absent target evidence
can settle that retirement without an unlink. Existing structural directories are
not missing obligations.
Actual unlink/reflink/copy/create counters remain separate. Validate counter bounds
and derive complete only after all required work and cleanup/permission finishes
succeed. A returned error cannot carry `complete: true`; it retains accepted prefix
coverage and actual returned progress without a post-failure census.

For CreateAbsent, obsolete-owned obligations are empty. `seed_leaves` becomes a
small wrapper preparing SupportedLeaf specs and selecting that mode. The new
grouped pre-effect failure boundary is documented rather than confused with the old
leaf-prefix boundary. All listed calls close live resources before returning;
there is no saved interior manifest cursor or leaf-sized pause promise.

JW must gate S-cache adoption on successful **complete** manifest coverage, then
perform its native D restore/reparent/conformance sequence. Clear-first remains the
parent-selected default/control; ReplaceAuthorized uses the independently proved
same-tree-O child protocol. Generic provider success proves its declared installation
and owned-removal work, not JJ cache truth, native parentage or final-D sharing.
No fallback to the other route occurs on failure.

## 10. Saved schema and restoration

Use one explicit schema transition: comparison `Parts<C>` version **2**, sync
`SyncParts<C>` version **2**, and the example transport's next outer schema version.
Reject v1 by default. A silent v1→v2 migration would invent missing context,
comparator and installation authority. If historical checkpoint migration is later
needed, it must be an explicit tool for the exact unit-context/StatKey contract.

`Parts<C>` adds per-binding semantic IDs and initial contexts, contextual lane
frames/edges and expanded coverage counts. `SyncParts<C>` shares selection bodies
in memory and transports their values. Its barrier stores exact input and child
contexts, comparison state, authority, exclusions, selected actions and dispositions.
The wire codec does not serialize Arc identity, packed-name offsets or runtime keys.
The coordinated comparison/sync composition accepts PhysicalUnion selections only;
listed execution is a separately callable coarse operation using the same lower
selection contract. Its runtime manifest recursion is not silently inserted into
the paired frontier or serialized as partial opaque progress. If a native phase
is saved externally, that native owner binds the immutable manifest descriptors
and saves only the supported returned-call disposition.

Validation layers and order:

1. Codec rejects unsupported outer schema, malformed/oversized fields and invalid
   platform-native names under its existing complete-file size limit.
2. Inspection validates selection semantics; compare validates all exact frontier
   relationships/reductions; sync validates barrier/application relationships.
3. Bind matches policy/context/comparator/copy-policy interpretation **before I/O**,
   validates every required context against rebound frozen inputs, then reconstructs
   live source/runtime resources and validates active physical routes.
4. Before effects, refresh only unstarted applications together using their saved
   input contexts and exact descriptor-bound comparator. Compare entire selections,
   including descendant contexts and authority. Acquire execution inputs before any
   refreshed effect. Already accepted applications/ancestors do not refresh or replay.

Original binding indices stay stable when runtime lanes are sparse/reordered.
Terminal lanes need no executable policy or existing root. Continue to issue a new
runtime control token without reemitting an accepted group. `parts`/`into_parts`
must not fold partial frames; `prefix_summaries` remains nonmutating.

The source cache, route epoch, copied source Metadata, output descriptors and pool
settings are absent from saved state. Rebinding different resource budgets is
legal. Different comparator meaning is not. A stale checkpoint still has no
exactly-once guarantee and cannot recover an opaque failed copy.

## 11. Discriminating acceptance tests

These gates test consequences, not field-by-field implementation mirrors. Use
tiny filesystem fixtures plus counted/faulted acquisition hooks, then the existing
fresh-process consumer. No representative timing claim precedes these gates.

| Gate | Required discriminator |
| --- | --- |
| Shared selection ownership | A wide clean/work batch's plan, group and barrier point to one immutable body; with source retention disabled, dropping live preparation releases facts while the saved selection survives. Restored values compare semantically without Arc identity. |
| Common validation | Mutate decoded comparison/action/authority/context-role combinations; live finalization and restore reject the same invalid combinations. Sync still independently rejects barrier/binding mismatch. |
| Context qualification | Source admits Q once. Nested selected leaves do not restart selector/ancestor-ignore queries. A target-only descendant is removed under inherited Q; a vanished unadmitted Q is retained. |
| Residual exception | Remove a target-only region with preserved `cache/local/`; siblings remove, necessary ancestors survive. Directory→leaf replacement of that region refuses intact before staging/chmod/dismantling. |
| Mixed continuation | A lacks Q; B has Q and stale descendants. Apply A, save with B unstarted, resume in a fresh process. B retains context, removes only its authorized stale names; A is never replayed or added to paired traversal. |
| Deep partial heads | A admits `a/` and `b/`, B only `b/`; pause deep in `a/`. Restore exact contexts, memberships, return obligations and subsequent child order without replaying parents. |
| Context/comparator binding | Change a context schema, residual input or comparator parameters while actions would remain identical. Bind refuses before filesystem access. Changing only resource limits succeeds. |
| Predicate difference | Same-size differing-mtime regular pair: StatKey selects difference, SizeAndLink selects equal. Exercise paired, opaque context and restored unstarted application paths. |
| Honest results | Exercise NotAssessed, Unsupported and Unresolved separately. None certifies clean or manufactures absence; default unresolved mirror refuses, explicit renewal can still select installation. |
| Equal renewal publication | Equal regular leaf with Renew+ReplaceLeaf yields selected work in sequential and parallel PlanWalk and coordinated sync while baseline diff stays empty; an unscoped fully assessed renewal remains comparison-clean. Fresh private installation leaves source and old hardlinked sibling unmodified. |
| Zero-planned opaque work | Target-only Keep-container+descend has zero immediate namespace count but removes an admitted child. No-effect optimization must execute it. Clean paired-only groups perform no staging/chmod/write calls. |
| Full visitor contract | Late visitor sees all old metadata/link fields and can override earlier absence/Seed/Prune choices. Known/Unknown hints, explicit special preservation, root suppression and original ordered failure cases agree, including earlier source-readlink versus later kind failure. |
| Staged acquisition | Known directory pair needs no attributes under narrow policy; Unknown kind→attributes uses one acquisition; selected copy reuses it; type-mismatched target link avoids readlink. |
| Lane-local stronger demand | A requests source link text and gets a fault, B needs kind only. B succeeds; A keeps original cause. Repeat attributes-versus-known-kind and sequential/parallel ordered faults. |
| Whole-batch execution demand | Later sibling's required source attributes fail. Earlier sibling has zero effects in that immediate batch. Previously completed ancestor/other-lane effects remain truthful. |
| Both missing | Two absent Q targets with different nested preservation. With cross-call cache disabled and subtree larger than its normal capacity, joint opaque execution still lists/acquires each demanded shared source level once. Target decisions/results differ correctly. |
| Mixed bounded reuse | A opaque and B future paired: retained source batch avoids reacquisition, but B policy/target observation still runs. Tiny/zero capacity induces counted misses without changing output, coverage or recursive owners. |
| Cohort failure | One target fails clone/permissions mid-opaque work; peer completes. Both close arenas/leases; actual progress and original causes are separate. No further work starts for failed lane and no paired interior appears for it. |
| Owned child names | Long/non-UTF8 names survive batch drop. Retaining one pending child retains only child storage, not the complete name/fact arena; all native ordering/exclusion identities match. |
| Resource lifetimes | Buffer reuse at EOF and error, growth bound, concurrent borrow without waiting, reconfigure while leased, zero retention and unsupported backend controls. Save/rebind has no buffer/pool payload. |
| Route boundaries | Within apply-many, shared source prefixes are checked once. Replace a route with a symlink between returned calls: next call refuses. Internal refresh+apply does not duplicate checks; standalone refresh does not create a lasting certificate. |
| Prepared list | Prepare once, install into two targets without sorting/group-building twice or enumerating unlisted source. Duplicate and leaf/ancestor conflicts refuse before writes; occupied strict-create refuses even if policy tries to widen it; withheld required leaf cannot report satisfied; authorized equal renewal installs; unlisted local survives. |
| Desired coverage beyond physical union | Desired leaf missing on both sides, wrong donor kind and wrong executable bit each fail with incomplete manifest coverage. An earlier successful sibling remains reported. No state-adoption success can be inferred from effects alone. |
| Owned collapse veto | Desired leaf q, O leaves q/a and q/b, actual unowned q/local (also test an empty local directory). Refusal leaves q/a, q/b and local intact; no subordinate O-minus-S deletion, staging or chmod occurs first. |
| Listed expansion boundary | O leaf p expands to desired p/a and p/b while donor also contains p/unlisted. Only a/b enter preparation and final target. A failed selected source leaves old p intact; source/staging/target accounting stays distinct. |
| Owned obsolete semantics | Remove obsolete exact O leaves without querying unrelated physical source; already-absent obligations settle without a deletion count. Preserve empty structural directories. An O leaf now holding a directory refuses recursive deletion. |
| Complete manifest gate | Success has installed every desired leaf and settled every obsolete owned leaf once, including coupled transitions. Clone/cleanup/restoration failure cannot return complete coverage, even if all desired leaf renames happened. |
| Restore corruption | Context member/frame mismatch, wrong role, missing opaque context, omitted paired child, altered grant and lost partial reduction reject. Existing barrier/window/terminal-lane/counter-limit cases remain. |

Run existing conditional-container, replacement-intact-refusal, output-mtime,
required-reflink-no-fallback and explicit-subset-refresh regressions as relevant
slices change. Feature-disabled/default/all-feature checks and fresh-process tests
are integration gates; no need to rerun the entire matrix after every document or
private helper edit. Update examples and public Rustdoc with the API changes.

Place acquisition/selection discriminators beside `tree/inspect/src/shared_tests.rs`
and the new inspection modules; frontier/corruption cases in
`tree/compare/src/coordinated/tests.rs`; effect/barrier cases in
`transfer/sync/src/coordinated/tests.rs` and `transfer/sync/tests/visitor.rs`;
prepared-list cases beside its new module. Extend
`examples/reconcile/tests/process.rs` for actual process reconstruction. Keep
fault/counter hooks private to tests, as current acquisition substitution already
does. Buffer/name tests belong to `tree/observe`, not sync's native-consumer tests.

## 12. Sol implementation sequence and parent checkpoints

Each row is a coherent implementation stage with explicit owned-path commits.
Stages can span several commits; save predecessor state before review revisions.

| Order | Build / primary paths | Parent review and exit |
| --- | --- | --- |
| 1 | Share current decisions; then introduce inspect `selection.rs` and eliminate duplicated sync semantic validation. Start `Comparison`, declared versus physical input, source-observation state, install/removal authority, interior input, coverage and `ApplicationNeed` together. | Review the complete legality/coverage table, both-missing desired obligations, equal-renewal and zero-planned opaque discriminators. Do not freeze a replacement schema containing only old fields. |
| 2 | `policy.rs`, `comparison.rs`, contextual prepared batches; generic compare frames/edges and sync barrier. FullVisitor projection uses the common engine. | Review root-interior context transition, role table, unstarted-refresh input and exact equality. Complete contextual paired/opaque propagation before calling this feature delivered. |
| 3 | `acquire.rs`, `shared.rs`, observer listing/name representation and staged effect demand. Fix PlanWalk/parallel clean-work publication. | Full visitor behavior, second comparator, equal renewal and counted kind→attributes/source reuse pass. Inspect source/target failure isolation and fact release. |
| 4 | Version-2 parts/transport and fresh-process contextual resume. Integrate runtime resource handle, raw-buffer pool and RouteEpoch through the already-migrated call graph. | Deep partial heads, A-accepted/B-unstarted, bind-before-I/O mismatch and all unstarted-refresh preflight pass. Public APIs and saved shapes are now ready for JW integration. |
| 5 | Extract `kernel.rs`; add `opaque.rs` apply-many and bounded SourceSession cache sharing compare/sync. | Both-missing source-once with cache disabled, mixed bounded reuse, conditional ancestry, per-lane failure and closed runtime resources. No new saved recursive owner. |
| 6 | `listed.rs` prepared desired/owned tries and strict/RenewOwned execution. Bind expected donor kind/mode, owned removal and listed-only expansion to the common kernel/validator; retain convenience seed wrapper. | Full manifest coverage including both-missing names, collapse veto before subordinate deletion, no unlisted source expansion, no empty-directory pruning and prepare-once counts reviewed with parent. This is a prerequisite to native S-cache adoption. |
| 7 | Public owner guides, facade/CLI/examples, coordinated versions/pins and consumer integration receipts. | Relevant workspace feature matrix, strict clippy/Rustdoc, formatting and fresh-process suite pass; parent integrates native outcomes and bounded full-command measurements. |

The schema design is reviewed before stage 2 coding; stage 4 completes its codec
and process proof rather than postponing the decision. JW can extract shared native
inputs independently, then adopt contextual policy after stage 4 and shared opaque /
prepared installation after stages 5–6. Parent coordinates that dependency; this
document does not instruct another design agent.

### Decisions requiring parent confirmation

The recommendation is to accept these together:

1. **Intentional v2 checkpoint/API break** and contextual new core with a narrowly
   faithful FullVisitor adapter, rather than preserving old field layouts/generics.
2. **Apply-many as one coarse returned call**, retaining apply-one for selective
   pauses. Its budget charges one application per selected lane, not one for the
   entire cohort; no pause occurs inside that call.
3. **Guaranteed grouped opaque sharing plus bounded mixed reuse**, with 32-batch /
   8 MiB charged cache defaults configurable and excluded from saved meaning.
4. **Prepared proactive installation belongs in this provider build**; JW must
   supply complete desired/owned membership and native-state authority. The concrete
   handoff is section 9's manifest program. The replacement route cannot be
   implemented by merely disabling strict existence checks or physical union walking.
5. **Resource and name representation work is included now**, because otherwise
   staged acquisition and source-session ownership would immediately be rebuilt.

One optional performance choice remains deliberately local: retaining an entire
directory→leaf removability preflight can remove its second traversal, but imposes
subtree-sized retention. This design keeps the current bounded-memory two-pass
protocol initially and passes the same context through both. It can later use a
bounded retained preflight without changing selected-state or native authority.

## 13. Connections and handoff boundary

- [Rebuild](rebuild0.gpt6a.md#partial-traversal-is-the-saved-state) supplies the
  governing checkpoint semantics; opaque cohort state here is runtime-only.
- [Facts](/design/observation/facts0.gpt6am.md#3-smallest-demandacquisition-contract)
  supplies the demand table; this design chooses a unified engine plus full adapter
  rather than leaving two observation surfaces to diverge.
- [Modes](/design/transfer/modes0.gpt6am.md#1-mode-and-context-interface) supplies
  inherited authority and residual narrowing; the new context is a production
  carrier, not a global library enum of JW meanings.
- [Names](/design/observation/names0.gpt61sxh.md#lifetime-checkpoints) explains why
  frontier child promotion is mandatory even with a compact shared batch.
- [Dispatch](/design/actions/dispatch0.gpt61sxh.md#1-the-smallest-useful-boundary)
  separates desired work from physical classification; prepared physical renewal
  is the concrete implementation selected here, with ordinary finite dispatch.
- [Native execution](file:///home/rektide/src/jj-waltz/design/coordinated-refill/execution0.gpt6a.md#required-provider-handoff)
  supplies the concrete both-missing, ownership and coupled-collapse requirements.
  Section 9 integrates that now-authorized same-wave handoff; its installed-JJ proof
  remains attributed to the native design, not repeated by this provider study.
- [Architecture home](/doc/architecture/README.md#components-and-ownership) remains
  the maintained owner account. Parent should connect this design from the active
  delivery/index and promote verified contracts there as Sol builds them.

The provider handoff is typed region context, explicit predicate/work semantics,
source-session resources and truthful returned effects. JW's handoff back is its
closed context type, frozen policy inputs, concrete replacement grants and native
postconditions. Neither side can infer the other's authority from an empty diff.
