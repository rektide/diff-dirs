---
type: ImplementationAccount
title: Rust-only performance tooling — completed takeover
description: Native controls, real-tree import, exact artifact recipes, deliberate Python retirement and bounded verification evidence across diff-dirs and jj-waltz.
resource: /design/performance/tooling-completion0.sol61m.md
status: implemented-and-verified
generated: { by: "model:openai/gpt-6.1-sol#medium", at: 2026-10-18 }
sources:
  - resource: "opencode://ses_eb2165cc8ffe4vOmOvDBe7aXQu"
    title: Assessed the predecessor, implemented native Rust controls/import/build/count/report workflows, verified executable smokes, and retired active Python with historical provenance intact
  - resource: /design/performance/tooling-takeover0.sol61m.md
    title: Accepted assessment and takeover boundaries; retained as historical inventory
  - resource: /perf/runner/README.md
    title: Supported library artifacts, scenario controls and native diagnostics
  - resource: "file:///home/rektide/src/jj-waltz/tools/performance/README.md"
    title: Supported real donor import, isolated N2 native campaigns, application variants and process-count controls
  - resource: /perf/results/checked/README.md
    title: Preserved original checked-streaming experiment inputs, hashes and raw observations
  - resource: /design/performance/reassessment0.sol61x.md
    title: Independently owned real-shaped clean assessment; not a result inferred from this tooling smoke
---

# Outcome

**Supported performance tooling in both repositories is checked-in Rust.**
Retired 19 active diff-dirs Python files, six JW Python files and three obsolete
JSON recipes. Historical source snapshots under `perf/results/checked/` remain
explicitly historical; no `.test-agent`, raw result, frozen executable or retained
failed attempt was removed. Original code is revision-bound in local history:
D `997183df472f81d21e8d553a9002d083d636ef05`, J
`a35da35d7635e14be70fb4aab37ea6edbd933ef8` (`git show COMMIT:PATH`). No push is implied.

The [assessment](/design/performance/tooling-takeover0.sol61m.md) remains the
accepted basis and timestamped predecessor inventory, not current availability.
Historical accounts link exact sources and point readers to the Rust guides.

## Cohesive checked-in base

- D `perf/runner`: artifacts, seeded mirrored scheduling, fixture/scenario kernels,
  balanced execution, descriptive reporting, bounded native processes and diagnostics.
  J implements the same compile-time `Workload` interface; there is no second
  registry, generic config language, runtime cache or full-tree operation map.
- D `perf/operation`: one selected init/clean/sparse/subtree API operation over N
  independent destinations. The same source builds against exact baseline/candidate
  git archives; candidate-only `checks` gates the newer APIs, not library changes.
- J `tools/performance`: native tracked-tree import, isolated JJ setup, one native
  prepare/preview/reseed (or explicit first-fill phase bundle), streaming payload
  inode diagnostics, Rust JJ/Git exec wrappers, count/scaling gates, native cp baseline,
  and source-checked serial/payload-parallel artifact builds.
- The only JW Cargo edits were the development member and specifically requested
  optional `payload-parallel` wiring, committed at `3d206c42`. Production pool code
  and its tests belong to the application owner. No later Cargo edits were needed.

## Ready artifacts and recipes

### Library: baseline/default/noop/admin, serial and P4

From D, using NEW output names:

```sh
cargo build --locked --release -p diff-dirs-perf-runner
target/release/diff-dirs-perf-runner setup .test-agent/final-library-fixture
target/release/diff-dirs-perf-runner run .test-agent/final-library-fixture .test-agent/final-library-serial --frozen .test-agent/checked/frozen-fa91 --blocks 8 --seed 1701 --threads 0
target/release/diff-dirs-perf-runner run .test-agent/final-library-fixture .test-agent/final-library-p4 --frozen .test-agent/checked/frozen-fa91 --blocks 8 --seed 1701 --threads 4
```

To use real shape, give `setup` `--source` pointing to an already owned plain
snapshot/payload directory, never a live writable target. For rebuilt single-scenario
controls use `build-operation . fd277144c51700ca70d068fc88f3ff72a0d64293 NEW_BEFORE`
and `build-operation . 243be8e2a28a796c61652161c9dd30c6b35fc5d0 NEW_AFTER --checks`,
then `run ... --operation-builds NEW_BEFORE NEW_AFTER --scenario sparse
--destinations 2 --threads 4 --blocks 8 --seed 1701`. These uppercase arguments
are explicitly NEW directory arguments, not fabricated ready artifacts.

Actually verified builds are `.test-agent/rust-operation-{baseline,candidate}-v2/`;
all eight tiny scenario/N1/N2 campaigns passed (64/64 samples). Frozen originals
remain unchanged. Their reviewed benchmark SHA-256 values are baseline
`ab4a8dfe029892efc9a00cc92815c5cb3143dcb609c0db13eb4f6d27fd18afc9` and candidate
`a6906bea63e9fcaf8ccbb498a83e30ded4f0d93db28b1895962d207768fee362`.
Frozen default/noop/admin share the candidate executable but select distinct check
paths. The preset embeds original compiler/manifests/lock/source identities and
does not relabel later bytes as the original historical timing bytes.

### JW: actual opencode, all roots, N2 feature histories

