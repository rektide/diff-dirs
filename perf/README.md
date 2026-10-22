---
type: PerformanceMethod
title: Reproducible directory-walk and frontier harness
description: Separately generated trees, strictly read-only live comparisons, same-build interleaved controls, and separate physical scheduler diagnostics.
resource: /perf/README.md
status: implemented
generated: { by: "model:openai/gpt-6.1-sol#xhigh", at: 2026-10-18 }
updated: { by: "model:openai/gpt-6.1-sol#xhigh", at: 2026-10-22 }
sources:
  - resource: opencode://ses_e9d314c29ffeboK450qrHM3qhF
    title: Relocated the perf example into the facade package and updated active build selectors and synchronization coordinates, preserving historical results
  - resource: opencode://ses_eb2165cc8ffe4vOmOvDBe7aXQu
    title: Replaced active Python performance entrypoints with checked-in Rust controls, reports, audits and resource observation; retained historical evidence
  - resource: opencode://ses_eb24f98a3ffewdVqa9lao6tXQu
    title: Added checked-streaming benchmark reproduction and baseline/no-op/name-check controls, preserving negative initial path extraction and the lazy-path correction
  - resource: opencode://ses_eb39c93e9ffeNju5a7XEsB3wGY
    title: Switched active benchmark recipes to the configured unpinned Rust toolchain while retaining historical measurement provenance
  - resource: opencode://ses_eb3033c79ffeRIkn14uSNhs7O6
    title: Added separate marked-owned-fixture synchronization benchmark method and retained copy/reflink/no-op/delta measurements without changing read-only live comparison commands
  - resource: opencode://ses_eb3948bedffe0T37Xc2wI2S5Jg
    title: Built reproducible fixtures/live tests, then same-build serial/per-frame/frontier comparisons, interleaved repeats, physical diagnostics, and process resource sampling
  - resource: /tree/api/examples/perf.rs
    title: Executable fixture setup, scenarios, validation, and timing boundaries
  - resource: /README.md
    title: Metadata-only comparison contract and unrecovered historical measurement claims
  - resource: /design/fit/fit0.glm53m.md
    title: Initial request for warmed 30k and roughly 192k representative cases
---

# Directory-walk harness

Current orchestration is **Rust-only**: [native control guide](/perf/runner/README.md)
and [migration/evidence account](/design/performance/tooling-completion0.sol61m.md).
The comparison example below remains the domain instrument. Former Python
drivers are retired, not an alternative supported path. Historical sources are
recoverable with `git show 997183df472f81d21e8d553a9002d083d636ef05:perf/PATH`.

The source is now [`tree/api/examples/perf.rs`](/tree/api/examples/perf.rs), owned
by `diff-dirs-tree-api`; target name and output path remain `perf` and
`target/release/examples/perf`. Current operation controls consume the relocated
facade and `transfer/sync` packages. Revision-bound results and archived experiment
sources retain their original names and paths. The runner's generated archived
`build-operation` compatibility is a separate follow-up, not verified by this
relocation; ordinary workspace packages can be built directly.

This measures the generic comparison library, **not copy/sync or jj-waltz**.
Owned metadata-shape fixtures do not reproduce the historical 8 GB workload.
Their payloads are empty except the tiny deliberately dense size differences.
Actual supplied live trees are a separate, explicitly labeled read-only scope.
All setup and scenario mutation are outside timing intervals.

## Reproduce

Use the configured toolchain, without a project version pin. Record
`rustc --version --verbose` with each new run (currently Rust 1.101 nightly).
Build both candidate and control with that same compiler; historical Rust 1.88
results describe their original builds, not the current compiler's performance.

```sh
cargo build --release -p diff-dirs-tree-api --example perf
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
records yielded, committed directory pairs, entry counts, prunes, and (in new
observations) first-record latency. Blank first-record latency means no record,
not zero latency.

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
cargo run --release -p diff-dirs-perf-runner -- summarize-csv perf/results/btrfs-30k.csv perf/results/btrfs-192k.csv
cargo run --release -p diff-dirs-perf-runner -- audit .test-agent/perf/io.trace .test-agent/perf/btrfs-30k
```

The Rust CSV report computes per-file medians, ranges, means, and population
standard deviations without pooling unrelated scopes. The Rust audit checks matching fixture opens are
directory-only and rejects fixture data reads/mmaps; pass the same trace flags
as above and a substring matching only the fixture paths, not the working dir.

