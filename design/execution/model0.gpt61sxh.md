---
type: ExecutionModelDesign
title: Child-sized continuation with explicit finish obligations
description: A minimal ordered progression protocol, exact pause/failure states, retry and skip rules, component construction, and a tiny filesystem discriminator for effectful continuation.
resource: /design/execution/model0.gpt61sxh.md
status: proposed-with-bounded-prototype
generated: { by: "model:openai/gpt-6.1-sol#xhigh", at: 2026-10-20 }
sources:
  - resource: "opencode://ses_ea7dc862fffeEktnr59cgwuJHJ"
    title: Independently selected child-sized continuation, worked exact pause/retry/skip states, and tested owned requests plus a second stateful component on tiny ordinary filesystem fixtures
  - resource: /design/roadmap/brief0.gpt6a.md
    title: Independent commission, reading boundary, good-source assumption, and explicit absence of rollback, recovery, history and concurrency requirements
  - resource: /doc/architecture/README.md
    revision: 1193a1ee
    title: Returned architecture baseline and created-directory/two-installs/finish discriminator
  - resource: /design/architecture/synthesis0.gpt61sxh.md
    revision: 1193a1ee
    title: Prior-wave integration choices and execution-model research scope
  - resource: /sync/README.md
    title: Delivered recursive application, coupled replacements, staged resources, permission restoration and fresh-comparison retry
  - resource: /src/visitor.rs
    title: Public immediate inspection, finalized batches, immutable observations and legal actions
  - resource: /src/pair.rs
    title: Current paired-child projection and clean-plan release
  - resource: /src/walk.rs
    title: Difference-yielding traversal, frames, prefix aggregation and terminal failures
  - resource: /sync/src/apply.rs
    title: Current recursive edge split, staged leaf lifetime, directory finish and permission scopes
  - resource: /sync/src/lib.rs
    title: Existing comparison/application composition and whole-root entrypoints
  - resource: /examples/cursor/README.md
    title: Delivered assessment-only cursor, budget, retry, skip, output and codec boundaries
  - resource: /examples/cursor/src/cursor.rs
    title: Existing owned assessment frames, pure reduction and child-result folding
  - resource: /examples/cursor/src/codec.rs
    title: Existing Snapshot reconstruction and separately published cursor file
  - resource: /.test-agent/execution-model/README.md
    title: Ignored local prototype method and limits; not a portable committed implementation
extensions:
  ticket: diffdir-tree-execution-model
  reading_boundary: No withheld parent statement, full roadmap, parent transcript, or other current Wave 1 report was opened, searched, cited or evaluated; a broad tracker ownership diff incidentally exposed other-ticket return summaries after this design and its tests were written, and those summaries were not used; no subagents used
verification:
  method: Targeted source reading plus five new scratch-only Rust tests using public immediate inspection, tiny owned filesystem fixtures, JSON state roundtrips and reconstructed runtimes; scratch clippy and formatting; document checks
  result: Five tests passed, none skipped; scratch clippy with warnings denied passed; no production changes, existing test-suite reruns, JJ fixture operations or benchmarks
---

# Child-sized continuation with explicit finish obligations

## Selected recommendation

Use **one sequential, caller-built DFS driver owning every expanded recursive
edge**, with explicit **before → assess → children → finish** phases. Admit a
pause between joined, resource-closed effect units. An ordinary leaf install is
one such unit; a directory's effectful finish is another. Accept results and fold
them into the parent in one synchronous state transition. Persist active frames,
remaining children, aggregates, outstanding semantic intent and every component's
necessary data—not `DirectoryPlan`, live staging, or a completed-effect catalog.

Keep core's `inspect_directory` as the shared observation/decision seam. Initially
keep the driver and its program-specific state in the consumer/example. A future
sync adapter needs **nonrecursive units** for this mode; current public recursive
application is not that adapter. Do not change read-only `PlanWalk` into a universal
work scheduler, nor put a codec or executor into core just to demonstrate stepping.

**Opaque recursive application remains viable** when its return is the desired
pause granularity. It cannot satisfy “pause after the first visible child” merely
by wrapping its outer call. Retaining its internal child position and finish state
would be an alternative implementation of substantially the continuation below.

