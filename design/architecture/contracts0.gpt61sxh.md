---
type: ArchitectureStatement
title: Tree reconciliation interfaces and composition contracts
description: Independent Wave 0 architecture separating physical observation, pure assessment, selected work, ordered child progression and completion, with optional component-built continuations and generated inputs.
resource: /design/architecture/contracts0.gpt61sxh.md
status: independent-proposal
generated: { by: "model:openai/gpt-6.1-sol#xhigh", at: 2026-10-20 }
sources:
  - resource: "opencode://ses_ea7ecbfcaffex1sh8pvXu7Yp55"
    title: Independently derived interface and composition contracts from established specifications, current code and the bounded cursor specimen, under the scoped human constraints
  - resource: /design/roadmap/brief0.gpt6a.md
    title: Dispatch-safe architecture questions, independence boundary and human constraints
  - resource: /README.md
    title: Governing comparison, visitor, traversal, coverage and caller-mutation specification
  - resource: /sync/README.md
    title: Governing application, staging, recursive policy, progress and failure specification
  - resource: /src/visitor.rs
    title: Immutable observations, early and late steering, typed actions, finalization and immediate inspection
  - resource: /src/listing.rs
    title: Private enumeration handles, nonprobing hints and deferred physical observations
  - resource: /src/pair.rs
    title: Root binding, paired-child lowering and release of clean plans
  - resource: /src/walk.rs
    title: Ordered sequential frames, divergent-only publication and prune window
  - resource: /src/frontier.rs
    title: Speculative inspection, ordered commitment, finalized-child admission and joined epochs
  - resource: /src/model.rs
    title: Baseline differences and conservative committed-prefix coverage
  - resource: /sync/src/lib.rs
    title: Bound applier, caller loop, root dispatch and application aggregation
  - resource: /sync/src/apply.rs
    title: Public versus immediate plan consumption, recursive work, staged installation and postorder obligations
  - resource: /sync/src/visitor.rs
    title: Explicit compatibility composition and compiled seed, update and prune policies
  - resource: /sync/src/model.rs
    title: Observed, planned, policy and completed counters without an effects catalog
  - resource: /design/execution/flow0.gpt6a.md
    title: Established information, control and file-data flow account
  - resource: /design/execution/state0.gpt6a.md
    title: Established context, traversal, application and native-state ownership account
  - resource: /examples/cursor/README.md
    title: Delivered assessment-only continuation semantics and transport limitations
  - resource: /examples/cursor/src/cursor.rs
    title: Actual frames, pending tips, descriptors, postorder reduction, retry and skip
  - resource: /examples/cursor/src/codec.rs
    title: Actual optional snapshot transport and semantic validation seam
  - resource: /examples/cursor/tests/continuation.rs
    title: Inspected assertions for uninterrupted equivalence, retained ancestors, real failures and incomplete skips; not rerun
  - resource: /design/cursor/implementation0.gpt61sxh.md
    title: Earlier twelve-test verification report and its explicitly bounded delivered slice
  - resource: "file:///home/rektide/src/jj-waltz/src/cow/transfer.rs"
    title: Native consumer's frozen visitor inputs and caller-owned execution resources
  - resource: "file:///home/rektide/src/jj-waltz/src/lifecycle.rs"
    title: Current creation-mode selection and native lifecycle composition, not future feasibility
  - resource: "file:///home/rektide/src/jj-waltz/src/workspace.rs"
    title: Current native workspace creation, materialization and local tree-state adoption boundary
  - resource: "file:///home/rektide/src/jj-waltz/src/jj.rs"
    title: Executable capability probe and native working-copy-state helper boundary
  - resource: "file:///home/rektide/src/jj-waltz/src/ownership.rs"
    title: Native Git worktree topology and creation ownership distinct from copied files
verification:
  method: Source reading of the named specifications, implementation and earlier evidence, plus local document-link, cited-path and code-fence checks; no runtime probes, suite reruns or benchmarks
  result: Document checks passed; interface claims distinguish specified and delivered mechanisms from proposed contracts; native colocation feasibility remains unresolved here
extensions:
  ticket: diffdir-tree-architecture-contracts
  wave: 0
  reading_boundary: Withheld parent statement, full roadmap and other same-wave outputs were not read, searched or evaluated
---

# Tree reconciliation contracts

## Architectural statement

