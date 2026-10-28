---
type: ArchitectureDraft
title: Coordinated source-to-many comparison
description: Refinement of the composable tree architecture with one coordinated comparison frontier, shared immediate source acquisition, destination-specific branch membership, deterministic publication, and explicit resource and failure boundaries.
resource: /design/architecture/draft1.gpt6a.md
status: draft-for-design-selection
generated: { by: "model:openai/gpt-6-astra#high", at: 2026-10-22 }
sources:
  - resource: "opencode://ses_e9e45e9f4ffeZUWoZCK5VDjJvv"
    title: Human requests coordinated comparison; develops its contracts and connects the subsequently accepted domain-owned crate decomposition
  - resource: /design/architecture/draft0.gpt6a.md
    revision: 7bde51f6
    title: Parent architecture refined here, especially source-to-many composition and parallel resource ownership
  - resource: /README.md
    revision: 7bde51f6
    title: Delivered comparison specification and the existing coordinated-multiple-destinations direction
  - resource: /doc/architecture/README.md
    revision: 7bde51f6
    title: Maintained component boundaries and comparison versus effectful progression
  - resource: /design/transfer/multi-target0.gpt61sxh.md
    revision: 947f02b7
    title: Bounded source-acquisition study, per-target independence and the contrasting detach-and-reobserve execution composition
  - resource: /design/architecture/statement1.gpt61sxh.md
    revision: 269b81f2
    title: Previous amended architecture's independent target-continuation recommendation
  - resource: /src/visitor.rs
    revision: 7bde51f6
    title: Inspected immediate listing, pair union, early admission, ordered acquisition and finalization seam
  - resource: /src/pair.rs
    revision: 7bde51f6
    title: Inspected root gating, finalized paired children and clean-fact release
  - resource: /src/walk.rs
    revision: 7bde51f6
    title: Inspected deterministic DFS, committed summaries and post-yield pruning
  - resource: /src/model.rs
    revision: 7bde51f6
    title: Inspected immediate diff counts and independent scan/coverage states
  - resource: /src/lib.rs
    revision: 7bde51f6
    title: Read-only physical comparison boundary and caller-owned quiescence
extensions:
  refines: /design/architecture/draft0.gpt6a.md
  related_work: [diffdir-tree, diffdir-tree-transfer-multi-target]
verification:
  method: Source-grounded design review and document checks; proposed implementation gates are listed separately
  result: Four connected documents passed YAML, native-session provenance, source-path, whitespace and fence checks; 91 local links and anchors resolved
  scope: Interface and state sketches are proposals; the earlier study's bounded experiments are attributed evidence, not tests of this coordinated walker
---

# Coordinated source-to-many comparison

**Accepted packaging refinement:** [Domain-owned crates](draft2.gpt6a.md) records
the subsequent human-selected crate layout and path-derived names. Its extraction
sequence prepares the inspection/traversal boundary before implementing this walk.

## 1. Design delta

**Make one-source, many-destination comparison a first-class read-only composition:
one coordinated directory frontier, one immediate source observation per visited
source directory, and independent comparison results and coverage for each
participating destination.**

