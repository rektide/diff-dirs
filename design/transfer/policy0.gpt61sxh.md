---
type: PolicyStudy
title: Transfer authority without implied cleanup
description: Concrete overwrite, retention, removal and type-transition policies for ordinary directories and JW creation/reseed, including snapshot classification and generated or mapped destinations.
resource: /design/transfer/policy0.gpt61sxh.md
status: independent-research-for-selection
generated: { by: "model:openai/gpt-6.1-sol#xhigh", at: 2026-10-20 }
sources:
  - resource: "opencode://ses_ea7db9f63ffeeKrXJOTes1YxLb"
    title: Independently derived transfer matrices from current core/sync and JW source, separated snapshot history from mutation authority, and exposed bounded policy choices without selecting defaults
  - resource: /design/roadmap/brief0.gpt6a.md
    title: Independent commission, ordinary-directory scope and failure boundaries
  - resource: /doc/architecture/README.md
    revision: 1193a1ee
    title: Prior-wave proposed architecture and scoped transfer-policy question
  - resource: /design/architecture/synthesis0.gpt61sxh.md
    revision: 1193a1ee
    title: Prior-wave integration and unresolved transfer choices
  - resource: /design/native/metadata0.gpt6am.md
    revision: 014db6e3
    title: Completed factual study of shared repository storage and receiving-owned native metadata
  - resource: /design/native/workspace-source0.gpt6am.md
    revision: d6f19aeb
    title: Completed factual study separating registered donor, repository owner and policy roots
  - resource: /README.md
    title: Governing read-only comparison and early/late visitor contract
  - resource: /src/visitor.rs
    title: Immutable presence, pair-wide preservation and coupled action authority
  - resource: /sync/README.md
    title: Governing mirror/update/seed, removability, transport and failure contracts
  - resource: /sync/src/visitor.rs
    title: Actual Seed, Update, Prune and compatibility adapter order
  - resource: /sync/src/lib.rs
    title: Root binding and visitor composition
  - resource: /sync/tests/visitor.rs
    title: Read tree-seed and collision tests; not executed in this study
  - resource: /sync/tests/authority.rs
    title: Read intact directory-replacement and listed-seed tests; not executed in this study
  - resource: "file:///home/rektide/src/jj-waltz/src/cow/transfer.rs"
    title: Current auxiliary selection, routing, creation obstruction and explicit clear policies
  - resource: "file:///home/rektide/src/jj-waltz/src/cow/ignored.rs"
    title: Current list precedence, source-root discovery and tracked-overlap admission
  - resource: "file:///home/rektide/src/jj-waltz/src/cow/payload.rs"
    title: Frozen tracked ignore rules and native tracked-entry extraction
  - resource: "file:///home/rektide/src/jj-waltz/src/lifecycle/ignored.rs"
    title: Creation's independently selected source payload and absent-root requirement
  - resource: "file:///home/rektide/src/jj-waltz/src/lifecycle.rs"
    title: Current creation modes, base resolution and actual donor selection
  - resource: "file:///home/rektide/src/jj-waltz/src/workspace.rs"
    title: Physical primary ownership, native creation and tracked clone materialization
  - resource: "file:///home/rektide/src/jj-waltz/src/reseed/engine.rs"
    title: Current snapshot/classification, obstruction checks, tracked refill and selected payload execution
  - resource: "file:///home/rektide/src/jj-waltz/src/reseed/payload.rs"
    title: Current outside-payload validation and administrative-entry guard
  - resource: "file:///home/rektide/src/jj-waltz/src/reseed/bookmarks.rs"
    title: Bookmark coverage of post-snapshot work, not initial-untracked preservation
  - resource: "file:///home/rektide/src/jj-waltz/src/reseed/prepare.rs"
    title: Source-only preparation, without target transfer or durable progress
  - resource: "file:///home/rektide/src/jj-waltz/src/reseed.rs"
    title: Public request semantics and explicit empty-list representation
  - resource: "file:///home/rektide/src/jj-waltz/README.md"
    title: Current CLI creation, copy selection, links and reseed eligibility
  - resource: "file:///home/rektide/src/jj-waltz/docs/reseed.md"
    title: Maintained selected-root transfer, auto-tracking and explicit replacement contract
  - resource: "file:///home/rektide/src/jj-waltz/tests/reseed.rs"
    title: Read tracked-to-payload, retained-secret and selected-root type-transition precedents; not executed
  - resource: "file:///home/rektide/src/jj-waltz/design/sync-scope/design0.gpt6a.md"
    title: Historical unaccepted broad-update proposal, read as alternatives rather than selected JW policy
