---
type: ResearchReport
title: Performance tooling audit — ad hoc scratch vs maintained harness
description: Inventory of .test-agent perf/sync/optimization/leaf-pipeline scratch and tracked harness machinery, verified hyperfine/perf/strace capabilities against actual workload sizes, hazards, and a staged promotion path — audit only, no tool changes performed.
resource: /design/performance/tooling-audit0.glm53fx.md
status: draft
generated: { by: "model:zai-coding-plan/glm-5.3-flash#max", at: 2026-10-18 }
sources:
  - resource: "opencode://ses_eb2b25eb7ffe8EMkSQFlGkfQDc"
    title: Audited scratch tooling and tracked harness read-only, probed installed hyperfine/perf/strace semantics, and proposed a staged promotion workflow without changing any tool or running benchmarks
  - resource: /perf/README.md
    title: Maintained harness method — fixtures, scenarios, interleaved controls, syscall audit, resources, owned sync benchmarks
  - resource: /examples/perf.rs
    title: Comparison runner — setup/shape/run/once/repeat/inspect plus hidden compare/diagnose/backend-once
  - resource: /sync/examples/sync_perf.rs
    title: Marked-owned-fixture sync benchmark — marker guard, rotation, four hardcoded scenarios
  - resource: /perf/leaf.rs
    title: Active leaf-pipeline probe — owned setup, five scenarios, process CPU clock, private targets
  - resource: /perf/leaf.py
    title: Active interleaved variant driver — rotation, confirm (wait4 resources), isolate modes
  - resource: /perf/compare.py
    title: Sequential vs 1/2/4/8 pool matrix driver with CSV combining
  - resource: /perf/frontier.py
    title: Read-only interleaved P:J:K matrix driver
  - resource: /perf/resources.py
    title: Child rusage and sampled VmHWM observer
  - resource: /perf/audit.py
    title: strace fixture-payload IO audit
  - resource: /perf/report.py
    title: elapsed_ns-schema CSV summarizer
  - resource: /perf/results/leaf/README.md
    title: Leaf evidence bundle — variants, executables.json hashes, fixture shapes, timing boundaries
  - resource: /design/performance/leaf0.sol61x.md
    title: Controlled leaf measurements and negative cases being reviewed concurrently
  - resource: /design/performance/comparison0.gpt6a.md
    title: Parent synthesis of the two optimization inventories
  - resource: /design/performance/inspection0.sol61x.md
    title: Syscall anatomy and toolchain provenance sections
  - resource: /perf/results/sync/README.md
    title: Historical Rust 1.88 sync evidence bundle
  - resource: /.test-agent/perf/README.md
    title: Scratch perf directory self-description
  - resource: /.test-agent/optimization/README.md
    title: Owned syscall probe method and Rust 1.88/nightly provenance split
  - resource: /.test-agent/leaf-pipeline/README.md
    title: Active leaf-pipeline scratch scope statement
  - resource: /design/index.md
    title: Reading path that a future entry should join
---

# Performance tooling audit

Audit of the ad hoc tooling under `.test-agent/` against the tracked harness,
and of what external benchmark tooling actually offers for this project's
workloads. Commissioned as research only: nothing here was rewritten, cleaned
up, deleted, or newly benchmarked. The concurrently active leaf-pipeline work
(`ses_eb2c07f5cffe2uTLIC7b1rrD99`) was observed read-only; no performance
campaign was run. Live roots `/home/rektide/a/a/opencode` and
`/home/rektide/src/opencode-export-qa` were only stat-checked as existing.

Everything claimed about installed tools below was verified against the local
binaries (`--version`, `--help`, and tiny `/tmp/opencode` behavior probes), not
recalled from memory. Everything claimed about workload sizes comes from
already-retained CSVs and evidence bundles, not new runs.

## 1. Inventory

Tracked, maintained machinery (not scratch — keep and consolidate):

