---
type: ResearchPlan
title: Tree reconciliation — layered research and capability map
description: Architecture-statement-first epic map for execution, actions, observation, transfer, archive-backed colocated workspaces and trustworthy evidence.
resource: /design/roadmap/waves0.gpt6a.md
status: planned
generated: { by: "model:openai/gpt-6-astra#high", at: 2026-10-20 }
sources:
  - resource: "opencode://ses_ea82cf969ffeAXqmUxHChTJar2"
    title: Human makes Wave 0 an architecture statement with withheld early bets, asks independent research to test default-on colocation feasibility, and identifies archive and workspace-source scenarios
  - resource: /design/cursor/cursor0.gpt6am.md
    title: AM's approved first bounded-assessment and kopacetic-rkyv example design
  - resource: /design/cursor/implementation0.gpt61sxh.md
    revision: 6c893ca1
    title: Completed first demo and twelve passing focused tests; parent source-reviewed the state machine, codec and subprocess continuation tests
  - resource: /src/visitor.rs
    title: Actual observation, immutable-slot, selected-action and pure-visitor boundaries
  - resource: /src/listing.rs
    title: Native RawDir names currently copied into OsString and cloned at the union boundary
  - resource: /src/walk.rs
    title: Existing pre-order divergence iterator and private continuation frames
  - resource: /design/execution/state0.gpt6a.md
    title: Context lifetimes, native backend and donor state ownership
  - resource: /design/performance/opportunities0.sol61x.md
    title: Existing observation, subtree and multi-target optimization inventory
  - resource: "file:///home/rektide/src/jj-waltz/design/sync-scope/cursor-actions0.gpt6a.md"
    title: Human-directed continuation, action descriptors, generated content and inherited-mode assessment
  - resource: "file:///home/rektide/src/jj-waltz/src/ownership.rs"
    title: Existing Git topology detection, common/admin directory distinctions and checkout ownership
  - resource: "file:///home/rektide/src/jj-waltz/src/workspace.rs"
    title: Registered-but-missing workspace path disagreement between removal and pruning
  - resource: "file:///home/rektide/src/jj-waltz/src/lifecycle.rs"
    title: Current CreationMode combines CoW materialization and colocation as exclusive alternatives
  - resource: "file:///home/rektide/src/jj-waltz/src/jj.rs"
    title: Existing capability probing and native workspace-add colocation integration
  - resource: "file:///home/rektide/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rustix-1.1.5/src/fs/raw_dir.rs"
    title: Borrowed CStr entry names, reusable buffer, alignment and lending next interface
extensions:
  ticket: diffdir-tree
---

# Broad effort, foundational order

The effort is **tree reconciliation and its workspace consumers**, not cursor
serialization alone. The cursor is one mechanism through which observation,
decisions, traversal, generated content and execution can be composed.

**Wave 0 states the architecture. The document is the architecture.** Write the
components, ownership, interfaces and composition directly, using the architecture
already established. Subsequent waves develop its mechanisms. Source and prior
designs inform the statement; implementation discrepancies become explicit work
items rather than silently redefining the architecture.

The parent has written [early architectural stakes](../architecture/statement0.gpt6a.md).
That materialized statement is a bet held apart from independent research.
Dispatch researchers with the [question brief](brief0.gpt6a.md) and their scoped
tickets, **not the statement or this full roadmap**. After committed independent
returns, `diffdir-tree-compare` compares the findings against the original stakes
and proposes an amended architecture. The early statement remains preserved.

The existing diff-dirs tracker was empty (`bd info`: zero issues; prefix
`diffdir`). This map starts one shared effort there. Each ticket names its code
owner: generic mechanisms in diff-dirs, persistence fit in kopacetic, and workspace
semantics/metadata in jj-waltz. Cross-repository tracking does not move ownership
of those implementations. There was no local `.beads` tracker in jj-waltz.

## Operating premises

