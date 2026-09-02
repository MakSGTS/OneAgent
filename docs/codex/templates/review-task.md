# Review Task Template

## Purpose

Use this template for review-only tasks over an implementation, document set,
diff, architecture decision, or completed capability.

## Recommended profile

- `docs/codex/profiles/review.md`

## Required base template

- `docs/codex/templates/task-prompt.md`

## Required task-specific sections

- Review target
- Reviewed baseline / commit or diff range
- Review Criteria
- Acceptance evidence matrix, when the review is a completion gate
- Independent reviewer contract and output, for Sprint 27 or later integration
  reviews
- Automatic independent-reviewer authorization, when independent review
  applies
- Primary/reviewer evidence reconciliation, when independent review applies
- Authorized review outputs and state transition, when changes are requested

## Additional acceptance requirements

- Do not modify files unless explicitly requested.
- Prioritize confirmed findings by severity.
- Separate defects, missing evidence, and open questions.
- Compare behavior against accepted architecture and task scope.
- Verify that explicitly excluded or deferred scope was not pulled into the
  reviewed change.
- For a sprint, release, or capability completion gate, issue one explicit
  decision: `pass`, `pass with non-blocking follow-ups`, or `blocked`.
- When independent review applies, follow
  `docs/codex/workflows/review.md` without copying its authorization, input,
  output, reconciliation, or consistency rules into the child prompt.
- Base a completion decision on executed evidence for every applicable
  acceptance criterion, not on implementation claims alone.
- Keep review artifact creation and Roadmap state transitions explicit and
  bounded. Do not treat an integration review as permission to fix findings.
- Transition a sprint or release only after a non-blocking decision and all
  required validation succeeds.

## Additional report sections

- Reviewed baseline
- Independent reviewer handoff, observed baseline, and recommended decision,
  when applicable
- Primary/reviewer evidence reconciliation and artifact-consistency result,
  when applicable
- Acceptance evidence matrix, when applicable
- Findings
- Missing evidence
- Scope and exclusion conformance
- Completion decision, when applicable
- Review artifacts and state transition, when authorized
- Deferred or non-blocking follow-ups
- Risk assessment
- Recommended next action

## Additional validation

- Run only review-appropriate checks unless the user asks to fix issues.
- For a completion gate, run or verify the focused and integration checks named
  by the authoritative acceptance contract and record their exact results.
- When independent review applies, preserve the reviewer's command results
  separately and have the primary agent independently rerun the required
  focused and complete validation matrix. A copied result is not independent
  validation.
