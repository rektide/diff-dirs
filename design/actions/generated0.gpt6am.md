---
type: ProtocolProbe
title: Generated destinations, scoped discovery and pending bodies
description: Eight bounded Rust probes distinguish generated desired input from physical facts and establish a narrow claims/publication/dispatcher handoff.
resource: /design/actions/generated0.gpt6am.md
status: proposed-and-bounded-model-tested
generated: { by: "model:openai/gpt-6-astra#medium", at: 2026-10-20 }
sources:
  - resource: "opencode://ses_ea7be8196ffeqAPpgAKHuKBI1o"
    title: Independently modeled generated ownership and virtual children, ran eight Rust probes, and specified the narrow dispatch handoff
  - resource: /design/roadmap/brief0.gpt6a.md
    title: Independent commission and good-source/failure constraints
  - resource: /doc/architecture/README.md
    revision: 1193a1ee
    title: Completed proposed component baseline, particularly physical/generated/mapped inputs
  - resource: /design/execution/contracts0.gpt61sxh.md
    revision: 5d6c003a
    title: Completed synthesis, section 5B generated-input commission and typed finish contract
  - resource: /design/actions/protocol0.gpt6am.md
    revision: 134e0c35
    title: Typed production, claims, clean-work publication and satisfaction alternatives
  - resource: /design/observation/facts0.gpt6am.md
    revision: 5423a71f
    title: Explicit generated satisfaction demands and full physical-fact compatibility
  - resource: /src/visitor.rs
    revision: 5d6c003a
    title: Checked sealed physical facts, InstallLeaf legality and public immediate inspector
  - resource: /src/pair.rs
    revision: 5d6c003a
    title: Checked physical clean-plan release
extensions:
  ticket: diffdir-tree-actions-generated
  reading_boundary: Named independent brief, scoped ticket, completed baseline/synthesis/protocol/facts and targeted core source only; withheld statement, full roadmap waves, parent transcript and concurrent builder/modes/native-graph/name/codec drafts were not read, searched, cited or evaluated; no subagents
verification:
  method: Ignored Rust model with eight nextest tests, real immediate physical inspection and four tiny filesystem fixtures; strict clippy, format and document checks
  result: Eight tests passed; clippy and formatting passed; no production changes, native JJ probes, large workloads, codec or process-continuation proof
---

# Generated destinations, scoped discovery and pending bodies

## Result and recommendation

**Use a typed finish producer/installer for one report; introduce a tiny desired
provider only when generated names must join child discovery.** Both share explicit
origins, destination claims and validated publication. Neither extends physical
`Action`, invents source presence, or alters ordinary physical differences.

Eight focused tests passed in ignored scratch
`.test-agent/generated-inputs/`. The model demonstrates a report on empty equal
physical roots, A→B→B byte satisfaction, a real virtual directory and file, and
refusal of competing logical/destination ownership. **Refusal and exact bytes are
probe policies, not inferred product defaults.** The scratch code is evidence in
this checkout, not a committed framework or independently distributed test crate.

## 1. Evidence and smallest seam

The checked implementation still establishes:

- `src/visitor.rs:300–317,320–365`: real full `Metadata`, immutable source/target
  facts and factual presence. Generated `virtual/report` cannot be inserted here.
- `src/visitor.rs:437–467`: `InstallLeaf` needs a real source leaf and physical
  difference. It cannot represent generated work on equal inputs.
- `src/visitor.rs:969–987`: public immediate inspection returns a complete local
  physical batch. This is the usable seam for a caller-built composition.
- `src/pair.rs:92–103`: paired lowering releases clean plans. Work must not depend
  on this divergence-only publication path retaining a generated obligation.

The smallest addition is application-owned data beside that immediate batch:

```rust
enum Origin { Physical { source: SourceCoordinate }, Generated { provider: ProviderId } }
enum Content<R> { Bytes(Vec<u8>), Recipe(R) }
enum DesiredPayload<R> { Directory, File(Content<R>) }
struct Desired<R> { logical: LogicalId, target: Target, origin: Origin, payload: DesiredPayload<R> }
struct Claim { owner: OwnerId, target: Target, shape: ClaimShape }
struct Pending<R> { owner: OwnerId, target: Target, body: Content<R> }
struct Envelope<R> { claims: Vec<Claim>, children: Vec<Desired<R>>, pending: Option<Pending<R>> }
```

`Target` includes a rebound root identity and validated lossless relative route in
a real adapter. Scratch uses one separately supplied root and small UTF-8 fixture
routes, not a portable native-path encoding. Desired file kind is regular; desired
attributes are **unspecified**, including mtime/mode. Directory is a desired
container, not an assertion that a physical source directory exists.

