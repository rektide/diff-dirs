---
type: ImplementationHandoff
title: Separate action deviation from intentional retention
description: Bounded correction to selection counters, opaque accounting and saved continuation before native reporting freezes.
resource: /design/architecture/reporting-handoff0.gpt6a.md
status: proposed-correction
generated: { by: "model:openai/gpt-6-astra#high", at: 2026-10-29 }
sources:
  - resource: "opencode://ses_e7951c1b6ffeyQeGeqQnqwkKil"
    title: Traces committed 0.5 accounting and specifies distinct deviation and retention counters, once-only aggregation and a clean continuation migration
  - resource: "opencode://ses_e7a2771c3ffez3sv1oPgD5gPH4"
    title: Parent commissions the bounded correction before JW JSON freezes and owns implementation dispatch and integration
  - resource: /design/architecture/execution0.gpt6a.md
    title: Accepted separation of observed comparison, deviation, selected work and actual effects
  - resource: /design/architecture/kernel-handoff0.gpt6a.md
    title: Accepted opaque ownership, provisional progress and failure boundaries
  - resource: /design/architecture/execution2.sol61x.md
    title: Verified contextual runtime checkpoint and outstanding grouped and listed owners
  - resource: /tree/inspect/src/selection.rs
    revision: c42fd4e1
    title: Derived action deviation, application need and structural-root baseline
  - resource: /tree/inspect/src/shared.rs
    revision: c42fd4e1
    title: Paired root gate elision and selected target-only structural container
  - resource: /tree/compare/src/model.rs
    revision: c42fd4e1
    title: Committed comparison-prefix reduction and cleanliness meaning
  - resource: /tree/compare/src/coordinated/state.rs
    revision: c42fd4e1
    title: Saved summary and exact partial-frontier validation
  - resource: /transfer/sync/src/opaque.rs
    revision: c42fd4e1
    title: Deviation counted as retention, recursive container credit and uncounted removability preflight
  - resource: /transfer/sync/src/lib.rs
    revision: c42fd4e1
    title: Ordinary serial and parallel walk/application subtraction
  - resource: /transfer/sync/src/model.rs
    revision: c42fd4e1
    title: Public report and failure counter promises
  - resource: /transfer/sync/src/coordinated/mod.rs
    revision: c42fd4e1
    title: Root acceptance, pending refresh, returned-call totals and lane report derivation
  - resource: /transfer/sync/src/coordinated/state.rs
    revision: c42fd4e1
    title: Recursive totals bounds and barrier validation
  - resource: /transfer/sync/tests/visitor.rs
    revision: c42fd4e1
    title: Special-preservation, mixed-batch and clear-container expectations
  - resource: /transfer/sync/src/coordinated/tests.rs
    revision: c42fd4e1
    title: Zero-planned opaque container currently counted twice and accepted structural-root preservation
  - resource: /examples/reconcile/src/transport.rs
    revision: c42fd4e1
    title: Explicit v2 summary and lane-total transport requiring migration
verification:
  method: Committed-source inspection at c42fd4e1 and explicit rkt Markdown/provenance checking
  scope: Architecture correction only; no production edits, runtime tests or inspection of unfinished grouped/cache changes
---

# Separate action deviation from intentional retention

**Confirmed defect. Keep both counters, with different derivations.** Equal
`Renew` is action deviation, not retention. Renaming sync's existing `retained`
to `deviated` wholesale would instead mislabel necessary retained containers.
The correction belongs to inspect's selected semantics, compare's reduction and
sync's reporting/continuation composition; physical kernel effects do not change.

## 1. Committed 0.5 trace

These findings are against `c42fd4e1`, not the active grouped/cache working tree.

