---
prompt_contract: v2
task_kind: investigation
profile: docs/codex/profiles/investigation.md
template: docs/codex/templates/investigation-task.md
fresh_context: required
context_static_max_percent: 15
context_authorities_max_percent: 20
context_prework_hard_stop_percent: 50
context_working_min_percent: 35
context_reserve_min_percent: 15
---

# Investigate Sprint 40 Refactoring Planner

## Reporting

- Communicate with the user in Russian.
- Keep repository artifacts and the commit message in English.

## Context manifest

### Must read

- `AGENTS.md` — sections: repository scope, change discipline, validation, Git
  branch/release workflow, and GUI validation.
- `docs/Roadmap.md` — sections: task-template readiness row, v0.7 status table,
  and Sprint 40 Refactoring Planner execution plan.
- `docs/reviews/sprint-39-change-impact-analysis.md` — sections: Decision,
  Residual risks and Sprint 40 hand-off, and Prompt retirement.
- `docs/codex/profiles/investigation.md` and
  `docs/codex/templates/investigation-task.md` — complete selected contract.
- `docs/codex/workflows/refactoring-safe-edits.md` — sections: Authority and
  phase boundary through Preview, plus Boundary.
- `crates/common/src/source.rs` — symbols: `SourcePath`, `SourcePosition`,
  `SourceSpan`, and `SourceLocation`.
- `crates/graph/src/{node,provenance,query,impact}.rs` — symbols and direct
  consumers found by bounded query: `GraphNode`, `Provenance`,
  `SemanticGraphQuery`, and `SemanticImpactAnalyzer`.
- `adapters/edt/src/bsl_graph.rs` — symbols: `AnalyzedBslModule`,
  `analyze_module`, `declaration_location`, and provenance helpers.
- `adapters/designer-xml/src/{semantic_graph,module_reader}.rs` — symbols:
  module source evidence, `source_id`, `declaration_location`, and consumers.
- `apps/runtime/src/workspace/mod.rs` — symbols:
  `WorkspaceConfigurationSnapshot`, `WorkspaceSnapshot`, and publication/change
  impact accessors.
- repository tests and fixtures found by bounded queries for source location,
  provenance, paired EDT/Designer conformance, controlled changes, Workspace
  publications, MCP symbol/impact tools, ordering, bounds, and redaction.

### Lookup on demand

- `docs/Architecture.md` — trigger: an ownership term remains ambiguous;
  sections: current Intelligence boundary and Planned boundaries only.
- `docs/architecture/semantic-model-2.md` — trigger: target identity, provenance,
  resolution, impact, or source authority cannot be resolved from live code;
  sections: only matching current semantic-model headings.
- `docs/adr/0061-change-impact-analysis.md` and ADRs cited by live source —
  trigger: a current accepted compatibility or authority boundary needs exact
  decision text; sections: decision and deferred scope only.
- implementation history — trigger: current definitions and tests disagree;
  query: the smallest path-scoped `git log -S` or `git blame` range.

### Excluded from initial context

- complete Roadmap, Architecture, and semantic-model documents;
- unrelated historical ADRs, reviews, and prompt suites;
- complete generated projects, fixture corpora, and successful command logs;
- Sprint 41 transaction implementation design beyond hand-off constraints.

### Preflight

- Record effective window or `unknown`, telemetry or labelled estimate basis,
  admitted static and authority material, and `pass|warning|blocked`.
- Narrow source/test queries at warning and stop before substantive work at the
  hard limit.

## Prerequisites / required gate

- `HEAD` is the committed amended Sprint 40 planning baseline on
  `codex/v0.7-sprint-40`, descending from planning commit `2bc6afb7` and
  framework commit `5c273da1`.
- The worktree has no conflicting change and Sprint 40 is the unique `next`
  target.

## Task

Investigate the smallest repository-evidenced read-only Refactoring Planner
slice and create
`docs/architecture/refactoring-planner-investigation.md`. Record the confirmed
evidence and decision-ready questions; update only the Sprint 40 investigation
evidence subsection and its `next` to `active` state in `docs/Roadmap.md`.

## Investigation objective

Determine whether and how OneAgent can produce one complete deterministic
semantic refactoring plan and preview without reading or mutating source after
the accepted immutable input is captured.

## Questions to answer

- Which first refactoring family has complete semantic, source-location,
  provenance, conflict, and fixture evidence?
- Who owns targets, snapshot identity, source/version preconditions, plan and
  operation identity, ordering, duplicates, conflicts, preview, bounds, and
  failures?
- What Graph, adapter, Workspace, impact, diagnostics, Runtime, Tool Policy,
  MCP, client, cache, and compatibility boundaries are affected?
- Which evidence is absent, point-only, path-only, format-specific, sensitive,
  or insufficient for later mutation?
- What deterministic focused, cross-format, public-process, and full-validation
  oracles can prove the first slice?

## Evidence scope

- Definitions, constructors, consumers, public exports, tests, fixtures,
  Coverage state, Cargo boundaries, and current Git history only as triggered.
- Confirmed facts, accepted constraints, alternatives, unknowns, and external
  blockers must remain separate.

## Evidence sources / fixtures

- Paired EDT/Designer module fixtures and controlled source-change cases.
- Workspace service and MCP process fixtures that publish immutable semantic
  state.
- Synthetic unit graphs only as supplements to production adapter evidence.

## Scope

### Included

- The investigation artifact and bounded Roadmap evidence update.
- Non-mutating focused evidence commands with meaningful non-zero outcomes.

### Excluded

- ADR decisions, production implementation, dependency changes, source edits,
  transactions, protocol changes, and Sprint 41 design.

## Acceptance criteria

- The artifact inventories live owners, definitions, consumers, fixtures,
  tests, compatibility, data gaps, and exact oracles.
- It identifies a smallest coherent first slice and classifies any missing
  repository-implementable immutable source/version/range evidence as the
  internal prerequisite owned by Tasks 3–4, not as external missing data.
- It reports `SPRINT_BLOCKED_MISSING_DATA` only when the required source corpus
  or oracle is unavailable outside the repository, or no implementable
  repository-owned contract can close the evidence gap; the exact missing
  artifact and consequence are recorded.
- Every ADR-0063 question is decision-ready and no edit capability is claimed.

## Completion Criteria

- Evidence is reproducible from named paths, symbols, queries, and commands.
- Roadmap records only investigation evidence and the authorized Sprint 40
  transition from `next` to `active`; no later sprint state changes.

## Task-specific validation

- Run non-zero focused source/provenance, paired-adapter, Workspace, and MCP
  evidence tests selected from the live inventory; record exact counts.
- Validate changed Markdown links and run `git diff --check`.

## Suggested commit message

`Investigate Sprint 40 Refactoring Planner`

## Final report additions

- Report decision readiness, unknowns, exact focused evidence, and whether the
  data/oracle gate remains open.
