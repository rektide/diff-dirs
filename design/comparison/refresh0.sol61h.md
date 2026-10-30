---
type: ImplementationAccount
title: Seeded stat refresh and optional installation postcheck
description: Commissioned heuristic retention, exact manifest completion, general installer postcheck and the native consumer handoff decision.
resource: /design/comparison/refresh0.sol61h.md
status: provider-implemented-native-checkpoint
generated: { by: "model:openai/gpt-6.1-sol#high", at: "2026-10-30" }
sources:
  - resource: "opencode://ses_e744de129ffeJFNaoEhNajEXMV"
    title: Implements the commissioned seeded refresh provider and primary consumer integration
  - resource: "opencode://ses_e74f4db73ffebUUrvE3MbpCHWY"
    title: Human selects default refresh, size and mtime heuristic, optional mode and no default postcheck
  - resource: /doc/architecture/README.md
    title: Inspect owns physical acquisition and comparison; transfer owns effects and manifest coverage
  - resource: /design/comparison/reflink0.gpt6a.md
    title: Separates retained satisfaction from installation; its storage-proof and private-inode proposal is not this commission
  - resource: "file:///home/rektide/src/jj-waltz/design/coordinated-refill/policy-scale0.gpt6a.md"
    title: Existing native routes, unconditional tracked installation and incremental ordinary comparison
  - resource: /transfer/sync/src/listed/selection.rs
    title: Exact owned namespace authority and coupled replacement rules
  - resource: /transfer/sync/src/kernel.rs
    title: Shared staged copy/reflink installer and current mandatory mtime read-back
---

# Selected behavior

The user accepts seeded **size + exact mtime equality as a heuristic**. Refresh
retains compatible owned regular files under the selected checks and literal-equal
symlinks. No byte hashing, FIEMAP, hardlink/privacy gate or mandatory fresh inode
is introduced for retained matches. Changed/new leaves use the existing staged
installer; obsolete ownership and kind transitions keep their existing rules.

JW will default to `--renewal-route=refresh`. Explicit clear-then-populate and
replace-authorized retain installation-only tracked behavior. `--refresh-check`
selects the entire set from `mtime,size,mode`, default `mtime,size`; malformed,
empty and duplicate lists are rejected. `mode` means Unix permission and special
bits (`0o7777`), not file-type bits or just executable state. The selected mask
will also govern ordinary reconciliation on all three routes, replacing its
fixed StatKey with the same typed comparator. Existing defaults remain identical
there. Symlinks always compare literal targets, independently of this mask.

`--postcheck=mtime` applies to regular installations on every route and both
cohorts. No postchecks is the default. Setting mtime and permissions remains
unconditional; only read-back verification becomes conditional. The provider
mechanism belongs to effect binding, not the refresh comparator.

## Provider shape

Inspection gains a configurable stat comparator with a parameterized semantic
identity, reused by physical-union and declared inspection. Declared selection
can opt into compatible-leaf comparison; old declarations remain NotAssessed.
The common validator admits a desired KeepLeaf only with assessed compatible
equality. Sync still binds exact manifests and recomputes lowering against live
prepared facts before effects. Equality never grants ownership.

`RefreshOwned(checks)` is distinct from `RenewOwned`. Coverage adds `retained`;
completion requires disjoint installed + retained desired obligations, complete
obsolete retirement, and successful scope/staging cleanup. Retention increments
neither clone nor copy effects. Old intents still require every desired leaf
installed. Public API changes receive a coordinated provider minor version and
exact receiving pins/locks. JW wire coverage must evolve explicitly, not silently
reinterpret schema-one installed counts.

## Native checkpoint

JW currently adopts S tree_state and requires exact native D conformance. A
retained size/mtime-equal file with different executable bits would fail the
final native snapshot, especially for D=S where restore is skipped. Optional
mode cannot quietly become a mandatory comparator or reinstall condition.

Recommended native handoff: normalize tracked executable state separately when
needed, preserving retained inode identity, with explicit native effects. Full
mode checking remains optional and causes ordinary provider renewal when selected.
This consequential decision is raised to the parent before consumer implementation;
provider implementation is independent. Source attributes/expectations still
verify actual S kind and executable state. Native adoption remains consumer-owned.

## Verification scope

Use tiny existing Btrfs/native fixtures. Check equality mask identity, retained
inode and zero installation effects, changed metadata, links/transitions/obsolete
and unowned cases, complete/failure-prefix accounting, actual optional installer
read-back, and native D=S/D!=S plus legacy routes. No imported workload,
deployment, representative latency claim or speculative proof policy is in scope.
Exact checks and resulting commits will be recorded as implementation completes.

## Provider checkpoint and receipts

Provider products are now **0.8.0**, with exact internal and example pins/locks.
Delivered: `StatChecks`/`StatComparator`, declared compatible-leaf assessment,
`RefreshOwned(checks)`, separate `ManifestCoverage.retained`, and optional
`Postcheck::{None,Mtime}` on EffectBinding/Applier. Regular retained matches have
zero installation effects; executable normalization is **not implemented** in
this provider checkpoint. That remains the native consumer decision above.

