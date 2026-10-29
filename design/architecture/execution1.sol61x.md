---
type: ImplementationAccount
title: Typed contextual frontiers, work publication and generic v2 data
description: Verified provider frontier checkpoint, its production comparison and transport interfaces, and the exact remaining contextual effect-runtime migration.
resource: /design/architecture/execution1.sol61x.md
status: implemented-frontier-checkpoint
generated: { by: "model:openai/gpt-6.1-sol#xhigh", at: 2026-10-29 }
sources:
  - resource: "opencode://ses_e79d26e21ffestzkMAfQRZS702"
    title: Implements generic contextual comparison and saved data, shared typed planning and work seals, comparison coverage, generic v2 transport and fresh-process frontier discriminators; returns the compiling frontier boundary before effect migration
  - resource: "opencode://ses_e7a2771c3ffez3sv1oPgD5gPH4"
    title: Parent commissions the next contextual checkpoint after JW foundation adoption, authorizes the completed kernel handoff and owns maintained documentation and native integration
  - resource: /design/architecture/execution0.gpt6a.md
    title: Accepted contextual selection, policy, comparator, traversal and effect architecture
  - resource: /design/architecture/execution0.sol61x.md
    title: Actual selection and immediate-inspection foundation, including the unit-context executor and transport boundary inherited here
  - resource: /design/architecture/delivery0.gpt6ax.md
    title: Joined provider and native commission, checkpoint ownership and integration decisions
  - resource: /design/architecture/kernel-handoff0.gpt6a.md
    revision: 1c36cbc9
    title: Parent-accepted private effect-call, placement, recursion, sparse-fact, provisional-progress and route-epoch handoff; read only after explicit authorization
verification:
  method: Focused contextual discriminators, integrated nextest suites, feature-disabled owner tests, strict Clippy, Rustdoc, formatting and explicit Markdown/provenance checking
  scope: Linux contextual paired traversal and persisted data plus established full-visitor effects; not end-to-end contextual sync, grouped opaque execution, route reuse, bounded source retention, listed execution or JW 0.4 adoption
extensions:
  parent: "opencode://ses_e7a2771c3ffez3sv1oPgD5gPH4"
  code_checkpoints: [139cdf7a, 7ed693b6]
---

# Contextual frontier checkpoint

**Typed contextual comparison, planning and v2 data are implemented and verified.**
This is the permitted compiling frontier checkpoint, **not an end-to-end generic
contextual executor**. The remaining effect migration is one consequential slice;
it has not been hidden behind a second driver or a unit-context opaque fallback.

Code checkpoints:

- `139cdf7a` — contextual bindings, memberships, partial frames, pre-I/O interpretation
  checking, explicit summary coverage and generic sync data.
- `7ed693b6` — the existing sequential/parallel planning engine made contextual,
  clean-work publication and seals, initial execution-demand plumbing, shared paired
  source ownership, generic v2 transport, process evidence and coordinated 0.4 pins.

No JW production files, maintained provider guides, indexes or tickets were edited.
The pre-existing `.beads/issues.jsonl` changes were left outside both commits.

## Delivered interfaces and ownership

