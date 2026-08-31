# Define Sprint 39 Change Impact Analysis

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
- `docs/architecture/change-impact-analysis-investigation.md`
- `docs/architecture/incremental-index-consumer-integration.md`
- `docs/architecture/mcp-semantic-tools-investigation.md`
- `docs/architecture/git-change-adapter-evidence.md`
- `docs/adr/0017-depends-on-semantics.md`
- `docs/adr/0027-incremental-semantic-index-maintenance.md`
- `docs/adr/0039-workspace-service.md`
- `docs/adr/0040-graph-query-api.md`
- `docs/adr/0041-file-watching.md`
- `docs/adr/0042-persistent-cache.md`
- `docs/adr/0049-tool-execution-policy.md`
- `docs/adr/0051-mcp-semantic-tools.md`
- `docs/adr/0060-git-change-adapter.md`
- `docs/reviews/sprint-29-mcp-semantic-tools.md`
- `docs/reviews/sprint-38-git-change-adapter.md`

## Prerequisite

Task 1 is committed and the investigation contains no blocking evidence gap or
unapproved production dependency.

## Task

Create `docs/adr/0061-change-impact-analysis.md` and synchronize only
planning-level architecture text required by the accepted decision. Implement
no production behavior.

## Required decisions

- Fix the bounded first-slice product workflow owner, dependency direction,
  canonical inputs, and explicit rule that Graph remains the sole semantic,
  diff, dependency, propagation, and impact authority.
- Define complete previous/current publication and Configuration identity,
  matching, addition/removal/unchanged/equal rebuild, source-format or identity
  transition, initial/warm startup, failed attempt, recovery, and retention
  behavior.
- Define immutable report identity and content, direct/transitive status,
  availability, reasons, Graph result reuse, duplicates/conflicts, canonical
  total order, completeness vocabulary, summaries, checked arithmetic, bounds,
  omission/truncation, closed redacted failures, and sensitive-data policy.
- Select exact computation, storage, observation, and lifecycle behavior from
  investigation evidence. Define atomic publication, concurrency,
  cancellation, shutdown, repeated service, stale-result, and resource
  ownership without hidden tasks or mutable global state.
- Decide cache and persistence behavior explicitly: serialized or recomputed
  derived evidence, schema/semantic compatibility, warm/cold equivalence,
  corruption/recovery, and behavior when no previous publication exists.
- Define the filesystem/Git change-input equivalence contract over complete
  semantic end states. Forbid repository path, status, baseline, completeness,
  operation order, or source identity from entering impact seeds, reasons,
  Configuration matching, canonical results, persistence, or wire output.
- Fix the accepted compatible MCP `oneagent.impact` request/result/error,
  catalog/capability, bounds, truncation/completeness, Tool Policy, immutable
  process, public-process, and supported-client behavior. Identify exact
  migrations and preserve unsupported behavior explicitly.
- Preserve complete production discovery/build/validation, immutable
  last-valid Workspace publication, cache recovery, Graph/Analysis/Diagnostics/
  Rules ownership, protocol revision, Tool Policy, source confinement,
  supported consumers, and Coverage unless one exact accepted migration is
  required.
- Fix repository-owned acceptance evidence for domain, Graph equivalence,
  Configuration matching, report identity/order/completeness/summary/bounds,
  Workspace publication/failure/recovery/cache/lifecycle, filesystem/Git
  equivalence, MCP schema/policy/process, compatibility, sensitive data, scope,
  and full validation.
- Record rejected alternatives and defer selective/incremental semantic
  rebuilding, new graph facts or impact relations, diagnostics/rules, scoring,
  risk prediction, refactoring, source edits, transactions, rollback, Git
  mutation/remote access, new product UI, telemetry, benchmarks, and broad
  performance/security claims.

## Acceptance Criteria

ADR-0061 is `Accepted`, maps every investigation question to one explicit
decision or deferral, assigns Tasks 3–6, identifies public consumers and any
migration, preserves accepted authority, introduces no dependency without
approval, and agrees with the Roadmap and Sprint 40 boundary.

## Excluded scope

Rust implementation, behavior-encoding fixtures or tests, Cargo changes,
unapproved dependency use, source or repository mutation, review artifacts,
prompt-suite retirement, Sprint completion, refactoring planning, code actions,
source edits, transactions, rollback, and product UI.

## Validation

Run investigation-question coverage; authority/input/identity/report/order/
completeness/summary/bound/failure/snapshot/cache/lifecycle/protocol/policy/
consumer consistency; sensitive-data and deferred-scope audits; Markdown link
checks; `git diff --check`; and unrelated-change inspection.

## Suggested commit message

`Define Sprint 39 Change Impact Analysis`

## Final report additions

Report accepted authority and inputs, publication/Configuration identity,
report vocabulary/order/completeness/bounds/failures, snapshot/cache/lifecycle,
filesystem/Git equivalence, MCP compatibility and Tool Policy, evidence,
rejected alternatives, deferred scope, and unchanged production behavior.
