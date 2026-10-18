---
type: Assessment
title: Performance tooling takeover — keep the instruments, bound the engine
description: Read-only cross-repository inventory and takeover boundaries after the Python experiment prototype; no benchmark or implementation performed.
resource: /design/performance/tooling-takeover0.sol61m.md
status: assessed-not-implemented
generated: { by: "model:openai/gpt-6.1-sol#medium", at: 2026-10-18 }
sources:
  - resource: "opencode://ses_eb2165cc8ffe4vOmOvDBe7aXQu"
    title: Inspected existing Rust runners, Python prototype, frozen controls and JW adapters read-only; identified concrete gaps and bounded Rust-first takeover
  - resource: /design/performance/tooling-audit0.glm53fx.md
    title: Earlier inventory and locally verified hyperfine semantics; no tool capability probes repeated here
  - resource: /perf/README.md
    title: Maintained comparison and synchronization measurement contracts
  - resource: /perf/results/checked/README.md
    title: Historical checked-streaming experiment source, retained rows and executable provenance
  - resource: /perf/experiments/runner.py
    title: In-flight experiment engine inspected with its registry, fixtures, scheduler, reporting and tests
  - resource: /perf/experiments/tooling0.sol61h.md
    title: Completed stopped checkpoint, twelve reported passing tests and unrevalidated frozen-adapter repair
  - resource: "file:///home/rektide/src/jj-waltz/tools/performance/experiments/handoff0.sol61h.md"
    title: Completed JW tooling checkpoint with sixteen passing and four failing matrix attempts
  - resource: "file:///home/rektide/src/jj-waltz/tools/performance/README.md"
    title: Maintained preparation/reseed correctness and subprocess-count harness
  - resource: "file:///home/rektide/src/jj-waltz/tools/performance/jw_sample.py"
    title: In-flight single-native-reseed adapter and its verification boundaries
  - resource: "file:///home/rektide/src/jj-waltz/.test-agent/inline-policy/integration-handoff.md"
    title: Preserved baseline, clean-payload invariant and actual opencode workload constraints
---

# Verdict

**Stop growing the Python framework; do not discard its useful work.** Existing
Rust runners already own the difficult measurement semantics. The new prototype
adds worthwhile immutable executable copies, balanced scheduling and raw paired
results, but it is not yet an accepted cross-repository measurement foundation.
The next slice should prove one narrow clean-control path, not build another
catalog, plan/runtime abstraction or general experiment platform.

This assessment ran no benchmark, test, build, cleanup or live-donor command.
Only this new document is changed. Application implementation correctness and
speed are outside this review. Existing tests reported by other work are not
independently rerun here.

## Observation boundary and ownership

At first inspection, diff-dirs working copy was `lqypxrxw / f3dfdf5e`, parent
`8811c056` (documented frozen identities; accepted runtime checkpoint `243be8e2`).
Its new `perf/experiment.py` and `perf/experiments/**`, including eleven generated
`__pycache__/*.pyc` files, were added but uncommitted. JW working copy was
`ztymxynk / 406eb9e7`, parent `1ffcf652`; additions were
`design/up-performance/handoff0.glm53m.md`, `tools/performance/experiment.py`,
`tools/performance/experiments/reseed.json`, and `tools/performance/jw_sample.py`.
Those were observations of moving shared working copies. Before this assessment
finished, the writer wrapped at **D `997183df` and J `a35da35d`**; both final
handoffs were read. These are the completed tooling inventory targets. Line
references below describe the inspected source, with final check results added
from those handoffs.

Ownership: checked/frozen controls belong to Sol xhigh
`ses_eb24f98a3ffewdVqa9lao6tXQu`; new experiment tooling to Sol high
`ses_eb224880fffeZxuFkeb0yDaCeN`; JW baseline/handoff to parent
`ses_eb262a81effea1HgF0pPS3f1sy`. Sol high's session was inspected once for
coordination: it acknowledged stop/checkpoint direction. Final handoffs report
**12 unit tests passed**, no real-payload campaign and no performance win. No
other author's files were touched.

## Actual inventory

Line counts are source lines at inspection, not payload or build-directory size.
`D` below means `/home/rektide/src/diff-dirs`; `J` means
`/home/rektide/src/jj-waltz`. Paths are linked where the distinction matters.