- Failure leaves the iteration's actual effects. No recovery promise, effects
  journal, rollback, or exactly-once contract is being commissioned.
- Serializable continuation retains tips, unfinished work and the state needed
  to keep iterating, retry or skip. Concurrent mutation/locking is out of band.
- Assume a good donor. Research reuse of its already-computed work rather than
  repeatedly proving the donor premise.
- In the JW scenario, name the normal source **the JJ archive**. Generic library
  APIs may say source. A registered JJ workspace as source is a separate requested
  exploration; it is not an assumption that all source topologies are equivalent.
- Preferred product outcome: `jw add` creates Git-colocated workspaces by default,
  with `--no-colocate` to opt out. Research must establish feasibility and may
  identify legitimate reasons this cannot work or is not worth its costs.
- Let the first Sol xhigh snapshot demo finish unchanged (now completed). Its
  result is evidence for a subsequent component-builder/walking demo; the new
  planning did not interrupt or expand it midway.
- Design/research stubs authorize investigation and bounded proposals. Preferred
  outcomes remain distinguishable from proved capabilities and implemented
  defaults. Experiments remain small and question-driven.
- Unsafe implementation is a permitted research option. Compare concrete layouts,
  lifetimes and savings; borrowed names do not inherently require unsafe code.

## Epic map

All IDs below are in the `diffdir-` namespace. The umbrella is `diffdir-tree`.
The 37 work items and 73 parent/prerequisite edges were created; dependency-cycle
checking passed. The initial snapshot-demo ticket is now closed after completion
and parent source review. Its [runnable example](../../examples/cursor/README.md)
and [implementation evidence](../cursor/implementation0.gpt61sxh.md) remain the
baseline for the next demo. Actual dispatches, returns and integration are recorded
in the [running flight ledger](flight0.gpt6a.md).

| Epic | Scope and tension | Child work IDs (suffix after epic ID) |
| --- | --- | --- |
| `diffdir-tree-architecture` | State the architecture: components, contracts and end-to-end composition | `structure`, `contracts`, `scenarios`, `statement` |
| `diffdir-tree-execution` | Stateful stepping, entity completion, construction and continuation across processes | `snapshot-demo`, `model`, `builder`, `walk-demo` |
| `diffdir-tree-actions` | Observed facts versus chosen work; extensible parameters; generated entries, mappings and dispatch | `protocol`, `generated`, `dispatch` |
| `diffdir-tree-observe` | Minimum useful facts; names/path ownership; borrowed native records and persisted representation | `facts`, `names`, `codec` |
| `diffdir-tree-transfer` | What copying means; preserve/update/remove authority; subtree specialization, kernels and donor reuse across targets | `policy`, `modes`, `kernels`, `multi-target` |
| `diffdir-tree-native` | Archive-to-workspace Git colocation, native metadata, workspace-source exploration and JW integration | `metadata`, `colocation`, `workspace-source`, `identity`, `graph`, `integration` |
| `diffdir-tree-evidence` | Trustworthy tests and experiments while development continues | `fixtures`, `artifacts`, `budgets` |

`diffdir-tree-contract` is the cross-epic decision account, joining the foundational
mechanism-design findings after the architecture statement and before a
shared production execution API is selected. It records open
choices as open; the ticket is not an instruction to manufacture consensus.
`diffdir-tree-compare` later compares independent conclusions with the withheld
early architecture statement. These are separate synthesis and comparison jobs.

### Architecture statement

- **structure:** state components and relationships: observation, assessment,
  traversal, actions/application, continuation and consumer adapters. Define
  ownership and where values flow, are retained, or are returned.
- **contracts:** state the interfaces and composition rules, including pure
  observation/visitor work versus ordered stateful iteration, selected actions,
  completion and serializable state. Say which decisions belong to callers.
- **scenarios:** express the architecture through ordinary tree copying,
  archive-backed workspace creation, and pause/resume walking with generated
  output. The scenarios make the component contracts concrete.
