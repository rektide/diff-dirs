---
type: ResearchEvidence
title: Optimization ground truth — std directory handles, copying, and current syscall costs
description: Baseline source and syscall investigation separating userspace paths from fd-relative metadata, kernel-assisted ordinary copying, and sync staging; preserves historical compiler provenance and records the user's unpinned-toolchain override.
resource: /design/performance/inspection0.sol61x.md
status: research-only
generated: { by: "model:openai/gpt-6.1-sol#xhigh", at: 2026-10-18 }
sources:
  - resource: "opencode://ses_eb3033c79ffeRIkn14uSNhs7O6"
    title: After building sync, investigated optimization opportunities without production edits; verified historical and configured-nightly std source/syscalls, then incorporated the user's unpinned-toolchain correction
  - resource: /src/pair.rs
    title: Eager child paths, retained std DirEntry, combined listing sort, metadata, and literal readlink
  - resource: /src/frontier.rs
    title: Directory-job ownership, candidate search, J/K admission, ordered commitment, and joined epochs
  - resource: /sync/src/apply.rs
    title: Duplicate inspection, per-leaf staging, clone reopen, metadata finalization, and private cleanup
  - resource: /sync/src/paths.rs
    title: Root canonicalization, repeated ancestor validation, and owned-name category map
  - resource: /design/performance/performance0.sol61x.md
    title: Historical metadata syscall census and strictly read-only live roots
  - resource: /design/performance/parallel1.sol61x.md
    title: Measured frontier budget, wide-directory, dense-epoch, CPU, and memory limits
  - resource: /perf/results/frontier/confirmation.csv
    title: Recomputed historical five-observation live medians
  - resource: /perf/results/frontier/guided.csv
    title: Recomputed J sweep and independent K256 live repeat
  - resource: /perf/results/frontier/shapes.csv
    title: Recomputed wide, dense, and small-directory medians
  - resource: /perf/results/btrfs-192k-strace.txt
    title: Historical 383784-statx clean comparison census
  - resource: /perf/results/sync/observations.csv
    title: Existing 60 owned-fixture synchronization observations, not remeasured here
  - resource: "https://github.com/rust-lang/rust/blob/1.88.0/library/std/src/sys/fs/unix.rs#L883-L969"
    title: Historical DirEntry path, fd-relative metadata, d_type, and fallback implementation actually inspected
  - resource: "https://github.com/rust-lang/rust/blob/1.88.0/library/std/src/sys/pal/unix/kernel_copy.rs"
    title: Historical io::copy kernel offload, cached metadata, Take support, and fallback logic
  - resource: "file:///home/rektide/.rustup/toolchains/nightly-2026-10-14-x86_64-unknown-linux-gnu/share/doc/rust/html/src/std/sys/fs/unix.rs.html"
    title: Installed current std source: DirEntry shares InnerReadDir, uses dir-handle metadata, and exposes cached d_type
  - resource: "file:///home/rektide/.rustup/toolchains/nightly-2026-10-14-x86_64-unknown-linux-gnu/share/doc/rust/html/src/std/sys/fs/unix/dir.rs.html"
    title: Installed current directory-handle backend: metadata_c statx/fstatat and relative open/rename/mkdir/unlink
  - resource: "file:///home/rektide/.rustup/toolchains/nightly-2026-10-14-x86_64-unknown-linux-gnu/share/doc/rust/html/src/std/fs.rs.html"
    title: Current experimental std::fs::Dir API under feature dirfd, including its actual method coverage
  - resource: "file:///home/rektide/.rustup/toolchains/nightly-2026-10-14-x86_64-unknown-linux-gnu/share/doc/rust/html/src/std/sys/io/kernel_copy/linux.rs.html"
    title: Current File/Take kernel-copy specialization, descriptor metadata probes, and copy_file_range loop
  - resource: "file:///home/rektide/archive/bytecodealliance/cap-std/cap-std/src/fs/dir.rs"
    title: Inspected capability API at archive revision 715e4ed607ae9a93c7446b0fa63296f7898831c2, including literal-link contents variants
  - resource: "file:///home/rektide/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rustix-1.1.5/src/fs/at.rs"
    title: Verified AsFd-based openat/statat/readlinkat/unlinkat/renameat/mkdirat/symlinkat APIs
  - resource: "file:///home/rektide/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/reflink-copy-0.1.30/src/sys/unix/linux.rs"
    title: Actual Linux clone operation, source descriptor metadata, and destination permission finalization
  - resource: "file:///home/rektide/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/reflink-copy-0.1.30/src/sys/utility.rs"
    title: AutoRemovedFile is a guard around the supplied destination, not a second random temporary directory
  - resource: "file:///home/rektide/src/diff-dirs/.test-agent/optimization/README.md"
    title: Exclusive owned-fixture probes and retained historical/current syscall traces; excerpts promoted below
