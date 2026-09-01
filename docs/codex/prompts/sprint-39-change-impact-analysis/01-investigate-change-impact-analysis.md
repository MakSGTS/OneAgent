# Investigate Sprint 39 Change Impact Analysis

Continue OneAgent development.

## Reporting

- Prompt and repository artifacts: English.
- User-visible reports: Russian.

## Profile and template

- `docs/codex/profiles/investigation.md`
- `docs/codex/templates/investigation-task.md`

## Authoritative documents and evidence

- `docs/Roadmap.md`
- `docs/Architecture.md`
- `docs/architecture/semantic-model-2.md`
- `docs/architecture/incremental-index-consumer-integration.md`
- `docs/architecture/mcp-semantic-tools-investigation.md`
- `docs/architecture/git-change-adapter-investigation.md`
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
- repository Git history and Graph diff/impact, Workspace snapshot/update/cache,
  filesystem/Git input, Runtime/MCP, fixture, test, public-process, dependency,
  and consumer code

## Prerequisites / required gate

- The committed Sprint 39 planning baseline is an ancestor of HEAD.
- Sprint 38 is completed and Sprint 39 is the unique eligible target.
- The working tree has no uncommitted task-created changes.

## Task

Create `docs/architecture/change-impact-analysis-investigation.md` and update
only the Sprint 39 Roadmap state needed to record Task 1 start. Produce
decision-ready repository evidence for ADR-0061 without production
implementation.

## Questions and required evidence

- Inventory `SemanticGraphDiff`, `SemanticGraphBuildDiff`,
  `SemanticImpactAnalyzer`, options, seeds, reasons, result ordering, summaries,
  errors, bounds, Coverage, production adapter evidence, and every public or
  internal consumer. Separate canonical Graph behavior from candidate product
  orchestration.
- Trace complete Workspace Configuration identity, immutable snapshot
  construction, initial publication, replacement, equal rebuild, failed build,
  last-valid retention, recovery, watcher/Git input coalescing, cache warm/cold
  reconstruction, cancellation, shutdown, and observer behavior.
- Confirm that no product report currently compares successive publications.
  Characterize the existing `oneagent.impact` contract that compares distinct
  Configuration IDs inside one immutable snapshot, including schema, Tool
  Policy, projection, truncation, errors, process behavior, clients, and
  compatibility constraints.
- Define decision-ready alternatives for owner and dependency direction;
  previous/current publication identity; Configuration matching, addition,
  removal, unchanged/equal rebuild, source-format or canonical-ID change;
  initial/warm startup; failed attempts; and retention lifetime.
- Define report identity, canonical inputs, affected item vocabulary, reasons,
  availability, ordering, exact duplicates, conflicts, completeness,
  truncation/omission, summaries, bounds, closed failures, redaction, and
  sensitive-data alternatives without duplicating Graph semantics.
- Determine whether the report is stored in the current Workspace snapshot,
  observed separately, computed on demand, serialized, or deterministically
  recomputed. Record cache-version, semantic-compatibility, stale-result,
  concurrency, atomicity, lifecycle, and memory implications for every viable
  first-slice family.
- Define a filesystem-input/Git-input equivalence oracle that compares only
  complete previous/current semantic graphs and canonical diffs. Prove why
  repository paths, statuses, baselines, and completeness cannot enter impact
  identity, seeds, reasons, summaries, persistence, or public output.
- Define compatible MCP alternatives for the existing tool's inputs, result,
  bounds, errors, catalog/capability, policy request, immutable process mode,
  and public clients. Identify any migration that cannot be additive.
- Build deterministic positive, empty, added/removed Configuration, unchanged,
  reordered, repeated, exact/over-bound, conflicting/missing, failure/recovery,
  warm/cold cache, cancellation/shutdown, public-process, sensitive-data, and
  compatibility test matrices. Record exact likely production/test areas and
  every ADR-0061 decision.
- Keep selective/incremental rebuilding, new graph facts or impact relations,
  diagnostics/rules, risk scores, refactoring, code actions, edits,
  transactions, rollback, Git mutation/remote access, new HTTP/CLI/LSP/IDE UI,
  telemetry, and broad performance/security claims deferred.

## Excluded scope

Architecture acceptance, Rust or Cargo changes, impact-report implementation,
Workspace/MCP migration, production configuration, new protocol/IDE surface,
Coverage transition, refactoring/edit behavior, review artifacts, prompt-suite
retirement, and Sprint completion.

## Validation

Run focused non-mutating source/API/consumer/history/dependency/test/fixture
inventories and existing Graph impact, Workspace, watching, Git-to-Workspace,
MCP semantic/public-process, cache, and Runtime tests needed to confirm the
baseline. Validate Markdown links, `git diff --check`, Roadmap state, and
unrelated-change absence. Record exact commands, non-zero matched counts,
zero-test targets, failed probes, and inconclusive evidence separately.

## Suggested commit message

`Investigate Sprint 39 Change Impact Analysis`

## Final report additions

Report Graph and product authorities, current consumers, snapshot/configuration
identity alternatives, report/completeness/bound/failure questions,
cache/lifecycle options, filesystem/Git equivalence, MCP compatibility,
deterministic matrices, unresolved ADR decisions, exact affected areas, and
unchanged production behavior.
