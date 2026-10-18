---
type: PerformanceAssessment
title: Independent Rust-runner reassessment of real-shape clean synchronization
description: Matched frozen baseline/default/no-op/native-name controls on a private copy of the actual opencode payload shape; clean only, preliminary and not end-to-end JW evidence.
resource: /design/performance/reassessment0.sol61x.md
status: assessed-clean-only-preliminary
generated: { by: "model:openai/gpt-6.1-sol#xhigh", at: 2026-10-18 }
sources:
  - resource: "opencode://ses_eb24f98a3ffewdVqa9lao6tXQu"
    title: Independently reviewed the frozen Rust clean protocol, completed all 64 predeclared real-shape samples, assessed paired ratios and verification costs, and bounded JW selection guidance without changing tooling or production
  - resource: /perf/runner/README.md
    title: Read the clean-only native runner contract and proof at checkpoint 453ec94d
  - resource: /perf/runner/src/main.rs
    title: Reviewed source matching frozen controller hash afc26acf540276590c20a271cbf3e0f991d529d06b6e849ca603961bb062e811
  - resource: /.test-agent/checked/frozen-fa91/identity.json
    title: Verified exact baseline and candidate child hashes and their original build identities
  - resource: /.test-agent/checked/frozen-fa91/source-bench.rs
    title: Checked child-owned API timer, same-mode untimed warmup, zero-operation assertion and serial/P4/J4/K256 dispatch
  - resource: /.test-agent/clean-reassessment0.sol61x/README.md
    title: New private case/output paths, exact campaign commands and predeclared mirrored execution-mode order
  - resource: /design/sync/checked0.sol61x.md
    title: Implementation readiness, stat-key/safety limits and deferred independent reassessment
  - resource: /design/performance/evidence/reassessment0.sol61x/README.md
    title: All raw samples, native paired summaries, byte/inode identities, unchanged executable hashes and host context for this assessment
  - resource: /design/performance/tooling-takeover0.sol61m.md
    title: Read the takeover's instrumentation, interleaving, fingerprint, load and scope distinctions during the assessment doc-pass
---

# Independent real-shape clean reassessment

**Selection: keep the guard; prefer caller-owned P4/J4/K256 for this single large
payload when four workers are available.** All 64 samples passed. There is no
attributable checker regression warranting a production change from these data.
This is **preliminary, warm clean only, not end-to-end JW**. Production and runner sources are
untouched. Only the checked-in Rust clean protocol is used; no new Python,
scenario workaround, controller or benchmark script is introduced. Mutable
scenarios/N-destination/JW migration remain with the tooling/application owners.

## Trust boundary and method

Controller: preserved `.test-agent/rust-clean-proof-runner`, SHA-256
`0e4e8a09b8cde94f0dc3464331f628379f66e099870452c5fe343cb30bb7bb32`, matching
the reviewed `453ec94d` source and its embedded proof source hash. Frozen child
controls are baseline `fd277144` (`ab4a8dfe…afc9`) and candidate runtime `243be8e2`
(`a6906bea…e362`); candidate `none`, enabled no-op and native `.git`/`.jj` check
are selected in the **same candidate executable**. Original builds use configured
Rust 1.101.0-nightly `822182db4`, LLVM 23.1.3, default release, library `parallel`
only, no diagnostics; harness feature `checks` only enables candidate API branches.
Full identities/provenance are retained by each campaign.

Independent review found the clean protocol fit for this bounded question:
hash-checked preserved artifacts; seeded permutation/reverse-permutation blocks;
strict one-row clean/label/mode/thread parser; a child assertion of all-zero
completed progress; metadata/inode verification before/after every sample;
full byte verification at campaign boundaries; retained failed attempts with no
usable API value or invalid pair. No correctness/control defect requiring a
source change was identified. References: runner `tree`/`setup`, `frozen_controls`,
`schedule`, `clean_ns`, `run`, `pairs`; frozen child `clean-repeat` and `apply`.

The timer is the child's `Instant` around mirror, including its root binding.
Pool construction and one **same-control clean warmup** are outside it.
`process_ns` additionally includes those, startup and 2 ms wait polling: it is
not an exact operation or whole-process comparison. Controller preservation,
hashing, setup and verification remain outside API timing but condition caches
and add real I/O/CPU cost. Verification binds native names/type/mode/size/mtime,
device/inode and literal links, with full content hashes at boundaries; it does
not promise atime/ctime/ACL/xattr invariance or protection against malicious
change-and-restore between observations.

Per-sample metadata verification and the child warmup intentionally warm the
same namespace. Normal checked mirror requires **no such prewalk**. These API
values therefore describe warm repeated clean work, not an uncached first payload
walk or total application latency. The two frozen Cargo locks were independently
compared: only the runner package name differs; dependency versions/checksums match.

Read-only suitability census of the previous **owned** real-donor clone recovered
**191,805 entries: 160,407 regular files, 22,827 directories excluding root,
8,571 symlinks, zero specials, 8,291,060,621 logical file bytes**. No live workspace
is synchronized. Native `setup --source` creates a NEW private pair from that
owned clone and verifies source/target byte/metadata parity. This is actual donor
topology, not a flattened synthetic tree or a claim of the latest live revision.
The private two-copy setup took **21.835 s**, and setup's byte/metadata verification
took **140.589 s**. The latter reads source and target for parity, then again for
the full identity hashes: roughly 33.16 GB logical data, plus metadata hashing.
Those costs are not library initialization or timed clean work. The source and
target have separate recorded inode identities. Subsequent campaign boundary
hashing similarly reads both trees and conditions caches; it is retained, not
subtracted from or confused with the child API timer.
Full comparison should visit 22,828 directory pairs; this clean child protocol
does not emit measured directory/inspection/syscall counters.