For the finish-only case, `children` is empty: reserve `report`, accept the physical
batch even if its diff is empty, wait for accepted child aggregates, derive body,
then publish/dispatch pending installation. A reservation exists before the body.
The probe does not build an ordered recursive driver: it demonstrates the data and
publication discriminators that such a driver must retain.

For physical work, keep the existing verified `DirectoryPlan`/`Action` path.
Adapt its selected destinations to claims for composition; do not translate
generated payloads into physical operations. The model's `Physical` candidate is
used to test duplicate ownership, not to claim a physical copy implementation.

## 2. What the eight probes establish

All test names below are in `.test-agent/generated-inputs/src/lib.rs`. Production
source coordinates above are separate from the compiled scratch-model coordinates.

| Probe | Observed result / bounded inference |
| --- | --- |
| `empty_physical_diff_has_finish_a_b_b` | Real `inspect_directory(... NoVisitor)` on two empty roots returns empty diff on all three runs. Separate report root receives A then B (two successful installations); B again returns no-write satisfaction. Final body is `B:0:1\n`. Physical difference never becomes generated work accounting. |
| `previous_target_is_not_a_producer_but_physical_writer_is` | Existing target `report = previous` is assessed and replaced with A. Adding a competing physical-copy claim to generated B refuses publication; the file remains A. Actual target existence does not create desired ownership. |
| `logical_generated_physical_duplicate_refuses_even_with_distinct_targets` | Physical and generated origins with logical name `report` refuse even when routed to distinct `one`/`two`. This tests logical ambiguity separately from destination collision. |
| `two_to_one_and_leaf_ancestor_refuse_in_both_orders` | `a→x,b→x`, leaf `x` plus `x/y`, and exclusive-tree removal `x` plus child `x/y` refuse in either order. Directory ensure `x` with leaf `x/y` is legal. |
| `provider_discovers_virtual_directory_preserving_origin_and_scope` | Generated root list yields `virtual/`; its list yields `virtual/report`. Both retain generated origin and complete `tiny/v1` domain labels. Unsupported parent refuses. Child dispatch before real parent creation errors; directory then leaf succeeds and yields A bytes. |
| `pending_body_and_reservation_survive_rebind_without_history` | Reservation-only envelope is publishable before a body exists. Owned pending bytes and claims survive a clone/rebind-shaped boundary. Competing later writer still refuses. Identical descriptor binds; changed label or version refuses. No codec/process claim follows from cloning. |
| `invalid_work_never_publishes` | Unclaimed pending destination, unknown recipe version and escaping route refuse before dispatch. |
| `actual_wrong_kind_refuses_without_replacing_directory_or_link` | Existing directory and (Unix) dangling symlink at the report path refuse, preserving their kind/link spelling. Equal-content semantics never follow a symlink to claim regular-file satisfaction. |

Four tests use real filesystem fixtures; four are pure model tests. The filesystem
installer checks root/ancestor kinds, probes with `symlink_metadata`, reads regular
target bytes explicitly, and uses same-parent create-new staging followed by rename.
It returns a boolean for actual successful installation versus satisfied no-op.
This is not a durability, permission-preservation or failure-cleanup implementation.
No failures are injected after creating staging; a failed write/rename may leave it.
No history, rollback or recovery promise is made.

The changed-label experiment is a **new run** B. Resuming frozen descriptor A as B
refuses; otherwise a continuation could silently change the meaning of pending work.
The produced bytes contain the label, accepted leaf count and ordinal. This fixture
therefore proves a parameter change that changes bytes, not that every parameter
change must rewrite. Parameters whose change produces identical required bytes can
still satisfy an exact-byte goal.

## 3. Desired names, physical observations and enumeration completeness

The optional provider has only two legitimate directory requests:

| Parent | Complete generated immediate children | Origin |
| --- | --- | --- |
| logical root | `virtual/` | `Generated(tiny/v1)` |
| `virtual/` | `virtual/report`, recipe A | `Generated(tiny/v1)` |

Its response is `Batch { parent, domain: tiny/v1, complete: true, children }`.
“Complete” means **all this provider's immediate generated children for that parent
under frozen parameters**. It says nothing about physical children or unlisted
targets. It confers no deletion authority. Physical observer completeness remains
a separate result from successful actual enumeration; a provider error is not an
empty complete list. The tiny model rejects requests outside its declared domain.

A combined desired input has two channels: actual physical candidates supplied by
the physical observer, and provider-declared candidates with explicit generated
origin. Validate logical duplicates across their union. Retain each channel's
completeness scope; do not manufacture a complete combined namespace from one
incomplete channel. A future paged provider needs a continuation/completion marker
or an explicitly frozen supplied list before negative conclusions are possible.