| Location | Actual role / language / size | Standing |
|---|---|---|
| `D/examples/perf.rs` | Rust, 624 lines: metadata fixture setup/shape/census, once/repeat/run, same-build backend compare/diagnose | Maintained instrument; optional `parallel-bench` and `parallel-diagnostics` seams already exist |
| `D/sync/examples/sync_perf.rs` | Rust, 172 lines: marked owned nonempty sync fixtures, fresh/clean/sparse/subtree; serial/P4 × copy/required reflink | Maintained mutating instrument, distinct from read-only diff |
| `D/perf/leaf.rs` | Rust, 131 lines: leaf pipeline fresh/dense/clean/sparse/diff-only, API wall/process CPU timers, pool and verification outside timing | Tracked campaign instrument, compiled through scratch manifest; not a normal Cargo example |
| `D/perf/{compare,frontier,leaf}.py` | Python, 53/64/106 lines: different matrix/interleave policies | Tracked specialized drivers; retain for historical campaigns, avoid extending all three |
| `D/perf/{report,audit,resources,leaf_trace}.py` | Python, 29/33/50/39 lines: old CSV descriptive report, payload-read audit, process resources, trace interpretation | Small useful utilities, not an engine; scopes differ |
| `D/perf/experiments/*.py` + `D/perf/experiment.py` | New Python prototype: 815 non-test module lines plus 230 test lines and 6-line entrypoint; registry, fixture cloning/import, scheduler, subprocess runner, report, frozen adapter, tree verifier | Stopped Sol high checkpoint, not covered by old maintained README; `examples/{smoke,diff}.json` are recipes, not results |
| `D/.test-agent/checked/{bench,live}.rs`, four `.py` drivers | Rust 103/114 lines; Python 107 combined: campaign-specific controls and orchestration | Scratch reproduction source, **not** disposable fixture payload; curated copies in `perf/results/checked/` |
| `D/.test-agent/checked/frozen-fa91/` | Five preserved executables, exact benchmark sources, manifests, locks, compiler and hash-match records | Frozen evidence; do not rebuild in place or infer identity from names |
| `D/.test-agent/{perf,sync,leaf-pipeline,optimization}/` | Owned trees, earlier probes, isolated build roots, saved variants, CSV/trace logs | Mixed fixture/data/source evidence, not four additional maintained frameworks; earlier audit details these |
| `D/.test-agent/{checked-tests,experiment-tests,sync-tests}/` | Test scratch/explainers | Test-owned paths, not benchmark results |
| `D/.test-agent/matched-smoke/` | Fixture, registries, raw cp and frozen-control smoke attempts | New prototype evidence; successful cp smoke and failed frozen attempt both retained |
| `J/tools/performance/{measure,fixture,copy_baseline,test_measure}.py` | Python 606/27/58/151 lines: isolated prepare/preview/first-fill reseed, native/instrumented modes, strict cp baseline, compatibility/count tests | Maintained, useful for process-count regression and correctness; not clean/sparse repeated reseed |
| `J/tools/performance/{experiment,jw_sample}.py` | Python 20/189 lines: sibling-engine launcher; setup/native timed reseed/verification adapter | New prototype, not a second copy of the engine |
| `J/tools/performance/experiments/reseed.json` | 36-line recipe with baseline and placeholder candidate; only unchanged N=1 job | Incomplete configuration, not a runnable completed campaign |
| `J/.test-agent/inline-policy/` | README/handoff, `baseline.json`, preserved `jw-before`, process-count evidence | Parent-owned baseline; source `7253ff6ddee639a98087e1729af58e0cbc546b6f`, SHA-256 `03e0ada406426d9496c602e22c9dd0de452f788e424dab9b7069c07f812e715c` |
| `J/.test-agent/performance/`, `matched-reseed-smoke/` | Retained old harness runs; new adapter smoke area | Evidence/fixtures, not entrypoints; new matrix passed 16/20, failed all four N2/feature attempts |

The completed new Python footprint is **1,260 lines including 230 test lines**
across both repositories, in addition to the older tooling. That is a framework,
not merely a small wrapper.

### Commands that exist now

Run from D, using NEW owned roots/output paths where indicated:

```sh
cargo build --release --example perf --features parallel-bench
target/release/examples/perf once ORIGINAL TARGET
target/release/examples/perf compare ORIGINAL TARGET 4 6 4 256 LABEL
cargo build --release -p diff-dirs-sync --example sync_perf --features parallel
target/release/examples/sync_perf setup NEW_ROOT 300 100
target/release/examples/sync_perf run NEW_ROOT 3
python3 perf/report.py perf/results/btrfs-30k.csv
python3 perf/experiment.py --help
python3 perf/experiment.py report .test-agent/matched-smoke/run/result.json
```

`once`/`compare` are read-only; sync commands mutate only their marked fixture.
The leaf runner requires its recorded scratch manifest; use
[leaf evidence](/perf/results/leaf/README.md), not `cargo run --example leaf`.
Do **not** blindly rerun old checked scripts: they use mutable `target/` binary
paths and output names already occupied by preserved observations.

From J:

```sh
python3 tools/performance/measure.py run --jw /ABS/PRESERVED/JW --jj /ABS/NATIVE/JJ --timing-mode native --files 100 --workers 1
python3 tools/performance/measure.py compare BEFORE/result.json AFTER/result.json
python3 tools/performance/measure.py check AFTER/result.json --max-file-show-calls total=0
python3 tools/performance/copy_baseline.py RETAINED/result.json
python3 tools/performance/experiment.py --help
```

The `check` count budget requires an **instrumented** result, not the native run
shown above. Existing verification commands are
`python3 -m unittest discover -s tools/performance -p 'test_*.py' -v` (J) and
`PYTHONPATH=perf python3 -m unittest discover -s perf/experiments -p 'test_*.py' -v`
(D); these run tests that create tiny private fixtures, not a performance campaign.
None was executed for this assessment.

New prototype workflow is `experiment.py fixture NEW_ROOT`, then
`register MANIFEST NEW_STORE [NAMES...] [--build]`,
`run MANIFEST NEW_STORE/registry.json NEW_OUTPUT --blocks 12 --seed 0`, then
`report NEW_OUTPUT/result.json`. `frozen-plan FROZEN FIXTURE NEW_PLAN --jobs clean`
generates a recipe. These are implemented interfaces, **not approval to start
measurements**. `diff.json`/JW `reseed.json` require explicit path/revision edits.

## What the prototype really guarantees—and does not

1. **Executable identity is materially better.**
   [registry.py](/perf/experiments/registry.py):15–19 refuses label overwrite;
   58–69 hashes and copies executable bytes to a hash-named, read-only artifact;
   75–81 verifies bytes and identity before use. Build mode records compiler,
   exact source command, source state, build argv and hashed explicit inputs.
   Imports cannot establish a historical build merely by running today's
   `rustc --version`: original compiler records must remain authoritative.
   `frozen.py`:35–40 does retain that historical record. Dirty builds permit any
   nonempty explicit input list (registry:47–48), not a proof that *all* dirty
   inputs are captured; source is checked only before build, inputs hashed after
   it. Build only checkpointed, quiescent sources for credible next evidence.

2. **Balanced order is real, drift removal is not.**
   [schedule.py](/perf/experiments/schedule.py):5–16 seeds shuffled permutation /
   reverse pairs, balancing positions over every two blocks. Runner:113–116
   keeps variants adjacent per job and alternates job order. Runner:159–171
   calls hyperfine for **one** command/one sample, so hyperfine's grouped
   multi-command behavior cannot undo alternation. This is a useful design to
   retain. PSI/load snapshots and the shared advisory lock (runner:44–60) do not
   prevent external host I/O. Older `compare.py`:27–32 groups policies;
   `frontier.py`:37–47 groups configurations while the Rust child rotates
   backends. Old checked `clean.py`:17–18 stops rotating once round exceeds the
   14 configurations; its 15 rounds are not a balanced-block schedule.

3. **Managed mutation resets have a sound narrow contract.**
   [fixtures.py](/perf/experiments/fixtures.py):107–166 creates separate scenario
   templates and fresh private required-reflink clones, compares metadata before
   and after reset, and never resets a live donor. This avoids clean-vs-first-fill
   mismatches. Census deliberately covers names/kind/mode/size/mtime/link targets,
   **not bytes or inode identity** (10–27). Thus same-size/same-mtime byte changes
   evade fixture fingerprints. Full byte/link verification is separate and reads
   contents, warming caches before subsequent samples. Huge per-sample cloning,
   census and verification are excluded from API timers but still generate host
   I/O; record these costs rather than describing them as isolation.

