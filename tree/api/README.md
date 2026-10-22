---
type: CrateGuide
title: Public tree comparison facade
description: Convenient exports of immediate inspection and deterministic paired traversal, without filesystem effects.
resource: /tree/api/README.md
status: implemented
generated: { by: "model:openai/gpt-6.1-sol#xhigh", at: 2026-10-22 }
sources:
  - resource: "opencode://ses_e9d314c29ffeboK450qrHM3qhF"
    title: Relocated the public facade, API integration tests and perf example into the path-derived workspace
  - resource: /design/architecture/draft2.gpt6a.md
    title: Accepted crate ownership, dependency direction and relocation scope
  - resource: /README.md
    title: Maintained comparison specification and embedding contract
  - resource: /tree/api/src/lib.rs
    title: Same-identity exports from inspection and comparison
---

# Public tree comparison facade

`diff-dirs-tree-api` is the convenient embedding entry point. Rust imports use
`diff_dirs_tree_api`; the old root package `diff-dirs` no longer exists. The
[`diff-dirs` executable](/app/cli/Cargo.toml) keeps its command name.

This crate re-exports the same inspection types and comparison functions, not a
second implementation. [Inspection](../inspect/README.md) owns immediate facts,
visitor policy and validated plans; [comparison](../compare/README.md) owns paired
traversal, pruning, audits and summaries. Neither depends on the facade or on
[filesystem synchronization](/transfer/sync/README.md).

Default builds are std-only. `parallel` forwards to inspection/comparison and
adds a direct optional Rayon dependency used by the `perf` example. `native-dirent`
forwards the opt-in Linux/nightly backend; it is never a default. The existing
`parallel-bench` and `parallel-diagnostics` features forward their separate control
and instrumentation paths. `walk()` remains sequential regardless of features.

API integration tests live in `tests/`; their dependency-free fixture is shared
from `/tests/support/fixture.rs`. The comparison harness retains target name
`perf` in `examples/perf.rs`:

```sh
cargo nextest run -p diff-dirs-tree-api --all-targets
cargo test -p diff-dirs-tree-api --doc
cargo check -p diff-dirs-tree-api --example perf --features parallel-bench,parallel-diagnostics
```

The [root specification](/README.md) governs behavior. The
[performance guide](/perf/README.md) owns harness commands and timing boundaries;
relocation is not a new performance result or coordinated-comparison capability.
