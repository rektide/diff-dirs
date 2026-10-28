---
type: ArchitectureStatement
title: Composable tree reconciliation
description: Component ownership and contracts for immediate inspection, selected work, ordered progression, completion and optional continuation, including generated inputs and native workspace adapters.
resource: /doc/architecture/README.md
status: proposed-research-baseline
generated: { by: "model:openai/gpt-6.1-sol#xhigh", at: 2026-10-20 }
updated: { by: "model:openai/gpt-6-astra#high", at: 2026-10-22 }
sources:
  - resource: /design/architecture/draft2.gpt6a.md
    title: Accepted domain-grouped crate decomposition and path-derived names; implementation authorized through extraction and relocation before coordinated comparison
  - resource: "opencode://ses_e9e45e9f4ffeZUWoZCK5VDjJvv"
    title: Connected coordinated comparison and the accepted domain-owned crate decomposition with path-derived package names to the maintained architecture
  - resource: /design/architecture/draft1.gpt6a.md
    title: Coordinated comparison refinement with shared source acquisition, per-destination assessment and deterministic grouped traversal
  - resource: "opencode://ses_ea82cf969ffeAXqmUxHChTJar2"
    title: Integrated reviewed evidence and comparison, then connected the parent's detailed post-flight architecture draft and selected resource-control direction
  - resource: /design/architecture/draft0.gpt6a.md
    title: Parent's extensive technical design, including runtime resource ownership and separate dirent tuning/pool tickets
  - resource: /design/architecture/comparison0.gpt61sxh.md
    revision: 269b81f2
    title: Authorized final comparison with the early architectural stakes
  - resource: /design/architecture/statement1.gpt61sxh.md
    revision: 269b81f2
    title: Separate proposed amended architecture, not a silent replacement of earlier evidence
  - resource: /design/execution/walk0.gpt61sxh.md
    revision: b49cfc59
    title: Delivered component-built effectful specimen and focused process/feature verification
  - resource: /design/native/colocation0.gpt61sxh.md
    revision: 9b4fc6ec
    title: Ten bounded native creation and CoW/colocation cases
  - resource: /design/native/graph0.gpt61sxh.md
    revision: 204653e6
    title: One existing-colocated sparse-reset transition preserving native identity and local data
  - resource: /design/native/graph1.gpt61sxh.md
    revision: 730e53b0
    title: Truthful full-source cache adoption changes tracked membership and can remove a retained local
  - resource: "opencode://ses_ea7e3beebffeSBW1V9Q77Gjweu"
    title: Synthesized the independently returned architecture accounts into a maintained statement, bounded native claims with returned evidence, and scoped mechanism research
  - resource: /design/roadmap/brief0.gpt6a.md
    title: Commissioned scope and human constraints
  - resource: /design/architecture/structure0.gpt6am.md
    revision: e4eeb968
    title: Shared immediate inspection, component ownership and recursive surfaces
  - resource: /design/architecture/contracts0.gpt61sxh.md
    revision: 037b124a
    title: Interface authority, generated satisfaction, completion and continuation contracts
  - resource: /design/architecture/scenarios0.gpt6am.md
    revision: b046bbfb
    title: Ordinary copying, archive-derived workspaces and generated assessment continuation
  - resource: /design/native/metadata0.gpt6am.md
    revision: 014db6e3
    title: Versioned native metadata creation evidence and sparse-empty Git-index limitation
  - resource: /README.md
    title: Governing delivered read-only comparison and visitor specification
  - resource: /transfer/sync/README.md
    title: Governing delivered synchronization and failure specification
  - resource: /examples/cursor/README.md
    title: Delivered assessment-only continuation and generated-output boundaries
  - resource: /design/execution/flow0.gpt6a.md
    title: Established information, control and byte-transfer flows
  - resource: /design/execution/state0.gpt6a.md
    title: Established observation, traversal, runtime and native-state lifetimes
extensions:
  ticket: diffdir-tree-architecture-statement