4. **Frozen evidence is recoverable, but the old failed smoke is not a result.**
   Frozen README:13–22 honestly distinguishes later trace binaries from original
   timed bytes. Observed `matched-smoke/run/result.json` passed 6 cp samples
   (includes warmups); that tests plumbing, not an optimization. Observed
   `matched-smoke/frozen-run/result.json` failed with 36 samples and clean target
   fingerprint mismatches at the older shared `frozen-plan/case/control-target`.
   Current [frozen.py](/perf/experiments/frozen.py):44–64 separates clean/measure/
   fill cases, and test_experiments:148–168 tests that separation using fake cp
   artifacts. It does **not** demonstrate an actual corrected frozen rerun.
   Frozen recipe supplies baseline/default/admin, not enabled-noop or initial
   implementation controls; those remain in older bench evidence. Its `measure`
   action still emits clean/sparse/subtree together, not one selectable scenario.

5. **Raw paired summaries are useful and appropriately descriptive.**
   Runner:126–193 retains sample order, argv, stdout/stderr, load, failures and
   raw CSV rows; CSV repetitions collapse to one per-block median, not invented
   independent samples. [report.py](/perf/experiments/report.py):33–64 matches
   baseline/candidate within job/block/metric, rejects mismatched fingerprints,
   retains failed/unpaired status, reports median/range/MAD and explicitly no
   significance or causal load adjustment. Six matched blocks is a heuristic,
   not a statistical assurance. Old `perf/report.py`:19 accepts only
   `elapsed_ns`, whereas sync/leaf/checked use `ns`; duplicate `combine()` and
   schema-specific drivers remain. Keep old schemas intact; normalize only new
   summaries. There is not yet one established report for both new and old data.

