# Sequential Sprint Execution Workflow

Use this workflow only when the current user instruction explicitly requests
execution of an ordered sprint prompt suite or master prompt.

## Required initial audit

- Record the exact starting `HEAD` and `git status --short`.
- Read the current Roadmap sprint state, ordered task plan, accepted ADRs, and
  every prompt named by the execution manifest.
- Verify that the manifest order, task prerequisites, expected outcomes,
  validation additions, and suggested commit messages agree with the live
  repository rather than a historical prompt baseline.
- Classify pre-existing modified, staged, and untracked paths and preserve them.
- Resolve commit authorization from the current user instruction. Stored prompt
  text by itself is not authorization to stage or commit.

## Ordered task loop

For each manifest entry, in order:

1. Refresh `HEAD`, `git status --short`, relevant repository evidence, and the
   task's authoritative documents.
2. Enforce the task prerequisite gate against committed repository evidence.
   Stop before edits when the gate is not satisfied.
3. Read the selected task prompt, Profile, Template, required Core modules, and
   Workflow modules.
4. Print the task Change Contract before implementation when changes are
   expected.
5. Implement only the task-owned coherent outcome and preserve unrelated or
   pre-existing work.
6. Run task-focused validation followed by every package or workspace check
   required by `docs/codex/core/validation.md`.
7. Recheck acceptance criteria, scope exclusions, diff contents, and final task
   state. A zero-match test filter is not passing evidence.
8. If commit mode is authorized, stage only explicitly enumerated task-owned
   paths and create exactly one logical commit with the manifest message. Never
   use broad staging. If commit mode is not authorized, do not stage or commit.
9. Record the outcome, validation result, commit hash when created, ending
   `HEAD`, and `git status --short`.
10. Proceed only when the next task's committed prerequisite is satisfied and
    no uncommitted task-created change remains. If commit mode is not authorized
    and the next gate requires a commit, stop after reporting the completed
    uncommitted task.

## Already-complete outcome

Classify a task as `already_complete` only when current committed repository
evidence and successful required validation prove every acceptance criterion.
Record the proving commit or baseline. Do not create an empty commit and do not
use a historical prompt claim as proof.

## Failure and blocker behavior

- Stop the sequence immediately after an implementation, validation, staging,
  commit, or prerequisite failure.
- Preserve the failed task's evidence and report the exact command and result.
- Do not skip a blocked task or start a dependent task.
- Do not repair unrelated pre-existing changes to obtain a clean status.
- If accepted architecture cannot be implemented, follow the blocker procedure
  in `docs/codex/workflows/implementation.md`.

## Integration review gate

Run the sprint integration review only after every implementation task is
committed or proven `already_complete` according to the manifest. The review
may create its explicitly authorized evidence artifact and Roadmap transition,
but it must not silently fix findings in the review change.

Only a non-blocking review decision plus successful required validation may
transition a sprint to `completed` and make the next sprint eligible for
planning.

## Final execution report

Report the ordered task outcome table, starting and ending `HEAD`, every created
commit, exact validation results, blockers or already-complete evidence, changed
and preserved paths, whether anything remains staged, and final
`git status --short`.
