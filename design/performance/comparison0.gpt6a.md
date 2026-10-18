---
type: ResearchSynthesis
title: Comparing the independent optimization inventories
description: Agreements, different sequencing judgments, and a crosswalk for shape-changing experiments informed by Astra and Sol.
resource: /design/performance/comparison0.gpt6a.md
status: proposed-for-assessment
generated: { by: "model:openai/gpt-6-astra", at: 2026-10-18 }
sources:
  - resource: "opencode://ses_eb39c93e9ffeNju5a7XEsB3wGY"
    title: Compared independent inventories and experiment controls, then recorded selection of leaf-pipeline work and future streaming multi-destination traversal
  - resource: /design/performance/opportunities0.gpt6am.md
    title: Forty independently assessed candidates and the appended architectural-value experiment order
  - resource: /design/performance/opportunities0.sol61x.md
    title: Thirty-five candidates, source/work-removal prioritization, and ten Sol high experiment briefs
  - resource: /design/performance/inspection0.sol61x.md
    title: Exact-toolchain source and owned syscall probes qualifying path, descriptor, copying, and staging hypotheses
---

# Comparing the inventories

Both inventories are independent research, not implemented optimizations. Their
candidate sets overlap; forty plus thirty-five does not mean seventy-five
distinct proposals. This synthesis selects no production change and commissions
no builders.

## Strong agreement on the actual starting point

Both authors independently verified that configured Rust 1.101 nightly already
performs Linux entry metadata lookup relative to a directory descriptor.
`entry.path()` is nevertheless constructed eagerly by our inspection kernel.
Userspace allocation and kernel pathname resolution therefore need separate
experiments. A manual replacement for the same relative stat call is not the
promised benefit of a handle-oriented backend.

They also agree that ordinary File-to-File copying already uses kernel-assisted
copying, and Linux reflink-copy does not create a second random staging
directory. The actual sync opportunities include repeated metadata acquisition,
per-leaf staging and empty-directory cleanup, and clone descriptors closed before
our metadata finalization reopens them.

Sol contributes actual owned syscall/type probes to the source evidence; Astra
contributes an explicit distinction between redundancy, likely measured payoff,
and architectural leverage. Neither supplies measured candidate speedups.

Experimental `std::fs::Dir` is compatible with threaded ownership on the
configured nightly. Its inspected public API is not a complete traversal/link
backend: enumeration, literal link reading, and link creation still need an API
coverage decision. Availability is useful evidence, not a reason to commit to a
full backend rewrite.

## The sequencing difference is useful

Sol's SH-A/B/C starts with low-coupling work removal: lazy paths, selective
metadata, internal metadata handoff, and cheaper successful cleanup. This is a
good sequence for incremental throughput improvements and syscall attribution.

Astra's appended ranking asks a different question: which experiments should
precede decisions about public ownership and information flow? Its first wave
is donor reuse, descriptor-owned leaf preparation, and a private prepared mirror
path. This better matches the user's latest emphasis on **the shape of the
work**. Small controls should support that wave, not become a prerequisite
cleanup project.

## First-wave crosswalk

| Experiment | Astra starting point | Sol support | What the result can change |
| --- | --- | --- | --- |
| Stable donor reused across targets | C01 / E6 / rank 1 | M04 / SH-I | Whether a generation-scoped source session is useful; measure N=1/2/4 including construction and disposal before exposing a catalog API. |
| Descriptor-owned leaf preparation and staging | S02–S04 / E2 / rank 2 | A01/A02/A04 / SH-C/D/J | Whether the operation kernel should own creation through metadata and install, and whether staging belongs to a leaf or touched parent batch. |
| Private prepared mirror information flow | C02, S01/S06 / rank 3 | M02/M03 and SH-C's internal reuse control | Whether compare/apply should share inspected state and ancestor knowledge; public richer records are a later choice, not the experiment's premise. |

Keep the leaf backend identical in the ordinary versus prepared-mirror
comparison. Measure cleanup, descriptor retention, and staging variants
separately so a successful change is credited to the right mechanism. Donor
reuse begins with sequential destinations; concurrent target execution is a
separate variable. Each experiment needs an unchanged control rebuilt with the
same configured compiler.

If actual near-term use has only one target per donor generation, start with
leaf preparation and move donor reuse behind it. Future reuse is an opportunity,
not current workload evidence. A negative result that avoids a costly new API is
a useful outcome.

## Later decisions and ordinary tuning

The assessments align on the next consequential questions: a shared inspection
budget for wide directories, a read-only traversal contract without mutable
consumer barriers, and explicit descriptor lifetimes for remaining full-path
operations. They also align on the need to test small-K starvation separately
from simply increasing the budget.

Many remaining ideas can be tuned internally after those boundaries settle:
vector capacity, category validation representation, success-result aggregation,
candidate-search scratch, release flags, and exact work-grain thresholds. Lazy
path construction is a particularly useful small control for any FD experiment.
Selective metadata is less purely internal: omitting unnecessary inspections
can change which failures are observed, so its error contract needs a decision.

Read Astra's [ranked addendum](opportunities0.gpt6am.md#addendum--what-i-would-try-first-and-what-should-shape-the-project)
for decision branches, and Sol's [experiment briefs](opportunities0.sol61x.md#commissions)
for discriminating controls. At the return checkpoint, compare counted work,
timing, memory/FD costs, and semantic changes before combining results. Do not
add independent speedup estimates together. Live opencode paths remain read-only;
sync experiments use separately owned disposable fixtures.

## Subsequent user selection

The user selected **leaf-pipeline optimization (#2)** for a Sol xhigh builder.
Descriptor ownership and staging variants are being tested against the existing
operation contract. The broader prepared-mirror proposal (#3) remains unselected:
duplicated work is evidenced, but useful end-to-end speedup from carrying
comparison information into application is not yet established.

Multiple destinations are recorded in the [README's future enhancements](../../README.md#coordinated-multiple-destinations).
The user's direction is an ordered, coordinated **streaming** walk of the source
and several destinations, sharing current source observations. A whole-source
catalog is not the selected design or a prerequisite. This refines the donor
reuse opportunity while preserving the independent catalog proposals above as
research alternatives.
