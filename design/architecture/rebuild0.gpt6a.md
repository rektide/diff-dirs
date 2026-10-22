---
type: ImplementationPlan
title: Coordinated comparison and resumable reconciliation rebuild
description: Recovered design sequence, current commission, implementation decisions and checkpoints for source-to-many comparison and usable continuation.
resource: /design/architecture/rebuild0.gpt6a.md
status: assessment-in-progress
generated: { by: "model:openai/gpt-6-astra#high", at: 2026-10-28 }
sources:
  - resource: "opencode://ses_e7eb31c84ffev4GUeYjdmZO7mY"
    title: Human commissions independently assessed coordinated comparison and resumability, sequential implementation with two Sol xhigh agents, and final design and ticket consolidation
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

## Implementation assessment

Two independent assessments are in progress: design history and continuation
scope; actual inspection/frontier seams and correctness obligations. Their
findings inform the concrete implementation sequence. Choices about saved-state
validation, effect granularity, control tokens, public identities and parallel
admission will be recorded here before the corresponding implementation.

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
