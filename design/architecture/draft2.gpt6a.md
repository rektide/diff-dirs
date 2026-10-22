---
type: ArchitectureDraft
title: Domain-owned crates and the extraction sequence
description: Accepted domain-grouped layout and path-derived package naming, with observation, immediate inspection, traversal and effects separated before coordinated comparison implementation.
resource: /design/architecture/draft2.gpt6a.md
status: accepted-direction-implementation-in-progress
generated: { by: "model:openai/gpt-6-astra#high", at: 2026-10-22 }
sources:
  - resource: "opencode://ses_e9e45e9f4ffeZUWoZCK5VDjJvv"
    title: Human accepts the tree/transfer/app crate breakdown, requires path-derived names for every crate, and authorizes Sol xhigh implementation through extraction and relocation, stopping before coordinated comparison
  - resource: /design/architecture/draft0.gpt6a.md
    title: Overall component ownership and optional future effectful progression controller
  - resource: /design/architecture/draft1.gpt6a.md
    revision: e588cdd0
    title: Coordinated comparison design motivating a shared inspection boundary, not yet authorized for implementation
  - resource: /Cargo.toml
    revision: e588cdd0
    title: Starting root package and workspace membership
  - resource: /src/visitor.rs
    revision: e588cdd0
    title: Starting combined facts, policy, immediate inspection and finalized-plan implementation
  - resource: /src/listing.rs
    revision: e588cdd0
    title: Starting platform enumeration and deferred metadata acquisition
  - resource: /src/error.rs
    revision: e588cdd0
    title: Starting shared physical and policy-refusal error boundary
  - resource: /sync/Cargo.toml
    revision: e588cdd0
    title: Existing filesystem effect package
  - resource: /cli/Cargo.toml
    revision: e588cdd0
    title: Existing executable package and feature forwarding
  - resource: /perf/runner/Cargo.toml
    revision: e588cdd0
    title: Existing path-derived performance runner package name
  - resource: /perf/operation/Cargo.toml
    revision: e588cdd0
    title: Existing operation benchmark dependencies to relocate
  - resource: /examples/cursor/Cargo.toml
    revision: e588cdd0
    title: Standalone cursor example workspace and local optional-ecosystem dependencies
  - resource: /examples/walk/Cargo.toml
    revision: e588cdd0
    title: Standalone walking specimen and optional codec feature boundary
  - resource: "file:///home/rektide/src/rekon/code/layout.md"
    title: Domain-grouped layout and registry-prefix guidance
extensions:
  refines: /design/architecture/draft1.gpt6a.md
  related_work: [diffdir-tree]
verification:
  scope: Records accepted direction and implementation boundaries; implementation results will be appended after the owned stages are verified
---

# Domain-owned crates

## Acceptance and stopping point

The human accepted the **tree / transfer / app** layout and the observation,
inspection, comparison, synchronization, facade and CLI responsibilities. Package
names must follow the complete repository-relative crate path, prefixed by
`diff-dirs-`. The filesystem tree itself carries no repeated package prefix.

This authorizes the extraction and relocation sequence below, through step 4.
Implementation of coordinated comparison (step 5) remains the next unstarted slice.
The [coordinated design](draft1.gpt6a.md) continues to define that future capability.

The facade follows the same naming rule: `tree/api` is `diff-dirs-tree-api`, with
Rust import name `diff_dirs_tree_api`. Preserve recognizable API symbols through
re-exports, while deliberately updating the package/import coordinates. There is
no exception retaining a second package named `diff-dirs`. The executable stays
`diff-dirs`; package identity and command name are different concerns.

## Layout and names

