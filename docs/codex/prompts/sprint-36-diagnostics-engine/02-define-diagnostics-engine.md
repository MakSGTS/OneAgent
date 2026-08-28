# Define Sprint 36 Diagnostics Engine

Continue OneAgent development.

## Reporting

- Prompt and repository artifacts: English.
- User-visible reports: Russian.

## Profile and template

- `docs/codex/profiles/architecture.md`
- `docs/codex/templates/architecture-task.md`

## Required workflow

`docs/codex/workflows/architecture.md`

## Authoritative documents

- `docs/Roadmap.md`
- `docs/Architecture.md`
- `docs/architecture/semantic-model-2.md`
- `docs/architecture/diagnostics-engine-investigation.md`
- `docs/adr/0008-semantic-model-2-knowledge-graph.md`
- `docs/adr/0024-reference-request-provenance.md`
- `docs/adr/0039-workspace-service.md`
- `docs/adr/0051-mcp-semantic-tools.md`
- `docs/adr/0054-lsp-adapter.md`
- `docs/reviews/v0.6-release-review.md`

## Prerequisite

Task 1 is committed and its investigation contains no blocking evidence gap or
unapproved production dependency.

## Task

Create `docs/adr/0058-diagnostics-engine.md` and synchronize only planning-level
architecture text required by the accepted decision. Implement no production
behavior.

## Required decisions

- Fix the first-slice diagnostic input families and retain Graph as authority
  for semantic facts, recoverable diagnostics, validation issues, provenance,
  source locations, and validation execution.
- Fix the source-independent diagnostic identity, normalized result fields,
  severity/category vocabulary, origin and location representation,
  deterministic equality/order, duplicate/collision handling, and stable public
  string contracts.
- Define orchestration admission and normalization, accepted static suppression
  policy, suppression evidence and counts, result bounds, truncation or
  fail-closed behavior, summaries, reports, and deterministic errors. Do not
  create a configurable Rules Engine.
- Assign ownership and dependencies across Graph, Analysis or another accepted
  domain owner, Workspace composition, cache/rebuild, Runtime, MCP, and LSP.
  Define immutable publication timing and repeated-build equivalence.
- Fix public API migrations and compatibility for raw `SemanticDiagnostic`
  access, graph validation, reports/diffs, Workspace consumers, cache schema,
  MCP tool schema/result, Tool Policy, LSP capability/result, and tests.
- Fix sensitive-data and confinement policy for messages, semantic identifiers,
  source locations, provenance, paths, candidates, and implicit errors.
- Define exact first-slice acceptance evidence: domain, orchestration,
  Workspace/cache/rebuild, MCP/LSP in-memory and public-process, compatibility,
  deterministic bounds, dependency, scope, and complete workspace matrices.
- Record rejected alternatives and defer configurable rule registration,
  third-party or scripted rules, new producers, diagnostics UI, push/workspace
  diagnostics, mutable documents, fixes/code actions, telemetry, remote
  transport, persistence of user preferences, and broad performance/security
  claims.

## Acceptance evidence

ADR-0058 is `Accepted`, maps every investigation question to one explicit
decision or deferral, assigns each production behavior to Tasks 3–7, identifies
all public consumers and migrations, preserves existing semantic/protocol
authority, introduces no dependency without approval, and agrees with the
Roadmap scope and Sprint 37 boundary.

## Excluded scope

Rust implementation, behavior-encoding fixtures or tests, Cargo changes, new
dependency approval, prompt-suite retirement, Sprint completion, and Rules
Engine implementation.

## Validation

Run ADR/investigation question coverage, input/identity/order/suppression/bound/
report consistency, ownership/dependency/API migration audits, MCP/LSP and
Sprint 37 scope checks, Markdown link checks, `git diff --check`, and
unrelated-change inspection.

## Suggested commit message

`Define Sprint 36 diagnostics engine`

## Final report additions

Report accepted inputs, identity, ordering, suppression, bounds, reporting,
ownership, compatibility/migration, sensitive-data, evidence, rejected
alternatives, deferred scope, and unchanged production behavior.