verification:
  method: Synthesis of named independent returns, specification reading and targeted source inspection; document checks only, no new runtime tests or benchmarks
  result: Local link and source-path, own-session provenance, fence and whitespace checks passed
---

# Composable tree reconciliation

Tree reconciliation composes **immediate inspection, pure assessment, selected
work, ordered child progression and completion**. One shared inspection boundary
supports read-only comparison, filesystem synchronization and caller-built
assessment continuations. Their coverage and completion meanings remain distinct.

This document states the architecture used as a **proposed research baseline**;
it does not claim every mechanism is implemented or human-accepted. The
[root specification](/README.md) and [sync specification](/transfer/sync/README.md) continue
to govern delivered APIs. The [synthesis account](/design/architecture/synthesis0.gpt61sxh.md)
records the independent evidence, integration choices and remaining tensions.
The completed flight's [comparison](/design/architecture/comparison0.gpt61sxh.md)
and [amended statement candidate](/design/architecture/statement1.gpt61sxh.md)
preserve the original bets and distinguish supported composition from unselected
production mechanisms and defaults.
The subsequent [parent technical draft](/design/architecture/draft0.gpt6a.md)
develops the overall design in detail: component dependencies, typed work,
progression and construction contracts, resources, native flows and landing slices.
It includes the newly selected runtime dirent-control direction; this maintained
baseline does not thereby claim those interfaces or defaults are implemented.

The [coordinated comparison refinement](/design/architecture/draft1.gpt6a.md)
develops one source against several destinations as a first-class read-only
composition. One frontier owns ordered child edges carrying destination membership;
source observations are shared, while policy, differences, errors and summaries
remain per destination. This is the proposed response to the requested coordinated
walk; independent effectful destination continuations remain a separate composition.

The accepted [crate decomposition](/design/architecture/draft2.gpt6a.md) places
observation, immediate inspection, traversal and the public facade under `tree/`,
filesystem effects under `transfer/`, and the CLI under `app/`. Package names follow
their full path (`diff-dirs-tree-inspect`, `diff-dirs-transfer-sync`, and so on).
Extraction and relocation are the current implementation scope; the coordinated
comparison capability follows that work.

Assume a good source. Concurrency, locking and stable path topology are caller
concerns. Failure promises **no effects history, recovery guarantee or rollback**.
Voluntary pause and continuation do not introduce a recovery transaction.

A **tip** is a scheduled logical unit with the bindings needed to inspect or
execute it. **Completion** closes that unit's declared obligations, which differ
between assessment, synchronization and output delivery.

## Components and ownership

| Component | Contract and owner |
| --- | --- |
| Consumer binding | Caller selects roots, operation intent, logical coordinates and frozen parameters. Binding validates the required root/routing shape; a missing source root is not deletion authority. |
| Physical observer | Core owns listing, cached hints, admitted no-follow metadata and needed literal links. Handles and platform details stay private. |
| Immediate inspector and assessment | Core merges native names, supplies immutable facts and baseline comparisons, invokes pure caller policy, then validates the complete immediate batch. No mutation or recursive descent occurs here. |
| Ordered driver | Owns active tips, child order, phases, accepted aggregates, coverage and completion. It admits children only after their dependencies are settled. |
| Optional executor | Sync owns filesystem preparation, installation/removal, permission scopes and joined runtime work. Other operations need an explicitly bound implementation. |
| Completion reducer and output consumer | A reducer folds local/child results and can produce a logical value. The consumer separately owns delivery or selected installation of that value. |
| Optional continuation transport | Encodes and validates owned continuation data. It owns neither tree semantics nor live runtime resources. |
| Native adapter | JW owns repository/workspace identity, revision transitions, administrative metadata, command defaults and native finish obligations. These are not generic copy policy. |

```text
bindings + frozen policy → immediate inspection → finalized assessment
                                  ↑                     ↓
                           next admitted tip ← ordered driver
                                                    ↕
                                         optional selected execution
                                                    ↓
                                  child folding / finish / completion
                                                    ↓
                                         separate output delivery
```

