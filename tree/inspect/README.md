---
type: CrateGuide
title: Immediate tree inspection
description: Ownership of local observations, visitor policy and validated plans during the staged crate extraction.
resource: /tree/inspect/README.md
status: implemented
generated: { by: "model:openai/gpt-6.1-sol#xhigh", at: 2026-10-22 }
verification:
  scope: Extraction boundary and existing single-target behavior; no performance claims
  method: Default, parallel and Linux nightly native nextest suites; clippy, doctests and Rustdoc
sources:
  - resource: "opencode://ses_e9d737832ffeiy0f6OibOWOKSp"
    title: Extracted immediate inspection, preserved private plan construction and added boundary verification
  - resource: /design/architecture/draft2.gpt6a.md#2-separate-immediate-inspection-from-traversal
    title: Accepted extraction scope and encapsulation requirements
  - resource: /README.md#visitor-steered-planning
    title: Governing visitor, fact acquisition and validation contract
---

# Immediate tree inspection

`diff-dirs-tree-inspect` owns immediate native-name unions, early admission,
full-fact late visitors, comparison predicates, action validation and immutable
`DirectoryPlan`s. It performs no recursive traversal or filesystem mutation.
`DirDiff`, immediate counts, checks, scopes and errors have this same owner.

`PairedRoots` validates real directory roots source-first and reuses their mandatory
facts for root steering. `bind_scoped` also returns the checked compatibility
visitor, keeping the root-check bypass private. `DirectoryPlan::paired_children`
provides a read-only, finalized, paired-only child projection; it does not expose
constructors or mutable plan fields. One-sided recursive consumers use the actual
operations under their own traversal rules.

For this checkpoint, listing, early physical hints and platform backends remain
private here. The next accepted stage extracts them to `tree/observe`, including
the physical error boundary. `native-dirent` remains opt-in (Linux needs nightly);
`parallel` enables immediate acquisition on the caller's Rayon pool. There is no
new pooling, tuning or coordinated comparison implementation.

[Comparison](../compare/README.md) owns recursive audit and paired walking. The
root `diff-dirs` package temporarily re-exports the existing public API. Owner unit
tests source-include only the dependency-free fixture in
[`tests/support/fixture.rs`](../../tests/support/fixture.rs); no test dependency
points back to comparison or the facade.
