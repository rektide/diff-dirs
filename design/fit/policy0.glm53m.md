---
type: DesignInquiry
title: "Walk-time policy and control: folding jj-waltz's preflight into the iteration"
description: Mulling handoff — the current diff-dirs operating model, the preflight work jj-waltz must perform before the walk and why, a candidate WalkPolicy (skip/refuse rules evaluated during iteration), and broader reshaping questions including ignore evaluation inside the loop and policy-aware sync. Posed for deliberation, not ratified; the root README's settled decisions stand until amended deliberately.
resource: /design/fit/policy0.glm53m.md
tags: [diff-dirs, jj-waltz, walk-policy, preflight, ignores, control-model]
status: draft
generated:
  by: "model:zai-coding-plan/glm-5.3#max"
  at: 2026-10-18
sources:
  - resource: "opencode://ses_eb8a7487affeE2smndZ914OQMS"
    title: Parent session — posed the problem after measuring jj-waltz's planning fan-out and drafting the candidate WalkPolicy shape
  - resource: /README.md
    title: Current contract — records, summary, driving model, settled decisions
  - resource: /src/model.rs
    title: WalkOptions today (execution only), DirDiff, Summary fields
  - resource: /sync/README.md
    title: Applier/mirror contract, no-exclusions design, caller-loop discipline
  - resource: /design/fit/handoff0.glm53m.md
    title: Integration handoff documenting the load-bearing recursive guard composition this inquiry proposes to fold
  - resource: "jj-waltz:src/reseed/engine.rs"
    title: Preflight call sites — selection-time payload validation, execute-time validate_contents, IgnoreRules constructions
  - resource: "jj-waltz:src/cow/payload.rs"
    title: IgnoreRules — isolated git tree from frozen desired rules, per-root check-ignore children
---

# Walk-time policy and control

## The problem, posed

jj-waltz today pays for safety with **separate full-tree passes before the
diff walk ever runs**: a donor-side payload validation walk, a target-side
whole-tree validation walk, and ignore-rule evaluation as its own child
process fan-out — and then the diff walk traverses much of the same ground
again. At pool scale that is roughly three payload-sized traversals plus
~159 child spawns (105 `jj` + 54 `git`, measured on a planning-only dry-run,
~9s warm) before any reconciling happens.

The operator's instinct: the walk already visits every entry and already
holds the stat data those preflights re-collect. Exclusions and tests should
ride the iteration, and the control vocabulary during iteration — today just
`prune_here`/`break` — could be much richer. This document records the
current operating model, the preflight obligations, a candidate direction,
and the broader reshaping questions worth mulling. Nothing here is ratified.

## How diff-dirs operates today

- **Core**: a pull iterator (`walk()` / `walk_with_options()`); the consumer
  drives everything in its own loop. `WalkOptions` currently carries only
  `execution` (Sequential default; Parallel via the optional frontier).
- **Records**: `DirDiff` per divergent directory — immediate children only,
  names + classification (`files: changed/added/removed`,
  `dirs: added/removed`), deliberately **stat-free**.
- **Control**: `prune_here()` (legal immediately after a record — skip
  descent into that directory; the subtree is recorded in `Summary.pruned`),
  `break` (stop), `into_summary()` (counts, `topmost_divergent`,
  `changed_dirs`, `examined_directories`, `state`, `pruned`).
- **Actions**: none in the core. Mutation lives entirely with consumers —
  `sync`'s `Applier`/`mirror` applies records (staged replacement, exact
  metadata, `CopyPolicy`); its contract is "finish applying before pulling
  the next record." `sync` performs **no** `.git`/`.jj`/mount/ignore
  exclusion of any kind — "compare exactly the roots given" — by design.
- **Gather vs apply-in-loop** are just two loop bodies over the same walk.

Notably absent: any way to tell the walk *in advance* "never descend into
X," "refuse if you see Y," or "consult these ignore rules as you go."
`prune_here` decides after a record is yielded; nothing decides from data.

## The preflight work jj-waltz must do, and why

Before the payload walk, jj-waltz's engine currently establishes:

1. **Donor payload validation** — per selected ignored root, a recursive
   `validate_tree`: administrative names (`.git`/`.jj`) at any depth,
   same-mount (statx mount-id, not just device) at every level, special
   entries refused. A full payload-sized traversal (~192k entries at pool
   scale).
2. **Target whole-tree validation** — `validate_contents` over the entire
   target: the same admin/special/mount checks, guarding both the tracked
   phase (sparse clear / materialization must not collide with surprises)
   and the payload phase.
3. **Ignore-rule proofs** — `IgnoreRules::new` builds an isolated git tree
   from the frozen *desired* tree's `.gitignore` files (~20 `jj file show`
   children per construction, rebuilt several times per run), then one
   `git check-ignore` child **per selected root** to prove each root is
   ignored under the desired rules.

