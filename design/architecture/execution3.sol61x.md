---
type: ImplementationAccount
title: Joined opaque execution, bounded source retention and truthful reporting
description: Production grouped apply-many, successful generation-scoped source observations, exact source-work evidence, and the separately reviewed V3 deviation-retention correction; handoff to declared execution.
resource: /design/architecture/execution3.sol61x.md
status: implemented-runtime-checkpoint
generated: { by: "model:openai/gpt-6.1-sol#xhigh", at: 2026-10-29 }
sources:
  - resource: "opencode://ses_e795502d3ffeuJ2kcDootC5FFb"
    title: Implements grouped opaque scheduling and bounded successful source retention, then separately corrects reporting and V3 continuation; verifies exact source work, independent failures and process restoration
  - resource: "opencode://ses_e7a2771c3ffez3sv1oPgD5gPH4"
    title: Parent commissions this provider slice, accepts the reporting correction with the absent-owned-marker qualification, and retains native adoption and maintained-document ownership
  - resource: /design/architecture/execution0.gpt6a.md
    title: Accepted source-once cohorts, bounded retention, recursive ownership and integration gates
  - resource: /design/architecture/kernel-handoff0.gpt6a.md
    title: Accepted private primitives, promoted child jobs, conditional chains and provisional progress
  - resource: /design/architecture/declared-handoff0.gpt6a.md
    title: Next inspect-owned declared preparation and sparse-versus-complete source-cache requirements
  - resource: /design/architecture/reporting-handoff0.gpt6a.md
    revision: 3664ad18
    title: Parent-accepted deviation, retention, necessary-container and V3 correction
  - resource: /design/architecture/execution2.sol61x.md
    revision: 08f9df12
    title: Delivered 0.5 contextual runtime predecessor and ownership boundary
  - resource: /tree/inspect/src/source.rs
    revision: 8d76f95e
    title: Actual source generation, successful cells, capacity charges, retention, diagnostics and concurrent acquisition tests
  - resource: /transfer/sync/src/opaque/cohort.rs
    revision: e3e6ef79
    title: Actual grouped scheduling around the shared entry and physical kernel
  - resource: /transfer/sync/tests/grouped.rs
    revision: 8d76f95e
    title: Exact counted source work, zero retention, mixed reuse, independent conditional chains, failures and caller-pool execution
verification:
  method: Workspace and feature-disabled nextest, real required reflinks, process restoration, strict Clippy and Rustdoc, doctests, formatting and explicit Markdown/provenance checking
  scope: Linux physical-union grouped effects, bounded observations and V3 accounting; not declared/listed execution, JW adoption, Windows runtime evidence or wall-time performance
extensions:
  code_checkpoints: [51046cc3, 4f0c07c0, e3e6ef79, 8d76f95e]
---

# Grouped execution and source retention checkpoint

**Actual opaque grouping and bounded source reuse are delivered.** The provider is
**0.6.0**, with coordinated exact internal pins and workspace/cursor/walk/reconcile
locks. The stable `Applier<P,M>`, `SyncRuntime<P,M>` and `SyncCursor<C>` ownership
shape remains; the reviewed reporting fields and continuation schema deliberately
change. No publish, push, native implementation or manifest runner occurred.

| Commit | Coherent result |
| --- | --- |
| `51046cc3` | Generation-scoped successful observations, bounded capacity, immutable original listing hints, demand masks and source diagnostics. |
| `4f0c07c0` | Actual grouped `apply_many`, ordered independent outcomes and promoted source-child scheduling before inspection. |
| `e3e6ef79` | Separate review correction: truthful deviation/retention, explicit returned additions, no-effect autoaccept, V3 transport and 0.6 pins. |
| `8d76f95e` | Concurrent source-cell coordination and caller-owned-pool cohort evidence. |

## Returned-call and recursive ownership

[`SyncCursor::apply_many`](/transfer/sync/src/coordinated/mod.rs) accepts a nonempty,
distinct subset of current **unstarted** applications. Token, membership,
disposition and counter bounds validate before filesystem work. Outcomes are in
frozen binding order, independently `Result<Report, Failure>` per selected lane.
Caller input ordering does not change that order.

Restoration still refreshes **all pending** saved decisions before any selected
resumed effect. A refresh failure returns the existing `Error::Refresh`, terminates
its actual bad lanes and leaves valid peers pending; that request performs no
effects. Once dispatch begins, a local route/kernel/opaque failure disposes only
that lane. Already-started work joins, scopes close, original causes/progress return
and healthy selected peers complete. There is no in-flight retry or saved opaque
interior. `apply` uses a one-member cohort; `drive` remains ordered apply-one, not
an implicit change to JW's fail-fast or application-budget policy.

The [cohort scheduler](/transfer/sync/src/opaque/cohort.rs) stages the existing
immediate entry kernel, releases the immediate preparation, then joins compatible
next promoted source children **before** their child inspection. Both absent
containers and private source-directory replacement preparations share an immediate
source listing/cell set. Lane contexts, logical coordinates, output/staging paths,
leases, stop flags, finish obligations and provisional totals stay separate.
Conditional ancestry is a lane-local linked depth stack; unused chains create
nothing. Its first selected ordinary work materializes it top-down, with postorder
finish and once-only creation credit even on failure.

