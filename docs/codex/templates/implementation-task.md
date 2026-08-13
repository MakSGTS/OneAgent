# Implementation Task Template

## Purpose

Use this template for implementing one accepted capability or behavior.

## Recommended profile

- `docs/codex/profiles/implementation.md`

## Required task-specific sections

- Authoritative ADRs / architecture documents
- Task
- Scope
- Included
- Excluded
- Acceptance Criteria
- Task-specific Validation
- Suggested commit message

## Additional acceptance requirements

- Preserve accepted architecture.
- Implement one coherent slice.
- Add focused tests and regression tests appropriate to the change.
- Update documentation only when behavior or public contracts change.

## Additional report sections

- Implementation summary
- Production path
- Tests added or changed
- Coverage transition, if any

## Additional validation

- Run focused checks first.
- Run affected package checks when public APIs or crate behavior change.
- Run full workspace validation when required by `docs/codex/core/validation.md`.
