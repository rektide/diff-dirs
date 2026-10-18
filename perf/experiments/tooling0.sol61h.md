---
type: ImplementationAccount
title: Matched experiment tooling prototype — stopped for assessment
description: Inspectable cross-repository prototype, checks, preserved smoke artifacts, failures and unfinished work; not accepted tooling or performance evidence.
resource: /perf/experiments/tooling0.sol61h.md
status: incomplete-stopped-by-user
generated: { by: "model:openai/gpt-6.1-sol#high", at: 2026-10-18 }
sources:
  - resource: opencode://ses_eb2165cc8ffe4vOmOvDBe7aXQu
    title: Recorded retirement standing and exact predecessor revision; prototype findings unchanged
  - resource: opencode://ses_eb224880fffeZxuFkeb0yDaCeN
    title: Implemented independent immutable-artifact and matched-block prototype; stopped expansion at human request and preserved successful and failed smoke controls
  - resource: /design/performance/tooling-audit0.glm53fx.md
    title: Existing runner inventory, grouped hyperfine execution and timing-scope hazards
  - resource: /design/performance/comparison0.gpt6a.md
    title: Architecture experiment context and sequential multi-destination distinction
  - resource: /design/sync/checked0.sol61x.md#tooling-handoff-and-independent-reassessment-checkpoint
    title: Exact frozen checked-streaming artifacts and deferred performance judgment
  - resource: /perf/README.md
    title: Existing comparison/sync runner contracts, retained evidence and cache conditions
  - resource: file:///home/rektide/src/jj-waltz/tools/performance/README.md
    title: Existing native/instrumented measurement contract and first-fill limitation
---

# Stopped prototype checkpoint

Historical stopped prototype; active Python modules/recipes were subsequently
retired. Use [Rust tooling](/perf/runner/README.md). Sources below belong to
`997183df472f81d21e8d553a9002d083d636ef05`; recover with `git show COMMIT:PATH`.
The old commands are transcripts, not current supported instructions.

**The user stopped expansion because too much Python was accumulating.** This is
preserved for independent read-only assessment, not a proposed mandatory framework.
No production Rust, Cargo profiles, existing runners, frozen controls, or existing
performance accounts were edited. No meaningful latency campaign was performed.
No further work is implied by this checkpoint.

## Implemented shape and footprint

- `perf/experiment.py` dispatches to `perf/experiments/`: registry, fixture handling,
  schedules, execution, reporting, CLI and the frozen-control adapter are modules.
- JW's `tools/performance/experiment.py` loads this same engine from the sibling
  checkout, or `DIFF_DIRS_PERF`. No copied engine and no installed dependency.
- Separate `register [--build]` preserves binaries at spec/hash-qualified paths;
  optional expected SHA-256 is checked. Saved binaries are rehashed before samples.
  Build argv/profile/features/environment and compiler output are recorded. Builds
  require a revision command and clean-state command; dirty builds need explicit
  permission and hashed inputs. Import provenance remains the operator's assertion.
- One permutation immediately followed by its reverse gives AB/BA (ABBA chronology)
  and mirrored positions for larger variant sets. Seed/order are retained. One
  shared advisory lock prevents these launchers from competing, not unrelated load.
- Hyperfine gets **one command and one observation** per invocation, `-N`, no
  internal warmup; JSON is retained. Installed 1.20.0/help were inspected; local
  archived source was not found. CSV mode reads existing runner-owned timers,
  preserving rows and aggregating internal replicates within each outer block.
- Private fixtures use GNU `cp -a --reflink=always`, metadata fingerprints, and
  fresh states per mutation sample. Setup/warmup/reset/verification are excluded
  and logged. The frozen adapter instead uses one owned shared source and separate
  clean/delta cases to avoid multiplying 191k-entry clones by every observation.
- Results retain raw order, failed attempts, per-block ratios, median/range/MAD,
  process/pressure snapshots and no statistical confidence or causal load claims.
- Python footprint: **1,051 lines here including 230 test lines; 209 additional
  lines in JW** (adapter/launcher). This is too substantial to describe as a tiny
  wrapper. No new packages were installed. Review/simplification is explicitly next.

## Commands actually run and retained evidence

From diff-dirs:

```sh
python3 perf/experiment.py fixture .test-agent/matched-smoke/fixture --files 4 --bytes 64
python3 perf/experiment.py register perf/experiments/examples/smoke.json .test-agent/matched-smoke/store
python3 perf/experiment.py run perf/experiments/examples/smoke.json .test-agent/matched-smoke/store/registry.json .test-agent/matched-smoke/run --blocks 2 --seed 17 --warmups 1
python3 perf/experiment.py report .test-agent/matched-smoke/run/result.json
PYTHONPATH=perf python3 -m unittest discover -s perf/experiments -p 'test_*.py' -v
```

