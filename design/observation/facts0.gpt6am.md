---
type: ResearchProposal
title: Demand-shaped physical facts and execution reuse
description: Source-grounded fact-demand table, acquisition ownership, retention boundaries and bounded experiments for inspection and transfer.
resource: /design/observation/facts0.gpt6am.md
status: proposed
generated: { by: "model:openai/gpt-6-astra#medium", at: 2026-10-20 }
sources:
  - resource: "opencode://ses_ea7dbee62ffeQLsnpAkxntkr5L"
    title: Independent wave-one source analysis of decision and execution fact demands, backend counts and reuse contracts
  - resource: /design/roadmap/brief0.gpt6a.md
    title: Independent commission and human constraints
  - resource: /doc/architecture/README.md
    revision: 1193a1ee
    title: Prior-wave proposed architecture and observation discriminator
  - resource: /design/architecture/synthesis0.gpt61sxh.md
    revision: 1193a1ee
    title: Prior-wave integration and scoped fact-demand commission
  - resource: /design/execution/state0.gpt6a.md
    revision: 1193a1ee
    title: Established fact lifetimes and delivered early-hint implementation evidence
  - resource: /src/listing.rs
    revision: 1193a1ee
    title: Cached hints, private entry handles and deferred metadata acquisition
  - resource: /src/visitor.rs
    revision: 1193a1ee
    title: Presence, full late observations, comparison and finalization
  - resource: /src/pair.rs
    revision: 1193a1ee
    title: Root reuse and release of clean plans
  - resource: /src/walk.rs
    revision: 1193a1ee
    title: Owned pending children and divergent-only publication
  - resource: /sync/src/apply.rs
    revision: 1193a1ee
    title: Fact reuse, recursive removal, staged copying and permission leases
  - resource: /sync/src/lib.rs
    revision: 1193a1ee
    title: Internal run/application seam and root dispatch
  - resource: /sync/src/visitor.rs
    revision: 1193a1ee
    title: Current late-only presets and adapter ordering
  - resource: /sync/src/platform.rs
    revision: 1193a1ee
    title: Platform-specific symlink creation/removal demands
extensions:
  ticket: diffdir-tree-observe-facts
  reading_boundary: Independent brief, ticket, named prior-wave documents and implementation only; withheld statement and same-wave outputs not consulted
verification:
  method: Source-only analysis at 1193a1ee; document checks; no production modifications, runtime probes, suite reruns or benchmarks
  result: Counts below are call-site derivations and conditional savings, not measured syscall totals or timings
---

# Demand-shaped physical facts

**Acquire facts for a stated decision or selected execution, and retain already
acquired facts until their last immediate consumer.** This does not mean avoiding
all stat calls. Copying regular files needs source attributes; ordinary physical
file comparison needs attributes from both sides. Much of the removable work is
target metadata for kind-only decisions, unnecessary literal-link reads, and
reacquisition of directory attributes already observed.

The smallest useful change is a demand-aware immediate inspector with a private
fact cache and an execution handoff. It need not introduce a universal provider,
content hash or new native parser. Existing full-fact visitors remain a full-demand
compatibility mode. A sparse observation cannot honestly implement today's
unconditional `Facts::metadata() -> &Metadata` contract.

## 1. What the implementation actually pays

Source coordinates below refer to `1193a1ee` (the checkout inspected).

- [Listing](/src/listing.rs), lines 68–103 and 151–245: Linux native is opt-in;
  it returns cached kind/inode and later uses one no-follow basename-relative
  metadata acquisition per admitted side. Default Unix returns inode but Unknown
  kind; it deliberately avoids `file_type()`'s probing fallback. Windows obtains
  cached enumeration metadata and returns that same value later. None of the
  early accessors performs I/O.
- [Inspection](/src/visitor.rs), lines 1004–1110: list each present containing
  directory, merge native names, run early admission, then fetch **full metadata
  for every admitted present side**. Every source symlink is readlinked. Every
  target symlink is readlinked when the name is Both, or purpose is Audit.
  Target-only removal already omits readlink.
