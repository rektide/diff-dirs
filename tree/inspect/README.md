---
type: CrateGuide
title: Immediate tree inspection
description: Ownership of immediate name unions, acquisition demands, visitor policy and validated plans above physical observation.
resource: /tree/inspect/README.md
status: implemented
generated: { by: "model:openai/gpt-6.1-sol#xhigh", at: 2026-10-22 }
updated: { by: "model:openai/gpt-6-astra#xhigh", at: 2026-10-29 }
verification:
  scope: Immediate source-once acquisition and pair/sync regressions; not coordinated traversal, resumable execution or performance
  method: Counted and faulted real-filesystem acquisition tests, pure-default owner and Linux nightly workspace default/all-features nextest, strict clippy, formatting, doctests and Rustdoc
  result: Baseline 2c29671b passed 123 pure-default owner/consumer tests, 172 default and 175 all-feature workspace tests and 18 doctests; the lowering/admission successor passed 27 default and 30 all-feature inspection tests
sources:
  - resource: "opencode://ses_e7a2771c3ffez3sv1oPgD5gPH4"
    title: Reviews and promotes the shared selection, contextual inspection and finite acquisition foundation while preserving traversal/executor handoff boundaries
  - resource: /design/architecture/execution0.sol61x.md
    title: Foundation code checkpoints, compatibility and acquisition checks, and exact pending integration
  - resource: /tree/inspect/src/selection.rs
    title: Shared selected semantics, domain-specific authority and common validation
  - resource: /tree/inspect/src/policy.rs
    title: Contextual policy stages and faithful full-visitor adapter
  - resource: "opencode://ses_e7eb22fb2ffeH5O20EZV5nW4B5"
    title: Implemented and verified source-once inspection, original shared error causes, consumer-selected full-plan lowering and already-admitted inspection without root-policy replay
  - resource: "opencode://ses_e9d314c29ffeboK450qrHM3qhF"
    title: Updated facade navigation after the final workspace relocation
  - resource: "opencode://ses_e9d737832ffeiy0f6OibOWOKSp"
    title: Extracted immediate inspection, preserved private plan construction and added boundary verification
  - resource: "opencode://ses_e9d3bf383ffeIFJ9ghJT4v2a4w"
    title: Separated physical observation from inspection demands, preserving error causes and early-admission coverage
  - resource: /design/architecture/draft2.gpt6a.md#2-separate-immediate-inspection-from-traversal
    title: Accepted extraction scope and encapsulation requirements
  - resource: /design/architecture/draft2.gpt6a.md#3-extract-physical-observation
    title: Physical acquisition extraction below inspection demand and policy orchestration
  - resource: /README.md#visitor-steered-planning
    title: Governing visitor, fact acquisition and validation contract
---

# Immediate tree inspection

`diff-dirs-tree-inspect` owns immediate native-name unions, staged acquisition,
contextual policy and full-fact visitor adapters, comparison predicates and common
selected-work validation. Immutable `DirectorySelection<C>` is separate from
runtime `PreparedDirectory<C>` facts; `DirectoryPlan` is the full-visitor adapter.
It performs no recursive traversal or filesystem mutation.
`DirDiff`, immediate counts, checks, scopes and errors have this same owner.

`PairedRoots` validates real directory roots source-first and reuses their mandatory
facts for root steering. `bind_scoped` also returns the checked compatibility
visitor, keeping the root-check bypass private. `DirectoryPlan::paired_children`
provides a read-only, finalized, paired-only child projection; it does not expose
constructors or mutable plan fields. One-sided recursive consumers use the actual
operations under their own traversal rules.

[Observation](../observe/README.md) owns listing entries, early physical hints,
platform backends and explicit no-follow acquisition. Inspection decides which
entries and link texts to acquire, orders failures and maps physical errors while
retaining the original I/O cause. Physical hint types are re-exported unchanged;
backend handles never enter visitor payloads or finalized plans. `native-dirent`
forwards to observation and remains opt-in (Linux needs nightly); `parallel`
orchestrates admitted acquisition on the caller's Rayon pool. Observation owns
caller-supplied buffer resources; comparison owns coordinated traversal.

## Contextual inspection and shared selections

`inspect_group` takes one `SourceSession`, a `DirectoryRequest`, ordered
`TargetInspection` lanes, a pure `ExecutionDemand` table and an `InspectionEpoch`.
Each lane supplies its typed policy context and comparator. The engine runs:

```text
names admission → kind resolution → shape choices → requested facts
→ comparison → final selection → semantic/context validation
→ missing selected-execution facts → immutable prepared batch
```

