# Refactoring and Safe Edits Implementation Profile

## Purpose

Use this profile for deterministic semantic refactoring plans, checked edit
previews, conflict detection, or reversible source-edit transactions over
accepted canonical semantic and source evidence.

## Required Core modules

- `docs/codex/core/repository-safety.md`
- `docs/codex/core/repository-investigation.md`
- `docs/codex/core/context-management.md`
- `docs/codex/core/change-contract.md`
- `docs/codex/core/validation.md`
- `docs/codex/core/final-report.md`

## Required Workflow modules

- `docs/codex/workflows/implementation.md`
- `docs/codex/workflows/refactoring-safe-edits.md`
- `docs/codex/workflows/runtime-service.md` when immutable Runtime snapshots,
  lifecycle, cancellation, or supported public composition changes
- `docs/codex/workflows/git-change-adapter.md` when repository state is an
  explicit precondition or conflict input
- `docs/codex/workflows/ai-tool-policy.md` when a public AI-facing surface can
  request preview or mutation
- the applicable protocol or IDE workflow when plans, previews, or edit results
  are exposed through a supported client

## Task-family expectations

- Preserve Graph, source adapters, Workspace publications, diagnostics, rules,
  and impact reports as evidence owners; a refactoring plan is not new semantic
  authority or edit authorization.
- Define target identity, immutable input snapshot, preconditions, plan and
  operation identity, ordering, duplicates, conflicts, bounds, completeness,
  preview, failures, and sensitive-data rules before implementation.
- Separate read-only plan construction from any source mutation. A plan must be
  independently inspectable and reject stale, ambiguous, incompatible, or
  incomplete evidence without partial output.
- For edit transactions, define filesystem confinement, exact write set,
  atomicity boundary, conflict recheck, rollback, reversibility, crash and
  cancellation behavior, and post-edit semantic validation.
- Prove deterministic planning and, when applicable, all-or-nothing application
  across reordered evidence, repeated execution, concurrent change, injected
  failure, rollback, recovery, and equivalent source end states.
- Keep concrete refactoring kinds, source grammars, formatting policy, protocol
  shape, authorization UI, persistence, and deployment scope outside the task
  unless separately accepted and explicitly included.
