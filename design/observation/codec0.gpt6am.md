---
type: CodecObservation
title: Cursor Snapshot fit and bounded same-shape archive costs
description: Tiny in-memory assessment and effectful continuation specimens distinguish reflected row costs from typed archive costs and packed-name layout changes.
resource: /design/observation/codec0.gpt6am.md
status: observed-with-recommendation
generated: { by: "model:openai/gpt-6-astra#medium", at: 2026-10-20 }
sources:
  - resource: "opencode://ses_ea7cb68e0ffe6pZoU77scgXcUb"
    title: Independently inspected codec and cursor sources, constructed bounded fixtures, measured checked same-shape and packed components, and assessed fit
  - resource: /design/roadmap/brief0.gpt6a.md
    title: Independent research commission and reading boundary
  - resource: /doc/architecture/README.md
    title: Prior architecture baseline and continuation ownership
  - resource: /design/execution/model0.gpt61sxh.md
    revision: 3113dfd2
    title: Completed execution-model design and blocked prefinish discriminator
  - resource: /design/cursor/implementation0.gpt61sxh.md
    title: Delivered cursor report and prior verification scope
  - resource: /examples/cursor/src/cursor.rs
    title: Actual assessment DTOs and filesystem-free semantic validator
  - resource: /examples/cursor/src/codec.rs
    title: Actual application schema, identity, size bound and Snapshot transport
  - resource: "file:///home/rektide/src/kopacetic/crates/kopacetic-rkyv/src/lib.rs"
    title: Snapshot projection, validated envelope and owned reconstruction
  - resource: "file:///home/rektide/src/kopacetic/crates/kopacetic-rkyv/src/codec.rs"
    title: Native row DTOs, pinned profile, path copies and checksum envelope
  - resource: "file:///home/rektide/src/kopacetic/crates/kopacetic-core/src/domain/mod.rs"
    title: Entity identity and standalone fields
  - resource: "file:///home/rektide/src/facet-format-path/src/projection/serializer.rs"
    title: Byte-sequence scalar projection
  - resource: "file:///home/rektide/src/facet-format-path/src/parsing/mod.rs"
    title: Structural row validation and borrowed parser input
extensions:
  ticket: diffdir-tree-observe-codec
  reading_boundary: No withheld parent statement, full roadmap, parent transcript or current name-storage/core-synthesis drafts read or searched; no subagents
verification:
  method: One bounded offline scratch executable; whole-value equality after both checked codecs; actual Cursor validation; fixture-specific effect-state checks; resolved rkyv feature inspection
  result: Eleven paired specimens completed successfully; no filesystem traversal, timing campaign, production changes or dependency-source edits
---

# Recommendation

**Kopacetic-rkyv fits the actual owned cursor parts and the proposed effectful
component state. Keep it in the delivered demo.** Its benefit is a common Facet
projection/reconstruction transport with explicit application schema and identity,
not compact native cursor layout or zero-copy typed reconstruction.

The overhead is material but small in absolute terms for these tiny continuations:
the blocked two-frame effect fixture is **4,394 bytes versus 730 bytes** for the
same data shape with a checked native rkyv envelope. That is a reason to retain a
specialization option, not evidence that the demonstration needs another codec.
No second codec was added to the demo or production code.

For a stable large/high-frequency continuation format, direct typed rkyv is a
credible specialization. First retain the same data model and compatibility checks;
consider compact names independently. There is no measured throughput requirement
or wall-time result here from which to justify replacing the default now.

## What actually gets archived

`Snapshot::from_value` projects an agnostic Facet value into backend `StoredRow`
objects, archives those objects, then calls its own checked `from_bytes`.
`StoredRow` contains a complete `Vec<Segment>` path and a tagged scalar/container
value. Each key segment owns a string; repeated paths are not interned.
The envelope is 56 bytes: magic, format version, profile, body length and SHA-256.
The body carries application schema name/revision and all four identity fields.

`from_bytes` checks envelope/checksum and rkyv structure, exact schema, then copies
row paths and validates the logical row structure. The example separately checks
expected identity. `reconstruct::<Cursor>` accesses the archive again, copies
paths again, creates another validating PathParser and performs owned Facet
reconstruction. `Cursor::validate` then checks application invariants. These are
distinct checks; none reauthorizes physical effects or proves current tree state.

