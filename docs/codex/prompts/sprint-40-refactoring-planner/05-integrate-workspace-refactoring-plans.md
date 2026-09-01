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

# Integrate Sprint 40 Workspace Refactoring Plans

## Reporting

- Communicate with the user in Russian.
- Keep code, APIs, tests, docs, errors, and the commit message in English.

## Context manifest

### Must read

- `AGENTS.md` — sections: repository scope, change discipline, validation, and
  Git branch/release workflow.
- `docs/adr/0063-refactoring-planner.md` — sections: Workspace owner,
  publication/configuration/source preconditions, lifecycle, failures,
  compatibility, and first Runtime slice.
- `docs/Roadmap.md` — sections: Sprint 40 exclusions and Task 5.
- `docs/codex/profiles/refactoring-safe-edits-implementation.md`,
  `docs/codex/templates/refactoring-safe-edits-task.md`,
  `docs/codex/workflows/refactoring-safe-edits.md`, and
  `docs/codex/workflows/runtime-service.md` — selected and conditional Runtime
  contracts.
- `crates/analysis/src/refactoring.rs` and focused tests — complete committed
  Task 4 planner API and evidence.
- `apps/runtime/src/workspace/mod.rs` — symbols:
  `WorkspaceConfigurationSnapshot`, `WorkspaceSnapshot`, publication identity,
  configuration lookup, service snapshot observation, and direct tests.
- `apps/runtime/tests/fixtures/workspace_service/` — only exact EDT/Designer
  files named by ADR-0063 and their owning test functions.

### Lookup on demand

- `apps/runtime/src/workspace/{cache,repository_change}.rs` — trigger: ADR-0063
  changes or explicitly preserves cache/Git precondition behavior; symbols:
  only matching schema/input boundaries.
- filesystem-watching tests — trigger: publication freshness or change
  lifecycle cannot be proven by Workspace unit tests; named tests only.
- adapter source — trigger: a production fixture produces contradictory source
  evidence; exact failing function and path only.

### Excluded from initial context

- complete Runtime, cache, Git, adapter, protocol, and client implementations;
- unrelated Workspace tests and historical logs;
- source reads after publication, source mutation, persistence of plans, and
  Sprint 41 transaction behavior.

### Preflight

- Record effective window or `unknown`, measurement basis, admitted material,
  and `pass|warning|blocked` before implementation.
- Narrow Runtime/test selectors at warning and stop at the hard limit.

## Prerequisites / required gate

- `HEAD` is exactly the committed Task 4 result with subject
  `Implement Sprint 40 validated refactoring planner`.
- The planner and production adapter oracle targets pass unfiltered and the
  task-owned worktree is clean.

## Task

Integrate ADR-0063 planning with one immutable complete Workspace publication
and its accepted configuration/source evidence. Keep planning read-only and
on-demand unless ADR-0063 explicitly accepts another immutable composition.

## Scope

### Included

- Publication and configuration matching, snapshot/source preconditions,
  planner invocation, immutable result ownership, deterministic order,
  cancellation, failure/recovery, repeated calls, and EDT/Designer equivalence.
- Affected Workspace public API, tests, semantic compatibility, and cache/
  watcher behavior only as required by ADR-0063.

### Excluded

- Source re-read or write, snapshot mutation, edit application, plan history or
  persistence, Git mutation, protocol handlers, clients, and Sprint 41.

## Acceptance criteria

- One call observes one immutable complete snapshot; publication/configuration
  and source preconditions cannot be mixed across snapshots.
- Missing, stale, ambiguous, incompatible, cancelled, build-failed, and
  out-of-bound cases expose no partial plan and do not replace valid state.
- EDT and Designer fixtures prove the accepted semantic/source evidence and
  deterministic repeated behavior.
- Existing Workspace rebuild, impact, diagnostics, rules, cache, watcher,
  cancellation, and shutdown behavior remains compatible.

## Task-specific validation

- Run non-zero focused Workspace refactoring-plan tests and the accepted
  EDT/Designer fixture matrix.
- Run affected Workspace/watching/cache/Git-input tests plus the canonical
  validation triggered by `docs/codex/core/validation.md`.

## Suggested commit message

`Integrate Sprint 40 Workspace refactoring plans`

## Final report additions

- Report snapshot/precondition ownership, lifecycle/failure behavior,
  cross-format evidence, cache/watcher impact, and preserved no-mutation state.