| Owner | Delivered seam |
| --- | --- |
| [Coordinated data](/tree/compare/src/coordinated/state.rs) | `TargetBinding<C>`, `Bindings<C>`, `ChildMember<C>`, `ChildEdge<C>`, `LaneFrame<C>`, `GroupFrame<C>` and `Parts<C>`. |
| [Coordinated runtime](/tree/compare/src/coordinated/mod.rs) | `Coordinated<C>` and executable `TargetPolicy<P, M>`; separate caller descriptor, policy ID, context-schema ID, comparator ID and initial context. |
| [Planning](/tree/compare/src/walk.rs), [parallel frontier](/tree/compare/src/frontier.rs) | `PolicyWalk<P, M, E>` over the existing DFS/reducer, with typed child jobs and live `PreparedDirectory<C>` publications. |
| [Borrowed-pool planning](/tree/compare/src/parallel.rs) | `ParallelPolicyWalk` and `walk_in_pool_policy`, using the same joined frontier and J/K controls. |
| [Pair preparation](/tree/inspect/src/roots.rs), [pair lowering](/tree/compare/src/pair.rs) | Generic immediate preparation, actual retained binding-root metadata, promoted child names/contexts and execution demand before publication. |
| [Summary](/tree/compare/src/model.rs) | Bound comparator, per-assessment counts, stopped-paired coverage and `deviated`, separate from physical entry counts and planned work. |
| [Sync data](/transfer/sync/src/coordinated/state.rs), [decision alias](/transfer/sync/src/coordinated/decision.rs) | `SyncParts<C>`, `Barrier<C>`, `Application<C>` and `DirectoryDecision<C> = Arc<DirectorySelection<C>>`. No second selection body or legality table. |
| [Application codec](/examples/reconcile/src/transport.rs) | Generic v2 DTOs, including initial/frame/member/input/interior contexts and all interpretation descriptors; comparison-only and sync-state codecs share the same comparison DTO. |

`TargetPolicy` owns its small P/M values. Usual contextual loans use `P = &Policy`
and `M = &Comparator`; reference comparators now forward the same trait. The
`TargetPolicy::full` and `TargetBinding::full` constructors provide the full-visitor /
StatKey compatibility interpretation without an executable registry. Context remains
owned semantic data, never a lowering callback payload.

### Bind and continuation invariants

Binding checks all supplied identities and initial-context equality before root
I/O. It then runs pure policy interpretation checks for saved initial inputs,
accepted frame inputs and queued paired-child contexts. No early/late admission
callback or accepted ancestor listing is replayed. A missing root cannot conceal a
changed schema, comparator or invalid saved residual.

The initial root gate may derive a new context for the root listing. That derived
input is saved on the accepted frame, not replaced by `initial_context()` during
restoration. Promoted paired children carry independently owned names and contexts;
they do not retain a complete selection/fact arena.

Structural validation requires an active child's ordered `(target, context)`
membership to equal its frame's lanes exactly. Pruning, failure and termination
remove memberships with their contexts. Existing independently waiting heads,
return edges, postorder reductions and empty-active-edge rules remain intact.

Generic sync data additionally checks selection interpretation against its binding,
selection input against the accepted lane frame, and each selected paired child
against its contextual edge. It still uses inspection's one common validator.
Coordinated sync accepts only `PhysicalUnion` / Compare selections; declared/listed
authority fields remain enforced by the common model, not executed here.

## Work publication, sparse preparation and compatibility

The shared plan-publication condition is now observed divergence **or**
`ApplicationNeed::LocalOrOpaque`. A clean renewal retains its actual preparation
and is delivered to the caller. Every such parallel publication seals its paired
children until the next joined call; speculative ready clean-work nodes are also
sealed. A caller can prune at that work-only publication.

`PlanWalk<V>` is a narrow full-visitor projection over `PolicyWalk<FullVisitor<V>>`,
not another traversal implementation. Plain `Walk` and `ParallelWalk` continue to
filter work-only publications from their divergence-only outputs. The rich and
plain paths retain the same ordered reducer, explicit prune semantics and caller
pool ownership.

New entry points:

```rust,ignore
walk_with_policy(source, target, options, policy, comparator, execution_demand)
walk_with_source(source_session_arc, target, options, policy, comparator, execution_demand)
walk_in_pool_policy(source, target, pool, limits, policy, comparator, execution_demand)
coordinated.next_group_prepared(&policies, &execution_demand, lower)
```

Comparison-only wrappers supply `NoExecutionDemand`. Both generic pair planning
and coordinated initial publication pass E into the actual inspector, before
lowering or fact release. The focused omitted-comparison renewal test distinguishes
NoExecutionDemand from a source-attributes request: only the latter exposes actual
source metadata in the initial prepared publication; target attributes stay absent.
This is not a late fallback stat or synthetic full-fact projection.

