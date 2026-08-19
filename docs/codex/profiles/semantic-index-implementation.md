# Semantic Index Implementation Profile

## Purpose

Use this profile for repeated Semantic Index implementation tasks that build or
integrate a deterministic derived view over the source-independent semantic
graph. It covers complete-snapshot indexing and, only under separately accepted
architecture, incremental maintenance.

## Required Core modules

- `docs/codex/core/repository-safety.md`
- `docs/codex/core/repository-investigation.md`
- `docs/codex/core/change-contract.md`
- `docs/codex/core/validation.md`
- `docs/codex/core/final-report.md`

## Required Workflow modules

- `docs/codex/workflows/implementation.md`
- `docs/codex/workflows/semantic-index.md`
- `docs/codex/workflows/graph-model.md` when public graph or query APIs change

## Task-family expectations

- Treat the semantic graph as the single canonical semantic authority.
- Preserve accepted query, resolution, identity, ownership, and ordering
  behavior.
- Define snapshot lifecycle and staleness behavior explicitly.
- Prove indexed behavior is equivalent to the canonical behavior it replaces or
  accelerates.
- Keep complete-snapshot and incremental maintenance scope in separate tasks
  unless accepted architecture explicitly requires one coherent transition.
- Do not pull persistence, Runtime, transport, source-adapter, or new semantic
  inference concerns into index implementation tasks.
