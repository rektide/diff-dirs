---
type: Example
title: Bounded directory assessment with a serialized continuation
description: A standalone Unix Rust library and CLI that assesses real trees, transfers DFS state through kopacetic-rkyv, and emits generated logical summary files.
resource: /examples/cursor/README.md
status: implemented
generated: { by: "model:openai/gpt-6.1-sol#xhigh", at: 2026-10-20 }
updated: { by: "model:openai/gpt-6.1-sol#xhigh", at: 2026-10-22 }
sources:
  - resource: "opencode://ses_e9d314c29ffeboK450qrHM3qhF"
    title: Updated standalone package and facade coordinates while preserving local codec dependencies and persisted schema identity
  - resource: "opencode://ses_ea8162bcbffe20eNrM60NJtgVS"
    title: Implemented and verified the isolated cursor library, CLI, generated summary handler, process continuation, retry and subtree skip
  - resource: /design/cursor/cursor0.gpt6am.md
    title: Reviewed design implemented by this example
  - resource: /tree/inspect/src/visitor.rs
    title: Public immediate-directory inspector and pure visitor contract
  - resource: "file:///home/rektide/src/kopacetic/crates/kopacetic-rkyv/src/lib.rs"
    title: Actual Snapshot codec, validation, identity and schema interfaces
verification:
  method: Example-focused nextest, clippy, formatting, default core dependency tree, and a tiny two-process demonstration
  result: 12 tests passed; clippy and formatting passed; core dependency isolation retained
---

# Bounded assessment cursor

An **assessment-only**, JJ-independent example. Each successful directory
assessment uses public `diff_dirs_tree_api::inspect_directory`. A sequential driver owns
its DFS frames, counts, child indices and continuation; it never restarts a walk
at the root to skip old results. `NoVisitor` remains pure.

The CLI writes a binary cursor file and JSON Lines describing attempted
assessments, postorder completions, skips and an invocation summary. Every
completion manufactures a logical `summary.txt` with actual bytes using the
persisted `directory-summary/v1` descriptor and its `label` parameter. **Generated
files are emitted, not installed into either input tree.** No sync actions run.

## Prerequisites

- Unix native paths; verified on Linux with Rust 1.101 nightly.
- Sibling checkouts `../../../kopacetic` and `../../../facet-format-path` relative
  to this directory. These are local path dependencies, not vendored code.
- Package `diff-dirs-examples-cursor` is its own nested workspace with a committed
  lockfile and isolated `target/`; its executable remains `cursor`. Its facade
  dependency is `../../tree/api` (`diff-dirs-tree-api`), and codec dependencies
  stay outside the main workspace. `kopacetic-core`'s default Kubernetes feature
  is disabled here.

From the diff-dirs checkout, make a tiny fixture outside the input trees:

```sh
demo=$(mktemp -d /tmp/opencode/cursor-demo.XXXXXX)
mkdir -p "$demo/source/branch" "$demo/target/branch"
printf hello > "$demo/source/new.txt"
printf old > "$demo/target/old.txt"

cargo run --manifest-path examples/cursor/Cargo.toml -- start \
  --source "$demo/source" --target "$demo/target" --units 1 \
  --label tiny-fixture --cursor-out "$demo/state.rkyv"

cargo run --manifest-path examples/cursor/Cargo.toml -- resume \
  --cursor-in "$demo/state.rkyv" --units 1 --cursor-out "$demo/state.rkyv"
```

The first process assesses the root and pauses with `branch` pending; the second
assesses `branch`, emits its completion followed by the root's, and reports
`done: true`. The label survives without being supplied again. In the verified
tiny fixture, the paused cursor was **3,618 bytes** (including that run's absolute
paths); this is an observation, not a size/performance claim.

Resume accepts stored roots only, not replacement roots. The binary input is
fully read before writing output, so the same filename is supported. Keep cursor
output outside the compared trees so transport does not alter their contents.

## What is bounded, counted and saved?

- `--units N` must be positive. One unit is **one attempted immediate-directory
  assessment**, including validating its scheduled present sides as real
  directories. Failed attempts count and stop that invocation. Start's root
  binding checks are command validation, not assessment units.
- Empty and clean directories cost units too. One-sided interiors are visited;
  at a directory/leaf collision only the directory side is descended. This
  assessment policy does not split or apply the core's coupled replacement.
- Completion/push/pop transitions cost no units. The last assessment can finish
  several ancestors. N does **not** bound entries, syscalls, completion events,
  elapsed time or memory; wide immediate listings remain wide. Events are held
  in memory for the current invocation only.
