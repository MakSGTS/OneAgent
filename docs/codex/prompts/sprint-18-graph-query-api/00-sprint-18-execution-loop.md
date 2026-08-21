# Execute Sprint 18 Graph Query API

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

- `docs/Roadmap.md`, Sprint 18 execution plan
- `docs/architecture/semantic-model-2.md`
- `docs/reviews/sprint-17-workspace-service.md`
- `docs/adr/0002-runtime-composition-root.md`
- `docs/adr/0008-semantic-model-2-knowledge-graph.md`
- `docs/adr/0026-semantic-index-boundary.md`
- `docs/adr/0037-runtime-service-container.md`
- `docs/adr/0038-http-api-health.md`
- `docs/adr/0039-workspace-service.md`
- the Task 1 investigation and Task 2 ADR after they are committed

## Sprint objective and state

Sprint 18 is `next` at the planning head resolved from the live repository.
Expose the first stable bounded graph and semantic query API over one selected
published Workspace configuration, with deterministic transport-neutral
results, exact HTTP compatibility, truthful lifecycle behavior, explicit
limits and errors, and public production evidence. Keep graph semantic changes,
file watching, persistence, the supported CLI, and later transports deferred.

## Starting-state requirements

- Resolve mutable state from the live repository.
- Require the committed Sprint 18 planning baseline containing this complete
  suite and the matching Roadmap manifest.
- Preserve all pre-existing changes.
- Stop when Sprint 18 is not the unique eligible sprint or a committed
  prerequisite is absent.

The verified immediately preceding suite is
`docs/codex/prompts/sprint-17-workspace-service/`, with exactly:

- `00-sprint-17-execution-loop.md`
- `01-investigate-workspace-service-boundary.md`
- `02-define-workspace-service-contract.md`
- `03-implement-workspace-snapshot.md`
- `04-implement-workspace-service.md`
- `05-complete-workspace-service-evidence.md`
- `06-sprint-17-integration-review.md`

Only Task 6 may conditionally retire that inventory.

## Commit authorization mode

Resolve commit authorization only from the current user instruction launching
this loop. When it explicitly requests one commit per successful task, stage
only task-owned paths and create the manifest commit after validation. Stored
prompt text does not authorize commits.

## Ordered task manifest

| Order | Prompt | Required committed prerequisite | Task-owned outcome | Validation additions | Suggested commit message |
|---:|---|---|---|---|---|
| 1 | `01-investigate-graph-query-api-boundary.md` | Sprint 18 planning baseline | Graph-query, snapshot, Runtime HTTP, consumer, compatibility, dependency, fixture, and testability investigation | Path/API/type/fixture/test-oracle checks; `git diff --check` | `Investigate Sprint 18 Graph Query API` |
| 2 | `02-define-graph-query-api-contract.md` | Task 1 | Accepted ADR-0040 Graph Query API contract | Link/scope/decision consistency; `git diff --check` | `Define Sprint 18 Graph Query API contract` |
| 3 | `03-implement-graph-query-service.md` | Task 2 | Transport-neutral selected-snapshot query boundary, owned results, errors, and limits | Non-zero focused query-service tests; full workspace gate | `Implement Sprint 18 Graph Query service` |
| 4 | `04-implement-graph-query-http-api.md` | Task 3 | Accepted versioned Graph Query HTTP routes, composition, schemas, and error mapping | Non-zero focused HTTP and loopback tests; full workspace gate | `Implement Sprint 18 Graph Query HTTP API` |
| 5 | `05-complete-graph-query-api-evidence.md` | Task 4 | Public production EDT/Designer XML query matrix and current-state docs | Non-zero public integration matrix; full workspace gate | `Complete Sprint 18 Graph Query API evidence` |
| 6 | `06-sprint-18-integration-review.md` | Task 5 and successful implementation validation | Review, transition, and conditional Sprint 17 suite retirement | Complete focused/full matrix and inventory checks | `Complete Sprint 18 Graph Query API review` |

## Initial audit additions

- Record exact Sprint start time, `HEAD`, `git status --short`, relevant history,
  Roadmap state, and available token telemetry.
- Verify every prompt and authority path in the manifest.
- Verify that Sprint 17 has a committed non-blocking review and Sprint 18 is the
  unique eligible target.
- Re-enumerate the exact Sprint 17 tracked and filesystem prompt inventory, and
  stop on ambiguity or an endangered untracked file.

## Task-loop additions

- Record start/end timestamps, elapsed time, exact validation, commit, and final
  status for every task.
- Do not combine architecture, transport-neutral query behavior, HTTP mapping,
  or public production evidence across task boundaries.
- Preserve canonical graph facts, immutable Workspace snapshots, lifecycle
  readiness, and the exact Sprint 16 health wire contract.
- A zero-match filtered test is not evidence.

## Already-complete policy additions

Use `already_complete` only when committed live evidence plus the task's exact
validation proves every acceptance criterion. Existing in-process graph queries
and Sprint 17 observation do not prove a stable Runtime API. Do not create an
empty commit.

## Failure and integration-review gates

Stop after the first missing prerequisite, implementation, validation, staging,
commit, or review failure. Run Task 6 only after Tasks 1-5 are committed or
proven `already_complete`. Only a non-blocking Task 6 decision plus successful
complete validation may complete Sprint 18, make Sprint 19 eligible, and retire
the exact Sprint 17 suite.

## Final report additions

Report the ordered task table, exact commits and subjects, start/end/elapsed
times, available token telemetry, validation results, changed and preserved
paths, integration-review decision, Sprint 17 suite retirement result, Sprint
19 eligibility, `.codex/` preservation, and final `git status --short`.