extensions:
  ticket: diffdir-tree-transfer-policy
  reading_boundary: Withheld parent statement, full roadmap, parent transcript and concurrent Wave 1 outputs were not consulted; only the explicitly authorized completed native studies were read
verification:
  method: Current source and maintained specification inspection, prior-wave and authorized factual research reading, and document-only checks
  result: Own-session provenance, Markdown fence balance, whitespace, ten linked files/anchors and all frontmatter source paths checked successfully
  scope: No production changes, native-workspace mutation probes, new runtime tests, suites or performance campaigns; only the required owned-document VCS checkpoint
---

# Transfer authority without implied cleanup

**Copying source-present contents, removing target-only contents, and preserving a
region are three different grants.** “Mostly copy a directory” does not select
all three. Ordinary mirror, update and seed already supply useful coherent
operations; JW adds native revision/identity ownership and currently a narrower
selected-payload policy. Neither requires making ignoredness the universal future
ownership mechanism.

This is decision-equipping research, not a new default. The source is assumed
good. Concurrency and locking are out of band. Failure creates no effects history,
rollback or recovery guarantee. Existing JW's explicit out-of-band recovery
instructions are an application contract, not a generic transfer mechanism.

## 1. Policy axes and authority

Resolve these inputs once, then let pure policy consume the supplied facts:

| Axis | Actual question | Must not imply |
| --- | --- | --- |
| Participating region | Which logical destinations may this transfer address? | Unselected source means absent source. |
| Source-present work | Create missing destinations, refresh different entries, or keep existing entries? | Refresh means delete every extra. |
| Absence authority | Which genuinely target-only destinations may disappear? | A missing source root or unfinished producer is an empty authoritative tree. |
| Preservation/reservation | Which names/subtrees belong to another owner or must remain untouched? | Only source omission; target protection is equally necessary. |
| Desired shape and satisfaction | What file/link/directory is wanted, and what proves no install is needed? | Stat equality proves byte, mode, generated-parameter or CoW-sharing equality. |
| Transport | Ordinary copy, required reflink, or allowed fallback? | A particular overwrite/removal policy. |
| Classification boundary, for JW | Which pinned tracked trees and current ignore rules govern each phase? | Historical “initially untracked” is an enduring preservation right. |

Preservation ends descent for the whole pair. Container retention alone does not:
keeping `packages/` can still allow work in `packages/cache/`. Required ancestors
survive retained children. Structural root/routing validation remains necessary.
Core has no built-in `.jj`, `.git`, ignore or mount policy.

Current adapter composition is not an unoverrideable policy algebra:
`Steering::late` applies compatibility absence decisions before the caller's
visitor (`sync/src/visitor.rs:14–43`). A later visitor can revise those actions.
Use terminal early preservation for native exclusions; do not mistake a prior
late `retain` or `NeverRemove` projection for an immutable security boundary.

## 2. Generic physical-tree matrix — delivered behavior

Here **F** is a regular file, **L** a literal symlink, **D** a real directory,
and **—** genuine absence. These are child entries inside a valid binding, not
permission to replace the bound root. “Refresh” installs only when the delivered
comparison differs: F uses type/size/exact supported mtime; L uses literal native
link text. Equal leaves are not force-installed. A changed F↔L is a leaf refresh.

