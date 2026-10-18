---
type: EvidenceBundle
title: Owned btrfs synchronization observations
description: Sixty unfiltered timed observations of copy/reflink and serial/caller-pool synchronization, with explicit logical operation counts and comparison-only controls.
resource: /perf/results/sync/README.md
status: measured
generated: { by: "model:openai/gpt-6.1-sol#xhigh", at: 2026-10-18 }
sources:
  - resource: "opencode://ses_eb3033c79ffeRIkn14uSNhs7O6"
    title: Generated owned nonempty btrfs fixture and measured initialization, clean comparison/sync, sparse delta, and subtree operations
  - resource: /sync/examples/sync_perf.rs
    title: Marked-fixture benchmark and outside-timing setup, caller pool, warmup, and verification
  - resource: /perf/results/sync/observations.csv
    title: Exact nanoseconds and operation counts; column headers disambiguated after the initial run
---

# Owned synchronization observations

2026-10-18, Linux 7.1.0-debplus.1 x86_64, Ryzen 7 5800X host, btrfs
`/dev/nvme0n1p6` (3.4 TB, 99% used, ~50 GB available). Actual Rust 1.88.0,
ordinary release profile, same executable/configuration for all observations.
No cache dropping, affinity, exclusive-machine, cold-cache, disk-I/O, or RSS claims.

Initial executable SHA-256:
`6c111f4f53c4955dcfe6635f1ce44afbc1eb8ec7f5e55474a754cc5fd7fc1d5c`.
The harness was initially named `perf`; renamed `sync_perf` after workspace
checks revealed a collision with the core example. Source/operation kernel is
unchanged by that rename. CSV numeric rows are untouched; the initial header's
duplicate observed/completed directory labels were disambiguated to the current
`observed_*` column names. No samples were removed or replaced.
The final implementation also corrects a failure-only permission counter when
source directory listing fails after temporary relaxation; successful benchmark
paths and their measured operation counts are unaffected. That failure is
separately exercised under an isolated restrictive umask.

Commands actually run (old example name):

```sh
cargo +1.88.0 build --release -p diff-dirs-sync --example perf --features parallel --target-dir target/sync-perf
target/sync-perf/release/examples/perf setup .test-agent/sync/btrfs-30k 300 100
target/sync-perf/release/examples/perf run .test-agent/sync/btrfs-30k 3 > .test-agent/sync/results.csv
```

Current reproduction (use a fresh nonexistent setup root):

```sh
cargo +1.88.0 build --release -p diff-dirs-sync --example sync_perf --features parallel --target-dir target/sync-perf
target/sync-perf/release/examples/sync_perf setup .test-agent/sync/btrfs-30k-new 300 100
target/sync-perf/release/examples/sync_perf run .test-agent/sync/btrfs-30k-new 3
```

The source has 300 package directories × 100 nonempty 4096-byte regular files,
plus 300 Unix links: **30,600 entries, 122,880,000 logical file bytes** excluding
source root. No live checkout content is used. Every configuration gets a fresh
private target inside a `TempDir`; setup/scenario edits/cleanup and a full clean
post-sync diff occur outside each timer. One untimed pass per configuration
precedes three timed passes with rotating serial/pool × copy/reflink order.
The application creates one P4 pool outside timing and uses explicit J4/K256.
There were brief concurrent workspace check builds during the initial warmup;
host activity was not otherwise controlled.

Four sync scenarios and one comparison-only control × four configurations ×
three repetitions = **60 observations** (48 sync, 12 read-only diff controls).
Fresh includes absent-root initialization; `observed_* = 0` there is not a
claim that a comparison saw zero differences: there was no both-sided diff.
Copy/reflink byte counts are logical successful leaf bytes, not physical I/O.

See [analysis](/design/performance/sync0.sol61x.md) for medians, limits, and counts.