- [Comparison](/src/visitor.rs), lines 785–825: paired directories need kind,
  paired regular files need size and exact supported mtime, paired symlinks need
  literal native link text. A size mismatch short-circuits mtime extraction but
  does not undo the metadata acquisition. One-sided entries and unlike supported
  kinds differ without comparing attributes. Unsupported kinds are not equal.
- [Application](/sync/src/lib.rs), lines 309–322, and
  [internal apply](/sync/src/apply.rs), lines 149–170, 490–543: normal `run`
  passes the actual plan to execution; it does **not** restat every copied leaf.
  Source metadata/link text and target removal metadata are reused.
- [Public apply](/sync/src/apply.rs), lines 34–146, deliberately re-inspects roots,
  routing ancestors and the immediate batch. This is a separate external-plan
  boundary, not overhead paid by every internal run. Preview is also explicit
  extra work, not an implicit run preaudit (lines 173–201).

For an admitted immediate batch let `S` and `T` count present source and target
entries, and `Ls`, `Ltb`, `Lta` count source links, Both target links and target-only
links. Current listing-backed metadata calls are exactly `S + T`; current readlink
calls are `Ls + Ltb + (Audit ? Lta : 0)`. Early-excluded slots contribute zero.
These are successful-path call-site counts, excluding roots, directory listing,
permission preparation, output verification and transfer-kernel internals.
On Windows the metadata calls reuse enumeration data, so they are **not** that
many additional filesystem queries. On Linux they are metadata acquisitions;
exact syscall implementation is platform/toolchain dependent.

## 2. Practical demand table

`K` = supported no-follow kind (including observed unsupported kind);
`A` = actual no-follow attributes; `L` = literal native link spelling.
Listing presence and native name are always distinct from these fields.
“Minimum” assumes no custom policy demanding more, a good stable source, and
the current physical comparison predicate where comparison is requested.

| Situation | Facts to decide the step | Additional facts for selected execution | Acquisition and retention |
| --- | --- | --- | --- |
| Name-based preservation, e.g. root `.jj` | Logical coordinate, name, pair presence; no K/A/L | None; preserve both sides and subtree | Inspector enumerates containing sides; keep only coverage/boundary data after admission. Structural root checks remain. |
| Kind-based preservation | K on whichever sides policy examines; Unknown requires an explicit request or deferred decision | None if preserved | Inspector resolves Unknown only after name admission. Known listing K can decide without A. Never infer absence from Unknown. |
| Paired regular physical comparison | Both K, both size; both mtime if sizes match | If install: source permissions and source mtime, plus source size for reflink byte accounting | One A acquisition generally supplies all fields on each side; retain source A through selected installation, discard target attributes after their last policy/comparison use. |
| Matched directories | Both K; directories are structural, no mtime equality | Target container permissions only when writes need a lease; source mode not copied onto an existing matched directory | Keep pending child identity; transfer target permissions to the active write lease if already observed. |
| Source-only regular file | Source K and target absence suffice for “different/install” | Source mtime, permissions, size; source file handle and bytes/clone source | Inspector or executor obtains A once after selection; installer retains attributes through final metadata and rename, handles only for the live operation. |
| Source-only directory | Source K, target absence, container/descent selection | Source permissions for final new-directory mode; enumerate admitted source descendants if descending | Retain directory mode through postorder finish. No target interior listing for a newly created empty container. |
| Paired symlinks | Both K and both L; compare `OsStr`, not normalized `Path` equality | Source platform link subtype where required; reuse source L | Inspector reads text once; installer borrows/moves it until creation. Unix needs no source link permissions/mtime for current installer. Windows uses symlink-dir subtype. |
| Source symlink versus non-link/absent target | Source K, target K/absence establish difference | Source L only if installed; Windows subtype | Defer source readlink until install is selected unless a policy explicitly requests it. |
| Target symlink versus non-link source | Both K establish difference | No target L; removal platform subtype may matter | Current Both target-link read is unnecessary to baseline comparison or installation. Preserve custom policy demand explicitly. |
| Target-only leaf removal | Target K and true source absence; policy decision | Unix unlink needs no leaf attributes; K classifies progress. Windows distinguishes directory symlink removal | No source query, no target L. Target parent permissions are separate execution facts, not attributes of the removed leaf. |
| Target-only directory removal | Target K, source absence, descent/removal authority | Descendant names/kinds and policy demands; writable directory modes as necessary | Enumerate recursively; retain original permissions until removal or restoration. Descendant retention can keep ancestors. |
| Directory → leaf replacement | Source leaf K, target directory K; removable descendant decisions before dismantling | Source installation facts above, target removal traversal and modes | Couple replacement authority; a retained descendant refuses intact. Reuse local precheck results where retained, rather than automatically scanning twice. |
| Leaf → directory replacement | Source directory K, target leaf K | Source directory mode and selected descendants; target removal subtype | Prepare source subtree in a private location, then remove leaf/install directory. No target leaf size, mtime or L needed by baseline. |
| Generated-target satisfaction | Desired output kind, parameterized satisfaction rule; physical target presence/K and exactly the facts that rule requests | Generated bytes/recipe and intended attributes, destination preparation facts | Explicit producer/satisfaction observer; desired fields never become physical source A. Unknown satisfaction stays Unknown, not equality. |
| Explicit audit/custom metadata policy | Whatever the declared audit/check consumes; current Check receives full A and available L | Only selected work's additional demands | Compatibility mode requests full existing payload. Narrow consumers opt into sparse facts; accessors never fill gaps. |

