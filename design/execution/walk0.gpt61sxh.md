---
type: ImplementationReport
title: A component-built walking specimen with real process continuation
description: Delivered bounded examples/walk implementation, exact process and failure gates, dependency isolation, checks and deliberate limits.
resource: /design/execution/walk0.gpt61sxh.md
status: implemented-and-verified-bounded-specimen
generated: { by: "model:openai/gpt-6.1-sol#xhigh", at: 2026-10-20 }
sources:
  - resource: "opencode://ses_ea7b7c0a4ffe1KVm1nvPn7svzl"
    title: Implemented the reviewed walking builder, corrected staging ownership, tested component substitution and fresh-process gates, and documented scope
  - resource: /design/roadmap/brief0.gpt6a.md
    title: Independent reading boundary and no-history/no-recovery assumptions
  - resource: /design/execution/builder0.gpt6am.md
    revision: 91d2f7e3
    title: Reviewed concrete builder, component ownership, optional codec and acceptance handoff
  - resource: /design/execution/contracts0.gpt61sxh.md#4-next-commission-a-component-builder-and-real-walking-specimen
    revision: 5d6c003a
    title: Fixed demo semantics, four process gates and focused acceptance criteria
  - resource: /design/execution/model0.gpt61sxh.md
    revision: 3113dfd2
    title: Before/assess/children/finish ownership, acceptance, retry and skip
  - resource: /design/observation/codec0.gpt6am.md
    revision: 8b709ce0
    title: Completed Snapshot fit and byte-vector/allocation qualifications
  - resource: /src/visitor.rs
    title: Public full-fact immediate inspection and unsupported-entry refusal requiring a narrow target-only retention adapter
  - resource: /examples/cursor/src/codec.rs
    title: Actual Snapshot reconstruction and separate publication precedent, unchanged
  - resource: /examples/walk/README.md
    title: Delivered semantics, runnable fresh fixture and library example
  - resource: /examples/walk/src/driver.rs
    revision: 36f212ef
    title: Delivered Bon builder, actual component calls and checked request/ack progression
extensions:
  ticket: diffdir-tree-execution-walk-demo
  reading_boundary: Named completed handoffs and relevant implementation only; withheld parent statement, full roadmap, parent transcript and current generated/modes/native-graph research not read, searched, cited or evaluated; no subagents
verification:
  method: Example-only nextest in codec-free and all-feature configurations, warnings-denied clippy, formatting, dependency graphs, runnable README fixture and protected-path diff
  result: Nine codec-free and eleven all-feature tests passed with none skipped; four separate process gates, uninterrupted equivalence and terminal roots-removed resume passed; base/all-feature clippy and format passed; core normal graph remains dependency-free
---

# Delivered outcome

**`examples/walk` now really walks, writes bytes, pauses and resumes from supplied
components.** The [guide](/examples/walk/README.md) and runnable
[builder example](/examples/walk/examples/builder.rs) cover CLI and codec-free
library use. There is one recursive owner, a concrete desired-byte program and a
separately saved manifest ordinal. No root API, sync behavior, original cursor,
native JJ integration, registry or production-default change was made.

## Implementation boundaries and coordinates

Coordinates below refer to delivered code through `36f212ef`:

| Source | Delivered responsibility |
| --- | --- |
| `examples/walk/Cargo.toml:7–29` | Nested workspace; own lock/target; empty default features; optional codec and CLI only. |
| `src/driver.rs:110–166` under the example | Real Bon builder accepts `StartParts::Fresh/Restored`, supplied inspector/executor and concrete program/manifest; validates data/config without filesystem access; moves those objects into `Running`. |
| `src/program.rs:8–61,66–104` | Whole-batch pure lowering independently selects desired-file goals, including physically equal regular files; saved manifest config is separate from executable recipe derivation and `ManifestState`. |
| `src/state.rs:9–129,259–363` | Native Unix bytes, lexical/disjoint roots, injective direct-child report mapping, active front/suffix/local/phase/coverage/ticket/recipe consistency, checked aggregate arithmetic and terminal/live exclusivity. |
| `src/driver.rs:221–257,303–362,365–434` | Explicit filesystem bind; pure owned requests; candidate-state acceptance; retry/skip; two independent attempt limits; once-per-lineage closure/fold. No effects ledger or completed-edge history. |
| `src/driver.rs:259–298` | Dispatch invokes **the supplied objects**, not hidden defaults. Transient complete plans enter only an acknowledgment, never saved state. |
| `src/io.rs:12–44,58–127` | Full sequential immediate inspection, pure retained-extra adapter, explicit current route/kind checks, ordinary directory creation and three nonrecursive work kinds. |
| `src/io.rs:131–165` | Exact-byte satisfaction; closed same-parent staging and rename; successful rename consumes staging ownership; failures explicitly attempt and report owned cleanup. |
| `src/codec.rs:14–77` | Actual `Snapshot::from_value/from_bytes/reconstruct`, schema `diff-dirs.walk-example` v1, `WalkingCursor/example` identity, 8 MiB archive cap and separate complete cursor publication outside the roots. |
| `src/main.rs`, `examples/builder.rs` | Fresh/resume CLI with explicit limits/retry/skip and separate JSON output; codec-free library construction, saved-parts rebind and manual loop. |

