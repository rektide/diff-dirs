---
type: Index
title: diff-dirs design and implementation evidence
description: Reading paths through consumer fit, I/O research, implementation decisions, and measured performance.
resource: /design/index.md
status: maintained
generated: { by: "model:openai/gpt-6.1-sol#xhigh", at: 2026-10-18 }
sources:
  - resource: "opencode://ses_eb3033c79ffeRIkn14uSNhs7O6"
    title: Added implemented filesystem synchronization, independent operation semantics, verification, and owned-fixture measurements to the reading path
  - resource: "opencode://ses_eb335933bffeUeaPkxqOtKDOWi"
    title: Linked the paused runtime refactor assessment and smaller two-crate option without changing the implemented contract
  - resource: "opencode://ses_eb3948bedffe0T37Xc2wI2S5Jg"
    title: Linked accepted frontier implementation, bounds/liveness verification, and measured single-root scale-out while preserving independent designs and baseline reports
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
   per-frame scheduling choice is superseded by the accepted frontier below.

## Preserved baseline and measurements

- [Implementation account](implementation/implementation0.sol61x.md): generic
  diff state machine, native filenames, summary coverage, CLI, and tests.
- [Parallel measurements and handoff](performance/parallel0.sol61x.md):
  same-build sequential versus 1/2/4/8-worker results on synthetic and live
  trees; explicit limitations of predecessor current-frame-only concurrency.
  Sync was still pending at that historical checkpoint.
- [Initial performance evidence](performance/performance0.sol61x.md): fixture
  census, live-tree census, environment, and the sequential baseline.
- [Reproduction commands](../perf/README.md): harness usage and retained raw
  observations.

## Directory frontier — accepted design, implemented checkpoint

- [Bounded directory frontier](concurrency/design0.gpt6am.md): separates parallel
  directory inspection from deterministic DFS commitment; specifies explicit
  caller-pool borrowing, divergence seals, demand-reserved admission bounds,
  prefix summaries, joined epochs, and discriminating verification experiments.
  Revisits the earlier per-frame choice. The independent proposal retains its
  original standing; parent acceptance and implementation evidence are recorded
  in the maintained specification and accounts below.
- [Frontier implementation](implementation/implementation1.sol61x.md): borrowed
  pools, reusable arena, global J/K admission, reserved demand, divergence seals,
  prefix-only summaries, completion permutations, lifecycle, and apply tests.
- [Frontier measurements](performance/parallel1.sol61x.md): interleaved controls
  on actual live roots and small/skewed/dense/wide fixtures; physical concurrency,
  budget saturation, CPU, sampled RSS, first-record/drain costs, and default-policy
  review choices. Includes a repeatable many-small-directory single-root win.
- [Raw frontier observations](../perf/results/frontier/README.md): all 495 timing
  observations plus separate physical diagnostics, resources, and fresh census.

## Runtime separation — scope under discussion, not implemented

- [Runtime seam assessment](implementation/refactor0.sol61h.md): recommends a
  pure engine module in the existing runtime-free library plus a separate Rayon
  crate; explains when a distinct pure-core package earns its cost and why a
  Tokio blocking-task adapter is not native asynchronous filesystem I/O. Retains
   the larger initial proposal, paused before code changes pending user selection.

## Filesystem synchronization — separate optional consumer

- [Selected sync brief](sync/brief0.gpt6a.md): application direction, copy policy,
  metadata/replacement discipline, and the bounded scope without a runtime refactor.
- [Maintained sync API](../sync/README.md): plain, caller-loop, and borrowed-pool
  examples; root/record validation, staged replacement, partial progress, and
  stable-tree/nonhostile-alias contract.
- [Sync implementation](sync/implementation0.sol61x.md): shared operation kernel,
  joined recursive Rayon work, exact timestamp and permission handling, native
  links, deterministic encountered-error selection, tests, and platform limits.
- [Sync measurements](performance/sync0.sol61x.md): owned nonempty btrfs fixtures,
  fresh/copy/reflink/no-op/sparse/subtree operations, comparison-only controls,
  actual operation counts, and retained raw observations. No live-root mutation
  or jj-waltz speedup claim.