Exact supported import/build/interleaved commands are in the
[J guide](file:///home/rektide/src/jj-waltz/tools/performance/README.md#actual-opencode-import-and-n2-campaign).
`snapshot "$HOME/a/a/opencode" NEW_IMPORT` performs read-only native JJ inventory
queries and required private reflinks. Inventory is pinned to a JJ operation and
commit; observed live disk bytes are explicitly not an atomic commit snapshot.
Only tracked files/links plus outermost Git-eligible node_modules/dist roots are
copied. An isolated Git index preserves forced-tracked semantics; administrative
state, unrelated ignored caches and donor freshen actions are excluded.

Pass `--fixture NEW_IMPORT/fixture` to BOTH candidates' otherwise identical runs.
Use `--destinations 2 --feature-files 2 --blocks 8 --seed 1701`, separate NEW outputs,
and scenarios init/clean/sparse. `--timeout-seconds` is explicit for slow native
commands. Only clean incremental candidates get `--require-clean-label candidate`.
Prepare/preview/native-count controls remain available without invoking live donors.

`jw-perf build . NEW_BUILD --variant serial|payload-parallel` pins and verifies
the current committed production inputs, records actual compiler/profile/features/
environment and input hashes, and preserves `NEW_BUILD/jw` plus provenance. It never
changes revisions or production profiles. Exact full revision overrides require
matching prepared checkouts; mismatches fail. Build all variants before campaigns.

Verified immutable app artifacts (source `48557e892b6cb5d09de342be1f8e285967b194df`,
library coordinate `ce1de9f8b08c9ed0b016d8cef60d682e5894e937`, whose runtime core is
the approved implementation) are:

| Variant | Location in J | SHA-256 |
|---|---|---|
| Preserved BEFORE, source `7253ff6ddee639a98087e1729af58e0cbc546b6f` | `.test-agent/inline-policy/jw-before` | `03e0ada406426d9496c602e22c9dd0de452f788e424dab9b7069c07f812e715c` |
| Candidate serial, features `[]` | `.test-agent/native-build-serial/jw` | `6ee5b43abde77cb63f1912e62b91cafc07c35236d0a723129c48974fa35d14e8` |
| Candidate, features `[payload-parallel]` | `.test-agent/native-build-payload-parallel/jw` | `8470f8a15b2cb435f2ee4d6ce82a0ce8c9250fd76491ad085d2a42167a637145` |

All record configured Rust 1.101 nightly `822182db4`, LLVM 23.1.3, unchanged
release settings. Provenance sidecars retain exact commands and source/manifest/
lock hashes. Later builds may produce different bytes: preserve and identify them,
never assume a filename or current compiler proves an old build's identity.

## Verified evidence, not speed claims

- Accepted clean proof: 64/64 frozen controls; seeded replay/position balance,
  corrupt-hash rejection, retained nonzero exits, zero clean operations and
  unchanged bytes/metadata/inodes. Preserved controller/data are still intact.
- Native library instrument: 48 executable cases across all scenarios, N1/N2,
  serial/P4 and none/noop/admin, plus 64 archived baseline/candidate samples.
- J native prepare/preview/reseed, sparse/subtree, copy baseline and N2 feature
  smokes passed. Rust instrumentation showed zero JJ/file-show/Git process growth
  from 4 to 100 tracked files. Native and instrumented timings remain incompatible.
- The retained real-tree integration smoke at
  `J/.test-agent/native-import-tests/.tmpujiIEy/` passed four eligible roots, a
  literal native link, forced-tracked binary data, N2 distinct feature histories
  and an unchanged donor JJ operation. A prior CWD-relative inventory failure
  remains at `.tmpLIv1TB/`; explicit donor CWD fixes it. Operation-pinned import
  also passed at `J/.test-agent/native-import-op-proof/`.
- Current serial and parallel candidates each passed four imported N2 clean
  samples at `J/.test-agent/native-import-{serial,payload-parallel}-clean/`,
  asserting zero candidate payload inode/mtime changes. BEFORE rewrites remain
  counted, valid historical behavior—not an invalidated baseline latency sample.
  The deliberate legacy-as-incremental failure is retained with no speed pairs.
- Rust retirement tests check active source trees, supported guides/CLI help and
  historical CSV reporting. Scoped importer/native-link verification is opt-in
  because it needs the preserved BEFORE; its exact command is in J's guide.
  Final tool-scoped Nextest passed 9 D tests and 7 default J tests; the retained
  opt-in native import test separately passed. Clippy and fmt passed in tool
  scope. No application correctness suite was repeated for retirement.

## Limits

No real opencode campaign was run by this implementation session. The independent
64/64 real-shaped clean assessment belongs to its original author; whole-JW real
measurements remain with the app owner. These tiny controls establish usable tools,
not speedup, significance, cold-cache behavior or elimination of unstable host I/O.

Setup, reflink resets, hashing and full verification can cause substantial I/O and
warm caches; costs/chronology are retained outside operation timers. Source copying
is non-quiesced. Unsupported submodules/missing tracked paths, unsafe/root symlinks,
embedded administrative metadata and tracked/payload overlap refuse explicitly.
The JW opencode adapter requires a tracked root `.gitignore` rather than silently
adding a tracked fixture file; the supplied actual workload satisfies this.
The fixture-only preparation installer expects a top-level node_modules (as the
actual opencode donor has); imported reseed-only cases do not run installers.
PATH exec counts can miss absolute child paths/private Git helpers. Process groups
are bounded on timeout, not a hostile-filesystem/security sandbox. No automatic
pruning, scratch deletion, hidden failed-sample filtering or causal load correction.
