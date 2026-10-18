---
type: PerformanceReport
title: Owned filesystem synchronization — copy, reflink, delta, and zero-write repeat
description: Sixty owned-btrfs observations distinguish actual application from read-only diff cost, comparing serial and caller-pool execution without touching live trees.
resource: /design/performance/sync0.sol61x.md
status: measured-awaiting-parent-review
generated: { by: "model:openai/gpt-6.1-sol#xhigh", at: 2026-10-18 }
sources:
  - resource: "opencode://ses_eb3033c79ffeRIkn14uSNhs7O6"
    title: Built the separate applier and measured owned 30k nonempty-file copy/reflink, no-op, sparse, and subtree synchronization with caller-pool controls
  - resource: /sync/examples/sync_perf.rs
    title: Marked owned-fixture harness, rotating configurations, explicit pool limits, and outside-timing verification/setup
  - resource: /perf/results/sync/README.md
    title: Exact commands, environment, binary hash, fixture census, and data-schema note
  - resource: /perf/results/sync/observations.csv
    title: All sixty nanosecond observations and logical operation counts, without sample filtering
  - resource: /sync/README.md
    title: Implemented safety, copy, metadata, operation-count, and parallelism contract
  - resource: /design/performance/parallel1.sol61x.md
    title: Existing read-only frontier evidence, explicitly not an application benchmark
---

# Actual filesystem synchronization on an owned fixture

## Scope and method

This measures the new optional applier, **not jj-waltz or supplied live roots**.
No mutation occurred in `/home/rektide/a/a/opencode`,
`/home/rektide/src/opencode-export-qa`, or jj-waltz. A newly generated btrfs
fixture has 300 package directories, 30,000 regular 4096-byte files, and 300
literal links: 30,600 entries and 122,880,000 logical bytes, excluding the root.
These are nonempty real-clone inputs, not the older metadata-only empty files.

Linux 7.1.0-debplus.1 / Ryzen 7 5800X / btrfs `/dev/nvme0n1p6`, ~50 GB free
at 99% reported usage. Actual Rust 1.88.0 release build, one P4 caller pool
outside timing, explicit comparison J4/K256. No cache dropping, exclusive-host,
RSS/CPU, cold-cache, physical disk-byte, extent-sharing, or network-storage claim.
[Data bundle](/perf/results/sync/README.md) gives commands and executable hash.

Each configuration receives a fresh private owned target. Initial fixture setup,
scenario changes, cleanup, and full clean post-sync verification are outside the
timer. All API/root validation, walk/application, staging, metadata, join, and
report consumption are inside sync timing. Diff-only control includes its normal
root validation and summary. One untimed pass precedes three timed repetitions;
serial/pool × Copy/ReflinkRequired order rotates per repetition. Clean follows
fresh, sparse follows clean, and subtree follows sparse within a configuration,
so this is a warm-cache operational sequence, not independent cold trials.
All 60 observations are retained, including noisy outliers.

## Medians and ranges

Milliseconds; n=3 per cell. Pool is P4/J4/K256. Fresh copies the whole absent
target, including root creation. The old example name was `perf`; it is now
`sync_perf` to avoid a workspace executable collision, with unchanged operations.

| Execution / policy | Fresh median (range) | Clean sync median (range) | Clean diff-only median (range) |
| --- | ---: | ---: | ---: |
| Serial / Copy | 3450.268 (3322.531–3561.726) | 75.532 (75.514–75.964) | 76.689 (75.033–78.058) |
| Serial / Required reflink | 3649.647 (3625.323–4107.703) | 78.246 (74.807–78.298) | 76.794 (76.281–80.991) |
| Pool / Copy | 1668.983 (1623.038–1767.921) | 21.715 (21.281–30.345) | 21.737 (21.209–22.083) |
| Pool / Required reflink | 1708.751 (1615.485–2436.779) | 22.226 (21.901–26.669) | 21.743 (21.153–28.383) |

| Execution / policy | Sparse delta median (range) | Subtree delta median (range) |
| --- | ---: | ---: |
| Serial / Copy | 76.460 (74.656–78.033) | 90.095 (87.246–97.204) |
| Serial / Required reflink | 76.344 (75.591–77.434) | 90.189 (89.740–91.469) |
| Pool / Copy | 21.266 (21.251–22.451) | 30.744 (30.168–34.949) |
| Pool / Required reflink | 21.686 (21.542–25.262) | 31.250 (29.348–67.682) |

Clean sync and clean diff-only have essentially the same cost at this resolution;
the small differences are noise, not negative application overhead. No-op and
sparse remain dominated by the full paired walk, not an undocumented cache.
Pool fresh Copy median is ~2.07× lower than serial, required reflink ~2.14×.
Tiny-file reflinks are **not faster than ordinary copies** in the serial median:
staging/create/metadata syscall cost dominates this 4 KiB fixture. This does not
measure the benefit on large payloads, physical I/O, or end-to-end reseeding.

## Work actually completed

Counts were identical across all corresponding policy/execution repetitions:

| Scenario | Observed diff | Completed target work |
| --- | --- | --- |
| Fresh | No both-sided comparison (`observed=None`) | 30,000 installed regular files, 122,880,000 logical bytes; 300 links; 301 directories and 301 final directory permission calls |
| Clean / diff-only | Zero changed/added/removed entries | **Zero** copied/reflinked/removed/created entries and zero permission calls |
| Sparse | 1 changed, 1 added, 1 removed leaf | 2 installed regular files (8192 logical bytes), 1 removed file |
| Subtree | 1 added and 1 removed opaque directory | 100 installed regular files (409,600 logical bytes), 1 link, 1 directory; 16 removed files and 1 removed directory; 1 final permission call |

Copy rows report only `files_copied`; required rows report only
`files_reflinked`, with **no fallback**. Bytes mean successful logical leaf
payload, not allocation or disk traffic. The subtree operation demonstrates why
diff counts cannot stand in for copied-file counts. Sparse does not recopy
unchanged source files; clean repeat does no application writes or staging.
Every application was independently followed by a clean complete core diff.

## Interpretation and limits

The useful result is filesystem synchronization as an optional consumer, with
steady-state cost equal to comparison and mutation proportional to the reported
delta. Actual caller-pool application overlaps independent file operations,
separately tested with forced distinct-worker rendezvous and joined-on-error
gates; timing alone is not the concurrency proof.

This small campaign deliberately avoids duplicating the earlier 495-run scheduler
study. It offers neither universal pool sizing nor a new application scheduler.
Listings/pending siblings remain width-sensitive memory; active worker count is
not a constant-byte bound. Per-leaf private staging prioritizes replacement
safety and has visible syscall cost. Larger files, cold storage, other filesystems,
Windows/macOS, hostile aliases, crash durability, and consumer integration are
not inferred from these observations.

Implementation and correctness: [sync account](/design/sync/implementation0.sol61x.md).
