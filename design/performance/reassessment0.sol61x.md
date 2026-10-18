---
type: PerformanceAssessment
title: Independent Rust-runner reassessment of real-shape clean synchronization
description: Matched frozen baseline/default/no-op/native-name controls on a private copy of the actual opencode payload shape; clean only, preliminary and not end-to-end JW evidence.
resource: /design/performance/reassessment0.sol61x.md
status: in-progress
generated: { by: "model:openai/gpt-6.1-sol#xhigh", at: 2026-10-18 }
sources:
  - resource: "opencode://ses_eb24f98a3ffewdVqa9lao6tXQu"
    title: Resumed the original implementation session to independently review the frozen Rust clean protocol and assess real-shape matched controls without changing tooling or production
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
---

# Independent real-shape clean reassessment

**Preliminary, clean only, not end-to-end JW.** Production and runner sources are
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

Read-only suitability census of the previous **owned** real-donor clone recovered
**191,805 entries: 160,407 regular files, 22,827 directories excluding root,
8,571 symlinks, zero specials, 8,291,060,621 logical file bytes**. No live workspace
is synchronized. Native `setup --source` creates a NEW private pair from that
owned clone and verifies source/target byte/metadata parity. This is actual donor
topology, not a flattened synthetic tree or a claim of the latest live revision.
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

Campaigns in progress. No performance selection or regression conclusion is
made from the earlier chronology-separated runs; this report will assess only
the predeclared Rust-runner samples and their correctness/cost evidence.
