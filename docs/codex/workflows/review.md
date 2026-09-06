# Review Workflow

Use this workflow for review-only tasks.

## Required review scope

- Inspect current diff and repository state.
- Compare implementation against applicable ADRs and architecture documents.
- Check correctness, regressions, validation, provenance, identity, and scope.
- Prioritize findings by severity.
- Report missing evidence separately from confirmed defects.

## Targeted pre-implementation design review

For architecture-sensitive Sprint 41 and later work, run one separate
fresh-context read-only reviewer before production implementation. Review the
exact committed planning/ADR range and committed ADR-invariant matrix, without
implementation transcripts or an expected decision. The reviewer returns one
decision, `pass` or `blocked`, plus matrix coverage, findings with exact
evidence, missing evidence, scope conformance, and the recommended next action.

A current user instruction that plans or launches a Sprint 41 or later suite
under the Sprint Planning or Sequential Sprint Execution workflow authorizes
exactly this one read-only design reviewer. Start it automatically when the
committed pre-implementation gate is reached. The authorization does not permit
delegation, mutation, staging, commit, or another reviewer, and an explicit user
prohibition on subagents makes the gate blocked.

The design review is blocked when an applicable accepted production invariant
lacks an exact owner or production location, an order-sensitive guard lacks the
operation or retention point it must precede, a negative production oracle is
missing, accepted owners or boundaries contradict each other, or planned scope
cannot implement the invariant. Commit the non-blocking decision with the
planning evidence before starting production implementation. If the required
fresh reviewer is unavailable, stop at this gate. This targeted review does not
run a full production validation gate and does not replace the final integration
review.

## Independent sprint integration review

Starting with Sprint 27, every sprint integration-review completion gate
requires one separate read-only reviewer agent with a fresh context. This is a
blocking requirement, not an optional parallelization hint.

### Reviewer authorization

A current user instruction that invokes a review task selecting this workflow
and requiring an independent reviewer, or that launches a sprint execution whose
final integration review requires this workflow, explicitly authorizes exactly
one fresh-context read-only reviewer agent for that review. Start that reviewer
automatically when the review gate is reached. Do not ask the user for a separate
delegation confirmation.

This automatic authorization is limited to the one mandatory reviewer and the
read-only review contract below. It does not authorize additional subagents,
reviewer delegation, repository mutation, staging, committing, or any external
side effect. An explicit current user prohibition on subagents overrides this
default and makes a mandatory independent review blocked.

The primary agent must:

1. finish and commit, or prove `already_complete`, every preceding task;
2. verify a clean task-owned working tree and resolve an exact immutable review
   range whose endpoints are committed objects;
3. use the active agent runtime's context-selection controls to start the
   automatically authorized reviewer without inherited implementation
   conversation turns, and block the review when a fresh context cannot be
   guaranteed;
4. provide only the repository root, exact range, authoritative documents,
   acceptance criteria and exclusions, required validation commands, and the
   output contract below;
5. omit the primary agent's expected decision, implementation rationale,
   acceptance summary, and proposed findings;
6. make no review-owned edit or state transition until the reviewer returns a
   complete result; and
7. independently inspect the same range and rerun the required review matrix
   before issuing the completion decision.

For Sprint 41 and later, never dispatch a reviewer against a floating working
tree, an uncommitted diff, an open-ended range, or a branch name whose head may
move during review. Pass the exact commit IDs and range. Do not mutate the
reviewed paths or move either endpoint while the reviewer is running. If the
baseline changes, invalidate the in-flight result and start a new review only
after the replacement range is committed, stable, and has passed its required
focused validation. Earlier sprint reviews retain their committed exact-range
contracts.

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