---

# What the current implementation actually pays for

This is **research, not an optimization implementation or a tuning decision**.
Production code was not changed. Small disposable probes call existing APIs;
there is no candidate copier, scheduler, or native traversal implementation.
The opportunity inventory uses this evidence rather than treating plausible
mechanisms as measured speedups.

## Toolchain direction and provenance

**User override during this investigation:** the project should be unpinned.
Use ordinary configured `cargo` and `rustc`; do not constrain designs, APIs, or
dependencies to Rust 1.88. The parent owns removing the old manifest baseline
and updating active root/perf instructions. This investigation does not edit
those files or introduce a toolchain pin.

The first source inspection/probes occurred before that correction and used
actual Rust **1.88.0**, commit `6b00bc3880198600130e1cf62b8f8a93494488cc`,
LLVM 20.1.5. Historical performance reports keep that actual compiler identity;
their timings are **not** current-nightly controls.

After the correction, every new build/probe used the configured toolchain:

```text
rustc 1.101.0-nightly (822182db4 2026-10-13)
commit-hash: 822182db4c4a1827256881d8c7aa685f0bcf2b11
host: x86_64-unknown-linux-gnu
LLVM version: 23.1.3
cargo 1.101.0-nightly (29c5daa1a 2026-10-07)
```

`rust-src` was absent for both exact toolchains. For 1.88, the versioned upstream
source was inspected. For the configured nightly, the **installed rust-docs
source HTML** was inspected under its sysroot, not the unrelated installed
stable toolchain's source. Both actual executables were traced independently
on this Linux/btrfs host with strace 6.13. Traced durations are not performance
measurements; no compiler-speed comparison is drawn.

<a id="e1"></a>

## E1 — std metadata is already parent-fd-relative

Historical std 1.88 `DirEntry::metadata` obtains `dirfd` from a retained `DIR*`,
then calls `statx(fd, basename, AT_SYMLINK_NOFOLLOW, BASIC_STATS|BTIME)`, falling
back to `fstatat64` on this Linux GNU target. `DirEntry::path()` is separately a
root/name join; metadata does not require that joined child path here.

Current nightly reaches the same syscall shape through its new internal
directory-handle backend: `DirEntry::metadata` in installed `unix.rs` lines
1027–1045 calls `dir_handle.metadata_c(&self.name, true)`; `unix/dir.rs`
lines 157–174 perform relative `statx`/`fstatat64`.

Both real traces contain the equivalent of these normalized excerpts:

```text
openat(AT_FDCWD, ".../source", O_RDONLY|O_DIRECTORY|...) = 3
getdents64(3<.../source>, ...)
statx(3<.../source>, "link", AT_STATX_SYNC_AS_STAT|AT_SYMLINK_NOFOLLOW, STATX_ALL, ...)
statx(3<.../source>, "nested", AT_STATX_SYNC_AS_STAT|AT_SYMLINK_NOFOLLOW, STATX_ALL, ...)
statx(3<.../source>, "payload", AT_STATX_SYNC_AS_STAT|AT_SYMLINK_NOFOLLOW, STATX_ALL, ...)
```

`STATX_ALL` here is strace's name for the requested BASIC_STATS|BTIME mask;
returned masks can also include MNT_ID/SUBVOL. A narrower request need not cause
the kernel to return less data or become faster.

The probe drops `ReadDir` before using its collected entries. The FD stays open
until those entries are dropped because each entry shares `Arc<InnerReadDir>`.
There is **one retained parent descriptor, not a new descriptor per entry**.
Another thread successfully borrows one of those entries and issues the same
relative metadata syscall. Compile-time Send+Sync assertions succeed for
`DirEntry`, `File`, `OwnedFd`, and `BorrowedFd` on both compilers.

The existing core, rebuilt with each compiler and traced on the owned clean
pair, already issues relative child stats. Its directory opens and link reads
remain path-based:

```text
statx(3<.../source>, "payload", ..., STATX_ALL, ...)
readlink(".../source/link", "missing//./literal", 256) = 18
openat(AT_FDCWD, ".../source/nested", O_RDONLY|O_DIRECTORY|...) = 3
```

**Consequence:** replacing `DirEntry::metadata` with manual `fstatat` is not a
new prefix-walk elimination on this target. A dirfd backend may still improve
child directory opens, literal readlink, sync operations, name conversion,
root/ancestor validation, or handle reuse. Separately, `pair.rs::inspect_entry`
eagerly constructs a full child `PathBuf` before the already-relative metadata
call. Removing that userspace allocation is a different hypothesis, not a
kernel-path-resolution improvement.

