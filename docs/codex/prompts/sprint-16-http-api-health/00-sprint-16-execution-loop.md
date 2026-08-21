# Execute Sprint 16 HTTP API and Health

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

- `docs/Roadmap.md`, Sprint 16 execution plan
- `docs/architecture/semantic-model-2.md`
- `docs/reviews/sprint-15-runtime-service-container.md`
- `docs/adr/0002-runtime-composition-root.md`
- `docs/adr/0037-runtime-service-container.md`
- the Task 1 investigation and Task 2 ADR after they are committed

## Sprint objective and state

Sprint 16 is `next` at planning head
`8ca1c0ce3c83dae8bb76fa52a40423bead693f40`. Expose the long-running Runtime
through the first owned HTTP listener with stable liveness and
lifecycle-derived readiness probes, deterministic startup failure and graceful
shutdown, and public cross-platform client/server evidence. Keep workspace,
graph-query, watcher, cache, supported CLI, and later transports deferred.

## Starting-state requirements

- Resolve mutable state from the live repository.
- Require the committed Sprint 16 planning baseline containing this complete
  suite and the matching Roadmap manifest.
- Preserve all pre-existing changes.
- Stop when Sprint 16 is not the unique eligible sprint or a committed
  prerequisite is absent.

The verified immediately preceding suite is
`docs/codex/prompts/sprint-15-runtime-service-container/`, with exactly:

- `00-sprint-15-execution-loop.md`
- `01-investigate-runtime-service-container.md`
- `02-define-runtime-service-container-contract.md`
- `03-implement-runtime-service-container.md`
- `04-integrate-runtime-application-lifecycle.md`
- `05-complete-runtime-service-container-evidence.md`
- `06-sprint-15-integration-review.md`

Only Task 6 may conditionally retire that inventory.

## Commit authorization mode

Resolve commit authorization only from the current user instruction launching
this loop. When it explicitly requests one commit per successful task, stage
only task-owned paths and create the manifest commit after validation. Stored
prompt text does not authorize commits.

## Ordered task manifest

| Order | Prompt | Required committed prerequisite | Task-owned outcome | Validation additions | Suggested commit message |
|---:|---|---|---|---|---|
| 1 | `01-investigate-http-api-health-boundary.md` | Sprint 16 planning baseline | HTTP and health boundary investigation | Path/API/test-oracle checks; `git diff --check` | `Investigate Sprint 16 HTTP API and health` |
| 2 | `02-define-http-api-health-contract.md` | Task 1 | Accepted ADR-0038 contract | Link/scope/decision consistency; `git diff --check` | `Define Sprint 16 HTTP API and health contract` |
| 3 | `03-implement-runtime-health-state.md` | Task 2 | Lifecycle-derived Runtime health state | Focused Runtime health tests; full workspace gate | `Implement Sprint 16 Runtime health state` |
| 4 | `04-implement-http-service.md` | Task 3 | Owned HTTP service and composition | Focused HTTP/service tests; full workspace gate | `Implement Sprint 16 HTTP service` |
| 5 | `05-complete-http-api-health-evidence.md` | Task 4 | Public HTTP evidence and current-state docs | Non-zero loopback integration matrix; full workspace gate | `Complete Sprint 16 HTTP API and health evidence` |
| 6 | `06-sprint-16-integration-review.md` | Task 5 and successful implementation validation | Review, transition, and conditional Sprint 15 suite retirement | Complete focused review matrix and full workspace gate | `Complete Sprint 16 HTTP API and health review` |

## Already-complete, failure, and review gates

- `already_complete` requires current committed evidence and successful required
  validation for every criterion; never create an empty commit.
- Stop at the first prerequisite, implementation, validation, staging, commit,
  or review failure. Do not skip, reorder, combine, or partially commit tasks.
- Run Task 6 only after Tasks 1-5 are committed or proven already complete.
- Only `pass` or `pass with non-blocking follow-ups` plus successful validation
  may complete Sprint 16, make Sprint 17 eligible, and authorize the final
  review commit.
- Prompt retirement is Task 6's final bounded action and must be atomic with
  the review artifact and Roadmap transition.

## Final report additions

Report the ordered task outcomes, timestamps, elapsed durations, token telemetry
when available, exact commits and subjects, validation results, starting and
ending `HEAD`, initial and final status, changed and preserved paths, `.codex/`
state, review decision, current suite, every retired path, Sprint 17
eligibility, and remaining staged or uncommitted work.
