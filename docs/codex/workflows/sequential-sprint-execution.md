# Sequential Sprint Execution Workflow

Use this workflow only when the current user instruction launches an ordered
Prompt Contract v2 sprint suite.

The efficiency gates added for invariant mapping, targeted design review,
numeric scope baselines, staged validation, and remediation stop-loss apply to
Sprint 41 and later suites. Earlier active suites continue under their committed
execution prompts.

## Dispatcher boundary

The master prompt is a dispatcher and durable ledger. It must not execute two
child tasks in one conversation context or retain child implementation
transcripts.

The launch authorizes exactly one sequential fresh-context runner per manifest
child. A runner receives no prior conversation turns and may not delegate. The
independent Review workflow receives its own separate read-only reviewer
authorization.

If a guaranteed fresh context is unavailable, stop before that child and report
its exact prompt path, committed prerequisite, current `HEAD`, branch, and
status. Do not fall back to the accumulated dispatcher context.

## Initial audit

Before dispatch:

1. Record exact `HEAD`, branch, `git status --short`, and the manifest.
2. Reconcile branch, merge, review, remediation, and immediate-push behavior
   with applicable `AGENTS.md`.
3. Resolve commit mode only from the current user instruction.
4. Validate the Sprint 41+ master together with every Prompt Contract v2 child
   using `scripts/validate-codex-prompts.sh`; earlier suites retain their
   committed child-only validation contract.
5. Verify manifest order, prerequisites, outcomes, validation additions,
   suggested commit messages, current Roadmap state, and previous-suite
   inventory against live committed evidence.
6. Record and preserve pre-existing modified, staged, and untracked paths.
7. For Sprint 41 and later, resolve the committed invariant-matrix selector,
   optional design-review gate, and each implementation task's
   `expected_path_count`, `expected_text_line_churn`, expected binary paths, and
   validation budget from the authoritative Roadmap execution plan and master
   prompt. When the design-review gate does not use the suite-specific
   `none|<master-prompt-path>` form, also resolve its child, planned decision
   artifact, and commit boundary. Stop before dispatch when a gate definition or
   baseline is missing or inconsistent; an applicable design-review decision
   remains pending until its manifest child executes.

## Child input contract

Pass only:

- the child prompt;
- exact current `HEAD`, branch, and status;
- its committed prerequisite;
- applicable `AGENTS.md`;
- the selected Profile, base Template, specialized Template, Core, and Workflow
  modules; and
- material admitted by its `Must read` Context Manifest; and
- for Sprint 41 and later implementation tasks, the committed invariant-matrix
  selector, `expected_path_count`, `expected_text_line_churn`, expected binary
  paths, validation budget, exact task start commit, and, when applicable, the
  committed targeted design-review `pass` artifact and commit.

Do not preload `Lookup on demand` material. Do not pass previous child
transcripts, implementation reasoning, conclusions, or complete logs.

The child performs Context Management preflight before substantive
investigation and returns a context-budget blocker if admission exceeds the hard
limit.

## Ordered task loop

For each manifest entry:

1. Enforce the committed prerequisite and clean task-owned state.
   Before the first Sprint 41 or later production implementation child whose
   efficiency contract has a design-review gate, require the predeclared
   artifact to record `pass` and resolve to the separate committed design-review
   boundary from the manifest. No artifact is required when the record is
   the suite-specific `design_review_gate: none|<master-prompt-path>` form.
2. Start one guaranteed fresh-context runner with the child input contract.
3. Require the runner to print its Change Contract before edits.
4. Implement only the child-owned outcome and preserve unrelated work.
5. Run focused validation and the canonical checks triggered by
   `docs/codex/core/validation.md`.
6. Recheck acceptance, exclusions, diff, meaningful test counts, and state. For
   Sprint 41 and later, also calculate actual scope from the exact task start.
   Form the unique task-owned path set as the union of tracked paths from
   `git diff --name-only <task-start>` and untracked paths from
   `git ls-files --others --exclude-standard`, after subtracting the preserved
   initial untracked inventory recorded before the task. Include a pre-existing
   path only when the current task explicitly adopts it and records that scope
   transition. Textual line churn is the sum of numeric additions plus deletions
   from `git diff --numstat <task-start>` plus the line count of each remaining
   task-owned untracked text file as an addition from empty. Treat an empty
   untracked file as text with zero churn; classify every other untracked file
   with the repository's Git attributes and the same NUL-byte convention as
   Git, and enumerate tracked or task-owned untracked binary paths separately.
   Compare all values with the committed baseline before staging.
