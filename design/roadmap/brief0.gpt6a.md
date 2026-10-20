---
type: ResearchBrief
title: Independent architecture and mechanism questions
description: Dispatch-safe questions and constraints for research conducted independently of the parent's materialized architecture bet.
resource: /design/roadmap/brief0.gpt6a.md
status: completed-flight-brief
generated: { by: "model:openai/gpt-6-astra#high", at: 2026-10-20 }
sources:
  - resource: "opencode://ses_ea82cf969ffeAXqmUxHChTJar2"
    title: Human commissions independent research with early intent withheld; parent executes the flight and releases those stakes only for the completed final comparison
extensions:
  ticket: diffdir-tree
---

# Independent work brief

This brief governed the completed flight. Its reading boundary is preserved below
as provenance. The final comparison received the early statement after independent
returns; future commissions should state their own reading scope.

Read your ticket and the factual sources relevant to its question. Produce your
own answer from the code, native capabilities and explicit human constraints.

## Reading boundary

During independent research, **do not read, search, cite or evaluate**
`design/architecture/statement0.gpt6a.md`. Do not read other authors' same-wave
outputs. The parent statement is held for comparison after independent returns
are committed. Do not use the full roadmap as a substitute research prompt;
this brief plus the scoped ticket is sufficient. Earlier established specifications,
execution reports and implementation evidence remain available.

Research answers its question; it is not an exercise in confirming or refuting
the parent's proposed architecture. A different coherent result is useful.

## Human constraints and questions

- Wave 0 produces architectural statements: components, interfaces, ownership and
  end-to-end composition. Write architecture directly rather than an assessment
  report about a future architecture document.
- The system supports ordinary directory work. Determine which generic mechanisms
  and which consumer-specific behavior each scenario actually needs.
- Failure supplies no rollback/recovery guarantee or effects history. Explore
  serializable continuation, tips, retry and skip. Concurrent mutation/locking
  is out of band. Assume the source checkout is good.
- A cursor should be constructible from components. Determine a useful walking
  step/pause/resume boundary and how internal implementation state participates.
- Consider generated file contents, path mappings and parameterized actions.
  Determine whether existing action/disposition/observation mechanisms suffice
  and what extensions, if any, are needed.
- Identify when whole/subtree knowledge can eliminate work or simplify recursion.
  Distinguish assessment, enumeration and effects when explaining saved work.
- Investigate borrowed dirent/name/path representations, including zerocopy/native
  or unsafe options where useful. Establish actual ownership/layout costs.
- The normal JW source is the **JJ archive**. Determine actual archive versus
  receiving-workspace `.jj`/`.git` layouts and how creation/copying handles them.
- The preferred outcome is default-on Git colocation for `jw add`, with an opt-out.
  **Determine feasibility.** Report genuine limitations, relevant versions and
  concrete reasons it may not work or may cost too much. No preferred outcome
  needs to be forced past those findings.
- Separately explore a registered JJ workspace as source. Determine which archive
  assumptions hold, and whether existing `--from` support composes with the desired
  metadata and materialization behavior.

## Available factual entrypoints

- Root/sync specifications; `src/{listing,visitor,pair,walk,frontier}.rs`;
  `sync/src/{lib,apply,visitor}.rs`.
- `design/execution/flow0.gpt6a.md` and `state0.gpt6a.md`: prior source-grounded
  walkthroughs, to be checked against consequential current source claims.
- `examples/cursor/` and `design/cursor/implementation0.gpt61sxh.md`: completed
  example, with twelve reported focused tests. It is a specimen, not a required
  future architecture.
- JJ-waltz `src/{lifecycle,workspace,jj,ownership}.rs`, its actual CLI and tests.
- Native JJ/Git source or documentation and tiny isolated fixtures where native
  behavior is uncertain. Inspect capabilities rather than guessing by version.
- Kopacetic's rkyv API/implementation for codec-specific work.

## Return

Commit a model-suffixed document with your session provenance. State concrete
contracts or findings, inspected source coordinates, uncertainty and a smallest
discriminating experiment where needed. Separate observed results from proposed
mechanisms and preferred product outcomes. Update the scoped ticket's useful
pointers; expand its epic only where the evidence warrants it.

The later `diffdir-tree-compare` task receives the withheld statement and the
independent returns. It compares their actual conclusions and proposes amendments.
