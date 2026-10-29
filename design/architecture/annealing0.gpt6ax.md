---
type: ArchitectureAssessment
title: Annealing coordinated reconciliation into the next delivery
description: Integrates trajectory1, subsequent human steering and the broader research into present ownership decisions, a contextual persisted execution slice and a cross-project delivery sequence.
resource: /design/architecture/annealing0.gpt6ax.md
status: proposed-integration
generated: { by: "model:openai/gpt-6-astra#xhigh", at: 2026-10-29 }
sources:
  - resource: "opencode://ses_e7a559acfffeNLFScKfuc7htqW"
    title: Human commissions architectural integration and explains renewal as the baseline avoiding uncertain reflink-identity detection; parent traces current contracts, proposes an interleaved build and grounds the renewal cost question in existing Btrfs and kernel evidence
  - resource: "file:///home/rektide/src/jj-waltz/design/coordinated-refill/trajectory1.gpt6ax.md"
    title: Whole-command delivery sequence, shared input lifetimes and explicit product decisions
  - resource: "file:///home/rektide/src/jj-waltz/design/coordinated-refill/work-map0.gpt6ax.md"
    title: Existing three-lane queue and native/provider ticket ownership
  - resource: "file:///home/rektide/src/jj-waltz/design/coordinated-refill/discussion0.gpt6ax.md"
    title: Human requirements for one persisted model, inherited judgments, modular comparison and retained observation reuse
  - resource: "file:///home/rektide/src/jj-waltz/design/coordinated-refill/assessment1.gpt6a.md"
    title: Earlier human direction to redesign frequent reseed end to end with upfront routes and forward failure
  - resource: "file:///home/rektide/src/jj-waltz/design/coordinated-refill/reseed-checkin0.gpt6ax.md"
    title: Native outcome and cost assessment, attributed historical evidence and bounded measurement
  - resource: /doc/architecture/README.md
    title: Maintained component ownership and delivered versus wider capability distinctions
  - resource: /doc/consumer/jj-waltz.md
    title: Current consumer adoption and cross-project improvement requests
  - resource: /design/architecture/draft0.gpt6a.md
    title: Broad design for facts, selected work, child context, continuation and runtime resources
  - resource: /design/architecture/rebuild0.gpt6a.md
    title: Later human-selected partial-frontier and opaque-operation checkpoint scope
  - resource: /design/fit/jw-checkin0.gpt61sxh.md
    title: Source-grounded acquisition, policy, route, decision and opaque-work cost analysis
  - resource: /design/transfer/modes0.gpt6am.md
    title: Inherited region contracts, narrowed exceptions and explicitly bounded model evidence
  - resource: /design/observation/facts0.gpt6am.md
    title: Staged fact demand, full-visitor compatibility and execution handoff
  - resource: /design/observation/names0.gpt61sxh.md
    title: Allocation evidence and sparse-survivor ownership tradeoff
  - resource: /design/performance/opportunity-also.md
    title: Inspected Btrfs checksum coverage, compressed/inline limitations and distinct content, object and extent identity questions
  - resource: /design/performance/leaf0.sol61x.md
    title: Historical controlled production-kernel evidence for improving required-clone execution without detecting existing sharing
  - resource: /design/execution/contracts0.gpt61sxh.md
    title: Earlier typed-goal and fine-grained specimen contracts, distinct from production continuation
  - resource: /design/actions/dispatch0.gpt61sxh.md
    title: Typed dispatch, explicit satisfaction and claims without a universal registry
  - resource: /tree/inspect/src/visitor.rs
    title: Current full facts, built-in comparison, action validation and finalized-plan ownership
  - resource: /tree/inspect/src/shared.rs
    title: Source-once demand union and full-fact acquisition within one inspection epoch
  - resource: /tree/compare/src/coordinated/state.rs
    title: Actual child memberships and lane frames, with no inherited-policy payload
  - resource: /tree/compare/src/coordinated/mod.rs
    title: Lowering, accepted child publication, reductions and route checks
  - resource: /transfer/sync/src/coordinated/mod.rs
    title: Live plan, exact decision projection, barrier clone, refresh and application ownership
  - resource: /transfer/sync/src/coordinated/decision.rs
    title: Saved comparison/action assumptions and duplicated semantic validation
  - resource: /transfer/sync/src/apply.rs
    title: Opaque recursive policy calls, conditional containers and metadata handoff
  - resource: /tree/observe/src/listing.rs
    title: Cached hints, consumed acquisition handles and current native buffer ownership
  - resource: /examples/reconcile/src/transport.rs
    title: Application-owned wire DTOs independent of live ownership representation
  - resource: "file:///home/rektide/src/jj-waltz/transfer/workspace/src/streaming.rs"
    title: Prefix-replayed source qualification, target-specific guards and current one-lane loop
  - resource: "file:///home/rektide/src/jj-waltz/transfer/workspace/src/frozen.rs"
    title: Frozen ignore query work and tree-plus-effective-excludes sharing keys
  - resource: "file:///home/rektide/src/jj-waltz/refill/engine/src/engine/replacement.rs"
    title: N1 command capture, native sequence, shared-input opportunities and finish
