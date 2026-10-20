---
type: ImplementationHandoff
title: A component-built walking cursor
description: Concrete builder, runtime seams, saved parts and acceptance gates for the isolated walking specimen.
resource: /design/execution/builder0.gpt6am.md
status: proposed-implementation-handoff
generated: { by: "model:openai/gpt-6-astra#medium", at: 2026-10-20 }
sources:
  - resource: "opencode://ses_ea7bed76affemZoRhULOaLPhBZ"
    title: Selected a concrete builder and narrow replaceable I/O seams for the commissioned walking implementation
  - resource: /design/roadmap/brief0.gpt6a.md
    title: Independent commission and reading boundary
  - resource: /doc/architecture/README.md
    title: Component ownership and filesystem-free continuation validation
  - resource: /design/execution/contracts0.gpt61sxh.md
    revision: 5d6c003a
    title: Selected specimen semantics, state ownership and four exact process gates
  - resource: /design/execution/model0.gpt61sxh.md
    revision: 3113dfd2
    title: Ordered child ownership, pending requests and retry semantics
  - resource: /design/actions/protocol0.gpt6am.md
    revision: 134e0c35
    title: Typed desired work distinct from physical plan actions
  - resource: /src/visitor.rs
    title: Public immediate inspector signature and native-name ordering
  - resource: /src/lib.rs
    title: Public physical inspection exports
  - resource: /examples/cursor/Cargo.toml
    title: Existing standalone workspace and local Snapshot dependencies
  - resource: /examples/cursor/src/codec.rs
    title: Actual Snapshot reconstruction and complete-file publication precedent
extensions:
  ticket: diffdir-tree-execution-builder
  reading_boundary: Only commissioned prior sources and scoped ticket read; withheld parent statement, full roadmap, parent transcript and other current-wave drafts not consulted; no subagents
verification:
  method: Source-informed API design and document checks; no code implementation or Rust typecheck
  result: Proposed signatures are implementation targets, not claims of delivered APIs
---

# A component-built walking cursor

