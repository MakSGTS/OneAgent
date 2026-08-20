# Sprint 10 Subsystems and Composition execution loop

Continue OneAgent development by executing the accepted Sprint 10 prompt suite
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

- `docs/Roadmap.md`, Sprint 10 execution plan;
- `docs/architecture/subsystem-hierarchy-source-investigation.md`;
- `docs/architecture/semantic-model-2.md`;
- `docs/adr/0020-includes-semantics.md`;
- `docs/adr/0024-reference-request-provenance.md`;
- `docs/adr/0025-references-endpoint-validation.md`;
- `docs/adr/0032-subsystem-hierarchy-semantics.md`.

## Sprint objective and current state

Discover repository-proven nested EDT Subsystems, preserve direct hierarchy and
direct metadata membership as canonical Includes, expose deterministic
transitive metadata membership without persisted closure, and retain existing
top-level and unrelated behavior.

The stored plan is not proof of current state. Recheck `HEAD`, Git history,
working tree, Roadmap status, authorities, implementation, tests, fixtures, and
Coverage before Task 01. Sprint 10 must be the unique live target and the
accepted planning baseline must be committed.

Preserve all pre-existing changes. In particular,
`docs/codex/prompts/run-next-sprint.md` and
`docs/roadmap-calendar-forecast.md` were unrelated untracked user files when the
planning baseline was prepared; never stage or modify them unless a later
explicit instruction changes their ownership.

## Ordered task manifest

| Order | Prompt | Required committed prerequisite | Owned outcome | Validation additions | Suggested commit message |
|---:|---|---|---|---|---|
| 1 | `01-implement-subsystem-hierarchy-graph-rules.md` | Accepted Sprint 10 planning baseline | Hierarchy endpoint, cycle validation, transitive membership Query, generic graph/index evidence | Graph Query, Validation, Impact, incremental index | `Implement Sprint 10 subsystem hierarchy graph rules` |
| 2 | `02-parse-nested-subsystem-hierarchy.md` | Task 01 | Recursive source model and strict three-projection hierarchy agreement without graph emission | EDT hierarchy, metadata-object, and content parser tests | `Parse Sprint 10 nested subsystem hierarchy` |
| 3 | `03-emit-nested-subsystem-composition.md` | Tasks 01–02 | Nested nodes, ownership, direct hierarchy/content Includes, provenance, diagnostics, and statistics | EDT hierarchy/Includes and graph validation tests | `Emit Sprint 10 nested subsystem composition` |
| 4 | `04-complete-sprint-10-production-evidence.md` | Tasks 01–03 | Representative source, consumers, index equivalence, Coverage regression, current-state docs | Graph, EDT hierarchy, Includes, Coverage, Semantic Index | `Complete Sprint 10 production evidence` |
| 5 | `05-sprint-10-integration-review.md` | Task 04 and all implementation validation | Independent review, sprint decision, Sprint 9 suite retirement, and Sprint 11 hand-off | Complete focused and workspace gates | `Complete Sprint 10 subsystems and composition review` |

Prompt paths are relative to this directory. Verify every prompt and authority,
and ensure manifest metadata agrees with the live Roadmap before Task 01.

## Verified immediately preceding prompt suite

The planning baseline verified this exact tracked suite:

```text
docs/codex/prompts/sprint-9-roles-access-rights/00-sprint-9-execution-loop.md
docs/codex/prompts/sprint-9-roles-access-rights/01-implement-conditional-access-right-model.md
docs/codex/prompts/sprint-9-roles-access-rights/02-emit-conditional-role-grants.md
docs/codex/prompts/sprint-9-roles-access-rights/03-complete-sprint-9-production-evidence.md
docs/codex/prompts/sprint-9-roles-access-rights/04-sprint-9-integration-review.md
```

Tasks 01–04 must not modify or delete it. Task 05 may retire only these exact
files, only after its non-blocking decision and successful complete validation,
and only atomically with the review artifact and Roadmap transition. Re-enumerate
and compare the tracked inventory before deletion; any mismatch or endangered
untracked file blocks retirement and the final review commit.

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

Run Task 05 only after Tasks 01–04 are committed or proven `already_complete`
and no task-created uncommitted change remains. A blocked review leaves Sprint
10 incomplete, keeps the Sprint 9 suite intact, and leaves Sprint 11 ineligible.
Only a non-blocking review decision plus successful validation may complete
Sprint 10 and authorize the bounded previous-suite retirement.

## Repository Safety

- Follow every applicable `AGENTS.md` and selected Profile safety module.
- Preserve unrelated tracked, staged, ignored, and untracked files.
- Do not modify `.codex/`, rewrite history, or use destructive Git commands.
- Do not add dependencies or broaden scope without explicit authority.

## Final report additions

Report starting and ending `HEAD`, initial and final `git status --short`, every
task result and commit, exact validation evidence, already-complete proof,
blockers, changed and preserved paths, staging state, Sprint 10 decision/state,
every retired Sprint 9 prompt path, and Sprint 11 eligibility.
