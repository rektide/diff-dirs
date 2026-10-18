---
type: ImplementationAccount
title: A batch owns private staging; a leaf owns its usable descriptors
description: Internal leaf-pipeline optimization preserves public validation, exact metadata, staged hardlink-safe replacement, joined partial failures, and portable fallback.
resource: /design/sync/leaf0.sol61x.md
status: implemented-verified-awaiting-parent-review
generated: { by: "model:openai/gpt-6.1-sol#xhigh", at: 2026-10-18 }
sources:
  - resource: "opencode://ses_eb2c07f5cffe2uTLIC7b1rrD99"
    title: Implemented and measured separately preserved cleanup, descriptor, staging, and internal metadata variants; verified metadata, hardlinks, joined failures, cleanup, restrictive umask and collision ownership
  - resource: /design/sync/brief0.gpt6a.md
    title: Existing staged replacement, validation, metadata, and joined application obligations
  - resource: /design/sync/implementation0.sol61x.md
    title: Original public API and shared serial/caller-pool kernel
  - resource: /design/performance/leaf0.sol61x.md
    title: Controlled same-compiler experiment and eventual selection evidence
  - resource: /design/performance/opportunities0.gpt6am.md#2-establish-the-right-leaf-operation-boundary
    title: Selected rank-two operation-boundary experiment and explicit rejection gates
  - resource: /design/performance/opportunities0.sol61x.md#a04
    title: Independent descriptor ownership, staging, and internal metadata handoff hypotheses
  - resource: /sync/src/apply.rs
    title: PreparedLeaf, lazy parent-local Staging, recursive metadata handoff, and cleanup kernel
  - resource: /sync/tests/sync.rs
    title: Public sibling-width, restrictive mode, source preservation, exact timestamps, and error regressions
  - resource: /sync/tests/permissions.rs
    title: Isolated restrictive-umask staging and final-mode tests
  - resource: "file:///home/rektide/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rustix-1.1.5/src/fs/ioctl.rs"
    title: Verified safe AsFd whole-file ioctl_ficlone API actually used on Linux
  - resource: "file:///home/rektide/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/reflink-copy-0.1.30/src/lib.rs"
    title: Preserved no-fallback error classes and portable path-based backend
---

# Internal operation ownership

Public `Applier`, `mirror`, `DirDiff`, policies, pool APIs, and progress types are
unchanged. The core remains read-only and acquires no sync/native dependency.
Public record syntax/type/ancestor validation still completes before mutation;
there is no retained observation contract across comparison and application.

## Leaf descriptors

Linux opens source and an exclusively created staging output once. Copy still
uses ordinary until-EOF `io::copy` (which already offloads on Linux). Required
reflink uses rustix's safe whole-file `ioctl_ficlone(&output,&source)` and never
falls back. Reflink-or-copy excludes NotFound, PermissionDenied, and AlreadyExists
from fallback; other clone failures reset the privately owned output and copy
using the same descriptors. Open/create failures are errors, not fallback cues.
No capability probes, source writes, clone-support cache, range clone, manual
copy length, changed stat mask, or new public backend trait is introduced.

`PreparedLeaf` retains the source/output `File`s through exact mtime setting,
descriptor readback, final source mode, and rename. Source attributes come from
the operation's existing observation under the stable-tree contract; this does
not claim concurrent snapshot protection. Other targets retain reflink-copy's
existing path backend and its private chmod/reopen procedure, then retain the
reopened output through finalization/install. Linux's direct rustix dependency
is target-specific. No new compiler pin is introduced.

## Parent-local staging

Each public record and each recursive copied directory owns a lazy arena. Its
first leaf or leaf-to-directory transition creates one target-parent-local
temporary directory. An exclusively owned child name is the operation's stable
batch index. Sibling jobs share no file descriptors or mutable file contents.
Only lazy arena initialization/path allocation takes its short mutex; cloning,
metadata and rename happen outside it. Rayon scheduling and active-worker bounds
are unchanged, with no extra admission mechanism or whole-tree operation list.

Creation excludes planned sibling names **before** creating anything. Names
matching `.diff-dirs-sync-N-...` (ASCII case-insensitively) reserve numeric prefix N; allocation selects an
unreserved prefix. Existing unrelated names are protected by tempfile's exclusive
directory creation. This avoids relying on random-name improbability while a
concurrent job may add or replace a source entry resembling the internal prefix.
The directory is owner-private on Unix, including explicit repair if umask
removed owner rights. Only the newly owned directory is repaired, never sources,
old leaves, or unrelated aliases.

