---
type: ImplementationPlan
title: Coordinated comparison and resumable reconciliation rebuild
description: Recovered design sequence, current commission, implementation decisions and checkpoints for source-to-many comparison and usable continuation.
resource: /design/architecture/rebuild0.gpt6a.md
status: implementation-in-progress
generated: { by: "model:openai/gpt-6-astra#high", at: 2026-10-28 }
sources:
  - resource: "opencode://ses_e7eb31c84ffev4GUeYjdmZO7mY"
    title: Human commissions coordinated comparison and resumability, distinguishes partial traversal heads from unsaved in-flight operation progress, and requests sequential implementation plus final design and ticket consolidation
  - resource: "opencode://ses_e7eb2765effe0QpPEd1cYR8AXz"
    title: Independent design reconstruction identifies production retry ownership and distinguishes research completion from delivered capabilities
  - resource: "opencode://ses_e7eb22fb2ffeH5O20EZV5nW4B5"
    title: Independent source assessment identifies shared acquisition, lane reduction, ordered-error and resumable frontier obligations; commissioned for the first inspection implementation
  - resource: "opencode://ses_e7ea319dfffeZci454IqpWo77c"
    title: Fresh continuation review models partial destination heads and selects exact unstarted-decision refresh when opaque failures are terminal
  - resource: /design/architecture/draft0.gpt6a.md
    title: Broad component and continuation design after the research flight
  - resource: /design/architecture/draft1.gpt6a.md
    title: Later coordinated comparison refinement and projection obligations
  - resource: /design/architecture/draft2.gpt6a.md
    title: Accepted domain decomposition and the previous extraction-only stopping point
  - resource: /examples/walk/README.md
    title: Delivered bounded effectful continuation specimen and its production-semantic limitations
  - resource: /README.md
    title: Starting delivered pair comparison, visitors, coverage and synchronization boundary
  - resource: /transfer/sync/src/lib.rs
    title: Existing production root binding, record application and caller-owned traversal seam
---

# Coordinated comparison and resumable reconciliation rebuild

## Commission

The October 28 commission prioritizes **one source compared/reconciled against
multiple destinations**, and also asks that the resumable design be tackled.
Implementation proceeds sequentially, supported by two independent Sol xhigh
analysts/builders. The parent owns architectural decisions and integration.
The completed result will receive a maintained design account and an epic with
explicit completed and remaining work.

The earlier extraction-only stop is historical scope for a completed pass. It is
not a prohibition on this new commission. Conversely, a research ticket's closure
does not establish that its proposed production capability has been implemented.

## Which document is which

| Reading order | Document | Role and starting standing |
| --- | --- | --- |
| 1 | [Broad design](draft0.gpt6a.md) | Post-research component, typed work, continuation and resource architecture; several proposed capabilities remain unimplemented. |
| 2 | [Coordinated comparison](draft1.gpt6a.md) | Later refinement: one traversal frontier, shared immediate source observations and destination-specific policy, membership, failure and coverage. |
| 3 | [Crate decomposition](draft2.gpt6a.md) | Accepted packaging and extraction sequence, delivered through facade/sync/CLI relocation. Its stop before coordination belonged to that pass. |
| Supporting research | [Source cohorts](/design/transfer/multi-target0.gpt61sxh.md) | Earlier independently advancing effectful destinations and detach/reobserve; a different scheduling composition from the coordinated comparison frontier. |
| Runnable precedent | [Walking specimen](/examples/walk/README.md) | Component construction, pending requests, budgets, retry/skip and actual fresh-process continuation, with deliberately limited copying semantics. |
| Delivered contract | [Root specification](/README.md), [sync specification](/transfer/sync/README.md) | Actual pair comparison and production filesystem effects at the start of this rebuild. |

The history confirms extraction commits `6b8d6ccb` (inspection/traversal),
`3db92935` (observation), and `46f61d30` (facade/sync/CLI relocation).
The initial working copy also contains three small documentation updates from
that work; their content is preserved.

## Architectural obligations

