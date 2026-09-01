---
prompt_contract: v2
task_kind: architecture
profile: docs/codex/profiles/architecture.md
template: docs/codex/templates/architecture-task.md
fresh_context: required
context_static_max_percent: 15
context_authorities_max_percent: 20
context_prework_hard_stop_percent: 50
context_working_min_percent: 35
context_reserve_min_percent: 15
---

# Define Sprint 40 Refactoring Planner

## Reporting

- Communicate with the user in Russian.
- Keep repository artifacts and the commit message in English.

## Context manifest

### Must read

- `AGENTS.md` — sections: repository scope, change discipline, validation, and
  Git branch/release workflow.
- `docs/Roadmap.md` — sections: Sprint 40 Refactoring Planner execution plan and
  Task 1 investigation evidence.
- `docs/architecture/refactoring-planner-investigation.md` — complete accepted
  Task 1 evidence and decision questions.
- `docs/codex/profiles/architecture.md`,
  `docs/codex/templates/architecture-task.md`, and
  `docs/codex/workflows/architecture.md` — complete selected contract.
- `docs/codex/workflows/refactoring-safe-edits.md` — complete reusable planner
  contract, with mutation-only requirements retained as Sprint 41 constraints.
- live definitions and tests cited by the investigation — symbols and exact
  bounded queries named in that artifact only.

### Lookup on demand

- `docs/adr/0061-change-impact-analysis.md` — trigger: impact authority or
  Workspace publication compatibility needs exact language; sections: Decision,
  Workspace composition, MCP projection, and Deferred scope.
- `docs/Architecture.md` or `docs/architecture/semantic-model-2.md` — trigger:
  the investigation identifies one unresolved ownership or identity conflict;
  sections: only the matching current boundary.
- recent ADR structure — trigger: a decision category lacks an established
  format; paths: ADR-0058 through ADR-0061, matching headings only.

### Excluded from initial context

- complete architecture history and unrelated ADRs;
- full fixture corpora and successful logs;
- production source not cited by Task 1;
- transaction application, rollback implementation, and Sprint 41 prompts.

### Preflight

- Record effective window or `unknown`, measurement basis, admitted static and
  authority material, and `pass|warning|blocked` before architecture work.
- Narrow optional authority selectors at warning and stop at the hard limit.

## Prerequisites / required gate

- `HEAD` is exactly the committed Task 1 result with subject
  `Investigate Sprint 40 Refactoring Planner`.
- Task 1 records a non-blocked data/oracle gate and no conflicting worktree
  change exists.

## Task

Create accepted ADR-0063 for the smallest evidence-backed read-only Refactoring
Planner slice. Update only the matching Sprint 40 architecture-decision
subsection in `docs/Roadmap.md`.

## Scope

### Included

- Canonical owners and immutable inputs.
- First refactoring family and supported target/source matrix.
- Target, publication, source/version precondition, request, plan, operation,
  preview, summary, and completeness contracts.
- Deterministic identity/order, duplicates, overlaps, conflicts, bounds,
  cancellation, errors, and sensitive-data rules.
- Workspace lifecycle, Runtime, Tool Policy, MCP projection, compatibility,
  evidence, implementation prerequisites, and explicit Sprint 41 hand-off.

### Excluded

- Production code, source reads after accepted snapshot capture, source edits,
  transactions, atomicity/rollback/reversibility implementation, persistence,
  new UI, and unsupported refactoring families.

## Acceptance criteria

- ADR-0063 answers every Task 1 decision question with one implementable
  canonical contract and records rejected alternatives.
- Planning and preview are explicitly read-only and cannot be interpreted as
  edit authorization.
- Bounds, completeness, failures, compatibility, and a deterministic production
  oracle are exact enough for Tasks 3–7.
- Deferred scope preserves Sprint 41 transaction ownership.

## Task-specific validation

- Validate Markdown structure, internal links, Roadmap/ADR agreement, and
  `git diff --check`.
- Do not run production tests because this task changes architecture only.

## Suggested commit message

`Define Sprint 40 Refactoring Planner`

## Final report additions

- Report the accepted decision, rejected alternatives, implementation gates,
  and unchanged product behavior.
