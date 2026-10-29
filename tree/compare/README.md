---
type: CrateGuide
title: Paired and coordinated tree comparison
description: Deterministic pair and source-to-many traversal, lane-local coverage, and owned partial-frontier continuation.
resource: /tree/compare/README.md
status: implemented
generated: { by: "model:openai/gpt-6.1-sol#xhigh", at: 2026-10-22 }
updated: { by: "model:openai/gpt-6-astra#xhigh", at: 2026-10-29 }
verification:
  scope: Paired traversal, facade compatibility, partial-frontier continuation and lane-specific closed-reduction validation; effect and transport integration are verified separately
  method: Workspace default and all-feature nextest, affected default/all-feature checks, strict workspace clippy, doctests, Rustdoc, formatting and Rekon document validation
  result: 214 workspace-default and 217 all-feature tests passed; 163 feature-disabled library/consumer tests and 26 focused all-feature coordinated tests passed; 20 workspace all-feature doctests and strict checks passed
sources:
  - resource: "opencode://ses_e7a2771c3ffez3sv1oPgD5gPH4"
    title: Reviews and promotes typed frontiers, initial execution demand, work-only publication and generic v2 comparison continuation
  - resource: /design/architecture/execution1.sol61x.md
    title: Verified 0.4 frontier checkpoint and remaining contextual effect-runtime boundary
  - resource: /design/architecture/execution3.sol61x.md
    title: Delivered cross-owner source retention and schema3 truthful comparison/application accounting
  - resource: "opencode://ses_e7e6c1941ffeFqyy1fEbFTKVmf"
    title: Reproduces and fixes inconsistent closed reductions on restoration while preserving lane-specific partial-child returns
  - resource: "opencode://ses_e7ea319dfffeZci454IqpWo77c"
    title: Implements coordinated comparison with partial ancestor state, per-call policy loans, validated restoration, independent controls and fresh-runtime continuation tests
  - resource: /tree/compare/src/coordinated/mod.rs
    title: Delivered source-to-many frontier, pure plan lowering, runtime binding and controls
  - resource: /tree/compare/src/coordinated/state.rs
    title: Owned semantic state and data-only restoration validation
  - resource: "opencode://ses_e9d314c29ffeboK450qrHM3qhF"
    title: Updated facade and API-test ownership after the final workspace relocation
  - resource: "opencode://ses_e9d737832ffeiy0f6OibOWOKSp"
    title: Extracted traversal and recursive audit while preserving frontier semantics and facade type identities
  - resource: /design/architecture/draft2.gpt6a.md#2-separate-immediate-inspection-from-traversal
    title: Accepted traversal boundary and preservation obligations
  - resource: /README.md#the-walk
    title: Governing paired descent, ordering and summary specification
---

# Paired and coordinated tree comparison

`diff-dirs-tree-compare` owns sequential paired DFS, the optional joined directory
frontier, coordinated source-to-many comparison, pruning, recursive read-only
audit and committed-prefix summaries.
Immediate inspection and validated plans come from
[`diff-dirs-tree-inspect`](../inspect/README.md); the same types are re-exported,
not copied or wrapped into incompatible public identities.

The private pair adapter promotes finalized child names and contexts independently
of each selection's facts. Plans with neither differences nor selected application
work release observations in the inspecting worker. Work-only plans publish and
seal paired descent just like divergent plans. One-sided interiors remain opaque
to comparison. Parallel J/K and joined control behavior remain; this crate never
performs filesystem effects.

Features forward down the dependency graph: `parallel`, `native-dirent`, and the
existing `parallel-diagnostics` / `parallel-bench` controls. Sequential execution
remains the default. The [`diff-dirs-tree-api` facade](../api/README.md) provides
convenient public exports; external API integration tests now live there.
Frontier unit tests live here and share only dependency-free fixture source with
inspection, avoiding cyclic dev dependencies.

## Coordinated comparison

The additive `coordinated` module is also exported by the facade. `Coordinated`
compares one source against a finite ordered set of destinations. Stable `TargetId`s
identify lanes; the original binding order orders outcomes, including after a
restoration that binds only a sparse subset of live destinations.

