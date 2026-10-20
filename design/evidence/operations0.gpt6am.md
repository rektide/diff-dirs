---
type: OperationalDesign
title: Bounded, attributable native-test and measurement evidence
description: Fixture topology, executable identity, and bounded measurement contracts for diff-dirs and JJ-waltz, grounded in current test and harness code.
resource: /design/evidence/operations0.gpt6am.md
status: proposed
generated: { by: "model:openai/gpt-6-astra#medium", at: 2026-10-20 }
sources:
  - resource: "opencode://ses_ea7ec000bffeGz2916lafDGvQV"
    title: Independent operational research, source inspection, implementation slices and ticket findings
  - resource: /design/roadmap/brief0.gpt6a.md
    title: Commission and independent reading boundary
  - resource: "file:///home/rektide/src/jj-waltz/design/up-performance/streaming0.sol61m.md"
    title: Historical native-test results and checkout-contained discovery incident
  - resource: "file:///home/rektide/src/jj-waltz/design/sync-scope/recovery0.gpt6a.md"
    title: Cancellation, retained-copy burden and proportional measurement requirement
  - resource: "file:///home/rektide/src/jj-waltz/CONTRIBUTING.md"
    title: Current Nextest workaround and fixture-placement guidance
  - resource: "file:///home/rektide/src/jj-waltz/tests/colocate.rs"
    title: Isolated configuration and fixed interceptor delegation
  - resource: "file:///home/rektide/src/jj-waltz/tests/reseed.rs"
    title: Checkout-local CoW fixture and separate native executable selection
  - resource: "file:///home/rektide/src/jj-waltz/tests/cli.rs"
    title: Runtime binary lookup and remaining proxy delegation gap
  - resource: "file:///home/rektide/src/jj-waltz/.github/workflows/ci.yml"
    title: Pinned JJ compatibility matrix and current checkout commands
  - resource: "file:///home/rektide/src/jj-waltz/tools/performance/README.md"
    title: Current supported Rust performance interface
  - resource: "file:///home/rektide/src/jj-waltz/tools/performance/src/build.rs"
    title: Committed source checks, unique target and saved executable
  - resource: "file:///home/rektide/src/jj-waltz/tools/performance/src/context.rs"
    title: Clean environment and explicit native repository ownership checks
  - resource: "file:///home/rektide/src/jj-waltz/tools/performance/src/workload.rs"
    title: Per-sample topology, repeated copies and timing boundaries
  - resource: /tests/support/mod.rs
    title: Ordinary library fixture placement
  - resource: /perf/runner/README.md
    title: Existing Rust runner guarantees and limitations
  - resource: /perf/runner/src/artifacts.rs
    title: Preserved control copies and per-invocation hashes
  - resource: /perf/runner/src/build_operation.rs
    title: Archived-source instrument builds
  - resource: /perf/runner/src/execution.rs
    title: Sample lifecycle, failure records and timing scopes
  - resource: /perf/runner/src/scenario.rs
    title: Retained per-attempt reset topology and verification
  - resource: /perf/runner/src/fixture.rs
    title: Required reflink copy and streaming fingerprints
  - resource: /perf/runner/src/diagnostics.rs
    title: Existing trace audit and whole-process resource scope
  - resource: /perf/results/leaf/README.md
    title: Historical bounded syscall/resource evidence and separately saved final build
  - resource: /design/performance/reassessment0.sol61x.md
    title: Historical clean measurement and explicit verification/cache costs
extensions:
  tickets: [diffdir-tree-evidence-fixtures, diffdir-tree-evidence-artifacts, diffdir-tree-evidence-budgets]
verification:
  method: Source inspection and existing historical reports; no tests, benchmark runs, fixture creation or production changes
---

# Operational evidence architecture

Use the existing test helpers and Rust runner. The missing contracts are **where a
fixture may discover repositories, which executable a run actually uses, and how
much fixture material may remain resident**. They do not require changing the
directory engine's architecture or introducing a general experiment framework.

## Scope and inspected coordinates

Inspected on 2026-10-20:

- JJ-waltz Git HEAD `3b99f1d70a64c1251ccae9f4ca1942751671fe8a`.
- diff-dirs Git HEAD `013bec8a28d563ce47b30b8eedb7301815828d7a`.
- Scoped `git diff --stat HEAD` was empty for JW's `Cargo.toml`, `CONTRIBUTING.md`,
  `tests`, `.github`, `tools/performance`, and D's `Cargo.toml`, `tests`,
  `sync/tests`, `perf/runner`, `perf/operation`. Thus the code claims below refer
  to those committed paths, not the unrelated working-copy documents. These are
  inspection coordinates, **not** identities of an executable tested here.