Thread count is campaign-wide, so eight balanced blocks **per mode** are
partitioned into four-block campaigns ordered **serial-A → P4-A → P4-B → serial-B**.
A pair uses seed 1701, B pair 1702. Each mode has 32 samples (eight per control),
all retained, with no repeat-until-fast rule. Both P4 campaigns use P4/J4/K256.
Mirrored positions mitigate order/drift; execution modes are still not adjacent
randomized trials. Host I/O is unavoidably unstable and builders may be active;
the advisory lock coordinates only cooperating campaigns. Raw outputs and exact
commands live under `.test-agent/clean-reassessment0.sol61x/`.

## Results

**64/64 successful samples; zero failures/exclusions/retries.** All four native
summaries report `passed: true`, no initial/final error, zero completed operations,
and unchanged byte/mode/size/mtime/link/device/inode identities. Both private data
trees were recounted after the runs and retain the exact 191,805-entry shape.
Original frozen controller/child hashes are unchanged. No new syscall or physical
inspection counts are invented: this protocol emits operation zeros, not those
counters. Earlier trace evidence is separate.

API milliseconds, **median [min–max]**, n=8 per cell, all observations retained:

| Control | Serial | P4/J4/K256 |
|---|---:|---:|
| Baseline | 697.28 [680.58–1526.52] | 237.76 [224.49–278.86] |
| Default `NoCheck` | 722.29 [683.51–832.94] | 232.72 [222.94–316.73] |
| Enabled no-op | 711.29 [679.04–810.22] | 235.96 [221.39–531.76] |
| Native name check | 726.10 [688.59–928.98] | 239.57 [226.52–323.86] |

Actual within-block API ratios, **median [min–max]**, n=8 per cell; 1 means equal.
Block identity is `(campaign, local block)`. These are medians of individual
ratios, **not ratios of aggregate medians**:

| Numerator / comparator | Serial | P4 |
|---|---:|---:|
| Default / baseline | 1.007 [0.491–1.201] | 0.997 [0.799–1.362] |
| No-op / baseline | 0.994 [0.468–1.048] | 1.033 [0.794–2.135] |
| Name check / baseline | 1.011 [0.510–1.339] | 1.024 [0.837–1.398] |
| No-op / default | 0.973 [0.825–1.170] | 0.997 [0.745–2.252] |
| Name check / default | 1.040 [0.858–1.125] | 1.035 [0.887–1.387] |

The name-check/default centers are about +4%, but spreads include both signs;
they do **not** establish an exact causal predicate tax or zero overhead. These
are full public-API specializations: frozen child `apply` calls free `mirror*`
for `none`, while `noop`/`admin` directly construct generic checked appliers.
Code generation/entry routes differ too. No-op sometimes being faster than
default and its 532 ms P4 outlier reinforce why small centers are not an isolated
callback instruction measurement. Keep every tail; do not tune source or rerun
until favorable from this result.

Common-seed/block name-check P4/serial ratios are 0.344 median [0.255–0.414].
Those comparisons are cross-campaign, not adjacent randomized trials; nevertheless
every observed P4 name-check call was below every serial name-check call. This
supports the stated **N=1 warm-latency profile**, not a universal concurrency win.

## Verification cost, load and limits

Campaign initial/final full verification seconds: serial-A **49.670 / 81.287**,
P4-A **46.490 / 45.087**, P4-B **51.355 / 45.169**, serial-B **59.679 / 47.761**.
That is **426.498 s** of campaign boundary verification, outside API time, in
addition to setup's 140.589 s verification. Per-sample preparation costs
**1.013–1.674 s**, post-verification **1.023–2.724 s**; executable preservation
costs **15–41 ms** per campaign. They are real costs/cache conditioning, not
production-library overhead. `process_ns` remains unusable as exact API timing.

Boundary snapshots on this 16-online-CPU host recorded one-minute load
**5.32–21.69**, I/O PSI `some avg60` **0.13–2.51%**, and about **10.4–13.7 GiB**
available memory. They are sparse observations, not continuous extrema or
external-load-only measurements: our own large byte verification also causes
I/O. Builders/background work were not isolated or causally corrected. The
serial default/baseline half medians even reverse direction (A ~0.803, B ~1.165).
No claim of a universal before/after speedup or regression follows.

**JW guidance:** retain the compiled name/type guard and reuse already-observed
facts. This sample supports P4/J4/K256 for one large selected root when a four-
worker budget is available; do not disable checks over a small noisy center.
Serial remains the conservative resource fallback. K16 (`for_pool`'s P4 default),
shared-pool competing walkers, N destinations, absolute-link diagnostic collectors,
first/cold calls, initialization/sparse/subtree work and full JW commands were
**not measured**. Their tooling belongs to the ongoing Rust migration. Do not add
a production prewalk to reproduce this benchmark's warming. Mode-only and
same-size/same-mtime byte edits remain outside incremental repair semantics.

Raw evidence and exact hashes: [bundle](evidence/reassessment0.sol61x/README.md).
No production change is selected; reassess mutable/JW scenarios separately when
their Rust paths are proven.