This is normal deliberate continuation, not a transaction: filesystem work and
cursor publication remain separate. No locking, concurrency protocol, rollback,
recovery guarantee, exactly-once external delivery, or effects history is required.
Assume the source is good and the caller owns stable routing and mutation policy.

## What the current mechanisms actually provide

| Mechanism | Useful seam | Why it is not already the proposed effect cursor |
| --- | --- | --- |
| `inspect_directory`, `src/visitor.rs:969–1130` | Finalizes one immediate batch, without writes/descent; callers supply actual physical sides separately from logical coordinates. Handles are dropped before publication. | Supplies transient facts, not a serializable work authority or a recursive driver. The caller validates real routing ancestors. |
| `Roots::inspect_visitor`, `src/pair.rs:67–104` | Lowers finalized paired children, counts and baseline difference. | Drops a clean plan even if steering ran. An extended program must consume the immediate plan directly and retain explicit finish/work obligations even when `diff()` is empty. |
| `PlanWalk::advance`, `src/walk.rs:175–211` | Ordered paired-directory traversal; frames keep pending children and changed-subtree state. | `next()` may assess many clean directories before yielding. One-sided interiors are opaque; failure sets a terminal failed summary (`224–237`). No retryable effect phase exists. |
| `Context::execute_entry`, `sync/src/apply.rs:490–572` | Existing legal-action execution and the current recursive ownership split. | Paired descent can stay with the walker; source-only copy, removal and replacements recurse inside sync. Those stack frames are not returned continuation data. |
| Directory operations, `sync/src/apply.rs:589–669` | Legal ancestor creation, recursive child processing, retained-container reduction and postorder permission work. | A single call can create a directory, install many children, then fail. A joined return reports partial progress, not the remaining child/finish position. |
| Cursor example, `examples/cursor/src/cursor.rs:127–182,318–413` | Owned frames, one attempted-directory budget, pending assessment retry, subtree skip and postorder folding. | Its two phases and completion-time generation are sufficient only because assessment has no effectful before/finish work. Its descriptor has parameters but no independently changing component state. |

The delivered sync composition in `sync/src/lib.rs:309–348` alternates divergent
paired plans and application. It must continue to have its present meaning. The
proposed expanded driver instead owns paired **and** one-sided edges itself; it
must not also let `Context` recurse through those same edges.

## Smallest progression state

The following shapes are schematic Rust, not a proposed public generic list.
`Node`, `Local`, `Aggregate`, `FinishIntent` and `Work` belong to the selected
program. The control shape can be reused without a trait for each noun.

```rust
enum Phase { Before, Assess, Children, Finish }

struct Frame {
    node: Node,                    // identity, binding, parameters, declared goal
    phase: Phase,
    remaining: VecDeque<Node>,     // active child stays at the front until closed
    local: Option<Local>,          // accepted immediate assessment, at most once
    children: Aggregate,           // only closed child outcomes
    finish: FinishIntent,          // obligation, not proof it ran
    incomplete: bool,
}

struct Cursor {
    stack: Vec<Frame>,
    pending: Option<PendingUnit>,  // one semantic unit, ticket, attempt, phase
    blocked: Option<Block>,
    final_value: Option<EntityOutcome>,
    next_ticket: u64,
}

struct Saved {
    bindings: Bindings,
    program: ProgramDescriptor,    // compiled interpretation + frozen parameters
    cursor: Cursor,
    components: ComponentsState,  // e.g. mutable manifest ordinal, not its writer
}
```

The leaf case can be a frame whose only before unit installs/settles the leaf and
whose remaining phases are empty. The scratch makes it an explicit `Leaf` phase
instead; either representation is small. Do not require a directory assessment
for a leaf just to make every entity look identical.

### Transition rules

1. **Before:** settle prerequisite local work, such as ensuring an absent real
   container. A failed before unit admits no descendants. Success advances the
   phase; it does not close the entity or fold it into its parent.