<a id="e2"></a>

## E2 — cached d_type can avoid some metadata, with real caveats

In both compilers the fixture's three immediate `file_type()` calls (directory,
regular file, link) produce **zero metadata syscalls** between their BEGIN/END
markers. The separate borrowed-metadata marker then produces one expected stat.
Installed current std lines 1053–1076 dispatch recognized d_type values directly;
unknown values fall back to metadata.

The core currently stats **every** immediate entry. In the historical large
clean census, 383,784 statx calls equal two root checks plus two stats per one-side
entry count of 191,891. The regular-file key still needs size/mtime. Directory
comparison does not; one-sided leaf classification and file↔link transitions
also do not need all currently fetched values. Reading link contents only for
paired links could avoid other work. But DT_UNKNOWN filesystems may turn a
naive `file_type`-then-`metadata` strategy into **two** stats per regular file.
Skipping observations also changes failure timing/coverage on changing trees.

This establishes available machinery, not an implemented lazy-inspection result.

<a id="e3"></a>

## E3 — ordinary copying already has kernel offload

Both actual compilers' probes show `io::copy(&mut File,&mut File)` issuing:

```text
statx(source_fd, "", AT_EMPTY_PATH|..., STATX_ALL, ...)
statx(output_fd, "", AT_EMPTY_PATH|..., STATX_ALL, ...)
copy_file_range(source_fd, NULL, output_fd, NULL, 1073741824, 0) = 4096
copy_file_range(source_fd, NULL, output_fd, NULL, 1073741824, 0) = 0
```

The second call determines EOF. `fs::copy` on the same owned source also uses
copy_file_range. The current sync Copy branch does too. Current installed
`kernel_copy/linux.rs` retains the offload/fallback strategy and File/Take
specializations; its private cached-metadata wrapper is not a public reusable
API. Arbitrary application Read/Write wrappers can lose specialization.

**Consequence:** proposing copy_file_range/sendfile as if Copy were a naive
userspace loop is wrong for these paths. A future experiment can investigate
known-length copying, descriptor metadata reuse, offload selection, and its
fallback behavior. Ordinary Copy can itself exploit CoW inside the kernel;
neither the Copy counter nor successful copy_file_range proves exclusively
physical copying, shared extents, or guaranteed CoW. Required reflink remains
a separate guarantee obtained through the explicit clone operation.

<a id="e4"></a>

## E4 — sync's removable costs are concrete, but not yet timed separately

Current nightly baseline trace, two nonempty files plus one literal link:

- Each recursively copied child is inspected in the directory dispatch and
  again in `copy_leaf` or `copy_directory`: consecutive full-path stats are visible.
- Each leaf, **including a new link**, creates a private directory, installs the
  absent `entry`, renames it, then invokes the generic recursive-removal kernel
  on the now-empty private directory. That cleanup includes two path stats,
  a directory open, two getdents calls on this fixture, close, and rmdir.
- Strict clone creates the supplied staged file once, calls FICLONE, fchmods
  and closes it inside the dependency; the applier then chmods the private inode,
  reopens it, sets mtime, stats its exact readback, fchmods final permissions,
  closes, and renames. The trace includes three permission-setting calls per
  regular clone on this writable-mode fixture.
- Root binding/initialization repeats prefix checks and realpath/readlink work;
  `paths::relative` also checks every supplied directory ancestor per record.
  This is a fixed-root or depth/record cost, not necessarily the dominant 30k
  initialization cost.

The dependency's Linux `AutoRemovedFile` wraps **the destination passed to it**;
it does not introduce another random directory/rename stage. Its documented
"temporary" semantics are cleanup ownership. The actual opportunities are the
outer per-leaf directory and the descriptor/metadata lifecycle, not an invented
second Linux staging directory. `ReflinkBlockBuilder` borrows File handles but
is not a portable whole-file drop-in: zero lengths, range/cluster rules, Windows
setup, and absent Apple support require separate handling.

<a id="e5"></a>

## E5 — current nightly expands the backend choices

The configured compiler already exposes experimental **`std::fs::Dir`** behind
`#![feature(dirfd)]` (tracking issue 120426). Installed public source lines
1558–1950 and its documented method list were inspected. A separate read-only
probe compiles, asserts Dir: Send+Sync, shares one Dir across scoped threads for
symlink_metadata, opens a child directory, and opens a file relative to it.
Its trace confirms basename-relative statx and openat. This is API availability
and ownership evidence, **not an implemented traversal backend or speed result**.

