# Implement Sprint 36 Diagnostic Orchestration

Continue OneAgent development.

## Reporting

- Prompt and repository artifacts: English.
- User-visible reports: Russian.

## Profile and template

- `docs/codex/profiles/diagnostics-engine-implementation.md`
- `docs/codex/templates/diagnostics-engine-task.md`

## Required workflow

- `docs/codex/workflows/implementation.md`
- `docs/codex/workflows/diagnostics-engine.md`

## Authoritative documents

- `docs/Roadmap.md`
- `docs/Architecture.md`
- `docs/architecture/semantic-model-2.md`
- `docs/architecture/diagnostics-engine-investigation.md`
- `docs/adr/0008-semantic-model-2-knowledge-graph.md`
- `docs/adr/0024-reference-request-provenance.md`
- `docs/adr/0058-diagnostics-engine.md`

## Prerequisite

Task 3 is committed, its complete validation passes, and the diagnostic domain
matches ADR-0058.

## Task

Implement the deterministic source-independent Diagnostics Engine over only the
ADR-0058-accepted existing input families. Keep Workspace and protocol
composition outside this task.

## Required behavior and evidence

- Accept the exact immutable input views selected by ADR-0058 and normalize
  recoverable semantic diagnostics and graph validation evidence without
  changing their canonical producers or executing hidden source reads.
- Apply accepted identity, duplicate/collision, severity/category,
  source-location, suppression, ordering, bounds, summary, report, and failure
  rules exactly once. Preserve evidence needed to explain each retained or
  suppressed result.
- Keep orchestration deterministic for arbitrary input order, repeated
  execution, duplicate provenance, missing optional location, and mixed
  diagnostic families. Do not infer absent graph/source facts.
- Fail or report partial/bounded outcomes exactly as ADR-0058 requires. Never
  silently drop a result unless the accepted suppression or bound contract
  records that outcome observably.
- Add focused unit and integration tests using repository-owned semantic
  diagnostic and graph-validation fixtures for empty, positive, mixed,
  duplicate, collision, suppression, missing-location, exact/over-bound,
  reordered, repeated, and invalid-input behavior.
- Preserve all existing raw diagnostic, validator, graph report, build diff,
  adapter, and Coverage tests. Do not create rule registration, plugin, script,
  or dynamic dispatch abstractions assigned to Sprint 37.

## Excluded scope

New diagnostic producers, parser/adapter changes, graph-semantic changes,
Workspace/cache integration, MCP/LSP changes, rule registry or third-party
rules, UI, source mutation, current-state docs, and Sprint completion.

## Validation

Run non-zero focused domain/orchestrator tests, existing Graph diagnostic,
validation, report, diff, reference, adapter, and Coverage regressions affected
by the input boundary, then the canonical full Rust workspace gate and
`git diff --check`.

## Suggested commit message

`Implement Sprint 36 diagnostic orchestration`

## Final report additions

Report accepted inputs and owners, orchestration pipeline, suppression/bound/
failure behavior, exact focused test names and counts, raw-producer
preservation, dependency/API impact, and full validation outcomes.