A reconciliation composes **observation, assessment, selected work, child
progression and completion**. The components exchange explicit data and
obligations; none of these stages stands in for the others. The default composition
compares physical directories and performs ordinary filesystem copying. Reporting,
generated content and repository-aware consumers reuse the boundaries without
making their policy part of that default.

An **ordered driver** owns logical progress. Read-only inspection may run ahead;
pure decisions may run speculatively. Effects and decisions depending on prior
accepted results run only after their dependencies are satisfied. A cursor is the
driver's continuation state, not the runtime, an effects history or a filesystem
snapshot. It may be assembled from components and optionally serialized.

This is proposed architecture, written directly. **Specified/delivered** paragraphs
identify existing obligations and mechanisms; they do not imply the proposed
extension interfaces already exist. Diff-dirs was read at parent `013bec8a`; the
commission identifies `c7b86ffb` as the unchanged core-code baseline. JW files were
read only to establish consumer boundaries, not to settle native feasibility.
The existing specifications retain authority over the implemented APIs.

### Terms used here

| Term | Meaning |
| --- | --- |
| Logical coordinate | The identity under which the caller reconciles an entry, independent of its backing source or staging location. |
| Observation | Facts actually supplied by an input provider; physical filesystem facts retain physical provenance. |
| Assessment | A decision over supplied observations and frozen inputs, including comparison, work selection and child selection. |
| Selected work | An authorized operation and its parameters, not a description of what already happened. |
| Tip | A scheduled logical unit plus the bindings needed to inspect or execute it. It need not represent a whole subtree already scanned. |
| Completion | Closure of a declared unit's obligations, with a result and coverage; not automatically equality, installation or output delivery. |
| Cursor / runtime | State sufficient to continue logical progress / the live implementations and resources that perform that progress. |

## 1. What crosses each interface

The following are semantic interfaces, not a requirement to introduce one public
trait per row or a universal rule engine.

| Interface | Input | Output and authority |
| --- | --- | --- |
| Binding → observation | Bound roots/providers, logical tip, requested observation scope | Structural validity and provider-local references. Binding authorizes neither arbitrary deletion nor native metadata adoption. |
| Listing → early assessment | Ordered union names, actual side presence, already-available hints and their provenance | Admit, preserve the whole pair/subtree, or refuse. Filtering cannot rewrite presence into absence. |
| Admitted observation → assessment | Complete required immediate facts, comparison rule, frozen parameters | Immutable baseline comparison plus proposed work and candidate children. Observation failure is not fabricated metadata or equality. |
| Assessment → execution | Validated selected work, input references, target coordinate and operation parameters | Runtime executes only the selected authority; reports a result or a failure. A plan is not completed work. |
| Assessment/execution → child progression | Final child decisions and satisfied prerequisite obligations | Ordered tips. Child publication is not implicit permission to bypass parent decisions. |
| Children → completion | Local result, accepted child results, coverage and outstanding finish obligations | Postorder aggregate, optional generated value and/or explicitly selected finish work. Effects remain separate from pure folding. |
| Driver → continuation transport | Quiescent logical state plus any required component continuation data | Optional validated representation. Transport does not attest effects or freeze the filesystem. |

Information may flow through all rows while bytes do not. Physical file contents
normally stay in their provider until execution opens them. An ordinary copy needs
neither a whole-source catalog nor file bodies in the cursor.

## 2. Observation is factual, scoped and replaceable

### Physical observations

A physical observer reports native names and no-follow facts. It distinguishes:

1. **Absent:** the participating provider has no entry at this coordinate.
2. **Present, incompletely known:** enumeration supplied a name but not a usable
   kind or all comparison facts.
3. **Present, observed:** required facts were obtained; unsupported kinds can still
   lack a comparison rule.
4. **Excluded:** policy declined observation of the pair/subtree. This is a coverage
   decision, never evidence of source absence.

These states must not collapse. Neither Unknown kind nor an unsupported comparison
authorizes treating a name as missing or clean. A failure to obtain required facts
fails that assessment unit; it does not publish a partial successful batch.

Early assessment runs before **additional child observation**, not necessarily
before every fact exists. Providers can already possess kind or metadata facts.
Accessing a hint must not silently trigger a metadata lookup to fill a gap. The
observer performs admitted I/O; the visitor does not acquire observations itself.
Structural root validation is separate and can precede early steering.