2. **Assess:** attempt one immediate-directory inspection and lower its complete
   validated batch to owned local data and ordered remaining children. Install
   that data only on success. Failure retains the unassessed frame, with no partial
   local counts/children accepted. A child batch may fail after its legal container
   was created, just as current sync permits.
3. **Children:** push the front child without removing the parent's edge. Only
   closure of that active child removes the edge and folds its result. No other
   component independently discovers or executes that recursive edge.
4. **Finish:** derive pure output or a selected effectful obligation from the
   accepted local/child data. An effectful finish must be acknowledged or explicitly
   abandoned incomplete before closure. It is never hidden in a free stack pop.
5. **Close:** compute and check the candidate parent aggregate/component update;
   then retire the child frame, remove its parent edge and install that aggregate
   together. At the root, install the terminal outcome. There is no supported
   pause in the middle of this transition.

Each active parent below the top has exactly its front remaining child active
above it. A removed child descriptor need not be retained: its contribution is
already in the aggregate. Keeping a full sorted list plus an index is also viable,
but retaining only the unclosed suffix better demonstrates that no history is
necessary. Neither layout proves any previous physical effect still exists.

An accepted local assessment is installed once, not added again on every retry.
An accepted child result is folded once when its frame is retired, **not** when
the executor returns, then again on parent finish. “Once” means within this
advancing state lineage. Loading an older snapshot or forking it can repeat work
and outputs; no durable deduplication promise is inferred.

## The discriminator, fully worked

Use existing real roots `source/`, `target/` and separate `finish-output/`:

```text
source/d/a = "one"       target/ initially empty
source/d/b = "two"       finish-output/ initially empty
```

The program copies the two desired leaves into a created `target/d`. Its directory
finish installs a summary at `finish-output/d.txt`. Root finish similarly writes
`finish-output/root.txt`. Separate output keeps summaries out of the source/target
union; this is a demo binding, not a general output-location policy.

The manifest component has durable state `{label, next_ordinal: 1}`. It reserves
the current ordinal by deriving a finish recipe, but advances that ordinal **only
when finish is accepted**. A retry generates the same desired bytes. A second
directory completion therefore exposes whether this state survived reconstruction.
This is an accepted-completion ordinal, not a globally unique external-output
number: skipping an uncertain visible finish can leave an unaccepted artifact
whose ordinal is reused by the next accepted completion.

### Exact paused states

| Boundary | Physical state | Driver state | Manifest component |
| --- | --- | --- | --- |
| Before creation | `target/d` absent. | Root assessed; remaining `[d]`. Active `d` is `Before`, pending ensure-container. | Label retained; next ordinal 1. |
| **After `a` is visibly installed and accepted**; before starting `b` | Real `target/d`, `a="one"`; no `b`, no summary. No active staging/permission lease. | Root: `Children`, remaining `[d]`, child aggregate 0. `d`: `Children`, remaining `[b]`, accepted child aggregate 1, local assessment retained, finish outstanding. Optionally `b` already has a top leaf frame and issued owned install intent; it contributes nothing yet. `a`'s frame/edge are gone. | Next ordinal still 1; no completion consumed. |
| **Before finish**, after accepting `b` | Both leaves visible; no `d.txt`. | Root still has unclosed `[d]`, aggregate 0. `d`: `Finish`, remaining empty, child aggregate 2, finish recipe pending. Both leaf frames are gone. | Pending recipe uses ordinal 1 and the retained label; next ordinal still 1. |
| After accepting `d` finish | Both leaves and `d.txt` visible. | `d` retired; root remaining empty, child aggregate 2; root finish outstanding. No stored finished `d` record is needed. | Next ordinal 2. |
| After accepting root finish | Both summaries and leaves visible. | Stack empty; terminal `{leaves: 2, incomplete: false}`. | Next ordinal 3. |

These are **logical acceptance states**, not receipts proving that the described
physical state survives outside modification. The table names actual filesystem
state at that boundary under the fixture's stable-input assumption.

### Fail after one visible install

There are two importantly different failures:

- `a` was accepted, and attempting `b` fails before installing it: retain the
  after-`a` state, plus a block identifying unresolved `b` and its attempt error.
