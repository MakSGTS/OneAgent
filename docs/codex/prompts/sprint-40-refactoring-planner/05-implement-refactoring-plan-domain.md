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

# Implement Sprint 40 Refactoring Plan Domain

## Reporting

- Communicate with the user in Russian.
- Keep code, APIs, tests, docs, errors, and the commit message in English.

## Context manifest

### Must read

- `AGENTS.md` — sections: repository scope, change discipline, validation, and
  Git branch/release workflow.
- `docs/adr/0063-refactoring-planner.md` — sections: accepted decision, domain
  contract, bounds/failures, compatibility, and first implementation slice.
- `docs/Roadmap.md` — sections: Sprint 40 objective, exclusions, and Task 5.
- `docs/codex/profiles/refactoring-safe-edits-implementation.md`,
  `docs/codex/templates/refactoring-safe-edits-task.md`, and
  `docs/codex/workflows/refactoring-safe-edits.md` — complete selected contract.
- `crates/common/src/{identity,source}.rs` and `crates/analysis/src/lib.rs` —
  symbols cited by ADR-0063 and direct public consumers found by bounded `rg`.
- `crates/analysis/src/{change_impact,diagnostics,rules}` — query: existing
  immutable result, summary, bound, closed-error, and public-export conventions.

### Lookup on demand

- `crates/graph/src/{kind,node,query}.rs` — trigger: an accepted domain type must
  reuse an exact Graph-owned vocabulary; symbols: only those cited by ADR-0063.
- Cargo manifests and reverse consumers — trigger: a public export or dependency
  question remains; query: exact crate name or symbol only.
- implementation history — trigger: live naming conventions conflict; query:
  path-scoped history for the conflicting symbol.

### Excluded from initial context

- complete Graph, adapter, Runtime, and protocol implementations;
- unrelated Analysis engines and historical sprint diffs;
- fixture corpora, generated outputs, and successful logs;
- planner evaluation, Workspace integration, public projection, and mutation.

### Preflight

- Record effective window or `unknown`, measurement basis, admitted material,
  and `pass|warning|blocked` before implementation.
- Narrow consumer queries at warning and stop at the hard limit.

## Prerequisites / required gate

- `HEAD` is exactly the committed Task 4 result with subject
  `Integrate Sprint 40 adapter source evidence`.
- ADR-0063 is accepted and the task-owned worktree is clean.

## Task

Implement only the source-independent immutable Refactoring Plan domain accepted
by ADR-0063, expected under `crates/analysis/src/refactoring.rs` with public
exports and focused tests under `crates/analysis/tests/refactoring_plan.rs`.

## Scope

### Included

- Accepted request, target, snapshot/source precondition, operation, plan,
  preview, completeness, summary, bound, and closed failure types.
- Validated construction, stable equality/order, duplicate/conflict rejection,
  checked counters, redacted errors, public exports, Rustdoc, and focused tests.

### Excluded

- Graph queries or plan generation, filesystem/source access, Workspace/Runtime,
  protocol/policy/client changes, persistence, and all mutation.

## Acceptance criteria

- Types and constructors enforce ADR-0063 without partial values or encounter-
  order behavior.
- Identity fields remain separate from preview/display content and sensitive
  values do not enter errors.
- Exact and one-over bounds, duplicate/conflict cases, ordering, summaries, and
  public API behavior have non-zero focused evidence.
- No production dependency or unrelated public API changes.

## Task-specific validation

- Run `cargo test -p oneagent-analysis --test refactoring_plan` and confirm a
  non-zero unfiltered result.
- Run affected Analysis package checks and the canonical validation triggered
  by `docs/codex/core/validation.md`.

## Suggested commit message

`Implement Sprint 40 refactoring plan domain`

## Final report additions

- Report the public domain surface, bounds/failures, tests, API/dependency audit,
  and preserved no-evaluation/no-mutation boundary.