The same `LaneCall::prepare_entry` and policy-free
[physical primitives](/transfer/sync/src/kernel.rs) serve the cohort and established
single-lane orchestration. Source-free removal/collapse uses the existing serial
recursive owner, including intact two-pass removability. Recursive reconciliation
remains serial; immediate independent leaf work may use the caller's pool. Private
expansion remains provisional until publication. A failed private preparation
does not erase visible siblings or an actually removed old node.

Compare still owns paired descent. A created/replaced directory never becomes a
paired edge. In a mixed A-opaque/B-paired region, the opaque call does not inspect
B's future target or decide B's future policy; B later makes its own inspection.
Different inspection purposes are not silently merged into one policy input.

## Source observations and capacity model

[`SourceSession`](/tree/inspect/src/source.rs) keys observations by its runtime
generation and relative containing directory. Default binding/restoration creates
a fresh generation and empty cache. An explicitly lent session represents the
caller's existing physical source phase; changed source phases require a new
session, not path-string equality. No generation/cache/epoch enters saved Parts.
Compatibility inspection with a distinct physical source override cannot populate
the bound generation's cache.

The batch distinguishes sparse successful positive cells from
`complete_names: Option<OriginalListing>`. Sparse cells cannot establish absence of
unqueried names or satisfy a physical union. Only actual successful enumeration
sets completeness. Original enumeration hints remain unchanged when a cell later
obtains metadata. Each lane sees only its declared facts, regardless of peer or
cache history.

Reusable cells contain successful metadata/link values only. Call-local acquisition
wrappers own failure outcomes and full-visitor projections; retained storage owns
neither executable errors nor compatibility `Facts` wrappers. Per-cell locks
coordinate acquisition; no whole-cache index lock spans filesystem I/O. Every
joined inspection closes enumeration handles. A later missing fact uses its freshly
admitted path; an already acquired attribute does not restat. Retained source
attributes cannot seed a new route certificate: actual epoch admission precedes
retained use, including names-only callbacks and child-directory entry.

Defaults are **32 retained batches / 8 MiB charged owned capacity**.
`SourceSession::with_cache_limits`, `cache_limits` and `set_cache_limits` provide
actual configuration. Zero in either dimension disables cross-call retention.
Limits can change while batches are active. Idle LRU entries evict without waiting
for pinned users; an oversized active batch serves its consumers then is not
retained. The index releases historical capacity rather than becoming a catalog.

The charge is explicit owned-capacity accounting, **not RSS or peak memory**:

- Batch/mutex and Arc headers; sparse cell-vector capacity, native key capacity;
  complete original-name/hint-vector capacity and each original native name.
- Successful cell/mutex/Arc storage, full physical path capacity, actual Metadata
  allocation and actual retained link `PathBuf` allocation/capacity.
- Cache coordinate capacity and a conservative hash-index envelope derived from
  its capacity, including load/control slack; the complete index is charged whenever
  idle retained entries exist.

Pinned active batches are reported separately and do not count as idle reusable
retention. Live directory width/depth, caller-retained preparations and fixed
session/counter/resource-buffer ownership are outside this cache budget. Thus the
limits bound retained reusable observations, not a wide active directory or total
process memory. All actual retained dynamic fact/name storage is charged.

### Honest diagnostics and exact evidence

`SourceSession::diagnostics()` exposes small snapshots split into paired, opaque
and reserved declared attribution. Counters describe enumeration attempts,
entry-metadata acquisition attempts, literal-link attempts, successful-fact reuse
and complete-listing cache hits/misses, plus retained charged capacity, active
batches and eviction count. Opaque attribution includes read-only opaque preview.
These are not raw syscall totals: a backend may supply already cached attributes.
Root binding, route checks, target observations, permission/output verification and
actual effects are not disguised as source-entry acquisitions. Epoch route checks
remain separately observable. Diagnostics have no scheduling or persisted meaning.

[Production discriminators](/transfer/sync/tests/grouped.rs) establish:

- Two missing Q lanes, and missing-Q plus private replacement-Q: with retention
  **zero**, a 40-branch tree with different nested preservation performs **82 opaque
  source listings and 122 entry-metadata acquisitions**, not one full traversal per
  receiver. Paired root work is one listing/one entry acquisition. Targets install
  40 versus 41 files. The replacement variant also runs in a named caller-owned
  two-thread pool. Both backends pass these exact counts.
- Mixed opaque/paired Q: the opaque owner lists four source levels. With 32 retained
  batches, later paired work has four retained hits and only its initial root
  listing. One-batch/zero retention produces five paired listings and no retained
  hits, with identical final data and ownership.
- An optional stronger source-link failure keeps its original source cause while
  a kind-only peer succeeds. Real cross-filesystem required-clone failure stops one
  target while its peer installs a real reflink; original read-only root modes and
  staging/lease closure are checked independently.