This refines [draft0](/design/architecture/draft0.gpt6a.md), particularly
[§12](/design/architecture/draft0.gpt6a.md#12-source-to-many-composition), and
develops the already-recorded
[coordinated comparison direction](/README.md#coordinated-multiple-destinations).
The human has requested this capability in the design. The mechanisms below are
the proposed refinement; they are not delivered APIs or separately accepted details.

The main architectural change is that comparison coordination owns traversal,
rather than waiting for independent destination walkers to meet at the same source
coordinate. Destination independence means independent policy, classification,
failure and coverage; it need not mean an independent traversal cursor for each.

| Concern | Refined decision |
| --- | --- |
| Recursive owner | One coordinated comparison frontier with destination membership on every child edge. |
| Source acquisition | One complete immediate listing and explicit union of participating source-fact demands. |
| Comparison | One source/destination name union and pure assessment per destination. |
| Publication | Joined directory groups in native-name DFS order; destinations within a group use binding order. |
| Failure | A failed destination comparison terminates that destination's lane; healthy lanes continue. |
| Slow destination | Backpressure within the current joined group; no automatic timeout or detachment. |
| Resources | Current source batch, bounded active destination work, owned future edges; no source-tree catalog. |
| Effects | A separate composition using the existing executor/continuation boundaries. |

The [bounded source-cohort study](/design/transfer/multi-target0.gpt61sxh.md)
remains useful for independently advancing effectful destinations. Its
detach-and-reobserve recommendation is not the default scheduler for this new
comparison composition. Both share the acquisition/assessment seam.

Read [§2](#2-components-and-bindings) for ownership,
[§3](#3-one-directory-inspection) for acquisition,
[§4](#4-one-frontier-with-destination-membership) for traversal,
[§5](#5-publication-pruning-and-summaries) for the consumer contract, and
[§10](#10-landing-and-proof-obligations) for implementation gates.

## 2. Components and bindings

A **lane** is one destination's comparison identity, policy, active/failed/stopped
state and summary. It is not a second walker. A **group** is one immediate source
directory together with the lanes admitted to compare that directory. An **epoch**
is the joined observation/assessment work needed to produce that group.

```text
source binding ───────────────┐
destination bindings + policy ├── coordinated comparison frontier
                             │       ↓ one group
observation resources ────────┴── source listing + destination listings
                                     ↓
                              per-lane early admission
                                     ↓
                              shared source fact demands
                              lane-local destination facts
                                     ↓
                              per-lane finalized assessment
                                     ↓
                              group publication + child memberships
```

The read-only core owns this comparison frontier, as it owns today's pair walker.
It need not depend on the optional effectful `walk/` controller proposed in draft0,
sync, a native repository client, or a codec. Immediate inspection remains usable
without either walker. Shared algorithms may be extracted where actual code reuse
justifies it; a universal traversal framework is not a prerequisite.

Bindings supply:

- One physical source root and its interpretation.
- A nonempty, finite ordered collection of unique `TargetId`s, real directory
  roots, and per-target frozen policy inputs.
- Caller-owned observation resources and execution limits.

Use existing root requirements first: source and destination roots are real
directories, not symlinks. Invalid root binding fails construction with the
offending source/target identity; no lane has advanced. Missing-target initialization
belongs to the desired-work composition. Structural source metadata may be acquired
once, while root policy still runs independently for each lane. Root preservation
can remove a lane before any interior listing is needed; it records preservation,
not a failed binding or a clean tree. If every lane preserves the root, do not list it.

`TargetId` is stable identity; caller-supplied binding order is the deterministic
publication tie-break. IDs need not be sortable, and changing a vector's internal
layout must not reinterpret saved identities in a future continuation adapter.

Because this operation is read-only, equal or nested physical roots are not write
conflicts. Two distinct IDs may intentionally compare the same directory under
different policies. Do not silently coalesce their results. A later effectful
consumer must apply its own alias, overlap and claim checks before mutation; a
comparison binding grants no write authority. The initial multi-target run assumes
caller-controlled stable source and destination trees throughout comparison.

## 3. One directory inspection

The useful refactoring is inside immediate inspection. Today's
[`inspect_directory_execution`](/tree/inspect/src/visitor.rs) lists both sides, then privately
merges names, applies early admission, acquires facts and finalizes one plan.
Reusing a sealed `DirectoryPlan` from A as B's plan is incorrect.

The coordinated inspector performs these explicit stages:

1. **List source once.** Retain complete sorted native names, enumeration hints and
   the private acquisition handles needed for this epoch. A failed or capacity-
   refused listing is not an empty directory or evidence of absence.
2. **List each participating destination.** Build each lane's own ordered pair
   union, including destination-only names. Store pair slots referring to common
   source entries rather than copying all source names into every internal union.
3. **Run early admission per lane.** Each visitor sees exactly its pair union and
   its own roots/context. A preservation decision removes only that lane's demand
   for the name or subtree. It never changes common source presence.
4. **Acquire the union of source demands.** Acquire each needed physical fact once
   for this epoch, including literal link text where required. Cache an acquisition
   result or failure for the epoch. Stat used to resolve kind also supplies its
   available attributes; do not issue another stat just to expose those attributes.
5. **Acquire lane-local destination facts.** Preserve each lane's required fact
   profile. Neither A's target satisfaction nor its target error is a source fact.
6. **Assess and validate per lane.** Supply immutable facts to ordinary comparison
   and late policy. A lane publishes a complete successful immediate assessment or
   its ordered error, never a partially successful count for that directory.
7. **Lower and release.** Produce owned differences, summary contributions and
   sorted child memberships. Release scratch handles and source facts before
   descending. A source entry's name may be promoted/shared into owned child state;
   its entire metadata batch must not remain pinned just for that name.

The first implementation can use existing full-fact visitor demands. Narrow
`Kind`/`Attributes`/`LiteralLink` demand remains an optimization-compatible extension,
not a requirement to redesign every public visitor now. Full-fact visitors retain
their current payloads; sparse fact access must never fabricate missing Metadata.

The source batch is privately assembled, then frozen for assessment. Accessors do
not perform I/O or acquire hidden locks that mutate shared observations. Consumers
cannot supply invented source facts as if the physical observer acquired them.

**Example of demand isolation:** A preserves `secret`, B compares it, and C failed
its destination listing. Only B requires source facts for `secret`. If acquiring
those facts fails, A can still complete its directory and C retains its earlier
listing error. B receives the source error at B's ordered observation boundary.

The core predicate stays type/size/exact supported mtime for regular files and
literal native symlink spelling for symlinks. Sharing comparison observations does
not request content reads, hashing, or source-file handle fanout.

## 4. One frontier with destination membership

A minimal sequential frame is conceptually:

```rust,ignore
struct GroupFrame {
    relative: NativePath,
    lanes: Vec<LaneFrame>,
    remaining_children: VecDeque<ChildEdge>,
}
struct ChildEdge {
    name: NativeName,
    members: TargetSet,
}
struct LaneFrame {
    target: TargetId,
    direct_divergent_ancestor: bool,
    subtree_changed: bool,
    // Local coverage and any required lane-specific reduction state.
}
```

These are semantic sketches, not a prescribed vector/deque wire layout.
`TargetSet` initially can be sorted binding indices; a bitset or interned repeated
set is a representation choice informed by fanout and branch divergence.

For each successful lane, obtain its finalized paired-directory child selection
from the same rules as the existing pair walker. Merge those child lists by native
name. Each resulting edge carries precisely the lanes that admitted that child.
Traverse the merged child sequence in DFS preorder. At child closure, fold only
into participating lane states. One owner advances and closes each grouped edge.

This is the union of admitted **paired** traversals, not unconditional traversal
of the source tree. A source directory with no participating destination is never
opened. Equal immediate files do not imply that a paired child directory is clean;
the ordinary recursion rules still apply.

### Different shapes at the same name

Suppose the source contains `pkg/`:

| Lane's target `pkg` | Immediate result | Membership in `pkg/` comparison |
| --- | --- | --- |
| Real directory, descent admitted | Ordinary directory pairing | Included |
| Absent | Source-only added directory | Excluded; interior remains opaque |
| Regular file or symlink | Existing kind-collision classification | Excluded; no invented directory pairing |
| Real directory, preserved by policy | Scoped preservation/retention | Excluded |

Only the included lanes share the source `pkg/` listing. Similarly, a target-only
`extra/` is reported for that target and remains opaque. This retains current
one-sided subtree counts and comparison coverage. Recursive inventories of missing
subtrees are a different comparison mode with different result/count semantics.

### Projection invariant

Under stable trees, identical per-target policy and equivalent pruning, filtering
the coordinated results to lane T must give the same differences, successful
directory sequence, ordered first error and final summary as independently walking
source versus T with today's comparison semantics. An immediate successful clean
assessment may appear in the richer group surface; the legacy diff projection
filters it out.

This is an observable-result invariant, not identical callback counts, syscall
timing or a promise under concurrent edits. Pure visitors may already be repeated
or speculative. Pruning successful clean groups is an additional control opportunity
of the richer API; equivalence with today's iterator restricts explicit prunes to
the divergent records that iterator exposes.

## 5. Publication, pruning and summaries

Use a complete immediate group as the primitive pull result. Illustrative shape:

```rust,ignore
struct ComparedDirectory {
    token: GroupToken,
    relative: NativePath,
    outcomes: Vec<TargetOutcome>, // exact participating set, in binding order
}
enum TargetOutcome {
    Compared { target: TargetId, diff: DirDiff, coverage: LocalCoverage },
    Failed { target: TargetId, error: CompareError },
}
```

`Compared` includes clean outcomes. An absent lane means it did not participate
in this directory, not that its target was equal or absent. Overall lane state and
parent membership explain whether it was preserved, one-sided, failed or stopped.
Root-only preservation can be represented in the same successful coverage surface
without claiming that a source directory was enumerated.

The primitive `next_group` performs one group epoch and required pure stack
transitions. It can return a clean group; it is not the difference-only iterator
that can silently scan arbitrarily many clean directories in one call. The public
shape can be a method or iterator, but errors inside a group stay target-tagged.
Every normal return is joined: no listing or metadata task continues in the
background after the caller receives the group.

If an invocation budget is offered, charge one attempted group epoch, including a
failed attempt; report lane assessments separately. Zero budget performs no new
epoch. A group can contain many destinations and a very wide directory, so this
unit is not a byte, syscall, memory or latency limit.

Commit successful lanes' immediate contributions before returning the group;
mark failed lanes failed. This matches today's committed-on-yield comparison
behavior. There is no external effect acknowledgment. A group token only identifies
the still-open control window for its children; it is not an execution ticket.

Before the next pull, the caller may:

- **Prune selected successful lanes here.** Remove those lanes from every pending
  child of this group. Record this directory in each affected lane's `pruned`
  coverage, even if it has no children. Other lanes retain their child memberships.
- **Stop selected lanes for the run.** Remove their future memberships throughout
  the frontier and retain their already committed summaries as incomplete. This
  is not normal exhaustion or failure.
- **Advance unchanged.** Close the control window and admit the selected children.

An illustrative `prune_here(group_token, targets)` validates the whole request
before applying it: current token, successful participants and not already pruned
in this window. Distinct subsets may be pruned in successive calls. Stale, failed,
unknown or repeated members are rejected without partial changes. Selection is
explicit; an unqualified prune-all operation must be named as such.

A difference-only convenience projection retains a group boundary while filtering
clean successes; it must still surface failures in an otherwise clean group.
Entirely clean groups can be drained internally, as with today's iterator. Prefer
this grouped projection before a flattened target-event stream: the latter needs
careful buffering to avoid closing the pruning window while another lane's record
from the same directory is still awaiting delivery.

### Separate result and resource accounting

Every lane has its own existing `Summary` semantics: counts, excluded/retained
coverage, `topmost_divergent`, `changed_dirs`, examined directories, prune records
and scan state. Direct-divergent ancestry and subtree-changed propagation are
per-lane, not a group-wide flag. Postorder closure must preserve each lane's
`changed_dirs` ordering even when other lanes visit additional branches.

Resource diagnostics separately count source directory acquisitions, source facts,
destination acquisitions, group attempts, peak memberships and retained bytes.
Ten destination comparisons of one source directory contribute ten lane directory
assessments but one source listing. Neither count is a descendant file count.

Normal group traversal exhaustion marks each nonfailed, nonstopped lane complete
as a scan. Scoped/preserved/pruned coverage still prevents full-clean certification
according to `Summary`. Early drop produces committed-prefix summaries without I/O.
An all-target-clean convenience predicate requires every requested lane to be clean;
iterator exhaustion alone is not that predicate. Group-level failures should be
reported through lane identities, so a result cannot hide an unsuccessful target
merely because its peers completed.

## 6. Errors, backpressure and ordering

Preserve each lane's logical failure precedence:

```text
source listing → target listing → early policy
→ admitted union-name order, source before target at the same name
→ late policy → action validation
```

Parallel acquisition may discover errors in another order. Retain enough indexed
outcomes to select the lane's first semantic error. An eagerly acquired source
metadata failure must not outrank that lane's target-listing/early error, or an
earlier name's metadata failure. One lane's policy refusal is never cached as a
physical source failure for its peers.

- A source-directory listing failure fails every participating lane that needs it.
  Lanes excluded by root/ancestor preservation never participated in that listing.
- A source-entry fact failure fails only the lanes requiring that fact, subject to
  their own earlier error precedence.
- A target listing, target fact, policy or validation failure fails that lane.
  It contributes no counts for the failed directory and no children from it.

A failed lane terminates at its first ordered error, like the current pair walker.
Remove it from outstanding sibling memberships as well as the current children;
retain earlier committed counts and changed-ancestor evidence, never mark it clean
or completed. Healthy lanes can continue through the shared frontier. Emit its
error once. A later retry is a fresh run for that destination in this first design,
not automatic reattachment to the current frontier.

The current epoch is a barrier. A slow destination can delay group publication;
bounded worker admission controls resource use, not latency. There is no automatic
detach-on-slowness policy, timeout, or promise to interrupt a filesystem syscall.
After a returned failure or an explicit between-group stop, healthy lanes proceed.
Applications requiring independently progressing destinations can choose the
cohort/continuation composition instead. This is a declared tradeoff for deliberate
source-once comparison, not a hidden scheduler behavior.

The first walker is sequential between groups. Later parallel source-directory
lookahead must commit in the same DFS order and carry lane-specific seals. The
simpler baseline seals **all** current group children until publication and advance,
including children belonging to a currently clean lane. Target-level and
within-directory metadata tasks may run concurrently inside the joined epoch under
one caller-supplied executor and explicit shared admission budget. Do not construct
an unbounded nested worker pool per destination.

## 7. Lifetimes, capacity and what source-once means

For the initial sequential directory frontier, retain:

- One source batch of names, hints and demanded facts for the current group.
- Current destination listings and pair-slot/assessment data for participating lanes.
- Owned not-yet-visited child names and their destination memberships along the DFS
  stack, plus per-lane summary state.
- The published result while the caller owns it.

The dirent pool from [draft0 §8](/design/architecture/draft0.gpt6a.md#resource-control-and-dirent-buffers)
can be shared across all listings. Its scratch-buffer lease ends after enumeration;
it is separate from owned name/fact batches and future child storage.

If a group has S source entries and target widths T_i, source facts occupy O(S),
target data O(sum T_i), and pair references can still occupy O(N*S + sum T_i).
Child membership can be O(N*C) for C admitted child names. Sharing source facts
does not eliminate those pairwise decisions or make total memory independent of N.
Full public `DirectoryPlan` ownership might duplicate Metadata further; the
comparison-only group result need not expose or retain those full plans.

Existing summary vectors such as `changed_dirs`, `topmost_divergent` and `pruned`
can grow with the reported tree, separately for each lane. Caller-retained groups
can also accumulate. Bounded observation storage is not a total-memory guarantee;
a count-only or streamed-summary alternative would need an explicit result contract.

Specify separate limits for active jobs, source/destination name and fact payloads,
pair slots, future edge/membership storage, and buffered results. Complete-directory
visitor semantics mean a single wide directory can exceed a configured capacity.
Refuse an affected lane/group at its declared boundary with explicit incomplete
coverage; never truncate a listing and publish absence or a successful partial
batch. A common source capacity failure affects all participating lanes. A
destination-local capacity failure can remain local. If a global frontier-capacity
limit cannot accommodate the proposed group, reject that group before committing
its contributions and fail its affected lanes; do not select winners by worker race.

A small job-concurrency limit does not bound the memory for all completed target
listings waiting on joint demand collection. The first implementation may retain
all bounded current-group unions. Supporting arbitrarily many destinations with a
small fixed memory budget would require a deliberate windowed policy: retain more
source facts, replay destination discovery, spill, or accept repeated source
acquisitions. That tradeoff is not hidden inside the word "streaming."

For an ordinary successful run with sufficient configured capacity, each visited
source directory is listed once and each demanded source fact acquired once per
epoch. Source bindings use identical relative routing in this initial mode, so the
frontier does not revisit that directory under alternate logical mappings. A future
retry/restore or explicit fallback may reobserve; diagnostics must count those
additional acquisitions honestly.

Sharing establishes that participating comparisons consumed the same acquired
source evidence. It does not establish an atomic source snapshot or protect a live
tree from change. As with current comparison, callers own quiescence. A program
needing revision-consistent comparison should bind an appropriate stable checkout
or snapshot rather than ask the walker to infer stability from directory mtime.

## 8. Worked traversal

Consider the root pair unions below. `same`, `old`, and `new` denote metadata
outcomes under the ordinary comparison predicate, not byte verification.

```text
source             destination A        destination B       destination C
config (new)       config (same)        config (old)        [missing config]
pkg/               pkg/                [missing pkg]       pkg/ [preserve]
  lib (new)          lib (old)
                   extra.log
```

1. The root group lists source once and each target once. `config` is equal for A,
   changed for B, added for C. A's `extra.log` is removed in the baseline diff;
   comparison itself deletes nothing.
2. B records `pkg/` as one added directory. C records scoped preservation. The
   merged `pkg` child edge contains only A. No descendant count is inferred for B
   or C, and their policy does not suppress A's source demand.
3. The root group is published. Pruning A here removes the only `pkg` membership,
   so the source `pkg/` is never listed. Pruning B affects B's coverage but cannot
   remove A's membership.
4. With no prune, descend into `pkg` for A, list that source directory once, and
   report A's changed `lib`. Close A's child and fold A's changed-directory ancestry.
5. Final summaries remain distinct: A has the extra file and nested difference;
   B has a changed file and one opaque added directory; C has an added file and
   preservation preventing a full-clean certificate.

For a two-member `pkg` edge, failure listing A's `pkg` produces A's terminal error
while B can publish and continue its own assessment from the common source batch.
Failure listing the source `pkg` fails both members. Neither failure reaches a
third lane that had already excluded `pkg` from its traversal.

## 9. Composition with the rest of the architecture

**Single-target comparison.** Keep the existing public API and contracts. Factor
the acquisition/finalization internals so the coordinated path can reuse them.
N=1 is the semantic equivalence gate; migrating the old walker implementation onto
the new frontier is optional and should follow evidence, not precede it.

**Synchronization and generated/native work.** Comparison results are observations,
not accepted effects or fresh authorization for later mutation. Generated work in
physically clean directories still needs the work-bearing protocol from draft0.
Do not make the coordinated comparison driver execute plans to obtain source reuse.
Fine-grained effectful progress retains its before/children/finish obligations,
target claims and pending acknowledgments. If an effectful coordinator is selected
later, it must own its expanded edges itself rather than race the comparison
frontier for child progression.

**Independent source cohorts.** The earlier study's source acquisition/assessment
split, native-name ownership and lane-local errors support this design. Its
detach/reobserve lifecycle remains an alternative for independent destination
progress. The read-only coordinator's membership frontier is new work; the study's
eight tests do not establish this frontier's projection, pruning or ordering.

**Pause and persistence.** The first supported boundary is an in-process joined
group return; stopping/dropping yields prefix summaries. Durable continuation can
later save bindings, frame suffixes, memberships, per-lane state/aggregates and any
unresolved group-control window. It must release live observations and reconstruct
runtime resources. This needs its own state-validation and process-rebind tests;
neither serializing N existing cursors nor serializing an `Arc<SourceBatch>` defines
the coordinated machine's continuation. Exactly-once output delivery is separate.

## 10. Landing and proof obligations

### Coherent implementation slices

1. **Factor immediate acquisition and assessment.** Keep pair inspection behavior
   intact; establish private owned source batches, per-pair early admission and
   reusable finalization. Preserve full-fact callbacks and error precedence.
2. **Add a sequential coordinated read-only walker.** One source, ordered target
   bindings, merged child memberships, complete group publication, per-target
   summaries/errors and explicit pruning/stopping. No persistence requirement.
3. **Expose convenient reporting.** Add a grouped diff-only projection and consumer
   examples once the primitive control window is sound. Keep IDs and incomplete
   outcomes visible. CLI schema/flags are a consumer decision, not implicit here.
4. **Add bounded within-group parallelism and measure.** Use supplied resources;
   test varied destination counts and uneven widths. Add inter-group speculative
   lookahead only if the simpler grouped path leaves a demonstrated opportunity.

### Checks that distinguish a correct implementation

| Discriminator | Required result |
| --- | --- |
| N=1, N=2, several destinations | Each lane projects to the same pairwise differences, ordered error and summary on stable fixtures. |
| Different presence/kinds | Paired, absent, colliding and target-only directories retain today's opaque/recursive boundaries. |
| Different policies | A's preservation never hides B's source presence/demand; all-lane exclusion avoids unnecessary facts and descent. |
| Clean parent, nested differences | Per-lane `topmost_divergent` and postorder `changed_dirs` agree with independent walks. |
| Prune A, continue B | A's membership disappears at the current directory; B's descendants remain eligible; stale/repeated controls have no partial effect. |
| Failure in A's child | A's queued siblings are removed, its error is delivered once, B finishes, and A retains only its committed prefix. |
| Shared source failures | Only demanding lanes fail; earlier target/early/name errors still take precedence for their respective lanes. |
| Stop/drop | No extra I/O; stopped lanes remain incomplete; prefix summaries exclude speculative or unpublished contributions. |
| Source acquisition census | In a successful stable run, each visited source directory is listed once and each demanded source fact acquired once, not once per lane. |
| Lifetime and capacity | Parent fact batches are released before descendants; native names survive scratch reuse; cap failures never look like partial complete listings. |
| Ordering under parallel completion | Varied task completion orders give identical groups and per-lane summaries, and every return joins started work. |
| Alias bindings and native names | Repeated read-only roots under distinct IDs remain distinct lanes; lossless non-UTF8 names compare and order correctly. |

Use tiny fixtures and fault injection to verify semantics before workload claims.
Compare end-to-end cost with N independent walks: source listing/stat counts,
destination work, sorting/merge cost, allocation/peak retained storage, elapsed
time and first-result latency. Common source I/O is reduced; destination I/O and
N pairwise decisions remain. Source-sharing savings can be outweighed by large
pair-state costs or a slow group member, especially for N=1 or warm caches.

The earlier study measured source acquisitions falling from four listings to two
and seven metadata acquisitions to four in its tiny two-target fixture. Those
application counts motivate the seam; they are not a benchmark or verification of
the coordinated implementation proposed here.

## Cross-references and standing

- [Original parent draft](/design/architecture/draft0.gpt6a.md): retained overall
  architecture; this iteration refines source-to-many comparison ownership and its
  relationship to the optional effectful controller.
- [Maintained architecture](/doc/architecture/README.md): navigation home connecting
  the refinement with the current read-only, sync and continuation boundaries.
- [Root specification](/README.md#coordinated-multiple-destinations): existing
  streaming coordination intent; delivered APIs still compare one pair.
- [Source-cohort study](/design/transfer/multi-target0.gpt61sxh.md): supporting
  acquisition/lifetime evidence and the intentionally different independent-target
  scheduler. Its write-overlap restrictions belong to effects, not read-only aliases.
- [Previous amended statement](/design/architecture/statement1.gpt61sxh.md): prior
  architecture context; the independent-cursor preference is narrowed here rather
  than retroactively presented as coordinated-frontier evidence.

The newly chosen capability is coordinated comparison. Public Rust naming,
membership representation, explicit capacity defaults, CLI presentation and whether
durable comparison continuation is needed remain open implementation/product choices.
The ownership, projection, error-order and publication contracts above provide the
design boundary within which those choices can be made.