- **statement:** integrate those sections into the maintained architecture body,
  linked from the root specification. The artifact defines the architecture;
  implementation status and follow-up work are recorded separately. Carry
  consequential unresolved choices explicitly into the appropriate tickets.

### Execution

- **snapshot-demo:** finish the commissioned AM→Sol example using real immediate
  inspection and kopacetic-rkyv. Record what actually works and archive size on a
  tiny fixture. Completed through `6c893ca1`: 12 focused tests passed; parent
  reviewed the actual implementation. Paused tiny-fixture archive: 3,618 bytes.
- **model:** define step, assessment unit, completed entity, unfinished frame,
  tips, accumulated state and skip. Compare pre-order work with post-order
  completion without conflating iteration order and mutation order.
- **builder:** a component-built cursor API for fresh construction and restoring
  captured parts. Decide ownership/borrowing of roots, observation provider,
  assessor, descriptor handlers, frame state and codec. Compare a thin typed
  builder (bon where useful) with an unnecessarily coupled global context.
- **walk-demo:** follow the first demo with an explicit step/pause/serialize/
  deserialize/resume walking loop, constructed through the builder. Exercise
  direct component construction and reconstructing saved parts. No prefix replay.

### Actions

- **protocol:** define observations, application attributes, selected action,
  child traversal mode and completion output. Decide where the current enum is
  sufficient and where handler ID/version/parameters are needed. A disposition
  need not become another mandatory layer if action plus traversal already fits.
- **generated:** physical versus virtual inputs, byte/generator origins, injected
  tasks, path mappings, child-derived output and same-destination conflicts.
  Examine state lifetime and cursor representation with small examples.
- **dispatch:** how registered handlers consume parameterized descriptors, yield
  children, finish entities and expose resumable inner state. Keep pure speculative
  Visitor hooks separate from the ordered stateful driver.

### Observation and representation

- **facts:** what can be answered from listing facts, what needs stat/readlink,
  and which facts an action needs later. Equal physical pairs can still require
  generated/transformation work; action output cannot depend on physical divergence.
- **names:** dirent-to-borrowed-path/component views, packed arenas and offset
  tables, sorting, buffer lifetime and descriptor-relative operations. Compare
  safe borrowing first, bounded unsafe/native alternatives where they buy work.
- **codec:** kopacetic Facet-row/rkyv snapshots versus native typed rkyv and a
  packed-name cursor. Examine actual fixture sizes/allocations, reconstruction
  cost, parameter shapes and schema changes. No second codec in the running demo.

### Transfer

- **policy:** separate source-present overwrite, target-only removal, preserved
  regions, generated output and initial/final tracked classification. Select
  consumer semantics explicitly; do not derive ownership from ignoredness alone.
- **modes:** change recurse tips to inherited Compare/CopySource/RemoveTarget/
  Preserve or specialized modes. Preserve residual exceptions. Distinguish
  avoiding assessment from avoiding enumeration or physical installation.
- **kernels:** reuse observation in apply; avoid redundant metadata reads, path
  construction, staging and permissions work. Coordinate with existing delivered
  kernel optimizations rather than proposing them again as unimplemented.
- **multi-target:** source-to-many traversal and shared donor observations, with
  per-target scope and progress. Design after the single-target facts/lifetime
  model; do not introduce a persistent full-tree catalog by default.

### Native adapters and product behavior

- **metadata:** describe the actual JJ archive and receiving workspace layouts,
  including `.jj`, the Git backing/common directory and the receiving workspace's
  `.git` gitfile/worktree administration. State what the iterator preserves,
  suppresses or leaves to native creation. Focus on this flow; add other Git
  topologies only when the concrete source/target scenarios require them.
