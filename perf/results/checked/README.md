---
type: EvidenceBundle
title: Static checked-streaming controls
description: Same-compiler owned-fixture raw observations, benchmark source, exact commands and identities for the initial checked-execution checkpoint and its follow-up.
resource: /perf/results/checked/README.md
status: measured-initial-checkpoint
generated: { by: "model:openai/gpt-6.1-sol#xhigh", at: 2026-10-18 }
sources:
  - resource: opencode://ses_eb2165cc8ffe4vOmOvDBe7aXQu
    title: Distinguished immutable historical Python source snapshots from current Rust reproduction
  - resource: "opencode://ses_eb24f98a3ffewdVqa9lao6tXQu"
    title: Built identical baseline/candidate controls, collected all observations on owned fixtures and preserved the initial enabled-check overhead before tuning
  - resource: /design/performance/checked0.sol61x.md
    title: Method, measured table, interpretation and limits
---

# Checked-streaming evidence

The Python files here are immutable **historical experiment source snapshots**,
not supported entrypoints or dependencies. Historical reproduction commands below
describe those recorded campaigns only. Current baseline/default/noop/admin
reproduction uses [checked-in Rust controls](/perf/runner/README.md).

`initial.csv`: 480 rows (80 round-zero warmup, 400 retained), 10k × 4 KiB source
files, 100 package dirs/links, serial/P4 and Copy/ReflinkRequired. `initial-commands.json`
retains exact command invocations and process resource observations. These process
resources include setup/verification/cleanup, unlike each CSV operation timer.
`initial-identity.json` records baseline `fd277144`, candidate `8658b6da`, compiler
and executable hashes. `compiler.txt`/`environment.txt` record actual environment.

`initial-bench.rs` and `initial-run.py` are the exact scratch experiment source,
not new product APIs. To reproduce that historical checkpoint, put the benchmark
at `.test-agent/checked/bench.rs`, runner at `.test-agent/checked/run.py`, and the
two Cargo manifests under `.test-agent/checked/{before,after}-runner/Cargo.toml`.
Extract the baseline git archive into `.test-agent/checked/before-source/`, and
select candidate `8658b6da` in a separate prepared checkout. Do not overwrite an
active checkout or reuse/delete another owner's fixture. Include a scratch README.

```sh
cargo build --release --manifest-path .test-agent/checked/before-runner/Cargo.toml --target-dir target/checked-control-before
cargo build --release --features checks --manifest-path .test-agent/checked/after-runner/Cargo.toml --target-dir target/checked-control-after
python3 .test-agent/checked/run.py
```

The runner creates only a nonexistent marked fixture and privately owned targets.
It runs no builds during measurement and never synchronizes arbitrary live roots.
Do not run simultaneous tests/builds or another mutating matrix while measuring.

## Corrected controls and real payload

- `optimized.csv`, identity, commands, benchmark and runner: same matrix at the
  lazy-path checkpoint `243be8e2`; 480 rows including 80 warmup rows. The older
  baseline code is unchanged; only candidate-only guard syntax in the common
  benchmark changes to direct `Entry::name`.
- `clean.csv`, `clean-run.py`, commands and identities: 210 interleaved read-only
  clean observations on one shared owned target. The initial executable was
  preserved separately; copying it for reproduction must preserve executable mode.
- `live-census.txt`: fresh read-only actual donor/feature census, not old counts.
- `live.csv`, identity, commands: actual 191,805-entry donor used only read-only;
  every modified target is created under a new marked owned root. Four initial
  first fills plus 108 repeated rows (18 warmup, 90 retained). No live target is
  synchronized. Required reflink only; exact bytes/entries are reported.
- `live-bench.rs`, `live-run.py`, `live-{before,after}.Cargo.toml`: exact common
  real-shape experiment source/build inputs. Place `live-bench.rs` at scratch
  `live.rs`, then run the manifest's `checked-live` binary. The source chooser
  skips symlinks and edits only paths inside its newly cloned owned target.

The real donor resolves from `~/a/a/opencode/node_modules` to
`/home/rektide/archive/anomalyco/opencode/node_modules`; see the identity log.
Setup creates a **new** marked scratch root before recording that canonical
read-only source. Never repurpose the script to write to a feature workspace.
The `fill` command creates its own temporary target; `measure` uses only its
marked root's owned `control-target`. Initialization, source-path selection,
post-sync verification and target cleanup are outside each operation timer.