The guide explains the counters: invocation-local attempted assessments/effects,
accepted container/leaf/finish/no-write goals, and observed destination file
installations / created directories. A failed after-visible-effect attempt can
report an installation without any accepted leaf/finish. These are deliberately
**not a census of staging namespace syscalls**, fresh inventory, or saved effects
history. Aggregate physical differences/retained boundaries are separate from
settled leaves.

## Exact evidence

`tests/process.rs:39–164` launches actual fresh executables for the fixed fixture
`source/d/a="one", source/d/b="two"`, empty existing target/reports, label A:

| Process | Limits | Saved/physical evidence |
| --- | --- | --- |
| 1 start | 2 assessments / 2 effects | Root assessed, `d` ensured/assessed, `a` bytes installed and accepted; `b` pending. `d` leaves 1, root 0, ordinal 1. |
| 2 resume | 0 / 1 | `b` bytes installed and accepted; pending `d` finish, `d` leaves 2, root 0, ordinal 1. |
| 3 resume | 0 / 1 | Exact `dir-64.txt` bytes installed; `d` retired/folded, root leaves 2, pending root finish, ordinal 2. |
| 4 resume | 0 / 1 | Exact root report bytes installed; terminal leaves 2, complete-in-domain, ordinal 3. |

The test decodes Snapshot and checks frames, suffixes, pending work, recipe/label,
ordinal, aggregate and incomplete flags, not only stdout. It compares against a
separate uninterrupted run's final outcome, component state and all output bytes.
A later fresh resume after removing **all three roots** remains terminal without
tree I/O. The resumed effect calls report zero assessments.

The focused library gates in `tests/components.rs` cover:

- **122–166:** supplied counting inspector and once-failing executor really run;
  `b` blocks after accepted `a`; fresh wrappers restore saved parts without a codec,
  do zero ancestor inspections, preserve ordinal and eventually render `custom`.
- **169–201:** before/after-visible-effect failures for both `b` and `d` report;
  retry retains accepted prefix, advances leaf/fold/ordinal only on acceptance,
  and after-visible retries perform zero installations with unchanged output inode.
- **204–228:** leaf/finish skip preserves visible unaccepted artifacts and accepted
  siblings, marks all ancestors incomplete, permits explicitly partial reports,
  and consumes no skipped-finish ordinal.
- **231–291:** repeated owned request, wrong ticket/kind and duplicate refusal
  without mutation; an effect executed but not acknowledged restores unresolved
  and settles by fresh byte checks, not guessed acceptance.
- **294–338:** `0/0`, effects-zero ready finish, clean/empty roots with new runs
  A → B → B (changed desired output then a no-write goal), data-only construction
  without roots, explicit bind failure, and terminal no-I/O.
- **341–363:** equal size/mtime but different bytes still settle; non-UTF-8 directory
  names and hex mapping survive; target-only directories/interiors and a special
  socket boundary remain untouched.
- **366–437:** source links/specials, admitted target links, shape conflicts and
  overlong report routes refuse before child effects; failed before/assessment
  attempts charge their class and skip only their scheduled subtree.
- **440–466:** changed-label restore refuses; representative malformed route,
  ticket, ordinal, edge, local state, aggregate overflow and root alias data refuse.

`tests/codec.rs:11–76` checks exact whole-value native-byte/pending-finish roundtrip,
checksum corruption, malformed bytes, byte cap, incompatible schema/identity,
malformed reconstructed state, cursor-in-root refusal and data-only reconstruction
after removing roots. This is targeted malformed-data coverage, not a hostile-data
security campaign.

## Exact checks and staged corrections

From the repository root, final checks:

