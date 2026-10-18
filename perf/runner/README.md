---
type: ToolGuide
title: Rust clean-control runner — first proven slice
description: Checked-in native driver for frozen baseline/default/noop/name-check controls, mirrored samples and retained failures; migration beyond clean controls remains explicit.
resource: /perf/runner/README.md
status: implemented-clean-slice
generated: { by: "model:openai/gpt-6.1-sol#medium", at: 2026-10-18 }
sources:
  - resource: "opencode://ses_eb2165cc8ffe4vOmOvDBe7aXQu"
    title: Implemented and verified Rust clean-control foundation after tooling assessment and explicit Rust-only end-state direction
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
  The child separately creates its pool and performs an untimed clean warmup.
  It asserts zero completed operations. The controller validates exactly one
  correctly labeled clean CSV row with zero operations, and unchanged fixture
  metadata/inodes after each sample. Full byte checks run before/after campaign.
- `preparation_ns`, `verification_ns`, setup costs and artifact preservation are
  retained but excluded from API timing. `process_ns` includes startup, pool,
  warmup and 2ms wait polling; it is **not** a precision whole-process comparison.
  No cache drops, cold-cache, significance or causal host-load correction claim.
- `samples.jsonl` is append-only/flushed/synced per attempt; stdout/stderr are
  retained separately. Failed hash/child/parser/fixture checks retain their slot
  with no API value. `summary.json` includes all successful within-block pairs
  and descriptive median/min/max ratios; failed samples are never speed results.
  Campaign failure exits nonzero. Native child timeout is 1800s; this fixed
  protocol has no child subprocess tree to manage.
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

Four tests passed: mirrored seed/replay, strict clean parser, private reset/
change detection, and retained corrupt-artifact rejection without pairing.
Initial tests on `/tmp` correctly failed required reflinks; tests now create tiny
private roots on the checkout filesystem under `.test-agent/runner-tests/`.
Nextest emits build-script-output lookup warnings on this configured nightly;
the actual builds/tests succeed. Clippy and formatting checks passed.

## Explicit migration remainder

Human acceptance now requires **Rust-only active performance workflows in both
repositories**. This checkpoint is not that completed migration. Existing Python
entrypoints remain temporarily present: D `perf/{experiment,compare,frontier,leaf,
report,audit,resources,leaf_trace}.py` / `experiments/`; J `tools/performance/
{experiment,jw_sample,measure,fixture,copy_baseline}.py`. They are NOT dependencies
of this Rust path. Do not expand them or treat this as permission to retain them
as the final supported interface.

Next bounded slice: a checked-in Rust single-operation scenario instrument for
initialization/clean/sparse/subtree, equivalent owned resets, N destinations and
payload verification; reuse this ordering/artifact/raw-result core rather than
another registry/platform. Then a Rust JW-specific adapter: reproduce the old
N2/feature mismatch on the preserved baseline, assert requested feature counts,
final roots/target identities, and per-variant clean inode behavior. Baseline
full-copy clean behavior must be recorded, not rejected by candidate-only
zero-write requirements. Coordinate J Cargo membership with its production owner.
Rust process-count wrappers/diagnostic parity, native copy baseline, reporting and
read-only audit/resource workflows must be implemented/verified before deliberate
retirement of superseded tracked Python interfaces. Preserve committed history,
raw data, frozen executables and historical experiment sources; fix historical
links with revision-qualified references when active paths retire. No sweeping
scratch cleanup. Large campaigns belong to independent reassessment, not this
plumbing implementation session.
