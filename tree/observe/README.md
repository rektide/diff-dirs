---
type: CrateGuide
title: Physical tree observation
description: Owned listing entries, cached physical hints and explicit no-follow acquisition without comparison policy or traversal.
resource: /tree/observe/README.md
status: implemented
generated: { by: "model:openai/gpt-6.1-sol#xhigh", at: 2026-10-22 }
verification:
  scope: Physical acquisition and inspection error boundary; no performance claims
  method: Stable default owner nextest and Linux nightly all-features workspace nextest, clippy, doctests and Rustdoc
sources:
  - resource: "opencode://ses_e9d3bf383ffeIFJ9ghJT4v2a4w"
    title: Extracted and verified physical observation, retaining private backend resources and owner tests without cross-crate instrumentation
  - resource: /design/architecture/draft2.gpt6a.md#3-extract-physical-observation
    title: Accepted physical ownership, error boundary and feature preservation requirements
  - resource: /README.md#visitor-steered-planning
    title: Governing early facts and deferred no-follow observation contract
---

# Physical tree observation

`diff-dirs-tree-observe` owns directory enumeration, `FileKind`, `FactOrigin`,
`EarlyFacts`, deferred metadata and literal symlink reading. It owns no comparison
policy, visitor, finalized plan, traversal or mutation.

`read_directory` returns owned `Entry` handles in backend order. Read-only name,
path and hint accessors perform no child I/O; `Entry::metadata` consumes a handle
for explicit no-follow acquisition. Dropping an unneeded entry does not inspect
it. Backend descriptors and shared parent ownership remain private. Standalone
`symlink_metadata` and `read_link` support explicit acquisition outside a listing;
the caller decides whether either is needed. `EarlyFacts::from_root_metadata`
projects already-required binding facts without acquiring or validating a root.

Observation `Error` retains the physical path, operation and original I/O cause.
`into_parts` transfers that cause into a consumer boundary. It has no policy-error
dependency. [Inspection](../inspect/README.md) maps it to its existing public error,
adding root-binding context where needed while retaining the I/O cause.

The standard backend is default. `native-dirent` remains opt-in and uses Linux
nightly directory APIs; other platforms retain their standard backend. Unix
standard enumeration keeps kind unknown rather than triggering a hidden stat
fallback. Native hints retain unknown values where the filesystem provides them.
Windows reuses cached enumeration metadata. Linux listed metadata uses the held
parent rather than rebuilding a pathname lookup.

Owner tests cover cached hints, deferred acquisition errors, no-follow metadata,
literal link spelling and held-parent lifetime; native tests retain bounded-buffer
error coverage. They source-include the dependency-free
[fixture](../../tests/support/fixture.rs), without upward test dependencies.
No tuning, pooling or shared-source comparison protocol is implemented here.