These are semantic seams, not one mandatory trait per row. The core remains
read-only and independent of sync, JW and serialization. Ordinary copying needs
neither a repository adapter nor a codec. Ordinary comparison needs no executor.

## Immediate inspection and decision authority

The shared local sequence is:

```text
list present sides → native-name union → early admission
→ complete admitted facts → baseline comparison and proposals
→ late selection → validate the whole immediate batch → publish
```

The contract preserves these distinctions:

- **Presence is factual.** Absent, present with unknown hints, observed unsupported
  kind and policy-excluded are not interchangeable. Preservation applies to the
  whole name pair/subtree; hiding source presence cannot manufacture target-only
  deletion authority. Hint access does not secretly fetch missing metadata.
- **Early means before additional child observation.** Enumeration can already
  supply facts. Mandatory structural-root observations can precede root steering.
  Admitted physical facts retain physical provenance; neither generated attributes
  nor a failed lookup can masquerade as real `Metadata`.
- **Decisions are pure; observation is read-only I/O.** Policy uses supplied facts
  and frozen inputs, without filesystem queries, native commands, effects,
  printing or invocation-order state. Calls may repeat or be discarded. A decision
  needing a prior accepted result waits for that result to become an input.
- **Baseline difference, selected work and coverage remain separate.** Retaining a
  different leaf does not make it equal; preserving a subtree does not inspect it.
  The physical default compares regular-file type/size/exact supported mtime and
  literal native symlink text. Directories are structural, never mtime fingerprints.
  Mode-only and stat-preserving byte changes remain outside that predicate.
- **Validation precedes effects and dependent child admission.** Immutable identity,
  legal actions, target routing and coupled replacements must agree. Container
  creation and descent are separate obligations; refusing an absent container
  cannot leave child writes authorized. Retained descendants can block a coupled
  directory-to-leaf replacement.

Current inspection fails in listing → early → admitted observation
(native-name/source-first) → late → validation order, publishing no partial
successful batch or comparison counts. This is a directory-local boundary, not
whole-tree preflight. Policy composition declares terminal decisions and revision
authority: early exclusion cannot be reopened late; current sync adapters run
before the caller's late visitor, which can revise their action choices. Legality
checking is not a universal non-overridable policy algebra.

## Progression, selected work and completion

**Exactly one component owns each recursive edge.** The shared inspector does not
force all jobs into the paired comparison walk:

| Composition | Recursive owner and result |
| --- | --- |
| Read-only comparison | Core descends finalized admitted paired directories; one-sided directories remain opaque units. Only baseline-divergent batches are yielded. |
| Proposed coordinated comparison | One core frontier merges per-destination admitted paired children into edges carrying destination membership. Joined directory groups share source facts and publish separate destination outcomes; one-sided regions remain opaque for each destination. |
| Current sync | Paired walker owns matched-directory descent; sync recursively owns source-only copies, target-only removal and replacement interiors, using the same inspector/policy. |
| Assessment continuation | Caller-built driver can select one-sided interiors and the directory side of collisions too. This assessment coverage does not split coupled mutation authority. |

Work carries its inputs, parameters, target and dependency position: before
children (such as container creation), within admitted children, or after children
(conditional removal, final permissions, generated/native finish work). Selection
is not execution. A broad subtree operation must enforce descendant decisions or
have explicit authority covering that whole region; opacity alone supplies none.

The driver commits logical progress in deterministic native-name DFS order.
Permitted independent inspection and pure assessment may run ahead; only ordered
acceptance changes public coverage and accumulators. Effect dependencies need not
serialize independent siblings, and parallel failure does not promise a serial
mutation prefix. Started work joins before a supported return/pause boundary.

Descendants stay sealed while unresolved parent work can replace or discard them.
Today's divergence seals implement this for current actions. An extended work
driver must also publish **selected work or explicit control/finish obligations
when baseline physical differences are empty**. Merely adding a generated action
would lose it in today's clean-plan dropping/yielding path. The existing silent
clean comparison projection remains valid and need not become a mutation API.

