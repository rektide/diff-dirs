---
type: ImplementationAccount
title: Generic paired directory comparison — initial implementation
description: The std-only DFS, lossless records, scan coverage, and the distinction between immediate entry counts and aggregate directory divergence.
resource: /design/implementation/implementation0.sol61x.md
status: implemented
generated: { by: "model:openai/gpt-6.1-sol#xhigh", at: 2026-10-18 }
sources:
  - resource: opencode://ses_eb3948bedffe0T37Xc2wI2S5Jg
    title: Built and verified generic comparison core and lossless thin CLI; ratified direction, literal symlink, coverage, and consumer mutation contracts under parent direction
  - resource: /README.md
    title: Specification, deliberately amended alongside implementation
  - resource: /design/fit/fit0.glm53m.md
    title: Initial fit research, contradictions, and sequential DFS recommendation
  - resource: /src/walk.rs
    title: Native metadata capture, sorted merge, and explicit DFS state machine
  - resource: /src/model.rs
    title: Public record and summary types
  - resource: /tests/walk.rs
    title: Contract tests, including test-side immediate-child application
  - resource: /cli/src/render.rs
    title: Text escaping and string-or-tagged native JSON representation
  - resource: /cli/tests/cli.rs
    title: Exit codes, JSONL, failed summaries, non-UTF-8 output, and argument handling
---

# Generic paired directory comparison

## Situation and scope

The original specification joined a generic tree comparison with jj-waltz's
payload use case, but reversed added/removed labels between its model and its
application examples. Parent direction settled those contradictions before
implementation. The API has no jj concepts, runtime, ignore engine, concurrency,
content reads, or mutation. Actual copy/sync belongs in a separately authorized
component; it is not implemented in this core checkpoint.

## State machine and memory

[`walk`](/src/walk.rs) validates existing real roots. Contents remain lazy.
Each examined directory fully captures and sorts both immediate listings, then
merge-joins native names. Regular files carry size and `SystemTime`; symlinks
carry `OsString` readlink targets. Literal native target spelling matters:
`./file` differs from `file`, and `a//b` differs from `a/b`. Path component
normalization is never used for those comparisons.

After the merge, only the record being returned and sorted paired child
directory names remain. Frames retain pending sibling directories along the
active descent. No directory handle or full tree of records is retained.
Transient listings cost the current directory width; pending names cost the
sum of active-frame widths, not just tree depth. Summary vectors can grow with
divergent directories and explicit prunes.

The caller may apply the classified immediate children after a record: all of
that record's metadata was already captured. Both-sided child directories are
opened later, so replacing them requires pruning or stopping first. The test
applier deletes removed/conflicting leaves before copying, restores file
mtimes, and proves the next full walk is clean. It is intentionally test code,
not a reconciliation implementation exported by the library.

## Summary distinction

Immediate entry counts do not double-count aggregate directory divergence.
`EntryCounts.dirs` therefore contains only added and removed, with one-sided
subtrees counted as one. `Summary.changed_dirs` separately contains both-sided
directories with observed subtree divergence, ancestors and root included,
in post-order. `topmost_divergent` selects directly divergent directories
without directly divergent ancestors; it does not reduce every result to root.

`ScanState` records iterator termination. `Incomplete` lasts until actual
normal `None`; a terminal `Err` sets `Failed` permanently. Explicit prune paths
are separate from normal exhaustion. `is_complete()` requires normal exhaustion
without pruning, and `is_clean()` additionally requires no reported differences.
`into_summary()` only unwinds already examined frames; it performs no IO and
cannot turn an unfinished scan into a complete one.

## Verification at the core checkpoint

On Linux, all 22 integration tests passed under Rust 1.88.0, as did the compiled
doctest. The tests cover determinism/byte order, directions/counts, all pairs of
file/directory/symlink kinds, nested divergence under unchanged directory mtime,
same-size-and-mtime trust limits, nanoseconds and pre-epoch mtimes, literal and
cyclic/dangling links, non-UTF-8 names and targets, special-file rejection,
root rejection, prune/early-break/error states, and test-side apply-in-loop
steady-state. Mode-only divergence is deliberately demonstrated as invisible.

Cross-platform code uses `std`; Unix-specific tests are gated. Windows and
macOS behavior is not claimed to have been exercised. Nanosecond/pre-epoch
fixtures establish support on this test filesystem, not every filesystem.
The CLI follow-on checkpoint adds eight passing process-level tests (30 total)
and passes full-workspace clippy with `-D warnings`, formatting, and Rust 1.88.0
tests/doctests. Only the CLI depends on `serde_json`; no dependency or runtime
was introduced into the core. Rendering is streaming, always followed by a
coverage-bearing summary when a walk was constructed; it does not gather the
whole record stream. IO errors are exit 2 even after partial output.

JSON uses normal strings where valid, native byte tags on Unix, and native
UTF-16 unit tags on Windows otherwise. Text escapes names rather than rendering
newlines or silently replacing invalid native encodings. Windows rendering code
exists behind `cfg(windows)` but has not been run on Windows. Performance is a
separate follow-on checkpoint.

## Relationships and limits

- The [specification](/README.md) is the maintained contract; this account
  explains why its formerly ambiguous summary model now has two separate roles.
- The [fit assessment](/design/fit/fit0.glm53m.md) motivated sorted paired DFS
  and generic root validation. Its integration seam is research, not evidence
  of an end-to-end speedup from this library alone.
- No mode comparison, security sandbox, stable snapshot, absent-root dispatch,
  automatic cleanup, payload validation within opaque one-sided subtrees, or
  built-in applier is implied by the completed core.
