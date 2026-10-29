---
type: CrateGuide
title: Physical tree observation
description: Owned listing entries, cached physical hints and explicit no-follow acquisition without comparison policy or traversal.
resource: /tree/observe/README.md
status: implemented
generated: { by: "model:openai/gpt-6.1-sol#xhigh", at: 2026-10-22 }
updated: { by: "model:openai/gpt-6-astra#xhigh", at: 2026-10-29 }
verification:
  scope: Physical acquisition and inspection error boundary; no performance claims
  method: Stable default owner nextest and Linux nightly all-features workspace nextest, clippy, doctests and Rustdoc
sources:
  - resource: "opencode://ses_e7a2771c3ffez3sv1oPgD5gPH4"
    title: Reviews the implemented observation resource/name foundation and promotes its verified ownership boundaries into the guide
  - resource: /design/architecture/execution0.sol61x.md
    title: Implemented compact names and caller-owned buffers, with owner-test receipts and remaining recursive integration
  - resource: /tree/observe/src/resources.rs
    title: Actual runtime configuration, buffer leases and bounded idle retention
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

## Caller-owned enumeration resources

`read_directory_with(path, &resources)` uses a shared `ObservationResources` handle.
The native backend starts with a 64 KiB read buffer, lazily retains at most 16 idle
buffers and caps retained raw capacity at 1 MiB. Active readers allocate when no
idle buffer is available; idle retention is not a concurrency semaphore. Leases
return on EOF or error under current retention settings. `reconfigure` changes
future borrows coherently and drops excess idle storage. `require_native` refuses
when that backend is unavailable. `retained_capacity()` reports idle buffer
capacity, not total process memory or active listing size.

`read_directory` creates its own default resource handle. A caller seeking reuse
across calls supplies the shared handle explicitly. The contextual inspection
source session carries one; complete outer traversal/recursive-effect resource
threading remains part of the [active delivery](/design/architecture/delivery0.gpt6ax.md).

Native listing entries borrow names by index from a compact native-name table and
share their parent descriptor. `Entry::name()` avoids a name clone; `file_name()`
returns an owned copy. Packed Unix names outlive raw-buffer leases. Full listings
and selected child names have separate lifetimes. Shared-source policy and fact
demand remain owned by inspection. These ownership changes have focused test
evidence, without a measured throughput claim.