- `a` was accepted, and `b` installs but its executor returns an error afterward:
  retain **the same logical state**. The target may already contain `b`, but no
  successful `b` result was accepted. Do not fold a guessed success from partial
  namespace state or a generic `Failure.completed` counter.

Retry keeps `a`'s aggregate and recomputes **`b`'s unresolved goal against current
filesystem facts**. If current satisfaction is established, accept the goal without
reinstalling; otherwise perform freshly selected legal work. Then fold `b` once.
This needs neither a retained successful-effect list nor “the failed call did
nothing.” It does require a satisfaction/retry meaning for this operation.

### Fail before finish, or after its effect becomes visible

Retain the prefinish frame: aggregate 2, empty remaining children, unresolved
finish, root aggregate 0, next ordinal 1. A failure after `d.txt` became visible
has exactly the same unaccepted logical state as failure before writing it.

Retry reconstructs ordinal-1 desired summary from retained inputs and observes
the current output. If already satisfied, settle without another write; otherwise
retry the selected installation. Only accepted finish advances the ordinal,
retires `d` and folds 2 into root. Never rediscover/reinstall `a` and `b` merely
because finish failed. A finish needing a fresh tree scan must request that scan
explicitly; aggregate completion is not such a scan.

Not every caller finish has an observable satisfaction predicate. A subprocess
or external publication may require an explicitly repeatable attempt, or an
operator choice to retry/skip. No generic cursor can turn an arbitrary command
into exactly-once work. Keep that fork visible rather than inventing receipts.

## Retry is not stale-plan replay

Persist the **goal and binding**, not old `Metadata` or the last physical `Action`.
In particular:

- An unresolved `EnsureDirectory` can find a directory created by its failed
  attempt. Re-establish its route and settle the prerequisite; do not replay
  `create_dir` and fail forever on `AlreadyExists`. Finding a leaf does not grant
  new replacement authority.
- After creating `d`, later reinspection sees a present target directory. It
  must not erase the outstanding copied-directory finish merely because a new
  baseline action would now say `Container::Keep`.
- An unresolved leaf install re-observes source/target facts and satisfaction.
  An equal current leaf can settle the pending goal; a changed physical shape
  must be revalidated/refused under the binding, not handled by a stale action.
- If directory discovery itself failed, retry that immediate assessment in full;
  no local result was accepted. If execution failed after accepted discovery,
  refresh execution authority as needed without re-adding local observations or
  rebuilding the accepted child-result prefix.

The scratch has a deliberately narrow desired-tree program, no arbitrary visitor,
no target-only deletion and fixture-only byte satisfaction. Direct current
source/target checks suffice for its leaf retry. **A general adapter preserving
today's arbitrary directory-batched `Visitor` cannot assume per-leaf observation
suffices.** It may need to call `inspect_directory` on the current containing batch,
re-finalize policy, then select only the unresolved scheduled unit. Such a refresh
does not fold the refreshed whole batch into the logical aggregate. If a refreshed
decision retains a name, its outcome is retained/policy coverage—not a fictional
installation. Programs needing a fixed desired goal must still verify that goal.

Public `apply_plan` explicitly re-finalizes and compares action/presence signatures
(`sync/src/apply.rs:93–109,130–146`). It can reject a formerly absent entry now
present after partial execution. Therefore wrapping the old plan in a saved cursor
and replaying `apply_plan` is **not** the proposed retry adapter. Existing delivered
sync still recommends fresh comparison (`sync/README.md:400–406,600–607`). The
new program's narrower unresolved-goal retry is proposed behavior, not a retrofit
claim about that API.

Resumption does not relist assessed ancestors or discover arbitrary new siblings.
It validates necessary current routes and unresolved execution inputs; that is
different from revisiting every completed subtree. If the caller wants a new full
assessment after edits, start one. Snapshot decoding validates data; it does not
make filesystem facts fresh or authorize a new root/routing/policy binding.

## Skip has an explicit abandoned scope

Minimum useful API: `skip_blocked_unit()` acts on the single blocked top unit.
It does not mean “whatever part failed internally.”

