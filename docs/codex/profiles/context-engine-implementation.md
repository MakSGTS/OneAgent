# Context Engine Implementation Profile

## Purpose

Use this profile for deterministic semantic context selection and assembly over
canonical source-independent graph snapshots. It covers seed resolution,
candidate selection, bounded traversal, relevance ordering, budgeting,
truncation, provenance, explanations, and reproducible evaluation.

## Required Core modules

- `docs/codex/core/repository-safety.md`
- `docs/codex/core/repository-investigation.md`
- `docs/codex/core/change-contract.md`
- `docs/codex/core/validation.md`
- `docs/codex/core/final-report.md`

## Required Workflow modules

- `docs/codex/workflows/implementation.md`
- `docs/codex/workflows/context-engine.md`
- `docs/codex/workflows/graph-model.md` when public graph or query APIs change
- `docs/codex/workflows/runtime-service.md` when Runtime lifecycle, transport,
  or supported-client behavior changes

## Task-family expectations

- Keep the semantic graph and accepted derived query facilities authoritative;
  context selection is a read-only consumer and must not create semantic facts.
- Define seed identity and failure behavior, allowed traversal, relevance order,
  tie-breaking, deduplication, budget units, truncation, and output order.
- Preserve provenance and provide a deterministic explanation for every
  included context item.
- Prove repeatability and relevance through repository-owned evaluation cases
  with exact expected outcomes.
- Keep provider requests, model execution, tool authorization, MCP, IDE, and
  speculative source extraction outside Context Engine implementation tasks
  unless separately accepted architecture and task scope include them.