Observation ownership follows its useful lifetime. Names/facts may be borrowed
from an owned immediate batch. Shared directory handles can remain private until
admitted metadata is collected. No contract requires allocating a full path for
every field, retaining one descriptor per child, or archiving backend dirents.
Conversely, speculative or retained data must not outlive its owner. A future
borrowed/zerocopy backend changes representation, not presence or authority.

**Specified/delivered:** `listing::Entry` privately retains std/native handles;
`EarlyFacts` distinguishes absent sides from present Unknown and records listing
versus structural-root origin. `Facts` owns actual `Metadata` and needed literal
link text. Handles do not enter `DirectoryPlan`. See
[`src/listing.rs`](/src/listing.rs) and `Name`, `Facts`, `inspect_listed` in
[`src/visitor.rs`](/src/visitor.rs). The established
[state account](/design/execution/state0.gpt6a.md#enumeration-can-already-provide-facts)
records backend-specific capabilities and earlier bounded verification; this
statement adds no new syscall or performance claim.

### Scope of an observation

An immediate batch describes immediate children. It does not silently certify
opaque one-sided interiors, all policy coverage, or an immutable tree. Source
goodness is assumed for this architecture; concurrency and locking are out of
band. The physical observer still needs valid routing and meaningful actual
observations. No new donor-verification service is implied.

The default physical comparison retains its stated predicate: regular-file type,
size and exact supported mtime; native literal link text for symlinks. Directories
are structural. Directory mtime, inode hints and a nonempty `Metadata` value are
not subtree-equality proofs. Mode-only or stat-preserving byte changes remain
outside that default predicate.

## 3. Assessment chooses authority without changing facts

Assessment consists of pure transformations over the supplied batch and frozen
caller inputs. It may classify, refuse, preserve, choose work, choose parameters
and decide whether descendants should be considered. It must not copy files,
issue native commands, print ordered progress, or update the continuation by
counting hook invocations.

The decision result keeps **baseline observation**, **selected operation**, and
**coverage** distinct. Preserving an observed different leaf does not make the
baseline equal. Preserving an unsupported entry does not invent a comparison.
Suppressing descent changes coverage even when all immediate leaves are equal.

Every published immediate work batch validates before its effects or dependent
child admission. Validation covers:

- immutable coordinate/presence ownership and deterministic slot ordering;
- action compatibility with observed and declared desired shape;
- target routing/container dependencies;
- operation-specific parameters and their authority;
- conflicts among selected outputs and coupled replacements.

The default action semantics remain small and useful: keep/install/remove a leaf;
keep/create/remove a directory container independently of descent; preserve a
subtree; perform a coupled type replacement. A directory-to-leaf collision is not
a target-only deletion followed by an unrelated creation. A retained descendant
can block replacement of its container. Refusing container creation cannot leave
child writes implicitly authorized below an absent container.

### Composition is explicit, not an accidental policy algebra

A caller can combine policies in compiled code. The composition declares which
inputs are frozen, which decisions are terminal, and which later component may
revise earlier work selection. Refusal is terminal for that assessment; an early
preserved subtree cannot be reopened by a late component. Two work selectors are
not automatically commutative, and “last wins” is not a universal safety rule.
Legality validation does not by itself enforce a caller's non-overridable policy.

**Specified/delivered:** `sync::Steering` orders early Scope before the caller's
early visitor, then late Check, Absence and caller visitor. The caller can revise
the earlier late action selection; this is visible composition, not independent
hard authority in each compatibility adapter. `finish` validates actual actions
and unsupported kinds before constructing the immutable plan. See
[`sync/src/visitor.rs`](/sync/src/visitor.rs) and `Operation::validate`, `finish` in
[`src/visitor.rs`](/src/visitor.rs).

### Pure does not mean notification or dependency-free

Pure assessment may overlap, repeat or be discarded. It may not depend on ordered
invocation or treat a hook call as committed progress. Filesystem observation is
read-only I/O, **not itself a pure function**; repeatability relies on the stated
input conditions. Pure generation over already supplied arguments is different
again from generation that reads files or invokes a process.

An assessment depending on a parent/previous child result waits until that result
exists. The driver can freeze that result into the next assessment's inputs.
Purity permits speculative execution only after these data dependencies and
admission constraints are satisfied.

## 4. Selected work has a runtime and a declared dependency position

Selected work names **what** should happen and carries the parameters necessary
to bind a runtime implementation. Ordinary copy, required reflink and permitted
fallback are generic transfer choices, not repository policy. A parameterized
operation might select a content recipe, metadata profile or target mapping;
choosing it does not execute it.

Runtime implementations own live resources: opened files, clone/copy facilities,
temporary outputs, permission leases, processes and borrowed execution pools.
They consume authorized inputs and return the result needed by the driver. They
do not gain authority over unmentioned names merely because a subtree operation
is convenient. A whole-subtree operation either respects descendant decisions
through the shared mechanism or has an explicit whole-region authorization that
really supersedes that traversal.

Dependencies identify a work item's position:

- **Before children:** create a routing container, complete local replacement or
  finish other selected local work on which descendant traversal depends.
- **Children:** inspect/reconcile admitted descendants, with one owner of each
  logical descent rather than walker and applier both performing it.
- **After children:** conditional container removal, final permissions, aggregate
  output or a native consumer's declared finish obligation.

This is not a requirement that all work be serial. Independent admitted sibling
operations may overlap, and private preparation may precede installation. Ordered
driver state accepts their results in the declared logical order. Effect ordering
is promised only where a dependency requires it; a parallel failure need not leave
a serial mutation prefix.

**Specified/delivered:** paired descent belongs to `PlanWalk`/frontier. Sync owns
recursive one-sided work and replacement preparation through `Context::plan`,
`execute_entry`, `copy_directory_selected` and `reconcile_directory`. This is a
valid composition, but its recursive internals are not presently cursor steps.
The same visitors reach those interiors. Staging and postorder permission work
are real runtime obligations, not pure visitor decisions. See
[`sync/src/apply.rs`](/sync/src/apply.rs), especially lines 423–669 and 802–991.

## 5. Child progression and completion belong to ordered state

The ordered driver accepts an assessment, schedules its selected work, publishes
eligible children, folds their results and closes the parent. Deterministic
native-name DFS remains the default logical order. It is not filesystem completion
order and should not be reconstructed by sorting full path strings.

Child progression carries enough binding to distinguish source-present,
target-present, mapped or generated children. A stopped parent cannot publish
descendant jobs. A selected conditional container-removal action is not completed
until its declared child/finish dependencies have resolved; its selection can
already be finalized and immutable.

Speculation is private. Only ordered acceptance changes public coverage,
accumulators and progress position. A parent whose remaining decision may discard
or replace its descendants seals those descendants until that decision is settled.
The current divergence seal is a concrete way to maintain this property; baseline
divergence is not the only possible reason for such a dependency in an extended
composition.

A completion contains its coordinate, declared local/child result, coverage and
the aggregate needed by its parent. The pure reducer can construct a generated
value. Installing or delivering that value is separate work, with its own outcome.
An effectful finish must succeed or be explicitly skipped with incomplete coverage
before the driver claims the unit's obligations are closed. Otherwise it remains
unresolved.

An assessment-only cursor can complete a directory after its selected child
assessments without performing any mirror action. An effectful reconciliation
cannot use that same completion to claim target convergence. Likewise, an ordered
completion event is not an exactly-once notification or a receipt for stdout.

**Specified/delivered:** frontier `reduce` alone changes committed coverage;
`children` permits lookahead below clean finalized nodes and divergence seals stop
lookahead below yielded differences. Each epoch joins before returning. Sequential
and frontier frame closure propagate changed-directory state postorder. The cursor
specimen separately folds counts and generates summaries postorder. These are
related mechanisms with different meanings of completion, not one interchangeable
success flag. See [`src/frontier.rs`](/src/frontier.rs),
[`src/walk.rs`](/src/walk.rs) and
[`examples/cursor/src/cursor.rs`](/examples/cursor/src/cursor.rs).

## 6. Stepping, pause, failure, retry and skip

A step returns with no unrepresented in-flight work. Its state says what remains
to be done, including an outstanding caller-owned action or finish obligation.
That is a quiescent execution contract, **not a recovery or rollback guarantee**.
Live pool jobs join, and a component either releases runtime resources or supplies
the continuation data required to reconstruct its suspended work.

One attempted immediate-directory assessment is a useful default budget unit. It
includes clean and empty directories and failed attempts. It is not a bound on
entries, syscalls, bytes, memory, effects or elapsed time. A yielded diff is not a
sufficient general step unit: large clean regions might yield nothing, and an
opaque recursive application might do arbitrarily much work after one yield.

The semantic phases can be illustrated as:

```text
pending assessment
    → accepted local result / selected work
    → prerequisite work settled
    → ordered children and child-result folding
    → finish obligations settled
    → parent completion
```

Blocking can occur at any effectful/observational phase. A component-built cursor
must identify the unresolved unit and phase, rather than storing only a printed
error or pretending every failure happened before assessment. This does not
require exposing every syscall as a step. A runtime can make a recursive subtree
operation one unit, but cannot then promise cheap interruption inside it.

### Failure promises very little

Failure names the relevant unit/operation and cause. Earlier effects and started
sibling work may remain. Cleanup may fail. There is no rollback, coherent restored
state, effects history, crash transaction or guaranteed recovery path.

An optional saved continuation helps the caller retry or skip **actual state**:

- **Retry** re-observes the unresolved work as necessary and recomputes what is
  still required. It does not replay stale physical facts or assume the failed
  action did nothing. Successful earlier installations can naturally become no-ops
  under a fresh comparison.
- **Skip** explicitly abandons the declared remaining unit, marks the relevant
  coverage incomplete immediately, and continues where possible. It does not undo
  any effects already performed, and “skip subtree” must not masquerade as “skip
  only the offending leaf.”
- Reusing an old continuation may repeat actions or outputs. Callers must choose
  suitable operation semantics; serialization supplies no exactly-once execution.

Assessment-attempt counters may grow on retry. Accepted observation aggregates
must not grow merely because an unaccepted attempt was repeated. A retrying
effectful composition must define which local result is retained or recomputed
before folding it into ancestors; it cannot infer this from copy counters. The
smallest phase/aggregate protocol is still a discriminating design question (§11).

**Specified/delivered:** the example supports a blocked **pending assessment**,
retries that tip, and skips its scheduled subtree with incomplete ancestry. It
does not demonstrate effectful phase continuation. `Failure.completed` in sync is
aggregate partial progress, not an effects catalog, and sync currently recommends
fresh comparison. These mechanisms support the distinction; they do not establish
the unimplemented effectful cursor protocol.

## 7. Cursor versus runtime, and what a caller can assemble

### Ownership contract

| Continuation state | Live runtime/component state |
| --- | --- |
| Logical bindings and provider/operation identifiers necessary to continue | Bound implementations, directory/file handles and active native clients |
| Active frames, ordered pending child selections and next-child position | Enumeration buffers and private observation handles |
| Logical phase, unresolved tip/work and declared parameters | Staging arenas, writable leases and in-flight execution |
| Local/child accumulators, coverage, blocked information and final aggregate | Pure functions, visitor implementations, descriptor dispatch and codec |
| Component-specific state necessary to resume an allowed pause | Optional caches or speculative results whose loss may cause recomputation |

The right column is not intrinsically serializable. If a component's internal
state affects future behavior, one of three things must be true at a saved pause:
it is represented in validated continuation data; it is reconstructible from that
data and declared inputs; or the component does not permit pausing there. Hiding
such state behind a trait does not remove this obligation.

Speculative ready results need not be persisted. Dropping them can cause extra
inspection but must not change accepted logical results under the input contract.
Prior generated bodies, completed-action catalogs and all historical output are
not required cursor fields. Only information needed for future computation stays.

### Assembly contract

A caller can supply an observer, pure assessment/visitor, child-selection rule,
runtime executor and completion reducer in a combination appropriate to its job.
It can use existing default components without supplying a persistence codec or
repository integration. These combinations are meaningful:

1. **Read-only differences:** physical inspector + baseline assessment + paired
   DFS + observed summary; no executor.
2. **Ordinary sync:** physical inspector + compiled selection + generic applier +
   paired/one-sided child ownership + policy/progress aggregation.
3. **Bounded assessment:** immediate inspector + caller-owned DFS + postorder
   reducer + optional cursor codec; no mutation engine.
4. **Known/generated work:** explicit inputs + narrow validated operation/runtime
   + optional traversal, without enumerating an unrelated whole tree to fabricate
   ordinary directory records.

“Constructible from components” means these semantic seams are usable; it does
not mean callers may forge physical facts inside a sealed `DirectoryPlan`, mutate
private frontier internals, or deserialize arbitrary closures. Same-process custom
components need not be serializable. Cross-process use needs an agreed descriptor
or component snapshot and a compatible receiving implementation. A small caller
enum/adapter may suffice; a global action registry is not mandatory architecture.

Transport validates schema/version, component compatibility, lossless native
coordinates and continuation invariants before execution. A byte-valid archive is
not an authorized operation. Relative paths are interpreted under declared
bindings; transferring cursor bytes does not relocate those bindings by itself.

**Specified/delivered:** the public immediate inspector already enables the
separately implemented assessment cursor. The example saves frames, counts,
parameters and a pending tip through actual kopacetic snapshots; it rejects mapped
coordinates and knows one descriptor. It does not serialize `PlanWalk`, executor
resources or a generic component graph. Its 8 MiB archive cap is not a bound on
all reconstruction work. See the [example contract](/examples/cursor/README.md)
and [delivered report](/design/cursor/implementation0.gpt61sxh.md).

## 8. Logical/generated inputs coexist with physical facts

Logical identity, physical observation and desired content are separate axes.
The identity is stable through mappings and private staging. A generated desired
file does not become a real source-side filesystem observation, nor target-only
deletion authority because no source file physically exists.

Illustrative data distinctions, not a selected public Rust API:

```rust,ignore
enum DesiredContent {
    Physical { input: InputId, relative: NativePath },
    Generated { recipe: RecipeId, parameters: Parameters },
}

struct SelectedInstall {
    logical: LogicalPath,
    target: TargetRef,
    content: DesiredContent,
    metadata: InstallMetadata,
}
```

`InputId` resolves through a runtime binding; `RecipeId` selects a known compatible
implementation. Neither embeds an open descriptor or executable code. This leaves
the representation of bytes/streams, parameter typing and dispatch open. Ordinary
physical copying is simply the common content source, not a JW-only path.

### Generated desired entries enter through an input seam

A logical provider can declare generated names before assessment, or a completion
reducer can emit generated values after children. Neither requires permitting a
late physical visitor to insert fake `Metadata` or mutate observed slot identity.
The existing physical `Presence` remains a physical listing fact. A broader input
interface must separately express desired logical presence and backing provenance;
it must not silently overload that fact with virtual source entries.

For entries that participate in reconciliation, the provider declares desired
shape, content reference and enough semantics to assess target satisfaction.
Generated attributes are declared attributes, not `std::fs::Metadata` fabricated
as physical evidence. A comparison rule must explicitly say how target
satisfaction is established: the physical stat-key predicate does not automatically
compare a generated recipe to bytes already on disk. Until that rule exists,
“unknown” cannot be reported as equality.

A changed action parameter that changes intended output must participate in the
desired-content/satisfaction semantics. It cannot be smuggled past a physical
“equal, therefore no-op” result. This does **not** preselect hashing, repeated
rewriting, sidecar receipts or any particular generated-content equality scheme.

The composition reserves a generated name or resolves its conflict with physical
input before publication. Mappings likewise declare collisions, ancestor routing
and target ownership before effects. Two logical entries mapped to one target
cannot be independent sibling jobs merely because their logical names differ.
Rejecting collisions is a useful minimal policy; more elaborate merging is an
explicit caller decision, not accidental enumeration order.

### Completion-produced content is another valid composition

A postorder summary can be a logical result consumed by a renderer or another
reconciliation. Emission is not installation. If installed back into an input
tree, the composition must explicitly exclude/reserve that output from its own
aggregate or define the resulting dependency. Otherwise a summary can count its
prior version, and “finish after children” conceals a self-reference.

**Specified/delivered:** current late action slots permit neither new paths nor
content recipes, and the rich walker emits only baseline-divergent directories.
That is sufficient for today's action table, including its ban on force-installing
equal leaves. It is **not** an intended restriction that all future selected work
must coincide with baseline physical divergence. Publication in an extended
composition must include its selected work or explicit step/finish obligation,
even when no legacy `DirDiff` would be emitted.

The cursor specimen already generates real summary bytes from a persisted
parameterized descriptor at completion. It emits them outside both trees and
does not install them, compare generated desired inputs, support arbitrary mapping
or implement an extensible action framework. See `Descriptor::generate`,
`Cursor::advance`, `Tip::validate` in
[`examples/cursor/src/cursor.rs`](/examples/cursor/src/cursor.rs).

## 9. Work avoidance has separate proof and coverage obligations

A component can eliminate work when it has sufficient declared knowledge. It must
say **which work disappears**, **what evidence permits that**, and **which coverage
claim remains**.

| Knowledge/decision | Possible saving | What it does not imply |
| --- | --- | --- |
| Name-level preserve decision | Child metadata/readlink, subtree enumeration and all effects below that boundary | Equality of hidden contents or full-root coverage |
| Late preserve / keep-leaf decision | Selected effects; possibly descendant enumeration | Saved immediate observation already performed |
| Finalized stopped descent | All future child work for that unit | That its subtree was inspected or proved equal |
| Known explicit leaf list | Discovery of unlisted source directories | A complete tree inventory or authority to remove unlisted target names |
| Sufficient whole-subtree equality evidence | Per-entry assessment/descent and effects within that evidence's scope | Equality under a stronger predicate or of unmapped physical paths |
| Explicit authorized whole-region materialization | Fine-grained difference calculation may be unnecessary | That enumeration/transfer/removal costs or descendant retention obligations disappeared |

Proven satisfaction is not the same as preservation or a deliberate skip. If a
future composition accepts whole-subtree evidence, its result records the scope,
predicate and grounding needed to distinguish discharged coverage from unexamined
coverage. This is a result distinction, not a demand for a persistent certificate
service. The current conservative summary need not be reinterpreted to claim this
capability already exists.

For example, a native tracked-tree identity may establish source logical content.
It does not alone establish that the receiving filesystem matches it, that ignored
files match, or that a mapped/generated output is satisfied. Conversely, a caller
with a complete known input need not relist the source merely to conform to the
shape of the physical walker. The accepting adapter owns that scoped reasoning.

**Specified/delivered:** early preserve, finalized stopped descent, `prune_here`
and `seed_leaves` already save different categories of work. No default subtree
equality shortcut exists. `Summary` conservatively separates exhaustion from full
coverage/cleanliness; sync policy satisfaction is a different result. A future
evidence-based shortcut must preserve that honesty rather than relabeling a prune
as clean. No automatic pruning/reconciliation heuristic is selected here.

## 10. Native consumer boundary: JW is an adapter, not the transfer model

The normal JW input for this effort is the **JJ archive**, as commissioned. A
registered workspace as source is a separately selected binding, not an implicit
replacement for studying the archive. The native adapter resolves the source's
actual layout and exposes the ordinary desired data plus any native-specific
operation inputs needed by its command. Generic copying does not need to know why
that source was selected.

The native runtime owns workspace registration, receiving identity, repository
links and any supported working-copy-state adoption. A receiving `.jj`/`.git`
layout is not obtained by blindly copying the source's administrative directory.
Native operations that depend on copied contents occupy declared ordered
positions around the generic transfer. Their results are not fabricated by a
pure visitor or inferred from `Report.completed`.

**Specified/delivered:** JW's `Auxiliary` visitor captures frozen selected roots,
routes and device inputs; it invokes no native process in early/late hooks.
`workspace::materialize_clone` and `adopt_source_tree_state` are separately owned
native operations. `ownership::record_created` records actual new Git topology;
mere copied files do not establish it. These sources show useful boundaries, not
a selected future broad-copy policy.

Default-on Git colocation with opt-out is the preferred product direction, **not
an established capability claim**. Current `CreationMode::resolve` makes CoW and
colocation mutually exclusive, and `JjClient::require_workspace_colocation` probes
the executable and a Git backend. The exclusion is delivered selection policy;
it does not prove a combined native protocol impossible or feasible. That needs
native evidence for the archive/receiver layouts, materialization/state-adoption
interaction and ownership/topology outcomes. The generic interfaces can carry the
result either way without preselecting feasibility or an upgrade strategy.

## 11. Smallest unresolved contracts and discriminating work

The architecture above does not require a general action language, complete
component registry, effects journal or broad native-state framework. The remaining
decisions are narrower:

| Contract still to settle | Smallest discriminating work | What the result selects |
| --- | --- | --- |
| **Effectful pause granularity and aggregate ownership.** Is one opaque recursive apply unit sufficient, or must its child/finish state be externally continuable? | A tiny component-built cursor with one newly created directory, two leaf installs and a finish action. Fail after one visible install and separately before finish; save/retry/skip the unresolved phase. Trace attempts, accepted folds and actual target contents, not just counters. | The minimal pending-work/phase DTO and whether recursive runtime state must be exposed. It must not claim rollback or exactly-once effects. |
| **Generated target satisfaction.** What proves a parameterized generated file already satisfies its desired value? | One generated leaf with a stable descriptor, then changed parameters, against absent/equal/different targets. Compare candidate satisfaction rules under their stated metadata/content costs. | A narrow generated-input comparison contract and no-op rule; not automatically a hashing or receipt subsystem. |
| **Desired/logical name admission and mapping conflicts.** Where are generated names introduced, and what happens to a real name or another mapping at the same target? | An immediate batch with one physical source name, one generated desired name, a real target collision and a two-to-one mapping. Require conflict resolution before effects and preserve actual physical presence. | The smallest provider/slot extension and routing validator, without opening sealed physical plans to fabrication. |
| **Selected-work publication independent of baseline diff.** Can a clean physical directory carry legitimate generated or finish work? | Use equal physical inputs with one selected generated operation and stopped descent on another child. Drive once through a report-only projection and once through the component driver. | Whether a separate work/step projection suffices or an existing rich walker surface must broaden; legacy silent clean diffs can remain. |
| **Serializable component compatibility.** What must be persisted versus rebound? | Round-trip one parameterized reducer and one executor continuation through the existing optional codec seam; reject a missing handler or incompatible parameter schema before starting work. | Minimal caller-supplied descriptors/snapshots, not a mandatory registry for all users. |
| **Whole-subtree knowledge scope.** Which predicates can actually discharge observation/descent? | Feed one known logical subtree and separately a bound target-state assertion into a tiny adapter; withdraw each assertion in turn. Check which enumeration, comparison and effects remain necessary and how coverage is labeled. | An evidence acceptance contract that distinguishes proven satisfaction from preserved/skipped coverage. |
| **JW archive → colocated receiver native compatibility.** Does the desired combined path work on the actual native interfaces? | A separate tiny native fixture using the normal archive source, desired receiving registration/layout and the intended data/state materialization sequence, including opt-out. Inspect JJ and Git results and owned topology rather than only CLI acceptance. | Feasibility and adapter inputs/finish obligations. Current mode rejection alone answers neither question. |

The first and third rows are the strongest interface discriminators: they expose
whether the proposed boundaries are actually usable by an assembling caller
without demanding every runtime detail in the core. Generated equality and native
colocation are independent studies; neither should be silently decided by a
cursor DTO or today's action enum.

## Evidence and intersections

- The [execution flow](/design/execution/flow0.gpt6a.md) establishes information,
  control and bytes as separate flows. This statement adds ordered child/finish
  obligations and caller assembly rather than replacing that account.
- The [state ledger](/design/execution/state0.gpt6a.md) grounds short-lived physical
  facts and runtime ownership. Continuation adds only state required for future
  progress; it does not promote those facts into durable authorization.
- The [cursor specimen](/examples/cursor/README.md) establishes a useful assessment
  unit, no-root-replay frames and real generated values. It supplies no evidence
  yet for mutation continuation, arbitrary mappings or installing virtual inputs.
- `Roots::inspect_visitor` in [`src/pair.rs`](/src/pair.rs), lines 67–104, and
  `PlanWalk::advance` in [`src/walk.rs`](/src/walk.rs), lines 175–211, make the
  baseline-divergence publication dependency concrete. Extended selected work
  cannot inherit that dependency without examination.
- `DirectoryPlan::children` in [`src/visitor.rs`](/src/visitor.rs), lines 569–575,
  and `execute_entry` in [`sync/src/apply.rs`](/sync/src/apply.rs), lines 490–571,
  show today's split between paired progression and recursive runtime ownership.
  It is the key intersection with any structure study of component-built cursors.
- `apply_plan` and `apply_public` in [`sync/src/apply.rs`](/sync/src/apply.rs), lines
  34–147, distinguish a public revalidation boundary from immediate internal reuse.
  A future restored operation must not inherit freshness merely by carrying a
  plan-shaped object.

No code, other authors' documents, indexes or native checkout state were changed.
Earlier reported tests were inspected as evidence, not rerun or presented as new
verification. This independent statement leaves synthesis and native feasibility
to their commissioned work.