Generated satisfaction is not necessarily stat comparison: a rule may require
target bytes, a domain-specific marker, or choose unconditional installation.
Recipe identity alone says nothing about installed bytes. No universal hash is
selected. A requested content read is explicit read-only observation, not I/O
inside the pure policy callback. Generated output work must also reach execution
when physical diff is empty; today's clean-plan dropping and divergent-only
yielding cannot provide that merely by adding a fact field
([pair](/src/pair.rs):92–103; [walk](/src/walk.rs):191–209).

### Quantified source/target opportunities

For Linux native entries with Known K, the following are **additional metadata
acquisitions**, per slot, before versus after a narrow-demand design. They exclude
root/parent preparation and newly installed output verification.

| Case | Current source / target | Minimum source / target | Qualification |
| --- | --- | --- | --- |
| Paired regular comparison | 1 / 1 | 1 / 1 | No metadata-I/O saving from d_type alone. Copy reuses source A. |
| Matched directories, read-only descent | 1 / 1 | 0 / 0 | Need no directory attributes unless policy requests them. |
| Source-only regular, installed | 1 / 0 | 1 / 0 | Decision can be cheap, but execution still needs source A. |
| Source-only directory, created | 1 / 0 | 1 / 0 | Source permissions remain necessary for current copy semantics. |
| Source-only Unix link, installed | 1 / 0 | 0 / 0 | One source readlink remains; platform subtype can change the demand. |
| Target-only Unix leaf, removed | 0 / 1 | 0 / 0 | Known supported K suffices; parent-mode acquisition remains. |
| Source regular → target link | 1 / 1 | 1 / 0 | Current target readlink also disappears. |
| Source directory → target leaf | 1 / 1 | 1 / 0 | Preserve source final mode; target removal kind sufficient on Unix. |
| Paired Unix links | 1 / 1 | 0 / 0 | Two readlinks remain for physical comparison. |
| Name-preserved pair | 0 / 0 | 0 / 0 | Already delivered, even with Unknown hints. |

On default Unix, Unknown K generally forces a no-follow metadata acquisition;
the same cache must satisfy later A demand without a second acquisition. Thus many
native-K savings collapse to zero on that backend. Windows already caches A;
there the likely gain is payload lifetime/construction and fewer unnecessary
readlinks, not avoidance of per-entry stat syscalls that are not being issued.
No timing percentage follows from these counts.

## 3. Smallest demand/acquisition contract

Use a **finite staged demand protocol**, not arbitrary accessor-triggered I/O:

1. List and merge physical names. Record per-side `Absent` or `Present` with
   cached physical hints. Record exclusion separately. Listing-incomplete/error
   is not absence. An admitted listed entry disappearing during acquisition is
   an observation failure, as today, not manufactured deletion authority.
2. Pure name admission can preserve/refuse immediately. A pure demand function
   over names/hints requests missing K or declares it irrelevant. Inspector owns
   acquisition and caches the resulting A when stat is the way K is resolved.
3. From resolved shape, the core comparison and declared policy request their
   fact union: stat key/full A, literal L, or an explicit generated satisfaction
   input. Fetch missing facts once, then assess/select and validate the complete
   immediate batch. Full-metadata compatibility visitors request A up front.
4. Selected execution requests only its missing inputs. Fill before the relevant
   effect unit, reusing all current observations. If acquisition failure must
   precede every effect in a batch, complete these inputs for the whole batch
   first; otherwise document the smaller failure boundary. Do not silently move
   current late observation errors behind earlier mutations.

The minimal vocabulary can be `Kind`, `Attributes`, `LiteralLink`, with
per-side demand and a distinct generated satisfaction request. Fine-grained
size/mtime/mode bits may document semantic needs but do not imply cheaper OS
stat variants. Prefer one real `Metadata` cache when acquired; project borrowed
fields without building fake partial `Metadata` values.

Represent sparse payloads as present-side observation objects containing explicit
`Unknown/NotRequested` versus `Known(value, origin)` fields. Physical origin includes
listing, root metadata or admitted metadata; generated desired attributes live in
a separate input. A known unsupported kind is a positive observation, not missing
data. The existing outer presence `Option` must not become the missing-A marker.
Comparison needs its own NotAssessed/Unsupported/Equal/Different distinction if
operation-only modes omit baseline comparison; today's `comparison() == None`
specifically means unsupported, so repurposing it would be misleading.

Two incremental options:

- **Compatibility-first:** preserve current `inspect_directory` and full late
  visitor contract, add a narrow-demand inspection entrypoint/component for new
  consumers. Share merge, acquisition cache and validation internally. This is
  the smallest honest public transition and exposes the savings table directly.
- **Unified demand visitor:** change late facts to explicit availability and give
  old visitors an adapter requesting full A/current L. This removes duplicated
  surface area but is a larger API migration. Do not infer demand from whether
  a callback happened to access a field in one invocation.

Recommendation: test the compatibility-first shape before choosing the unified
API. Both require explicit policy precedence. Current Update, Seed and Prune
are late-only ([sync visitor](/sync/src/visitor.rs):46–104); absence adapters run
before the caller's late visitor (26–41). Moving these decisions to terminal early
preservation can suppress checks or prevent later overrides. Their observation
cost is avoidable **for a newly declared terminal intent**, not automatically a
semantics-preserving edit to current presets. Likewise, omitting comparison in
seed/prune modes changes reported baseline evidence and must be explicit.

## 4. Ownership and retention

| Owner | Lifetime and retained inputs |
| --- | --- |
| Listing/inspector | Parent handles and raw entries through name admission and requested acquisition only. Current native handle owns a name plus shared parent. No live directory handle in public facts or continuation. |
| Policy | Borrows immutable facts during pure calls; frozen caller parameters are its own input. No filesystem probes, caching by callback order or effect history. |
| Final immediate work | Owns/moves names and just the facts needed by pending work, public evidence contract or imminent child/finish obligations. Retaining full A already acquired can be cheaper than converting it repeatedly. |
| Driver | Owns pending child names/coordinates. Clean fact payloads can die after child lowering, as already implemented in pair.rs:101–103. Carry a directory's known permissions into its child execution context only if needed. |
| Executor | Source A/L through installation; source directory mode through finish; target original permissions through lease closure. Live source/output descriptors remain runtime-only. |
| Pause representation | Own pending coordinates/parameters and required finish values or declare reacquisition at resume. Do not serialize std Metadata or dirent handles by assumption. A saved value remains an observation under the caller's stability agreement, not a validity certificate. |

