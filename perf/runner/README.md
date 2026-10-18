---
type: ToolGuide
title: Rust performance controls — clean and selectable scenarios
description: Checked-in native driver for frozen baseline/default/noop/name-check controls, mirrored samples and retained failures; migration beyond clean controls remains explicit.
resource: /perf/runner/README.md
status: implemented-native-scenarios
generated: { by: "model:openai/gpt-6.1-sol#medium", at: 2026-10-18 }
sources:
  - resource: "opencode://ses_eb2165cc8ffe4vOmOvDBe7aXQu"
    title: Implemented clean proof, modular Rust primitives and selectable verified N1/N2 scenarios; continuing Rust-only migration
  - resource: /design/performance/tooling-takeover0.sol61m.md
    title: Inventory, failure findings and bounded takeover basis
  - resource: /.test-agent/checked/frozen-fa91/README.md
    title: Preserved executable/build/source identities, including later rebuilt baseline benchmark
---

# Rust clean-control runner

Development-only Cargo member; **no production-library dependency changes**.
Linux/Unix, GNU `cp` and required-reflink filesystem. No Python dependency.
This first slice drives the frozen `checked-bench clean-repeat` protocol, not an
arbitrary command/config framework. It supports baseline/default/enabled-noop/
native-name-check, serial or explicit borrowed P4/J4/K256 in the child.

## Selectable scenario instrument

The checked-in `perf/operation` crate adds `init`, `clean`, `sparse`, `subtree`,
and N independent destinations. `scenario.rs` owns fresh private resets, exact
starting-equivalence signatures, source immutability and independent payload
verification (bytes/modes/file mtimes/literal links; directory/link mtimes are
not promised by synchronization). Clean additionally asserts unchanged inodes
and all entry metadata. Setup/verification remain outside the operation timer.
Every attempt and its reset are retained, with wall chronology and I/O pressure.

Build baseline and candidate from exact local git archives with the SAME checked-in
instrument source; the candidate-only `checks` feature is the only API gate:

```sh
cargo build --locked --release -p diff-dirs-perf-runner
target/release/diff-dirs-perf-runner build-operation . fd277144c51700ca70d068fc88f3ff72a0d64293 .test-agent/operation-before-NEW
target/release/diff-dirs-perf-runner build-operation . 243be8e2a28a796c61652161c9dd30c6b35fc5d0 .test-agent/operation-after-NEW --checks
target/release/diff-dirs-perf-runner run .test-agent/clean-fixture-NEW .test-agent/scenario-NEW --operation-builds .test-agent/operation-before-NEW .test-agent/operation-after-NEW --scenario sparse --destinations 2 --threads 4 --blocks 8 --seed 1701
```

Use a separate NEW output for each scenario/count/pool selection. Archived sources,
instrument/helper source copies, compiler/profile/features, exact build command,
resolved lock and immutable executable hashes are retained in each build root.
The builds share a common lock seed and preserve the actual resolved lock rather
than claiming an old library lock includes new development tools. No production
code or profile is rewritten. Building and measuring both honor the cooperative
host lock. An initial attempt correctly refused `WouldBlock` during independent
clean reassessment. The first archive build then exposed a nested-workspace
manifest error; its failure logs remain at `.test-agent/rust-operation-baseline/`.
The explicit `source` workspace exclusion fixes that development manifest.
Actual builds at `.test-agent/rust-operation-{baseline,candidate}-v2/` succeeded;
all **64/64** tiny balanced samples passed across 4 scenarios × N1/N2 × 4 controls
× 2 blocks. Raw campaigns are `.test-agent/rust-operation-SCENARIO-nN/`.

Nine Rust tests pass, including an actual native instrument integration test with
48 tiny cases: 4 scenarios × N1/N2 × serial/P4 × none/noop/admin. Required reflink
reset tests assert matching starts across repeats and independent target inodes.
The accepted clean CLI remains compatible after splitting artifacts, scheduling,
fixture/scenario, execution/report and explicit operation builds into modules.

## Ready commands (from diff-dirs)

Use NEW fixture/output paths. The frozen inputs are never overwritten.

```sh
cargo build --locked --release -p diff-dirs-perf-runner
target/release/diff-dirs-perf-runner setup .test-agent/clean-fixture-NEW
target/release/diff-dirs-perf-runner run .test-agent/clean-fixture-NEW .test-agent/clean-serial-NEW --frozen .test-agent/checked/frozen-fa91 --blocks 8 --seed 1701 --threads 0
target/release/diff-dirs-perf-runner run .test-agent/clean-fixture-NEW .test-agent/clean-p4-NEW --frozen .test-agent/checked/frozen-fa91 --blocks 8 --seed 1701 --threads 4
```

`setup NEW --source OWNED_SOURCE` privately reflinks a supplied read-only payload
directory, then creates a matching independent target. For independent real-shape
reassessment, the original checked-library owner can choose its existing owned
`.test-agent/checked/live-node-modules/control-target` as `OWNED_SOURCE`, verify
its suitability and use NEW cases. No JJ command, donor preparation or live reseed
is involved. Source copying is non-quiesced; the resulting owned pair is checked
for payload/metadata equivalence before use. Setup reads and hashes contents;
that can be significant I/O. Do not run competing campaigns/builds/tests.

Custom **checked-bench-protocol** controls use repeated
`--control LABEL BINARY EXPECTED_SHA256 none|noop|admin PROVENANCE_JSON` instead
of `--frozen`. Provenance is embedded and hashed, not inferred from a label or
current compiler. Imports are operator assertions; frozen presets explicitly
record runtime revisions, original compiler/profile/features, source/lock hashes
and manifests. Baseline is `fd277144`, candidate runtime `243be8e2`.
Build variant executables separately, never during `run`. Existing same-source
rebuild recipes/manifests are retained in [checked evidence](/perf/results/checked/README.md).
The `checks` harness feature enables candidate-only APIs without changing the
library interface; the same candidate binary selects none/noop/admin at runtime.

