---
type: DesignProposal
title: "Inline inspection policy across comparison and application"
description: A shared inspection gate for diff-dirs and recursive sync, command-scoped frozen selection proofs in jj-waltz, explicit refusal timing, and alternatives for strict preflight, pull control, and shared-source execution.
resource: /design/fit/policy0.gpt6a.md
status: proposed
generated: { by: "model:openai/gpt-6-astra", at: 2026-10-18 }
sources:
  - resource: "opencode://ses_eb262a81effea1HgF0pPS3f1sy"
    title: Independently designed the shared inspection gate and caller integration after tracing both repositories; compared alternatives and failure semantics
  - resource: /design/fit/policy0.glm53m.md
    title: Commissioned starting inquiry and prefilter alternative
  - resource: /design/fit/handoff0.glm53m.md
    title: Recursive guard obligations and preservation of payload through tracked refill
  - resource: /README.md
    title: Current opaque-directory, stat-comparison, pull-loop, frontier, and coverage contracts
  - resource: /sync/README.md
    title: Recursive application, staged transitions, partial progress, and stable-tree requirements
  - resource: /src/pair.rs
    title: Immediate inspection discards metadata and leaves one-sided directories opaque
  - resource: /src/frontier.rs
    title: Worker completion admits descendants of speculative clean results before ordered commitment
  - resource: /sync/src/apply.rs
    title: Independent recursive copy and removal paths that must enforce the same policy
  - resource: /design/performance/comparison0.gpt6a.md
    title: Metadata handoff opportunities and selected future streaming multi-destination direction
  - resource: "file:///home/rektide/src/jj-waltz/src/reseed/engine.rs"
    title: Current preflight, repeated rule construction, tracked refill, and destructive payload replacement
  - resource: "file:///home/rektide/src/jj-waltz/src/cow.rs"
    title: Existing administrative, mount, special-entry, and symlink-warning checks
  - resource: "file:///home/rektide/src/jj-waltz/src/cow/payload.rs"
    title: Frozen ignore tree construction and per-query native Git children
  - resource: "file:///home/rektide/src/jj-waltz/src/cow/ignored.rs"
    title: Adaptive outermost-root selection and tracked-overlap proof
  - resource: "file:///home/rektide/src/jj-waltz/docs/reseed.md#safety-and-recovery"
    title: Current batch readiness, advisory quiescence, and journal-free recovery promises
  - resource: "https://github.com/git/git/blob/master/Documentation/git-check-ignore.adoc"
    title: Read in the local Git archive; documented streaming non-matching output, NUL framing, negation, and buffering
---

# Inline inspection policy across comparison and application

## Recommendation

**Share inspection with the operation that already needs it. Freeze selection
facts once per command. Keep the public diff small.**

I recommend a directory-local **inspection gate**, used by both the diff walker
and sync's recursive operations. It sees every immediate entry, including clean
ones, and can preserve a name or refuse it before descent or mutation. The gate
edits a restricted decision view; the library derives the diff and descent list
from that view. It does not freely rewrite a `DirDiff`.

On the jj-waltz side, separately create one immutable **selection context** for
the command: frozen ignore inputs, tracked membership, selected roots, and target
scope. Native ignore queries become batched or session-based. Payload interiors
then use the shared gate rather than independent validation traversals.

The important extension beyond a walker hook is **guarded recursive copy and
removal**. Without it, the most dangerous paths still bypass the policy.

