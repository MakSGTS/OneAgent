# Graph Model Task Template

## Purpose

Use this template for graph model changes involving `NodeKind`, `EdgeKind`, graph
identity, endpoint compatibility, validation infrastructure, query/filter
behavior, serialization, or public graph APIs.

## Recommended profile

- `docs/codex/profiles/graph-implementation.md`

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

- Define deterministic identity and collision behavior.
- Preserve existing identifiers unless the task explicitly changes them.
- Define validation, query, equality, ordering, and serialization impact.
- Do not change parser or producer behavior unless explicitly included.

## Additional report sections

- Graph model impact
- Public API impact
- Validation/query behavior
- Serialization impact, if any

## Additional validation

- Run graph crate tests and any affected producer tests.
- Run full workspace validation when graph public APIs change.