## Guarantees and output

- `setup` refuses an existing root; `.owned` and `fixture.json` bind the canonical
  case, data, modes, links, mtimes and inodes. Clones are private, required reflink.
  Special/admin entries are refused. There is no cleanup/prune command.
- `run` creates a NEW output, preserves hash-checked executable copies under
  `artifacts/` (mode 0555), and rehashes before every invocation. Same-user edits
  are detectable, not prevented by a security sandbox.
- Seeded permutations immediately followed by reverse permutations balance
  positions every two blocks; eight blocks produce 32 chronological samples for
  four controls. Same seed replays order. Shared advisory lock prevents overlap
  with cooperating runners, not unrelated host I/O.
- API nanoseconds come only from the child's `Instant` around the mirror call.
  The frozen clean child separately creates its pool and performs an untimed warmup.
  It asserts zero completed operations. The controller validates exactly one
  correctly labeled clean CSV row with zero operations, and unchanged fixture
  metadata/inodes after each sample. Full byte checks run before/after campaign.
- `preparation_ns`, `verification_ns`, setup costs and artifact preservation are
  retained but excluded from API timing. `process_ns` includes startup, pool,
  warmup for the frozen protocol, distinct from API scope. The current process
  helper blocks in `wait4` and retains child CPU/peak RSS; the preserved first
  proof executable used 2ms polling. Never conflate those process scopes.
  No cache drops, cold-cache, significance or causal host-load correction claim.
- `samples.jsonl` is append-only/flushed/synced per attempt; stdout/stderr are
  retained separately. Failed hash/child/parser/fixture checks retain their slot
  with no API value. `summary.json` includes all successful within-block pairs
  and descriptive median/min/max ratios; failed samples are never speed results.
  Campaign failure exits nonzero. Native child timeout is 1800s; this fixed
  native process helper kills ordinary descendant process groups on timeout.
- `campaign.json` retains exact order, controls, original build provenance, runner
  executable/source hashes, compiler/profile and build source/manifest. Initial
  and final full verification failures are explicit. A killed run can lack its
  final summary; never call a raw partial file a completed campaign.

## Checked executable proof

Actual controls (not cp stand-ins): `.test-agent/rust-clean-proof-{serial,p4}/`
each passed **32/32 samples, eight matched blocks per candidate**, seed 1701.
`.test-agent/rust-clean-proof-fixture/` remained byte/metadata/inode-equivalent.
`.test-agent/rust-clean-proof-failures/` retained two successful baseline slots,
two rejected wrong-hash candidate slots and two nonzero-exit baseline/noop slots;
it failed honestly, with **zero pairs**. No original frozen artifact was changed.
`.test-agent/rust-clean-proof-runner` preserves the actual controller executable.
These tiny runs prove plumbing, **not a performance win**; raw spreads are large.

```sh
cargo nextest run -p diff-dirs-perf-runner
cargo clippy -p diff-dirs-perf-runner --all-targets -- -D warnings
cargo fmt -p diff-dirs-perf-runner -- --check
```

The original four tests covered seed/replay, strict clean parsing, private reset/
change detection and corrupt-artifact rejection. Current tests also cover N2
scenario equivalence, executable modes, process failure/group timeout and CSV/
PID-prefixed trace auditing. Include `-p diff-dirs-perf-operation` for its native
48-case executable test.
Initial tests on `/tmp` correctly failed required reflinks; tests now create tiny
private roots on the checkout filesystem under `.test-agent/runner-tests/`.
Nextest emits build-script-output lookup warnings on this configured nightly;
the actual builds/tests succeed. Clippy and formatting checks passed.

## Rust-only endpoint and historical source

Supported performance workflows in both repositories are **checked-in Rust**.
The 19 active D Python entrypoints/modules (the old matrix/leaf/report/audit/
resource scripts and experiment package), six J Python files and three obsolete
JSON recipes are retired. Their exact source remains in committed history:
D `997183df472f81d21e8d553a9002d083d636ef05`, J
`a35da35d7635e14be70fb4aab37ea6edbd933ef8`. `perf/results/checked/*-run.py`
are explicitly historical experiment-source snapshots, never supported commands.
No `.test-agent` tree, raw data, original frozen binary or source snapshot was
removed. Historical accounts are retained, with revision-bound source pointers.

The scenario instrument and the JW native adapter now exist. J's Rust-owned
prepare/preview/reseed, exec process-count wrappers, count/scaling gates, feature
history/root checks and raw-copy baseline passed tiny real-binary controls. Its
historical BEFORE clean rewrites remain valid measured behavior; zero-write
requirements apply only to an explicitly named incremental candidate. Both
projects share one compiled `Workload`/balanced execution interface, not catalogs.
Rust `report`, `summarize-csv`, `audit` and `resources` replace report/audit/resource
entrypoints. J's native `snapshot` additionally imports the actual tracked
inventory and outermost Git-eligible payload roots into a verified owned fixture;
its `build` records/preserves serial and payload-parallel app artifacts. See the
[J guide](file:///home/rektide/src/jj-waltz/tools/performance/README.md) for the
actual opencode/N2 commands. [Completion and evidence](/design/performance/tooling-completion0.sol61m.md)
records usable identities, checks, failure retention and genuine limitations.
Large workload campaigns remain independently owned, not results inferred from
these plumbing smokes.