| Path | Capability | Maintenance worth |
| --- | --- | --- |
| `examples/perf.rs` | Comparison runner: `setup`/`shape` (owned, refuse existing roots), `run ROOT N [scenario]` (single-scenario selector exists here), `once`, read-only `repeat`, `inspect` census; in-process timing, untimed warmup, scenario mutation/restoration outside timing, expected-result validation; hidden `compare`/`diagnose`/`backend-once` behind `parallel-bench`/`parallel-diagnostics` | High. The comparison-side runner is settled; do not rewrite |
| `sync/examples/sync_perf.rs` | Owned-fixture sync benchmark: `.diff-dirs-sync-benchmark` marker guard, private `tempfile` targets, one warmup per config, rotation of serial/P4 × Copy/ReflinkRequired, scenarios `fresh/clean/sparse/subtree`, diff + completed-operation counts. No single-scenario selector: `run` always executes all scenarios × configs | High, with one known gap (see §2-H8) |
| `perf/leaf.rs` (tracked, active) | Leaf-pipeline probe: `setup ROOT DIRS FILES SIZE` with `OWNED` marker, `run ROOT POLICY EXECUTION` over hardcoded `fresh/dense/clean/sparse/diff-only`, `CLOCK_PROCESS_CPUTIME_ID` CPU including pool workers, private targets, independent core verification outside timing | High. This is the current commission's instrument, not junk |
| `perf/leaf.py` (tracked, active) | Interleaved driver over saved same-build variant binaries; rotates variant and task order per round; `confirm` mode (3-round endpoint confirmation + `os.wait4` whole-process CPU/RSS/wall); `isolate` mode (staging-only vs final descriptor pricing) | High while leaf work is open |
| `perf/compare.py` | Matrix driver: sequential + pools 1/2/4/8 over `--fixture` (mutating) and `--pair` (strictly read-only); combines per-cell CSVs. No order rotation | Medium. Superseded for frontier work by `frontier.py`; still the 1/2/4/8 scaling path |
| `perf/frontier.py` | Read-only P:J:K config matrix; rotates pair order between configurations; combines CSVs | High |
| `perf/resources.py` | One command's child rusage (exact CPU incl. startup) + 5 ms-sampled VmHWM | Medium. Two RSS methods coexist with `leaf.py confirm` (§2-H9) |
| `perf/audit.py` | Asserts an fd-annotated strace shows directory-only fixture opens, zero payload reads/mmaps | Medium-high. Cheap correctness guard for the read-only claim |
| `perf/report.py` | Median/min/max/mean/pstdev summarizer; requires `elapsed_ns` schema | Medium, currently schema-locked (§2-H3) |
| `perf/README.md`, `perf/results/**` | Method + curated evidence bundles (`btrfs-*.csv`, `frontier/`, `sync/`, `leaf/` incl. `executables.json` SHA-256 map) | High. This bundle discipline is the project's provenance backbone |

Scratch (gitignored via `/.test-agent/`; sorted by what it is, not junk vs not):

| Path | What it is | Maintenance worth |
| --- | --- | --- |
| `.test-agent/leaf-pipeline/` (192 MB, active session `ses_eb2c07f5cffe2uTLIC7b1rrD99`) | Scratch cargo manifest (`[[bin]] path = "../../perf/leaf.rs"`, retained `Cargo.lock`), seven saved same-build variant binaries (~1.1 MB each: `unchanged/cleanup/descriptor/shared/handoff/staging-only/final`), OWNED fixtures (`single/shallow8/dense1k/large/tiny/tree30k`), raw `observations.csv`/`confirmation.csv`/`descriptor-isolation.csv`, `resource-*.{stdout,stderr}`, strace files, isolated `build/` target dir | Reproduction evidence for the leaf campaign. Curated copies already tracked in `perf/results/leaf/`. Retain until leaf review lands; then deliberate cleanup decision. The manual "copy binary to variant name before each edit" step is the fragile part (§2-H1) |
| `.test-agent/optimization/` | `setup.py` + `src/main.rs` baseline-API syscall probes (`metadata/file-type/copy-io/copy-fs/sync-copy/sync-reflink`), OWNED fixture, ~12 retained strace files split Rust 1.88 vs 1.101-nightly provenance | Conclusions already promoted into `inspection0.sol61x.md`; keep as evidence and as the template for future "what does std actually do" probes. No timing role |
| `.test-agent/perf/` (incl. 9.7 MB `btrfs-192k` fixture) | Generated metadata fixtures (`btrfs-30k/192k`, `frontier-small/skewed/dense/wide`), raw per-cell CSV/stderr matrices from `compare.py`/`frontier.py`, `io.trace`, `census.txt`, `audit.csv` | Regenerable fixtures + raw provenance for already-curated `perf/results/`. Cleanup candidates once campaigns close; nothing here is the only copy of a promoted conclusion |
| `.test-agent/sync/` (119 MB) | Superseded 30k nonempty sync fixture (`btrfs-30k`) + `final-smoke` + raw `results.csv` from the Rust 1.88-era campaign | Curated copy is `perf/results/sync/`. Fixture shape was regenerated fresh for the leaf campaign (`leaf0` used its own `tree30k`); deliberate deletion candidate after parent confirms no re-measurement need — not silently |
| `.test-agent/sync-tests/` | Explainer: tests create private reflink-capable fixtures here | Doc only |