An arena never moves staging to the root: each touched parent remains on its own
filesystem, preserving parent-local placement around potential nested mounts.
No mount test or universal arbitrary-mount-topology guarantee is claimed.

All sibling jobs join before cleanup; cleanup precedes parent mode restoration.
An unstopped successful batch proves all slots were renamed out, permitting one
`remove_dir` without stat/list traversal. A failed/stopped batch may retain files
or a deliberately discarded staged subtree and uses robust owned recursive
cleanup. Cleanup errors append after operation/sibling errors without clearing
completed visible installations. Cleanup/preparation counters remain private.

Existing directory↔leaf transition sequencing is retained: prepare before old
entry removal, discard incomplete preparations without installed counts, and
make no transaction promise once removal begins. A failed leaf preparation never
writes through or chmods the old destination inode or its hardlink aliases.

## Internal metadata handoff

Recursive child dispatch passes its just-observed `Metadata` into the leaf or
directory operation instead of immediately repeating the same path stat. This
is a separately preserved/measured variant after shared staging; it does not
retain a wide metadata vector, change public validation, or reuse core records.

## Verification additions

- 1/8/1000 changed siblings, recursive opaque copies, empty/unaligned files,
  Copy/Required/Fallback, ordinary/P4, internal-looking native names, links and
  directories, readonly parent restoration, exact pre-epoch nanoseconds, source
  inode/mode/mtime/bytes unchanged, independent post-sync walk, clean repeat.
- Real simultaneous local clone success and cross-filesystem failure share one
  arena: old failing leaf survives, successful sibling is counted, no residue.
- Readonly-parent cleanup failure retains installed progress and primary/secondary
  errors; failed mode-000 private subtree cleanup has no installed counts.
- Open/create errors retain NotFound/AlreadyExists without clobbering slots;
  unreadable sources remain PermissionDenied without fallback or hardlink damage.
- Open output survives mode-000 finalization and exact readback before rename;
  isolated umask 0777 proves new arena and output descriptors remain usable.
- Existing blocked-started-sibling test additionally checks readonly parent mode
  restoration and permission counts after joined failure.

## Selected landing and verification

Landed all four bounded mechanisms: direct successful-empty cleanup, the small
Linux held-FD clone/fallback adapter, lazy per-parent joined staging, and direct
recursive metadata handoff. The final controlled fresh-30k P4 median is roughly
half the same-compiler control for both Copy and Required. Shared staging earns
most of that gain. A separate staging-only control supports smaller serial
descriptor benefits (~10% fresh / ~5% dense medians), **not a P4 fresh latency
win**. Tiny/sparse and larger-file negatives remain visible in the
[performance account](/design/performance/leaf0.sol61x.md).

Rejected/deferred: full FD-relative traversal, public prepared records/catalogs,
new schedulers, direct final-path preparation, global staging, manual length
copying, capability caches, metadata masks and skipped timestamp readback. None
was implemented to make the benchmark look better. There is no new public API.

Configured Rust 1.101 nightly checks: **51 default workspace tests**, **77
all-feature/all-target workspace tests**, with zero skips and actual nonempty
btrfs clones required using `DIFF_DIRS_SYNC_REQUIRE_REFLINK=1`. Default and
all-feature/all-target clippy passed with `-D warnings`; all five all-feature
doctests and four default doctests compiled. Workspace formatting and standalone
probe rustfmt checks passed. Library documentation built with `RUSTDOCFLAGS='-D
warnings'`. Full workspace documentation also built, with Cargo's existing
CLI-bin/core-lib `diff_dirs` output-name collision warning; no crate rustdoc
warning or unrelated naming change was introduced. Core-alone default dependency
tree is still just `diff-dirs`, and all-feature core dependencies remain Rayon
only. The five maintained/evidence documents' session provenance and local file
links were checked; all 2590 API timing rows were audited for exact expected
policy, installed-file, logical-byte and symlink counts without filtering.
Mode-000 metadata finalization runs unprivileged using an already-open output;
the ordinary account correctly gets PermissionDenied when it cannot read a
mode-000 source. The public test also exercises genuinely readable mode-000
sources when privileged access permits them; no privileged execution is claimed
on this host.

Code checkpoints: `274e0ad3` control/harness, `58beac82` successful cleanup,
`60a4a03e` descriptors, `cc0fe85f` shared staging and initial regressions,
`939a6f3d` recursive handoff and restrictive/error regressions, `5319ad0e`
all-policy hardlink/no-op coverage. Original implementation and historical Rust
1.88 performance records remain untouched. `9f46584e` upgrades cleanup-failure
coverage to actual shared-arena installed work plus a failed started sibling,
rather than fabricated completed counts. No push or consumer integration.
