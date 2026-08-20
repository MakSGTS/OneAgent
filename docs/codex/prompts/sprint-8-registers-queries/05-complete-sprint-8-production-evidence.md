# Task 05: Complete Sprint 8 production evidence

Continue OneAgent development.

## Reporting

- Communicate with the user in Russian.
- Keep code, identifiers, repository documentation, comments, Rustdoc, tests,
  errors, public APIs, prompt text, and commit messages in English.
- Report only live repository evidence or accepted architecture.

## Profile

`docs/codex/profiles/graph-implementation.md`

## Template

`docs/codex/templates/graph-emission-task.md`

Read the Profile, Template, their required Core and Workflow modules, and
`docs/codex/README.md` completely before acting.

## Authoritative documents

- `docs/Roadmap.md`, Sprint 8 Task 05 and completion criteria;
- `docs/architecture/semantic-model-2.md`;
- `docs/architecture/query-language-parser-investigation.md`;
- `docs/architecture/register-query-source-investigation.md`;
- `docs/adr/0017-depends-on-semantics.md`;
- `docs/adr/0021-reads-semantics.md`;
- `docs/adr/0024-reference-request-provenance.md`;
- `docs/adr/0030-register-query-semantics.md`.

## Required gate

Proceed only when Tasks 01–04 are committed in dependency order, ending with
`Emit Sprint 8 query data dependencies`, or current committed evidence proves
every task `already_complete`. All focused and full implementation validation
must be green and no task-created uncommitted change may remain apart from the
preserved prompt suite.

## Task

Add the smallest representative full-builder fixture and evidence matrix that
proves every accepted ADR-0030 criterion, then synchronize Coverage evidence
and current-state documentation without changing capability statuses or counts.

## Source contract and production source

Use reduced static Query declarations mapped to the confirmed Common Module
sources for `AccumulationRegister.InventoryCost` and
`AccountingRegister.FinancialAccounting`, their Configuration declarations,
and exact target descriptors. Preserve the existing Reads fixture as Catalog
and Information Register compatibility evidence where reuse is sufficient.

Every reduced artifact must have a manifest entry with origin path/range,
treatment, preserved qualified source/alias, target descriptor, and expected
request and edge results. Do not describe generated scaffolding as verbatim
source.

## Scope

Representative production integration, complete consumer evidence, Coverage
evidence synchronization, and current-state documentation.

## Included

- Add a minimal `adapters/edt/tests/fixtures/sprint8_registers_queries_project`
  or an equivalently named task-owned project proven by live conventions.
- Add one focused full-builder integration test module for the complete Sprint
  8 matrix.
- Prove Query identity/ownership, all four request target kinds, Reads and
  DependsOn projections, exact provenance, diagnostics, statistics, request
  reports and build Diff, graph validation, Query dependency/usage, reverse
  Impact, source-order independence, repeated builds, and zero unchanged graph
  and build-result diffs.
- Prove complete/incremental Semantic Index equivalence with a clean rebuild for
  the expanded edge set.
- Cover missing, ambiguous, incompatible, partial, duplicate, and parser-rejected
  paths without placeholders or partial edges.
- Verify Writes and every unrelated existing semantic slice remain compatible.
- Update representative Coverage tests/evidence and limitations for Reads,
  DependsOn, and ReferenceRequest only where complete production evidence
  justifies it.
- Recompute live registry aggregates and assert that statuses and counts remain
  unchanged.
- Synchronize `docs/architecture/semantic-model-2.md` and `docs/Roadmap.md` from
  accepted-future wording to exact implemented current state and remaining
  limitations.

## Excluded

- New capability IDs, required-evidence dimensions, status/count transitions,
  or closing a non-existent gap.
- Calculation Registers, virtual tables, JOIN/UNION/nesting/batches, broader
  grammar, new Query declaration sources, Query mutation, write-derived
  DependsOn, register payload/members, or placeholder targets.
- Editing `OneAgent_EDTproject/`, adding dependencies, serialization, Runtime,
  API transport, or later-sprint work.

## Acceptance criteria

- Every accepted family is proven through `FileSystemEdtSemanticGraphBuilder`,
  not only parser tests or manually inserted edges.
- Source manifests make reduction and provenance independently reviewable.
- Existing Catalog and Information Register Reads gain the accepted normalized
  dependency without identity or ownership changes.
- Both new register families resolve to exact top-level metadata nodes and emit
  one edge of each accepted kind.
- Negative and partial cases produce stable terminal requests and diagnostics
  with no resolved edge or placeholder.
- Reports and statistics derive canonical requests exactly once.
- Repeated and reordered builds are equal; complete/incremental results equal a
  clean rebuild.
- Coverage registries remain consistent, Supported/NotApplicable aggregates
  remain exactly live-baseline values, and no Critical/High/Medium gap appears.
- Roadmap and Semantic Model describe only executed production evidence and
  retain every deferred boundary.

## Repository Safety

- Recheck Git state, fixture conventions, current registry aggregates,
  representative tests, docs context, and applicable `AGENTS.md` before edits.
- Preserve unrelated work and never modify the real EDT source corpus.
- Do not stage or commit without explicit launching authorization.

## Task-specific validation

Run focused checks first:

```bash
cargo test -p oneagent-bsl query_language
cargo test -p oneagent-graph coverage
cargo test -p oneagent-edt reads
cargo test -p oneagent-edt coverage
```

Also run the new Sprint 8 integration test by its exact non-zero filter. Then
run the complete workspace validation from `docs/codex/core/validation.md`,
including `git diff --check`.

## Commit

When explicitly authorized, stage only the task-owned fixture, manifest,
integration tests, Coverage evidence, and synchronized current-state docs, then
create one commit:

```text
Complete Sprint 8 production evidence
```

Never stage the prompt suite, `docs/roadmap-calendar-forecast.md`, broad paths,
or an empty commit.

## Final report additions

Report fixture provenance and reductions, full production matrix, requests,
edges, diagnostics/statistics, Query/Diff/Impact/report/index evidence, Coverage
statuses and exact aggregates, synchronized docs, preserved behavior, files,
tests, validation, commit hash, final Git status, and the Task 06 review gate.