Duplicate capabilities worth consolidating (not deleting): four fixture
generators (`perf.rs setup/shape`, `sync_perf setup`, `leaf.rs setup`,
`optimization/setup.py` — each with a different shape, which is legitimate, but
three re-implement "dirs × files × one symlink" in Rust); three Python drivers
with three different rotation policies and two private copies of CSV `combine()`
(`compare.py`, `frontier.py`; `leaf.py` has its own interleave logic); two RSS
methods (`resources.py` sampling, `leaf.py confirm` `wait4`); three CSV schemas
with one schema-locked summarizer.

## 2. Top reproducibility and scope hazards

Verified references given; ordered by practical risk to future results.

1. **Manual variant-binary labeling.** `perf/results/leaf/README.md`: "Before
   each next production edit: copy the built binary to its variant name." A
   forgotten copy silently mislabels a variant; the `executables.json` SHA-256
   map records identity but nothing asserts it at run time. Cheap fix (driver
   loads the map and hashes before each campaign); high payoff for trust.
2. **External drivers cannot interleave.** Verified live: hyperfine 1.20.0
   executes commands grouped (`A A A B B B` probe); `--sort` affects display
   only; no randomization/rotation option exists. The project's own discipline
   (`perf.rs compare` rotates backends per iteration; `leaf.py` rotates variant
   and task order per round) is *stronger* than what hyperfine offers. Any
   proposal to hand orchestration to hyperfine loses interleaving — keep
   rotation in the project's drivers.
3. **CSV schema fragmentation.** `report.py` requires `elapsed_ns` + `sample`;
   `sync_perf` emits `ns,…`; `leaf` emits `fixture,policy,execution,scenario,ns,cpu_ns,…`.
   One summarizer serves one schema; the rest is analyzed ad hoc (unreproducible).
   Small consolidation, no rewrite of retained files needed (§6).
4. **Fixture lifecycle has no reset — by design, but with teeth.** `setup`
   refuses existing roots everywhere; `optimization/README.md` states
   "deliberately no automatic reset or prune". An interrupted campaign leaves
   blocking roots; scratch now holds ~320 MB (leaf-pipeline 192 MB, sync 119 MB).
   The OWNED/marker guards (`OWNED`, `.diff-dirs-sync-benchmark`) correctly
   prevent mutation of non-owned trees — keep those; add an explicit,
   operator-invoked cleanup list rather than ad hoc `rm -rf`.
5. **Compiler provenance is documented but easy to violate.** `perf/results/sync/`
   is Rust 1.88.0; current configured toolchain is `rustc 1.101.0-nightly
   (822182db4 2026-10-13)`; `optimization/` traces deliberately preserve both
   generations. `leaf0` correctly treats old numbers as shape, not controls.
   The standing rule (record `rustc --version --verbose`, build candidate and
   control with the same compiler) lives in `README.md`/`perf/README.md` and is
   honored only by discipline today.
6. **Timing-scope mixing.** In-process timers exclude process startup, dynamic
   linking, and pool creation; `resources.py`/`leaf.py confirm`/hyperfine
   include them. `leaf`'s bundle already warns "do not treat those resource
   CPU/wall totals as the API timer". At 20 ms scale, ~1–2 ms of process
   overhead is ~5–10% — numbers from the two scopes must never be tabulated
   together without a column saying which scope produced them.
7. **Read-only claim enforcement is good but partial.** `repeat`/`compare`/
   `frontier.py` are read-only by construction, and `audit.py` proves it for a
   trace — but only for runs someone traces. Keep the audit as a per-campaign
   spot check, not a one-time proof.
