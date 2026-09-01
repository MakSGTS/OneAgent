# Sprint Execution Loop Template

## Purpose

Use this template for a master prompt that executes an already accepted sprint
plan strictly in dependency order.

## Required workflow

- `docs/codex/workflows/sequential-sprint-execution.md`

## Required task-specific sections

- Reporting language
- Canonical authorities
- Sprint objective and current state
- Starting-state requirements
- Ordered task manifest
- Commit authorization mode
- Fresh-context child-runner authorization and unavailable-runtime fallback
- Initial audit additions
- Task-loop additions, if any
- Already-complete policy additions, if any
- Failure and integration-review gates
- Automatic mandatory reviewer authorization, when the manifest includes an
  independent integration review
- Final report additions

The ordered manifest must identify, for every task:

- order and prompt path;
- required committed prerequisite;
- task-owned outcome;
- task-specific validation additions;
- Prompt Contract v2 and Context Manifest validation state;
- suggested commit message.

## Additional acceptance requirements

- Treat the manifest as an execution plan, not proof of current repository
  state.
- Resolve every mutable baseline from the live repository before execution.
- Do not reorder, skip, combine, or partially commit dependent tasks.
- Dispatch every child in a guaranteed fresh context and retain only its compact
  verified ledger row. Never execute two children in the master context.
- Do not permanently encode commit authorization in a stored prompt. Determine
  commit mode from the current user instruction that launches the execution.
- Treat the current user's launch of the master prompt as authorization for one
  sequential fresh-context runner per manifest child and one mandatory
  fresh-context read-only reviewer when required. Neither runner nor reviewer
  may delegate further. This does not change commit authorization.
- Stop at the child boundary and emit the exact continuation prompt when the
  runtime cannot guarantee fresh context.
- Preserve prompt-suite files unless their modification is explicitly part of
  the current task scope.
- Stop after the first blocking failure.

## Additional report sections

- Starting and ending `HEAD`
- Ordered task outcomes
- Commits created
- Validation evidence
- Already-complete evidence
- Blocker and stopping point, if any
- Final repository state

## Additional validation

- Validate that every manifest prompt and authoritative document exists before
  starting the first task.
- Validate every Prompt Contract v2 child with
  `scripts/validate-codex-prompts.sh`.
- Validate that prerequisite and commit-message metadata agree with the accepted
  Roadmap execution plan.