| Path / branch | Current behavior and consequence |
| --- | --- |
| `selection.rs::from_parts` | `deviated += Some(action) != baseline`. Correct action-choice diagnostic; a missing baseline also contributes. |
| `opaque.rs::LaneCall::count` | Adds that deviation to `retained`, before direct dispatch. Equal renewal therefore reports retention on both success and later failure. |
| `opaque.rs::reconcile_directory` | Adds another retained unit when a successfully reconciled container is kept. Target-only explicit Keep is consequently counted twice: once as deviation, once as a container. |
| `lib.rs::{run,run_in_pool}` | Adds `observed.deviated - applied_retained` using saturating subtraction. This preserves nonyielded special-preservation counts but propagates the wrong meaning and obscures ownership mistakes. |
| `coordinated/mod.rs::{apply,reports}` | Saves `policy.retained - immediate.deviated` as `recursive_retained`; publishes `recursive_retained + observed.deviated`. The subtraction assumes the kernel reached counting even on failure. |
| Structural-root preservation | Compare commits a singleton selection. Sync accepts it without an application; its reported retention currently comes entirely from `observed.deviated`. Admitted ordinary paired-root gates disappear into their listings and must not gain a new root count. |
| Other no-effect selections | 0.5 marks ordinary groups Unstarted even when `ApplicationNeed::None`; only structural-root no-ops are accepted directly. The accepted design already permits the broader no-effect acceptance. |
| Removability preflight / private staging | Preflight is not counted; the subsequent dispatch is. Discarded private effects are cleared, while encountered policy counters survive. Preserve that distinction. |

The existing `zero_work_count_still_reconciles_opaque_kept_target_container` test
asserts retained **2 for one container**. Correct that expectation to 1; it is
evidence of duplicated categories, not a compatibility requirement.

## 2. Exact counter contract

**Counters describe selected boundaries, not a census of surviving files.** One
preserved tree contributes one boundary; descendants are not invented. An explicit
source-only preservation/KeepLeaf contributes even though no target exists: it
selects leaving that boundary unmaterialized. This preserves meaningful full-visitor
special/root behavior without claiming a target inode was retained.

### Static selection counts: inspect owns both

Keep `DirectoryCoverage.deviated`; add `DirectoryCoverage.retained`, derived in
`DirectorySelection::from_parts` through one shared predicate on the finalized
entry. Add the same `retained: u64` reduction to compare `Summary`.

| Final action / shape | Static retained contribution |
| --- | --- |
| `PreserveTree` | 1, including supported/unsupported, source-only and structural-root preservation. |
| `KeepLeaf` | 1 iff `baseline != Some(KeepLeaf)`. Ordinary assessed equal leaves contribute 0; explicit retention without an assessed baseline contributes 1. |
| `Directory { Keep, descend }` | 1 iff `!descend` **or** source is not a present directory. Thus paired routing contributes 0, stopped paired descent contributes 1, and target-only kept containers contribute 1. |
| All other actions | 0. Renewal, conditional creation, creation with omitted descent, replacement with omitted descent, removal and ConfirmAbsent are not retention of that entry. |

This is derived selected meaning, not whether a callback happened to call a setter.
No new policy flag or per-entry reason enum is needed. Unassessed paired routing
alone is not retention; its assessment gap remains `not_assessed`.

`deviated` remains exactly `Some(action) != baseline`, including baseline `None`.
It does not count changed contexts/grants when the physical action stays identical.
For example, Different+InstallLeaf with Renew may have deviation 0; Equal+Renew
has deviation 1. Neither has retention. An unused CreateIfNeeded has deviation,
not retention, and retains its existing conditional planned-work rules.

Excluded ordinary names contribute only to `excluded`; they have no selected
entry. The existing excluded structural-root sentinel is a real PreserveTree
selection and contributes one retained boundary as well as its exclusion. These
categories are not disjoint. Clear's target-only root Keep contributes retention 1
even though its structural baseline is already Keep and deviation is 0.

### Necessary containers: sync owns the additional count

Count one additional retained boundary when a selected `Container::Remove`
cannot be removed because its successfully processed interior keeps/excludes
something. Credit it at the existing successful reconciliation/cleanup decision,
before final permission restoration; a later restoration failure retains the
credit. Do not credit directories merely left standing by error/cancellation.

Remove the unconditional `if kept { retained += 1 }`: explicit Keep was already
counted statically. Propagate the retained-descendant fact to necessary ancestors,
one unit per ancestor, without collecting their paths. A normal paired containing
directory is not a retention diagnostic. A cancelled child is not evidence of
intentional retention.