Why they exist: `sync` will happily sweep a target-only nested `.git` as a
deletion and copy anything the walk classifies — the recursive validations
ARE the safety story (recorded as load-bearing in the
[integration handoff](handoff0.glm53m.md)); ignore proofs pin selection to
the frozen desired state; and everything runs **before mutation** so refusal
costs nothing. The cost is the duplication above.

## Candidate direction: WalkPolicy, evaluated during iteration

```rust
struct WalkOptions {
    execution: Execution,
    policy: WalkPolicy,            // new; default empty
}

struct WalkPolicy {
    skip:   Vec<SkipRule>,         // subtree invisible + Summary.skipped
    refuse: Vec<RefuseRule>,       // fail closed: Err naming the path
}
```

- Rules are predicates over an **`EntryInfo`** (kind, size, mtime, name, and
  unix-gated device + statx mount-id) assembled from stats the walker takes
  anyway — zero additional IO, evaluated in sorted order (deterministic).
- Built-ins without closures: `AdminNames`, `SameMount` (unix, cfg-gated;
  non-unix refuses to enable rather than silently passing),
  `RegularFilesAndLinks`, and plausibly `IgnoredUnder(rules)` (see below).
- Semantics: **skip** → never descended, never classified, recorded in a new
  `Summary.skipped` (mirroring `pruned`); **refuse** → the iteration yields
  `Err` naming the offending path. The frontier evaluates skip at enqueue
  time; a refusal aborts the epoch.
- Records stay stat-free. `EntryInfo` exists only as predicate input — not a
  back door to stats-in-records.

**What jj-waltz folds:** donor payload validation and target
payload-root interiors ride the diff walk (`refuse: AdminNames +
SameMount + specials`); `validate_contents` shrinks to the tracked-relevant
tree (~9k entries instead of ~200k). Net: one payload traversal warm instead
of ~three, and cold-cache IO paid once.

**The acknowledged trade — fail-late.** With apply-in-loop, a violation deep
in the tree surfaces after earlier records applied. Under jj-waltz's already
blessed convergent-not-transactional recovery, retry heals; gather-mode
consumers keep the old refuse-before-touching-anything behavior for free.
This is the main semantic change to weigh.

## Broader reshaping to mull

- **Ignore evaluation inside the loop.** `IgnoredUnder(frozen_rules)` as a
  built-in rule would let "is this root ignored under the desired tree" be
  proven during descent instead of as a per-root child process. Open
  questions: the rules are frozen policy state that must be passed *into*
  the walk (a policy payload the core currently knows nothing about);
  whether evaluation belongs per-directory (cheap, at candidate boundaries)
  or per-entry (fan-out); and whether the isolated-git-tree machinery
  belongs in diff-dirs at all or stays a jj-waltz-side rule implementation
  behind the predicate interface.
- **Policy-aware sync.** Should `sync` accept a `WalkPolicy` and enforce it
  during apply (skipped subtrees never applied; refusals abort mid-apply),
  so `mirror()` is safe standalone? Or does policy stay at the consumer
  seam between `walk` and `Applier`, keeping sync dumb? The latter is the
  current shape; the former concentrates power but risks duplicating checks
  consumers already run.
- **More control operations.** Preemptive descent filters (predicate-based
  `prune` decided before listing a directory — IO-identical to `prune_here`
  but set-once); depth limits; `resume_from(path)` to retry after a refusal
  without re-walking the clean prefix. A per-entry decision *callback* was
  considered and declined during the pull-model design (it re-introduces
  push control); declining it again unless a concrete need appears.
- **Execution reassessment.** Policy at enqueue-time must preserve the
  frontier's determinism and join guarantees; `Summary.skipped` joins
  `pruned` in the completeness story (`is_complete()` semantics for
  deliberately-skipped subtrees need deciding).

## Perils to keep in view

1. Fail-late vs fail-closed is a real semantic shift for apply-in-loop
   consumers, not just an optimization.
2. Scope creep: skip/refuse only — the walker gains *policy*, never
   *actions*. Mutation stays at the consumer.
3. The stat-free record settlement and the "no callbacks" settlement both
   stand unless deliberately revised; `EntryInfo` must not erode the first,
   built-ins must not erode the second.
4. Mount identity is unix/statx-specific; core portability (std-only,
   Windows/musl in jj-waltz's dist matrix) constrains where `SameMount`
   can live.
5. Whatever folds into the walk, jj-waltz's *tracked-phase* validation
   cannot disappear — only the payload-interior portion can ride the walk.

## Standing

A mulling document. The [root README](../../README.md) is unchanged and its
settled-decisions list governs until a ratified amendment lands. The natural
ratification path, if this holds up: amend the README with a `WalkPolicy`
section, implement rules + `Summary.skipped`, then fold jj-waltz's preflight
per the integration handoff's Option-B sequencing.