verification:
  method: Source and document assessment at diff-dirs parent b17f66f9 and jj-waltz parent 47947f25; read-only queries of existing work items
  scope: Current code paths and interface constraints; earlier runtime evidence is attributed to its owners. This assessment supplies no new runtime or performance result.
extensions:
  related_work: [diffdir-tree-coordinated, diffdir-tree-coordinated-light-state, diffdir-tree-coordinated-region-context, diffdir-tree-coordinated-comparison-policy, diffdir-tree-observe-fact-reuse, jjwaltz-reseed-coordinated]
---

# Anneal the architecture while delivering it

**Keep trajectory1's whole-command destination, and bring inherited judgments,
comparison boundaries and observation lifetimes into the next construction.**
The three-lane work map supplies useful ticket ownership and bounded tasks. Its
placement of region context and modular comparison under “later design” is the
part I would change in response to this session's steering.

The architectural issue is already concrete: the next command will multiply
today's per-path policy work across receivers, and the current persisted decision
format already encodes assumptions that stronger comparison or specialized
execution will need to change. A shared `Arc` fixes one copy; it cannot answer
those questions by itself.

My recommendation is **interleaved delivery**: settle the interacting ownership
contracts, implement their smallest useful path, exercise it in a real heterogeneous
cohort, and complete the public command. Each stage should improve the working
architecture and leave evidence for the next decision. This is a proposed build
scope following an assessment commission.

<a id="annealing-reading"></a>

## 1. What the existing material contributes

