# Sprint 9 Roles and Access Rights execution loop

Continue OneAgent development by executing the accepted Sprint 9 prompt suite
strictly in dependency order.

## Reporting

- Communicate with the user in Russian.
- Keep code, identifiers, repository documentation, comments, tests, errors,
  public APIs, prompt text, and commit messages in English.
- Report only live repository evidence or accepted architecture.

## Template and workflow

- `docs/codex/templates/sprint-execution-loop.md`
- `docs/codex/workflows/sequential-sprint-execution.md`

Read both files, `docs/codex/README.md`, and every Profile, Template, Core, and
Workflow module selected by the current child prompt completely before acting.

## Canonical authorities

- `docs/Roadmap.md`, Sprint 9 execution plan;
- `docs/architecture/semantic-model-2.md`;
- `docs/adr/0019-grants-semantics.md`;
- `docs/adr/0024-reference-request-provenance.md`;
- `docs/adr/0025-references-endpoint-validation.md`;
- `docs/adr/0031-conditional-grants-semantics.md`.

## Sprint objective and current state

Preserve optional EDT row-restriction conditions as deterministic typed
AccessRight content and conditional direct Grants without interpreting them as
deny, inheritance, expression semantics, or effective authorization.

The stored plan is not proof of current state. Recheck `HEAD`, Git history,
working tree, Roadmap status, authorities, implementation, tests, fixtures, and
Coverage before Task 01. Sprint 9 must be the unique live target and the
accepted planning baseline must be committed.

Preserve all pre-existing changes. In particular,
`docs/codex/prompts/run-next-sprint.md` and
`docs/roadmap-calendar-forecast.md` were unrelated untracked user files when the
planning baseline was prepared; never stage or modify them unless a later
explicit instruction changes their ownership.

## Ordered task manifest

| Order | Prompt | Required committed prerequisite | Owned outcome | Validation additions | Suggested commit message |
|---:|---|---|---|---|---|
| 1 | `01-implement-conditional-access-right-model.md` | Accepted Sprint 9 planning baseline | Typed conditional AccessRight payload and identity with unconditional compatibility | AccessRight, node, Query, Diff, Impact, Coverage | `Implement Sprint 9 conditional access rights` |
| 2 | `02-emit-conditional-role-grants.md` | Task 01 | Conditional private resolution observations, provenance, AccessRight, References, and Grants emission | EDT role-right parser, Grants builder, graph validation | `Emit Sprint 9 conditional role grants` |
| 3 | `03-complete-sprint-9-production-evidence.md` | Tasks 01–02 | Full-builder consumers, index equivalence, Coverage regression, and synchronized current-state docs | Graph, EDT Grants, Coverage, Semantic Index | `Complete Sprint 9 production evidence` |
| 4 | `04-sprint-9-integration-review.md` | Task 03 and all implementation validation | Independent review, sprint decision, and Sprint 10 hand-off | Complete focused and workspace gates | `Complete Sprint 9 roles and access rights review` |

Prompt paths are relative to this directory. Verify every prompt and authority,
and ensure manifest metadata agrees with the live Roadmap before Task 01.

## Commit authorization mode

Stored prompt text never authorizes staging or committing. Determine commit
mode only from the current instruction launching this loop. When authorized,
stage only enumerated task-owned paths and create exactly one logical commit per
completed task with the manifest message. Never use broad staging. Without
authorization, stop when the next gate requires a committed prerequisite.

## Task loop and already-complete policy

For each task, refresh live evidence, enforce its gate, print its Change
Contract, execute only its owned outcome, run focused and required full
validation, inspect the diff, and record the result before continuing.

Use `already_complete` only when current committed evidence plus successful
required validation proves every criterion. Record the proving commit or
baseline and do not create an empty commit.

## Failure and review gates

Stop after the first prerequisite, implementation, validation, staging, commit,
or review failure. Do not reorder, combine, skip, or partially commit dependent
tasks. Preserve failed-task evidence and unrelated work.

Run Task 04 only after Tasks 01–03 are committed or proven `already_complete`
and no task-created uncommitted change remains. A blocked review leaves Sprint
9 incomplete and Sprint 10 ineligible. Only a non-blocking review decision plus
successful validation may complete Sprint 9.

## Repository Safety

- Follow every applicable `AGENTS.md` and selected Profile safety module.
- Preserve unrelated tracked, staged, ignored, and untracked files.
- Do not modify `.codex/`, rewrite history, or use destructive Git commands.
- Do not add dependencies or broaden scope without explicit authority.

## Final report additions

Report starting and ending `HEAD`, initial and final `git status --short`, every
task result and commit, exact validation evidence, already-complete proof,
blockers, changed and preserved paths, staging state, Sprint 9 decision/state,
and Sprint 10 eligibility.