`Policy` has independent policy and context-schema identities. `StatKey` preserves
size/exact-mtime and literal-link equality; `SizeAndLink` deliberately ignores
regular-file mtime. `Comparison` distinguishes Equal, Different, NotAssessed,
Unsupported and Unresolved. Explicit Renew authority can select an equal leaf
without changing its comparison. `DirectoryCoverage` derives comparison gaps,
action deviation and `ApplicationNeed` separately; zero immediate planned work
does not suppress an opaque recursive obligation.

`DirectorySelection<C>` owns metadata-free semantics and compact names. Live plans,
coordinated groups and application barriers share its immutable body. Decoded
`SelectionParts<C>` use the same validator as live finalization; decoding supplies
no executable facts. PhysicalUnion and Listed domains distinguish actual absence
from unrequested source observation. Listed references are represented and checked
structurally here; their prepared-input binding and executor are subsequent work.

Acquisition cells retain metadata obtained while resolving Unknown kind. Later
attribute/execution demands reuse it. Policy and comparator views expose only their
declared facts, even when a peer requested more. A source link/attribute failure
affects only lanes that require that fact. `FullVisitor` uses this engine with its
legacy full-observation schedule and early/late precedence, including per-name
source-before-target error ordering.

Root gating may derive a context for the root's contents. AlreadyAdmitted calls
reuse their exact supplied input without root-policy replay. The
[foundation account](/design/architecture/execution0.sol61x.md) records tested
immediate behavior. The [next checkpoint](/design/architecture/execution1.sol61x.md)
delivers generic contextual frontiers and fresh-process comparison transport;
the [0.5 runtime](/design/architecture/execution2.sol61x.md) now executes contextual
opaque effects and resumed applications. Source cells currently share
within one joined call; bounded cross-call retention remains pending. Route epochs
now provide actual no-follow admission and invalidation within a returned call,
with no lasting cross-call route authority.

## Shared immediate inspection

`SharedRoots::new(source, targets)` validates one source root and ordered real
target directory roots. `inspect_visitor(dir, targets, execution)` accepts a
binding-ordered subset of `TargetVisitor { target, visitor }` requests. Each
request has its own full early/late visitor; target indices retain their original
binding meaning. Equal or nested read-only roots are allowed, not coalesced.

One complete source listing supplies every lane's own name union. After all
ready lanes' early admission, the epoch acquires each demanded source entry's
metadata/link text once. Lanes preserve their own target facts, ordered first
error, late steering and whole-batch validation. Shared source failures use
`Arc<Error>` to retain the same original I/O cause, including its raw OS code.
Listing failure is never absence; a lane that preserves an entry does not demand
its source facts. Root preservation uses mandatory binding facts without listing.

By default, each `TargetComparison` contains either its tagged failure or a reduced
`DirectoryComparison`: baseline diff, `DirectoryCoverage`, paired child names and
a structural-root flag. `DirectoryPlan::into_comparison()` offers the same
fact-releasing lowering for ordinary immediate plans. Reduced results carry no
full metadata, acquisition handles or mutation authority.

`inspect_with` uses that same epoch but passes each successful validated
`DirectoryPlan` to a caller-selected lowering function. Its generic
`TargetComparison<T>` preserves consumer-owned decisions rather than forcing them
through the diff projection. The callback can capture names, presence/comparison,
actions/baselines, `excluded_names()` and descent into owned semantic signatures,
or deliberately retain a full plan. It is pure lowering, not policy acceptance or
effect execution. Failed lanes never call it; callback order is unspecified.

`inspect_admitted_with` and its reduced `inspect_admitted_visitor` convenience
explicitly require caller-accepted root/ancestor admission. They skip structural
root hooks even at the empty relative path, while still running normal directory
hooks. Validated root bindings remain mandatory. A resumed consumer is responsible
for compatible accepted decisions and structural route validation, without
replaying accepted root policy just to refresh an unstarted directory batch.

All started work joins before return. Root binding facts remain with the runtime;
child facts and handles do not remain in the reduced result. A caller choosing
full plans retains their immutable facts until lowering/dropping them; no listing
handles enter plans. Parallel execution uses the current caller's Rayon
pool for wide metadata phases, not a new pool or J/K-controlled directory frontier.
Complete unions and output can still grow with width and target count: no byte or
total-memory bound is claimed.

This is one immediate assessment, **not completed traversal or reconciliation**.
Roots and ancestors may remain partial. The caller still owns every destination
head, child/return obligation, summary reduction and any effect acknowledgment.
No coordinated walker, saved continuation, resumable sync or effect authority is
introduced by this seam.

[Comparison](../compare/README.md) owns recursive audit and paired walking. The
[`diff-dirs-tree-api` facade](../api/README.md) re-exports the same public types. Owner unit
tests source-include only the dependency-free fixture in
[`tests/support/fixture.rs`](../../tests/support/fixture.rs); no test dependency
points back to comparison or the facade.