7. If commit mode is authorized, stage only enumerated paths and create exactly
   one logical commit with the manifest message. Otherwise do not stage or
   commit.
8. Return a compact outcome containing status, starting and ending `HEAD`,
   changed paths, validation summary, commit, measured token telemetry when
   available, retained-log paths, and blocker when any.
9. In the dispatcher context, independently verify repository state, the
   committed path inventory, and the immediate push required by `AGENTS.md`.
   Stop on push failure before another change or commit.
10. Append the verified outcome to the ledger and discard the child context.

Proceed only when the next committed prerequisite is satisfied and no
uncommitted task-created change remains.

## Scope and remediation stop-loss

For Sprint 41 and later, if actual unique changed paths exceed
`2 * expected_path_count`, textual additions plus deletions exceed
`2 * expected_text_line_churn`, or a binary path is absent from the expected
inventory, stop before further implementation, full validation, or commit. A
zero baseline admits no change in that dimension. Report the exact counts,
commands, revised scope, risk, and validation budget, and obtain renewed user
agreement before continuing.

For Sprint 41 and later, count architecture-level blocked review outcomes across
the sprint. An architecture-level blocker is a confirmed finding that
invalidates or leaves unmapped an accepted production owner, authority,
boundary, ordering, lifecycle, failure, or cross-layer invariant; formatting,
an isolated regression, missing command output, and transient validation
failure do not qualify. After the second such blocker, stop patch-by-patch
remediation. Re-audit the complete ADR-invariant matrix against the latest
production paths and negative oracles, revise the implementation and validation
plan, and obtain renewed user agreement before another remediation. A local fix
for only the latest finding is not sufficient evidence to resume.

## Ledger

Keep one compact row per child:

| Order | Prompt | Status | Start HEAD | End HEAD | Validation | Commit/push | Tokens | Logs |
|---:|---|---|---|---|---|---|---|---|

Valid statuses are `completed`, `already_complete`, `blocked`, `failed`, and
`not_started`. Use `already_complete` only when committed live evidence and
successful required validation prove every criterion. Never create an empty
commit.

## Failure behavior

Stop after the first prerequisite, context admission, implementation,
validation, staging, commit, push, review, or artifact-consistency failure.
Preserve the failed task's evidence and diff, do not start dependent tasks, and
do not repair unrelated changes. Report the exact command and result, affected
paths, recoverability, and required next action.

## Integration review

Dispatch the final Review child only after all predecessors are committed or
proven `already_complete` and required validation succeeds.

The Review child follows `docs/codex/workflows/review.md`: one independent
fresh-context read-only reviewer, independent primary validation, evidence
reconciliation, and same-reviewer artifact consistency. It does not receive
implementation transcripts or an expected decision.

Only a non-blocking effective decision plus successful required validation may
transition the sprint to `completed` and make the next sprint eligible.

## Previous-suite retirement

The final Review child may retire the exact immediately preceding tracked suite
only after a non-blocking decision, complete validation, state transition,
authorized commit mode, unchanged planned inventory, and reviewer artifact
consistency.

Re-enumerate the directory, refuse untracked or out-of-bound targets, delete
only explicitly listed tracked prompt files through the normal file-editing
mechanism, and include review artifact, state transition, and deletions in the
single review commit. Never use recursive deletion, globs, `git clean`, or an
extra cleanup commit. Preserve the current suite, reusable bootstrap, and
non-adjacent suites.

## Final report

Report the ledger once, starting and ending `HEAD`, branch and state, commits
and pushes, validation summaries, context preflights, blockers or
already-complete evidence, changed and preserved paths, review result,
retirement result, measured token telemetry or `недоступно`, retained-log
paths, and final `git status --short`.