- **colocation:** determine whether and how default-on Git colocation for `jw add`
  can work, flaggable off with `--no-colocate`. Native JJ's `workspace add --colocate` creates
  the corresponding Git worktree; JW already exposes the flag. Resolve CLI/config
  precedence, actual executable capability and interaction with CoW copying.
  The current `CreationMode` treats colocation and CoW as mutually exclusive.
  Determine whether the constraint is local policy, native mechanics, or a real
  cost tradeoff. State legitimate obstacles and the scope of shared add/switch
  creation policy; do not force the preferred default through contrary findings.
- **workspace-source:** explore using an existing registered JJ workspace as the
  source instead of the normal JJ archive. Compare source binding, selected tree,
  ignored/installed contents and colocated/JJ-only metadata; retain the receiving
  workspace's own identity. Existing `--from` support is a starting point, not
  evidence that every composition has been verified.
- **identity:** repair the reproduced registered-but-missing checkout disagreement.
  `remove` loses its path before ownership validation; `prune` reaches an adapter
  that can recover the missing path. Share facts without weakening ownership.
- **graph:** ordinary copying versus requested revision advancement. Explain when
  snapshotting changes classification, which target identity must stay, and what
  donor tree-state reuse saves when S equals or differs from desired D. No locking
  research or repeated donor validation as the default premise.
- **integration:** once semantics are chosen, map generic mechanisms into creation,
  reseed and prepare, configuration/migration, and reported output. Keep existing
  `copy_ignored=[]` and other meanings explicit until deliberately replaced.

### Evidence and development environment

- **fixtures:** repository-free discovery roots, CoW-capable fixtures, isolated
  config and native executable delegation. The historical in-repository TMPDIR
  escape and mise-wrapper recursion are concrete precedents.
- **artifacts:** bind evidence to JW and diff-dirs revisions, features, immutable
  CLI artifacts and real JJ version. Remove inconsistent runtime binary discovery
  from the suite and avoid shared output paths being replaced during tests.
- **budgets:** proportional verification, bounded resident fixtures, observable
  setup/run/verification, and allocation/syscall counts for micro-optimizations.
  The cancelled large retained-copy campaign is evidence against repeating it.

## Layered waves

### Wave 0 — state the architecture

Write the architecture itself, in direct terms: "this component owns…", "these
values cross this interface…", "the caller selects…". Its three drafting concerns
are **structure**, **contracts**, and **scenarios**, followed by integration in
`architecture-statement`. This replaces the previously proposed anatomy/rationale/
fitness assessment wave; the renamed tickets retain their history.

Use the [execution walkthrough](../execution/flow0.gpt6a.md),
[state/lifetime account](../execution/state0.gpt6a.md), root/sync specifications,
and established JW architecture as writing inputs. The cursor example demonstrates
one composition. State ordinary copy and the archive/workspace flow; retain
colocation feasibility as an explicit open question until the research answers it.

Independent model-suffixed drafts live under `design/architecture/`; integration
produces the maintained `doc/architecture/README.md`, linked and reconciled with
the root specification. The document is the architectural authority in its scope.
Use a separate implementation-status account for gaps; existing code quirks do
not acquire authority just by appearing in a trace. Open choices stay explicit
rather than being filled by speculative defaults.

The parent early-bet statement is withheld from those authors and their initial
integration. Completion of the independent statement unblocks the four mechanism-design studies below.
Operational repairs and the bounded native colocation work remain independently
actionable; the generic architecture is not a prerequisite for fixing a known bug
or determining whether the desired `jw add` default is feasible.

### Wave 1 — design changes to the semantic core

Commission independent treatments of `execution-model`, `actions-protocol`,
`observe-facts`, and `transfer-policy`. Each starts from the Wave 0 architecture,
current code and human premises and does not read another author's same-wave draft. Outputs
identify concrete examples, design forks, proposed interfaces, and what remains
undecided. Epic owners may expand their own stubs with justified children and
acceptance criteria, keeping links back to this map.

Synthesize those four accounts in `diffdir-tree-contract`. Ask whether their data,
state and control ownership actually compose. Resolve the smallest common core
before selecting generalized dispatch, generated-tree policy or production copy
defaults. The first demo's completed evidence feeds this review.