- `source_only`, `target_only`, `changed`, `equal` count observed immediate
  supported slots, then accumulate through descendants. A matched directory slot
  is `equal` structurally even if its descendants differ. Leaves use core's
  size/mtime or literal-symlink comparison, not content hashing.
- `attempted_total` includes retries/failures. `completed_total` counts directory
  completions and generated logical files, **not** copied files or effects.
  `skipped_total` counts scheduled directory subtrees deliberately skipped.
- Completions are native-name DFS **postorder** with monotonically increasing
  `sequence`. `done` means no frames remain; consult `final_state.incomplete` for
  skipped coverage. It does not mean the trees match.
- Cursor state includes active frames, accumulated counts, descriptor parameters,
  counters, a possible blocked tip, and final aggregate state. It does **not**
  retain previous output or generated file bodies. Assessed ancestors are not
  relisted; previously selected pending names are not a live discovery feed.

All JSON coordinates are lossless Unix byte arrays: `[]` denotes the logical
root, `[98,114,97,110,99,104]` denotes `branch`. Generated `contents` is also a byte
array. These arrays, not lossy display strings, carry identity. Roots are absolute
Unix bindings, so transferring an archive does not relocate the filesystem.

## Failure, retry and skip

To demonstrate a blocked child deterministically, add `--fail-at branch` to the
**start** command above. Injection is invocation-local, outside Visitor, and not
persisted. That command consumes two attempts if its budget allows reaching the
child, saves a blocked cursor and exits 1. Use `--units 2` for that demonstration.

- Ordinary `resume` retries the same pending coordinate against actual state.
  Omitting `--fail-at` removes injection.
- `resume --skip-blocked` requires an existing block. It emits a skip, removes
  **that scheduled directory subtree**, marks all ancestors incomplete, and
  continues with the full N-assessment budget. It cannot skip just an offending
  immediate file within a failed batch. Skipping a blocked root ends incomplete.
- Missing or changed-to-leaf scheduled directories and unsupported observed
  special files also block without adding a partial batch's counts.
- Exit 0: invocation succeeded (possibly paused or incomplete); 1: assessment
  blocked and cursor saved; 2: invalid arguments/cursor, initial binding, codec,
  output or another command error. Command errors do not promise a new cursor.

## Codec and boundaries

`codec.rs` calls **actual `kopacetic_rkyv::Snapshot`**, not a Serde persistence
substitute. Serde is used only for JSON output and textual generated content.
Application schema is `diff-dirs.cursor-example`, revision 1; expected identity
is `DirectoryCursor/example`. Incompatible DTO changes require a schema revision.
The path-derived Cargo package rename does not change these persisted identities.
The descriptor accepts only `directory-summary/v1` and exactly its `label` key.

Backend validation checks envelope/checksum, checked rkyv access, exact schema
and logical rows. The example additionally checks identity, Unix/path encoding,
normal relative coordinates, disjoint absolute roots, parent/child consistency,
indices, phases, descriptor and basic progress invariants. Counts use checked
arithmetic. This is corruption/error detection, **not authentication or a sandbox**.
Input reads and output archives are limited to 8 MiB; that byte limit does not
bound all projection/reconstruction allocations, nesting work, or directory I/O.
Kopacetic reflects typed path rows and reconstructs an owned cursor; it is not a
native zero-copy `Archived<Cursor>`, and no comparative performance is claimed.

Cursor publication serializes first and replaces through a sibling temporary
file. It happens before stdout delivery. There is no transaction tying the two,
effects journal, exactly-once output receipt, rollback, recovery guarantee, or
power-loss guarantee. Losing stdout after publication can lose completion output;
reusing an older cursor can repeat it. A saved cursor is a continuation position,
not evidence that external effects happened. Terminal resume performs no input
tree filesystem work. Callers own donor validity, stable routing ancestors,
concurrent mutation and locking; symlink leaves are not followed.

## Checks and source map

```sh
cargo nextest run --manifest-path examples/cursor/Cargo.toml
cargo clippy --manifest-path examples/cursor/Cargo.toml --all-targets -- -D warnings
cargo fmt --manifest-path examples/cursor/Cargo.toml --check
cargo tree -p diff-dirs-tree-api --edges normal
```

- `src/cursor.rs`: DTO, sequential stepping, pure reduction and summary handler.
- `src/codec.rs`: Snapshot transport, byte limits and complete-file publication.
- `src/main.rs`: Clap commands, JSONL and exit codes.
- `tests/continuation.rs`: state/codec, postorder, real errors and skip coverage.
- `tests/process.rs`: successive executable invocations and transport failures.
- [Reviewed design](../../design/cursor/cursor0.gpt6am.md): rationale and deferred
  extensible action/mapping directions; this example does not implement that
  broader framework.
