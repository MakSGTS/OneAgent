# Review Workflow

Use this workflow for review-only tasks.

## Required review scope

- Inspect current diff and repository state.
- Compare implementation against applicable ADRs and architecture documents.
- Check correctness, regressions, validation, provenance, identity, and scope.
- Prioritize findings by severity.
- Report missing evidence separately from confirmed defects.

## Completion-gate outputs

When the current task explicitly authorizes a sprint, release, or capability
completion record:

- create only the named review evidence artifact and state transition;
- issue the required completion decision from executed evidence;
- keep findings and their fixes outside the review change;
- transition state only after a non-blocking decision and successful required
  validation.

## Boundary

Do not change implementation files unless the review is explicitly converted
into an implementation task. Do not create review artifacts or state
transitions unless they are explicitly requested. Do not expand a review into
architecture design unless a concrete inconsistency is found.
