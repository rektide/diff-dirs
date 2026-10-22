---
type: CrateGuide
title: Paired tree comparison
description: Ownership of deterministic pair traversal, the directory frontier and committed-prefix coverage.
resource: /tree/compare/README.md
status: implemented
generated: { by: "model:openai/gpt-6.1-sol#xhigh", at: 2026-10-22 }
verification:
  scope: Existing paired traversal and facade compatibility, not coordinated comparison
  method: Default, parallel and Linux nightly native nextest suites including frontier controls; clippy, doctests and Rustdoc
sources:
  - resource: "opencode://ses_e9d737832ffeiy0f6OibOWOKSp"
    title: Extracted traversal and recursive audit while preserving frontier semantics and facade type identities
  - resource: /design/architecture/draft2.gpt6a.md#2-separate-immediate-inspection-from-traversal
    title: Accepted traversal boundary and preservation obligations
  - resource: /README.md#the-walk
    title: Governing paired descent, ordering and summary specification
---

# Paired tree comparison

`diff-dirs-tree-compare` owns sequential paired DFS, the optional joined directory
frontier, pruning, recursive read-only audit and committed-prefix summaries.
Immediate inspection and validated plans come from
[`diff-dirs-tree-inspect`](../inspect/README.md); the same types are re-exported,
not copied or wrapped into incompatible public identities.

The private pair adapter lowers finalized paired child names and coverage through
read-only methods. Clean plans release their observations in the inspecting worker
before frontier publication; differing plans retain their immutable full facts.
One-sided directories remain opaque to comparison, and divergent parents still
seal paired descent until delivery and caller acknowledgment. No filesystem effects,
coordinated multi-target frontier or generic progression controller is added.

Features forward down the dependency graph: `parallel`, `native-dirent`, and the
existing `parallel-diagnostics` / `parallel-bench` controls. Sequential execution
remains the default. The root `diff-dirs` package temporarily preserves existing
consumer imports; most integration tests remain there as external API tests.
Frontier unit tests live here and share only dependency-free fixture source with
inspection, avoiding cyclic dev dependencies.