| Blocked phase | Abandoned scope and retained result |
| --- | --- |
| Before container or failed initial assessment | Close that scheduled entity/subtree incomplete. No unaccepted local assessment is added; no dependent child effects start. Already visible effects, including a partly created container, remain. |
| Leaf `b` | Close `b` as abandoned/incomplete, contributing no successful leaf settlement. Retain accepted `a`. Mark **every active ancestor incomplete immediately**, not only when `d` eventually closes. A visible unaccepted `b` is not undone or relabeled completed. |
| Directory finish | Abandon that finish, preserve the directory's accepted local/child aggregate, and close it incomplete. For the example this folds 2 into root, advances no manifest ordinal, and leaves `d.txt` absent or in whatever state the failed attempt produced. |

After skipped `b`, `d` has one accepted leaf result and incomplete coverage even
if `b` happens to exist. Its finish can emit an explicitly **partial** summary if
the program allows that. A native “ready” finish requiring complete descendants
must instead be omitted/abandoned incomplete or refuse; it cannot silently mark
the directory ready. Put this requirement in the finish program, not a universal
“all finishes run after skip” rule.

A broader `abandon_remaining(frame_id)` can later close a whole active directory,
keeping its accepted aggregate and abandoning its unfinished children/finish.
It is distinct from skipping one failed child and is unnecessary for the first
demo. Neither operation removes prior installs or prunes the filesystem.

Runtime housekeeping is not skippable work: join started jobs, close descriptors,
attempt owned staging cleanup and permission restoration before returning the
attempt. A cleanup/restoration failure is reported; no repair/recovery guarantee
is created by retaining the logical pending unit. Normal voluntary pauses occur
only after successful scope shutdown, not while a caller must retain a lease.

## Effects can be yielded without serializing resources

Use **owned semantic intents** plus one outstanding ticket. The ticket identifies
the pending phase/attempt in this lineage; it is not an idempotency key, effects
receipt, secret capability or cross-process delivery guarantee.

```rust
enum Request<W> {
    Inspect { ticket: Ticket, tip: Tip, role: InspectRole },
    Effect  { ticket: Ticket, intent: W },
    Done(EntityOutcome),
}

// The host runs all actual I/O outside driver/reducer acceptance.
fn execute(intent: &Work, runtime: &mut Runtime) -> Result<Settled, Issue>;
fn acknowledge(ticket: Ticket, result: Result<Settled, Issue>,
               saved: &mut Saved) -> Result<(), BadAcknowledgment>;
```

An install intent can be `{logical, physical_source, physical_target, parameters,
goal}`; a finish intent can carry a recipe and aggregate input. It contains no
borrow into `DirectoryPlan`, open file, `TempDir`, permission guard or closure.
Issue the request **before** staging starts. Executor acquires those resources,
performs and joins its unit, settles scopes, and only then returns an acknowledgment.
A separate output consumer can be that executor. No caller effect needs to occur
inside `Visitor` or a pure completion reducer.

On `Err`, retain the phase and accepted aggregates; only mark the unit blocked.
Retry invalidates the failed attempt and prepares current work again. Reject a
success reply for the wrong ticket/phase, or a duplicate reply after the pending
unit was consumed. Do not maintain an ever-growing consumed-ticket set. Two copies
of a snapshot can still issue the same logical work; preventing that is outside
the requirements. A saved unacknowledged request is unresolved, even if a caller
knows it might have executed. Re-examination/retry is necessary, not automatic
acceptance of its old result.

Reducers and mutable components must not partly advance before a fallible fold
fails. One implementable initial technique is to build a checked candidate of the
affected driver/component state and replace it only after validation. Scratch
`Saved::acknowledge` uses a candidate clone. An optimized implementation can
precheck arithmetic and replace only the affected records. This is ordinary
in-memory transition discipline, **not** a filesystem/cursor-publication transaction.

The delivered applier currently shares an arena and writable lease across a whole
batch (`sync/src/apply.rs:434–463,1302–1433`) and holds source/output leaf descriptors
through installation (`360–366,802–843`). Those are good runtime lifetimes, not
serializable fields. To pause per child, either finish/reopen such scopes at each
supported boundary, or retain them only during an uninterrupted invocation and
close them explicitly on pause. A suspended Rust stack or serialized staging path
is not a substitute for that shutdown.