1. **Share physical source observations, not a destination's decisions.** Each
   lane retains its own name union, early admission, late policy and ordered errors.
   A source fact failure affects only lanes that require it.
2. **One owner advances each recursive edge.** Coordinated comparison merges
   paired-child membership into one native-name DFS frontier. Effect execution
   must not recursively advance those same paired edges a second time.
3. **Keep observations and completed effects distinct.** Committing a comparison
   group establishes observed coverage, not successful installation in any lane.
   Applying or acknowledging work requires its own state and result accounting.
4. **Pause with owned semantic state.** Directory handles, metadata batches,
   callbacks, thread pools and other live resources do not become saved state.
   Restoration validates data and explicitly reconstructs execution resources.
5. **Use production effects for production reconciliation.** The walking
   specimen's simple byte-copy executor is evidence for control semantics, not a
   replacement for sync's staging, replacement, timestamp and policy handling.
6. **Keep evidence proportional and discriminating.** Lane projection, pruning,
   failure isolation, acquisition counts, join boundaries and fresh-process
   continuation need direct verification before workload claims.

## Partial traversal is the saved state

The human corrected an insufficient parent framing of “closed steps.” A returned
I/O attempt and a completed traversal node are different things. **The root and
active ancestors normally remain partial at a pause.** Closing runtime handles or
temporary permission scopes does not close those nodes or settle their descendants.

Different destinations may admit different children and require different work at
the same source coordinate. Saving only the currently executing leaf, one global
path, or a supposedly completed root loses the other destination heads. The owned
frontier must preserve:

- every active ancestor and its lane-local phase, accepted local contribution and
  accumulated child results;
- the remaining ordered child suffixes with destination memberships;
- each destination's pending evaluation/effect and the edge it still owns;
- return and finish obligations, including destinations waiting at an ancestor
  while another destination progresses deeper;
- lane-local preservation, stop, failure and incomplete coverage;
- any still-open control or acknowledgment window.

For example, if A admits `a/` and `b/`, while B admits only `b/`, a pause deep under
`a/` must retain B's position waiting for `b/`, A's remaining `a/` obligations and
later `b/`, and both lanes' unfinished root reduction. A merged child-membership
frontier can encode these heads without independent duplicate source walkers.
Returning from `a/` folds only A's accepted result; it cannot complete B's root.

**The subsequent human clarification fixes the checkpoint scope:** retain
traversal and decision states, not partial progress *inside* in-flight work at a
node. A production subdirectory copy or removal may remain one opaque call. Save
between returned calls; do not save its internal recursive position, copied-file
history, byte offsets, staging resources or temporary permission state. Parent
frames can remain incomplete across many such calls. A failed operation can leave
filesystem effects, but the traversal checkpoint is not their recovery log.

`into_parts` preserves this machine. It must not call a summary extraction method
that closes active ancestors. Prefix summaries may *project* pending ancestor
closure without mutating saved traversal. Restoration must not replay accepted
parents, change binding-index identity, or lose an unfinished recursive owner.

## Selected sequence and review boundaries

The two independent assessments agree that the extracted owners are sufficient.
Implementation starts with the actual missing inspection seam, followed by the
owned frontier and production reconciliation composition:

1. **Shared immediate inspection:** source-once listing/facts, per-lane unions and
   policy, existing comparison/finalization, ordered lane-local errors. Keep old
   pair behavior and prove sharing with acquisition counts and fault injection.
2. **Coordinated partial frontier and continuation:** grouped sequential DFS,
   destination-specific membership/reduction, subset controls, owned validated
   parts and explicit runtime binding. Prove deep pause/resume with differing
   lane heads, not just serialization of completed directory results.
3. **Production reconciliation goals:** target-tagged pending work and acceptance,
   using sync's actual effects. Preserve unresolved goal and recursive ownership
   independently of physical classifications. Prove fresh-process traversal and
   decision continuation, returned-failure handling and truthful per-lane coverage.
   Do not add continuation of an opaque operation's internal partial progress.
4. **Bounded parallel observation and reporting:** one supplied executor with
   explicit within-group admission, joined returns and unchanged ordered results;
   grouped consumer surfaces and bounded N=1/N=2/several-destination evidence.