The fixture expands the provider's two small lists before effects to validate its
two destinations. This is a two-entry generated declaration, **not an all-source
catalog**. A production driver may expand it per directory if its reservation
already owns the entire generated region. The parent container remains a dependency:
desired directory declaration alone does not authorize writing through an absent or
wrong-kind actual ancestor.

Finish-only production avoids provider enumeration, virtual frames and merged-name
admission entirely. It is enough for a postorder report over accepted results.
Virtual discovery earns these additions when the user wants generated directories
to participate as scheduled children with their own completion/coverage.

## 4. Claims and reservation lifetime

The model distinguishes `Leaf`, `Directory` (ensure only) and `ExclusiveTree`
(replacement/removal/protection boundary for overlap checking). Duplicate target
claims refuse regardless of owner spelling; leaf/exclusive-tree claims conflict
with descendant claims; a directory ensure permits descendant writers. It uses
path-component ancestry, so `x` is not an ancestor of `xy`.

For the tiny case, one generated report destination is reserved in a disjoint root.
Keep that reservation visible in saved program/binding state, even after its pending
body is acknowledged, for as long as competing discovery can occur. It records
ownership, not whether an effect happened. Claims can be regenerated from a fixed
descriptor on bind instead of stored repeatedly, provided reconstruction is exact.
Dropping the pending request must not accidentally release the namespace reservation.

Broader mappings change the memory/ordering obligation:

1. Predeclare and validate the relevant destinations before their effects; or
2. Restrict mappings to provably disjoint reserved regions; or
3. Retain admitted claims across traversal and explicitly accept **late conflict
   failure after earlier unrelated effects**.

A per-directory vector dropped on close cannot prevent a later subtree mapping to
an earlier target. Retained claims for that purpose are not an effects ledger, but
can approach a whole destination index for unrestricted mappings. No simple finish
case should pay that cost. Claim validation also needs any preserved regions and
physical removal/replacement claims; checking generated file writers alone misses
target-only mirror removal and ancestor dismantling.

An old generated target is just the target observed by the installer. Its provenance
does not need to be inferred from a receipt to assess exact bytes. A separately
selected physical source copy/removal at that destination is another claimant.
Reservation/exclusion of physical transfer must be deliberate, not forged absence.
The model refuses conflicts; priority, rerouting, coalescing or a combined producer
are alternative product policies, each needing an explicit acceptance rule.

## 5. Publication, pending production and dispatch handoff

Downstream `diffdir-tree-actions-dispatch` can implement this narrow protocol:

1. Bind a closed program descriptor: operation family/version, frozen label and
   other output-affecting parameters, routing/reservation interpretation, desired
   attributes and chosen satisfaction rule. Known typed Rust enums suffice.
2. Reserve destinations before any competing selected physical/generated effects.
   Accept immediate physical batches independently of divergence. Preserve finish
   obligations and child dependencies in the driver even with no local difference.
3. Produce from **accepted** child results and the current component state. Freeze
   required count/ordinal/label into a recipe, or retain produced bytes. Validate
   logical identities, routes, ownership overlap and body/recipe compatibility
   before publishing an executable envelope. A reservation is not itself a grant.
4. `match Job { Finish, Child(..), Physical(..) }` calls the bound implementation.
   The probe implements only generated `Finish`/`Child`; ordinary physical dispatch
   stays on its current validated API. Runtime target probing is still necessary:
   validated desired data never proves current actual presence or kind.
5. Explicit satisfaction acquisition requests presence/kind, then bytes for regular
   files in this fixture. Unknown or acquisition error cannot become satisfied.
   Required metadata attributes, if added later, require their own facts and checks.
6. Join installation, release live resources, then acknowledge pending settlement.
   Only the owning driver retires the pending obligation, advances component state
   and closes/folds the frame. Skip is incomplete, not satisfaction. An unacknowledged
   visible output is reassessed on retry; no completion record is inferred from it.

The scratch `publish` boundary has a private `Validated` wrapper; `dispatch` takes
that wrapper and cannot accept an arbitrary unvalidated envelope. It rejects
unclaimed output, duplicate logical names, incompatible provider/recipe, malformed
route, overlapping claims and duplicate pending/child ownership. The model doesn't
implement acknowledgment, budgets, skip or serialized continuation; those remain
the ordered driver's contract from the completed synthesis.

### Sufficient recipe state

