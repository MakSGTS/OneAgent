# Integrate Sprint 39 Workspace Impact Snapshots

Continue OneAgent development.

## Reporting

- Prompt and repository artifacts: English.
- User-visible reports: Russian.

## Profiles and template

- `docs/codex/profiles/diagnostics-engine-implementation.md`
- `docs/codex/profiles/runtime-service-implementation.md`
- `docs/codex/templates/diagnostics-engine-task.md`

## Required workflows

- `docs/codex/workflows/implementation.md`
- `docs/codex/workflows/diagnostics-engine.md`
- `docs/codex/workflows/runtime-service.md`
- `docs/codex/workflows/persistent-state.md` only if ADR-0061 accepts a
  persisted impact representation or cache schema/compatibility migration

## Authoritative documents

- `docs/Roadmap.md`
- `docs/Architecture.md`
- `docs/architecture/semantic-model-2.md`
- `docs/architecture/change-impact-analysis-investigation.md`
- `docs/architecture/git-change-adapter-evidence.md`
- `docs/adr/0027-incremental-semantic-index-maintenance.md`
- `docs/adr/0037-runtime-service-container.md`
- `docs/adr/0039-workspace-service.md`
- `docs/adr/0041-file-watching.md`
- `docs/adr/0042-persistent-cache.md`
- `docs/adr/0060-git-change-adapter.md`
- `docs/adr/0061-change-impact-analysis.md`

## Prerequisite

Task 3 is committed, its complete validation passes, and the Change Impact
report domain matches ADR-0061.

## Task

Integrate the accepted Change Impact report into the ADR-0061 Workspace
complete-publication lifecycle. Implement only accepted previous/current
snapshot composition, cache behavior, and source-neutral rebuild integration.
Do not change MCP yet.

## Required behavior and evidence

- Compose the report only from complete validated previous/current semantic
  Configuration graphs and the Task 3 evaluator. Preserve Graph authority and
  do not admit filesystem or Git path/status evidence into semantic inputs.
- Implement the accepted Configuration matching, addition/removal/unchanged/
  equal rebuild, identity or source-format transition, initial startup, warm
  startup, successful replacement, failed attempt, recovery, and retention
  behavior exactly.
- Construct and validate all accepted derived evidence before one atomic
  publication. A failed build or impact evaluation must follow ADR-0061's
  last-valid/failure policy and must never publish a partial graph/report pair.
- Preserve complete discovery, EDT/Designer builds, Graph validation,
  diagnostics/rules composition, stable source rescans, cache write order,
  watcher/Git input coalescing, and immutable observer behavior.
- Implement ADR-0061 cache behavior exactly. If the report is recomputed, prove
  warm/cold canonical equality and unchanged schema claims. If persisted,
  implement only the accepted schema/semantic-compatibility migration,
  corruption handling, invalidation, and clean rebuild equivalence.
- Prove equivalent complete semantic end states reached through filesystem
  observation and accepted Git change input publish equal Change Impact reports
  regardless of input or operation order. Empty/equal rebuilds follow the ADR
  and never use repository evidence as a semantic shortcut.
- Keep every blocking build, report computation, source observer, channel,
  cancellation handle, and long-lived resource under its accepted owner.
  Prove cancellation during active work, source failure, impact failure,
  shutdown ordering, observer/input closure, cleanup, and repeated fresh service
  runs with deterministic synchronization rather than arbitrary sleeps.
- Add production-entry-point evidence over disposable tracked EDT and Designer
  fixtures for accepted semantic modifications, additions/removals,
  Configuration transitions, invalid build/recovery, equal rebuild,
  burst/follow-up, warm/cold cache, filesystem/Git equivalence, and unchanged
  Graph Query/diagnostic/rule consumers.
- Preserve current HTTP/CLI/MCP/LSP/VS Code/EDT behavior, protocol catalog,
  Tool Policy, source confinement, public errors, and Coverage until Task 5.

## Excluded scope

MCP schema/handler changes, new protocol or IDE surfaces, selective/incremental
Graph mutation, repository-path impact seeding, new source parsers or graph
facts, diagnostics/rules, scoring/risk prediction, refactoring, source edits,
transactions, current-state documentation, and Sprint completion.

## Validation

Run non-zero focused publication/Configuration-matching/report-composition/
equal-rebuild/failure/recovery/cache/watching/Git-equivalence/lifecycle/
cancellation/shutdown/repetition tests; affected production Workspace,
watching, cache, Graph Query, diagnostics/rules, HTTP/CLI/MCP/LSP and adapter or
public-process compatibility tests; then the canonical full Rust workspace gate
and `git diff --check`.

## Suggested commit message

`Integrate Sprint 39 Workspace impact snapshots`

## Final report additions

Report Workspace/report ownership and API, previous/current and Configuration
matching, atomic publication, failure/recovery, cache and lifecycle behavior,
filesystem/Git equivalence, cancellation/cleanup, consumer compatibility,
exact focused/public counts, preserved Graph authority, schema/API/dependency
impact, and full validation.
