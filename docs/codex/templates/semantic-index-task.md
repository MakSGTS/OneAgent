# Semantic Index Task Template

## Purpose

Use this template for complete-snapshot or incremental Semantic Index work over
the canonical source-independent semantic graph.

## Recommended profile

- `docs/codex/profiles/semantic-index-implementation.md`

## Required task-specific sections

- Authoritative ADRs / architecture documents
- Task
- Canonical source snapshot / semantic authority
- Index dimensions and lookup contracts
- Lifecycle and staleness contract
- Compatibility and consumer impact
- Scope
- Included
- Excluded
- Acceptance Criteria
- Task-specific Validation
- Suggested commit message (recommendation only)

## Additional acceptance requirements

- Define deterministic keys, values, collision behavior, and result ordering for
  every lookup dimension in scope.
- Preserve canonical graph facts, identities, resolution rules, and observable
  query behavior.
- Prove equivalence with the accepted scan-based, resolution, or full-rebuild
  behavior being indexed.
- Cover empty, duplicate, missing, ambiguous, and invalid-state behavior where
  applicable.
- Define graph/index/query ownership, lifetime, construction, and staleness.
- Audit all consumers before changing public or internal compatibility surfaces.
- Keep incremental maintenance excluded from snapshot tasks unless separately
  accepted architecture includes it.
- For incremental tasks, prove every supported change sequence produces the same
  result as a clean full rebuild.

## Additional report sections

- Canonical authority and derived-state boundary
- Indexed dimensions
- Lifecycle and staleness behavior
- Compatibility and consumer impact
- Equivalence and determinism evidence
- Deferred optimization or incremental scope

## Additional validation

- Run focused index, Query, and Resolution tests first.
- Run affected graph and consumer tests.
- Run full workspace validation for production index behavior or API changes as
  required by `docs/codex/core/validation.md`.