Preview can derive the same necessity with a depth-sized `keeps_target` return
alongside its current `needed` return; no write or verification scan is required.
Removability preflight remains an uncounted validation pass, including on veto;
its original refusal path/cause explains the blocker. Its later dispatch/preview
pass counts selected batches once.

## 3. Owners, public fields and once-only aggregation

Use the following small surface change:

| Owner | Required change |
| --- | --- |
| Inspect `DirectoryCoverage` | Add derived `retained: u64`; preserve `deviated` and ApplicationNeed. SelectionParts needs no extra serialized field. |
| Compare `Summary` | Add `retained: u64` to every committed-prefix reduction, including nonpublished batches, structural-root no-ops and parallel ordered commit. Never include speculative work. |
| Sync `model::Policy` | Add `deviated: u64`; redefine `retained` by section 2. Propagate both through Report, Failure and merging. Keep `excluded` separate. |
| Sync `LaneTotals` | Replace `recursive_retained` with `opaque_deviated: u128` and `additional_retained: u128`. The latter includes opaque static retention **and** necessary-container credits, including a container whose own slot belongs to compare. |
| Sync `LaneReport` | Add total `deviated: u128`; derive total `retained` honestly. Preserve observed Summary, planned/completed totals and satisfied. |

In the private runner retain separate constant-size accumulators for (a) the
immediate input selection, (b) opaque descendant selections, and (c) necessary
container credits. Pass the two additional totals back privately alongside the
existing success/failure result. This is a returned-call accounting seam, not a
new public report framework or persisted application receipt.

```text
per-call Policy.deviated = immediate.deviated + opaque.deviated
per-call Policy.retained = immediate.retained + opaque.retained + necessary

LaneTotals.opaque_deviated += returned opaque.deviated
LaneTotals.additional_retained += returned (opaque.retained + necessary)

LaneReport.deviated = observed.deviated + LaneTotals.opaque_deviated
LaneReport.retained = observed.retained + LaneTotals.additional_retained
```

Ordinary `run` and `run_in_pool` use the same composition: committed walk Summary
plus returned additions. Replace both `applied_retained` subtraction paths, on
success **and** failure. Standalone initialization, root replacement, clear,
preview and per-record application have no enclosing committed comparison;
their Policy includes their own immediate selection. For the legacy per-record
subset API, retain whole-final-batch policy diagnostics while restricting effects
to requested slots; these counters are not a claim to count only modified names.

Count an immediate validated preparation when its operation accepts it for
application/preview; nested batches count when accepted by that recursive owner.
Refresh, pure route admission and repeat preflight do not create contributions.
The private additional totals must be available on every returned error, including
an early gate failure with zero additions. Eliminate `checked_sub(...).expect(...)`
as well as saturating subtraction: already-committed immediate selection meaning
does not depend on whether execution reached `LaneCall::count`.

Both selected counters survive discarded private staging; only provisional
completed effects are discarded. Started visible sibling effects and successful
old-node removal survive later publication/cleanup failure. Failed/unvisited
interiors contribute no guessed counts. `satisfied` stays false on failure and
continues to describe the declared operation, not post-effect equality.

## 4. Acceptance and saved continuation

Use derived `ApplicationNeed::None` to accept a freshly published sync selection
without kernel dispatch, including ordinary clean/retained-only batches. Record
its comparison/retention counts once through compare; increment no application,
planned-effect or completed-effect totals. Keep the group/window and paired child
edges so the caller retains prune/control semantics. Root preservation already
uses this pattern. Ordinary run may likewise bypass application for such a batch.

`planned == 0` is never this test: target-only Keep+descend still owns opaque work.
Its one retained container and recursively removed leaves must both appear.
An equal Renew is LocalOrOpaque and must execute despite comparison cleanliness.

Counts in `observed` describe committed choices, **including an unstarted B**.
They do not assert those choices executed. Save with A accepted/B pending keeps
A's returned additions/effects and both lanes' committed immediate counts. Restore
does not replay A or add those counts again. Refresh of B compares the entire
selection and contributes no new count; its returned execution adds only its
opaque/necessary extras. Refresh failure keeps the historical observed prefix,
zero new effect contributions and the existing terminal disposition.

