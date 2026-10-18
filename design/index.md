---
type: Index
title: diff-dirs design and implementation evidence
description: Reading paths through consumer fit, I/O research, implementation decisions, and measured performance.
resource: /design/index.md
status: maintained
generated: { by: "model:openai/gpt-6.1-sol#xhigh", at: 2026-10-18 }
sources:
  - resource: "opencode://ses_eb2c07f5cffe2uTLIC7b1rrD99"
    title: Connected implemented leaf ownership and batch staging to same-compiler controls, source-preservation and failure tests, syscall attribution and explicit negative performance cases
  - resource: "opencode://ses_eb3033c79ffeRIkn14uSNhs7O6"
    title: Added filesystem synchronization and owned-fixture evidence, verified current/historical std syscalls, and wrote the research-only performance opportunity inventory and independent Sol high experiment briefs
  - resource: "opencode://ses_eb335933bffeUeaPkxqOtKDOWi"
    title: Linked the paused runtime refactor assessment and smaller two-crate option without changing the implemented contract
  - resource: "opencode://ses_eb3948bedffe0T37Xc2wI2S5Jg"
    title: Linked accepted frontier implementation, bounds/liveness verification, and measured single-root scale-out while preserving independent designs and baseline reports
  - resource: "opencode://ses_eb37f1883ffeZ8HgL0aFBFJf4R"
    title: Added the post-handoff bounded-directory-frontier design proposal to the reading path
  - resource: "opencode://ses_eb39c93e9ffeNju5a7XEsB3wGY"
    title: Coordinated research and implementation, reviewed concurrency limits, and connected the independent optimization inventories and architecture-sensitive ranking
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
- [Leaf operation ownership](sync/leaf0.sol61x.md): held Linux clone descriptors,
  parent-local lazy staging arenas, planned-name collision exclusion, recursive
  metadata handoff, exact metadata, joined cleanup and partial progress. Public
  APIs and the comparison/application seam are unchanged.
- [Controlled leaf measurements](performance/leaf0.sol61x.md): same-compiler
  unchanged/cleanup/descriptor/staging/handoff controls, separate staging-only
  and final-build confirmations, syscall attribution, raw observations and
  single-leaf/large-file/P4 descriptor negatives. Qualifies which mechanisms
  actually helped rather than promoting the entire opportunity inventory.

## Performance opportunities — research, not implementation selection

- [Inventory comparison](performance/comparison0.gpt6a.md): parent synthesis of
  shared findings and different prioritization judgments; maps the proposed
  donor, leaf-pipeline, and prepared-mirror experiments across both inventories.
- [Independent Astra inventory](performance/opportunities0.gpt6am.md): forty
  candidates, seven experiment briefs, and an appended ranking emphasizing
  architectural information value. Recommends testing donor reuse, descriptor
  ownership, and retained comparison information before selecting new APIs.
- [Opportunity inventory](performance/opportunities0.sol61x.md): identifiable
  path/name, metadata, descriptor, scheduler, staging/copy/removal, multi-target,
  reporting, and contract candidates; each distinguishes evidence from payoff
  hypotheses, names negative outcomes, and supplies discriminating experiments.
  Includes independent Sol high briefs and generic versus tuned-profile choices.
  Reconsiders the existing contract deliberately without implementing or selecting
  a rewrite; current toolchain direction is unpinned configured nightly.
- [Source/syscall ground truth](performance/inspection0.sol61x.md): verifies
  that both historical Rust 1.88 and the configured 1.101 nightly already use
  fd-relative DirEntry metadata and kernel-assisted ordinary copying; identifies
  remaining full-path/duplicate/staging work. Records current experimental std
  directory handles, the user's unpinned-toolchain override, raw-data checks,
  and owned-only probe scope. This qualifies future hypotheses; it does not
  change defaults or promote a new backend.
