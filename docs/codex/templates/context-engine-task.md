# Context Engine Task Template

## Purpose

Use this template for deterministic semantic context selection, budgeting,
assembly, explanation, rendering, or evaluation work.

## Recommended profile

- `docs/codex/profiles/context-engine-implementation.md`

## Required task-specific sections

- Authoritative ADRs / architecture documents
- Prerequisites / required gate
- Task
- Canonical snapshot and data boundary
- Request, seed, and policy contract
- Selection, relevance, and ordering contract
- Budget, cost, and truncation contract
- Provenance, explanation, and rendering contract
- Evaluation corpus and oracle
- Compatibility and consumer impact
- Scope
- Included
- Excluded
- Acceptance Criteria
- Task-specific Validation
- Suggested commit message (recommendation only)

## Additional acceptance requirements

- Consume canonical graph or accepted query state without creating a competing
  semantic authority.
- Define deterministic validation, resolution, selection, ordering,
  deduplication, budgeting, truncation, and assembly behavior for the task slice.
- Preserve exact provenance and explain every included context item.
- Prove boundary budgets, explicit omissions, input reordering, and repeated
  requests against a repository-owned oracle.
- Audit affected consumers before changing a public request, bundle, rendering,
  or Runtime surface.
- Keep LLM providers, model execution, tool policy, MCP, IDE, and unaccepted
  source-fragment behavior excluded.

## Additional report sections

- Canonical authority and data boundary
- Request and seed behavior
- Selection and relevance behavior
- Budget and truncation behavior
- Provenance and explanation evidence
- Evaluation results
- Compatibility and consumer impact
- Deferred AI integration scope

## Additional validation

- Run focused context request, selection, budget, rendering, and evaluation
  checks applicable to the changed slice.
- Run affected graph, analysis, Runtime, or consumer checks when their public or
  observable behavior changes.
- Run full workspace validation for production Context Engine behavior or API
  changes as required by `docs/codex/core/validation.md`.