| Source / target | Mirror (`NoVisitor`) | Update (`Update` or unmodified `NeverRemove`) | Tree seed (`Seed`) |
| --- | --- | --- | --- |
| F or L / — | Install source leaf. | Same. | Same. |
| D / — | Create container; recursively apply policy. | Same. | Same. |
| F/F or L/L, equal predicate | No work. | No work. | Keep. |
| F/F or L/L, different; F/L; L/F | Refresh target leaf without following it. | Same. | Keep existing target leaf. |
| D / D | Keep target container; reconcile children. | Same; retain target-only subtrees. | Descend to seed missing children; keep existing leaves/collisions/extras. |
| — / F or L | Remove target leaf. | Preserve. | Preserve. |
| — / D | Recursively remove; remove container only if no retained descendants. | Preserve entire subtree. | Preserve entire subtree. |
| D / F or L | Prepare admitted source subtree, then replace target leaf. | Same. | Keep target leaf; no descent. |
| F or L / empty D | Locally check removability, prepare leaf, replace directory. | Same. | Keep target directory. |
| F or L / nonempty D | Replace only if every affected descendant is removable. | Refuse intact: descendants are retained by update policy. | Keep target directory, not a refusal. |
| Any / any at a preserved name or under a preserved ancestor | No copying, deletion, type change or descent, on either side. | Same. | Same. |
| Unsupported special kind on either admitted side | Refuse unless caller explicitly preserves the whole pair. | Preserve a target-only special; source-present special pairs otherwise refuse unless explicitly preserved. | A pair with an existing target is explicitly preserved by Seed; a source-only special refuses. |

The directory-to-leaf precheck protects the **whole affected old subtree** from
static preservation conflicts, not merely the retained descendant. It is not a
whole-operation transaction: prior batches can have changed, and later I/O can
fail after authorized removal begins. Leaf-to-directory preparation preserves
the old leaf if preparation fails. Symlink leaves are never traversed.

**Two seeds are intentionally different.** `Seed` quietly keeps existing target
leaves/collisions and descends paired directories. `seed_leaves` installs an
explicit known list into absent destinations and **errors** if a listed target
already exists; it does not discover unlisted inputs or authorize deletion. JW
tracked prefills use the latter, not the tree-seed collision behavior.

**Roots:** missing source is an error, never deletion authority. Default sync
permits an absent target with an existing real parent; an existing file/link
target root needs the explicit replacing binding. JW does not grant generic
replacement of its registered workspace root. Existing equal payload files are
not recopied merely because `ReflinkRequired` is selected; tracked CoW renewal
has a different, explicit create-only prefill purpose.