Copied-directory permissions deserve special care. The proposed driver must not
leave a temporary permission lease open until logical directory finish. A simple
adapter establishes the intended copied-directory mode at each pause boundary,
and each later child unit relaxes/restores it as needed. The final logical finish
can still have independent summary/native work. Existing target modes similarly
use per-unit or invocation-scoped restoration. This can cost more chmod/staging
work than today's whole batch. If a target mode/topology cannot support the chosen
boundary, use a coarser opaque unit or refuse that pause mode; do not serialize a
live guard. The scratch does **not** test this full permission adaptation.

## Assessment budget, effect units and completion are different

Offer separate limits, for example `{assessments, effects}`; zero is meaningful
when requesting “inspect but do not execute” or “drain ready transitions only.”

- One **assessment** is one attempted immediate-directory inspection, including
  clean, empty and failed attempts. Consume before calling the inspector. A
  successful accepted expansion installs local data once.
- One **effect unit** is one issued resource-closed attempt: ensure a container,
  settle one leaf, or run one finish. A failed attempt consumes a unit too. A
  satisfaction no-op can settle an effect obligation without a namespace write;
  units are not syscall counters.
- Pure child push, aggregate fold and closure can drain without another assessment
  or effect. An effectful finish **cannot** drain just because children are done.
  One final effect acceptance may permit multiple pure ancestor closures.
- Neither cap bounds entries in a listing, byte size of one leaf, syscalls,
  memory, output events or elapsed time. A single leaf or native command can stall.
  There is no newly promised cancellation latency or byte-level checkpoint.

Execution revalidation must not hide extra immediate inspections. If the adapter
needs a complete fresh batch, request an `InspectRole::RefreshPending` through
the host and charge another assessment attempt; keep its transient plan in the
runtime, not the saved cursor. Refresh changes no accepted local aggregate. A
normal pause may drop this cache, so reconstruction can need another refresh.
The host must be able to stop **before** that inspection when the assessment cap
is exhausted, leaving the semantic effect pending. Any prepared executable grant
is invocation-local and is rebuilt after reconstruction; a serialized effect
intent alone never bypasses fresh authorization.

The narrow prototype has discovery inspections plus direct leaf/output checks,
so it does not implement that general visitor-refresh adapter or numeric limits.
Those are specified next-demo work, not tested budget behavior. This distinction
also avoids pretending `PlanWalk::next()` is a one-assessment step.

Opaque recursive apply consumes one **opaque effect unit**, but may contain any
number of assessments/child effects internally. It cannot claim this bounded
assessment protocol unless it also exposes the internal requests. A demo must
label that granularity rather than count the outer call as a directory step.

## A practical component-construction boundary

Use one application `Program` grouping pure lowering, child folding, finish
derivation and state validation. It can be a concrete struct/enum or one trait;
no handler registry or trait per row of the architecture table is required.
Bind a concrete executor or ordinary `match Work` callback outside it.

```rust
// Descriptive boundary, not exact final builder spelling.
let saved = Saved::start(bindings, program.descriptor(), component_state)?;
let running = Running::bind(saved, program, inspector, executor, runtime)?;

// At a quiescent pause: drop runtime resources, retain only owned data.
let saved = running.into_saved()?;

// A different process supplies executable components again.
let running = Running::bind(decoded_saved, program, inspector, executor,
                            freshly_constructed_runtime)?;
```

`Running` need not retain an immutable inspector callback if the host loop calls
the public function directly. A codec accepts only `Saved`; executable components
are supplied by the application. `bind` validates descriptor/schema compatibility,
native coordinates, saved root/route shape, frame/edge/phase invariants and every
component's state. Keep this restored-data validation filesystem-free; a terminal
cursor can remain terminal without consulting either input tree. The initial
binding's structural filesystem validation and active execution's current-route
checks are separate from decoding and from per-attempt assessment accounting.
This does not require a general serialization trait or a filesystem scan in every
component constructor.