This is a proposal for deliberate contract changes. No implementation or new
speed measurement accompanies it. The [current README](../../README.md) remains
the specification. Companion: [jj-waltz integration and landing map](file:///home/rektide/src/jj-waltz/design/up-performance/inline-policy0.gpt6a.md).

### Terms

- **Selection context:** command-lifetime facts establishing which roots may be
  operated on and under which frozen repository rules.
- **Observation:** short-lived filesystem facts for one directory or entry.
- **Gate:** deterministic decision over an observation, before it authorizes
  traversal or application.
- **Preserve:** leave this relative name and its descendants untouched on both
  sides; do not mistake exclusion for absence.
- **Refusal:** fail the operation at the discovered violation. Earlier permitted
  work may already have completed.

## 1. What the current code actually requires

Inspected bases: diff-dirs `5cf20bb1ca8125e0350205b742caeca2de14ec00`;
jj-waltz `324606bed3b562d866d57b0adc449f90c355c37c`. These are source-review
coordinates, not claims about future integration branches.

| Current seam | Consequence for this design |
| --- | --- |
| `pair.rs::inspect_entry` collects metadata, reduces it to comparison keys, then drops it. | Retain a directory-local observation until the gate runs; do not collect the same metadata in a callback. |
| `pair.rs::one_sided` reports a directory as one opaque name. | A walk-only gate cannot inspect `new-package/deep/.git` or `obsolete-package/deep/mount`. |
| `sync::Context::{copy_directory_with,remove}` recursively visits those opaque units. | Put the same inspection contract here; initialization and type transitions must use it too. |
| `frontier.rs::children` permits descendants of a ready, clean speculative node. | A filter run only at ordered commitment is too late to prevent speculative descent. |
| `cow::validate_same_mount` repeats parent/child metadata and Linux mount-ID calls. | Sharing ordinary stat data helps, but mount IDs require an explicit inspection capability. |
| `engine.rs::validate_source` reconstructs ignore rules repeatedly. | Reusing frozen semantic facts is a different optimization from combining filesystem walks. |
| `execute` removes selected roots both before and during final payload copying. | Preserve those roots through tracked refill before expecting incremental sync to help. |

Three corrections to tempting assumptions:

1. **Gathering existing `DirDiff`s is not a full validation pass.** Opaque
   one-sided directories still hide descendants. A strict gather design needs an
   explicit recursive inspection path for them.
2. **Mount checks are not zero additional I/O today.** The current walker has no
   mount-ID observation. `MetadataExt::dev()` does not detect same-device Linux
   bind mounts, including mounted regular files.
3. **“The reducer is ordered” does not mean “the callback runs on the caller.”**
   Today workers call `complete` and `pump` under the frontier mutex. Inserting an
   arbitrary `FnMut` there changes threading/reentrancy constraints and can block
   the scheduler. A caller-thread callback needs a different barrier.

## 2. Separate two lifetimes

### Command lifetime: selection context in jj-waltz

Keep repository meaning here:

- exact source and desired tree coordinates, tracked entries, and root identities;
- effective selection patterns and outermost selected roots;
- frozen tracked `.gitignore` contents plus effective backend/global excludes;
- root eligibility under both source and desired policies, tracked disjointness,
  and configured-link disjointness;
- the paths that tracked clearing/materialization can actually touch.

Share equal contexts across targets. Source policy and desired-target policy are
not necessarily equal: `--at` may choose a different tree, and effective excludes
must be included in the context identity. Sharing only by commit ID is insufficient.

This context is not an immutable payload snapshot. Equal JJ commits say nothing
about ignored bytes. Retain the current paused-writer requirement and cheap
identity/revision checks at mutation boundaries. Reuse ends with this invocation;
no receipt, persistent catalog, or directory-mtime cache is required.

### Directory lifetime: observations in diff-dirs

The comparison and application kernels own short-lived observations:

```rust,ignore
// Illustrative responsibilities, not a final public signature.
struct EntryFacts<'a> {
    name: &'a OsStr,
    kind: EntryKind,                    // includes unsupported kinds for refusal
    metadata: &'a Metadata,
    link_target: Option<&'a OsStr>,     // literal; never followed
    mount: Option<MountId>,            // present only when requested/available
}

struct NamePair<'a> {
    original: Option<EntryFacts<'a>>,
    target: Option<EntryFacts<'a>>,
}
```

No content hashing. No whole-tree observation store. Avoid eager full-path
allocation where relative directory + name suffice. Release observations after
the directory decision unless a private same-iteration sync path consumes them.

Initially retain ordinary applier validation of public records. A later private
prepared-application path may reuse observations, but **a stat-free public record
is not proof that a gate ran**. Keep root binding and record validation intact.

## 3. The gate: a constrained prefilter

Use one closure at a directory boundary, rather than a public rule language or a
callback for every syscall:

```rust,ignore
fn inspect_policy(view: &mut Inspection<'_>) -> Result<(), PolicyError> {
    for entry in view.entries() {
        // All immediate names: equal leaves and paired clean dirs included.
        // Read facts; record decisions through stable entry identifiers.
    }
    // view.preserve(id, reason);
    // view.refuse(id, side, reason)?;
    Ok(())
}
```

`Inspection` provides a root-relative directory, side-specific parent facts, an
operation purpose, immutable sorted name pairs, and mutable decision slots.
Implementation can separate the facts and decision slices to make Rust borrowing
straightforward. Entry identifiers are frame-local indices, not arbitrary paths.

**Allowed edits are monotone:** process normally, preserve an entire name pair,
or refuse. The gate cannot invent names, change observed types, relabel a removal
as an addition, or independently erase half a file/directory transition.

After the gate, one lowering step derives both `DirDiff` and paired-child descent.
This avoids maintaining two editable lists that can disagree. A clean directory
still runs the gate. Directory roots receive an equivalent root inspection before
opening them; their names and boundary checks cannot be skipped accidentally.

The default path has no gate and retains today's behavior. A guarded constructor
can carry observation requirements and the closure without making the existing
small, execution-only `WalkOptions` a generic policy container.

### Preservation and refusal are different

- **Preserve is pair-wide.** Hiding an original-only entry is not a request to
  delete a matching target. Preserve suppresses comparison, mutation, and descent
  for that relative name on both sides, regardless of type.
- **Refuse checks actual present sides.** A `.git` on either side fails even if it
  is byte-identical and would otherwise emit no diff.
- Preserving a subtree makes no assertion about its descendants. It is an
  exclusion, not a recursive validation certificate.
- An uninspectable immediate entry can still fail before the gate: the first
  version promises no descent below preserved entries, not zero stat calls on
  their root names. A pre-stat name filter is a separate, measurable extension.

jj-waltz's payload gate should **refuse**, not preserve, nested `.git`/`.jj`,
unsupported special entries, and mounted boundaries. At the workspace orchestration
level it deliberately leaves `.jj` and authorized donor `.git` to their owners.
Those are different scopes; do not install a universal “ignore admin names” rule.

Keep warnings, such as absolute dependency symlinks, in a directory-local result.
Do not print from a speculative callback. Commit them in deterministic order;
application failures can report warnings for work actually encountered.

## 4. Cover recursive actions, not just comparison

The gate belongs at a shared **inspection seam** in the read-only crate, callable
by the sync crate. Reuse facts, ordering, root handling, and policy evaluation;
do not force copy/remove into the paired-directory traversal state machine.

| Path through the system | Who inspects and gates descendants? |
| --- | --- |
| Both-sided directory, including completely clean trees | Diff walker |
| Original-only directory | Recursive copier |
| Target-only directory | Recursive remover |
| Missing target root | Initialization copier, including source-root guard |
| File/link → directory | Staged copier before old leaf removal |
| Directory → file/link | Recursive remover before installing staged leaf |
| Read-only audit/dry-run | Audit traversal, explicitly including opaque descendants |

In copy and removal, inspect/gate the directory's immediate entries before
dispatching child operations. Validate an entry's root before opening, chmodding,
or removing it. Never call unguarded `remove_dir_all` on a caller-owned subtree
that may contain unexamined entries. Private staging cleanup has separate owned
provenance and must not be blocked by the policy for user data.

For jj-waltz, the common gate is accept-or-refuse, which makes recursive outcomes
simple. A generic preserve-capable sync must additionally obey:

- a retained descendant retains its ancestor directories; postorder removal
  returns `Retained`, not an unconditional `rmdir`;
- original-only copying can omit excluded descendants, leaving them absent;
- a directory-to-leaf transition obstructed by a retained descendant **fails**
  without installing the leaf. It cannot both retain that subtree and become a
  file. Earlier removals can already have happened under streaming semantics;
- filtered convergence accounts for retained structural ancestors and exclusions.
  It does not claim equality of the full roots.

Do not initially add preserve-capable sync unless these outcomes are implemented.
The first usable jw integration can expose an accept/refuse guarded sync and
reserve pair-preservation for read-only comparison and explicit scope partitioning.
That is a coherent smaller implementation, not an exclusion hole.

### Observation requirements and mount checking

The caller requests facts, rather than issuing fresh path queries in its gate.
For jw: kinds, basic metadata, literal link text for source warnings, and Linux
mount identity. Parent facts are carried down, not re-read for every child.

Initially, obtain mount IDs with a no-follow `statx` supplement and reuse ordinary
metadata already collected. Later test a Linux inspection backend requesting
basic fields and `STATX_MNT_ID` together, so one syscall supplies both. Verify
returned masks, timestamp precision and symlink semantics; the optimization is
not established merely by choosing `statx`.

Compare each present side to its own authorized parent/root mount. Source and
target need not have equal mount IDs. Check regular files too: a bind-mounted
file is a boundary. Never inspect a symlink's referent. Explicitly document the
existing symlink exception if retaining it; `cow::validate_directory` currently
continues before mount checking symlink leaves.

Linux support can be feature-gated so the default core remains std-only. An
unsupported requested capability fails at setup; do not silently substitute
device equality for mount identity. macOS's current device-based policy is a
separately named platform guarantee. This is still stable-tree traversal, not a
sandbox against concurrent hostile path replacement.

## 5. Parallel placement: gate before admission

For the performance path, the gate is a pure deterministic
`Fn(&mut Inspection) -> Result<...> + Sync`, borrowing immutable command context.
“Mutable” here means its invocation-local decision slots, not shared policy state.

1. Worker enumerates and observes an immediate directory.
2. Worker evaluates the gate **outside the frontier mutex**.
3. Worker lowers the accepted view to diff, allowed children, and diagnostics.
4. Worker publishes that result. Only now can a clean result expose children.
5. Ordered reduction commits records, diagnostics, errors, and coverage.

No admitted job exists below a preserved/refused directory. A refusal still
surfaces at its ordered position, so an earlier valid prefix may have been applied.
Retain divergence seals, joined epochs, J/K bounds, and the demand reserve.
An empty filtered diff may release only its filtered child list.

The gate must not spawn processes, mutate files, print, or rely on invocation
order. Parallel speculative invocations are not exactly-once externally visible
events. Built-in helpers are ordinary predicates/combinators over this seam;
serializable rule vectors are optional sugar.

A sequential-only `FnMut` convenience adapter is plausible. Supporting a
non-`Sync` caller-thread callback in parallel requires the separate alternative
below; do not put a lock around it and call that equivalent.

## 6. Freeze and batch ignores outside the payload loop

Moving `git check-ignore` into a callback without changing its process lifetime
only relocates the cost. The question jw asks is primarily “is this selected root
eligible under these frozen rules?”, not “is every payload file ignored?”.

Keep native Git parsing in jj-waltz initially:

1. Build each distinct frozen ruleset once. Reuse source/desired contexts when
   their actual inputs match. Tracked rule contents and effective excludes are
   immutable command inputs, not repeatedly rediscovered live configuration.
2. Retrieve a tracked ignore file at most once per frozen tree/path initially.
   A later batched JJ/tree-object extraction is worthwhile, but requires a real
   binary-safe multi-file framing design; concatenated `file show` output is not
   such a protocol.
3. Batch known selected-root and link-path proofs in one NUL-input invocation per
   ruleset. Preserve directory-vs-file query form and forced-tracked checks.
4. During adaptive glob discovery, use **one long-lived Git oracle per ruleset**.
   A rejected candidate may expose eligible descendants; collecting only the
   first matched roots and batching afterwards changes selection semantics.

Git explicitly documents this session protocol:

```text
GIT_FLUSH=1 git check-ignore --no-index --stdin -z --verbose --non-matching
input:  pathname NUL
output: source NUL linenum NUL pattern NUL pathname NUL
```

Use the existing isolated Git environment/tree. Nonmatching records have empty
source/line/pattern fields; a matching leading `!` pattern means **not ignored**.
Validate each returned pathname, retain directory trailing-slash semantics, drain
stderr, bound in-flight requests, and close/join the child. Avoid pipe deadlock
by coordinated reading/writing; no-output timeouts are not negative matches.
Memoize answers by frozen-context identity, native path, and query kind.

This removes per-candidate spawns while preserving Git's precedence semantics.
An in-process ignore parser is an alternative, not a prerequisite. Payload
inspection gates use frozen membership/prefix data and do no ignore subprocess
work. This also keeps the generic filesystem library independent of Git and JJ.

## 7. Failure timing is the decision that matters most

**Recommend streaming payload checks after complete scope/ownership preflight.**
Accept that a deep policy violation may be discovered after earlier payload or
tracked mutations. The offending entry is not copied/deleted, but preceding work
is not rolled back. A deep mounted path must be refused before entering it.

This changes more than an error string. Today jw advertises real whole-batch
preflight: it does not knowingly start a valid subset of a blocked batch. Under
this proposal, all targets still pass root/identity/bookmark/selection/obstruction
checks first, but an interior payload violation in a later target can emerge
after earlier targets finish. State that distinction in the command contract.

Likewise, a nested-repository refusal no longer promises that other contents of
the same removable subtree remain unchanged. Permission relaxation on accepted
ancestors may have occurred and must be restored where they survive.

Retry is not “heal any refusal.” Resolve the invalid entry or change the selected
scope first. Then a fresh payload comparison can converge; a partial tracked
refill still follows jw's existing out-of-band JJ recovery instructions.
Do not resurrect the obsolete journal/receipt/resume design for this work.

### Dry-run and summaries

A readiness dry-run should inspect every selected payload descendant, including
one-sided units, through the same gate without applying. This replaces its
separate validation walk; it cannot skip those entries and honestly say READY.
Dry-run remains a current observation, not a certificate reusable after writers
resume. Existing JJ snapshot effects in preview remain separately documented.

Keep comparison counts and physical inspection/application counts distinct.
`examined_directories` retains its paired-comparison meaning. Add explicit
guard coverage for recursive one-sided inspections and exclusions; normal
iterator exhaustion alone cannot certify full policy coverage.

- Existing `is_complete`/`is_clean` must not become true for skipped full-root
  coverage. Add a distinctly named in-scope completion/convergence answer.
- An opaque record in a read-only walk does not prove its descendants passed.
- A guarded mirror can combine walker coverage with successful guarded recursive
  application to report in-scope completion. Its observed diff remains pre-apply.
- Do not add an unconditional second comparison pass to recover confidence in
  the first. Test that the operation kernel establishes its postconditions; offer
  a separately requested audit when a fresh observation is useful. Neither pass
  detects same-size/same-mtime content edits under the existing comparison model.

## 8. Alternatives worth seriously considering

### A. Strict inspect → plan → apply

Choose this if discovering *any static policy violation before any target mutation*
is more important than streaming latency/memory.

Inspect the entire batch, including one-sided descendants, and retain a validated
operation plan. Apply only after it passes. Store recursive operation structure,
not just public opaque records. Reuse retained metadata/listings in application;
this can remove repeated enumeration without relaxing the preflight promise.

Cost: O(entries/operations) retained state in the worst case, delayed first write,
and freshness tied to the same paused-writer assumption. Copy/open/delete syscalls
still happen; permission changes, I/O failures, and crashes can still cause partial
application. A strict plan prevents discovered policy failures, not all failures.

If retaining a full plan costs too much, validate then stream-apply is the simpler
two-pass variant. It still benefits greatly from frozen ignore contexts and
parent-metadata reuse. Do not represent it as one-pass execution.

**My second choice**, especially if whole-batch readiness is non-negotiable.

### B. Caller-owned lending cursor instead of a callback

Expose an inspected directory frame to the caller, who edits decisions and
explicitly advances. It shows clean frames as well as divergent ones; owned
`DirDiff`s remain a derived reporting view. The same primitive can inspect
one-sided directories.

This keeps all control in the pull loop and permits ordinary mutable policy
state. It needs a policy barrier on every unresolved frame: workers may observe
permitted children, but cannot expand below a frame awaiting caller decisions.
The clean-tree path now pays caller round trips and retains unapproved results.

Choose it if an interactive/stateful consumer actually needs these decisions.
For jw's pure entry guards it buys little while weakening today's completion-driven
frontier. Prototype clean-tree throughput and K saturation before selecting it.

### C. A uniform one-sided/two-sided tree engine

Represent each frame as optional original/target sides; empty side means recursive
copy/remove. Enter/leave structure drives pre-order copying and postorder removal.
Generate current stat-free diffs as an adapter rather than making them the only
internal representation.

This gives comparison, audit, policy, and sync one visitation model and naturally
supports preserved descendants. It is the strongest long-term simplification if
more operations accumulate. It also changes opaque-unit handling, failure
boundaries, scheduling, and summary adaptation together. A mirror engine can
reuse the observation kernel first without requiring this rewrite.

Choose it if guarded recursive action develops enough duplicate traversal logic
that the narrower seam no longer stays small. Avoid claiming that an enter/leave
public API is necessary merely to reuse observations internally.

### D. Coordinated source-to-many-target streaming

Share each source directory observation and source-side policy decision across
several destinations, as already contemplated in the README. Destination checks
remain separate. Immutable source facts plus local target frames make the gate
design compatible with this direction.

Without sharing, N pair walkers still inspect the donor N times. Removing global
preflights does not eliminate that cost. With sharing, keep bounded lookahead and
define whether a slow/failed target stalls, detaches, or is re-walked; do not let
the “stream” grow into an accidental full-source cache. This is the best next
architecture experiment for group reseeding, independent of single-target policy.

### E. In-process ignore matching

Compile the frozen rule bundle to an in-memory matcher. This removes even the
long-lived Git request/response latency and is attractive if discovery remains
expensive after process reuse. Verify precedence, negation, ignored-parent
behavior, directory-only rules, escaping, global/info excludes, and native path
handling against Git before replacing the oracle. The matcher belongs in jw's
policy adapter, not the generic walker.

## 9. Performance model and discriminating experiments

For one target, the intended filesystem cost becomes:

```text
scope checks outside payload interiors
+ one observation per existing side of each paired payload directory
+ one recursive gated visit per one-sided copy/removal unit
+ actual divergent leaf/subtree work
```

There are still syscall/type checks at public record application boundaries.
The target is **no redundant payload-wide validation pass**, not literally one
stat of every inode for every possible execution. Parent mount observations are
reused, selected ignore answers are reused, and a clean payload creates no staging
directory and performs no writes. Some JJ snapshot work remains independently.

The policy inquiry reports a ~9 s planning dry-run and ~159 children; those are
prior reported measurements, not reproduced here. Its costs include process
fan-out as well as enumeration. Measure them separately rather than predicting
that a fast raw walk makes jw fast automatically.

Build controls with the same compiler and separate:

1. current jw; incremental sync with existing preflights;
2. frozen/batched ignores only;
3. shared inline gate with ordinary metadata + mount supplement;
4. combined Linux metadata/mount inspection;
5. strict retained-plan alternative if refusal timing remains contested.

Use clean, sparse-delta, large added/removed subtree, absent target, and dense
replacement fixtures; N=1/2/4 destinations. Report warm and cold conditions,
full command and phase times, child processes, listings, metadata/mount calls,
readlinks, reflinks, unlinks, writes/chmods, memory, and first-write latency.
Count physical speculative work separately from committed coverage. Sync only
owned fixtures; a live tree can supply read-only comparison measurements.

Discriminating correctness cases:

- identical nested `.git` on both sides; nested `.git` in each one-sided direction;
- deep special entry and same-device bind mounts on both directories and files;
- source/target symlink leaves, mounted selected roots, and absent targets;
- deep refusal after some successful siblings: offender untouched, partial work
  reported, surviving permissions restored, no false complete summary;
- clean subtree preservation never admits descendant jobs, including P>1/K>0;
- type transitions remain coherent; preservation blocking dir→leaf never installs
  the replacement; clean repeat sync has zero writes;
- ignore negation and nonmatching protocol responses, ignored parents, different
  source/desired rules, forced-tracked descendants, and malformed/failed Git output;
- dry-run checks opaque interiors; whole-batch versus streaming refusal timing
  matches the chosen contract rather than accidentally inheriting old wording.

## 10. Decision and landing order

1. **Freeze/batch selection facts in jw.** Independently useful and does not depend
   on accepting late payload refusal.
2. **Preserve selected roots and establish incremental integration.** Keep existing
   recursive checks for this checkpoint; measure the true payload delta benefit.
3. **Select refusal timing.** My recommendation is complete scope preflight plus
   streaming payload guards; choose alternative A if strict batch preflight wins.
4. **Implement the shared inspection seam in comparison and all recursive sync
   paths.** Initially accept/refuse is sufficient for jw. Establish sequential
   behavior and the pre-admission parallel placement together in the design.
5. **Remove the replaced payload-interior checks**, including early whole-checkout
   scans that would otherwise keep paying the same cost. Preserve tracked-phase
   obstruction and ownership checks through explicit scope partitioning.
6. Measure combined statx and source-to-many-target sharing as independent follow-ons.

The [GLM inquiry](policy0.glm53m.md) contributes the useful prefilter instinct.
This proposal narrows its editing surface, extends its reach through recursive
application, makes parallel placement explicit, and treats frozen ignore-process
work as a separate lifetime problem. Those changes are what make the proposal
an end-to-end performance design rather than only a richer walker API.
