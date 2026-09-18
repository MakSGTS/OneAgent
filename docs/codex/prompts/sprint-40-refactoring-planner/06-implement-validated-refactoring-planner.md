---
prompt_contract: v2
task_kind: implementation
profile: docs/codex/profiles/refactoring-safe-edits-implementation.md
template: docs/codex/templates/refactoring-safe-edits-task.md
fresh_context: required
context_static_max_percent: 15
context_authorities_max_percent: 20
context_prework_hard_stop_percent: 50
context_working_min_percent: 35
context_reserve_min_percent: 15
---

# Implement Sprint 40 Validated Refactoring Planner

## Reporting

- Communicate with the user in Russian.
- Keep code, APIs, tests, docs, errors, and the commit message in English.

## Context manifest

### Must read

- `AGENTS.md` — sections: repository scope, change discipline, validation, and
  Git branch/release workflow.
- `docs/adr/0063-refactoring-planner.md` — sections: canonical inputs, first
  refactoring family, planner evaluation, conflicts, completeness, bounds,
  failures, preview, and evidence matrix.
- `docs/Roadmap.md` — sections: Sprint 40 exclusions and Task 6.
- `docs/codex/profiles/refactoring-safe-edits-implementation.md`,
  `docs/codex/templates/refactoring-safe-edits-task.md`, and
  `docs/codex/workflows/refactoring-safe-edits.md` — complete selected contract.
- `crates/analysis/src/refactoring.rs` and
  `crates/analysis/tests/refactoring_plan.rs` — complete committed Task 5 domain
  and focused evidence.
- `crates/graph/src/{query,node,provenance,kind}.rs` — symbols cited by ADR-0063
  plus direct consumers found by bounded query.
- repository production-adapter tests named by ADR-0063 as the deterministic
  source/provenance oracle; load only named test functions and fixtures.

### Lookup on demand

- `crates/graph/src/{impact,resolution}.rs` — trigger: ADR-0063 explicitly reuses
  an accepted Graph algorithm; symbols: only the accepted API and errors.
- adapter implementation — trigger: a named production fixture contradicts its
  asserted provenance/location behavior; paths and functions from the failure.
- `docs/architecture/refactoring-planner-investigation.md` — trigger: ADR-0063
  refers to an evidence-table row without restating its path or oracle.

### Excluded from initial context

- complete adapters, Runtime, MCP, and client code;
- unrelated Graph algorithms and historical implementation diffs;
- full fixture corpora and successful logs;
- source reads/writes, Workspace composition, public projection, and Sprint 41.

### Preflight

- Record effective window or `unknown`, measurement basis, admitted material,
  and `pass|warning|blocked` before implementation.
- Narrow Graph/fixture selectors at warning and stop at the hard limit.

## Prerequisites / required gate

- `HEAD` is exactly the committed Task 5 result with subject
  `Implement Sprint 40 refactoring plan domain`.
- The accepted domain passes its unfiltered focused target and the worktree is
  clean.

## Task

Implement the ADR-0063 deterministic Graph-backed planner evaluation and
read-only preview for the accepted first refactoring family.

## Scope

### Included

- Exact target/kind/name and immutable precondition validation.
- Accepted source-evidence admission, canonical operations/dependencies/order,
  duplicate/overlap/conflict behavior, bounds, summaries, cancellation, closed
  failures, deterministic preview, and focused production-fixture evidence.

### Excluded

- Source or filesystem access, source edits, transactions, Workspace/Runtime,
  protocol/policy/client changes, persistence, scoring, and unsupported kinds.

## Acceptance criteria

- Equivalent Graph/provenance inputs produce equal plans and previews across
  reorder and repeated fresh evaluation.
- Missing, ambiguous, conflicting, stale, incomplete, incompatible, cancelled,
  and out-of-bound inputs fail atomically with no partial plan.
- Accepted production adapter fixtures prove the supported source evidence; a
  synthetic-only oracle is insufficient.
- No path, impact, diagnostic, Git, or model evidence becomes semantic or edit
  authority.

## Task-specific validation

- Run the full non-zero `cargo test -p oneagent-analysis --test refactoring_plan`
  target plus the exact named EDT/Designer oracle targets accepted by ADR-0063.
- Run affected Graph/Analysis checks and the canonical validation triggered by
  `docs/codex/core/validation.md`.

## Suggested commit message

`Implement Sprint 40 validated refactoring planner`

## Final report additions

- Report target/precondition behavior, operation/conflict/completeness evidence,
  production fixture oracles, and preserved read-only boundary.
