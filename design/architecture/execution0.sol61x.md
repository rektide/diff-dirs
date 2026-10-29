---
type: ImplementationAccount
title: Shared selections and contextual inspection foundation
description: First provider checkpoint for the annealing delivery, including the implemented selection/acquisition engine, faithful full-visitor adapter, legality gates, version migration and precise continuation handoff.
resource: /design/architecture/execution0.sol61x.md
status: implemented-checkpoint
generated: { by: "model:openai/gpt-6.1-sol#xhigh", at: 2026-10-29 }
sources:
  - resource: "opencode://ses_e7a064cf8ffeJhoqfCnySR4Q6l"
    title: Implements and verifies the shared immutable selection, contextual immediate inspector, finite acquisition, full-visitor compatibility, compact-name/resource foundations and v2 unit-context migrations
  - resource: "opencode://ses_e7a2771c3ffez3sv1oPgD5gPH4"
    title: Parent commissions this provider foundation slice and accepts the five execution decisions; owns maintained documentation, integration and tickets
  - resource: /design/architecture/execution0.gpt6a.md
    revision: 1132c5c1
    title: Full provider design, including declared versus physical input, complete desired obligations, shared body ownership and the implementation checkpoints
  - resource: /design/architecture/delivery0.gpt6ax.md
    title: Joined commission and provider/native architectural review boundaries
  - resource: /tree/inspect/README.md
    title: Predecessor immediate-inspection and already-admitted compatibility contract
  - resource: /tree/observe/README.md
    title: Predecessor physical-observation, original error and backend ownership contract
  - resource: /doc/architecture/README.md#components-and-ownership
    title: Maintained component boundaries; parent will integrate this checkpoint's delivered changes
  - resource: /design/observation/facts0.gpt6am.md#3-smallest-demandacquisition-contract
    title: Finite acquisition stages, actual Metadata retention and compatibility precedence
  - resource: /design/observation/names0.gpt61sxh.md#2-where-current-ownership-pays
    title: Native handle/name duplication and child-promotion lifetime rationale, not performance evidence for this implementation
  - resource: /tree/inspect/src/selection.rs
    revision: 48d50af0
    title: Sole metadata-free selected-work model, compact selected names and common semantic validator
  - resource: /tree/inspect/src/shared.rs
    revision: 48d50af0
    title: Production contextual/full-profile engine and joined source acquisition
verification:
  method: Focused owner and existing consumer nextest suites, default-feature checks, strict workspace/example Clippy, formatting, Rustdoc and explicit Markdown checking
  scope: Linux immediate inspection and legacy traversal/effect/transport compatibility; not completed contextual traversal, grouped opaque execution, manifest execution, native adoption or performance
  result: Implementation receipts and their checkpoint boundaries are recorded below; no benchmark or cross-platform runtime claim
extensions:
  parent: "opencode://ses_e7a2771c3ffez3sv1oPgD5gPH4"
  code_checkpoints: [fb4719cc, 48d50af0]
---

# First provider foundation checkpoint

**One immutable selection body and one inspection engine are implemented.**
The default full-visitor path now uses that engine, not a parallel inspector or
a second saved action table. New policies can inspect real immediate batches
with typed context, finite sparse demand, explicit comparison and execution-input
preparation.

Code checkpoints:

- `fb4719cc` — contextual inspection, shared selections, acquisition cells,
  native-name/resource foundations and coherent consumer/version migrations.
- `48d50af0` — separate context-schema meaning and pre-I/O legacy binding refusal.

