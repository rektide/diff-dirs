---
type: PerformanceReport
title: Own the leaf pipeline — controlled clone and staging experiments
description: Same-compiler controls isolate successful cleanup, held Linux descriptors, and parent-batch staging without changing public comparison or synchronization contracts.
resource: /design/performance/leaf0.sol61x.md
status: experiment-in-progress
generated: { by: "model:openai/gpt-6.1-sol#xhigh", at: 2026-10-18 }
sources:
  - resource: "opencode://ses_eb2c07f5cffe2uTLIC7b1rrD99"
    title: Commissioned leaf-operation optimization, preserved unchanged control, and designed isolated owned-fixture experiments
  - resource: /design/performance/inspection0.sol61x.md
    title: Actual syscall anatomy, current compiler identity, and corrected std copying and dependency behavior
  - resource: /design/performance/opportunities0.gpt6am.md#2-establish-the-right-leaf-operation-boundary
    title: Selected operation ownership experiment and independently attributable cleanup, clone, and staging variants
  - resource: /design/performance/opportunities0.sol61x.md#a01
    title: Independent cleanup, shared staging, held clone descriptor hypotheses and rejection criteria
  - resource: /design/sync/brief0.gpt6a.md
    title: Preparation safety, exact timestamp verification, hardlink safety, joined progress, and unpinned compiler direction
  - resource: /design/sync/implementation0.sol61x.md
    title: Existing implementation and test contracts; historical Rust 1.88 evidence remains historical
  - resource: /design/performance/sync0.sol61x.md
    title: Prior 30k owned fixture shape, not a current-compiler control
  - resource: /perf/leaf.rs
    title: New owned-source and private-target benchmark with independent verification and process CPU clocks
---

# Leaf pipeline experiment

Selected work is internal preparation-through-install ownership, not donor reuse,
rich public records, FD-native traversal, copy-length specialization, or timestamp
verification weakening. Root README is owned by the parent session.

## Controls preserved before implementation

Starting checkout was clean at `7614ea08`. Configured compiler is actual
`rustc 1.101.0-nightly (822182db4 2026-10-13)`, LLVM 23.1.3. The new probe builds
against unchanged production code and saves its executable before edits. Scratch
manifest lock, binaries, raw traces, generated sources, and private targets live
only under `.test-agent/leaf-pipeline/`. Sources in prior fixture/live roots are
not modified or synchronized.

Variants: unchanged; known-empty-success cleanup; that cleanup plus held Linux
clone descriptors with per-leaf staging; then parent-batch staging. Source sizes
and syscall operations are held constant. Every successful sync is checked using
the independent core walk outside timing. CPU uses the process clock including
caller-pool workers. One P4 pool is created outside the timers and reused for
the run's scenarios. Initialization/setup and target damage are outside timing.

Initial baseline verification: 43 default workspace nextest tests passed with
`DIFF_DIRS_SYNC_REQUIRE_REFLINK=1`; nonempty clone tests ran on real btrfs.
Tiny two-file/one-link strict and Copy traces were captured separately from
performance. Measurements and selected design will be appended after experiments.