`rows()` offers borrowed string/byte **values**, but allocates a row vector, paths
and key strings. It is not an allocation-free typed `Archived<Cursor>` view.
`reconstruct` returns owned strings, vectors, maps and structs. No live runtime,
closure, handle, staging resource or borrowed dirent is captured.

**NativePath's `Vec<u8>` is a Bytes scalar.** Serializer lines 226–233 emit one
Bytes row for a byte sequence/array. The 4,096-byte specimen below has only four
total rows and one Bytes row. Ordinary NativePath adds its struct container plus
the `bytes` scalar, rather than thousands of indexed byte rows. Non-UTF8 native
coordinates survived exact whole-value equality; there was no lossy conversion.

## Procedure and isolation

Scratch: `.test-agent/cursor-codec/`, with README, nested Cargo workspace, lockfile
and isolated `target/`. It is intentionally uncommitted; this document records
the reproducible construction rather than treating ignored files as durable API.
Only the model-suffixed report is committed.

1. Copy `examples/cursor/src/{lib,cursor,codec}.rs` into the scratch crate. Add
   `rkyv::{Archive,Serialize,Deserialize}` derives to NativePath, Counts,
   Descriptor, Tip, Phase, Frame, Block, FinalState and Cursor. Change no fields,
   types or validation logic. Both codecs see this identical Cursor type.
2. Use local kopacetic-core with default features off and local kopacetic-rkyv;
   Facet `=0.50.0-rc.5`, rkyv `=0.8.18`, defaults off, exactly
   `std,bytecheck,little_endian,unaligned,pointer_width_32`. `cargo tree -e features
   -i rkyv` confirmed only these plus implied `alloc`. No conflicting aligned,
   endian or pointer-width features were unified.
3. Construct DTOs directly in memory. Never call start/advance, inspect a tree,
   create real fixtures or access JJ. Call actual `Cursor::validate` before
   encoding and after both reconstructions. Exact `PartialEq` compares every
   descriptor, path, counter, component and optional state after decoding.
4. Snapshot uses identity `standalone("DirectoryCursor", "example")` and schema
   `("diff-dirs.cursor-example", 1)`. Native archives an owned tuple
   `(String,u32,(Option<String>,String,Option<String>,String),T)` with identical
   metadata values. Its separate 56-byte `TYPEDCUR` header checks magic, version 1,
   profile 1, body length and SHA-256. Encode also checks the emitted body with
   `rkyv::access`; decode uses checked `rkyv::from_bytes`, verifies exact metadata,
   and applies the same semantic validator. Both return owned T.
5. Reset a scratch System allocator wrapper around each operation, counting
   alloc/alloc_zeroed/realloc calls and requested bytes. Exclude DTO construction,
   envelope DTO preparation, input buffer copying, row counting, equality checks
   and printing. No loops over timings or profiler. Each specimen runs once.

Run from the repository root:

```sh
cargo run --offline --manifest-path .test-agent/cursor-codec/Cargo.toml > .test-agent/cursor-codec/results.txt
cargo tree --offline --manifest-path .test-agent/cursor-codec/Cargo.toml -e features -i rkyv
```

Dependency checkout HEADs were kopacetic `56e9bba26c8728c14aef16b6d20bb0d7f0720ff1`
and facet-format-path `e5727be3e64883c13e1ecbd473f920a0d258dc81`. Sources were read
in place, not reset; unrelated work was untouched. Builds used Rust
1.101.0-nightly and the local resolved dependencies.

### Native-comparison qualifications

This is not raw `to_bytes(Cursor)` against a validated Snapshot file. Both byte
counts include metadata and checksum envelopes; both decode paths use bytecheck,
metadata equality and application checks. Native has no logical-row validator
because it has no logical rows; typed archive validation replaces that layer.
Native metadata equality is checked after owned reconstruction in this scratch,
whereas Snapshot schema/identity are checked before payload reconstruction. A
production native reader should inspect validated archived metadata first.
Neither checksum is authentication. Scratch has no production file-read size
limit or adversarial allocation budget; fixtures are bounded by construction.

The native tuple is equivalent metadata, not kopacetic's binary Document layout
or format. A production typed codec would own an explicitly versioned wire DTO.
The same schema string is reused across isolated statically typed scratch cases
to keep metadata size constant; this is **not** a proposed schema registry mapping
all these incompatible types to revision 1.

## Specimens and deterministic byte results

All assessment roots are `/fixture/source` and `/fixture/target`; platform `unix`;
generator `directory-summary/v1` with parameter `label=codec-fixture`.
Counts default to zero unless stated. Root Tip has three empty coordinates.