This is **not** the completed contextual provider or a native renewal delivery.
The [next slice](#next-coherent-slice) must propagate these contracts through
traversal and sync before JW can adopt its contextual ordinary policy. Actual
grouped opaque sharing, bounded cross-call source retention and desired/owned
manifest execution remain commissioned subsequent work.

## Implemented ownership and stable seams

| Owner | Implemented seam |
| --- | --- |
| [Selection](/tree/inspect/src/selection.rs) | `DirectorySelection<C>`, owned `SelectionParts<C>`, entries, authorities, domain/input references, derived coverage and one semantic validator. |
| [Policy](/tree/inspect/src/policy.rs) | `Policy`, owned `Context`, separate policy/context-schema identities, pure context validation, shape/final choices and borrowed coordinates. |
| [Comparison](/tree/inspect/src/comparison.rs) | Explicit outcomes, comparator identity, finite demand, structural wrapper, working `StatKey` and `SizeAndLink`. |
| [Acquisition](/tree/inspect/src/acquire.rs) | Private per-present-entry `AcquisitionCell`, independently cached metadata/link outcomes, actual shared Metadata and demand-masked views. |
| [Preparation](/tree/inspect/src/prepared.rs) | Runtime facts/binding separate from the same `Arc<DirectorySelection<C>>`; no public constructor pairing a decoded selection with historical facts. |
| [Engine](/tree/inspect/src/shared.rs) | `SourceSession`, `DirectoryRequest`, `TargetInspection`, `inspect_group`, two explicit acquisition schedules and selected-execution fact preparation. |
| [Physical names](/tree/observe/src/names.rs), [listing](/tree/observe/src/listing.rs) | Packed Unix native-name tables; native handles use a shared parent plus `NameId`; `Entry::name()` borrows without cloning. |
| [Resources](/tree/observe/src/resources.rs) | Caller-owned, live-reconfigurable enumeration pool and `read_directory_with`; raw buffers do not become semantic state. |

`EntrySelection<C>` contains semantic values but no name/path allocation. A
selection owns its containing coordinate, compact admitted-name table and entry
array. `EntrySelectionRef` borrows the name and semantic body. `to_parts()` makes
owned transport values deliberately; ordinary barrier/group sharing does not call
it or clone the full body.

The [sync decision module](/transfer/sync/src/coordinated/decision.rs) is now only
a compatibility alias and composition restrictions. `DirectoryDecision` aliases
`Arc<DirectorySelection<()>>`; it is not a second structure or semantic validator.
The live plan, returned group and saved barrier share that exact Arc. Decoded
values compare semantically, independently of Arc identity.

`DirectoryPlan` is the narrow full-visitor adapter around `PreparedDirectory<()>`.
Its read-only `PreparedOperation` views borrow actions from the selected body and
facts from runtime storage. Mutable legacy `Operations` exist only during the
callback, then are discarded. Legacy relative paths are materialized in that
adapter, rather than retained in each selected entry.

## Legality and coverage review table

Every row also permits explicit whole-tree preservation unless a declared desired
obligation requires installation/descent. Unsupported physical kinds permit only
preservation, without unused grants or interior context. Physical structural roots
must be supported; a selected target-only directory root is canonically a retained
container, preserving predecessor reporting.

| Observed physical shape | Additional legal work | Required interior when recursive |
| --- | --- | --- |
| Source leaf, target absent | Keep leaf or install source leaf. | None. |
| Source absent, target leaf | Keep or remove target leaf. | None; removal requires true absence plus `SourceAbsent`. |
| Two supported leaves | Keep or install; unlike kinds cannot compare equal. | None. |
| Source directory, target absent | Eager create with/without descent; conditional create only with descent. | `SourceOnly` when descending. |
| Source absent, target directory | Keep with/without descent; remove only with descent and `SourceAbsent`. | `TargetOnly`. |
| Two directories | Keep with/without descent; assessed local comparison is Equal. | `Paired`. |
| Source leaf, target directory | Keep target container, or coupled leaf replacement. | `TargetOnly` for kept-container descent; `RemoveForReplacement` for replacement. |
| Source directory, target leaf | Keep target leaf, or coupled directory preparation/replacement. | `PrepareReplacement`, including an explicitly empty preparation. |

All installation actions require `InstallAuthority`. `ReconcileDifference`
requires Different; Renew permits other assessment outcomes but never an
unsupported physical kind. Destination grants cover actual absence, supported
leaves and trees in increasing order. Equal renewal therefore needs no fabricated
diff. Coupled installation remains distinct from an independent delete half.

| Declared/listed rule | Common validation now | Executor bind still required |
| --- | --- | --- |
| Desired leaf, including both-missing physical sides | Requires a present supported/exact-kind donor and selects installation, not keep/preserve. | Directly look up every desired obligation; check executable expectations and complete installed coverage. |
| Desired routing directory / expansion | Requires actual source-directory shape, descent and `InteriorInput::Listed` with matching desired ID. | Bind subtree IDs and stage only the declared desired slice, never unlisted donor children. |
| Obsolete owned leaf | Uses NotRequested source plus an exact owned grant; does not fabricate source absence. | Prove exact membership and actual target shape against the supplied immutable O input. |
| Owned obsolete leaf found absent | `ConfirmAbsent` is legal only in Listed, with absent target, NotRequested source, no desired slot and exact owned-leaf authority. | Prove no desired leaf exists at/below that coordinate and establish no-follow absence. |
| Directory-to-leaf collapse | Requires a tree grant, `CoupledOwned`, proper interior role and listed input. Scoped coupled directory removal can carry that authority internally. | Bind the ownership prefix and preflight the entire actual subtree before subordinate dismantling; locals/unknown directories veto intact. |
| O-only structural directory | Keep/routing is representable; exact leaf authority cannot authorize recursive removal. | Retain structural directories; do not add an obsolete-directory prune pass. |

Generic structural validation checks references against the selection's declared
domain IDs. It **cannot** certify an external manifest's contents, O authority,
CreateAbsent/RenewOwned mode or native state. Those checks belong to the future
listed executor's actual prepared-input bind. No listed selection is currently
accepted by coordinated compare/sync, and no listed effect API is claimed.

The common model includes every commissioned distinction from the start:
PhysicalUnion versus Listed, Present/Absent/NotRequested source observation,
installation/removal authority, desired expectation/reference, interior role,
interior input and context. A role named TargetOnly is not source-absence evidence.

Coverage is derived, not accepted as caller-supplied counters:

- `equal`, `different`, `not_assessed`, `unsupported`, `unresolved` count admitted
  assessments; excluded names stay excluded.
- `planned` counts immediate namespace actions; `deviated` compares action with
  the canonical baseline. Stopped paired descent has its own count.
- `ApplicationNeed::LocalOrOpaque` includes sync-owned recursive interiors even
  with zero planned work. Pure paired descent alone needs no immediate application;
  listed routing can own desired descendants independently of direct effects.
- Immediate comparison cleanliness requires unscoped complete coverage, no gaps,
  no stopped paired descent and no differences. An unscoped equal renewal is clean
  **and** needs application. The legacy recursive Summary has not yet migrated to
  this new completeness definition.

## Context, acquisition and compatibility behavior

`inspect_group` is usable for one real directory coordinate. A caller supplies
admitted physical routes, logical roots, per-lane context/policy/comparator, an
execution-demand provider and a returned-call epoch. Inspection does not itself
grant an external effect binding or traverse children.

The staged schedule is:

```text
enumerate → all ready lanes' names callbacks → resolve kinds → shape choices
→ policy/comparator facts → compare → final choices → semantic/context validation
→ missing selected execution inputs → publish prepared lane
```

`RootGated` derives the root's interior context before assessing its listing. The
returned listing stores that derived input. `AlreadyAdmitted` skips structural
policy replay and uses the exact supplied input. Final interiors carry owned
contexts, not lowering payloads or raw facts. Context validation is pure; no
ancestor rescanning/native calls are hidden in it.

Unknown-kind resolution retains the entire actual Metadata. Later attributes
demand, including execution preparation, reuses it. Native known kinds need no
metadata merely to establish shape. Metadata and literal-link outcomes are cached
independently in a cell; errors retain their original shared cause. A link failure
does not poison a kind-only peer. A known kind remains usable after a peer's
optional attributes demand fails. A failed metadata acquisition needed for kind
does affect all lanes needing that kind.

Policy and comparator views are masked by their own declared demands. Neither
opportunistic peer facts nor kind-resolution attributes become silently visible
to a policy. Acquisition accessors perform no I/O. Per-lane demands share source
cells without a cache lock over unrelated entries; indexed parallel outcomes are
reduced in native-name/source-first order, on the caller's pool.

FullVisitor uses the **same** cells, structural wrapper, comparator, selected body
and validator, but its explicit LegacyFull schedule ranks each entire name as
source metadata/readlink → target metadata/readlink → default comparison before
the next name's outcome. This preserves an early source-readlink or comparison
failure ahead of a later metadata/kind failure. Root full observations remain
mandatory before early steering. Source links, Both/Audit target links, full
metadata, root suppression and overrideable late decisions retain their earlier
meaning. [Sync adapter order](/transfer/sync/src/visitor.rs) remains check/absence
first and caller late visitor last; Seed/Update/Prune were not moved to terminal
name/shape preservation.

`ExecutionDemand` is a pure finite table. The engine acquires all missing selected
inputs for the immediate lane before publication. Sync currently still reaches it
through the full adapter, so its existing kernel receives full facts; the next
contextual kernel must supply the closed sparse physical action-demand table.

## Resource and runtime boundary

Observation resources are implemented, not just declarations: default native read
size 64 KiB, lazy maximum 16 idle buffers and 1 MiB retained raw capacity, finite
EINVAL growth, no waiting for an idle slot, coherent reconfiguration and return
under current limits. Backend activation is exposed; an explicit native-backend
requirement refuses when unavailable. Packed names outlive raw-buffer leases.

`SourceSession` establishes a runtime generation and shares these resources through
new immediate calls. Source cells currently live for **one joined call**. The
`SourceCacheLimits` default is defined as 32 batches / 8 MiB charged owned capacity,
but cross-call cache execution is deliberately not implemented or configurable
through an ignored setter. There is no unbounded retained source catalog.

`InspectionEpoch` establishes the opaque returned-call interface but currently
stores no route certificates. Existing compare/sync route validation remains in
force, including its repeated checks. The next owner must implement private route
reuse and target invalidation, not treat this placeholder as completed admission.

Legacy standalone/recursive convenience operations do not yet share one outer
resource owner throughout their call graph. Native handle/name storage and final
selection names are compact; temporary legacy names/paths and acquisition-cell
physical paths still allocate. No allocation/syscall/timing improvement is claimed.

## Versions and consumer migration

The six provider product packages are `0.3.0`, with exact internal pins. Workspace
and all three excluded-example receiving locks are updated. No publish/push was
performed and JW production was not edited.

Comparison parts, sync parts and the reconcile example outer codec now reject v1
and use version 2. The example explicitly encodes outcome/code, source observation,
installation/removal semantics, interior role/input and separate policy/context-
schema/comparator descriptors. It reconstructs through the common validator.
It is currently the **unit-context physical-union adapter**, not the completed
generic contextual v2 transport. Finish that schema before calling it a delivered
contextual checkpoint format; do not invent a silent v1 migration.

Public external `apply_plan` now compares entire selections after fresh inspection,
not only its old small diff/action signature. The unit compatibility composition
also refuses a changed context-schema or comparator descriptor without filesystem
access, even when actions would remain identical.

## Verification receipts

| Check | Observed result |
| --- | --- |
| `cargo nextest run -p diff-dirs-tree-inspect -p diff-dirs-tree-observe --all-features` | 49 passed at the principal code checkpoint. |
| Same owner packages with `--no-default-features` | 44 passed. |
| `cargo nextest run -p diff-dirs-transfer-sync -p diff-dirs-tree-compare --all-features --no-fail-fast` | 121 passed, including conditional ancestry, zero-planned opaque work, intact replacement refusal, original timestamp/CoW behavior and explicit-subset refresh. |
| `cargo nextest run --manifest-path examples/reconcile/Cargo.toml --all-features` | 15 passed, including fresh processes, native names, failed-peer disposition and transport limits. |
| Strict workspace all-target/all-feature Clippy | Passed, including after the separate schema-identity follow-up. |
| Strict reconcile example all-target/all-feature Clippy | Passed at the principal checkpoint. |
| Workspace `check --no-default-features`; all-feature walk/cursor example checks | Passed; receiving locks refreshed. |
| Workspace no-dependency all-feature Rustdoc; workspace/example formatting | Passed. |
| Descriptor follow-up checks | Legacy bind/corruption regression and two focused fresh-process/malformed-transport cases passed; this was not another full-matrix rerun. |

[Selection tests](/tree/inspect/src/selection_tests.rs) exercise the finite physical
action/authority table, equal renewal, coverage gaps, zero-planned opaque ownership,
decoded corruption and listed obligation/input legality. [Engine tests](/tree/inspect/src/shared_tests.rs)
exercise actual contextual inspection, root-derived context, independently meaningful
comparators, sparse masking, kind→attributes reuse, execution-input publication,
lane-local stronger-demand faults, full-profile error precedence and fact release.
The sync test `plan_group_and_barrier_share_one_immutable_selection_body` verifies
actual Arc identity, not merely equal cloned decisions.

These checks establish the stated foundation and predecessor compatibility, not
N1/N2 command adoption, grouped opaque source-once behavior, complete manifest
coverage, bounded-cache measurements or a performance win.

## Next coherent slice

1. **Contextual paired and sync propagation.** Migrate compare's bindings,
   memberships, partial frames and input/child context validation to generic C;
   bind policy/context-schema/comparator meaning before I/O. Migrate the Applier /
   SyncRuntime surface deliberately to P/M, with FullVisitor convenience adapters.
   Keep the new selected body and common validator, not a fresh decision type.
2. **Close the execution loop.** Feed `PreparedDirectory<C>` directly to the shared
   effect kernel and supply its physical `ExecutionDemand`. Pass selected context
   through every opaque copy/removal, conditional ancestry, preview, coupled
   removability preflight and subsequent removal. The current kernel is still unit-
   context/full-fact; the immediate context test is not that whole-path proof.
3. **Publish work independently of diff.** [Pair lowering](/tree/compare/src/pair.rs)
   still drops clean plans; PlanWalk/frontier publication still needs the new
   `ApplicationNeed` discriminator and clean-work seals. Plain Walk remains a
   divergence projection. Migrate [Summary](/tree/compare/src/model.rs) to explicit
   comparison coverage; `retained == 0` must not be the universal completeness test.
4. **Complete generic v2 and process proof.** The existing unit-context v2 codec is
   coherent and rejects v1, but contextual frame/member equality and an A-accepted /
   B-unstarted contextual fresh-process test remain. Accepted roots/ancestors must
   not replay on refresh. Include deep independent heads and mismatched schema,
   comparator and residual inputs.
5. **Runtime sharing.** Thread one source session/resource owner through paired and
   opaque paths; implement RouteEpoch reuse inside a returned call. Add actual
   grouped opaque `apply_many`, then bounded charged-capacity source retention.
   Retained cells must close listing handles at call return, keep successfully
   acquired metadata/link facts, drop cross-call executable errors and support fresh
   route-admitted path acquisition for previously unrequested facts. Cache residency
   must never become recursion ownership or policy authority.
6. **Desired/owned program.** Implement prepared S/O indexes and BLAKE3 identities,
   direct desired lookup, guard demands, complete coverage, strict versus owned
   placement, listed-only expansion and ancestor-owned collapse. Bind all reference
   and expectation relations against actual prepared inputs before effects. Preserve
   unowned locals and O-only containers; settle absent obsolete leaves honestly.

The [parent delivery account](/design/architecture/delivery0.gpt6ax.md) owns review,
maintained guide/index integration and native coordination. The predecessor owner
guides still say pooling/contextual behavior is unimplemented; they need promotion
of the verified subset above, while preserving these explicit pending boundaries.
