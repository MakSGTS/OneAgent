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

# Complete Sprint 40 Refactoring Planner Evidence

## Reporting

- Communicate with the user in Russian.
- Keep repository artifacts and the commit message in English.

## Context manifest

### Must read

- `AGENTS.md` — sections: repository scope, change discipline, validation, Git
  branch/release workflow, and GUI validation.
- `docs/adr/0063-refactoring-planner.md` — sections: complete acceptance matrix,
  compatibility, evidence, deferred scope, and Sprint 41 hand-off.
- `docs/Roadmap.md` — sections: Sprint 40 objective, manifest, exclusions, and
  state/validation gates.
- `docs/codex/profiles/refactoring-safe-edits-implementation.md`,
  `docs/codex/templates/refactoring-safe-edits-task.md`,
  `docs/codex/workflows/refactoring-safe-edits.md`, and
  `docs/codex/core/validation.md` — complete evidence contract and canonical
  validation source.
- committed Task 2 through Task 6 diff — diff: ADR commit through current
  `HEAD`, with path inventory first and only changed symbols/tests loaded.
- exact focused targets, compatibility consumers, fixtures, schemas, public
  exports, manifests, lockfile, Coverage, and current-state sections cited by
  ADR-0063 and the changed diff.

### Lookup on demand

- full source file — trigger: a changed hunk cannot establish its surrounding
  invariant; load only that file and matching direct consumer.
- client or GUI validation — trigger: the Task 2–6 range changes its source,
  manifest, schema, package, or workflow; run only the affected documented host
  gate under `AGENTS.md`.
- retained command log — trigger: a required command fails or a complete audit
  needs durable evidence; store under `local-artifacts/codex-runs/`.

### Excluded from initial context

- unrelated implementation and historical sprint ranges;
- complete generated projects, fixture corpora, target outputs, and successful
  full logs;
- Sprint 41 implementation and unsupported refactoring families.

### Preflight

- Record effective window or `unknown`, measurement basis, admitted material,
  and `pass|warning|blocked` before the evidence audit.
- Start from diff/path/test inventories, narrow at warning, and stop at the hard
  limit.

## Prerequisites / required gate

- `HEAD` is exactly the committed Task 6 result with subject
  `Integrate Sprint 40 product refactoring planning`.
- Tasks 1–6 are committed and pushed in order with no task-owned uncommitted
  change.

## Task

Create `docs/architecture/refactoring-planner-evidence.md`, synchronize only
current-state and Sprint 40 evidence sections in `docs/Architecture.md`,
`docs/architecture/semantic-model-2.md`, and `docs/Roadmap.md`, and run the
complete accepted validation and audit matrix. Do not mark Sprint 40 completed.

## Scope

### Included

- Exact focused and canonical validation outcomes with non-zero counts.
- ADR acceptance, cross-format/source evidence, plan/precondition/conflict/
  preview behavior, Workspace lifecycle, schema/policy/public process,
  compatibility, dependency/license, API, cache, Coverage, sensitive-data,
  deferred-scope, tracked-artifact, and cleanliness audits.
- Current-state documentation and exact review hand-off range definition.

### Excluded

- New production behavior, architecture changes, fixes hidden in evidence work,
  Sprint completion, prompt retirement, and Sprint 41 implementation.

## Acceptance criteria

- Every ADR-0063 criterion maps to executed evidence or an explicit blocker;
  no zero-match, filtered-only, skipped, or unavailable check is reported as a
  pass.
- Documentation and measured counts agree with the immutable current `HEAD`.
- The range contains no source mutation, edit transaction, authority
  duplication, sensitive leak, unrelated dependency/API change, or deferred
  scope.
- Review hand-off names the exact planning-through-Task-7 range and complete
  validation matrix.

## Task-specific validation

- Run every non-zero focused Graph/Analysis/adapter/Workspace/Runtime/Protocol/
  Tool Policy/MCP/public-process and compatibility target accepted by ADR-0063.
- Run the complete canonical workspace validation from
  `docs/codex/core/validation.md`, diff checks for the task and sprint range,
  executable test enumeration, and the documented API/dependency/scope audits.

## Suggested commit message

`Complete Sprint 40 Refactoring Planner evidence`

## Final report additions

- Report the acceptance matrix, exact counts, compatibility/API/dependency and
  scope audits, documentation changes, retained logs, and immutable review
  hand-off.