```text
Cargo.toml                   virtual workspace
tree/
  observe/                   diff-dirs-tree-observe
  inspect/                   diff-dirs-tree-inspect
  compare/                   diff-dirs-tree-compare
  api/                       diff-dirs-tree-api
transfer/
  sync/                      diff-dirs-transfer-sync
app/
  cli/                       diff-dirs-app-cli, executable diff-dirs
perf/
  runner/                    diff-dirs-perf-runner
  operation/                 diff-dirs-perf-operation
examples/
  cursor/                    diff-dirs-examples-cursor
  walk/                      diff-dirs-examples-walk
```

Examples may retain their existing standalone workspaces and optional local codec
dependencies. Rename their packages and update dependencies/imports, without making
the main workspace require those external checkouts. Performance package names
already satisfy the convention. Root documentation remains the project entry point;
each extracted crate has a concise explanation of its owned boundary.

## Responsibilities and dependency direction

| Crate | Owns | Excludes |
| --- | --- | --- |
| `tree/observe` | Physical listing, native names/hints, provenance of physical facts, deferred no-follow metadata and backend-owned acquisition resources | Comparison policy, traversal, target mutation and policy-refusal errors |
| `tree/inspect` | Pair name unions, admission, comparison predicates, late policy, validation, immediate plans and operation-specific acquisition orchestration | Recursive traversal and mutation |
| `tree/compare` | Pair traversal, ordered frontier, summaries and pruning; later coordinated traversal | Filesystem effects and native repository orchestration |
| `transfer/sync` | Transfer policy, staged replacement, permissions, current recursive application and filesystem effect results | Ownership of edges expanded by a different driver |
| `tree/api` | Stable convenient public exports and comparison entry points | A second implementation or a dependency on effects |
| `app/cli` | CLI arguments, configuration and output | Library policy ownership |

Dependency arrows below mean **depends on**:

```text
tree/inspect  → tree/observe
tree/compare  → tree/inspect
transfer/sync → tree/inspect + tree/compare
tree/api      → tree/compare + tree/inspect (+ observation exports as needed)
app/cli       → tree/api
```

Sync legitimately needs immediate inspection and paired walking today. It should
consume those owning crates rather than depend on the facade to hide its actual
requirements. Examples and external-style tools may use the facade. No lower crate
depends on it, and there must be no dependency cycle, including test-only cycles.

### Type and error ownership

Move physical `FileKind`, `FactOrigin` and early physical hints downward with
observation. Keep admission decisions, comparison operations, visitors, finalized
plans and policy refusal with inspection. `DirDiff` and immediate entry-count
categories belong with inspection because finalized immediate plans contain them;
recursive `Summary`, walk options and traversal state belong with comparison.
An execution setting used by immediate acquisition can have its natural lower
owner and be re-exported rather than copied into a second incompatible enum.

Observation errors retain path, operation and underlying I/O cause. Inspection
maps those into its public comparison/validation error boundary while retaining
the cause. An observation crate must not depend on `CheckError` merely because
the old listing module used one monolithic error enum. No catch-and-print layers.

Avoid a miscellaneous shared-types crate. Public types have a semantic owner;
re-export the same type identity rather than introducing conversion wrappers
solely for crate boundaries.

### Encapsulation across crates

An extracted crate necessarily exposes some interfaces previously private within
one package. Give them deliberate responsibilities:

- Observation exposes owned entries/batches or bounded acquisition handles with
  read-only hints and explicit acquisition methods. Backend descriptors, scratch
  allocation and platform implementations stay private.
- Inspection owns construction and validation of finalized plans. Traversal can
  consume/read their selected children, counts and coverage through narrow methods
  or a deliberately owned lowering result. Do not make every plan field public.
- Pure policy and physical acquisition remain distinct even when they are called
  within one immediate inspection operation.
- Keep native enumeration and standard enumeration in one observation crate behind
  the existing feature choice. Resource pooling also belongs there when implemented.

Extraction does not itself implement the future shared-source batch protocol,
runtime dirent tuning or buffer pooling. It establishes their correct owner and
the reusable immediate inspection boundary without claiming those capabilities.

## Implementation sequence

