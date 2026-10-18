---
type: EvidenceBundle
title: Static checked-streaming controls
description: Same-compiler owned-fixture raw observations, benchmark source, exact commands and identities for the initial checked-execution checkpoint and its follow-up.
resource: /perf/results/checked/README.md
status: measured-initial-checkpoint
generated: { by: "model:openai/gpt-6.1-sol#xhigh", at: 2026-10-18 }
sources:
  - resource: "opencode://ses_eb24f98a3ffewdVqa9lao6tXQu"
    title: Built identical baseline/candidate controls, collected all observations on owned fixtures and preserved the initial enabled-check overhead before tuning
  - resource: /design/performance/checked0.sol61x.md
    title: Method, measured table, interpretation and limits
---

# Checked-streaming evidence

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
