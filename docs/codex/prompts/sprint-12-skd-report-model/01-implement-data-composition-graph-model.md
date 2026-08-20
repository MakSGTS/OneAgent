# Task 01: Implement the Data Composition graph model

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

Read the Profile, Template, required Core and Workflow modules, and
`docs/codex/README.md` completely before acting.

## Authoritative documents

- `docs/Roadmap.md`, Sprint 12 Task 01;
- `docs/architecture/report-data-composition-source-investigation.md`;
- `docs/architecture/semantic-model-2.md`;
- `docs/adr/0006-semantic-graph.md`;
- `docs/adr/0021-reads-semantics.md`;
- `docs/adr/0023-typed-metadata-payload.md`;
- `docs/adr/0030-register-query-semantics.md`;
- `docs/adr/0034-report-data-composition-semantics.md`.

## Required gate

Proceed only when ADR-0034, the source investigation, Sprint 12 Roadmap plan,
Semantic Model planning text, and complete prompt suite are one committed
immutable planning baseline and Sprint 12 is the unique live target.

## Task

Implement only the source-independent nodes, typed payloads, collision-safe
identities, immediate ownership, validation, generic consumers, indexes, and
graph-domain Coverage accepted by ADR-0034. Keep EDT parsing and production
emission outside this task.

## Scope

One coherent public graph-model boundary making Report Data Composition facts
typed, validatable, queryable, diffable, and deterministic before a producer can
emit them.

## Included

- Add `NodeKind::DataCompositionSchema`, `NodeKind::DataSet`, and
  `NodeKind::DataCompositionField` with stable machine codes without changing
  existing codes or identities.
- Add closed compatible payloads for Schema main role, `DataSetKind` plus
  optional local data source, and Field data path.
- Enforce Query/Object versus Union data-source invariants through constructors.
- Add collision-safe owner-scoped identity helpers for direct Data Set, Field,
  and fixed-role metadata-owned Query IDs; preserve Schema UUID identity.
- Extend Contains only for Report-to-Schema, Schema-to-DataSet,
  DataSet-to-Field, and DataSet-to-Query while preserving Procedure/Function
  Query ownership.
- Require exactly one immediate owner for every new node and metadata-owned
  Query; preserve deterministic cycle and issue ordering.
- Update every exhaustive `NodeKind` and `GraphNodePayload` consumer with
  explicit behavior rather than wildcarding away new semantics.
- Prove equality, payload/content-only Diff, generic Query, Validation, reports,
  Impact exclusion, complete index, incremental index, Coverage enumeration,
  and stable ordering.
- Update public Rustdoc only where the public model changes contracts.

## Excluded

- EDT Report/DCS XML parsing, directory discovery, graph insertion, provenance
  producers, diagnostics, statistics, fixtures, or EDT Coverage.
- Query-language parsing, QuerySource requests, Reads, DependsOn, References,
  nested Union entities, field folders, or specialized query APIs.
- Serialization, persistence, Runtime, API, CLI, Designer XML, or unrelated
  graph refactoring.

## Acceptance criteria

- Existing NodeKind codes, identities, constructors, Query owners, equality,
  ordering, Query, Diff, Impact, Validation, report, and index behavior remain
  compatible.
- Each new payload accepts only its exact node kind; wrong-kind payloads and
  invalid Query/Object/Union data-source combinations fail deterministically.
- Main flag, data-set kind/source, and field-path changes preserve node IDs and
  report one modified semantic-content node rather than removal/addition.
- Delimiter-containing owner/local names do not collide; source order and
  content never enter IDs.
- Exactly the four ADR-0034 Contains pairs validate; reversed, transitive,
  unrelated, other-Metadata, Unknown, and missing pairs fail.
- Existing Procedure/Function Query ownership remains valid, while every Query
  still has one owner and new ownership cycles/multiple owners are rejected.
- New nodes and ownership are visible in generic Query, Diff, report, complete
  index, and incremental index; Contains creates no dependency/Impact reason.
- Graph-domain Coverage evidence and derived counts match executable registry
  state; EDT production/Coverage does not change.
- Full workspace validation succeeds.

## Repository Safety

- Recheck Git state, public enum/payload/identity consumers, graph tests,
  indexes, Coverage, and applicable `AGENTS.md` before editing.
- Preserve unrelated user files and the committed prompt suites.
- Do not add dependencies or modify EDT production behavior.
- Do not stage or commit without launch-time authorization; never use broad
  staging or destructive Git commands.

## Task-specific validation

```bash
cargo test -p oneagent-metadata
cargo test -p oneagent-graph
```

Then run the complete workspace validation from
`docs/codex/core/validation.md`, including `git diff --check`. Report zero-match
filters separately.

## Suggested commit message

```text
Implement Sprint 12 data composition graph model
```

When authorized, stage only task-owned metadata/graph code, public exports,
affected exhaustive consumer updates, focused tests, graph Coverage evidence,
and necessary Rustdoc. Do not stage EDT parser/production code, planning prompts,
or unrelated paths; do not create an empty commit.

## Final report additions

Report node/payload APIs, identity encoding and collision evidence, Contains and
ownership matrices, public API migration, Query/Diff/Validation/index behavior,
Impact exclusion, graph Coverage state/counts, files/tests, validation, commit
hash, final Git status, and the Task 02 gate.
