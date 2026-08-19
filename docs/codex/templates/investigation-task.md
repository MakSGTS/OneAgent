# Investigation Task Template

## Purpose

Use this template for read-only repository, architecture, implementation,
fixture, source-format, or compatibility investigations.

## Recommended profile

- `docs/codex/profiles/investigation.md`

## Required task-specific sections

- Authoritative ADRs / architecture documents, if any
- Prerequisites / required gate, if any
- Investigation objective
- Questions to answer
- Evidence scope
- Evidence sources / fixtures
- Excluded
- Completion Criteria
- Task-specific Validation, if any

## Additional acceptance requirements

- Recheck mutable repository facts instead of relying on a historical prompt
  baseline.
- Separate confirmed repository evidence, accepted decisions, assumptions, and
  unresolved unknowns.
- Trace relevant definitions, consumers, tests, fixtures, and history only as
  far as needed to answer the stated questions.
- Do not invent source formats, behavior, architecture, or completion evidence.
- Do not modify files unless the task explicitly authorizes investigation notes.
- Stop before implementation and identify the missing prerequisite when the
  required evidence is unavailable.

## Additional report sections

- Confirmed findings
- Accepted constraints
- Assumptions and unresolved unknowns
- Consumer or compatibility inventory, if relevant
- Decision readiness
- Recommended next action

## Additional validation

- Run only non-mutating, evidence-producing checks appropriate to the
  investigation.
- Report unexecuted, zero-match, or inconclusive checks separately from confirmed
  evidence.