```rust,no_run
use diff_dirs_tree_compare::{Execution, NoVisitor};
use diff_dirs_tree_compare::coordinated::{Bindings, Coordinated, TargetBinding, TargetPolicy};
# fn main() -> Result<(), Box<dyn std::error::Error>> {
let bindings = Bindings {
    source: "/source".into(),
    targets: vec![TargetBinding::full(
        "receiver-a".into(), "/receiver-a".into(), "mirror/v1".into(),
    )],
};
let visitor = NoVisitor;
let policies = [TargetPolicy::full(
    &bindings.targets[0].id, &bindings.targets[0].policy, &visitor,
)];
let mut run = Coordinated::new(bindings.clone(), Execution::Sequential)?;
run.bind(&policies)?;
if let Some(group) = run.next_group(&policies)? {
    // Clean outcomes are included. Errors remain tagged with their target ID.
    for lane in group.outcomes { let compared = lane.outcome?; println!("{:?}", compared.diff); }
}
let parts = run.into_parts(); // partial root/ancestors and control window survive
let mut run = Coordinated::from_parts(parts, Execution::Sequential)?;
let reissued_control = run.current_token();
run.bind(&policies)?;
while run.next_group(&policies)?.is_some() {}
let summaries = run.prefix_summaries();
# let _ = (reissued_control, summaries);
# Ok(()) }
```

### Groups, independent lanes and controls

`Coordinated<C>` stores typed context in bindings, partial lane frames and ordered
child memberships. `TargetBinding::new` freezes policy/context-schema/comparator
identities and initial context; `TargetPolicy` supplies executable P/M values,
commonly borrowed references. The `full` constructors above select the legacy
full-visitor/StatKey interpretation. Binding validates interpretation and saved
contexts before filesystem access, without replaying admission. A root-derived
interior context survives as the accepted frame's actual input.

Each `next_group` attempts one complete immediate source-directory group, including
clean/empty groups. The shared inspector lists source once and acquires each
demanded admitted source-entry fact once for that epoch. Destination policies,
unions, errors and contributions remain independent. Paired child selections merge
in native-name DFS order with explicit destination memberships. Source-only,
target-only and colliding directory interiors remain opaque **per destination**.

Successful immediate contributions are accepted before delivery. This does not
complete the directory or root: accepted ancestors, unclosed child edges, pending
sibling suffixes and lane-local divergence evidence remain in `Parts`. `heads()`
shows destinations waiting at an ancestor while another lane walks deeper. A lane
can return/fold before peers finish their deeper work. Closed reductions occur
once in this lineage; `prefix_summaries()` only projects active ancestor evidence
and never closes or changes saved frames.

The returned token opens a control window until the next pull:

- `prune_here(&token, &targets)` removes only those successful lanes from this
  group's paired-child memberships and records their prune coverage.
- Subsets validate atomically: unknown, duplicated, failed, stopped, previously
  pruned, stale or foreign selections refuse without partial changes.
- `stop_lanes(&targets)` removes live lanes from **all** future memberships and
  ends them incomplete. Comparison failure likewise terminates only that lane,
  emitting its original error once; healthy lanes continue. Neither operation
  reattaches/retries the terminated lane. Start a fresh run for it.

Normal lane-root return marks that lane complete as a scan; exclusions,
preservation and explicit pruning still prevent `Summary::is_clean()`. An observed
diff is neither a mutation nor post-reconciliation verification.

### Parts, binding and token restoration

`new`, `Parts::fresh`, `Parts::validate` and `from_parts` perform no filesystem
access. Roots must be absolute, lexically normalized native paths. Validation
checks version, identities, paths, phases/control positions, ordered memberships,
active return edges, summary ordering, ancestry and checked counters. Closed
reductions cannot overlap a lane's unstarted children or lose evidence already
folded into a frame. Evidence inside that lane's still-active child need not yet
have propagated to its parent. Validation does not authenticate a checkpoint or
prove past observations/decisions.

`bind(&policies)` checks compatible descriptors **before I/O**, then validates real
live roots and active routing ancestors without replaying accepted root/ancestor
callbacks. Failed, stopped and completed roots may be missing and are not rebound.
Terminal binding/advancement/parts operations need no tree access. Future queued
source/target routes receive no-follow structural checks when they become current,
so a restored sibling replaced by a symlink is refused, not followed. These are
defenses under the stable/nonhostile topology contract, not race-resistant sandboxing.