In parallel, `native-identity`, `evidence-fixtures` and `evidence-artifacts` can be
investigated or repaired independently: they do not need a cursor redesign.
`native-metadata`, `native-workspace-source` and `observe-names` can gather layout evidence now;
their implementation choices wait for the relevant ownership/lifetime contracts.

### Wave 2 — a component-built walking specimen and representation choices

After the current demo and execution-model account, design the builder and then
the walking demo. This is the first practical follow-up: explicit small steps,
pause with unfinished children, cursor roundtrip, direct construction from parts,
and continuation to post-order results. Keep it an example until it exposes a
reusable API worth promoting.

After the semantic synthesis, investigate generated inputs and handler dispatch.
Use a tiny physical-copy case, a generated-file case and a mapped-path case to
test that one contract describes them. The names and codec studies compare
representation choices against those actual component/lifetime needs.

### Wave 3 — work avoidance and metadata adapters

Design inherited modes, revisit kernel costs, and complete `.git`/`.jj` adapter
and graph-intent accounts. A source-only subtree is the first useful shortcut:
drop paired assessment and carry just the residual rules. A whole-tree action
must say whether its interior can pause; one opaque action is not bounded work.

Avoid pushing native metadata into the generic enum. An adapter should express
preservation or generated content through the shared mechanism, then perform
only the native initialization/graph steps actually needed.

### Wave 4 — composition and production adoption

Investigate source-to-many reuse and JW command/config integration using the
selected single-target contracts. Expand implementation epics here rather than
pre-writing a speculative task tree for every possible architecture. Measure only
bounded cases that answer a decision. Parallel execution is a later question if
the sequential state model and evidence justify it.

The human subsequently commissioned this flight and selected AM + Sol xhigh.
The [flight ledger](flight0.gpt6a.md) records independent sessions, returned
evidence and synthesis checkpoints. The original cursor specimen remains preserved.

### Comparison with the early stakes

Once the foundational contracts, walking specimen, action dispatch, subtree modes,
name representation and native source/colocation investigations have returned,
`diffdir-tree-compare` reads their committed outputs alongside the withheld parent
statement. Classify each bet as supported, conditional, contradicted or unresolved.
Explain how the resulting architecture and implementation sequence should change.
This comparison may reject a preferred mechanism or colocation default. Preserve
the original statement; publish amendments with their evidence and decision owner.

## Dirent research starting point

On Unix, an existing dirent name can already become a borrowed path without
allocation or a transmute:

```rust,ignore
use std::{os::unix::ffi::OsStrExt, path::Path};
let name = std::ffi::OsStr::from_bytes(entry.file_name().to_bytes());
let path = Path::new(name);
let components = path.components();
```

`PathBuf` is owned storage. A borrowed variable-length dirent is not layout- or
ownership-compatible with it. The promising work is retaining packed name bytes
and sorting offsets/indices, with borrowed `OsStr`/`Path` views and lazy full-path
construction. Current native `listing.rs` allocates `OsString` per name and
clones it for later union processing, so there is concrete work to count.

Rustix `RawDir::next` lends names from a reusable buffer; the borrow cannot survive
the next mutable iteration. An arena, retained batches, or another parser API is
needed to keep sortable borrowed names. `zerocopy` may help validate a chosen raw
header layout; it does not supply buffer lifetime or turn a name into owned
`PathBuf`. A native/custom-unsafe route remains eligible if its measured advantage
justifies that implementation. Persistent cursors should encode names/offsets,
not process pointers or assume native directory seek cookies are durable positions.

## Completion of a research stub

Each return supplies: inspected code coordinates, applicable prior work,
competing designs and tensions, smallest discriminating experiment, selected or
still-open decisions, and proposed next children/dependencies. Commit independent
model-suffixed documents; maintain the epic's few useful pointers. Evidence of an
experiment and human selection of a direction remain separate facts.