8. **Sync-side runners have no single-scenario process mode.** `sync_perf run`
   and `leaf.rs run` always execute all scenarios per invocation (verified in
   source). `perf.rs` has `run ROOT N SCENARIO`. This blocks clean process-level
   timing of one scenario by any external driver and is the one small runner
   change this audit recommends (§6, decision-gated).
9. **Two RSS semantics.** `resources.py` samples `/proc` VmHWM at 5 ms (may miss
   a peak in the final gap, documented); `leaf.py confirm` uses `wait4`
   `ru_maxrss` (exact, whole process including launcher). Both are labeled, but
   a future reader can compare them as if equivalent. Standardize on `wait4`
   where the driver spawns the child; keep sampling only for multi-process
   children.
10. **hyperfine shell/prepare quirks.** Verified: `--prepare` executes before
    *warmup* runs too (2 warmup + 2 timed → 4 executions); with `-N`
    (`--shell=none`) shell syntax in `--prepare`/commands is not interpreted
    (an `echo x >> count` prepare silently ran `/bin/echo` with literal args);
    a <5 ms command with the default shell produces hyperfine's own accuracy
    warning. Also verified: non-zero exit aborts unless `-i`; for diff CLI use
    scoped `--ignore-failure=1` (differences found) so exit-2 IO errors still
    fail the benchmark.

## 3. Tool comparison and recommended minimal stack

Available locally (verified): `hyperfine 1.20.0`, `perf 7.2.8`
(`perf_event_paranoid=2` — unprivileged per-process counting works, verified
with `perf stat`), `strace 6.13`, `valgrind`, `bpftrace`. Not installed, and not
referenced anywhere in the tree: Criterion, Divan, cargo-flamegraph. Toolchain:
ordinary configured `cargo`/`rustc` 1.101.0-nightly (29c5daa1a / 822182db4);
Rust 1.88 historical reports are provenance only.

Actual workload magnitudes (from retained data, not new runs):

| Measured scope | Retained magnitude | Source |
| --- | --- | --- |
| `prune` scenario | ~10 µs | `perf/results/btrfs-30k.csv` |
| Single-leaf sync (`sparse`, `single`) | ~0.18–0.9 ms | `leaf0` negative cases |
| 30k no-op walk | ~21–25 ms (tree30k controls) to ~69–77 ms (btrfs-30k fixture) — shape-sensitive | `leaf0`; `perf/results/btrfs-30k.csv` |
| Live 191k node_modules diff | ~750–850 ms serial; ~264 ms median frontier P4/J4/K256 | `perf/results/frontier/live.csv` |
| Fresh 30k sync (120 MB payload) | ~1.7–1.9 s unchanged; ~0.88–1.0 s final candidate | `leaf0` + `.test-agent/leaf-pipeline/confirmation.csv` |

What this means for the user's actual question — "hyperfine might be for too
small tasks":

- Hyperfine's floor is the process: with `-N` a Rust binary costs ~0.3–1 ms of
  exec + runtime init before any measured work, and hyperfine warns below 5 ms
  with the default shell. So `prune` (~10 µs) and single-leaf scenarios
  (~0.2–0.9 ms) are categorically out of hyperfine's reach — not because
  hyperfine is weak but because the workload is smaller than its fixed cost.
  The in-process scenario loop (one process, warmup, CPU clock, parity checks)
  is the only honest instrument there, and the project already has it.
- At ≥ ~50 ms (30k walks, live walks, fresh sync), whole-process timing is a
  2–5% overhead at most, and hyperfine becomes genuinely useful: solid
  statistics with outlier detection, per-run JSON, per-run child peak RSS
  (`memory_usage_byte` — verified in the JSON export), child user/system time,
  `--setup/--prepare/--cleanup` lifecycle, parameterization via `-L`, and
  `$HYPERFINE_ITERATION` for per-run logs.
- What hyperfine verifiably does *not* do, so nobody expects it to: no
  interleaving/rotation (grouped execution); no pinning, cache dropping, or
  host isolation; no in-process/per-record counters; no output-parity checking
  (exit codes only); `--prepare` before warmups means it cannot create a
  "warmup-only" state distinction. It measures the process, full stop.

