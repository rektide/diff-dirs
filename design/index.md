---
type: Index
title: diff-dirs design and implementation evidence
description: Reading paths through consumer fit, I/O research, implementation decisions, and measured performance.
resource: /design/index.md
status: maintained
generated: { by: "model:openai/gpt-6-astra", at: 2026-10-18 }
sources:
  - resource: "opencode://ses_eb37f1883ffeZ8HgL0aFBFJf4R"
    title: Added the post-handoff bounded-directory-frontier design proposal to the reading path
  - resource: "opencode://ses_eb39c93e9ffeNju5a7XEsB3wGY"
    title: Coordinated research and implementation, reviewed the measured concurrency limits, and assembled the reading index
  - resource: /design/fit/fit0.glm53m.md
    title: Consumer fit and corrected reseed integration requirements
  - resource: /design/fit/io0.glm53m.md
    title: General filesystem options and source-referenced jj-waltz execution map
  - resource: /design/fit/rayon0.glm53m.md
    title: Shared pools and bounded traversal-head research
  - resource: /design/implementation/implementation0.sol61x.md
    title: Implemented comparison and CLI contract
  - resource: /design/performance/parallel0.sol61x.md
    title: Measured live-tree scaling and the remaining concurrent traversal work
---

# Design and implementation evidence

The [root specification](../README.md) owns the current public contract.

## Consumer fit and I/O

- [Fit assessment](fit/fit0.glm53m.md): jj-waltz's payload seam, the earlier
  deletion that integration must address, and initial contract contradictions.
- [I/O assessment](fit/io0.glm53m.md): std, async wrappers, filesystem libraries,
  and the concrete sequential jj-waltz execution path. Jujutsu itself is a
  subprocess, not an embedded runtime dependency.
- [Rayon research](fit/rayon0.glm53m.md): caller-pool sharing, bounded work,
  deterministic output, and the initial multi-head design analysis.
- [Parent implementation decisions](fit/decisions0.gpt6a.md): the original
  build brief and subsequent generic sync/shared-pool direction. The narrow
  per-frame scheduling choice is under redesign following live measurements.

## Implemented baseline and measurements

- [Implementation account](implementation/implementation0.sol61x.md): generic
  diff state machine, native filenames, summary coverage, CLI, and tests.
- [Parallel measurements and handoff](performance/parallel0.sol61x.md):
  same-build sequential versus 1/2/4/8-worker results on synthetic and live
  trees; explicit limitations of current-frame-only concurrency. The separate
  sync crate is still pending.
- [Initial performance evidence](performance/performance0.sol61x.md): fixture
  census, live-tree census, environment, and the sequential baseline.
- [Reproduction commands](../perf/README.md): harness usage and retained raw
  observations.

## Concurrency redesign — proposed for review

- [Bounded directory frontier](concurrency/design0.gpt6am.md): separates parallel
  directory inspection from deterministic DFS commitment; specifies explicit
  caller-pool borrowing, divergence seals, demand-reserved admission bounds,
  prefix summaries, joined epochs, and discriminating verification experiments.
  Revisits the earlier per-frame choice; not implemented or performance-proven.