- Read the commissioned brief and three tickets. The withheld architecture,
  full roadmap, parent transcript and current same-wave outputs were not read.
  Historical reports supply prior results; this session did not reproduce them.

## 1. Fixture placement, configuration and tool delegation

### Current state: fixed issues and remaining gaps

**Fixed locally:** `tests/colocate.rs:137–175` delegates an interceptor through a
launcher restoring the original PATH before executing its captured `jj`.
`with_shim_delegates_under_the_delegated_path_without_recursing` stages a
version-manager-style resolver and bounds recursion at depth eight. This is a
real fix with a meaningful regression specimen, not an open colocation-shim bug.

**Documented, incompletely encoded:** JW's CONTRIBUTING guide now says the full
suite's temporary root must be outside repositories. The historical checkout
TMPDIR incident is explained in `streaming0.sol61m.md:78–110`: a Git-only fixture
discovered the enclosing real `.jj`, created a real workspace, and was subsequently
deliberately cleaned up. It is not evidence that JJ routing itself was defective.
`colocate.rs:18` and `context_routing.rs:19` still call ordinary `tempdir()`;
their safety depends on caller TMPDIR. A new run can repeat the placement mistake.

**CoW placement already specialized:** `reseed.rs:39–59`, `prepare.rs`,
`creation_ignored.rs` and `reseed_bookmarks.rs` allocate checkout-local scratch
and explicitly initialize repositories. D's `sync/tests/{sync,checked,visitor,
authority,early_facts,permissions}.rs` and runner scratch likewise use checkout
storage. D's ordinary `tests/support/mod.rs:17–30` prefers `/tmp/opencode` and
otherwise the OS temporary directory; it does not invoke native repository tools.
There is no reason to force that ordinary walker fixture onto CoW storage.

**Configuration differs:** colocation removes inherited `JJ_*` and `GIT_*`, then
sets private HOME/JJ/Git/XDG state (`colocate.rs:38–54`). Reseed sets XDG config,
`JJ_CONFIG`, identity/editor but inherits HOME and Git environment
(`reseed.rs:74–85`); legacy `cli.rs:2039–2090` principally isolates XDG config.
The perf `Context` is stronger: `env_clear`, explicit environment and tool paths,
plus root checks before workspace creation (`context.rs:21–89,107–110` and
`workload.rs:253–283`). Reuse this discipline, with cross-platform adaptation.

**Delegation gaps remain elsewhere:** `cli.rs:1185–1223` canonicalizes the first
PATH `jj` and execs it under the intercepted PATH. Canonicalizing a manager shim
does not prove native JJ; this retains the recursion class fixed in colocate.
Reseed's interruption helpers instead skip canonical paths named `mise`
(`reseed.rs:218–224,1013–1016`), which is a manager-specific heuristic rather than
a complete delegation contract. The Linux perf adapter requires ELF
(`instrumentation.rs:49–56`); ELF rejects shell scripts but does not prove the
binary is JJ rather than an ELF tool manager. Do not generalize that check into
portable integration tests.

### Contract and smallest slice — `diffdir-tree-evidence-fixtures`

Add a small JW `tests/support` helper, initially adopted by discovery/colocation
and the CoW proxy tests, with two independent placement requirements:

1. **Discovery fixture:** canonical external parent with no enclosing `.jj` or
   `.git` (file or directory, including ancestors reached through symlinks).
   Reject a bad explicit override before running any native command. Git-only
   tests may then create their own Git root while remaining outside enclosing JJ.
2. **CoW fixture:** configured scratch parent on the intended filesystem; default
   to the existing project-local scratch. Native repository actions require an
   explicitly initialized owned root and `jj --ignore-working-copy root` equal
   to that root before workspace mutation. Also check each workspace's repo link.
   If a test needs discovery **and** CoW, both requirements must hold: use an
   external CoW parent or report an unavailable capability. Never silently fall
   back to checkout-contained discovery to obtain reflinks.

Use explicit fixture-root overrides instead of globally changing TMPDIR. An
ancestor-marker check catches malformed placement without probing a real repo;
owned root validation then checks the native result. Keep the test intended to
exercise discovery free to discover its *owned* root, rather than masking it
with a production `--repository` override or invented JJ ceiling configuration.

Give each native fixture private HOME, XDG and native config, removing inherited
JJ/Git selectors before installing the fixture's explicit values. Make required
CoW/mount settings consistently mean nonempty=true; current checks differ on
whether an empty variable counts as required. CI deliberately sets an empty
`JW_TEST_REQUIRE_COW` outside macOS (`ci.yml:58–59`). Capability absence must be
reported distinctly from a completed semantic assertion. Tiny required-reflink
tests prove clone success, different inode identity and source independence after
a destination write; a successful ordinary copy is not CoW evidence.

Centralize interception around the already-fixed PATH-restoring launcher. Record
the selected command, delegation PATH and version. For measurement, require an
explicit installed JJ path, obtain `--version`, and record hash; native magic alone
is insufficient. No interception helper should delegate through its own PATH.

**Discriminating checks:** an explicitly owned fake enclosing `.jj` makes fixture
allocation fail before spawning anything; an external Git-only fixture cannot
resolve an enclosing JJ repo; a symlinked bad parent is rejected; isolated configs
ignore deliberately injected JJ/Git selectors; the existing bounded shim test
runs against the common helper; one real CoW leaf verifies privacy. These are
focused fixture/helper tests, not another full-suite invocation.

## 2. Stable artifacts and source attribution

### Existing mechanisms worth preserving

- Mixed JW lookup remains: legacy `Command::cargo_bin`/`cargo_bin("jw")` in cli,
  context, routing and shell/UI tests versus compile-time `cargo_bin!("jw")` or
  `env!("CARGO_BIN_EXE_jw")` in newer tests. CONTRIBUTING's manual runtime
  `CARGO_BIN_EXE_jw=$PWD/target/debug/jw` repairs the historical Nextest failures,
  but can select a different binary from the compile-time paths.
- JW `build.rs:67–121` uses a unique output target, hashes app/library source
  inputs before/after, records features/compiler/environment, copies `jw` mode
  0555 and records SHA-256. D `build_operation.rs:25–143` archives exact source,
  copies helper/instrument inputs, preserves resolved lock and binary. D's
  `artifacts.rs:35–54` copies controls and rehashes; `execution.rs:91–94` rechecks
  each invocation. These are implemented protections against shared target
  replacement, not proposals to rebuild from scratch.
- Perf Context records tool paths/hashes. The shared runner embeds its own source
  hashes (`perf/runner/src/lib.rs:27–46`). These distinguish a runner rebuild
  from a production algorithm change.

### Attribution holes verified in current source

1. JW's `--library` argument selects the tree that is checked and recorded, but
   the build still runs Cargo against the application's unchanged manifest.
   `Cargo.toml:28–29` resolves `../diff-dirs`; `build.rs:68–95` never compares the
   selected library with Cargo's resolved dependency paths. Supplying another
   library path can therefore report coordinates for a tree not actually built.
2. JW's tracked-source `git diff --exit-code` does not reject untracked source
   files. The directory hashes do record them, but provenance should not describe
   that state as exactly the named commit. Its selected input list also omits
   possible Cargo config/toolchain/build-script inputs. Before/after equality
   is useful detection, not an immutable build sandbox.
3. JJ/Git executable hashes are reported, not frozen and checked before every
   native use. Manager/native selection and version/capabilities are not all
   recorded. GNU `cp` used by `fixture::clone_tree:100–108` is PATH-resolved and
   outside Context; its identity and timeout are not covered by native-command
   records. This affects reset/setup evidence even if measured `jw` is frozen.
4. CI installs pinned JJ versions, but its root checkout steps show no sibling
   diff-dirs checkout despite current path dependencies. That is a source-layout
   mismatch visible in the workflow, not a claimed executed CI failure. Resolve
   the sibling revision policy before claiming that matrix verifies this tree.

### Contract and smallest slices — `diffdir-tree-evidence-artifacts`

**A. Uniform test selection:** one JW test helper uses compile-time Cargo binary
location by default; eliminate legacy runtime lookup. For deliberately testing a
preserved CLI, allow one explicit test-binary override in that helper and print
its canonical path/hash. No implicit fallback to `target/debug/jw`. Run Cargo and
Nextest against the same focused test with runtime `CARGO_BIN_EXE_jw` absent;
both must name the same intended artifact. A custom target-directory test and
an override-path test establish that selection is not hardcoded.

**B. Bind actual build inputs:** use Cargo metadata's resolved package manifest
paths to compare `diff-dirs` and `diff-dirs-sync` against the selected library
before building. Fail a mismatched `--library`. Extend the recorded coordinate to
the actual selected source hashes, lock, effective features, profile, target,
compiler/toolchain/config inputs and build argv. Either reject source additions
outside the commit or explicitly mark an experiment source overlay with its
hashes; never label it pristine. Archive builds already provide the stronger
pattern when exact committed reproduction is needed. Have CI explicitly fetch
the chosen sibling revision and include both source IDs in its report.

**C. Stable run:** finish builds before measurement; use per-build target roots
and the existing preserved controls. A run record binds application + sibling
library + driver/instrument + features to executable SHA-256, and records native
JJ/Git/cp path/hash/version plus relevant capabilities. Resolve tools once under
the run environment, verify hashes at phase boundaries, abort if replaced.
Keep manager launchers out of measurement. A frozen `jw` is separate from a
developer's live/demo CLI; do not rebuild or overwrite that demo during a run.

Verification is small: wrong-library refusal before Cargo build, a deliberate
source-overlay attribution test, corruption/replacement refusal using tiny fake
tools, and existing preserved-control corruption coverage. Reuse current artifact
code rather than invent a second registry. Mode 0555 and hashes detect accidents;
same-user concurrent mutation remains out of band.

## 3. Proportional checks and bounded residency

### Current cost and missing boundary

The historical cancellation (`recovery0.gpt6a.md:165–209`) involved approximately
seven payload copies per N2 sample and retained cases; setup/verification dwarfed
native operation time and the filesystem reached 99% occupancy. The report
explicitly does not attribute all disk usage to that campaign. Its unfinished
matrix remains cancelled; this design is no authorization to recreate it.

Current source still has that operational shape. `scenario.rs:95–151` creates
source plus N targets for each sample and retains removed subtrees.
JW `workload.rs:159–170,200–249,264–285` then constructs a remote, repository and
worker payload copies from that case. `execution.rs:69–107` creates a new case
stem for every slot and continues after sample errors. There is no resident-case
cap or phase progress emitted while a sample is running; the append-only sample
record appears after `sample()` returns. Existing command timeouts and process
groups do not bound `clone_tree`'s direct `Command::output` or synchronous hashing.
The host lock only coordinates participating programs.

The clean-only path already reuses a fixture and checks metadata each sample with
full byte checks at boundaries. Its historical reassessment explicitly records
cache warming and hundreds of seconds of verification. Reusing that verification
scope is defensible for warm-clean evidence, not a first-touch or content-integrity
claim. Keep semantic correctness and performance questions distinct.

### Experiment contract — `diffdir-tree-evidence-budgets`

Every proposed experiment specifies the following before allocating its fixture:

| Item | Required declaration |
|---|---|
| Question | One mechanism or end-to-end hypothesis and the discriminating observable |
| Scope | API, native process, setup/reset, verification, retirement; each separately timed |
| Fixture | Generator/seed or owned input fingerprint; entries, directories, links, logical bytes; N destinations |
| Artifact | Coordinates from §2; exact variants and enabled features |
| Residency | Maximum live cases, projected copies/entries/logical bytes, filesystem free-space reserve |
| Work | Bounded samples, hashed-byte budget, setup/verification allowance, phase deadlines |
| Retention | Compact records always; explicit owned-case retention/retirement policy |
| Failure | First budget/capacity failure stops new allocation; incomplete attempts are not results |

Use a first **tiny semantic case**, then a single bounded paired mechanism
experiment. An initial proposal can cap generated input at 1,024 entries and
16 MiB logical bytes, N1, two mirrored A/B blocks, one active case. These are
deliberately small proposed defaults, not new performance evidence or universal
limits. Increase one dimension only when the question requires it. N2 starts
with a few files to establish independent destination behavior. Deep and wide
shape tests need little payload data; dirent experiments need names and entries,
not gigabytes of contents.

Residency has two levels:

1. **First slice: stop rather than accumulate.** Add declared caps and admission
   checks to the existing runner; retain at most the permitted cases and stop
   before the next would exceed the cap. No automatic deletion or global prune.
   Emit the actual owned paths needing deliberate retirement. Even without reuse,
   this prevents an unbounded retained-case campaign.
2. **Explicit follow-up lifecycle:** after operator selection of an owned-case
   retirement policy, reuse one active slot across samples. Persist compact
   fingerprints, command logs, mismatch paths and sample status before explicit
   retirement/reset. On failure retain that one case and stop. Never sweep other
   campaigns. Reuse means fresh equivalent starting state, not letting a previous
   variant's mutations become the next input. For JW, simplify the adapter to
   create final owned repository/worker trees directly instead of generic targets
   followed by duplicate payload copies; prove identical start fingerprints and
   JJ feature/goal semantics before measuring that changed harness.

Logical bytes and copy counts bound planned work but do not predict Btrfs physical
allocation or metadata reservations. Include entry counts, free-space observations
and a conservative reserve; report that the check is advisory. CoW is not free
residency. Do not invoke filesystem maintenance as an experiment recovery step.

### Verification modes and progress

- **Small semantic mode:** exhaustive bytes, literal links, type/mode and expected
  membership; source unchanged; required reflink privacy; clean identity where
  claimed. Exercise sparse changes, type transitions, removal, refusal and N2
  independence in tiny fixtures. Keep errors with partial effects visible.
- **Warm clean mode:** one reusable source/target pair, full content hashes at
  boundaries, metadata/inode checks per sample, zero completed operations. Name
  the warm-cache scope and verification-induced warming. It does not establish
  cold performance or detect every transient/content-only modification.
- **Mutable mechanism mode:** bound the fixture so exhaustive post-state checks
  remain affordable; record reset equivalence. If later selecting sampled content
  checks on a larger shape, publish their coverage and weaker claim explicitly.
  Do not silently replace exhaustive correctness with sampling to make a run fit.

Append lightweight phase-start/phase-end records before entering setup, native
run, verification or retirement. Include sample/control, owned path, entries and
bytes processed when available, elapsed phase time and budget remaining. Flush
at phase boundaries; progress updates are throttled outside API timers. A killed
run should still identify its last started phase. Route external copy through the
existing timeout/process-group helper; check cancellation/budgets periodically
inside hashing/enumeration. Deadlines cannot promise immediate interruption of
an uninterruptible kernel filesystem wait; report the unfinished phase honestly.

For dirent work, retain **separate** tiny syscall and allocation experiments:

- Use fd-annotated, child-following traces of enumeration/stat/open/readlink
  activity, distinguishing setup and verification from the operation. Current
  `diagnostics::audit` is a path-filtered payload-I/O audit, not a complete syscall
  census. Record syscall set/filter and normalize by visited entries/directories;
  absence of a matching trace line is not proof of absence outside that scope.
- Use a benchmark-only counting allocator or allocation profiler around the
  chosen walk/list operation, after fixture and pool setup. Record alloc/realloc
  count, requested bytes and peak live bytes with thread scope. Separate this
  instrumented build from native timing; process RSS alone cannot identify dirent
  allocation cost. Reuse the exact fixture and source coordinate between variants.

### Smallest implementation and meaningful checks

1. Add phase events and a bounded-run admission policy to `execution.rs`, with
   domain estimates supplied by existing Workload implementations. Test an
   over-budget second allocation using a tiny fake workload: it must never enter
   setup, and the prior sample plus stopped status must remain readable.
2. Add explicit setup-copy timeout and bounded hash progress. Test cancellation
   with a bounded fake child; test partial-phase records with injected failure.
   Do not simulate disk exhaustion by actually filling a filesystem.
3. Add opt-in owned-slot retirement/reuse only after that policy is selected.
   On tiny N1/N2 cases, prove reset equivalence, maximum concurrent case count,
   one retained failure and untouched unrelated sentinel directories.
4. Add one small allocation instrument and use existing trace/resource support
   for the exact dirent question. Verify its counters on a known allocating
   operation and a no-work control; no wall-time CI gate or large matrix needed.

## Decisions and sequencing

Fixture and binary-selection helpers can land independently in JW; build input
binding and runner phase/budget admission can land independently in their current
owners. Cross-project measurement waits for all three evidence contracts.

Human/owner decisions needed: the CI sibling revision source; an external CoW
fixture location for environments needing both requirements; conservative default
run budgets; and whether to authorize explicit per-run owned-slot retirement.
Until the last decision, bounded stop-and-retain is sufficient. No new actual
workload campaign is proposed as acceptance for these tickets.

The ticket notes point here; parent owns index/export/closure. Related historical
evidence is linked above: streaming establishes the discovery failure mechanism,
recovery establishes the residency requirement, leaf evidence demonstrates frozen
variant separation, and clean reassessment bounds verification/cache claims.