| Specimen | Snapshot bytes | Typed bytes | Rows | Bytes rows |
| --- | ---: | ---: | ---: | ---: |
| Empty pending assessment root | 1,989 | 488 | 33 | 5 |
| Blocked multi-frame assessment | 3,710 | 631 | 60 | 12 |
| Terminal assessment | 1,347 | 412 | 22 | 2 |
| Blocked effectful prefinish | 4,394 | 730 | 74 | 14 |
| Empty path list | 208 | 149 | 2 | 0 |
| Packed empty path list | 242 | 157 | 3 | 1 |
| Four paths | 692 | 285 | 10 | 4 |
| Packed four paths | 518 | 277 | 7 | 1 |
| 128 paths | 15,696 | 4,501 | 258 | 128 |
| Packed 128 paths | 9,074 | 3,997 | 131 | 1 |
| One 4,096-byte native byte vector | 4,399 | 4,253 | 4 | 1 |

Empty pending has one Pending frame, no children, no block/final and all counters
zero. Blocked assessment has an Assessed root whose only child is `d\xff`, index 1,
and an active Pending child with all three coordinates exactly those two bytes.
Attempted total is 2 and block message is `injected assessment failure`. Terminal
has no frames, attempted/completed 1, skipped 0, and a zero-count complete final.

The effectful specimen explicitly lowers the execution model's **blocked before
directory finish, after two accepted leaves** (§Exact paused states and §Fail
before finish). It is a schematic concrete Saved type, not an executable new
driver or an assertion that Cursor already implements effects:

```text
Saved { platform, source_root, target_root, output_root, program,
        stack: Vec<EffectFrame>, pending: Option<Pending>, blocked,
        final_value: Option<FinalState>, next_ticket, manifest }
EffectFrame { node: Tip, phase, remaining: Vec<Tip>, local: Option<Counts>,
              children: u64, finish: bool, incomplete: bool }
Pending { ticket, attempt, phase, logical, recipe, ordinal, leaves }
Manifest { label: String, next_ordinal: u64 }
Phase = Before | Assess | Children | Finish | Leaf
```

Root is Children with remaining `[d\xff]`, children 0, finish true; the second
frame is Finish at `d\xff`, no remaining children, children 2, finish true.
Both have Some(zero Counts) local, incomplete false. Pending has ticket 7,
attempt 2, Finish, logical `d\xff`, recipe `manifest/v1`, ordinal 1, leaves 2.
Block is at the same path with `finish failed after visible output`; next ticket
8, final None. Output root is `/fixture/output`. Program kind `manifest/v1`
has `label=effect-label`; Manifest retains that label and next ordinal 1.
Validation checks frame adjacency, pending phase/path/aggregate, ticket ordering
and descriptor/component agreement. It is fixture-specific, not the complete
hostile-data validator the future execution driver requires. Full equality also
checks fields not independently constrained by that validator.

This demonstrates that mutable component state and a parameterized finish recipe
are ordinary encodable data. It does not test effect execution, retry correctness
or component rebinding; those are established/designated in the execution model.

### Separate compact-layout experiment

Baseline `Paths { names: Vec<NativePath> }` contains names
`shared-directory/child-000` through `...127` (26 bytes each), or the indicated
prefix length of that list. Packed is **only**
`{ bytes: Vec<u8>, ends: Vec<u32> }`: concatenation plus cumulative exclusive ends.
No prefix dictionary, deduplication, compression or altered encoding of name bytes.
Before measuring, each original name is checked against its packed slice. Packed
also roundtrips with exact whole-value equality through both codecs. The 4,096-byte
case is a scalar stress specimen of repeated `0xff`, not a realizable filesystem
name or an accepted Cursor path fixture. All payloads are far below 1 MiB.

This separation matters: same-shape native versus Snapshot establishes codec cost;
Paths versus Packed establishes layout cost. At 128 names, packing saves 6,622
Snapshot bytes (42%) but only 504 typed bytes (11%). Under Facet, `Vec<u32>` ends
still has one scalar row per offset; the bytes pool becomes one Bytes row. The
empty packed layout is larger. Do not extrapolate these component percentages to
whole cursors or claim this experiment removes the three-coordinate duplication.

## Allocation observations