### 1. Document the boundary — this stage

Record the accepted layout, names, graph, type ownership, preservation obligations
and stopping point. Connect this document to the maintained architecture and the
coordinated comparison refinement. Commit the document stage before code extraction.

### 2. Separate immediate inspection from traversal

Create `tree/inspect` and `tree/compare` at their intended paths. Move existing
inspection and walking responsibilities into those owners, initially retaining
listing privately under inspection if needed. The current root package can act as
a temporary facade until relocation. Move tests to their owning crates or maintain
external API tests at the facade; shared fixtures must not introduce crate cycles.

Keep current single-target behavior, including full-fact visitors, error precedence,
clean-plan handling, native-name order, paired-only descent, deterministic commitment,
divergence seals, pruning and prefix summaries. Audit stays with its immediate or
recursive owner according to the actual implementation, with facade exports intact.

### 3. Extract physical observation

Create `tree/observe`, move physical facts/listing/backend ownership downward, and
repair the error boundary. Inspect remains responsible for deciding what to acquire
and for assembling the comparison. Retain native feature behavior and propagate
features through the dependency graph. In particular, do not turn Linux/nightly
native enumeration into a mandatory default requirement.

Review the resulting public acquisition and lowering seams before relocation.
They should support future shared-source inspection without exposing unvalidated
plans or importing traversal policy into observation.

### 4. Relocate and integrate the workspace

Move the facade to `tree/api`, sync to `transfer/sync`, and CLI to `app/cli`.
Make the root manifest a virtual workspace. Apply the path-derived package names
everywhere, including the standalone examples, dependency declarations, executable
package selection, lockfiles, build scripts and current tool/test commands.

Update maintained documentation, source-path navigation and examples to the new
coordinates. Preserve revision-bound research as historical evidence rather than
rewriting old experiment commands into claims about new code. Give current readers
a relocation map for historical paths where useful. Native dependency feature
forwarding and diagnostic feature combinations must still work as documented;
changing defaults or adding a previously absent CLI capability is separate work.

### 5. Stop: coordinated comparison implementation

Do not implement the membership frontier, multi-target API, group tokens, pruning
protocol or new coordinated CLI modes in this pass. The decomposition is the
landing foundation for [draft1](draft1.gpt6a.md), not permission to execute its
entire roadmap. Likewise, leave generic resumable progression in its specimen
until a production consumer justifies `execution/progress`; keep JW-specific
adapters with JW.

## Verification and stage checkpoints

Each coherent extraction/relocation stage receives an explicit-path jj commit.
Review dependency direction and cross-crate encapsulation at the observation and
final-layout boundaries. Use Sol xhigh agents for implementation and verification;
the parent retains architectural integration responsibility.

Run existing meaningful tests with nextest, clippy checks and formatting appropriate
to the changed packages. Exercise default and parallel behavior and the existing
Linux/nightly native feature path where supported. Verify documentation tests,
CLI package/executable selection and standalone examples with their available
feature/dependency combinations. Do not run broad performance workloads merely to
validate a structural move; check affected benchmark packages/build scripts instead.

Inspect Cargo metadata for names, paths, feature forwarding and acyclic dependency
direction. Report unavailable external dependencies or pre-existing failures with
their actual boundary. Do not claim untouched standalone codec integrations passed
without checking them. Reuse completed stage checks; rerun only when subsequent
changes invalidate them or a concrete concern needs verification.

## Connections

- [Maintained architecture](/doc/architecture/README.md) is the navigation home.
- [Coordinated comparison](draft1.gpt6a.md) motivates immediate source acquisition
  reuse and stays the next capability after extraction.
- [Overall architecture](draft0.gpt6a.md) owns the wider distinction between
  observation, inspection, progression, effects, continuation and native adapters.
- [Root specification](/README.md) continues to govern delivered comparison behavior
  throughout the package move; namespace changes do not change its predicate.