SyncParts and reconcile-example transport deliberately advance to **version4**,
storing and rebinding a postcheck per lane. Old versions refuse. Comparison-only
parts keep their existing version. Example runtimes reconstruct the saved setting;
it cannot silently change across resume. JW is not yet migrated at this checkpoint;
its pins, struct initializers and public wire parser still need coordinated adoption.

Checks executed in this checkout (Linux/nightly, tiny local Btrfs fixtures):

- `cargo check --workspace --all-targets`: passed after fixing the new selector's
  lifetime shadow, slot enumeration and test calls for the added postcheck argument.
- `cargo nextest run -p diff-dirs-transfer-sync --test listed`: **21 passed**,
  run `cdcb6c17-e633-4b8b-b8bf-7e35ec4ff8f9`.
- `cargo nextest run --workspace --all-features`: **306 passed**, run
  `f6f86998-43bc-4efc-84bd-de320455a18d`. Nextest warned about the perf-runner's
  missing build-script output file; all tests executed and passed. No campaign ran.
- After adding the ordinary-path/matched-postcheck binding discriminator,
  `cargo nextest run -p diff-dirs-transfer-sync --test listed --no-default-features`:
  **22 passed**, run `6a497380-fa7c-4f4e-9758-42a7956341d7`.
  The same listed selection with `--all-features` also passed **22**, run
  `65850f48-5517-4ed8-bc29-5841eb311ecb`.
- `cargo nextest run --manifest-path examples/reconcile/Cargo.toml`: **20 passed**,
  run `14a9dd9f-f9c4-4190-954a-554834a09721`, including actual fresh-process resume.
- `cargo check --manifest-path examples/{walk,cursor}/Cargo.toml --all-targets`
  (each separately): passed and updated their receiving locks.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`;
  `cargo clippy -p diff-dirs-tree-inspect -p diff-dirs-transfer-sync --all-targets
  --no-default-features -- -D warnings`; and reconcile-example all-target Clippy:
  passed. The first rustfmt check showed new-code formatting; `cargo fmt --all`
  applied it and the subsequent check passed. Example formatting also applied.
- `cargo test --workspace --doc --all-features`: **16 doctests passed**.
- Explicit `rkt check --okf` on this account and revised sync, inspect, consumer
  and reconcile-example guides: passed (five documents; 52 links, 78 sources).

### Actual omitted read-back syscall

The named integration probes each execute one **Copy** and one actual
**ReflinkRequired** listed installation. Their only policy difference is None
versus Mtime. Both verify resulting mtime externally after publication. The
observed required clone was `ioctl(..., BTRFS_IOC_CLONE or FICLONE, ...) = 0`.

Build: `cargo test -p diff-dirs-transfer-sync --test listed --no-run`.
Binary: `target/debug/build/diff-dirs-transfer-sync/df5abf17074a5612/out/listed-df5abf17074a5612`,
SHA-256 `5b182f5a9153323d9405c87c84bff2aa2958823686748b324ee604d7517da44b`.
Run the binary with `--exact install_postcheck_none_probe` and
`--exact install_postcheck_mtime_probe` under:

```sh
strace -f -yy -o TRACE -e trace=statx,newfstatat,fstat,utimensat,fchmod,ioctl BINARY --exact TEST
```

Local traces are `.test-agent/refresh/postcheck-{none,mtime}.trace`. For **both**
installed staging descriptors, None performs `utimensat(fd, NULL, ...)` directly
followed by `fchmod(fd, ...)`; Mtime inserts exactly one
`statx(fd, "", AT_STATX_SYNC_AS_STAT|AT_EMPTY_PATH, STATX_ALL, ...)` between them.
Thus two read-back metadata calls are actually omitted in the None probe, one
per installation. Copy's earlier empty-output stat remains (from `io::copy`),
as do guard/route checks and the test's final external metadata assertions.
This is an actual installer syscall witness, not a whole-command syscall budget
or a universal latency claim.

## Parent decision needed before native default cutover

Keep optional full mode checking exactly optional. The recommended resolution is
to normalize only **tracked executable state** at the native handoff when it
differs, preserving the retained inode and other mode bits; report those writes
separately from installs. A native implementation must settle how to use existing
prepared metadata without an unnecessary second stat sweep. Provider's current
coverage return is aggregate, not a borrowed per-leaf handoff list.

Alternatives are materially different contracts: refuse a retained executable
mismatch at the native boundary, or require its reinstallation despite omitted
mode checking. Neither should be silently selected while advertising the user's
optional mode heuristic. No JW default, wire-schema change or native chmod loop
has been landed pending that checkpoint. All unrelated starting files remain
untouched, including `.beads/issues.jsonl` and the two JW inquiry documents.