See the [measured run](/design/performance/performance0.sol61x.md) for results
and environment; the [implementation account](/design/implementation/implementation0.sol61x.md)
explains the traversal and coverage states that the harness exercises.

## Sequential versus caller-owned Rayon pools

Build one executable with the optional backend, then select execution without
rebuilding. Pool creation is outside all timings and one pool is shared across
every walk/warmup in that process. For example:

```sh
cargo build --release -p diff-dirs-tree-api --example perf --features parallel-bench
target/release/examples/perf run .test-agent/perf/btrfs-192k 3
target/release/examples/perf parallel 4 run .test-agent/perf/btrfs-192k 3
target/release/examples/perf parallel 4 repeat /home/rektide/a/a/opencode/node_modules /home/rektide/src/opencode-export-qa/node_modules 3
target/release/examples/perf compare /home/rektide/a/a/opencode/node_modules /home/rektide/src/opencode-export-qa/node_modules 4 5 4 256 live-node
```

`parallel THREADS` is a harness-only prefix using a caller-created Rayon pool;
it is not a core-owned pool. The native `compare` command is read-only and
rotates same-build serial/per-frame/frontier controls. For 1/2/4/8 comparisons,
invoke explicit pool configurations sequentially and retain each CSV/stderr;
configuration order itself is not drift-balanced. `run` remains only for
generated owned fixtures and performs its own scenario restoration.
Report with `cargo run --release -p diff-dirs-perf-runner -- summarize-csv perf/results/parallel.csv`.
The [parallel report](/design/performance/parallel0.sol61x.md) preserves actual
results, counts, scheduling limits, and the selected default policy.

**Revision note:** that retained 150-observation report used the old per-frame
backend at its recorded revision/hash. `Execution::Parallel` and the `parallel`
harness prefix now use the accepted directory frontier. The explicit controls
below reproduce the predecessor alongside the new scheduler in one build.

## Joined frontier versus retained controls

Build timing and instrumented executables separately. Do not use an all-features
instrumented binary for the timing table:

```sh
cargo build --release -p diff-dirs-tree-api --example perf --features parallel-bench --target-dir target/frontier-timing
cargo build --release -p diff-dirs-tree-api --example perf --features parallel-diagnostics --target-dir target/frontier-instrumented
target/frontier-timing/release/examples/perf compare /home/rektide/a/a/opencode/node_modules /home/rektide/src/opencode-export-qa/node_modules 4 5 4 256 live-node
target/frontier-instrumented/release/examples/perf diagnose /home/rektide/a/a/opencode/node_modules /home/rektide/src/opencode-export-qa/node_modules 4 4 256
```

`compare ORIGINAL TARGET P N J K [LABEL]` is always **read-only**. It builds one
caller pool before timing, checks complete record vectors and summaries across
serial/per-frame/frontier outside timing, then warms each once. It rotates
backend order each iteration rather than timing one backend's entire group
first. Timing includes root validation, all `next()`/pool-install operations,
and summary consumption; printing/parity checks are excluded. The public
borrowed-pool API drives frontier, while per-frame uses the retained hidden
measurement control. No output is gathered during a timed walk.

Run explicit configurations with the native instrument, retaining raw CSV/stderr.
Each subprocess rotates backend order; this shell recipe does not make claims
about balanced order across configurations:

```sh
target/frontier-timing/release/examples/perf compare /home/rektide/a/a/opencode/node_modules /home/rektide/src/opencode-export-qa/node_modules 4 5 4 256 live-node
```

`diagnose ORIGINAL TARGET P J K` performs a separate instrumented frontier walk,
reporting admitted/completed/committed work, queued+running/active high-water
jobs, speculative and total private node high-water counts, worker IDs,
demand-reserve use, budget-stall events, cumulative active-job duration, and
epoch drain duration. Active jobs span inspection through completion publication;
they are not a kernel-syscall overlap profiler. Dividing their cumulative duration
by instrumented wall time gives mean active-job occupancy, not CPU utilization.
Budget-stall counts require an available J slot and eligible child blocked by
full K; they are admission observations, not time or distinct stalled intervals.
Drain measures ordered stop selection through scope join, not just filesystem
tail time (and excludes the final outer `install` return to its caller).
Retained-name bytes count paths/child basenames in scheduler state, **not** raw
listing allocations, diff records, allocator overhead, or whole process RSS.
Instrumentation itself adds work and is never substituted for timing samples.

