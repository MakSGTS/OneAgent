# Parser Task Template

## Purpose

Use this template for parsing one real serialized source artifact family.

## Recommended profile

- `docs/codex/profiles/parser-implementation.md`

## Required task-specific sections

- Authoritative ADRs / source-format documents
- Prerequisites / required gate
- Task
- Source evidence / fixtures
- Scope
- Included
- Excluded
- Acceptance Criteria
- Task-specific Validation
- Suggested commit message (recommendation only)

## Additional acceptance requirements

- Use real source evidence; do not invent formats.
- Define supported fields and explicitly unknown fields.
- Define malformed, missing, optional, duplicate, and ordering behavior.
- Keep parsing separate from graph emission unless explicitly included.

## Additional report sections

- Source evidence
- Parsed contract
- Unsupported or unknown source cases
- Fixture coverage

## Additional validation

- Run parser-focused tests and fixture checks.
- Run affected crate checks when parser behavior changes.