### The second stateful component is the decisive construction test

The driver alone already has mutable frames. Adding a handler ID and label does
not prove arbitrary component construction, because that handler can remain
stateless. Use the manifest component described above:

| Persisted data | Rebound implementation/runtime |
| --- | --- |
| Driver stack, remaining edges, aggregate and unresolved phase/ticket | Control code, borrowed pool if used; no started job at a pause |
| Manifest descriptor/label and **next accepted ordinal** | Pure manifest rendering code |
| Pending finish recipe inputs; no previous generated bodies | Output installer/sink and its roots/handles |
| Logical/physical bindings, policy/parameter compatibility data | Current inspector, filesystem access, invocation-local fault injection |

After `d` finishes, save `{next_ordinal: 2}` and root aggregate 2; discard and rebuild
the runtime; root must emit ordinal 2 with the original label. If the counter lives
only inside a runtime generator object, it resets to 1 and the construction is
wrong. If it is reconstructed by replaying previous completion outputs, the design
has introduced unnecessary history. Mutable component state can live in
`Saved.components`, or in an owned typed component object inside `Saved`; that
placement is not the important distinction. All behaviorally necessary data must
have one explicit saved owner. A component that cannot expose/reconstruct such
state must prohibit the affected pause boundary.

This component need not implement its own snapshot trait. Initially a concrete
`ComponentsState { manifest: ManifestState }` is sufficient. A second program can
choose a different typed saved state. Generalized heterogeneous registration and
dynamic component construction are downstream forks, not prerequisites.

## Recursive ownership and alternatives

| Design | Recursive owner / pause contract | Assessment |
| --- | --- | --- |
| **Selected: caller DFS plus child-sized units** | One driver owns every expanded edge; sync/external executor never descends that edge. Leaf and finish return independently. | Meets the discriminator and component construction. Needs nonrecursive effect seams or a narrow demo installer. More scope reopen/revalidation work is possible. |
| Opaque current apply | Paired `PlanWalk` owns matched-directory edges; sync owns one-sided/replacement interiors. Pause only after the apply returns and joins. | Smallest production reuse, valid when large pause units are acceptable. Does not expose voluntary intermediate child pauses. Retry is fresh comparison, not a saved inner position. |
| Sync-owned explicit continuation | Sync replaces recursive stack locals with owned frames and internally drives child units, optionally exposing typed work requests. | Equally viable if effectful stepping belongs primarily to sync. It must still represent pending finish, aggregates, component state and cleanup boundaries; merely hiding those fields does not remove them. |
| Universal core scheduler / trait-per-component / action registry | Core drives arbitrary operations and runtime resources. | Not selected. There is not yet evidence that these abstractions are the smallest reusable seam. |

Coupled type replacement is an important limit. Today's leaf-to-directory action
prepares the entire subtree before removing the old leaf
(`sync/src/apply.rs:532–558`); directory-to-leaf prechecks retained descendants
(`963–991`). The initial child-sized demo should keep these **opaque**, or reject
them explicitly. Fine-grained continuation inside private replacement preparation
would need declared private-artifact ownership/disposal semantics. It is not
just “treat the directory side as another ordinary child,” and is not needed to
represent the ordinary created-directory example. Do not weaken existing coupled
authority or intact-refusal guarantees to simplify the demo.

## Bounded experiment: observed, not a production implementation

Ignored scratch lives at `.test-agent/execution-model/` with its own README and
nested Cargo workspace. Its `src/lib.rs` has:

- owned `Cursor`/`Saved` and separate `ManifestState` at lines 62–100;
- polling, checked candidate acceptance, one-edge folding and retry/skip at
  lines 128–239;
- an independently constructed `Runtime`, public `inspect_directory` discovery,
  current byte-satisfaction checks and tiny staged writes at lines 242–347;
- five focused tests at lines 404–520.

Fixtures are tiny ordinary directories under the approved `/tmp/opencode/`,
outside live repositories. No JJ commands operate on them. Tests JSON-roundtrip
both driver and manifest data, then construct a fresh runtime. They passed:

1. Pause after accepting `a`: `d/a` exists, `b`/summary do not; root has not folded
   `d`; reconstructed execution does not reinstall `a`.
2. Fail **after `b` is visible**: retain one accepted leaf; retry observes current
   `b`, performs zero new namespace writes, folds it once and rejects a duplicate
   acknowledgment.
3. Pause **before `d` finish**, then fail after its summary is visible: ordinal
   remains 1 and root aggregate remains 0 until successful retry; retry does not
   rewrite the summary; a later reconstruction gives root ordinal 2 and retained
   label, with total 2 rather than 4.
4. Skip the uncertain `b` unit: both visible files remain, but accepted count is
   only 1 and all ancestors are incomplete; the permitted partial finish says so.
5. Skip failed finish: accepted child count 2 survives, `d` summary is not written,
   ordinal is not consumed, and root gets incomplete coverage.

Commands run:

```sh
cargo nextest run --offline --manifest-path .test-agent/execution-model/Cargo.toml
cargo fmt --manifest-path .test-agent/execution-model/Cargo.toml
cargo clippy --offline --manifest-path .test-agent/execution-model/Cargo.toml --all-targets -- -D warnings
```

Result: **5 passed, 0 skipped**; scratch formatting and warnings-denied clippy
passed. No existing suite was rerun. Reported earlier ancestor-preservation and
cursor/Snapshot evidence was trusted within its scope, not reverified here.

Limits: fixture-only UTF-8 paths/JSON state transport; no hostile-data validator,
generic `Visitor` refresh adapter, numeric budget implementation, sync metadata/
permission/clone backend, failure cleanup campaign, platform portability test,
performance measurement or crash test. Byte satisfaction belongs only to these
fixture operations; it does not change core's type/size/mtime predicate. Scratch
is deliberately uncommitted and may not exist in another checkout. The committed
design above, not the scratch path, is the durable contract proposal.

## Implementable next demo and open forks

Build the next `walk-demo`/builder slice around this exact fixture, not another
large framework:

1. Construct the driver from real bindings, immediate inspection, a concrete
   desired-tree program, a child-sized installer and the stateful manifest
   component. Preserve ordinary pure visitor behavior; no effects in callbacks.
2. Expose before/assessment/child/finish gates and separate assessment/effect
   limits. Show `effects=0` stopping before finish even after children are done.
3. Save at the two named boundaries, reconstruct executable components/runtime,
   retry failures before and after visible leaf/finish effects, and skip each
   blocked scope. Check aggregates, incomplete coverage and the second component's
   ordinal—not only final files.
4. Use an enum of the few concrete work variants. Generated finish output goes
   through an owned intent and external acknowledgment; no global handler registry.
5. For a narrow ordinary-copy demo, state its desired-goal/satisfaction semantics
   explicitly. For a general existing-`Visitor` demo, also implement budgeted fresh
   batch revalidation and mask already accepted schedule positions. Do not claim
   that dropping old `Metadata` alone preserves arbitrary visitor semantics.
6. Keep coupled replacements opaque/refused. Reuse sync's installer only after
   factoring an actually nonrecursive, resource-closed seam; public recursive
   `apply_plan` is not a shortcut to child-sized stepping.

Still viable/unsettled: whether the reusable driver ultimately lives in the
consumer or sync; typed concrete state versus a later generic `Program` interface;
per-child scope shutdown versus invocation-local batching; which caller finishes
permit partial descendants and which are safely retryable; the exact general
visitor refresh/lowering contract. None requires a universal scheduler or effects
journal, and none prevents the stated minimal demo from being implemented now.

The [maintained architecture](/doc/architecture/README.md#component-built-continuation-and-pause)
supplies the broader ownership constraints; this document selects a finer normal
pause mechanism for its execution discriminator. The
[assessment cursor](/examples/cursor/README.md#failure-retry-and-skip) is a precedent
for pending-tip state, but its pure completion path is deliberately not treated as
effectful finish. The [sync specification](/sync/README.md#replacement-permissions-and-metadata)
is the resource/authority constraint an actual sync adapter must preserve. These
connections are extensions and limits, not claims that the proposed API is delivered.