```sh
cargo nextest run --offline --manifest-path examples/walk/Cargo.toml --all-features
cargo nextest run --offline --manifest-path examples/walk/Cargo.toml --no-default-features
cargo clippy --offline --manifest-path examples/walk/Cargo.toml --no-default-features --all-targets -- -D warnings
cargo clippy --offline --manifest-path examples/walk/Cargo.toml --all-features --all-targets -- -D warnings
cargo fmt --manifest-path examples/walk/Cargo.toml --check
cargo tree --offline --manifest-path examples/walk/Cargo.toml --no-default-features --edges normal --prefix none
cargo tree --offline -p diff-dirs --no-default-features --edges normal --prefix none
jj diff --git --from 91d2f7e3 --to @ Cargo.toml Cargo.lock src sync examples/cursor
python .test-agent/walk-guide/doc_checks.py
```

Results: **11 all-feature tests passed, 9 codec-free tests passed, zero skipped**;
both clippy configurations and format passed. The no-default nextest/clippy
commands compiled the base library/tests and the latter compiled the library
example. Separate initial `cargo check` base/all-feature commands also passed.
The base example graph contains Bon, diff-dirs, tempfile and error support, **no
Facet/Kopacetic/Clap/Serde**; core's normal graph is just `diff-dirs`. The protected
path diff was empty. No broad root suite or existing cursor suite was rerun.
The two owned documents passed session provenance, local-link/path, balanced-fence
and trailing-whitespace checks; their reciprocal guide/report links are maintained.

The guide's actual four commands and codec-free builder command were additionally
executed with a new tiny `/tmp/opencode/walk-guide.*` fixture; all succeeded.
Ignored `.test-agent/walk-guide/` records those commands, not a delivered API.

Preserved stages:

1. `c0702402`: initial component-owned implementation; base/all-feature compilation
   passed, initial focused tests exposed a real cleanup bug after visible rename.
2. `07083895`: corrected consumed staging ownership with closed `TempPath::persist`
   and explicit failed-path cleanup; actual Snapshot/process gates, 10 tests passed.
3. `ef0f0774`: narrow retained-extra adaptation, immediate-batch/ticket validation,
   before/assessment gates and uninterrupted equivalence; 11 tests passed.
4. `36f212ef`: codec-free runnable builder example and explained narrow lint
   allowances for direct owned transient/construction enums; clippy/guide passed.

## Necessary adaptation and limits

The handoff suggested `NoVisitor`, but untouched core `visitor.rs:863–872`
rejects unsupported target-only kinds before pure program lowering could retain
them. **FullInspector therefore supplies a tiny pure late visitor preserving only
target-only slots.** It still acquires full actual facts in one public sequential
inspection; source-present proposals remain baseline and are checked as such.
This honors the fixed retained-extras semantics, including special boundaries,
without another uncharged scan, sparse API or core change. The socket test proves
the boundary. It is the one consequential adaptation from the proposed mechanics,
not a new generalized Visitor-refresh adapter or policy fork needing selection.

All other bounds remain the commission's: Linux/Unix tiny writable fixtures,
existing real disjoint roots, good stable source, caller-owned nonhostile topology
and concurrency. Root route validation also refuses symlink ancestors. Desired
attributes are unspecified; tempfile staging uses ordinary private-file modes,
directories ordinary creation/umask. No CoW, chmod adaptation, metadata preservation,
deletion, shape replacement, native JJ, arbitrary provider/registry or recursive
sync apply is supplied. File bytes are read into memory; listing width, frontier,
file size, runtime/syscalls and candidate-clone allocation costs are not bounded
by the two numeric attempt limits. This simple specimen uses cloned candidate
state and owned remaining vectors, not a large-frontier optimization.

Report names have the documented 255-byte specimen basename cap (nonroot relative
bytes ≤123), not a portable filesystem limit. Snapshot's 8 MiB bound covers archive
bytes only, not projection/reconstruction allocation. Validation checks available
data consistency, not the truth of old physical effects. External acknowledgments
are trusted. Reusing/forking old saved state can repeat effects. Cursor publication,
stdout and tree effects remain separate; no rollback, recoverability, durability
transaction, exactly-once promise or history is introduced. Cleanup errors are
reported, not converted into acceptance.

Parent retains root-index/tracker-export/promotion/closure work. Only this report,
the new example and a scoped ticket pointer/note are owned here. No unresolved
architectural fork blocks this bounded specimen; broader production adapter and
policy choices remain outside it.