The full compatibility projection can consume a genuinely full unit preparation
through `into_full_plan`; it refuses other interpretation IDs or missing legacy
facts without acquiring anything. Existing full-visitor override/error behavior
and all established effect regressions remain covered by the integrated tests.

`Summary::is_exhausted()` is traversal termination. `is_complete()` additionally
requires unscoped, unexcluded, unpruned and complete assessment coverage, including
no stopped paired descent. `deviated == 0` is no longer a completeness condition.
`is_clean()` means no observed differences under the saved comparator and that
complete coverage. Thus equal renewal can be comparison-clean **and publish work**.
It has not yet been executed through a generic contextual effect kernel.

## Runtime resources and v2 process evidence

`PolicyWalk` and `Coordinated` retain one `Arc<SourceSession>` across their paired
calls. `walk_with_source`, `bind_with_source` and `source_session` expose a real
shared owner for subsequent effect integration. Tests check Arc identity and
unchanged preparation generation across actual recursive paired listings.

This does **not** finish session propagation through the old opaque executor.
`InspectionEpoch` remains empty; there are no route certificates or invalidation
methods yet. Existing route checks and caller quiescence/confinement obligations
remain active. No route-reuse, source-cache or syscall improvement is claimed.

The v2 codec is now generic over C rather than silently replacing context with
`()`. `Comparison<C>::new/decode` supports comparison-only applications without
fabricating a sync barrier. `Snapshot<C>` supports honest unstarted contextual
barrier data; `read_context<C>` is the generic reader and `read` preserves the unit
example convenience. Native names retain the established platform encoding.

[Contextual process tests](/examples/reconcile/tests/contextual.rs) use a concrete
admitted/depth/residual context and real fresh processes. They save a three-frame
partial ancestor chain with a different lane waiting at another head, restore it,
and verify that root/accepted ancestor callbacks do not replay. Separate fresh
processes reject changed context schema, comparator and residual before route I/O
even after the source root has been moved away. A generic unstarted barrier round
trip preserves exact selection/input/interior/child contexts; corruption refuses.

**There is no new contextual A-accepted/B-unstarted effect-process claim.** The
existing unit-context process suite still verifies that production behavior. The
new generic barrier test leaves both applications unstarted; it does not pretend
to have performed contextual effects.

## Verification receipts

| Check | Observed result |
| --- | --- |
| `cargo nextest run --workspace --all-features --no-fail-fast` | 245 passed. This includes the first five contextual tests and all existing owner/consumer regressions. Nextest emitted existing perf-runner build-script-output-path warnings, without test failures. |
| Inspect/compare/sync nextest with `--no-default-features` | 148 passed. |
| Reconcile example all-feature nextest | 18 passed, including the three new contextual codec/process cases and the established production process suite. |
| Final compare `context_tests` filter, all features | Six passed after adding the initial sparse-publication discriminator. No second whole-workspace matrix was run for that additive test. |
| Final sparse-publication filter, feature-disabled | One passed. |
| Strict workspace Clippy, all targets | Passed with all features and with default features disabled, including the final discriminator. |
| Strict reconcile example Clippy, all targets/all features | Passed. |
| Cursor and walk example all-target/all-feature checks | Passed; receiving locks updated to 0.4. |
| Workspace no-dependency/all-feature Rustdoc and final formatting | Passed. |

The retained full-visitor tests include conditional ancestry, selected empty
directories, zero-planned target-only keep+descend, intact collapse veto, failed
private preparation, original error causes, permission/partial progress, exact
mtime/private clone behavior, explicit-subset refresh and corrupt-state refusal.
They establish predecessor effect compatibility, not contextual opaque execution.

## Exact next effect-runtime slice