An **entity completion** closes its declared local/child/finish obligations and
carries a result plus coverage. Reduction can produce generated content or select
finish work; effectful obligations must settle, or be explicitly skipped with
incomplete coverage, before that entity is closed. Assessment completion means
neither target convergence nor output delivery. Observed, planned, completed and
policy results remain distinct; none is an effects ledger. A completion event is
not an exactly-once delivery receipt.

## Component-built continuation and pause

A cursor is the ordered driver's **state for future progress**, not its runtime.
Construct it from bindings, inspector/policy, child selection/reduction, optional
executor/output handling, and initial or restored state. Executable components
are supplied by the application; serialization is optional.

| Owner | At a saved pause |
| --- | --- |
| Driver | Retain logical bindings, frames, selected pending children, positions/phases, local and child aggregates, coverage and unresolved tip/work. |
| Stateful component | Represent behaviorally necessary state as validated data, reconstruct it from declared inputs, or prohibit pausing at that point. |
| Inspector | Finish/drop listing buffers, handles and transient facts. Retained names need ownership or retained backing storage; a closed dirent stream cannot be borrowed. |
| Executor/runtime | Join jobs and settle temporary preparation/permission scopes at the supported boundary. Open files, pools and live leases are not serialized as cursor data. |
| Codec/receiver | Validate representation, lossless native coordinates, bindings, invariants and component compatibility; rebind implementations/resources. Byte validity is not execution authority or filesystem relocation. |

Use **one attempted immediate-directory assessment** as the baseline assessment
budget unit, including clean, empty and failed attempts. Ready push/pop and pure
postorder completions can drain without another assessment. This is not a bound
on entries, syscalls, bytes, memory, events or latency. `Iterator::next()` yielding
only differences is not such a step: it may traverse arbitrarily much clean work.

For effects, permit voluntary pauses **between joined effect units**, not inside
live staging or permission scopes. A whole recursive one-sided operation is a
valid initial unit but not a bounded directory step. Finer interruption requires
explicit remaining-child/finish state; its granularity and aggregate protocol are
still research questions, not an invented recovery mechanism.

Retry re-establishes actual unresolved work as necessary; it neither replays stale
facts nor assumes a failed effect did nothing. Skip abandons a declared remaining
unit and immediately propagates incomplete coverage, without undoing prior
effects. Repeated attempts must not double-fold an unaccepted local result.
Assessed ancestors need not be relisted, so continuation is not a fresh scan after
edits. Old cursor reuse can repeat outputs/effects; cursor publication and output
delivery have no joint transaction. Historical bodies and completed-effect
catalogs are not required state.

## Physical, generated and mapped inputs

Keep **logical identity, physical source, physical target and private preparation
locations** separate. Ordinary trees use coincident root-relative suffixes;
staging already needs logical and physical separation. That is not evidence that
arbitrary mapping is implemented.

Generated desired content enters through explicit production/input, never by
inserting fake physical facts into the existing visitor. A narrow composition
provides a logical destination, desired shape, bytes or a recipe with parameters,
and intended attributes to an explicitly selected installer. Alternatively, a
reducer emits a logical value for a consumer without installing it. These seams
do not select an all-input-provider framework or a universal handler registry.

Reconciled generated inputs must define **target satisfaction**. Physical stat
equality does not compare a recipe to installed bytes. Output-affecting parameters
participate in that rule; unknown satisfaction cannot be called equality. Hashes,
rewriting, receipts and a particular dispatch/byte-stream representation are not
selected here. Domain annotations remain declared inputs, not physical evidence.

Generated-name conflicts, multiple mappings to one target and ancestor routing
must be resolved or refused before their conflicting effects are authorized.
Logical sibling names do not prove physical independence. The general collision
policy remains open. Outputs written into an input tree additionally need an
explicit reservation/exclusion or dependency rule so they do not accidentally
count their previous selves. A separate output root is a simple composition,
not a newly imposed library default.

