# Git Change Adapter Implementation Profile

## Purpose

Use this profile for deterministic repository change-set discovery,
normalization, comparison, or integration with accepted Workspace change
inputs while preserving Git as evidence rather than semantic authority.

## Required Core modules

- `docs/codex/core/repository-safety.md`
- `docs/codex/core/repository-investigation.md`
- `docs/codex/core/context-management.md`
- `docs/codex/core/change-contract.md`
- `docs/codex/core/validation.md`
- `docs/codex/core/final-report.md`

## Required Workflow modules

- `docs/codex/workflows/implementation.md`
- `docs/codex/workflows/git-change-adapter.md`
- `docs/codex/workflows/runtime-service.md` when Runtime-owned Workspace
  observation, lifecycle, cancellation, or immutable publication changes
- `docs/codex/workflows/persistent-state.md` when repository baselines or
  normalized change evidence enter persisted state
- the applicable protocol or IDE workflow when repository changes are exposed
  through a supported public surface

## Task-family expectations

- Treat Git repository state as bounded source evidence and preserve the
  Workspace, adapters, Graph, Analysis, and diagnostics layers as the owners of
  semantic interpretation.
- Define repository boundary, baseline and current endpoint identity, included
  state layers, path normalization and confinement, change identity, status
  vocabulary, duplicate/conflict behavior, ordering, bounds, and failures.
- Make additions, modifications, deletions, rename or copy candidates,
  type changes, unmerged states, untracked paths, empty changes, incompatible
  repositories, and concurrent mutation explicit when applicable.
- Prove deterministic equivalence with accepted Workspace change inputs across
  input reorder, repeated reads, repository layouts, and equivalent end states.
- Keep semantic impact analysis, refactoring plans, source edits, transactions,
  remote repository access, credentials, hosting-provider behavior, telemetry,
  and broad performance or security claims outside the task unless separately
  accepted and explicitly included.