The first builder owns only stage 1. A fresh Sol xhigh reviewer takes over the
continuation assessment after the first analyst's 238.3k-token return; the stage-1
builder returned its assessment at 188.5k and will hand off at a coherent checkpoint
before exceeding the requested rollover range. Code writing and builds remain
sequential.

### Retry means an unfinished goal, not an old classification

Sync may partially create a source-only directory before returning failure. It is
then physically paired. Simply reclassifying it and applying a normal immediate
paired batch can omit the interior that the original recursive goal still owns.
If a whole-goal retry is supported, its reconciliation obligation must survive that
change. Likewise, replaying an old `Action` or `DirDiff` is not a correct general
retry algorithm. This is an implementation caution, not a requirement to implement
partial-copy recovery or track the operation's internal work.

The effects design must explicitly assign each expanded or opaque region one
recursive owner, preserve that ownership on failure, and distinguish accepted
comparison observations from settled reconciliation results. The exact supported
effect granularity will follow this model rather than assume that an ancestor must
be complete to permit saving.

The main crates already own their appropriate domains. New capabilities should
extend those boundaries, using the existing production effects where applicable.
An additional dependency or generic framework needs a concrete benefit to this
implementation rather than being a prerequisite inherited from exploratory work.

## Verification and final documentation

Checkpoint each coherent implementation stage, review interfaces before the next
dependent stage, and keep build/test execution sequential. Use meaningful focused
tests and the repository's nextest, clippy, doctest and formatting checks.
Record actual commands and limitations with the delivered result.

After implementation, reorganize the maintained architecture and reading paths
around the resulting capabilities. Preserve the numbered drafts as design history,
with clear links to the current account. Create the requested epic and readable
child tickets from actual delivered scope and identified remaining work.

## Implementation checkpoints

- `2c29671b` — shared immediate source inspection. Inspection now shares the
  physical source listing/facts and original physical error causes while retaining
  independent target unions, policy and ordered failures. Existing pairs keep
  inline fact ownership; shared lanes use immutable shared source facts. Reduced
  results release facts and carry comparison, coverage and paired children.
  Reported verification: 123 feature-disabled library/consumer tests, 172 default
  workspace tests, 175 all-feature workspace tests, 24 native-only inspection
  tests, 18 doctests, strict clippy/Rustdoc, formatting and document checks passed.
  The parent reviewed source acquisition, shared-error ownership and reduction.
  A successor seam will expose validated plans for action-decision lowering and
  already-admitted current-directory refresh, without replaying root policy.
- `397ca37b` — validated-plan lowering and already-admitted inspection. The
  generic `SharedRoots::inspect_with` hands each finalized plan to pure owned
  lowering; `inspect_admitted_with` skips already-accepted structural-root policy
  while retaining immediate-directory policy. `excluded_names()` exposes the
  complete early-preserved name set for decision signatures. Full plans may be
  retained by a runtime consumer but never contain acquisition handles. Reported
  checks passed: 27 default and 30 all-feature inspection tests, four doctests,
  strict clippy/Rustdoc, formatting and README checks. The inspection builder
  handed off and rolled over at 293.5k context.

### Selected continuation composition

The continuation review supports a comparison-owned partial frontier wrapped by
sync's current-group decision barrier. Sync accepts each destination's opaque
immediate application separately, then permits paired-child traversal. A snapshot
may therefore contain A's accepted application and B's unstarted one while both
root traversals remain incomplete.

For this slice, preserve exact selected action/entry decisions and validate fresh
facts before dispatching an unstarted application after restoration. Refresh only
unstarted lanes together, retaining source sharing. Do not replay accepted ancestor
policy or compare an ancestor's old whole-directory signature after its own writes;
validate its physical routing and retain its prior admitted-edge decision.

Whole-goal retry is not required. A failed opaque application can terminate its
lane and leave recovery to a fresh run; healthy lanes continue with their existing
heads. This avoids introducing an unnecessary retry interpreter or claiming that
the saved traversal can resume an operation's internal partial copy.