Final check: **12 tests passed**. They cover order symmetry, quoting/native paths,
failed commands/builds, saved-artifact corruption, fixture matching/private reset,
scenario starts, read-only snapshot selection, CSV aggregation, pairing identity,
and separate frozen clean/delta cases. Test-owned temporary files are normally
removed by unittest; campaign scratch and useful controls were not cleaned up.

`.test-agent/matched-smoke/run/` passed six same-cp-label observations. Two measured
paired changes were **−27.4355% and +30.1836%**, median **+1.3740%**: correctly
inconclusive, not an optimization. Every hyperfine JSON/output and reset remains.

`.test-agent/matched-smoke/frozen-plan/`, `frozen-store/`, `frozen-run/` preserve the
first full frozen-adapter tiny-fixture attempt: it failed when mutation jobs
changed directory mtimes in a shared clean case. Code now separates cases and its
regression test passes, but **the actual frozen-binary full run was not repeated**
after the user's stop. Do not present this adapter as end-to-end revalidated.

JW's sibling account records its passing and failing native smoke cases:
[JW handoff](file:///home/rektide/src/jj-waltz/tools/performance/experiments/handoff0.sol61h.md).

## Historical retired library recipe — NOT current commands

Frozen originals remain untouched under `.test-agent/checked/frozen-fa91/`.
The adapter checks `identity.json`'s hashes:

- baseline live `d222d7ef2911d22d81a20853414ee5ef01b00c23ed924c1d0f0619c2e22b51ef`
- candidate live `bbf8caee18fca5e75a1b43abe8b42b2a43a70bdaebabb6acd282d9a8d6fb7948`

These are the **later clean-capable trace-command executables**, not the original
timing hashes. Registry variants baseline/default/admin use exactly those bytes.
Compiler is retained Rust 1.101 nightly 822182db4 / LLVM 23.1.3; library parallel
only, original release profile. Candidate runtime source is
`243be8e2a28a796c61652161c9dd30c6b35fc5d0`, unchanged at the final comment checkpoint.

After assessment, an operator can create a NEW owned snapshot from the read-only
donor (no live JJ operation), using a JSON file containing `[]` for payload-only:

```sh
python3 perf/experiment.py fixture .test-agent/paired-real/fixture --donor /home/rektide/archive/anomalyco/opencode --tracked-json /path/to/empty-array.json --payload node_modules
python3 perf/experiment.py frozen-plan .test-agent/checked/frozen-fa91 .test-agent/paired-real/fixture .test-agent/paired-real/plan
python3 perf/experiment.py register .test-agent/paired-real/plan/manifest.json .test-agent/paired-real/store
python3 perf/experiment.py run .test-agent/paired-real/plan/manifest.json .test-agent/paired-real/store/registry.json .test-agent/paired-real/run --blocks 12 --seed 1701 --warmups 1
python3 perf/experiment.py report .test-agent/paired-real/run/result.json
```

`frozen-plan --jobs all` also registers initialization and the existing
clean/sparse/subtree bundled in-process runner. It does not pretend those are
single-scenario startup timings. Default clean jobs compare serial/P4 independently.
Do not run this while G builds/tests or another campaign runs. Original Sol xhigh
should independently inspect raw paired blocks and failures before any verdict.

## Known limits / unfinished

- The real snapshot recipe and repaired frozen full adapter are unexecuted.
  There is no real workload speedup result from this session.
- Generic private resets retain every state: metadata/storage cost can be large.
  No cleanup/reconciliation is implemented or authorized. Frozen shared cases are
  the more scalable clean-only path; mutation semantics are still runner-owned.
- Metadata fingerprints are not content snapshots or a hostile-filesystem sandbox.
  Donor copying is read-only but non-quiesced; concurrent donor mutation can spoil
  equivalence. External live jobs have only declared identity unless explicit
  `fixture_checks` are supplied. No global cache dropping or cold-cache promise.
- Build source/feature declarations and imported compiler provenance need review;
  dependency worktrees are not automatically archived. Hashed inputs are explicit.
- No library feature/crate refactor, existing-driver consolidation, general
  multi-destination library benchmark, or index/README integration was completed.
  Native paths are tested on this Linux/GNU-cp host only. Framework code is not
  production-hardened; inherited config, report schema strictness and failure
  boundaries merit assessment. Seeded scheduling does not remove unstable I/O.