## Work avoidance and honest coverage

Every shortcut names its evidence domain, the work removed and the coverage left.

| Knowledge or decision | What can disappear | What remains distinct |
| --- | --- | --- |
| Early pair-wide preservation | Child metadata/readlink, interior enumeration and effects | Containing listing/root checks; hidden contents are not proven equal. |
| Late preservation/stopped descent | Later enumeration and effects | Immediate observations have already been paid for. |
| Opaque one-sided comparison | Comparison of the interior | Recursive transfer/removal still enumerates and enforces policy. |
| Explicit known leaf list | Discovery of unrelated source directories | Current installation facts, routing checks and target-existence rules; no deletion authority over unlisted names. |
| Sufficient whole-subtree evidence or whole-region authority | Scoped assessment/descent or fine-grained diff calculation | Its predicate, descendant policy, enumeration/transfer costs and coverage must be stated, not assumed away. |

Directory mtime proves no subtree equality. Native tracked-tree identity alone
does not prove receiving filesystem satisfaction or equality of ignored/generated
contents. Proven satisfaction, preserved coverage and skipped coverage are
different results. No automatic prune/replacement heuristic is selected.

## End-to-end compositions

### Ordinary directory work

Bind ordinary roots and choose the operation. Read-only comparison reports
target-only entries without deleting them. Chosen sync **mirror** removes them;
chosen **update** preserves them. These are existing distinct operations, not a
new target-only default inferred from “copy.” Transfer policy selects ordinary
copy, required reflink or permitted fallback independently.

Inspect a root containing paired `src/`, source-only `assets/` and preserved
target-only `local/`. Comparison descends `src/` and reports opaque `assets/`;
sync creates `assets/`, inspects its admitted interiors and installs their bytes,
then advances paired traversal. `local/` stays pair-wide protected. Sync owns
staged leaf installation and postorder permissions. Completion reports the chosen
policy's result, not equality of deliberately preserved roots.

### JJ archive to receiving workspace

The normal JW donor here is the **JJ archive**: the primary checkout owning the
real `.jj/repo`, not necessarily a colocated checkout or a bare repository. A
registered-workspace donor is a separate binding whose repository owner must be
resolved. JW selects native creation/revision transitions; generic transfer fills
only authorized ordinary data around receiving-owned metadata.

- Archive `.jj/repo` is shared repository storage. Receiver `.jj/repo` links to
  it; receiver `working_copy/checkout` and native registration are its own identity.
  Donor `tree_state` is separate, version-sensitive native cache state, adoptable
  only through an explicitly native-aware operation.
- A colocated receiver's `.git` is a gitfile leading to private Git worktree
  administration and then the common store. It must not be copied from the donor.
  Protect **both sides** of root `.jj`/`.git`: source absence of `.git` never
  authorizes deleting a receiver's gitfile. Existing receivers retain their
  identity through deliberate materialization/revision transitions, not another
  `workspace add` into populated contents.
- The [native layout study](/design/native/metadata0.gpt6am.md) demonstrates normal
  and sparse-empty colocated creation from both colocated and internal-Git archive
  forms on JJ `0.45.1-33d3a595…` / Git `2.55.0`. Sparse-empty omits JJ payload but
  **retains parent entries in the Git index**, producing missing-worktree status.
  Metadata creation alone does not establish combined prefill/adoption convergence.
  Subsequent [creation experiments](/design/native/colocation0.gpt61sxh.md) passed
  ten tiny cases with real CoW/state reuse on the recorded runtime. An
  [existing-target case](/design/native/graph0.gpt61sxh.md) also converged through
  the equal-subset/sparse-reset route while preserving identity and an ignored
  local. These are bounded native results, not delivered JW integration or
  performance evidence.