| Material | Keep and use | Adjustment for this direction |
| --- | --- | --- |
| [Trajectory1](file:///home/rektide/src/jj-waltz/design/coordinated-refill/trajectory1.gpt6ax.md) | Shared command/source context; actual N1/N2; producer handoff; compact partial reports; preview; CLI completion. | Move the minimum region/comparison contract work ahead of cohort hardening. Keep measurements throughout construction as well as at the final command. |
| [Work map](file:///home/rektide/src/jj-waltz/design/coordinated-refill/work-map0.gpt6ax.md) | Existing owners, IDs and dependencies; concrete decision sharing; optional dirent adoption. | Treat the region and comparison tickets as inputs to this build's interface review. Their whole prospective capability need not become one prerequisite. |
| [Staged discussion](file:///home/rektide/src/jj-waltz/design/coordinated-refill/discussion0.gpt6ax.md) | One advanced persisted model; inherited judgments; modular comparison; native postcondition before algorithm. | Develop those requirements together where they touch the same saved selection, child edge and acquisition boundary. |
| [Broad draft](draft0.gpt6a.md) and [mechanism synthesis](/design/execution/contracts0.gpt61sxh.md) | Separate observations, desired meaning, selected work, progress, runtime and saved state. | Use the later [rebuild contract](rebuild0.gpt6a.md#partial-traversal-is-the-saved-state) for production pause/failure semantics. The earlier fine-grained retry specimen is a different composition. |
| [Modes](/design/transfer/modes0.gpt6am.md) and [facts](/design/observation/facts0.gpt6am.md) | Explicit region authority, residual exceptions, finite demand and full-visitor compatibility. | Turn one actual inherited-policy path into production work. The model's fewer policy calls are neither a shipped feature nor measured filesystem savings. |
| [Name study](/design/observation/names0.gpt61sxh.md) and [library check-in](/design/fit/jw-checkin0.gpt61sxh.md) | Trace allocation, acquisition, route checking and retention separately. | Let the ownership redesign accommodate compact names and shorter fact lifetimes; measure the resulting implementation before selecting a larger representation change. |

Two historical suggestions need explicit interpretation. The library check-in's
separate nonpersistent driver conflicts with the subsequent human requirement for
one persisted model. The broad draft's existing-target no-adoption native route
predates the delivered strict full-S route. Neither historical recommendation
overrides the later constraints or establishes the next public default.

<a id="annealing-present"></a>

## 2. Where the present architecture needs to move

### Decision ownership is the small entry, not the entire change

In `SyncCursor::next_group`, a successful lane retains its `DirectoryPlan`, builds
`DirectoryDecision::of(plan)`, then deep-clones that decision into the barrier and
returned group. The first light-state task can share that immutable decision.
It is a credible independent commit with the same checkpoint semantics.

The subsequent selection/facts split crosses `tree/inspect`: `Operation` currently
owns names, paths, facts, comparison, baseline and selected action together.
Meanwhile `DirectoryDecision::validate` in sync reconstructs baseline/action rules
also present in `Operation::validate` and `operation`. **Before extending those
rules, give their common semantic validation one lower-level owner.** Inspection
can own the metadata-free physical selection rules; sync continues to own barrier,
application and effect-binding checks. Inspection must not depend upward on sync.

This is more consequential than replacing a `Vec` clone. A comparator change,
explicit equal-leaf renewal grant or new comparison state otherwise has to evolve
two semantic validators, the live plan and the saved representation in lockstep.
Different checks remain appropriate at live finalization and untrusted restore;
their shared rules should derive from the same contract.

Prefer a shared immutable body and ordinary read-only views. Application-owned
[transport DTOs](/examples/reconcile/src/transport.rs) encode values, not `Arc`
identity. Sharing barrier/group storage alone need not change wire meaning.
Factoring the finalized selection out of the plan, or adding semantic fields,
requires its own explicit API and saved-schema assessment. Caller-retained views
can extend memory lifetime; count that rather than promising universally lower RSS.

### Region judgments currently have no continuation owner

JW's `SourceQualified::candidate` scans path prefixes; `disposition` invokes it
from both early and late callbacks. `FrozenIgnore::ignored` reconstructs ancestor
queries. This repeats the question “am I below an admitted selected source root?”
even after reaching such a root.

The library cannot presently carry the answer to the next child. `ChildEdge`
contains a name and destination indices; `LaneFrame` contains reduction flags.
The generic lowering value from `next_group_with` is returned to the caller, not
saved in the frontier. Adding information only to that value loses it on descent
and restoration. A command-local discovered-root map would create another owner
and make policy depend on visitation order.

The same problem exists inside sync: `Context::plan` supplies the root-level
visitor again during source-only copy, target-only removal and replacement.
**A useful inherited-context implementation must cross both recursive owners.**
Fixing paired traversal alone misses precisely the absent-routing and removal
cases that motivated the source-qualified design.

### Comparison is built in at more than one boundary

`operation` directly computes stat-key/literal-link equality. Late policy edits
actions over that immutable result. `InstallLeaf` is legal only for a supported
observed difference. Saved validation repeats this rule, and `comparison: None`
means unsupported kind, not omitted comparison or unresolved satisfaction.

Therefore “make comparison modular” entails comparison result meaning, fact
demand, action legality, reporting and resume compatibility. A callback returning
a boolean after full metadata acquisition would cover only part of the need.

### Source reuse is already real, and has a defined boundary

`SharedRoots::inspect_ready` unions source demand across ready lanes and acquires
each demanded source entry once. It currently consumes the entry handle to obtain
full `Facts`. Internal sync passes the resulting source metadata/link text into
installation. A staged sparse observer must retain an acquisition cell long enough
for kind-to-attributes escalation; it cannot consume the handle, throw away the
metadata and rediscover it for execution.

Source-only/replacement recursion remains per lane. In a mixed cohort, A can copy
an absent subtree opaquely while B later compares that subtree through paired
traversal. Those source observations are separate. Carrying a region judgment
removes repeated reasoning; it does not itself make those inspections source-once.

<a id="annealing-lifetimes"></a>

## 3. Use four explicit lifetimes

| Lifetime | Owned information | How it ends or survives |
| --- | --- | --- |
| Command/source | Resolved roots and native profile; pinned S/D; membership and query indexes; frozen ignore inputs; preparation result. | Shared by receivers. Mutable native revision facts are refreshed after their actual transition. Native-only inputs can be released before ordinary transfer. |
| Lane/region | Accepted policy judgment, remaining exceptions, comparison interpretation, descendant authority and outstanding traversal. | Moves into child obligations and saved semantic state. Rebind validates its interpretation without replaying accepted ancestors. |
| Observation epoch | Native listing handles, cached kind, acquired metadata/link/content facts, bounded route admission. | Shared only within its declared stable scope; released after last immediate use. Restored pending work acquires fresh facts. |
| Effect call | Target staging, permissions, output/source descriptors and actual returned work. | Joins and closes at the existing opaque application boundary; actual results enter lane accounting. |

This avoids using one growing “context” object for unrelated purposes. Sharing
native inputs by tree requires accounting for effective excludes too:
`FrozenIgnores` already keys compiled rules by equivalent tree **and exclude
bytes**. Lane-specific link claims, target devices and binding identities remain
lane-specific. A shared source does not make every policy judgment interchangeable.

Runtime tuning belongs to the observation/effect lifetimes. The broad draft's
64 KiB/16-idle-buffer direction is still proposed resource work; current native
listing allocates an 8 KiB buffer per listing. Buffer pools, packed names and
source-batch retention are different resources. None belongs in the meaning of a
saved region or should retain a whole batch merely to keep one pending child name.

<a id="annealing-regions"></a>

## 4. Make inherited policy useful in the next build

### Separate region authority from the physical operation

For JW, a useful first policy progression is:

```text
Search for a qualifying source directory
  → admit Q using source shape, selectors, frozen rules and ownership checks
  → carry SelectedMirror(Q, remaining guards)
  → specialize that context for each child
```

`SelectedMirror` is a suggested JW-side meaning, not a generic library enum.
Within that region, a paired child can compare, a source-only child can populate,
and a target-only child can remove. The region's authority survives those physical
roles. Outside an admitted Q, a vanished source root still preserves the target.
The distinction is crucial during opaque target-only removal, where there is no
source child from which to rediscover the original admission.

At Q, source qualification, native/link overlap and desired-ignore acceptance
are settled for the covered region under frozen inputs. Children need not restart
the selector from the root. Residual checks still include nested administration,
supported types and device boundaries. Current JW reads `metadata.dev()` for
admitted sides: inherited selection does **not** remove that fact demand.

Generic residual exceptions can narrow by native path component. In an explicitly
authorized empty region with one preserved `cache/local/` exception, unrelated
branches lose that exception search; its ancestor carries only the remaining
suffix. A preserved descendant keeps necessary ancestors and can prevent a coupled
replacement. This generic example is not a proposal to change JW's current refusal
for an active link overlapping a selected root.

### Smallest coherent carrier

Recommend a typed, immutable policy-context payload, with a unit/default form for
existing visitors. Its exact Rust signatures need a joint inspect/compare/sync
design review, but its obligations can already be fixed:

1. Pure assessment derives child contexts from supplied facts and parent context.
   The entire immediate selection validates before any child context is accepted.
2. The frontier owns context for each pending lane/child and active return chain.
   Context can be compact or shared within a run; restoration needs its actual
   value or an independently reconstructible, version-bound descriptor.
3. Sync receives the selected context for an opaque operation and specializes it
   internally. Logical final paths remain separate from staging paths.
4. A supported pause retains all context needed by unstarted decisions and future
   descendants. Refresh uses the saved region context and compares new selections
   against saved selections. It does not re-admit accepted Q from scratch.
5. Old full visitors adapt to unchanged full facts and precedence. Narrower demand
   is explicit; a formerly overrideable late choice cannot silently become terminal.

Avoid an untyped map that requires compare to interpret JW data. A generic carrier
can preserve policy-owned data while the policy validates its meaning on bind;
compare validates structural membership and ordering. Use a concrete closed payload
for the first production consumer rather than a universal registry.

**Mixed N2 is the design test:** A lacks Q, B has Q with one stale descendant.
Both inherit selection; A populates through sync and B compares through the frontier.
Only B's true source absence inside Q authorizes its stale removal. A's newly
created directory does not acquire a second owner in the paired frontier.
Pause after A's returned application and before B's: the restored B context and
decision must be sufficient without replaying A or the parent admission.

<a id="annealing-comparison"></a>

## 5. Comparison, authority and demand should evolve together

Use four independently meaningful inputs:

| Input | Question |
| --- | --- |
| Comparison/satisfaction rule | Which observed properties establish the result this operation needs? |
| Work authority | May this name be installed, replaced, removed or retained under the selected region policy? |
| Storage requirement | Must a new installation use private reflinks; must retained leaves also satisfy any stronger property? |
| Observation demand | Which missing physical facts must the observer acquire to answer those questions or execute the selected work? |

The first implementation can keep the present stat-key comparator as its cheap
default, extract it behind an explicit supplied strategy, and exercise a second
small predicate over already-acquired facts. That proves the parameter crosses
inspection, decisions, recursive application and restore, rather than existing as
an unused interface. A content-hash or filesystem-sharing observer can follow as
its own selected capability.

When specialization actually omits comparison, represent `NotAssessed` separately
from equal, different, unsupported and unresolved requested satisfaction. Keep the
legacy full-comparison projection honest. Choose these distinctions before fixing
the next saved selection schema; a single `Option<bool>` cannot carry all of them.
Frozen predicate identity and relevant parameters participate in runtime binding.
Matching the resulting actions is insufficient if their interpretation changed.

Acquire the union of demands from comparison, residual policy and execution.
Start with finite kind/attributes/literal-link stages and a full-demand adapter.
Unknown kind can require metadata; that same acquisition must satisfy subsequent
attribute demand. A lane requiring a stronger observation must not impose its
observation failure on a peer that did not require it. Preserve deterministic
lane-local error order and the current whole-batch pre-effect boundary where promised.

This can be staged: first the explicit default comparison/selection contract and
real region propagation, then a narrow demand path where it removes actual work.
The inspector's acquisition and selected-state ownership must be designed for both
from the first slice. **JW is not the proof that every sparse-fact optimization
helps:** its retained device guards can still require full metadata.

If equal-content storage renewal is selected, give it explicit installation
authority. Do not label equal files different to force the existing action validator
to accept them. Likewise, retain selected-work publication independently of physical
diff: future generated work and existing opaque work make “clean means no work”
an invalid universal rule. The [typed-action study](/design/actions/dispatch0.gpt61sxh.md)
supplies that separation without requiring a general action framework now.

<a id="annealing-native"></a>

## 6. Renewal is the baseline; optimize how it is achieved

The human's follow-up in this session supplies the missing rationale: **renew donor
sharing is the baseline objective because establishing whether existing storage
already shares with the donor may itself be costly or unavailable.** The hope is
to make that constructive operation reasonably fast without requiring clever
identity detection first. This resolves the assessment's earlier framing of an
unanswered renewal-versus-refresh preference for the tracked-file baseline.

Use fresh private destinations and successful required-clone operations to obtain
the filesystem's clone semantics directly. Their production execution does not
require a preceding whole-file sharing census. Focused native tests can verify the
protocol and representative sharing behavior separately from the normal command.

**Renewal is a postcondition; clear-first remains one protocol for achieving it.**
An alternative can improve the native composition while renewing every required
leaf. Retaining an existing leaf based only on content equality would instead
weaken that objective unless suitable sharing evidence were available.

The present strict route is an executable control: save O, sparse-clear, establish
a receiving child of D, install S, adopt S state, restore D when needed, verify,
transfer ordinary contents, finish. It deliberately reinstalls tracked leaves on
a clean D=S run. With D≠S, some S installations are temporary and confer no final-D
sharing guarantee.

Retain that control while extracting command inputs and native receiver execution.
Measure the necessary clone/native work and remove duplicated setup, acquisition,
index construction and protocol passes where a sound replacement is demonstrated.
A less destructive renewal route still needs truthful membership/cache state and
local-obstruction authority on tiny D=S/D≠S examples. Native meaning remains with
JW; generic equal-leaf installation cannot by itself make a partially installed S
tree compatible with an unmodified S cache. The final-D sharing scope when D differs
from S still needs an honest outcome statement.

### Why the Btrfs inquiry does not gate this baseline

The [checked Btrfs account](/design/performance/opportunity-also.md#btrfs-checksums-and-change-tokens)
confirms the remembered obstacle: the inspected `GET_CSUMS` implementation supplies
COMPRESSED/INLINE classifications without checksum payload for those ranges. At
that study's date the running kernel also returned ENOTTY; this assessment did not
recheck today's runtime capability.

Even complete matching content checksums would establish only their declared
content-comparison evidence, **not that two files share the same storage**. Matching
device/inode identifies a shared file object rather than a private reflink.
Different inodes do not establish extent sharing. FIEMAP can provide extent evidence,
but a SHARED flag alone identifies neither this donor nor complete file coverage.
A donor-specific sharing predicate needs its own range, lifetime and cost contract.
The compressed-checksum limitation concerns inspection; requesting a reflink does
not require obtaining those checksums first.

An optional inspect-and-skip optimization should earn its place by saving more
renewal work than its probing costs, with useful coverage on the actual compressed
workload. Unknown evidence would still require renewal under this baseline. Keep
that inquiry available while making unconditional required renewal work well.

### What performance evidence supports

The [earlier leaf-pipeline controls](/design/performance/leaf0.sol61x.md#larger-same-shape-confirmation)
show that required-clone execution can improve substantially without sharing
detection. On the recorded owned Btrfs fixture of 30k 4-KiB files, 300 directories
and 300 links, P4 fresh required-clone sync went from a median 1.789 seconds to
0.954 seconds through descriptor ownership, batch staging and metadata handoff.
Those optimizations are already delivered. This is historical library evidence,
not a prediction for the present JW native command or another unclaimed future win.

Renewal still pays per-file namespace/metadata work and filesystem clone work;
extent structure, dirty data and native transitions can affect cost. Source sharing
does not eliminate destination installations. The next evidence should separate
native commands, tracked installation, ordinary transfer and finish on a bounded
owned fixture. That can show whether the baseline is usable and identify the next
avoidable cost without introducing a prerequisite identity-probing campaign.

Ordinary-data semantics have an established useful baseline: selected source
regions mirror; vanished qualifying source roots retain their target; empty
selection does no ordinary work. Newly installed private reflinks and retained
stat-equal leaves have different storage evidence. The contract task should state
those outcomes, history treatment, preparation conformance and preview truthfully.

The earlier research on colocated creation and finer-grained generated execution
remains useful architectural context. The callable native route currently refuses
receiving `.git`; supporting colocation is a separate native protocol capability,
not something the new shared context may imply automatically.

<a id="annealing-delivery"></a>

## 7. Recommended build: joined design, coherent implementation slices

| Slice | Concrete outcome | Architectural review / evidence |
| --- | --- | --- |
| **1. Shared inputs and immutable decision views** | JW owns one command/source input set and extracted receiver-native operation. diff-dirs shares the existing selected decision between barrier and group. | N1 keeps its native result; repeated S/D/index extraction is counted; existing supported checkpoints survive; wide-group allocation/copy effects are attributed. |
| **2. Contextual persisted inspection and execution** | One selected-state contract with common semantic validation; real inherited child context through compare and opaque sync; default comparison supplied through an explicit seam. JW uses admitted-region context rather than root-prefix replay inside Q. | Preserved nested exception; mixed paired/missing lanes; selected-region target-only removal; Known/Unknown facts; restored partial heads and A-accepted/B-unstarted. A second bounded comparator proves the seam reaches saved meaning. |
| **3. One working N1/N2 command boundary** | One ordinary runtime/cursor over native-ready receivers, source-only producer handoff, loaded finish and compact phase-aware partial reports. | Different lane policies/shapes, real paired source sharing, clean ordinary zero writes and a surfaced failure. Count opaque source work separately. Review this callable end-to-end path. |
| **4. Public operation and bounded cost evidence** | Chosen native contract/profile; nonmutating PLANNED preview; CLI/schema/help; retirement of superseded orchestration; explicit release/install decision. | Real prepare-once, preview and failure cases; matched N1/N2 native and ordinary cost attribution with small reused fixtures. |

The contracts for slices 1–2 should be reviewed together before coding their public
surfaces. Their commits can remain small. For example, sharing the current decision
does not need to await all sparse-demand mechanisms, and source-context extraction
can proceed under the current explicit native control.

Slice 2 is the substantive change to the old queue: **working inherited context
and comparison composition are part of the next integrated build**, not a final
optimization pass after the command is fixed. Preparation and preview can develop
from the shared source interface while that library slice is being completed.
This is dependency structure, not a request for simultaneous agent work.

Keep evidence close to each change. Add the optional native-dirent wiring and JW
selector experiment alongside the observation work, distinguishing routing-region
early exclusions from admitted payload observations. Preserve actual Unknown
handling, byte names, mounted-device refusal and conditional-container behavior.
A result showing that only routing benefits is useful.

### Architectural acceptance, not just final-file equality

The next build should demonstrate:

- **Persistence:** partial ancestors, lane-specific child context, exact selected
  decisions and control windows survive a real fresh-process pause. Changed policy
  or comparator interpretation refuses; accepted ancestors/applications do not replay.
- **Avoided reasoning:** an admitted region does not restart selector/ancestor-ignore
  queries at every leaf. Residual exceptions and physical guards still run where
  needed, including opaque removal and replacement.
- **Avoided duplication:** shared decisions/facts have counted ownership and release
  points. A backend hint, a fresh fact and a route check have separate counters.
- **Authority:** preserved descendants and collisions retain their existing semantics;
  omitted comparison never becomes equality or source-absence deletion permission.
- **Delivery:** N2 is one real cohort; errors preserve their original cause and known
  native/ordinary/finish work. Completion means the requested finish succeeded.

Use the existing continuation and conditional-container cases as regression
evidence, then add discriminators for the new contracts. Tiny semantic fixtures
and allocation/acquisition counters precede representative timing. The existing
research's full-workload campaign is evidence to bound residency and setup, not
a template to repeat. No source-visible cost establishes its wall-time ranking.

<a id="annealing-follow-through"></a>

## 8. How to keep later architecture attached

| Remaining direction | Present design consequence | Trigger for its implementation |
| --- | --- | --- |
| Shared opaque source batches | Acquisition belongs below lane policy; an inherited region is distinct from a cached batch; paired/opaque recursion has one owner each. | Counted both-missing/mixed work justifies a bounded transfer cohort. Moving missing directories into comparison merely after creation is not that design. |
| Route-admission reuse | Separate region authority from a short-lived physical route check. Keep bind, comparison-route, application-route and restored refresh attributable. | Deep-route evidence warrants an explicit live admission lifetime and invalidation boundary. A saved region is not a permanent route certificate. |
| Proven effect-free dispatch | Selected work and recursive obligations are explicit, independent of immediate diff/planned count. | Derive a valid no-effect predicate, preserve control/coverage semantics and retain the existing zero-planned opaque-work counterexample. |
| Compact names and lighter plans | Accessors and ownership boundaries permit compact immutable bodies and owned child promotion. | Production allocation/retention evidence chooses layout; the name study shows why retaining an arena for one child can lose. |
| Dirent tuning and pooling | Caller-owned runtime resources remain independent of policy and checkpoint meaning. | The existing tuning/pool tickets choose actual buffer/resource changes with counted backend evidence. |
| Stronger content/storage predicates | Explicit demands, predicate interpretation and authority can compose without changing baseline facts. | A selected consumer requirement justifies byte reads, filesystem evidence or explicit renewal work. |
| Generated inputs and finish work | Desired inputs and target claims remain distinct from physical observations; clean comparison cannot suppress selected work. | A production consumer requires the already-studied generated/mapped capability. |
| Independently advancing cohorts | Semantic lane heads and runtime batch retention remain distinct. | Actual waiting/memory evidence warrants a different scheduler. Stopping a lane still means abandonment under today's API. |

### Existing work anchors

Keep `jjwaltz-reseed-coordinated` and `diffdir-tree-coordinated`; a new umbrella
would add little. On acceptance of this scope:

- Update `…-region-context` and `…-comparison-policy` from later discussion into
  the minimum next-build deliverables described above, retaining their research links.
- Keep `…-light-state`'s first decision-sharing commit bounded; explicitly connect
  its follow-on selection ownership to the contextual slice.
- Connect `diffdir-tree-observe-fact-reuse` to that slice's acquisition census and
  the native receiving experiment `jjwaltz-reseed-dirent`.
- Make the JW cohort/CLI reviews consume the contextual-library result and native
  contract decision. Use document/ticket pointers across the two native trackers.
- Append outcomes to the [consumer account](/doc/consumer/jj-waltz.md), and integrate
  accepted contracts into the [architecture home](/doc/architecture/README.md).

This assessment proposes those tracking changes; existing open items retain their
standing until scope is selected. Closed research and delivered capabilities remain
historical evidence for the new work.

**Bottom line:** trajectory1 supplies the destination. The local steering supplies
requirements for the architecture that gets there. The next build should carry
settled knowledge forward at command, region and decision boundaries, while acquiring
only the still-needed facts and preserving the same advanced continuation model.