The later native/packed-name study therefore needs **lossless native component
identity, deterministic sorting, two-side pairing, side provenance, and ownership
past enumeration**. Borrowing raw dirent bytes only until the next buffer refill
does not satisfy the existing collect/sort/early/late pipeline. Possible storage
must either retain backing buffers or copy names into an owned batch arena;
pending children and work must keep that backing alive or promote their names.
Relative paths can be materialized on demand, but logical final and private
physical preparation paths remain different. Early exclusions need only their
boundary evidence; child jobs, staging reservations and retained public plans
can outlive listing. No parser or unsafe representation is chosen here.

## 5. Re-queries and already-delivered savings

### Concrete remaining candidates

1. **Target link text on type mismatch.** `visitor.rs:1105–1108` requests target
   readlink for all Both slots. Only paired links use it in default comparison
   (`800–802`); install/removal uses source text and target kind. Shape-directed
   demand avoids one target readlink per such mismatch unless policy asks for it.
   Source-only link text is also unnecessary in read-only diff or late-retained
   installation; delaying it requires the sparse policy contract above.
2. **Directory permission handoff.** `Staging::new` builds `Writable::new`
   (`apply.rs:1311–1336`); first actual mutation invokes `relax`, which calls
   `paths::directory` again (`1407–1413`, `1469–1471`). Target directory A was often
   observed in its parent slot or at binding. Carry its permissions to child
   execution and use `Writable::verified`, which already exists and is used by
   grouped seed (`1243–1245`). Potential: one avoided directory metadata query
   per activated lease with available current permissions, not one per leaf.
   Private newly created directories still need their actual post-umask mode;
   source mode is not a substitute for that target fact.
3. **Directory → leaf double traversal.** `execute_entry` runs `removable`
   (`504–506`), recursively constructing target-only plans (`965–991`). After
   preparing the replacement, `copy_leaf_with` calls `remove_present` (`835`),
   restatting the target directory and re-inspecting descendants through
   `reconcile_directory` (`943–960`, `621`). For a removable subtree with `N`
   descendants and `D` directories including its root, this means two target
   listing passes over those `D` directories and two admitted metadata acquisitions
   per descendant today, plus the root removal stat. Holding precheck plans can
   avoid the second observation pass under the stated stable-tree assumption.
   Cost: O(N) retained local plans instead of transient depth/width state. This
   is a coupled-replacement optimization, not a reason to retain whole-tree
   catalogs for ordinary copying. Preview also invokes precheck then traverses
   removal children (`465–485`); it can reuse results similarly.
4. **Root handoffs.** `whole_root` observes target (`lib.rs:402–407`), then paired
   binding observes roots again (`pair.rs:30–53`). Initialization/replacement
   have additional explicit structural checks (`apply.rs:208–269`). Passing bound
   facts within one trusted execution may remove some calls; root validation and
   externally obtained plan validation remain separate obligations. This is a
   small per-run opportunity, not the dominant per-leaf opportunity.

### Do not claim these again as new wins

