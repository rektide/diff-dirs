---
type: PerformanceMethod
title: Reproducible metadata-shape walk harness
description: Separately generated synthetic trees and warmed in-process release walks with clean, sparse, one-sided, and explicitly pruned scenarios.
resource: /perf/README.md
status: implemented
generated: { by: "model:openai/gpt-6.1-sol#xhigh", at: 2026-10-18 }
sources:
  - resource: opencode://ses_eb3948bedffe0T37Xc2wI2S5Jg
    title: Built a std-only reproducible fixture generator and release-walk timing harness
  - resource: /examples/perf.rs
    title: Executable fixture setup, scenarios, validation, and timing boundaries
  - resource: /README.md
    title: Metadata-only comparison contract and unrecovered historical measurement claims
  - resource: /design/fit/fit0.glm53m.md
    title: Initial request for warmed 30k and roughly 192k representative cases
---

# Metadata-shape walk harness

This measures the generic comparison library, **not copy/sync or jj-waltz**.
It does not reproduce the historical 8 GB `node_modules` tree. Payload files
are empty; only metadata shape, native syscall cost, and symlink handling are
representative. All setup and scenario mutation are outside timing intervals.

## Reproduce

```sh
cargo +1.88.0 build --release --example perf
mkdir -p .test-agent/perf
target/release/examples/perf setup .test-agent/perf/btrfs-30k 30000
target/release/examples/perf setup .test-agent/perf/btrfs-192k 192000
target/release/examples/perf run .test-agent/perf/btrfs-30k 5 > .test-agent/perf/btrfs-30k.csv
target/release/examples/perf run .test-agent/perf/btrfs-192k 5 > .test-agent/perf/btrfs-192k.csv
```

Use a fresh, nonexistent fixture root: `setup` rejects existing roots rather
than resetting them. Fixture directories are deliberately left intact; cleanup
is a separate operator decision. `run ROOT N SCENARIO` measures just one
scenario. `once ORIGINAL TARGET` performs one un-warmed walk without setup,
scenario mutation, or restoration checks, useful for syscall auditing.

`repeat ORIGINAL TARGET N` is strictly **read-only**: one untimed warmup followed
by N arbitrary-root comparisons, with no fixture mutation/validation. Use it
for existing live trees, never `run` (which is only for generated fixtures).
`inspect ROOT` is a separate read-only metadata census, reporting files,
directories, links, specials, bytes, and total entries excluding the root. It
enumerates all directories, including ones the paired comparison would treat
as opaque one-sided entries. Neither command ignores admin paths or reads file
contents. Live trees are not snapshots; external mutation may cause failures
or change results between samples.

On Unix each package contains 150 empty regular files across three child
directories and one relative symlink. Including the package directory, that
is 155 entries. A `packages` wrapper contributes one additional entry. Budget
30000 produces **29,916 entries per side**: 28,950 files, 773 directories,
193 links. Budget 192000 produces **191,891 entries per side**: 185,700 files,
4,953 directories, 1,238 links. Counts exclude each side's root directory.
Non-Unix fixtures omit symlinks and report their actual counts at setup.

## Timing and scenarios

Every scenario gets one full untimed warmup, followed by `N` timed observations
in the same process. Timing begins before root validation and ends after normal
iterator exhaustion and `into_summary()`. Output formatting and assertion of
expected results happen after the timer stops. CSV records raw nanoseconds,
records yielded, examined directory pairs, entry counts, and prunes.

- **clean:** identical fixture trees; all paired directories examined, no records.
- **sparse:** target has one changed-size leaf, original has one added leaf,
  target has one stale leaf. Same traversal coverage, three entry differences.
- **one-sided:** move one original package and a different target package
  outside the compared roots; one added and one removed directory. Their
  descendants are not visited. Renames happen outside timing.
- **prune:** add a root-level original leaf to force a root record, then
  `prune_here()` on that record. Only immediate root listings are examined.
  This is explicitly incomplete coverage, not a faster equivalent full scan.

Every scenario is restored outside timing and followed by an untimed clean
verification. No caches are dropped, no CPU pinning or exclusive-machine
guarantees are assumed. Record variance, filesystem, compiler, root paths, and
entry counts with results. Warm-cache timings are not cold-start latency.

## Syscall audit

Example on an already generated clean fixture:

```sh
strace -c -e trace=%file,getdents64,read,pread64 target/release/examples/perf once .test-agent/perf/btrfs-192k/original .test-agent/perf/btrfs-192k/target
strace -yy -e trace=openat,read,pread64,mmap,readlink,readlinkat -o .test-agent/perf/io.trace target/release/examples/perf once .test-agent/perf/btrfs-30k/original .test-agent/perf/btrfs-30k/target
```

Inspect fd-annotated paths: fixture payloads should never be opened/read;
directory opens and `readlink` are expected. Dynamic loader and process startup
reads are normal. Traced timing is not performance data. The benchmark binary
itself reads no fixture manifest or payload content.

```sh
python3 perf/report.py perf/results/btrfs-30k.csv perf/results/btrfs-192k.csv
python3 perf/audit.py .test-agent/perf/io.trace .test-agent/perf/btrfs-30k
```

The stdlib-only report script computes medians, ranges, means, and population
standard deviations. The audit script checks all matching fixture opens are
directory-only and rejects fixture data reads/mmaps; pass the same trace flags
as above and a substring matching only the fixture paths, not the working dir.

See the [measured run](/design/performance/performance0.sol61x.md) for results
and environment; the [implementation account](/design/implementation/implementation0.sol61x.md)
explains the traversal and coverage states that the harness exercises.
