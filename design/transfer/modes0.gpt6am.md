---
type: MechanismProposal
title: Inherited subtree modes with residual obligations
description: Explicit region contracts that eliminate general assessment without disguising discovery, transfer, reduction or pause costs; counted Rust model.
resource: /design/transfer/modes0.gpt6am.md
status: proposed
generated: { by: "model:openai/gpt-6-astra#medium", at: 2026-10-20 }
sources:
  - resource: "opencode://ses_ea7be36f9ffeIdi3IMghFGDqBD"
    title: Scoped mode design, source checks and five fake-observer tests distinguishing assessment savings from required source work
  - resource: /design/roadmap/brief0.gpt6a.md
    title: Independent research boundary and human constraints
  - resource: /design/execution/contracts0.gpt61sxh.md
    revision: 5d6c003a
    title: Owned frame and reduction contracts, especially section 5D commission
  - resource: /design/observation/facts0.gpt6am.md
    revision: 5423a71f
    title: Demand acquisition, cached kind escalation and existing source-only savings
  - resource: /design/transfer/policy0.gpt61sxh.md
    revision: a63595a9
    title: Separate copy, removal and preservation grants and unresolved JW policies
  - resource: /src/listing.rs
    revision: 5d6c003a
    title: Unknown default Unix hints, cached Windows attributes and pure hint access
  - resource: /sync/src/visitor.rs
    revision: 5d6c003a
    title: Late Update, Seed, Prune and caller override ordering
  - resource: /sync/src/apply.rs
    revision: 5d6c003a
    title: Source-only copy plan and recursive removal still perform child work
extensions:
  ticket: diffdir-tree-transfer-modes
  reading_boundary: Only independent brief, scoped ticket, authorized completed studies and targeted source; withheld statement, full roadmap, parent transcript and other current-wave drafts not consulted; no subagents
verification:
  method: Five standalone Rust fake-observer tests in ignored .test-agent/subtree-modes; targeted source inspection and document checks
  scope: Model counters, not production measurements, syscalls or timing; no real-tree probe, core modifications or native repository operations
---

# Inherited subtree modes

**Yes: once the caller supplies a sufficient region contract, recursion can become
operation-specific and stop asking the general assessor.** Carry the contract in
the child context, narrow it at exceptions, and retain the discovery, input and
finish obligations the chosen operation actually needs. “All this source is
needed” alone does not establish target absence, overwrite permission, child-policy
uniformity, or satisfaction.

## 1. Mode and context interface

Suggested vocabulary, not a production API migration:

```rust
enum Mode {
    Assess { policy: PolicyId },
    Preserve { reason: Reason },
    PopulateEmpty { empty: EmptyRegion, copy: CopyContract },
    ListedLeaves { remaining: OwnedLeafList, collision: CollisionRule },
    Satisfied { evidence: Satisfaction, reduction: ReductionEvidence },
    RemoveRegion { authority: RemovalGrant },
}
struct Context {
    region: Region,                 // source AND resolved destination coordinates
    mode: Mode,
    residual: ResidualPolicy,       // none, indexed exceptions, or dynamic policy
    claims: Claims,                 // reservations/ancestor shape obligations
    result: ResultContract,
    program: ProgramDescriptor,     // frozen semantic version and parameters
}
enum ReductionEvidence<A> { Sufficient(A), Unavailable }
enum Coverage { CompletedByWork, CompletedByProof, Preserved, Incomplete }
```

`specialize(parent, child_name, supplied_facts)` is pure and returns a child
context, a finite fact demand, or refusal. It is not an accessor that secretly
stats. A terminal context closes before descendant discovery; a nonterminal one
produces before/children/finish obligations using the existing owned-frame proposal.
Both logical region and actual source/target routes matter. Prefix matching uses
lossless native components, not display strings or textual `starts_with`.

