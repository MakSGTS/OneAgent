# Review Task Template

## Purpose

Use this template for review-only tasks over an implementation, document set,
diff, architecture decision, or completed capability.

## Recommended profile

- `docs/codex/profiles/review.md`

## Required task-specific sections

- Authoritative ADRs / architecture documents
- Review target
- Reviewed baseline / commit or diff range
- Scope
- Excluded
- Review Criteria
- Acceptance evidence matrix, when the review is a completion gate
- Task-specific Validation, if any

## Additional acceptance requirements

- Do not modify files unless explicitly requested.
- Prioritize confirmed findings by severity.
- Separate defects, missing evidence, and open questions.
- Compare behavior against accepted architecture and task scope.
- Verify that explicitly excluded or deferred scope was not pulled into the
  reviewed change.
- For a sprint, release, or capability completion gate, issue one explicit
  decision: `pass`, `pass with non-blocking follow-ups`, or `blocked`.
- Base a completion decision on executed evidence for every applicable
  acceptance criterion, not on implementation claims alone.

## Additional report sections

- Reviewed baseline
- Acceptance evidence matrix, when applicable
- Findings
- Missing evidence
- Scope and exclusion conformance
- Completion decision, when applicable
- Deferred or non-blocking follow-ups
- Risk assessment
- Recommended next action

## Additional validation

- Run only review-appropriate checks unless the user asks to fix issues.
- For a completion gate, run or verify the focused and integration checks named
  by the authoritative acceptance contract and record their exact results.
