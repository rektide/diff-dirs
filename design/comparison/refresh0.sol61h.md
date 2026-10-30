---
type: ImplementationAccount
title: Seeded stat refresh and optional installation postcheck
description: Commissioned heuristic retention, exact manifest completion, general installer postcheck and the native consumer handoff decision.
resource: /design/comparison/refresh0.sol61h.md
status: implementation-in-progress
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