| Mode | Necessary caller knowledge | Residual work / closure |
| --- | --- | --- |
| Preserve | Terminal authority covering both sides and every competing producer in this region | No interior discovery, comparison or effects. Parent listing/root checks remain. Close Preserved, not Equal or completed desired-copy. |
| PopulateEmpty | Desired source domain, valid destination container/route, scoped empty-target assertion, create authority, copy semantics, child exceptions | Discover source unless a complete list is supplied; obtain kind and copy attributes/link text/bytes; create containers and settle leaves. No target interior listing or general physical comparison. |
| ListedLeaves | Explicit owned leaf coordinates and routing, completeness **only if claimed**, per-leaf target collision rule | No source-directory discovery for unlisted inputs. Still resolve source attributes/bytes, ancestor shape and target existence unless covered by a separate empty assertion. Unlisted targets remain outside authority. |
| Satisfied | Predicate proving this target satisfies this desired domain, attributes, parameters and descendant policy; sufficient reduction evidence if required | No covered descent/effects. Close CompletedByProof; uncovered domains stay separate residual obligations. |
| RemoveRegion | Explicit grant to remove all contents and container, with child reservations resolved/waived by the competent owner | No per-descendant general policy if genuinely uniform. Still enumerate/classify for deletion, manage permissions and remove postorder. No automatic prune selection. |
| Assess | No adequate specialized contract, or dynamic policy still needed | Existing/full or explicitly sparse assessment, potentially specializing children after its decision. |

An `EmptyRegion` is supplied by a caller that just created a private container,
validated emptiness, or explicitly guarantees it under the out-of-band stability
agreement. It is not inferred from “source-only” in an incomplete listing. A root
already containing another owner's output is not globally empty: split disjoint
regions or use Assess. Unexpected collisions refuse under PopulateEmpty; they do
not silently become permission to overwrite.

An empty assertion is consumed as a **construction invariant**, not saved as the
claim that a partly populated directory stays physically empty. After progress,
pending creates use their unresolved destination contracts; retry reobserves the
pending goal, and can recognize its already-installed result. Do not rerun whole
directory emptiness checks after acknowledged children. No receipts or durable
effects history are needed; old/forked continuations have no exactly-once promise.

RemoveRegion cannot subsume a preserved descendant. With an exception, split the
removal into authorized child regions and retain required ancestors; if the goal
specifically requires removing the ancestor, refuse that incompatible goal before
dismantling it. Removing a directory to replace it with a leaf remains a coupled
authority check. A late discovered exception must not authorize partial dismantling
of a subtree whose existing contract promises intact refusal.

## 2. Mostly uniform copy and the precise fallback boundary

Take an asserted-empty destination region, with source:

```text
root/
  one two three four       # four regular files
  bulk/a bulk/b            # two more files
  hold/c hold/d            # one exceptional subtree
```

The caller chooses PopulateEmpty with terminal Preserve at `hold`. At root, a
prefix index tests six child names; the four files inherit PopulateEmpty, and
`bulk` inherits it with **no remaining residual selector**. `hold` closes Preserved
without reading its interior. There is no need to call the general assessor for
every copied file merely because one sibling differs. The intended copy domain
excludes `hold`; preserving a required output would not satisfy a stricter goal.

If the exception is `bulk/b` instead, `bulk` retains a narrowed prefix suffix until
its children are listed. `bulk/a` has no residual; `bulk/b` gets the exception.
Unknown dynamic child rules cannot be discarded on a guess. Carry a smaller
kind/name assessor if sufficient; otherwise re-enter Assess at that child only.
Name-selected preservation precedes kind demand, even when kind is Unknown.

For an **existing paired** `hold`, give that child an Assess context and an
independent paired binding; the root cannot assert emptiness over it. Root context
is then a partitioned operation: PopulateEmpty only in empty child regions, Assess
at `hold`. This is where mode reverts. No empty assertion is inherited into an
existing exception. Once the exception assessor proves an empty child or chooses
terminal preservation, that child can specialize again. A completed exception
returns an outcome to its parent, not a global reset of all sibling modes.

Keep the cheap selector and general policy counters separate: an indexed exception
test is still work, even when a full callback disappears. If membership itself
requires expensive classification, its cost belongs to the residual observer.

## 3. Demand and fact origin

Use `Hint::Known(kind, origin)` / `Unknown`; `Attributes` is independently
`NotRequested` or `Known(actual_metadata, origin)`. Presence, exclusion and known
unsupported kinds remain distinct. Suggested physical origins are Listing,
BoundRoot and AcquiredMetadata. Caller assertions and proof aggregates belong in
context evidence, not in physical metadata origin.

For Unknown K, explicit `acquire(Kind)` may stat once and cache the full metadata.
A later `acquire(Attributes)` reuses it: **one K→Attributes acquisition total**.
For Known K, kind acquisition does nothing, but copy-attribute demand still stats
once. Accessors never fetch. A Windows listing may already supply full attributes;
the model's Listing origin for kind alone is not a claim of extra Windows syscalls.

The fake cache deliberately retains Listing origin for a Known hint while marking
its later attributes Acquired. For Unknown resolved through stat, both become
Acquired. Do not relabel all consumed facts as “from listing” just because the entry
was originally enumerated. Cache lifetime is one active inspection/execution
handoff, not indefinite serialized `Metadata` validity.

