---
type: DataBundle
title: Directory-frontier measurement observations
description: Raw same-build interleaved controls, guided admission sweeps, live and owned topology scopes, and separately instrumented physical/resource observations.
resource: /perf/results/frontier/README.md
status: measured
generated: { by: "model:openai/gpt-6.1-sol#xhigh", at: 2026-10-18 }
sources:
  - resource: opencode://ses_eb3948bedffe0T37Xc2wI2S5Jg
    title: Ran and promoted read-only live/frontier comparisons and owned fixture measurements, preserving separate instrumented observations
  - resource: /perf/README.md
    title: Reproducible timing, topology, diagnostics, and resource methods
  - resource: /perf/frontier.py
    title: Configured pair driver and unmodified raw CSV consolidation
---

# Frontier observations

CSV rows preserve raw nanoseconds, first-record latency (blank when clean),
complete difference/coverage counts, and explicit backend/P/J/K selection.
Each compared backend's full record vector and summary were checked outside
timing; backend order rotates within each process. These are warm-cache,
nonisolated-host observations, not cold-cache or network-filesystem guarantees.

- `live.csv`: initial supplied node_modules P/J/K curve, five observations per
  backend/configuration.
- `shapes.csv`: small-directory, skewed, dense, root-wide, legacy 192k synthetic,
  and whole-live-checkout scopes at P2/J2/K8, P4/J4/K16, P4/J4/K256.
- `guided.csv`: live node_modules K32/K128 at P2; J2/J1 at P4; independent P4/K256
  repeat. Each configuration has five observations per backend.
- `confirmation.csv`: final-build live node_modules/whole-tree repeats, including
  demand-only K=0 and K=256.

Live scope is always original `/home/rektide/a/a/opencode` against target
`/home/rektide/src/opencode-export-qa`, or their top-level `node_modules` pair.
No fixture setup, copy/sync, pruning, or mutation command was run on those roots.
Owned topology fixture creation is separate from every timed scan.

Physical scheduler diagnostics and process resource observations are separate
from uninstrumented timing CSV. Their commands, executable hashes, census,
interpretation, and limitations belong in the accompanying measured account.

`diagnostics-*.txt` records thirteen separate instrumented runs, each with its
exact command, physical high-water/occupancy/worker counters, and committed
summary counts. `resources-*.txt` records six separate one-backend CPU and
sampled VmHWM observations. `census.txt` is the fresh four-root census.
See the [measured account](/design/performance/parallel1.sol61x.md) for executable
hashes, environment, commands, repeated wins, unfavorable cases, and review
choices. No timings from instrumented runs are substituted into timing CSV.