The [accepted kernel handoff](kernel-handoff0.gpt6a.md) remains the implementation
contract. Start a fresh effect owner at this compiling frontier; do not retain the
old public generic ladder as the destination architecture.

1. **Migrate executable owners together.** Replace `Applier<C,S,A,V>` and
   `SyncRuntime<C,S,A,V>` with P/M ownership and `SyncCursor<P::Context>`. Keep full
   compatibility as owned `FullVisitor<...>` adapters in the existing order:
   check/scope/absence first, caller late visitor last. The current production
   cursor/runtime remain unit-context; generic SyncParts alone is not that migration.
2. **Use the new initial preparation seam.** Change SyncCursor's current
   `next_group_with(... into_full_plan())` compatibility bridge to
   `next_group_prepared(... PhysicalExecutionDemand ...)`, retaining
   `PreparedDirectory<C>` directly. The same selected Arc must remain in group,
   barrier and live preparation. Refresh all pending saved decisions with their
   exact saved inputs through `inspect_group` / AlreadyAdmitted, under that same
   source session, and compare whole selections before any resumed effect.
3. **Separate call, lane and placement from primitives.** Implement the accepted
   EffectCall/LaneCall/BatchPlace/RecursionOwner seams and policy-free leaf,
   publication, directory lease/finish and private cleanup primitives. Perform pure
   whole-batch required-input extraction before opening any effect scope. A public
   preparation is not permission to select arbitrary staging/output mappings.
4. **Carry selected context everywhere.** Migrate copy, conditional ancestry,
   target-only reconciliation, replacement preparation, removability preflight,
   second-pass removal and preview together. Logical coordinates stay final while
   physical output may be private. Retained descendants veto coupled collapse
   before any subordinate dismantling. Private-tree progress stays locally
   provisional until publication; visible siblings/removals must never be erased.
5. **Close runtime lifetimes.** Thread the real shared source owner through paired
   and opaque work. Implement one call-local route epoch with no-follow absolute
   ancestry admission and namespace/permission invalidation. Internal refresh and
   application share that epoch; separately returned calls do not. The present
   empty epoch is not reuse.
6. **Prove the whole loop.** Actual contextual A-accepted/B-unstarted fresh-process
   resume, SizeAndLink effects/restoration, inherited target-only policy, conditional
   and replacement paths, sparse later-input failure before any batch effect, and
   unchanged error/permission/provisional-progress cases remain required.

Grouped `apply_many`, charged-capacity cross-call source retention and prepared
desired/owned execution remain later owners. The declared acquisition entry point
must live in inspect and reuse its cells/common validator; no public constructor
from selection plus metadata. `InteriorInput::Listed` remains the accepted enclosing
desired slice, including O-only/collapse descendants with no desired entry. Do not
invent source absence, a second saved input enum or a provider-local desired copier.

## Versions and native integration

All six provider product packages are **0.4.0**, with exact internal pins. Workspace
and all three excluded-example receiving locks are refreshed. The public additions,
generic comparison parts, new binding fields and `Summary.retained → deviated`
justify this minor API marker. No publish or push was performed.

JW's accepted 0.3 adoption/20-case evidence belongs to that predecessor revision,
not this one. Its owner must deliberately update exact provider pins/receiving lock
and rebuild; any use of `Summary.retained` becomes `deviated`, and direct binding
literals should use `TargetBinding::full` or supply all frozen interpretation fields.
Existing full Applier/SyncRuntime construction remains available in this frontier
checkpoint. **Do not cut over JW's contextual ordinary policy yet.** That requires
the direct prepared/opaque runtime slice above, with its own reviewed API marker
and real command verification.

The v2 DTO now requires the completed contextual binding/coverage fields. Earlier
unit-only v2 documents missing them refuse; v1 still refuses. No silent checkpoint
migration is claimed. Parent owns maintained guide/index promotion and ticket/native
coordination after reviewing this bounded result.