6. **JW adapter is narrower than the takeover needs.**
   [jw_sample.py](file:///home/rektide/src/jj-waltz/tools/performance/jw_sample.py):66–127
   initializes owned JJ state, overrides donor-local freshen config, and overlays
   reset payloads before one native reseed. Setup is outside its CSV timer.
   Verification checks payload bytes/link/mode and tracked tree against the
   target (143–169). **Clean inode/mtime changes are only counted and written,
   never rejected** (163–169), despite the handoff's hard zero-change property.
   It does not assert final target commit IDs, and feature-file requests can be
   silently underfilled by `eligible[:args.feature_files]` (110–118). No dedicated
   adapter unit tests were observed. Final native smoke evidence passed 16/20
   matrix attempts; **all four N2/feature attempts failed tracked verification**
   with `worker-0: M tracked.txt`. The writer calls this an unresolved fixture/
   target semantic mismatch, not a demonstrated production bug. N1 unchanged,
   initialization, sparse and subtree passed tiny smokes, using the same baseline
   executable under both labels. The supplied recipe is unchanged N=1 only and
   has a placeholder candidate revision. First-fill/sparse/multi-target support
   exists in pieces, not as a verified integrated campaign.

7. **Cross-repository sharing is brittle, not duplicated.**
   [JW launcher](file:///home/rektide/src/jj-waltz/tools/performance/experiment.py):9–16
   loads a sibling Python engine or `DIFF_DIRS_PERF`; the sample adapter separately
   defaults a sibling engine path (jw_sample:181). Recipe setup uses absolute
   workstation paths. Adapters import `measure.py` and shared `verify_tree.py`,
   so JW is not independently runnable without that checkout. Runner hashes its
   own modules and configured job inputs, a good safeguard; script paths and
   environment do not yet constitute a portable installed tool/version contract.
   Generated `.pyc` additions are build byproducts, not maintained source; their
   disposition belongs to their writer, not this read-only assessment.

## Keep, consolidate, stop

**Keep:** maintained Rust runners, existing Cargo optional backend controls,
owned marker/refusal rules, full outside-timing validation, exact frozen source
and binary records, raw failed attempts, native-vs-instrumented separation,
process-count tests and small audit scripts. Preserve the Python prototype as a
checkpoint/specification; do not discard it for being Python.

**Consolidate only the repeated responsibility:** one small Rust-owned execution
path should own scenario preparation/reset/verification, one-operation timing,
balanced variant invocation and append-only raw sample output. Initially this
can be a focused Cargo example/tool in diff-dirs, using its `sync` crate and
`parallel` feature; retain `parallel-bench`/`parallel-diagnostics` as separate
controls, never time diagnostic instrumentation. Preserve baseline and candidate
as separately built, hash-checked executables; no need to import two production
crate revisions into one giant Cargo graph or change public APIs first. Use
configured Rust 1.101 nightly, same profile/compiler for both.

Do **not** first port all 1,100 prototype lines to Rust. The cohesive Rust base is
the stateful scenario/sample execution contract, not a registry/config language.
A short explicit build/hash command list and small descriptive report script are
acceptable adapters. Once that narrow path works, schedule balanced samples in
that executable or reuse the existing tiny scheduler temporarily—never grow a
second engine alongside the first. Hyperfine remains optional whole-process
cross-check; it cannot time sub-millisecond API scopes or provide interleaving
by itself. The prior audit verified those capabilities; no new probes needed.

Minimum per-repo surface: D exposes owned comparison/sync setup, single scenario,
timer and result verification; J exposes isolated JJ setup, one actual native
`jw reseed`, and payload/target/inode verification. JJ semantics stay in J, not
inside a generic fixture platform. Avoid adding a production Cargo dependency
or public API just for benchmark orchestration. Existing `measure.py` remains
the subprocess-count regression instrument until a concrete replacement earns
its removal; do not recast it as the new latency engine.

**Stop expanding:** Python catalog/manifest subcommands, arbitrary build/plugin
support, new runtime/plan layers, repeated fixture generators and parallel
statistics implementations. No scratch deletion or automatic cleanup. No
application tuning justified by plumbing smoke tests.

## Small staged takeover brief

1. **Settle continuation ownership.** Sol high has checkpointed and wrapped;
   retain `997183df` / `a35da35d`, its reported checks and failures, and select
   ONE continuation owner. No broad refactor required.
2. **Next implementation slice: one clean-control proof.** Expose/use one guarded
   Rust single-operation clean runner on a tiny owned fixture, preserve exact
   baseline/candidate artifacts, and emit balanced paired blocks. Acceptance:
   clean zero operations; corrupted artifact refused; failed sample retained
   and excluded from speed conclusions; identical seed replays order; reversed
   positions balance; fixture unchanged; setup/verification excluded and costs
   retained. Reuse frozen controls where possible, without overwriting them.
   Repairing the existing frozen adapter narrowly is acceptable before deciding
   what orchestration actually needs migration; no all-purpose engine rewrite.
3. **Independent library reassessment.** Original Sol xhigh consumes that proven
   path for baseline/default/enabled-noop/name-check controls, serial and P4,
   then first-fill/sparse/subtree on owned real-shaped payload. Keep negative
   results and both API/process scopes separate. Under unavoidable host load,
   preserve complete chronology, raw rows and paired spreads; report inconclusive
   when drift dominates. No fastest-run selection or promise of host isolation.
4. **Then whole JW reseed.** Compare preserved `jw-before` with the exact integrated
   checkpoint on first-fill/clean/sparse, N=1 and multiple destinations, including
   distinct feature target trees. First diagnose the retained N2/feature tracked
   mismatch on its tiny fixture; do not change application code merely to make
   the adapter pass. Acceptance: byte/link/appropriate metadata
   payload checks, unchanged intended target identities/trees, **asserted** zero
   clean regular-payload inode/mtime changes, exact requested feature count,
   private reset equivalence and no per-data-file process growth. Report actual
   native reseed latency, not library timing as an end-to-end proxy.
5. **Real shape, owned state only.** `~/a/a/opencode` is live/read-only; snapshot
   selected tracked paths and `node_modules`, `**/node_modules`, `**/dist` into a
   NEW owned fixture with private reflinks and preserved metadata, no `.git`/`.jj`
   or freshen/install actions. Never run `jw reseed`, even dry-run, against the
   live donor. Source copy is non-quiescent: record census/provenance and verify
   the resulting owned fixture before campaign use. Retain setup/reset/hash I/O
   costs and warm-cache label. No automatic pruning, cleanup or claim that a
   tiny synthetic smoke establishes opencode-scale performance.

This is a takeover boundary, not authorization for those measurements or a new
implementation commission. Parent can link this document from the maintained
reading path; shared indexes were intentionally left unchanged.
