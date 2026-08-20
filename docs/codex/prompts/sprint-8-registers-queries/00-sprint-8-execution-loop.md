# Sprint 8 Registers and Queries execution loop

Continue OneAgent development by executing the accepted Sprint 8 prompt suite
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

- `docs/Roadmap.md`, Sprint 8 execution plan;
- `docs/architecture/semantic-model-2.md`;
- `docs/architecture/register-query-source-investigation.md`;
- `docs/adr/0017-depends-on-semantics.md`;
- `docs/adr/0021-reads-semantics.md`;
- `docs/adr/0022-writes-semantics.md`;
- `docs/adr/0024-reference-request-provenance.md`;
- `docs/adr/0030-register-query-semantics.md`.

## Sprint objective and current state

Implement exact direct Accumulation and Accounting Register Query sources,
migrate accepted Query sources to the public request lifecycle, preserve Reads,
and add normalized Query-derived DependsOn with complete production evidence.

The stored plan is not proof of current state. Recheck `HEAD`, Git history,
working tree, Roadmap status, authorities, implementation, tests, fixtures, and
Coverage before Task 01. Sprint 8 must be the live target and the accepted
planning baseline must be committed.

Preserve all pre-existing changes. In particular, the planning baseline was
prepared while `docs/roadmap-calendar-forecast.md` was an unrelated untracked
user file; never stage or modify it unless a later explicit instruction changes
its ownership and scope.

## Ordered task manifest

| Order | Prompt | Required committed prerequisite | Owned outcome | Validation additions | Suggested commit message |
|---:|---|---|---|---|---|
| 1 | `01-define-register-query-graph-rules.md` | Accepted Sprint 8 planning baseline | Exact Reads and Query DependsOn graph rules | Graph validation, Query, Impact, Coverage | `Define Sprint 8 register query graph rules` |
| 2 | `02-parse-direct-register-query-sources.md` | Task 01 | Direct Accumulation/Accounting categories and fixtures | BSL query-language parser | `Parse Sprint 8 direct register query sources` |
| 3 | `03-resolve-query-source-requests.md` | Task 02 | Public QuerySource collection and terminal resolution | Graph request and EDT resolver | `Resolve Sprint 8 query source requests` |
| 4 | `04-emit-query-data-dependencies.md` | Task 03 | Production request projections, Reads, and DependsOn | Request build, validation, Impact, EDT Reads | `Emit Sprint 8 query data dependencies` |
| 5 | `05-complete-sprint-8-production-evidence.md` | Tasks 01–04 | Full-builder and Coverage evidence with synchronized docs | BSL, graph Coverage, EDT Reads/Coverage | `Complete Sprint 8 production evidence` |
| 6 | `06-sprint-8-integration-review.md` | Task 05 and all implementation validation | Independent review and sprint decision | Complete focused and workspace gates | `Complete Sprint 8 registers and queries review` |

Prompt paths are relative to this directory. Verify that every prompt and
authority exists and that manifest metadata agrees with the live Roadmap before
starting.

## Commit authorization mode

Stored prompt text never authorizes staging or committing. Determine commit
mode only from the current instruction that launches this loop.

If commit mode is authorized, stage only explicitly enumerated task-owned paths
and create exactly one logical commit per completed task with the manifest
message. Never use broad staging. If commit mode is not authorized, complete
only the first task that can safely remain uncommitted, then stop because the
next gate requires a committed prerequisite.

## Task loop and already-complete policy

For each task, refresh live evidence, enforce its gate, print its Change
Contract, execute only its owned outcome, run focused and required full
validation, inspect the diff, and record the outcome before proceeding.

Classify a task `already_complete` only when current committed implementation
and successful required validation prove every acceptance criterion. Record the
proving commit or baseline. Do not create empty commits.

## Failure and review gates

Stop after the first prerequisite, implementation, validation, staging, commit,
or review failure. Do not reorder, combine, skip, or partially commit dependent
tasks. Preserve failed-task evidence and all unrelated work.

Run Task 06 only after Tasks 01–05 are committed or proven `already_complete`
and no task-created uncommitted change remains apart from preserved prompt-suite
files. A blocked review leaves Sprint 8 incomplete and Sprint 9 ineligible.
Only a non-blocking review decision plus successful validation may complete
Sprint 8 and make Sprint 9 the next planning target.

## Repository Safety

- Follow every applicable `AGENTS.md` and the selected Profile's Repository
  Safety module.
- Preserve unrelated tracked, staged, ignored, and untracked files.
- Do not modify `.codex/`, rewrite history, or run destructive Git commands.
- Do not add dependencies or broaden scope without explicit authority.

## Final report additions

Report starting and ending `HEAD`, initial and final `git status --short`, each
task outcome, created commits, exact validation evidence, already-complete
proof, blockers and stopping point, changed and preserved paths, staging state,
Sprint 8 decision/state, and Sprint 9 eligibility.