- Name preservation already avoids child stat/readlink/descent on Known **and**
  Unknown hints; containing listing remains. The prior
  [state report](/design/execution/state0.gpt6a.md#enumeration-can-already-provide-facts)
  records backend implementation and narrow syscall evidence, not rerun here.
- Target-only non-Audit links already omit readlink; Windows already reuses its
  cached metadata; roots already reuse bound facts for root callbacks
  (`visitor.rs:1147–1163`).
- Internal application already reuses source A/L. Linux clone already retains
  open descriptors and uses FICLONE; copy uses `io::copy`, with a reused source
  descriptor on fallback (`apply.rs:846–884`). A universal “executor restats all
  source files” diagnosis is false.
- Output `metadata()` in `preserve_file` (`1283–1299`) verifies exact installed
  mtime after setting it. Source metadata cannot answer that question. Count this
  as an output-verification read, not duplicate source observation. Kernel-level
  internal metadata calls are outside this source-only census.

## 6. Whole-subtree knowledge: what disappears and what remains

**Preserve this region:** no interior listing, metadata or effects. Its contents
remain unassessed; this is not equality. Parent listing and root constraints stay.

**Source region is desired; populate an empty receiving region:** no target
interior listing/comparison is needed. Source names still must be discovered
unless an explicit complete name list is supplied. Each installed regular file
still needs source bytes/clone access and source attributes; directories need
source final permissions. Known K can remove classification-only queries, not
those copy attributes. The current source-only copy path already lists only the
source (`apply.rs:589–602`).

**Both regions satisfy a declared predicate, including all relevant descendants:**
the predicate can replace assessment/enumeration/effects within its stated scope.
The assertion must describe target satisfaction, content domain, intended metadata
and descendant-policy coverage. A source repository tree identity alone does not
assert physical receiver satisfaction or cover untracked/generated names. Do not
introduce recurring donor revalidation: the human's good-source premise is an
input, with concurrency/locking out of band.

**Delete the entire region with explicit whole-region authority:** descendant
policy assessment may vanish; filesystem deletion still enumerates descendants
internally or explicitly, chooses unlink versus directory removal and handles
permissions. Current per-kind counters also demand classification. A call to
recursive removal is not proof that enumeration I/O disappeared.

**Known leaf list:** source discovery of unrelated directories disappears, as
already delivered by `seed_leaves` (`apply.rs:1059–1086`, `1243–1269`). Source A/L,
target absence checks, routing checks and transfer remain. The list grants no
authority over unlisted target names. Replacing target existence checks with an
empty-region assertion is a different installation contract, not a free cache hit.

## 7. Bounded next experiments and handoff

No prototype was necessary to distinguish the present call paths. Before a public
API change, use a tiny fake acquisition source with counters, not a large benchmark:

1. **Demand discriminator:** one Known-directory pair, Unknown-directory pair,
   paired regular file, source-only file, target-only link, mismatched target link,
   preserved Unknown slot and unsupported Known slot. Assert separate S/T metadata
   and L counts, one acquisition on K→A escalation, unchanged absence/exclusion,
   deterministic failure order and full-demand compatibility. Include generated
   satisfaction returning Unknown and requiring target bytes; it must never
   manufacture physical source metadata.
2. **Modes ticket:** a two-level tree with one writable and one restrictive
   existing target directory, two sibling installs and one clean directory.
   Instrument lease activation and metadata calls. Compare parent-permission
   handoff with current `Writable::new`; check restoration and zero lease work in
   the clean directory. Include a newly created directory under restrictive umask
   to prevent incorrectly substituting source permissions for its actual mode.
3. **Kernel ticket (`diffdir-tree-transfer-kernels`):** one regular Copy, successful
   required clone and forced permitted fallback. Attribute source inspection,
   kernel-internal descriptor queries and output exact-mtime verification
   separately. Keep the latter until its correctness obligation changes. Non-Linux
   path-helper costs need platform evidence, not extrapolation from Linux.
4. **Name-storage ticket (`diffdir-tree-observe-names`):** a few long/non-UTF-8
   names, enough records to refill a listing buffer, excluded slots and one pending
   child retained beyond batch drop. Measure allocations/retained backing bytes
   across merge→early→acquisition→work/child promotion, without changing demand
   semantics. The required lifetimes are in section 4.
5. **Replacement discriminator:** a two-directory removable target with one leaf
   each and a variant containing a preserved descendant. Compare transient
   double inspection with locally retained precheck plans. Record retained bytes
   and avoided S/T reads; require intact refusal before staging/dismantling for
   the preserved variant. This decides the memory/I/O tradeoff without journals.

The proposed baseline in [architecture](/doc/architecture/README.md#immediate-inspection-and-decision-authority)
supplies physical provenance and pure assessment; this study refines its
“complete admitted facts” stage into explicit consumer demands. The
[state account](/design/execution/state0.gpt6a.md#2-library-execution-state-lifetimes-and-actual-file-data)
supplies existing retention precedent; this proposal extends directory-fact
handoff without turning transient facts into a permanent catalog. Parent owns
index integration, closure and export. No same-wave comparison is made here.
