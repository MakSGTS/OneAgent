# Review Workflow

Use this workflow for review-only tasks.

## Required review scope

- Inspect current diff and repository state.
- Compare implementation against applicable ADRs and architecture documents.
- Check correctness, regressions, validation, provenance, identity, and scope.
- Prioritize findings by severity.
- Report missing evidence separately from confirmed defects.

## Independent sprint integration review

Starting with Sprint 27, every sprint integration-review completion gate
requires one separate read-only reviewer agent with a fresh context. This is a
blocking requirement, not an optional parallelization hint.

The primary agent must:

1. finish and commit, or prove `already_complete`, every preceding task;
2. verify a clean task-owned working tree and resolve the exact review range;
3. use the active agent runtime's context-selection controls to start one
   reviewer without inherited implementation conversation turns, and block the
   review when a fresh context cannot be guaranteed;
4. provide only the repository root, exact range, authoritative documents,
   acceptance criteria and exclusions, required validation commands, and the
   output contract below;
5. omit the primary agent's expected decision, implementation rationale,
   acceptance summary, and proposed findings;
6. make no review-owned edit or state transition until the reviewer returns a
   complete result; and
7. independently inspect the same range and rerun the required review matrix
   before issuing the completion decision.

The reviewer may inspect repository files, Git evidence, and command output and
may run non-destructive read-only or validation commands. It must not:

- edit, create, delete, stage, or commit files;
- fix a finding or direct the primary agent to fix it inside the review task;
- change Roadmap or release state;
- retire prompt suites;
- inherit or request the primary agent's private implementation reasoning; or
- delegate its decision to another agent.

The reviewer must return:

- exact reviewed range and observed initial `HEAD` and Git status;
- one recommended decision: `pass`, `pass with non-blocking follow-ups`, or
  `blocked`;
- an acceptance-evidence matrix;
- blocking and non-blocking findings with exact file/line evidence when
  applicable;
- missing evidence separately from defects;
- commands run, exact outcomes, and every zero-match or unexecuted check;
- scope and exclusion conformance; and
- residual risks and the recommended next action.

The primary agent must compare its own evidence with the independent report.
The effective decision must not be less severe than the reviewer's decision.
Any unresolved disagreement, incomplete reviewer output, unavailable fresh-
context reviewer, reviewer mutation, or missing required validation makes the
review `blocked`.

After drafting an authorized review artifact, but before state transition,
prompt retirement, staging, or commit, the primary agent must ask the same
reviewer to verify read-only that the artifact preserves every finding,
missing-evidence item, decision, validation result, and risk without weakening
the independent report. A failed or unavailable consistency check blocks the
review.

## Completion-gate outputs

When the current task explicitly authorizes a sprint, release, or capability
completion record:

- create only the named review evidence artifact and state transition;
- issue the required completion decision from executed evidence;
- record the independent reviewer result and final artifact-consistency check
  when the independent sprint-review requirement applies;
- keep findings and their fixes outside the review change;
- transition state only after a non-blocking decision and successful required
  validation.

## Boundary

Do not change implementation files unless the review is explicitly converted
into an implementation task. Do not create review artifacts or state
transitions unless they are explicitly requested. Do not expand a review into
architecture design unless a concrete inconsistency is found.