The inspected public API covers directory/file open, metadata, create_dir,
remove_file/remove_dir, and cross-handle rename. It does **not** currently offer
public read_dir, literal readlink, or symlink creation methods in that method
list. A full backend still needs complementary facilities, or deliberately
smaller coverage. Std Dir is relative addressing, not automatically confinement:
the Unix implementation calls the ordinary *at APIs without cap-std's complete
path-resolution policy. Nightly API instability is a tradeoff, not grounds to
reject the candidate under the user's new toolchain direction.

`rustix` 1.1.5 exposes AsFd-based *at APIs and FICLONE. Its Linux `Dir::read_from`
reopens `"."` for an independent directory stream; blindly duping a handle
shares the open-file-description offset. `cap-std` 4.0.2 in the archive offers
broader portable handle operations and confinement machinery, potentially with
extra resolution work. For literal absolute links it is important to choose
`read_link_contents`/`symlink_contents`, not the variants that reject absolute
link contents. These libraries are not interchangeable syscall-count promises.

<a id="e6"></a>

## E6 — existing performance evidence still identifies the shape of the problem

Selected raw medians were recomputed, not newly measured:

| Historical scope/control | Observation | Interpretation, not attribution |
| --- | --- | --- |
| Live node_modules, P4/J4/K256 | 722.817 ms serial, 247.010 frontier | Existing across-directory win; not a future optimization claim |
| Whole live roots, same confirmation | 808.672 ms serial, 282.329 frontier | Same distinction; current live counts may change |
| Wide 30k root, P4/J4/K256 | 80.115 serial, 79.911 frontier, 40.119 predecessor per-frame | One frontier directory job leaves wide-entry work serial |
| Small-directory fixture | 293.419 serial, 84.618 frontier | Directory parallelism already matters |
| Dense fixture diagnostic | 4001 epochs, 26.771 ms cumulative drain, mean active ~1.136 | Mutation acknowledgment/seals constrain this shape, not saturated K |
| Live K16 versus K256 diagnostics | 14581 versus 392 K stalls; mean active ~1.373 versus ~3.242 | Admission policy merits study; neither is a byte bound |
| Owned sync fresh Copy | 3450.268 ms serial, 1668.983 P4 | Existing application baseline, not a current-nightly timing |
| Owned sync clean Copy | 75.532 ms serial, 21.715 P4 | Zero application writes; near its diff-only control |

CPU/RSS resource observations remain separate: the historical frontier improved
wall time while using more total CPU and sampled memory. A mutex, allocator,
or path-joining fraction was **not** isolated by those measurements. The 82.71%
statx share in an old strace census is traced syscall time, not a universal
untraced wall-time fraction or an Amdahl prediction.

## Reproduction, retained scratch, and scope safeguards

Owned probes and raw traces live in ignored `.test-agent/optimization/`; the
README records exclusive setup and each mode. After the toolchain correction:

```sh
rustc -Vv
cargo build --release --manifest-path .test-agent/optimization/Cargo.toml --target-dir .test-agent/optimization/target-current
strace -f -yy -e trace=%file,getdents64,close,write,copy_file_range,sendfile,splice,ioctl,fchmod -o .test-agent/optimization/current-metadata.trace .test-agent/optimization/target-current/release/optimization-probe metadata current
rustc --edition 2024 -O .test-agent/optimization/dirfd_api.rs -o .test-agent/optimization/dirfd-api-current
strace -f -yy -e trace=%file,getdents64,close -o .test-agent/optimization/current-dirfd-api.trace .test-agent/optimization/dirfd-api-current
```

Other modes: `file-type`, `copy-io`, `copy-fs`, `sync-copy`, `sync-reflink`.
Copy/sync modes refuse existing owned destinations; supply a new alphanumeric
suffix rather than resetting old fixtures. The historical trace filenames have
no `current-` prefix; they predate the override. Current sync traces add fchmod
to the filter, so absence of it in the old filtered traces is not evidence of
absence of the operation.

The pre-existing `.test-agent/sync/btrfs-30k` was inspected read-only: it currently
contains its marker and `source/`, **no retained paired target**. Its source
census remains 300 directories, 30,000 regular files, 300 links, 122,880,000
logical bytes. The performance harness used private temporary targets and
removed its own targets. Future experiments must not assume an extant clean
source/destination pair or delete that retained source fixture.

No new live-tree benchmark campaign was run. `/home/rektide/a/a/opencode` and
`/home/rektide/src/opencode-export-qa`, whole roots and node_modules, remain
strictly read-only comparison anchors. No sync or jj-waltz mutation was performed.