Full source-state adoption carries **tracked membership**, not only stat hints.
The [membership comparison](/design/native/graph1.gpt61sxh.md) showed a truthful
source cache absorbing a matching ignored local at a path in `tracked(S)` but not
`tracked(D)`; native restore then removed it. The no-adoption control preserved
it, while both ended at exact tracked `D` with clean Git status. Choosing an
existing-target adapter must reconcile its native membership changes with declared
local preservation. Source goodness alone does not grant that authority. Current
reseed's equal-subset/reset route avoids this source-membership import.

Native registration/setup precedes filling a new receiver; any selected cache
adoption and JJ/Git convergence are native ordered obligations, not visitor side
effects or generic stat-key proofs. Default-on colocation with opt-out remains
the preferred, **conditional** product direction. Current JW's CoW/colocation
mode exclusion proves neither combined feasibility nor impossibility. No broader
JW copy/default deletion policy is selected by this architecture.

### Paused assessment with generated output

Compose the immediate inspector with an all-directory assessment driver and a
parameterized postorder summary reducer. Pause after N assessment attempts with
frames, child positions and aggregates; resume at the pending tip without replaying
ancestors. A blocked tip can be retried or its scheduled subtree skipped incomplete.
Completion yields real summary bytes to a separate output consumer.

The [cursor example](/examples/cursor/README.md) implements this assessment-only
composition with one descriptor and optional kopacetic snapshots. It installs
nothing, rejects noncoincident mappings and has no effectful continuation. Adding
generated installation requires the explicit satisfaction/target boundary above;
adding finer effect pauses requires additional recursive/finish state.

The [walking specimen](/examples/walk/README.md) now demonstrates that next narrow
composition: supplied inspector/executor components through a Bon builder, owned
requests and acknowledgments, separate assessment/effect limits, actual byte-copy
and generated finishes, saved manifest state, and four fresh-process gates.
It uses explicit demo-only bytes/retention/reserved-output semantics and leaves
core/sync APIs unchanged. Its [implementation account](/design/execution/walk0.gpt61sxh.md)
records eleven all-feature and nine codec-free tests; it is not a general
permission, replacement, provider or native-workspace continuation adapter.

## Delivered seams and next interface discriminators

The public `inspect_directory` is the delivered immediate seam. `PlanWalk` and the
parallel frontier deliver paired comparison; sync provides recursive execution,
joint preview and separately revalidated public-plan application. Their current
code boundaries are mapped in the [execution flow](/design/execution/flow0.gpt6a.md)
and [state account](/design/execution/state0.gpt6a.md). No restored cursor makes an
old physical plan fresh authorization. The extended work-publication, generated
reconciliation and effectful cursor protocols above are **not delivered core/sync APIs**.
The bounded example supplies evidence for those seams; it does not promote them
to new core or sync APIs.

| Research owner | Smallest next discriminator |
| --- | --- |
| `diffdir-tree-execution-model` | One created directory, two installs and a finish obligation: specify pause/retry/skip after one visible install and before finish. Decide opaque versus child-sized units, pending phase, one recursive owner and fold ownership without an effects history. |
| `diffdir-tree-actions-protocol` | Equal physical inputs plus selected generated work, a physical/generated name conflict and two inputs mapped to one target: expose the smallest provenance/parameter/child/finish contract that reaches ordered execution without rewriting baseline diff. Compare narrow producer/installer and desired-input seams; do not presume a registry or collision policy. |
| `diffdir-tree-observe-facts` | Demand table for preserve, physical comparison, removal, copy and generated satisfaction with Known/Unknown hints: identify required facts and their acquisition owner/lifetime, including execution needs and a scoped subtree proof. Avoid hidden I/O and invented full facts. |
| `diffdir-tree-transfer-policy` | Matrix of source-present updates, target-only retention/removal, collisions and preserved native regions for new/existing receivers. Separate generic operation intent from unresolved JW breadth/ignoredness/classification choices; expose human decisions rather than select defaults. |

Exact Rust generics, a general input-provider framework, handler registration,
generated satisfaction mechanism, general output collision policy and finer
effectful pause granularity remain open. Their mechanisms must honor the ownership
and authority contracts above; they need not be settled to use the shared seams.