Numbers are allocation/reallocation **requests**, not retained allocations or
peak RSS. Requested bytes include resized capacities, not just useful payload.
One-shot debug builds can include lazy Facet caches; the first/next type ordering
is not a controlled steady-state benchmark. Treat these as concrete implementation
cost observations, not portable per-operation guarantees.

| Specimen | Snapshot encode calls / requested B | Typed encode | Snapshot decode | Typed decode |
| --- | ---: | ---: | ---: | ---: |
| Empty pending | 357 / 58,556 | 6 / 1,480 | 351 / 151,546 | 13 / 836 |
| Blocked assessment | 685 / 91,130 | 7 / 2,647 | 547 / 100,133 | 24 / 1,097 |
| Terminal | 224 / 34,708 | 6 / 1,404 | 174 / 73,718 | 12 / 692 |
| Effectful prefinish | 825 / 127,371 | 7 / 2,746 | 723 / 192,460 | 24 / 1,116 |
| 128 paths | 2,224 / 424,392 | 10 / 20,853 | 1,837 / 256,684 | 132 / 6,446 |
| Packed 128 | 559 / 128,242 | 4 / 8,189 | 560 / 141,832 | 5 / 3,886 |

The source mechanism explains the direction: projection clones complete paths and
payloads, encode self-validation reconstructs row paths, then decode validation
and reconstruction repeat path materialization and parser work. Typed decode
allocates owned application containers directly. The experiment does not isolate
every allocator site or measure a borrowed typed access-only path.

## Path/state layout and compatibility advice

- Keep explicit byte-preserving native coordinates and a platform contract.
  Avoid opaque PathBuf reflection or UTF-8 conversions. Absolute root spelling
  affects file size; archive format portability does not imply native-path portability.
- Current Tip repeats coincident logical/source/target bytes, and the assessment
  frame retains the full child list plus index, including the completed prefix.
  Those are layout opportunities independent of codec. The execution model's
  remaining-child suffix can avoid retaining closed siblings without history.
- Use a byte pool plus checked ranges when name-container count or retained memory
  actually matters. Preserve absent-side distinctions and logical/physical mapping;
  do not collapse coordinates based on today's coincidence assumption. A frame-local
  pool is a plausible first optimization; global interning introduces more invariants.
  Bounds, monotonic offsets, reconstruction identity and native normalization need
  validation. This probe uses concatenated complete names, not parent-relative paths.
- Save descriptor/version/parameters **and mutable component state**, including
  ordinal, pending ticket/phase/attempt and recipe inputs. Do not save previously
  generated bodies merely to reconstruct deterministic finish state. Rebind executable
  components and resources outside the codec after checking compatibility.
- Application schema is an assertion, not a Rust type fingerprint. Bump revision
  for changed interpretation and explicitly migrate if required. Kopacetic has a
  separate backend/profile contract; direct rkyv would additionally couple the
  application wire DTO's field/variant layout to that contract. Neither option
  supplies automatic migrations or proof that a known descriptor's code is unchanged.
- Do not hide a packed whole-cursor binary blob inside Facet just to improve the
  byte table: that would introduce an inner codec and discard most row inspection
  value. Specialized typed components or one explicit typed backend are clearer
  if specialization is later warranted.

Specialize when measured representative frontiers, save frequency, archive size or
allocation pressure exceed a real consumer budget; or when validated borrowed
typed reads are valuable enough to justify a stable wire DTO and archived-state
API. Here even the larger measured effect state is only a few KiB. Keep the
portable semantic Saved boundary so that this choice need not redesign traversal.

Unmeasured: large/deep frontiers, sustained throughput, wall-time, peak/live memory,
release-mode cache behavior, lazy archived typed state mutation, hostile resource
exhaustion, 32-bit hosts/Windows, migrations and generated-body-heavy components.
No filesystem benchmark, real archive traversal or production test rerun was used.

## Connections

The [assessment implementation report](/design/cursor/implementation0.gpt61sxh.md#codec-invariants-and-small-adaptations)
already identified generic rows and owned reconstruction; these measurements
quantify that boundary without replacing its verified codec. The
[execution model](/design/execution/model0.gpt61sxh.md#a-practical-component-construction-boundary)
supplies the new pending/finish/component data requirement; this probe establishes
representability, not execution semantics. The
[architecture's continuation boundary](/doc/architecture/README.md#component-built-continuation-and-pause)
remains the owner of runtime-versus-saved separation. Index and reciprocal-link
integration are intentionally left to the commissioning parent.