Those explicit route metadata checks are separate from source listing/admitted
entry-fact acquisitions. `route_metadata_checks()` reports the extra checks for
this runtime, sharing common source checks within a group; it is reset on restoration
and excludes the inspector's mandatory root-binding observations.

Policies are short-lived `TargetPolicy` loans to `bind` and each pull, never stored
inside the cursor. The descriptor must name every behavior-affecting parameter;
the caller promises that matching descriptors mean matching pure behavior. This
supports temporary visitors borrowing a consumer's owned policy components without
self-references or cloning unknown visitor state. Inactive policies may be omitted.

`into_parts()` retains the open window and its already-pruned subsets, but not
runtime tokens. After restoration obtain a **new** token using `current_token()`;
old/foreign tokens refuse. Restoration does not reemit the accepted group. An
application owns its transport/schema, native path encoding and separate output
delivery. No codec, effects history, cross-process exactly-once promise or automatic
filesystem relocation is provided. Trees and frozen policy/topology must stay
stable across voluntary pauses; continuation is not a fresh scan after edits.

### Consumer lowering and resources

`next_group_with(&policies, |target, plan| value)` advances the **same frontier**,
while giving the consumer ownership of each successful `PreparedDirectory<C>`.
The consumer can lower exact selected decisions and optionally retain an ephemeral
plan for an opaque application, without another comparison traversal. Lowering is
pure projection, not effect acknowledgment; invocation order is unspecified, and
`value` never enters `Parts`. Returning `Result` as the payload permits a consumer's
separate lowering-refusal policy. Filesystem execution belongs to sync, not this API.

`next_group_prepared(&policies, &execution_demand, lower)` also completes selected
execution inputs during initial inspection, before publication. Comparison-only
wrappers use `NoExecutionDemand`. `PolicyWalk<P,M,E>`, `walk_with_policy`,
`walk_with_source` and `walk_in_pool_policy` use the existing sequential/parallel
planner with that same demand boundary. `PlanWalk` is its full-visitor projection;
plain `Walk` remains divergence-only. Equal renewal can therefore be comparison-clean
while publishing work. `Summary::is_exhausted()` reports traversal termination;
`is_complete()` additionally requires complete assessment coverage. `deviated`
replaces the old action-deviation field `retained` and is not a completeness veto.

Paired walkers retain one shared `SourceSession` across listings. `bind_with_source`
can lend the same owner to another composition. Resource identity is runtime-only;
the effect runtime has call-local route reuse and bounded successful source
retention now spans paired/opaque calls. Generic
v2 comparison transport and fresh-process partial-head tests are verified in the
[frontier checkpoint](/design/architecture/execution1.sol61x.md). The subsequent
[runtime checkpoint](/design/architecture/execution2.sol61x.md) verifies contextual
opaque effects and actual contextual A-accepted/B-unstarted execution.

The [0.6 reporting correction](/design/architecture/execution3.sol61x.md#separate-reporting-correction-and-v3)
adds genuine selected-boundary `Summary.retained` independently of action
`deviated`. Equal renewal is not retention. Saved parts now use version3; version2
mixed aggregates refuse rather than being reinterpreted. Neither selected counter
is a post-effect file census or equality check.

Inter-group traversal is sequential. `Execution::Parallel` uses the current Rayon
pool only for the shared inspector's within-group metadata work; callers can use
`pool.install(|| run.next_group(&policies))`. It neither creates/configures a pool
nor reinterprets pair-frontier J/K limits. Every returned group joins started work.

Storage remains width-sensitive: current destination/source unions, selected child
suffixes, memberships and summary vectors can grow. Full plans retained by a
consumer add their own fact ownership. No fixed-byte, latency or whole-tree-memory
bound is claimed. [Production coordinated sync](../../transfer/sync/README.md#coordinated-reconciliation-and-continuation)
and the [fresh-process consumer](../../examples/reconcile/README.md) compose this
frontier; their effect and transport guarantees have separate verification.