Recommended minimal stack (no new frameworks, nothing installed):

| Layer | Tool | Role |
| --- | --- | --- |
| Timed scope | Existing Rust runners (`perf.rs`, `sync_perf.rs`, `leaf.rs`) | The only source of "measured scope" numbers: in-process timers, CPU clock, fixture guards, parity/verification outside timing |
| Orchestration/statistics | Existing Python drivers (`leaf.py`, `frontier.py`, `compare.py`) | Interleaving/rotation, raw CSV retention, resource confirmation; keep rotation here, never delegate it |
| Cross-check | `hyperfine -N` | Optional whole-process A/B ≥ 50 ms with exported per-run JSON/RSS; documented grouped-execution caveat; reverse command order in a second invocation as manual decorrelation |
| Attribution | `perf stat -d -r N`; `strace -c` / `audit.py` / `leaf_trace.py` | Why a number moved (instructions, faults, syscalls) — separate runs, never mixed into timing tables |
| Explicitly not adopted | Criterion/Divan, cargo-flamegraph, valgrind tools, bpftrace | In-process micro-bench frameworks re-solve what the runners already do (fixture state, parity, fs identity would be custom code anyway); profiling tooling only on a concrete question — flagging these as overengineering choices to *not* make now |

The "external wrapper + small Rust runner" shape the commission anticipated is
exactly what exists: wrappers can replace statistics/orchestration, but
fixture setup, guarded owned state, scenario semantics, and verification stays
in small Rust runners. No magic hyperfine feature substitutes for any of it.

## 4. Proposed workflows (illustrative commands — none executed in this audit)

Read-only live diff, whole-process A/B cross-check (both binaries same
compiler; `perf once` always exits 0, so no `-i` needed):

```sh
cargo build --release --example perf --features parallel --target-dir target/perf-ctrl
# candidate: same command in the candidate worktree, own target-dir
hyperfine --style basic -N -w 2 -r 10 \
  -n control   'target/perf-ctrl/release/examples/perf once ORIG TGT' \
  -n candidate '…/target/perf-cand/release/examples/perf once ORIG TGT' \
  --export-json .test-agent/perf/hf-once.json
# repeat once with the two commands reversed; keep both JSONs (manual decorrelation)
```

Benchmarking the actual CLI on a divergent pair (exit 1 = differences):

```sh
cargo build --release -p diff-dirs-cli
hyperfine --style basic -N -w 1 -r 10 --ignore-failure=1 \
  'target/release/diff-dirs ORIG TGT --format text' \
  --export-json .test-agent/perf/hf-cli.json
# scoped -i keeps exit 2 (usage/IO) fatal; JSON output cost is part of the CLI's job
```

Interleaved candidate-vs-control (already maintained — preferred for tight A/B;
shown unchanged, for contrast):

```sh
target/frontier-timing/release/examples/perf compare ORIG TGT 4 5 4 256 live-node
python3 perf/frontier.py --binary target/frontier-timing/release/examples/perf \
  --pair live-node ORIG TGT --config 4:4:256 --output .test-agent/perf/fm -n 5
```

Owned sync fixture under hyperfine (mutating; requires the §6 single-scenario
selector first — today `run` executes all five scenarios per process, so
hyperfine would time the bundle, not a scenario):

```sh
hyperfine --style basic -N -w 1 -r 5 \
  -n unchanged '.test-agent/leaf-pipeline/unchanged run tree30k strict p4 fresh' \
  -n final     '.test-agent/leaf-pipeline/final     run tree30k strict p4 fresh' \
  --export-json .test-agent/perf/hf-leaf-fresh.json
```

Attribution, separate from any timing table:

```sh
perf stat -d -r 3 target/perf-ctrl/release/examples/perf once ORIG TGT
strace -c -f -e trace=%file,getdents64,statx,readlinkat \
  target/perf-ctrl/release/examples/perf once ORIG TGT
```

Standing rules carried into any of these: record `rustc --version --verbose`;
build candidate and control with that same compiler into separate target dirs;
`-N` everywhere (and no shell syntax inside commands/prepare when `-N`);
fixtures only under `.test-agent/` with their OWNED/marker guards; live roots
only via read-only commands.

## 5. Promotion and replacement map