## Owned topology fixtures and process resources

```sh
target/frontier-timing/release/examples/perf shape .test-agent/perf/frontier-small small 16000
target/frontier-timing/release/examples/perf shape .test-agent/perf/frontier-skewed skewed 8000
target/frontier-timing/release/examples/perf shape .test-agent/perf/frontier-dense dense 4000
target/frontier-timing/release/examples/perf shape .test-agent/perf/frontier-wide wide 30000
```

Only `shape ROOT KIND COUNT` creates these fixtures and requires a nonexistent
owned root. `small` groups 64 leaf directories per parent, each with two empty
files (below the previous metadata cutoff). `skewed` puts that tree under the
first `a` branch with three tiny sibling directories. `dense` makes one source
file per leaf one byte larger, yielding a record at every leaf. `wide` has only
COUNT immediate regular files and no child directories: directory-only
parallelism has no fanout to exploit. `compare` thereafter changes none of them.
The command prints an independent census for both sides after setup.

For a separate process-resource observation, use one backend without the
outside-timing whole-record parity collector:

```sh
cargo run --release -p diff-dirs-perf-runner -- resources target/frontier-timing/release/examples/perf .test-agent/resources-NEW -- backend-once frontier /home/rektide/a/a/opencode/node_modules /home/rektide/src/opencode-export-qa/node_modules 4 4 256
```

`backend-once serial|per-frame|frontier ORIGINAL TARGET P J K` builds its pool
outside the internal timer and compares read-only once. The Rust Linux helper
uses `wait4` for child CPU accounting and peak RSS including startup, excluding
the controller. These process scopes are not the internal API timer or an
allocator bound. Historical `/proc` sampling has different semantics and is not
silently combined with the new observations. Resource observations are
separate from the interleaved timing matrix.

## Filesystem synchronization — separate owned-fixture harness

The `transfer/sync/` consumer has its own `sync_perf` example; it is **mutating**, unlike
the arbitrary-pair read-only commands above. It accepts only a freshly generated
marked fixture for benchmark runs and never offers an arbitrary-root sync mode.

```sh
cargo build --release -p diff-dirs-transfer-sync --example sync_perf --features parallel --target-dir target/sync-perf
mkdir -p .test-agent/sync
target/sync-perf/release/examples/sync_perf setup .test-agent/sync/btrfs-30k-new 300 100
target/sync-perf/release/examples/sync_perf run .test-agent/sync/btrfs-30k-new 3
```

Setup refuses an existing root. Its 30,000 × 4 KiB files are **nonempty**;
setup/scenario edits/private-target cleanup and complete post-sync verification
are outside timing. Each configuration initializes a fresh owned target, then
runs clean sync, diff-only control, sparse delta, and subtree delta. Four
serial/P4 × Copy/Required-reflink configurations rotate across repetitions after
one warmup each. The pool is caller-owned with explicit J4/K256. All reports
separate observed immediate diff counts from completed recursive operations and
logical copied/cloned bytes. No live tree or jj-waltz mutation is involved.

See [measured sync results](/design/performance/sync0.sol61x.md) and the
[60-observation bundle](/perf/results/sync/README.md) for raw data, exact commands,
environment, operation counts, and limits. These do not replace the preserved
read-only frontier measurements or establish end-to-end consumer speedup.

## Checked streaming controls

The [checked execution report](/design/performance/checked0.sol61x.md) measures
the unchanged predecessor against default `NoCheck`, an enabled no-op closure,
and a concrete native-name guard. It uses identical release benchmark source
and the same configured compiler for both library snapshots. Owned 10k × 4 KiB
fixtures exercise first fill, clean, sparse, recursive subtree and full leaf
replacement, with serial and explicit borrowed P4/J4/K256 execution. Setup,
scenario edits, complete post-sync comparison and owned cleanup are outside
operation timing. Clean reports must have zero completed operations.

The [evidence bundle](/perf/results/checked/README.md) preserves exact experiment
source/manifests, commands, compiler, binary hashes and all warmup/raw rows. The
first checkpoint's enabled-check path-construction penalty is preserved rather
than erased; a focused shared-target clean control measures its lazy-path
correction. These experiments neither replace the primary harness nor authorize
arbitrary-root mutation. Reproduce them in a new owned scratch directory and
never overlap measurements with builds/tests or another mutating experiment.
