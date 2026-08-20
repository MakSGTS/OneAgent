# Task 01: Define register Query graph rules

Continue OneAgent development.

## Reporting

- Communicate with the user in Russian.
- Keep code, identifiers, repository documentation, comments, Rustdoc, tests,
  errors, public APIs, prompt text, and commit messages in English.
- Report only live repository evidence or accepted architecture.

## Profile

`docs/codex/profiles/graph-implementation.md`

## Template

`docs/codex/templates/graph-model-task.md`

Read the Profile, Template, their required Core and Workflow modules, and
`docs/codex/README.md` completely before acting.

## Authoritative documents

- `docs/Roadmap.md`, Sprint 8 Task 01;
- `docs/architecture/semantic-model-2.md`;
- `docs/architecture/register-query-source-investigation.md`;
- `docs/adr/0006-semantic-graph.md`;
- `docs/adr/0017-depends-on-semantics.md`;
- `docs/adr/0021-reads-semantics.md`;
- `docs/adr/0030-register-query-semantics.md`.

## Required gate

Proceed only when the Sprint 8 investigation, accepted ADR-0030, Roadmap plan,
Semantic Model boundary, and complete prompt suite are committed as one
immutable planning baseline and Sprint 8 is the live target.

## Task

Implement only the source-independent endpoint and consumer rules required by
ADR-0030. Keep parser, resolver, request collection, EDT production emission,
fixtures, and Coverage transitions out of this task.

## Scope

One additive graph-domain contract with exhaustive endpoint and consumer
compatibility evidence.

## Included

- Allow `Query --Reads--> Metadata(AccumulationRegister)` and
  `Query --Reads--> Metadata(AccountingRegister)` in addition to the existing
  Catalog and Information Register targets.
- Allow `Query --DependsOn--> Metadata(Catalog | InformationRegister |
  AccumulationRegister | AccountingRegister)` in addition to existing member
  and Command dependency pairs.
- Reject every other source, target, reversed direction, Unknown, wildcard
  Metadata, member, flat semantic, placeholder, and missing endpoint.
- Prove generic edge filtering, direct dependency/usage results, reverse Impact
  with unique affected nodes and deterministic per-edge reasons, Diff, reports,
  Semantic Index, incremental-index clean-rebuild equivalence, and graph-domain
  Coverage declarations remain deterministic.
- Update public Rustdoc or current graph-domain documentation only where the
  accepted endpoint policy changes.

## Excluded

- New NodeKind, EdgeKind, MetadataKind, request category, identity, index
  dimension, serialization form, or dedicated Query API.
- Query-language parsing, metadata resolution, EDT source processing, edge
  production, diagnostics, statistics, fixtures, or Coverage status changes.
- Calculation Registers, virtual tables, write-derived DependsOn, and any
  wildcard endpoint rule.

## Acceptance criteria

- Existing edge machine identities and public construction APIs remain
  unchanged.
- Exhaustive positive tests cover all four Query Reads targets and all four
  Query DependsOn targets.
- Exhaustive negative tests cover every unrelated NodeKind and MetadataKind in
  both directions with deterministic validation issues.
- Existing Attribute, Dimension, Resource, and Command DependsOn matrices
  remain accepted exactly as before.
- Direct Query dependencies may contain distinct Reads and DependsOn relations
  to one target; filtering by either kind remains exact.
- Reverse Impact deduplicates the affected Query node while retaining stable
  reasons for both edges.
- Diff, reports, complete index, and incremental updates observe the endpoint
  contract without changing existing ordering.
- No parser or producer behavior changes and no capability status/count changes.

## Repository Safety

- Recheck `git status --short`, exact definitions, exhaustive consumers, tests,
  and applicable `AGENTS.md` before editing.
- Preserve unrelated work, including `docs/roadmap-calendar-forecast.md` when
  still user-owned and outside scope.
- Do not stage or commit unless the launching instruction explicitly authorizes
  it. Do not use broad staging or destructive Git commands.

## Task-specific validation

Run focused checks first:

```bash
cargo test -p oneagent-graph --test validation
cargo test -p oneagent-graph --test query
cargo test -p oneagent-graph --test impact
cargo test -p oneagent-graph --test coverage
```

Then run the complete workspace validation from
`docs/codex/core/validation.md`, including `git diff --check`. Report zero-match
filters separately.

## Commit

Commit only when the current launching instruction explicitly authorizes this
task or the Sprint 8 execution sequence. Stage only task-owned graph code,
tests, and necessary current-state documentation, then create one commit:

```text
Define Sprint 8 register query graph rules
```

Never stage the prompt suite, use broad staging, or create an empty commit.

## Final report additions

Report endpoint matrices, public API and identity impact, validation, Query,
Diff, Impact, report, index and Coverage-declaration behavior, files, tests,
exact validation results, commit hash, final Git status, and the Task 02 gate.