| Today | Becomes | Stays as-is / evidence |
| --- | --- | --- |
| `perf/leaf.rs` + `perf/leaf.py` (tracked, active) | Maintained owned-fixture sync runner + interleaved driver after the leaf review lands; add single-scenario mode, `wait4`-based resources, hash assertion against `executables.json` | During the campaign: untouched |
| `sync/examples/sync_perf.rs` vs `perf/leaf.rs` | One consolidated sync-side runner (decision D2; scenarios/marker scheme merge cleanly) | The other remains as historical instrument for its recorded bundle |
| `examples/perf.rs` | No change — comparison runner is settled | Hidden `compare`/`diagnose` controls stay hidden behind features |
| `perf/report.py` | Schema-tolerant (`elapsed_ns` or `ns`) summarizer; shared `combine()` helper used by all three drivers | Retained CSV files are never re-headed |
| `hyperfine` | Documented optional cross-check in `perf/README.md` with the verified semantics/limits of §2-H10/§3 | Not the statistics engine for interleaved campaigns |
| `perf stat`/`strace -c` | Documented attribution recipes next to the existing `audit.py`/trace practice | Separate runs; never inside timing tables |
| `.test-agent/optimization/` | Template for future baseline-API probes | Keep traces + Rust 1.88/nightly provenance split as evidence |
| `.test-agent/perf/` | Regenerable | Raw matrices remain provenance for curated `perf/results/`; fixture deletion is a later explicit decision |
| `.test-agent/sync/` (119 MB) | — | Superseded shape; deliberate deletion candidate after parent confirms (D3), never silent |
| `.test-agent/leaf-pipeline/` (192 MB) | — | Active-campaign reproduction evidence (saved variant binaries are the point); cleanup only after the leaf work is accepted or rejected (D3) |

## 6. Staged next work and decision points

No time estimates; each stage is coherent and independently committable.
Nothing below has been performed.

1. **Docs-only (safe, recommended first):** add a "Tooling" section to
   `perf/README.md` — workload-size→instrument table (§3), verified hyperfine
   semantics and limits, `perf stat`/`strace -c` attribution recipes, the
   timing-scope warning (§2-H6). No code, no results claims.
2. **Runner gap (small, decision-gated):** add a single-scenario selector to
   the sync-side runner (`run ROOT POLICY EXEC SCENARIO`), mirroring
   `perf.rs run ROOT N SCENARIO`. **D1 — user chooses:** extend `leaf.rs` in
   place, or fold `leaf.rs` into `sync_perf.rs` first (D2), or skip (current
   five-scenario bundle stays the only interface; hyperfine cross-check then
   applies only to the diff side).
3. **Driver hygiene (small):** schema-tolerant `report.py`; factor the two
   `combine()` copies into one helper; assert variant-binary SHA-256 against
   `executables.json` at the start of `leaf.py` campaigns; standardize on
   `wait4` resources where the driver spawns the child (§2-H9).
4. **Scratch lifecycle (decision-gated, after the leaf campaign closes):**
   **D3 — user chooses per path:** delete regenerable fixtures
   (`.test-agent/perf` trees, `.test-agent/sync` 119 MB) vs keep as evidence;
   delete or retain `leaf-pipeline` binaries/CSVs after the review verdict.
   An explicit checklist in `.test-agent` READMEs would make this repeatable;
   no automatic prune anywhere.
5. **Not now (explicitly deferred):** Criterion/Divan, flamegraph tooling,
   a general matrix DSL, compiler pinning, allocator instrumentation. Revisit
   only on a concrete question (e.g., a real RSS regression → dhat/valgrind).

**Decision points for the user:** D1/D2 (runner consolidation shape), D3
(scratch retention), and whether hyperfine's role should be "documented
cross-check" (recommended) or also the default quick-compare habit for
≥50 ms whole-process questions.

## Suggested index entry (not applied — index is concurrently maintained)

For `design/index.md` under "Preserved baseline and measurements" or the
performance-opportunities section:

> - [Performance tooling audit](performance/tooling-audit0.glm53fx.md):
>   scratch-vs-maintained inventory, verified hyperfine/perf capabilities
>   against actual workload sizes, reproducibility hazards, and a staged
>   promotion path. Audit only; no tool changes.