`Recipe { version: 1, label, accepted_leaves, ordinal }` deterministically renders
`label:count:ordinal\n`; all four fields matter. A descriptor saying only
`summary/v1` is insufficient. A component using `next_ordinal` must save that state
and bind the pending recipe's chosen ordinal; advancing it belongs to acknowledgment,
not speculative production or every retry. Provider-generated names additionally
need frozen enumeration inputs and any unconsumed child list/page position.

A clock, random choice, environment variable, external service or mutable file read
makes a serialized recipe insufficient unless its exact output-affecting values are
frozen. Options: retain chosen bytes; freeze every such input with the interpreter
version; or retain an immutable produced-body reference. A reference needs content
identity, store/binding identity, length/encoding as relevant, immutable retrieval
semantics and a lifetime through acknowledgment/abandonment. Missing or incompatible
content refuses; do not silently regenerate different content. The probe retains
`chosen-clock-value` bytes; it does not implement an object store or test reference
availability. A pending body is future input, not a historical receipt.

At a **real serialized descriptor boundary**, decode limits/schema first, then
resolve a finite known `(kind, version)` and validate its payload before affected
effects. Unknown versions refuse rather than default, skip or reinterpret bytes.
The model checks descriptor version/parameters in `bind`, recipe version in
production and generated provider identity in publication. It does not prove codec
version migration. No global registry, discovery mechanism or plugin ABI is needed.

## 6. Same-tree output requires a declared input domain

The tests deliberately use a separate output root. A same-tree report introduces
two declarations that cannot be inferred from successful bytes:

- **Writer domain:** reserve the destination against physical copying/removal and
  respect any user-preserved region. A generator needs its own granted authority;
  physical preservation alone is not that grant.
- **Input domain:** either exclude old reports from physical summary inputs, or
  explicitly include their pre-install observations as previous-output input.
  Inclusion can change each successive report and need not converge. A declared
  manifest/read phase must say which rule it uses.

Parent folding of accepted child logical results is not rediscovery of child output
files. If the requested report is a post-effect physical inventory, schedule a new
physical observation; pre-effect facts cannot establish it. These same-tree choices
were analyzed, not executed by this fixture, and remain product decisions.

## 7. Checks, scope and handoff

Executed from the repository root with isolated build output:

```sh
CARGO_TARGET_DIR=.test-agent/generated-inputs/target cargo nextest run --manifest-path .test-agent/generated-inputs/Cargo.toml
CARGO_TARGET_DIR=.test-agent/generated-inputs/target cargo clippy --manifest-path .test-agent/generated-inputs/Cargo.toml --all-targets -- -D warnings
cargo fmt --manifest-path .test-agent/generated-inputs/Cargo.toml --check
```

**8/8 tests passed, zero skipped; strict clippy passed.** Initial compilation exposed
an incorrect private-module import, fixed to the public reexports; one collapsible
`if` lint was fixed. Fixture contents are hardcoded tiny values: a conservative
simultaneous bound including private staging is fewer than 20 entries and 256 bytes,
below the commissioned 256 entries/1 MiB limits. Successful tests remove their owned
fixtures; the final fixture-root check found zero remaining entries. Build artifacts
are isolated from these workload bounds. No full suite or native tests were run.

Implementable next seam: **closed typed desired payload + visible reservation +
validated publication + bound installer**, attached to the immediate/finish driver.
Optional provider adds only scoped child enumeration/origin when required. Dispatch
study should check malformed/unknown descriptor rejection, declared write effects,
pending-body compatibility and acknowledgment ownership, not build a global registry.

Residual choices not settled by tests: collision winner/coalescing rules; byte,
digest or unconditional satisfaction; desired attributes; same-tree input inclusion;
general mapping breadth/claim retention; immutable-reference storage and lifetime;
partial-result finish policy. Good source and out-of-band concurrency/topology are
assumptions. The fixture's writable regular-file semantics do not weaken the
existing sync metadata/permission contract.

### Connections

- [Completed synthesis, section 5B](/design/execution/contracts0.gpt61sxh.md#b-generated-input-and-dispatch--diffdir-tree-actions-generated): this is its scoped discriminator and downstream handoff, not a revision of its builder commission.
- [Action protocol](/design/actions/protocol0.gpt6am.md#smallest-implementable-data-and-interface-boundaries): the model compiles a narrow version of its proposed seams and makes reservation-only publication concrete.
- [Demanded facts](/design/observation/facts0.gpt6am.md#3-smallest-demandacquisition-contract): target-byte reads remain explicit acquisition; current full-metadata physical visitors stay intact.
- [Architecture input boundary](/doc/architecture/README.md#physical-generated-and-mapped-inputs): virtual names retain desired origin without becoming physical slots.

Only this model-suffixed document is committed. Parent owns indexes, closure/export,
promotion and any later comparison with withheld material.