Evidence: [sync contract](/sync/README.md#visitor-steered-synchronization),
`sync/src/visitor.rs:46–104`, `src/visitor.rs:247–293`;
`sync/tests/visitor.rs:56–105` and `sync/tests/authority.rs:136–157` encode the
distinguishing seed and directory-to-leaf cases. Tests were read, not rerun.

## 3. JW source roles — archive normally, selected checkout actually supplies bytes

Use **A** for physical primary/archive, **S** for selected source checkout,
**T** for receiver, **Cs** for frozen source tracked tree, **B** for selected
target base/desired tracked tree, and **O** for target's post-snapshot old tree.
The new target working-copy commit is an empty child of B, not Cs's identity.

| Role | Current owner/input |
| --- | --- |
| Shared repository storage, verified physical primary | A, containing the real `.jj/repo`; not whichever workspace is named `default`. |
| Local ignored-copy list | `A/.jwreseed.local.toml`; CLI/global precedence still applies. Creation reads only the copy list, not maintenance source/freshening settings. |
| Actual tracked/payload donor | S: creation's current/`--from` context; reseed's CLI `--from` → primary-local source → verified A. Roots selected under S supply actual ignored bytes. |
| Desired tracked answer | B: normally source current commit, independently overridden by `--at`. `--source-at` is an assertion, not the target selector. |
| Target native identity | T's `.jj`, registration and optional private Git administration, supplied/preserved by native ownership. |
| Configured links | A separate link owner. Reseed uses verified primary config; creation currently resolves `inventory.default_name()`, which can be a named `default` distinct from A. Do not collapse this into the payload donor. |

The human's normal archive workflow is S=A. Exploring S as a registered workspace
does not require copying that workspace's native metadata or relocating policy
ownership. Current `add --from` changes source context; reseed resolves a donor
without changing caller role. No new source-resolution default is selected here.

If B differs from Cs, donor disk does not define the whole desired tree. The
current native phase clones only tracked paths equal between Cs and B, then JJ
materializes B. A future broad residual transfer must not copy donor tracked
paths intentionally absent in B back into T. Reserve the union of Cs and B
tracked **leaf/subtree coordinates**, while still admitting shared real directory
ancestors for unrelated auxiliary siblings. Old O alone must not permanently
reserve paths that are deliberately becoming new payload after native clearing.

Source coordinates: `workspace.rs:119–180,488–659`;
`lifecycle.rs:651–710`; `reseed/engine.rs:207–277,362–418,455–491,638–701`.
The completed [workspace-source study](/design/native/workspace-source0.gpt6am.md#source-binding-model)
supplies the separately bounded native evidence; this study performed no native
source/colocation experiments.

## 4. JW current product policy — not plain mirror, update or seed

Current JW combines native tracked replacement with a **source-discovered,
selected-root auxiliary mirror**. The entire command is not a root mirror.

| Case | New `add` / newly created `s` target | Existing `reseed` target, without `--replace-content` |
| --- | --- | --- |
| Tracked files in B | Native creation/materialization establishes B; optional tracked clone is an optimization with distinct state adoption. | O is saved by snapshot; tracked disk contents are cleared/refilled as a fresh child of B. Equal donor/B paths get create-only reflinks; JJ supplies other B paths. |
| Old tracked target leaf absent from B | Not applicable to new target. | Removed from working tree, retained in JJ history; not preserved merely because source lacks it. |
| Eligible selected source ignored D, target absent | Copy subtree using required private reflinks. | Create and mirror subtree. |
| Eligible selected source ignored D, target already exists | Refuse obstruction, including existing target D. This is stricter than generic Seed. | D/D mirrors recursively; target F/L is replaced with D without following links. |
| Changed source-present descendant inside selected root | Install as needed; normally destination is initially absent. | Refresh under generic mirror matrix, including allowed type transitions. Equal leaves keep inode/mtime; mode-only repair is not promised. |
| Target-only descendant inside selected root | No preexisting selected root is permitted. | Remove, subject to actual administrative/special/topology refusals. |
| Target-only whole root matching a pattern, but no eligible source root exists | No source root is selected or copied. | **Keep.** Discovery is source-only; the pattern is not standing absence authority. |
| Unselected ordinary/ignored contents | Not copied from donor by the auxiliary phase. | Keep remaining target data outside selected regions, except tracked clearing and separately configured links. Same-name donor data outside selection is not refreshed. |
| Routing ancestor of selected root | Must be real D if present; create missing real D. | Keep/create real routing D; refuse target file/link obstruction rather than replace unrelated local contents. Unselected siblings stay. |
| Source/desired tracked overlap with selected auxiliary root, in either prefix direction | Refuse. Selected root must be ignored under source and desired rules. | Same Cs/B exclusion; old O tracked contents becoming selected payload are supported after tracked clearing. |
| Existing `s` target | No creation/payload plan. | Switching is not reseeding; no implicit refresh. |

Current selection requires real source directories; it is not a selector for
arbitrary files/links. Once selected, descendants can be files, literal links or
directories. Ignore rules come from pinned **tracked** `.gitignore` bytes plus
frozen backend/global excludes, not a fresh search for untracked ignore files.
Being ignored does not make a previously tracked leaf untracked.

### Defined zero-auxiliary-copy outcomes

| Input | Current consequence |
| --- | --- |
| No effective list after CLI/local/global resolution | Empty auxiliary selection. |
| Missing local field/file | Inherit global, if any; otherwise empty. Missing local alone does not disable a nonempty global list. |
| Explicit empty list at the selected layer | Stop inheritance; copy no auxiliary data. |
| `--no-copy-ignored` | Disable even an explicit nonempty CLI list. |
| Patterns with no eligible source matches | Success with diagnostics, no selected roots; do not erase matching target-only roots. Nonignored matches are skipped, with eligible descendants still searchable. |

These outcomes do **not** disable tracked materialization, initial snapshots,
configured links, or an independently requested `--replace-content`. “Zero copy”
here means auxiliary transfer, not no effects by the entire command.

`--replace-content` is a distinct explicit **clear then refill** request. Current
code clears all remaining non-administrative target contents except `.jj`, then
refills B and the selected ignored roots. It does not broaden donor selection or
copy arbitrary source dotfiles. It is not a spelling for full donor mirror.

Evidence: `cow/transfer.rs:56–185`, `cow/ignored.rs:28–73,178–183,324–395`,
`lifecycle/ignored.rs:98–164,190–244`, `reseed/engine.rs:1081–1135,1436–1571`;
[maintained CLI contract](file:///home/rektide/src/jj-waltz/docs/reseed.md#ignored-data).
Read precedents include `tests/reseed.rs:1446–1491` (old tracked → payload),
`1595–1644` (retained obstructions), and `1913–1939` (target selected F/L → D).

## 5. Initial untracked → snapshot-tracked: a temporal classification, not a loophole

Consider this tiny existing receiver before reseed; B has no `notes.txt`:

```text
T/src/main.rs       already tracked feature work
T/notes.txt         new, nonignored, currently untracked, eligible for auto-track
T/.env              ignored, untracked secret
T/cache/old         ignored, untracked payload extra
S/cache/new         ignored, untracked donor payload
```

1. The command validates donor/target eligibility, then runs the initial target
   `status` snapshot (`engine.rs:380–397,833–837`). Unlike later verification,
   this snapshot does **not** set `snapshot.auto-track=none()`.
2. Under the example's JJ policy, `notes.txt` enters O. It is now tracked old
   target work, even though it was untracked at invocation. `.env` and `cache/old`
   remain untracked. This is conditional on actual native auto-tracking: ignored,
   excluded or otherwise ineligible data does not become tracked by assertion.
3. The bookmark guard evaluates the **post-snapshot** work tip. A bookmark only
   covering the former tip may no longer suffice. If coverage is missing, the
   command blocks before refill, but the snapshot already exists. JW creates or
   moves no bookmark for the guard.
4. After the operator supplies coverage or explicitly bypasses that guard,
   successful reseed to B removes `notes.txt` from disk as old tracked work.
   Its contents remain in JJ history. `.env` stays. With `cache` selected,
   `cache/old` disappears and `cache/new` arrives; with an empty/disabled auxiliary
   selection, the old cache stays instead. `--replace-content` would separately
   permit erasing `.env` too.

If auto-tracking is disabled initially, `notes.txt` remains untracked; absent a
selection, link operation or explicit clear it stays. If it obstructs a desired
tracked path, normal reseed refuses rather than silently overwriting it. Thus the
same on-disk entry can have different authorized outcomes under different native
snapshot policies. A broad promise that “all initially untracked data stays” is
**not** the delivered contract.

**Policy decision for this study:** retain the existing post-snapshot tracked
classification when describing current JW. Record the transition in explanation
and readiness reporting; do not add an initial-untracked exception or effects
history. A native snapshot saving content establishes recoverability through JJ,
not preservation of its filesystem name. Generic copy policies need no such fact.

If the human wants invocation-time untrackedness to be a new preservation promise,
that is an explicit fork: capture pre-snapshot tracked membership and relevant
present names, carry the protected set through **all native and auxiliary phases**,
and refuse any B/clear/type transition that would change a protected name. Decide
whether to prevent auto-tracking for those names or tolerate its native history
effect. Simply preserving them in the final diffdirs visitor is too late. This
adds real classification/enumeration work and can block ordinary reseed; it must
not be smuggled in as a “safer copy” default. It still requires no effects ledger.

Evidence: [current auto-tracking warning](file:///home/rektide/src/jj-waltz/docs/reseed.md#ignored-data),
`engine.rs:428–454,1030–1056,1452–1474,1574–1595`, and
`bookmarks.rs:94–100,187–215`. Later auto-track-disabled verification does not
retroactively undo the initial classification.

## 6. Native regions are symmetric reservations

The completed [metadata study](/design/native/metadata0.gpt6am.md#copy-boundary-new-versus-existing-receiver)
establishes this native ownership boundary; do not infer it from source shape:

| Source root entry / receiving entry | Authorized generic transfer result |
| --- | --- |
| `.jj` D in archive / `.jj` D containing receiver pointer and local state | Suppress source and preserve receiver subtree, not copy repository or donor identity. |
| Source `.jj` in a registered workspace / receiver `.jj` | Same; donor is not the repository owner and does not supply target identity. |
| Source `.git` D / receiver `.git` gitfile | Preserve whole pair despite type collision. |
| Source `.git` absent / receiver `.git` gitfile | Preserve receiver; absence grants no deletion. |
| Source `.git` gitfile / receiver `.git` absent | Do not copy donor-private admin reference into target. Native setup alone decides target colocation. |
| Either reserved root mapped/generated as a destination | Refuse unauthorized producer ownership before installation; physical source omission is insufficient. |

For new receivers, native creation supplies `.jj` and optional `.git` first;
ordinary transfer fills around them. Existing receivers retain their native
identity. Any explicit native cache adoption is separately authorized and is not
wholesale `.jj` copying. Nested `.jj`/`.git` in participating JW payload currently
**refuse**, rather than silently omitting metadata and copying a broken nested
checkout. Preservation of a whole nested checkout would be an explicit region
choice; generic library mode itself has no repository-name exclusions.

**Current colocation compatibility consequence:** `validate_target` rejects any
target root `.git` and external/owned Git metadata; `validate_identities` repeats
that check. `WorkspaceClear` preserves root `.jj` only and refuses `.git`, even
at root. Therefore merely changing `add` to default-on colocation makes those new
targets ineligible for current reseed. `--group all` includes such workers as
BLOCKED rather than automatically omitting them, so the whole refill can block.

The human's preferred default-on colocation remains conditional. The product
must either extend native reseed/clear and convergence support before changing
that default, or explicitly accept that reseed workers need `--no-colocate` for
now. No automatic conversion or silent opt-out is implied. Symmetric root `.git`
preservation is necessary but not sufficient: the metadata study's sparse-empty
Git-index finding leaves native convergence to the independent colocation study.
This return neither reads that concurrent output nor claims its feasibility.

Current code: `reseed/engine.rs:773–803,1364–1386`;
`cow/transfer.rs:165–185`. Current `CreationMode` is CoW/JJ-only by default and
rejects explicit positive CoW+colocation (`lifecycle.rs:126–151`). Existing switches
do not silently change a workspace's colocation mode.

## 7. Generated entries and path mappings: one destination needs one authority

This section states required policy decisions, **not delivered generated/mapped
installation support**. A generated result that is merely emitted to a consumer
needs no filesystem overwrite policy until an installer is selected.

Once installation is selected, use the physical matrix on the **resolved desired
destination shape**, with generated satisfaction supplied explicitly. Generated
presence means an admitted desired entry/recipe with frozen parameters; it is not
invented filesystem metadata. Parameter changes can require new work even when a
physical source/target pair compares equal. Unknown satisfaction is not equality.

| Collision / absence case | Policy obligation before conflicting effects |
| --- | --- |
| Physical `config.json` and generated `config.json` | Choose one declared owner, explicitly exclude the other, or refuse the conflict. Do not let enumeration/callback order pick a winner. |
| Inputs `a` and `b` both map to `out/x` | Establish an explicit merge/precedence producer or refuse ambiguity. Ordinary update authority does not choose between two sources. |
| One output is `out/x` leaf, another `out/x/y` | Resolve ancestor shape/ownership conflict; checking only duplicate exact destinations is insufficient. |
| Physical input maps to native/preserved destination | Enforce the destination reservation too; an innocuous source name cannot bypass `.jj`, `.git` or user preservation. |
| Generated source replaces nonempty target D | Apply the same retained-descendant blocker as physical directory-to-leaf replacement. Generated provenance grants no extra deletion right. |
| Mapped `a → out/b`, with no desired entry for `out/a` | Absence belongs to the declared **destination** namespace. Do not delete arbitrary input-shaped target names or names outside the mapped operation's owned output domain. |
| A producer is unfinished, skipped, disabled or produces no selected installation | Do not interpret this as authoritative desired absence and delete its old output. Removal requires a separately declared absence/clear policy. |
| Generated output lives inside scanned input | Reserve/exclude that destination or establish an explicit dependency rule so its previous output does not become another competing input. |

Consumer-declared collision resolution can be strict refusal, explicit producer
precedence, or an explicit combining producer. This study selects **no global
collision winner/default**. Unresolved ownership cannot authorize conflicting
writes. A target reservation must cover mapped routing ancestors and coupled
replacements as well as leaf names. A locally finalized physical batch alone
cannot prove that a later mapped producer will not collide; the binding/producer
composition must supply the relevant reservation or validation boundary, without
requiring an invented full-tree effects catalog.

JW's configured links are an existing narrow precedent: creation/reseed refuse
overlap with selected private payload and desired tracked contents, then the link
owner installs links separately. That is not acceptance of a universal “generated
wins” rule. A private source-equals-target link override is likewise explicit,
not precedence by arrival order (`lifecycle/ignored.rs:205–219`;
`reseed/engine.rs:1137–1178`).

## 8. A few meaningful configurations, not a flag cross-product

Keep operation selection and transport separate. These are semantic profiles,
not new CLI syntax or selected defaults:

| Configuration | Useful intent | Consequence requiring attention |
| --- | --- | --- |
| **Existing JW selected-payload lifecycle** | Preserve current native tracked refill plus explicitly selected ignored-root mirrors. | Same-name unselected donor files are not copied; vanished source roots do not prune target roots; initial snapshot classification still applies. |
| **Directory refresh with preserves** | Update source-present entries; retain extras; optionally name a small number of explicitly disposable mirror regions. | Same-name unpreserved local data is overwritten; nonempty directory-to-leaf changes can block. Extending a mirror declaration to a vanished source root requires explicit standing absence authority, unlike current source discovery. |
| **Explicit replica with preserves** | An ordinary disposable output root is genuinely meant to match the source. | Delete every unpreserved target-only entry in that bound domain. Not authorized just by “copy a directory,” and not equivalent to JW clear/refill. |
| **Initial seed** | Fill holes without overwriting existing data. | Existing collisions stay, so successful seed need not match source. Use listed strict seed when any existing destination should instead be an error. |

“Refresh with preserves” can bind an entire ordinary directory or explicitly
selected transfer roots. Root selection is useful where a donor contains much
more than should propagate; it need not retain the legacy two-sided-ignore gate.
Whole-directory refresh can be simple and useful without any native adapter.
For JW broad residual refresh, Cs/B and native/link ownership still apply first.

The earlier [JW authority proposal](file:///home/rektide/src/jj-waltz/design/sync-scope/design0.gpt6a.md#recommendation)
recommended broad update, explicit mirrors, and dropping two-sided ignoredness.
Those remain **unaccepted product alternatives**. Delivered generic mechanisms
and a bounded JW compatibility visitor do not accept that breadth or snapshot
policy. In particular `copy_ignored=[]`, disabled selection, and no effective
selection cannot silently acquire “copy everything” meaning. An explicitly
selected future model needs an explicit config migration or rejection of obsolete
inputs, not an accidental reinterpretation.

### Concrete human decisions still needed

1. **JW breadth:** keep selected roots, or select broad residual refresh? If broad,
   which local/control/link-owned destinations are explicitly reserved? Do not
   treat every dotfile as administrative, or silently start propagating secrets.
2. **Absence:** retain extras except within explicit mirrors, or choose an explicit
   whole-domain replica? Do mirror declarations stay authoritative when a complete
   source region vanishes? Current source-discovered selector says no.
3. **Snapshot promise:** keep post-snapshot tracked classification, or commission
   invocation-time untracked preservation across native phases? Separately, must
   newly transferred auxiliary data remain ineligible for future JJ snapshots?
   Dropping ignoredness changes that latter guarantee; it is not a copy-mode detail.
4. **Existing explicit clear:** if user preservation is introduced, should it also
   constrain `--replace-content`, or does that request deliberately relinquish it?
   State the contract; current clear knows only native `.jj` preservation.
5. **Colocation consumers:** support reseed for newly colocated receivers before a
   default change, or explicitly retain JJ-only reseed eligibility? This decision
   awaits native convergence evidence, not a filesystem transfer flag.
6. **Generated/mapped owners:** where two selected producers target one destination,
   should the consumer refuse, choose declared precedence, or name a combining
   producer? Selection belongs with that consumer, not generic traversal order.

No option above asks for automatic cleanup, default changes or implementation
authorization. In particular, neither broad donor copying nor initial-untracked
classification analysis selects a new deletion policy by implication.

## 9. Smallest implementation boundary and controlled next work

**No production change is needed to settle the matrix.** Existing generic
mirror/update/seed and pair-wide preservation already cover ordinary coincident
physical trees. JW can retain its current selected-root adapter until the human
selects a product delta. Keep native tracked materialization and source/policy
binding in JW, and filesystem installation/removal in sync.

The smallest next implementation, **after that selection**, is one command-owned
frozen policy compiled into the existing early/late visitor: participation and
terminal reservations first, source-present mode second, explicitly scoped absence
authority third, and routing/type-transition validation throughout. It need not
invent a rule interpreter, ownership database, new action registry, prune pass,
or a combinatorial collection of user-facing flags. Preserve current empty/disable
semantics on the legacy route until explicitly migrated. Future generated/mapped
work additionally needs destination ownership/satisfaction at its selected input
boundary; it cannot be injected by editing today's physical facts.

The smallest useful **future probe** is a policy model table plus one ordinary
directory pair, not a native feasibility campaign:

- `S/changed` versus different `T/changed`; `S/new`; `T/extra`.
- Source `shape` leaf versus `T/shape/keep` and `T/shape/stale`.
- Preserved target `.git` with source absent, and a separate source `.git` D
  versus target gitfile case. Include preserved `.jj` on both sides.
- One explicit mirror region existing only on T: this discriminates source-root
  discovery from standing destination absence authority.
- Pure desired-destination cases: physical/generated duplicate, two-to-one map,
  and mapped ancestor conflict; no filesystem production framework is required.

Assert the selected profile's outcomes, old `shape` subtree intact on a retained
descendant conflict, and zero namespace writes for preserved/equal regions. Keep
one tiny pair (at most a few dozen entries), literal expected bytes, no JJ commands
and no timing campaign. Existing tests already cover generic seed, preserved
directory replacement, and strict listed seeds; extend only the specific selected
delta instead of rerunning a full suite for research. A later JW regression can
check the worked snapshot scenario in a disposable repo **if separately authorized**;
it is not needed to infer current source semantics and was not run here.

## 10. Evidence and handoff

Current JW source is Git commit
`3b99f1d70a64c1251ccae9f4ca1942751671fe8a` in
`/home/rektide/src/jj-waltz`. Read-only `jj diff --stat` on inspected source,
README, maintained reseed guide and `tests/reseed.rs` reported zero changes from
that parent. Diffdirs prior-wave baseline is `1193a1ee`; this document changes no
library/source contract. Earlier studies' runtime results are attributed to them,
not reverified or generalized to other versions.

Doc-pass connections: this matrix answers the
[architecture transfer discriminator](/doc/architecture/README.md#delivered-seams-and-next-interface-discriminators),
grounds its preservation/type constraints in the [delivered sync policy](/sync/README.md#scope-and-absence-authority),
and joins the completed [native ownership](/design/native/metadata0.gpt6am.md)
and [source-role](/design/native/workspace-source0.gpt6am.md) findings without
claiming colocation convergence. It preserves the earlier authority proposal as
history while locating present behavior in current code and maintained CLI docs.

Only this study is committed. The scoped ticket receives its pointer, findings
and own-session provenance. Parent owns ticket closure/export, shared indexes,
cross-wave integration, and later comparison with withheld material. No scratch
runtime/model probe was necessary: the consequential current cases are explicit
in source, maintained contracts and existing read-only test precedents.