Implement **`examples/walk` only**, with a real builder accepting owned saved parts
and executable components. The selected [synthesis, section 4](/design/execution/contracts0.gpt61sxh.md#4-next-commission-a-component-builder-and-real-walking-specimen)
governs semantics. This handoff fixes construction and the smallest useful runtime
seams. Preserve `examples/cursor`, core and sync unchanged.

## 1. Builder and ownership

Use one builder function with a mutually exclusive `StartParts` input, two I/O
traits, and concrete program/manifest implementations. Bon generates required-field
setters and type inference; validation occurs at the finishing call. Proposed API:

```rust
pub enum StartParts {
    Fresh(Bindings),
    Restored(SavedWalk),
}

#[bon::builder(finish_fn = build)]
pub fn cursor<I: Inspector, E: Executor>(
    parts: StartParts,
    inspector: I,
    executor: E,
    program: DesiredCopy,
    manifest: Manifest,
) -> Result<Running<I, E>, Error> {
    // Fresh: initialize driver, descriptor and ManifestState from components.
    // Restored: validate data and compare supplied component descriptors.
    // Move the supplied objects into Running; perform no filesystem I/O here.
    todo!()
}
```

The following is the intended public usage (with application paths/error return
in scope), rather than just a config builder hiding the same global runtime:

```rust
let mut run = cursor()
    .parts(StartParts::Fresh(Bindings::new(source, target, reports)?))
    .inspector(FullInspector::default())
    .executor(ByteExecutor::default())
    .program(DesiredCopy::v1())
    .manifest(Manifest::builder().label("A".to_owned()).build())
    .build()?;
run.bind_filesystem()?;
let progress = run.drive(Limits { assessments: 2, effects: 2 })?;
let saved: SavedWalk = run.into_saved()?;

// Another process, or codec-free in-memory reconstruction:
let manifest = Manifest::from_config(saved.manifest_config().clone())?;
let mut resumed = cursor()
    .parts(StartParts::Restored(saved))
    .inspector(FullInspector::default()) // genuinely new executable objects
    .executor(ByteExecutor::default())
    .program(DesiredCopy::v1())
    .manifest(manifest)
    .build()?;
resumed.bind_filesystem()?;
resumed.drive(Limits { assessments: 0, effects: 1 })?;
```

`Bindings::new` converts paths to owned Unix bytes and performs **lexical** checks
only; require absolute normalized roots. CLI resolves relative arguments against
cwd before calling it. No fresh/restored state ambiguity or relocated-root setter:
restored parts already own their roots. `Manifest` may derive `bon::Builder`;
`label` is required, its rendering version is fixed by this implementation.
`DesiredCopy::v1`, `FullInspector::default`, and `ByteExecutor::default` supply
explicit defaults; avoid a second shortcut constructor with hidden components.

`Running<I,E>` owns `{ saved, inspector, executor, program, manifest, bound }`.
`bound` is invocation-local and initially false. `into_saved` consumes the runtime;
it may retain an issued-but-unacknowledged owned request, but never active staging
or jobs. Synchronous dispatch makes this a straightforward resource boundary.

The saved manifest has **config `{ version, label }` and state `{ next_ordinal }`**,
initially 1. Rebound `Manifest` holds rendering code/config, not another mutable
ordinal. `Manifest::from_config` must not reset saved state. Pure
`manifest.recipe(config, state, node, aggregate)` supplies finish inputs; driver
acceptance alone advances the saved ordinal. Program descriptor freezes the
desired-copy v1 interpretation, bytes-only rule, retention and report mapping.
Restoration compares supplied descriptors/config exactly, including label. The
program is deliberately concrete; arbitrary visitors/program languages are not
part of this specimen. Both supplied concrete objects are used by the runtime.

## 2. Useful replaceable components, not a trait per noun

These are the two replaceable seams; all result types belong to the example:

```rust
pub trait Inspector {
    fn inspect(&mut self, bindings: &Bindings, node: &Node)
        -> Result<diff_dirs::DirectoryPlan, Error>;
}
pub trait Executor {
    fn execute(&mut self, bindings: &Bindings, work: &Work)
        -> Attempt<EffectOutcome>;
}
```

`Attempt<T>` carries `Result<T, Error>` plus current-attempt actual namespace-write
counts, including failure after visible rename. It is not stored as an effects
ledger. `EffectOutcome` says goal settled, with installed/no-write information;
the driver derives the leaf contribution from the pending work, not a caller's
arbitrary numeric count. External acknowledgments remain trusted caller evidence.

`FullInspector` performs scheduled-directory route/type checks, then calls public
`inspect_directory` with real containing paths and `NoVisitor`, full facts and
sequential inspection. `DesiredCopy` borrows the resulting complete plan to lower
owned selected children and physical comparison counts. It does not follow
`Action` to decide desired-byte work: even physically equal regular files get a
settlement goal. Target-only entries are retained and their interiors not admitted.
For source-present entries reject links/specials on either side and leaf/directory
collisions; retained target-only subtrees are not recursively classified.

Lower the whole batch or none, checking every selected child's route and directory
report name before publishing it. Source-only directories get a `Before` ensure
unit and then ordinary immediate inspection after creation. A present matched
directory needs no ensure effect. Clean/empty root inspection still schedules its
finish. Never reuse the old cursor's all-directory reduction to bypass retention.

`ByteExecutor` matches exactly three `Work` variants: ensure directory, settle
source regular file, install generated report. It rechecks current relevant routes
and kinds, compares exact bytes, and installs through same-parent private staging
plus rename when needed. Close files and remove owned staging before return; report
cleanup errors. No recursive apply, chmod repair, metadata preservation or reflink
contract. Ordinary directory creation uses process umask. Existing output must be
regular or absent, with no symlink write-through or type replacement.

Callers can provide counting/read-failure `Inspector` wrappers and instrumented or
fault-injecting `Executor` wrappers without altering the driver. Their mutable
diagnostic counters are invocation-local. Implementations must obey the declared
full-inspection and three-work semantics and have no hidden behaviorally necessary
cross-pause state; additional durable component state is outside v1. No registry,
dynamic type loading, runtime downcasts, inspector factory or generic reducer needed.

**Construction proof:** supply a delegating counting inspector and an executor that
records requests and fails once before `b`, plus label `custom`. Assert inspector
calls, the blocked `b`, accepted `a`, and the eventual custom report bytes. Restore
with fresh wrappers and assert zero ancestor inspection calls and retained ordinal.
This fails if the builder discards supplied components or runtime calls defaults.

## 3. Manual protocol and numeric convenience drive

Expose owned `Request::{Inspect, Effect, Done}` with a ticket on work requests,
and `Ack::{Inspected, Effected, Failed}` carrying that ticket and matching payload.
`next_request(&mut self)` drains pure transitions and returns an owned request;
repeated reads return the same pending ticket. It does not perform I/O. Inspect
acks carry the transient complete plan; pure lowering and validation happen before
accepting it. Nothing stores that plan in `SavedWalk`.

`dispatch(&mut self, &Request) -> Ack` validates that the request matches the
pending unit and invokes **the supplied** inspector/program or executor. The caller
can instead execute externally and call `acknowledge(ack)`. Dispatch requires a
filesystem-bound nonterminal runtime. Acks check ticket, phase and work kind;
duplicate/old/wrong-phase acks fail without state mutation. Preparing candidate
folds, ordinal changes and checked counters precedes committing any transition.

`drive(Limits)` is a loop over this same protocol. Charge before each attempted
dispatch, join/close resources, acknowledge, then consider the next budget. Zero
assessments or effects is valid. Never leap over an exhausted class to run a later
unit. `0/0` may drain pure transitions and expose the next unresolved request.
Return progress with stop reason (`Budget`, `Blocked`, `Done`), attempted units,
accepted goal results and invocation-local actual writes separately. Manual drive
has no implicit numeric allowance; hosts choosing it own their dispatch budget.

A failed ack leaves the goal blocked. `retry_blocked_unit()` clears the block and
issues a fresh ticket for the same goal; an unacknowledged request restored from
parts remains unresolved and must be dispatched/re-examined, never assumed done.
`skip_blocked_unit()` abandons before/assess subtree, leaf, or finish respectively,
immediately marks every active ancestor incomplete and leaves visible effects alone.
Partial finishes are permitted. Before → assess → children → finish and active-front
edge ownership remain exactly as [the execution contract](/design/execution/contracts0.gpt61sxh.md#2-progress-acceptance-and-failure).
Closure folds only once in the advancing lineage. Skip consumes no finish ordinal;
retry uses the saved recipe's same label, aggregate and ordinal.

## 4. Saved representation, validation and binding

`SavedWalk` owns bindings, frozen descriptors, driver frames/remaining suffixes,
accepted local/child aggregates, coverage, pending goal/ticket/block, next ticket,
manifest config/state and optional terminal outcome. Finish recipes retain their
inputs, not filesystem inventory. No completed-edge history, old physical grants,
file handles or metadata. Unix names/paths use `Vec<u8>` with `OsStrExt`/
`OsStringExt`; never roundtrip identity through display strings.

`validate()` and builder restore are data-only. Check supported versions/config,
absolute normalized distinct non-nested roots; root-relative component shape
(no empty interior components, NUL, `.`/`..` or leading slash); native sorted unique
remaining children; stack/front-edge correspondence; phase/local/finish consistency;
pending request matches top phase; ticket progression; checked aggregate arithmetic;
incomplete propagation; ordinal/recipe consistency; terminal versus live-state
exclusivity. Validate representable shape and available consistency relationships,
not an unknowable proof that old effects occurred. Empty relative bytes denote root.

Reports are direct children: `root.txt` or
`dir-<lowercase hex of slash-joined native relative bytes>.txt`. Adopt a documented
255-byte specimen basename cap: nonroot relative bytes may be at most 123 bytes
(`4 + 2*n + 4 <= 255`). Reject longer directory routes before that entity's ensure
or descendant effects. This is an explicit Linux-fixture limit, not a portable
NAME_MAX claim; a filesystem with a smaller limit may fail normally. Directory
mapping is injective and separate from target paths. Root reservations are disjoint.

`bind_filesystem()` checks existing real source/target/reports roots and their
resolved disjointness without listing descendants. Recheck active routes during
each attempted operation. Those initial root checks are outside numeric budgets;
scheduled directory structural checks are part of charged assessment. Data-only
decode/build never calls binding. For terminal state `bind_filesystem` is a no-op
and drive returns Done without any tree I/O, even if roots have disappeared.
Cursor publication validates its path outside all three roots. Caller owns stable
nonhostile topology/concurrency; this is not a hostile archive/filesystem project.

## 5. Modules, optional transport and executable gates

Use a standalone nested `[workspace]`, own lockfile and example-local dependencies.
Suggested implementation map:

| File under `examples/walk/` | Responsibility |
| --- | --- |
| `src/lib.rs`, `src/build.rs` | Public exports, bon cursor builder and component compatibility |
| `src/state.rs` | Owned data, native coordinates, validation and descriptors |
| `src/driver.rs` | Request/ack, phases, retry/skip, folds and limits |
| `src/program.rs` | DesiredCopy lowering, concrete Manifest and report recipe |
| `src/io.rs` | FullInspector, ByteExecutor and filesystem binding/staging |
| `src/codec.rs` | Feature-gated Snapshot and separate cursor publication |
| `src/main.rs` | Fresh/resume commands, explicit limits and JSON progress |
| `tests/components.rs`, `tests/process.rs` | Construction/protocol and fresh-process gates |
| `README.md` | Runnable commands, semantics, counters and failure limits |

Default library features are empty. Bon and the path dependency on diff-dirs are
ordinary dependencies; tempfile/error support can be example-local. `codec` enables
optional facet (DTO `cfg_attr` derives), kopacetic-core and kopacetic-rkyv. `cli`
enables `codec` plus optional clap/serde/serde_json; bin uses
`required-features = ["cli"]`. JSON is CLI output, not the continuation format.
No optional transport dependency enters core's normal graph. Test codec-free
construction/restore with `--no-default-features` as well as feature-enabled tests.

Use actual `Snapshot::{from_value, from_bytes}` and reconstruction, following the
old codec's boundary but with **schema `diff-dirs.walk-example`, revision 1**, and
identity `EntityIdentity::standalone("WalkingCursor", "example")`. Do not reuse
the old schema. Bound archives at 8 MiB before decode and after encode; acknowledge
that this is not a projection/allocation bound. Validate reconstructed state before
binding. Publish a complete replacement cursor separately from effects/stdout;
publication failure supplies no rollback/recovery guarantee.

Implement four separate executable invocations for `source/d/a="one"`,
`source/d/b="two"`, empty existing target/reports and label A:

| Call | Assessments/effects | Required result |
| --- | --- | --- |
| 1 fresh | 2/2 | root assess, ensure d, d assess, a settled; d aggregate 1, root 0, ordinal 1; b pending |
| 2 resume | 0/1 | b settled; d aggregate 2, root 0, ordinal 1; d finish pending |
| 3 resume | 0/1 | `dir-64.txt` settled, d retired/folded; root aggregate 2, ordinal 2; root finish pending |
| 4 resume | 0/1 | `root.txt` settled; terminal leaves 2, complete-in-domain, ordinal 3 |

Tests inspect decoded state/coverage and actual bytes, not only stdout. Add focused
before/after-rename failure tests for b and d report (retry after visible install
does zero new installations), leaf/finish skip tests, duplicate/wrong ack tests,
effects-zero ready finish, `0/0`, failed-attempt charging and terminal no-I/O.
Test empty roots with new runs A → B → B (changed report then no-write settlement),
changed-label restore refusal, retained extras, shape conflicts, route limits,
equal-stat/different-bytes settlement, non-UTF-8 coordinates, corrupt/incompatible
Snapshot, unacknowledged-request restore, and codec-free saved-parts restore.
The component substitution test above is a separate essential acceptance gate.

Run specimen-focused nextest, clippy and formatting checks. No generic migration,
benchmark, broad suite or original-example changes are necessary. No unresolved
product decision blocks this implementation; the concrete APIs here are proposed
spellings, while the selected semantics and process boundaries are acceptance.
