# Execute Sprint 17 Workspace Service

Continue OneAgent development.

## Reporting

- User-visible reports: Russian.
- Repository content and commit messages: English.
- Report only live repository evidence and successful command results.

## Template and workflow

- `docs/codex/templates/sprint-execution-loop.md`
- `docs/codex/workflows/sequential-sprint-execution.md`

Read both files completely before execution, including every Profile, Template,
Core module, Workflow, ADR, and architecture document selected by each child
task.

## Canonical authorities

- `docs/Roadmap.md`, Sprint 17 execution plan
- `docs/architecture/semantic-model-2.md`
- `docs/reviews/sprint-16-http-api-health.md`
- `docs/adr/0002-runtime-composition-root.md`
- `docs/adr/0004-filesystem-workspace-discovery.md`
- `docs/adr/0036-designer-xml-adapter.md`
- `docs/adr/0037-runtime-service-container.md`
- `docs/adr/0038-http-api-health.md`
- the Task 1 investigation and Task 2 ADR after they are committed

## Sprint objective and state

Sprint 17 is `next` at planning head resolved from the live repository. Add the
first Runtime-owned workspace lifecycle and semantic-build orchestration service
for repository-owned EDT and Designer XML inputs, with deterministic startup,
immutable published state, failure, shutdown, and public integration evidence.
Keep graph-query APIs, file watching, persistence, supported CLI behavior, and
later transports deferred.

## Starting-state requirements

- Resolve mutable state from the live repository.
- Require the committed Sprint 17 planning baseline containing this complete
  suite and the matching Roadmap manifest.
- Preserve all pre-existing changes.
- Stop when Sprint 17 is not the unique eligible sprint or a committed
  prerequisite is absent.

The verified immediately preceding suite is
`docs/codex/prompts/sprint-16-http-api-health/`, with exactly:

- `00-sprint-16-execution-loop.md`
- `01-investigate-http-api-health-boundary.md`
- `02-define-http-api-health-contract.md`
- `03-implement-runtime-health-state.md`
- `04-implement-http-service.md`
- `05-complete-http-api-health-evidence.md`
- `06-sprint-16-integration-review.md`

Only Task 6 may conditionally retire that inventory.

## Commit authorization mode

Resolve commit authorization only from the current user instruction launching
this loop. When it explicitly requests one commit per successful task, stage
only task-owned paths and create the manifest commit after validation. Stored
prompt text does not authorize commits.

## Ordered task manifest

| Order | Prompt | Required committed prerequisite | Task-owned outcome | Validation additions | Suggested commit message |
|---:|---|---|---|---|---|
| 1 | `01-investigate-workspace-service-boundary.md` | Sprint 17 planning baseline | Workspace discovery, build, state, lifecycle, consumer, failure, and testability investigation | Path/API/fixture/test-oracle checks; `git diff --check` | `Investigate Sprint 17 Workspace service` |
| 2 | `02-define-workspace-service-contract.md` | Task 1 | Accepted ADR-0039 Workspace service contract | Link/scope/decision consistency; `git diff --check` | `Define Sprint 17 Workspace service contract` |
| 3 | `03-implement-workspace-snapshot.md` | Task 2 | Source-neutral immutable Workspace snapshot and build dispatch boundary | Focused snapshot/build tests; full workspace gate | `Implement Sprint 17 Workspace snapshot` |
| 4 | `04-implement-workspace-service.md` | Task 3 | Runtime-owned Workspace service and composition | Focused service/lifecycle/failure tests; full workspace gate | `Implement Sprint 17 Workspace service` |
| 5 | `05-complete-workspace-service-evidence.md` | Task 4 | Public EDT/Designer XML orchestration evidence and current-state docs | Non-zero public integration matrix; full workspace gate | `Complete Sprint 17 Workspace service evidence` |
| 6 | `06-sprint-17-integration-review.md` | Task 5 and successful implementation validation | Review, transition, and conditional Sprint 16 suite retirement | Complete focused/full matrix and inventory checks | `Complete Sprint 17 Workspace service review` |

## Initial audit additions

- Record exact Sprint start time, `HEAD`, `git status --short`, relevant history,
  Roadmap state, and available token telemetry.
- Verify every prompt and authority path in the manifest.
- Verify that Sprint 16 has a committed non-blocking review and Sprint 17 is the
  unique eligible target.
- Re-enumerate the exact Sprint 16 tracked and filesystem prompt inventory, and
  stop on ambiguity or an endangered untracked file.

## Task-loop additions

- Record start/end timestamps, elapsed time, exact validation, commit, and final
  status for every task.
- Do not combine discovery, build dispatch, snapshot publication, Runtime
  service ownership, or public evidence across task boundaries unless ADR-0039
  proves that a narrower implementation boundary cannot be coherent.
- A zero-match filtered test is not evidence.

## Already-complete policy additions

Use `already_complete` only when committed live evidence plus the task's exact
validation proves every acceptance criterion. Historical Sprint 16 deferrals do
not prove Sprint 17 behavior. Do not create an empty commit.

## Failure and integration-review gates

Stop after the first missing prerequisite, implementation, validation, staging,
commit, or review failure. Run Task 6 only after Tasks 1-5 are committed or
proven `already_complete`. Only a non-blocking Task 6 decision plus successful
complete validation may complete Sprint 17, make Sprint 18 eligible, and retire
the exact Sprint 16 suite.

## Final report additions

Report the ordered task table, exact commits and subjects, start/end/elapsed
times, available token telemetry, validation results, changed and preserved
paths, integration-review decision, Sprint 16 suite retirement result, Sprint
18 eligibility, `.codex/` preservation, and final `git status --short`.