- Conditional materialization, atomic invalid subsets, A-accepted/B-pending,
  fresh restored generations, immutable hints/masks, error/fact release, sparse
  completeness, LRU/byte limits, zero retention and concurrent successful-cell
  coordination have direct tests in sync and inspection.

No wall-time speedup or whole-tree source-once promise across different owners is
claimed. Cross-owner reuse is deliberately bounded and opportunistic.

## Separate reporting correction and V3

The parent's accepted [reporting handoff](reporting-handoff0.gpt6a.md) is implemented
as its own commit, after preserving the grouped predecessor:

- Inspect derives both `DirectoryCoverage.deviated` and `.retained`; compare reduces
  both into Summary. Equal renewal is deviation, not retention. Kept target-only
  containers count once; required retained removal ancestors add one each.
- `Policy.deviated` is separate from `.retained`. The private runner separates
  immediate, opaque and necessary counts. `ApplicationResult` returns explicit
  additions even on early failure. Ordinary run and coordinated reports combine
  committed observations with those additions; neither checked nor saturating
  subtraction is used to guess duplication away.
- `LaneTotals` stores `opaque_deviated` and `additional_retained`; `LaneReport`
  publishes both total deviation and total retention. Preview derives necessary
  containers without writes. Selected counts survive discarded private work;
  provisional completed effects do not.
- The listed proof-only absent O structural marker is **retained 0**: Listed,
  NotRequested source, absent target, no desired/interior/grants and PreserveTree.
  Physical source-only preservation remains retained 1. This domain-aware predicate
  is tested now; ManifestCoverage remains a separate future fulfillment proof.
- `ApplicationNeed::None` freshly autoaccepts, including clean/retained-only ordinary
  batches, without application/effect totals. The window and paired edges remain.
  Consumers must use `pending()` rather than blindly applying every successful
  outcome. Zero-planned opaque Keep+descent still executes; clean equal renewal
  still executes.

Compare Parts, SyncParts and the reconcile outer transport are **version 3**.
Version 2 is rejected: its old mixed aggregates cannot recover real retention.
Bounds include retained slot counts and both returned additions; Unstarted plus
no application need is invalid. Fresh-process tests preserve A's actual counters
and effects while B remains pending, then verify A's totals do not grow on resume.

## Verification receipts

| Check | Result |
| --- | --- |
| Integrated workspace all-feature nextest | 275 passed; existing perf-runner build-script-output-path warnings only. |
| Inspect/compare/sync no-default-feature nextest | 178 passed. |
| Reconcile all-feature nextest | 20 passed, including actual fresh processes and lossless wide-counter transport. |
| Final added concurrent-cell / caller-pool discriminators | Two focused tests passed with all features and again with defaults disabled; no redundant whole-matrix rerun for these test-only additions. |
| `DIFF_DIRS_SYNC_REQUIRE_REFLINK=1` focused production tests | Three passed: real COW clone, joined clone failure/success, independent cohort clone failure. |
| Strict Clippy | Workspace all targets with all features and without defaults; reconcile all targets/all features passed. |
| Strict no-dependency Rustdoc / doctests | Passed; 15 workspace doctests compiled/ran. |
| Cursor/walk receiving examples | All-target/all-feature checks and refreshed locks passed. |
| Formatting / knowledge account | Workspace and reconcile formatting, explicit Markdown and OKF provenance checking. |

Initial red tests exposed intentionally changed no-effect application expectations,
a fault test reusing successful observations without a new source phase, and a
missing fixture parent; these were corrected before the receipts above. Existing
special-preservation, serial removal, conditional, permission, private-discard,
output-mtime, native-name and process regressions remain in the integrated suite.

## Exact next declared-execution interfaces

No declared runner or public metadata-injection constructor was added. The next
owner should implement the accepted [declared handoff](declared-handoff0.gpt6a.md):

1. Inspect owns `inspect_declared_group`, Exact declarations versus TargetCensus,
   direct no-follow lookup and immutable finalization over these same cells. Add
   actual lookup-cell construction with acquired-metadata provenance. A sparse
   positive lookup must not set `OriginalListing`; a complete listing can answer
   later exact names after fresh route admission. Missing/failed outcomes stay
   call-local. O-only/census work makes no source route or acquisition request.
2. Sync owns immutable desired/owned tries and ManifestIds, pure selection/guard
   adaptation and actual reference/coordinate/ownership binding before effects.
   Source cache identity remains physical generation/directory, never a manifest
   or policy decision. The accepted enclosing-desired-slice interpretation remains.
3. Schedule compatible desired source children together before inspection, retaining
   lane target/O/context differences. Reuse the current promoted jobs, private
   primitives, scopes and explicit returned accounting. Keep ancestor-owned collapse
   intact before subordinate retirement and stage only declared desired expansion.
4. Add independent ManifestCoverage and strict-create/RenewOwned APIs. The current
   physical runner still rejects Listed input at its effect gate. Extracting a
   root-only EffectBinding from Applier remains available to that owner; this slice
   did not broaden the architecture speculatively.

Parent retains maintained-guide/index promotion, tickets, external JW pins/native
V3 adoption and later manifest commissioning. The existing `.beads/issues.jsonl`
changes were not edited or committed by this session.