Existing full Visitor compatibility explicitly requests every admitted side's
attributes and existing link payload before late policy and whole-batch validation.
It cannot expose sparse facts through today's unconditional metadata accessor.
The new operation-specific route reports physical comparison **NotAssessed**, not
Equal, Different or unsupported. For older visitors keep their comparison and
observation/error ordering, including errors before effects in the immediate batch.

`sync/src/visitor.rs:26–41,46–104` confirms that absence adaptation precedes caller
late policy and Update/Seed/Prune are late. They may be overridden. They cannot
be silently compiled into terminal early modes without suppressing late facts,
checks or overrides. Require an explicit new terminal contract, or prove an
adapter's declared composition is equivalent. The counted baseline below is a
simple fake full-demand assessor, **not a behavioral equivalence test for arbitrary
existing Visitors**.

## 4. Reduction and saved mode state

A scoped satisfaction predicate must bind target identity/region, desired inputs
and output-affecting parameters, content domain, metadata obligations and descendant
policy coverage. Directory mtime and source-only commit/tree equality are not such
proofs. Native tracked-tree identity cannot cover arbitrary ignored, generated or
receiving-private contents. Good-source knowledge is accepted; this adds no donor
validation loop. Evidence acquisition cost, if any, must be counted separately.

For the [owned finish specimen](/design/execution/contracts0.gpt61sxh.md#d-scoped-subtree-assessment-modes--diffdir-tree-transfer-modes),
the reducer wants exact settled-leaf counts. A proof that all desired contents
satisfy the goal does **not** reveal how many leaves there are:

1. Supply a sufficient aggregate with the same domain/result version; e.g. eight
   desired leaves with satisfaction coverage for all eight. Fold once on closure
   as CompletedByProof, keeping proof-settled count distinct from actual copies.
2. Choose a different result contract, such as region satisfaction with unknown
   descendant cardinality. Do not print unknown as zero or silently change the
   existing specimen's exact count contract.
3. Enumerate for reduction. Assessment/effects may still disappear, but discovery
   has not. If counts distinguish kinds, retain the required kind acquisition.

For mixed regions an aggregate must be partitionable or already exclude residual
regions; a total eight cannot be reused after preserving two leaves unless the
caller also supplies that partition. Preserve and explicit skip remain distinct:
the former is complete within its preservation policy but not desired-copy proof;
the latter propagates Incomplete. Neither fabricates descendant counts. Aggregate
arithmetic and one-time parent folding remain the frame owner's job.

Save mode, narrowed residual reference/state, region binding, outstanding child
suffix, pending goal and finish/reduction inputs in the frame. Program/policy and
aggregate schema versions must match on bind. Saved proof evidence must be meaningful
under the declared stable bindings; a label alone is not proof. Rebind executable
observers and effectors; runtime handles and permission leases stay outside saved
state. Existing accepted prefix is not recollected merely to resume an exception.

An implementation may perform whole-subtree work behind one interface, but then
it must expose an inner continuation if child-sized pause is promised. Charge one
listing/assessment attempt and one resource-closed container/leaf/finish effect
attempt at declared boundaries. An uninterruptible recursive copy/remove is one
**coarse**, potentially huge effect, not bounded work. Leaf bytes and listing width
remain unbounded by an effect-count limit. Permission restoration must finish at
the selected boundary or the continuation contract must explicitly change; this
probe does not implement permission leases or byte-chunked copying.

## 5. Actual counted probe

Ignored scratch: `.test-agent/subtree-modes/{README.md,model.rs,tests}`. Standalone
`rustc --edition=2024 --test` builds the model, with no crate dependencies or
production imports. Five tests passed: counted traversal modes; pure accessors,
origin and escalation; paired exception demand; full-demand compatibility;
sufficient-aggregate closure. No real tree, benchmark or native workspace was used.
The model represents ten descendants/eight files; it allocates no file payloads.

`lists` counts successful source-directory enumeration calls; `entries` yielded
source names; `A` explicit source metadata acquisitions; `policy` full callback
slots; `compare` physical classification slots (one-sided here, **not byte compares**);
`residual` prefix selector checks; `copies` selected regular-file copies, not real
I/O. Roots/parent binding, target guards, permissions, link work, staging, output
mtime verification and kernel internals are outside these counters.

| Model run | lists | entries | A | policy | compare | residual | copies |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Full general, empty target | 3 | 10 | 10 | 10 | 10 | 0 | 8 |
| PopulateEmpty | 3 | 10 | 10 | 0 | 0 | 0 | 8 |
| Full general + terminal hold | 2 | 8 | 7 | 7 | 7 | 6 | 6 |
| PopulateEmpty + terminal hold | 2 | 8 | 7 | 0 | 0 | 6 | 6 |
| Preserve root | 0 | 0 | 0 | 0 | 0 | 0 | 0 |

Both Known and Unknown hints produce **identical rows**: copy execution requires
attributes anyway, and Unknown's classification acquisition is cached. Uniform
populate saves ten full policy calls and ten one-sided classification calls;
mixed populate saves seven of each versus the same preserved-domain baseline.
It saves **zero source listings, entries, metadata acquisitions or file copies**
against those baselines. Target interiors are unlisted in both empty-target paths.
`sync/src/apply.rs:589–602` already constructs the source-only plan with target
`None`; that omission is not a new optimization. Terminal name preservation is
also already available; its row demonstrates contract composition, not invention.

The paired exception microcase models a real directory pair with two regular
leaves on each side: two listings, four yielded side entries, two policy and two
comparison slots. Sparse Known-K descent needs four attributes; Unknown needs six
(directory pair plus four leaf sides). Full-demand compatibility needs six for
either hint variant and exposes all six. Thus Known K saves two classification-only
acquisitions there; it saves none on copied files. The fixture does not execute
comparisons against real bytes or pretend to measure their timing.

Reduction assertions: exact-count proof without aggregate refuses closure;
proof plus eight closes with eight; a cardinality-free result closes with unknown;
inadequate proof coverage refuses even with aggregate eight. Preserve and Incomplete
are tested unequal to proof closure. These are protocol discriminators, not a
production driver/codec test. The scratch is ignored and uncommitted; the fixture,
counter definitions and expected values above preserve its evidence in this report.

## 6. Meaningful integration gates and limits

Before production adoption, test the actual observer/driver seams rather than
repeating the model as implementation-shaped tests:

- Mixed copy with one preserved nested region: verify untouched target exception,
  required source bytes copied, no interior observation at preservation, and
  prefix residual removed only on disjoint branches. Include Unknown K.
- Existing paired exception: demonstrate PopulateEmpty never extends across it,
  two changed/equal leaves use declared comparison, and siblings remain specialized.
- Full late Visitor requesting attributes and overriding an earlier retention:
  compatibility preserves payloads and override behavior; terminal mode requires
  separate explicit selection. Include an acquisition failure before any batch effect.
- Exact-count finish with satisfaction proof: no aggregate refuses or enters
  reduction enumeration; sufficient aggregate closes/folds once; partial coverage
  and wrong domain/version reject. Pause before fold and resume without double count.
- Explicit listed leaves: no unrelated discovery; existing listed target obeys
  strict-create refusal versus separately selected overwrite; unlisted extras stay.
- Explicit removal: child-sized postorder continuation, permission restoration and
  retained-child incompatibility; no inference of removal from missing source.
- Pause after one populated child: resume remaining goals without reasserting global
  emptiness or rediscovering accepted siblings; unexpected target collision refuses.

Those gates are proposed, not reported run. This probe does not establish complete
error ordering, symlink/platform behavior, permission handoff savings, real-copy
correctness or continuation serialization. Existing full apply remains the owner
of its coarser contract. No recurring donor proofs, recovery mechanisms, receipts,
automatic cleanup or cross-process exactly-once guarantee is proposed.

## 7. Generic intent versus JW adoption

The modes implement explicit ordinary-directory intents. Mirror, refresh, seed,
strict listed seed, preservation and whole-region removal remain separate choices.
Copy selection never grants absence authority. The
[policy study](/design/transfer/policy0.gpt61sxh.md#8-a-few-meaningful-configurations-not-a-flag-cross-product)
owns unresolved JW breadth, initial snapshot classification, vanished selected roots,
native reservations and clear semantics. This proposal selects no JW default.

The [facts study](/design/observation/facts0.gpt6am.md#6-whole-subtree-knowledge-what-disappears-and-what-remains)
supplies the acquisition boundary; this report adds inherited residual contexts and
measured model counts. The [execution synthesis](/design/execution/contracts0.gpt61sxh.md#2-progress-acceptance-and-failure)
supplies owned progression; this report makes mode evidence and reducer sufficiency
part of that state. Parent owns integration, shared indexes and any later comparison
with withheld material. Only this document is committed; the scoped ticket gets a
findings/pointer/session note without closure or export.
