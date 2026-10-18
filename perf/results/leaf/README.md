---
type: EvidenceBundle
title: Same-compiler leaf pipeline controls
description: Unfiltered owned-fixture timing and process CPU observations for cleanup, held clone descriptors, batch staging and recursive metadata handoff, with separate final-build resource confirmation and syscall counts.
resource: /perf/results/leaf/README.md
status: measured-awaiting-parent-review
generated: { by: "model:openai/gpt-6.1-sol#xhigh", at: 2026-10-18 }
sources:
  - resource: "opencode://ses_eb2c07f5cffe2uTLIC7b1rrD99"
    title: Preserved same-compiler controls, generated exclusive disposable fixtures, interleaved all variants, and retained raw timings without sample filtering
  - resource: /perf/leaf.rs
    title: Timer, process CPU clock, owned fixture guard, reused P4 pool, private targets, and outside-timing verification
  - resource: /perf/leaf.py
    title: Rotating same-compiler variants and bounded final-build resource confirmation
  - resource: /perf/leaf_trace.py
    title: Separate syscall census and explicit whole-process versus arena-path count scopes
  - resource: /design/performance/leaf0.sol61x.md
    title: Interpretation, negative cases, scope and selection limits
---

# Owned leaf-pipeline evidence

All mutation stays under newly owned `.test-agent/leaf-pipeline/`; every timed
run creates its own private target. Prior fixture and live roots were not used
as writable inputs. Linux 7.1.0-debplus.1, Ryzen 7 5800X, btrfs, actual configured
Rust 1.101.0-nightly `822182db4`, LLVM 23.1.3, ordinary release profile. No compiler
pin, cache dropping, affinity, exclusive-host, cold-cache or physical-I/O claim.
Short workspace checks overlapped the early main campaign; no observations were
discarded. All stages use the same scratch Cargo.lock and dependency versions.

`observations.csv`: **2350 rows**, five retained repetitions per cell. Variants:

| Name | Production source |
| --- | --- |
| unchanged | `7614ea08`; harness checkpoint `274e0ad3` |
| cleanup | `58beac82`: known-empty success cleanup only |
| descriptor | `60a4a03e`: cleanup plus Linux held descriptors; per-leaf staging |
| shared | `cc0fe85f`: descriptor plus lazy parent-batch staging; no metadata handoff |
| handoff | `cc0fe85f` plus the two recursive dispatch calls to `copy_directory_with` / `copy_leaf_with` with the already loaded Metadata |

The final build (`939a6f3d`) additionally excludes case-insensitive ASCII prefix
collisions and removes needless borrows, with further failure tests. It is saved
separately rather than overwriting a measured executable. `confirmation.csv`
and `resources.csv` record the bounded final-build comparison separately.
`staging-only` is the final source with the direct Linux clone branch disabled:
change the first `prepare_file` match guard to `policy == CopyPolicy::Copy`.
The production guard was restored immediately after saving that binary.
`descriptor-isolation.csv` retains nine alternating repetitions per serial/P4
cell on dense1k Required, comparing that control with the final build. It was
added because the initial standalone descriptor P4 wall signal was weak, not to
replace or filter the earlier data. No P4 fresh descriptor win is claimed.

Generated fixtures: single/8/1000 4-KiB siblings in one directory (plus a literal
link); two directories × four 4-MiB files (plus two links); 300 directories × 100
4-KiB files (plus 300 links). Sources are generated once, not reinitialized in
timers. Each run uses one P4 pool, created outside timing and reused. J4/K256.
Serial controls still create but do not execute work in that pool.

Fresh includes public root binding and absent-root initialization. Dense damages
every regular target leaf outside timing, then replaces only those leaves. Clean
must report zero operations. Sparse damages one leaf outside timing. Diff-only
walks the resulting clean pair. Each timer includes normal public validation,
application, metadata, joined cleanup and report return; full independent core
verification, fixture enumeration, damage, cleanup and pool construction are
outside it. CPU nanoseconds use CLOCK_PROCESS_CPUTIME_ID, including worker CPU.

Main driver warms every variant/configuration once on dense1k, then rotates
variant and configuration order across five rounds. All five variants run on
small fixtures, Copy/strict serial/P4; dense1k also includes successful fallback.
30k confirmation in the main campaign is deliberately limited to endpoints,
P4, Copy/strict. `confirmation.csv` is a second three-round endpoint control on
the final source. Resource rows are `wait4` rusage **whole-process** user/system
CPU, peak RSS and wall, including setup/damage/verification/cleanup; do not treat
those resource CPU/wall totals as the API timer.

Build the probe using an isolated scratch manifest with dependencies on root
`diff-dirs` (`parallel`), `sync` (`parallel`), rayon 1.12.0, tempfile 3, and libc
0.2, and `[[bin]] path = "../../perf/leaf.rs"`. Set `[workspace]` in the scratch
manifest. Its exact lock is retained as `probe.lock`.

Commands actually used:

```sh
cargo build --release --manifest-path .test-agent/leaf-pipeline/Cargo.toml --target-dir .test-agent/leaf-pipeline/build
# Before each next production edit: copy the built binary to its variant name.
.test-agent/leaf-pipeline/unchanged setup single 1 1 4096
.test-agent/leaf-pipeline/unchanged setup shallow8 1 8 4096
.test-agent/leaf-pipeline/unchanged setup dense1k 1 1000 4096
.test-agent/leaf-pipeline/unchanged setup large 2 4 4194304
.test-agent/leaf-pipeline/unchanged setup tree30k 300 100 4096
python3 perf/leaf.py 5
python3 perf/leaf.py confirm
python3 perf/leaf.py isolate
python3 perf/leaf_trace.py
```

The harness refuses existing setup roots and the driver refuses existing output
CSVs. Preserve old evidence and choose a new owned scratch root for reproduction.
Executable hashes, full census counts and normalized first-leaf syscall excerpts
are retained alongside data. `arena-path-mentioned` counts are operand filters,
not a complete lifecycle: closes after rename show the installed path instead.
An initial resource attempt could not start because `/usr/bin/time` is absent;
its header-only outputs are preserved in scratch, not treated as measurements.
The completed resource sample instead uses Python `os.wait4` rusage directly.