**Make a deliberate v3 break:** compare Parts 3, SyncParts 3 and reconcile-example
outer transport 3. Reject v2 rather than defaulting new fields to zero or renaming
the ambiguous old totals. Closed history cannot recover true retention from
`deviated`/`recursive_retained`; current barrier entries are insufficient.

Validation and transport changes:

- Derive selection coverage from existing entries; never trust encoded counters.
  Add Summary.retained to checked reduction arithmetic and unexamined-zero checks.
  Bound it by the sum of comparison-state slot counts, computing that sum widely.
  Do **not** require `retained <= deviated`: the clear structural root disproves it.
- Bound both new LaneTotals fields by `applications * u64::MAX`, as with returned
  work totals. Zero applications implies zero application additions/effects;
  comparison retention can still be nonzero. Keep application counts per entered
  lane call for apply-many, not per cohort.
- Keep exact frame/context/edge/window validation. Newly produced no-effect
  applications are Accepted; reject Unstarted+ApplicationNeed::None in v3.
  Acceptance of an effectful selection still requires application; a no-effect
  accepted parent does not mean its future paired descendants are satisfied.
- Update `examples/reconcile/src/transport.rs` Observation and Totals mappings,
  version checks, comparison-only transport and process fixtures; update example
  JSON in `src/main.rs`. Derived LaneReport totals need not be persisted again.
  Saved aggregate bounds cannot reconstruct discarded history; do not introduce
  a per-file catalog to pretend they can.

JW should adopt the corrected provider revision/API and explicit native schema
change before freezing JSON. Emit `deviated` and `retained` as different fields;
do not preserve an alias with the former mixed meaning. Public pre-1.0 API changes
need the coordinated minor/pin/receiving-lock update under project policy. Parent
assigns this cross-owner correction around the active grouped worker, then passes
the resulting versioned surface to the native owner.

## 5. Necessary discriminators and handoff

Extend the existing small fixtures rather than creating an accounting harness:

| Discriminator | Required result |
| --- | --- |
| Unscoped equal Renew, then forced publication failure | Both have deviation 1, retention 0 and unchanged observed equality. Success has one install; failure has only actual completed progress and false satisfaction. |
| Ordinary clean and no-effect preserved structural root | Clean baseline has both counters 0. Root preservation has retention 1 with zero applications/effects; save/restore does not repeat it. |
| Target-only Keep+descend, one stale leaf | Immediate planned 0, ApplicationNeed LocalOrOpaque, retention **1**, one actual leaf removal. |
| Remove ancestor with a preserved descendant | One static retained descendant plus one necessary-container credit per required ancestor; explicit Keep ancestors are not counted twice. Failure/cancellation alone creates no retention credit. |
| Existing socket/FIFO fixtures, alone and beside copied work | Preserve 5/2 selected special boundaries as before, including source-only specials; mixed publication does not double them. Clear with one retained FIFO has FIFO + structural root = 2. |
| Opaque partial/private failure and accepted-A/pending-B fresh process | Additional counters and actual effects survive correctly; discarded private installations do not. Repeated reports/parts/refresh and restore are nonadding. Old v2 and new malformed bounds refuse. |
| Readiness refusal before dispatch | Returns the original failure, with zero opaque additions; aggregation neither panics nor loses already committed immediate counts. |

This corrects the reporting seam of the [accepted execution design](execution0.gpt6a.md#4-comparison-identity-and-truthful-result-states)
and the [0.5 runtime receipt](execution2.sol61x.md), preserving the
[kernel's provisional-progress boundary](kernel-handoff0.gpt6a.md#5-publication-progress-and-failures).
Parent owns the reciprocal entry in the [active design reading path](/design/index.md#start-here-the-rebuild-and-its-design-sequence)
and maintained reporting/Rustdoc promotion after implementation. Listed
ManifestCoverage remains its own desired/owned fulfillment proof; these counters
neither replace it nor certify post-effect observed equality.
