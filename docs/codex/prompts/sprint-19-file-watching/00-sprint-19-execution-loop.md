# Execute Sprint 19 File Watching

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

- `docs/Roadmap.md`, Sprint 19 execution plan
- `docs/Architecture.md`
- `docs/architecture/semantic-model-2.md`
- `docs/reviews/sprint-18-graph-query-api.md`
- `docs/adr/0002-runtime-composition-root.md`
- `docs/adr/0037-runtime-service-container.md`
- `docs/adr/0038-http-api-health.md`
- `docs/adr/0039-workspace-service.md`
- `docs/adr/0040-graph-query-api.md`
- the Task 1 investigation and Task 2 ADR after they are committed

## Sprint objective and state

Sprint 19 is `next` at the planning head resolved from the live repository.
Detect relevant configured-Workspace changes and connect them to deterministic
Runtime-owned complete rebuild orchestration with atomic valid snapshot
replacement, explicit failure and recovery, truthful lifecycle behavior,
stable Graph Query observation, and public production evidence. Keep graph and
adapter semantics, incremental mutation, persistence, the supported CLI, and
later integrations deferred.

## Starting-state requirements

- Resolve mutable state from the live repository.
- Require the committed Sprint 19 planning baseline containing this complete
  suite and the matching Roadmap manifest.
- Preserve all pre-existing changes.
- Stop when Sprint 19 is not the unique eligible sprint or a committed
  prerequisite is absent.

The verified immediately preceding suite is
`docs/codex/prompts/sprint-18-graph-query-api/`, with exactly:

- `00-sprint-18-execution-loop.md`
- `01-investigate-graph-query-api-boundary.md`
- `02-define-graph-query-api-contract.md`
- `03-implement-graph-query-service.md`
- `04-implement-graph-query-http-api.md`
- `05-complete-graph-query-api-evidence.md`
- `06-sprint-18-integration-review.md`

Only Task 6 may conditionally retire that inventory.

## Commit authorization mode

Resolve commit authorization only from the current user instruction launching
this loop. When it explicitly requests one commit per successful task, stage
only task-owned paths and create the manifest commit after validation. Stored
prompt text does not authorize commits.

## Ordered task manifest

| Order | Prompt | Required committed prerequisite | Task-owned outcome | Validation additions | Suggested commit message |
|---:|---|---|---|---|---|
| 1 | `01-investigate-file-watching-boundary.md` | Sprint 19 planning baseline | Filesystem, Workspace build/publication, Runtime lifecycle, consumer, dependency, platform, fixture, and deterministic-test investigation | Path/API/dependency/platform/fixture/oracle checks; `git diff --check` | `Investigate Sprint 19 File Watching` |
| 2 | `02-define-file-watching-contract.md` | Task 1 | Accepted ADR-0041 ownership, change, coalescing, rebuild, publication, failure/recovery, lifecycle, dependency, and compatibility contract | Link/scope/decision consistency; `git diff --check` | `Define Sprint 19 File Watching contract` |
| 3 | `03-implement-file-change-watching.md` | Task 2 | Runtime-owned normalized file-change observation, bounded coordination, cancellation, cleanup, and focused evidence | Non-zero focused observation/lifecycle tests; full workspace gate | `Implement Sprint 19 file change watching` |
| 4 | `04-integrate-workspace-rebuilds.md` | Task 3 | Serialized complete rebuilds, atomic valid publication, accepted failure/recovery, and focused evidence | Non-zero rebuild/publication/query tests; full workspace gate | `Integrate Sprint 19 Workspace rebuilds` |
| 5 | `05-complete-file-watching-evidence.md` | Task 4 | Public production change/rebuild/query/failure/recovery/shutdown matrix and current-state docs | Non-zero public integration matrix; full workspace gate | `Complete Sprint 19 File Watching evidence` |
| 6 | `06-sprint-19-integration-review.md` | Task 5 and successful implementation validation | Review, transition, and conditional Sprint 18 suite retirement | Complete focused/full matrix and inventory checks | `Complete Sprint 19 File Watching review` |

## Initial audit additions

- Record exact Sprint start time, `HEAD`, `git status --short`, relevant history,
  Roadmap state, and available token telemetry.
- Verify every prompt and authority path in the manifest.
- Verify that Sprint 18 has a committed non-blocking review and Sprint 19 is the
  unique eligible target.
- Re-enumerate the exact Sprint 18 tracked and filesystem prompt inventory, and
  stop on ambiguity or an endangered untracked file.

## Task-loop additions

- Record start/end timestamps, elapsed time, exact validation, commit, and final
  status for every task.
- Do not combine investigation, architecture, observation, rebuild
  orchestration, or public production evidence across task boundaries.
- Preserve canonical graph and adapter semantics, immutable complete snapshot
  publication, lifecycle-derived readiness, and the Sprint 16/18 HTTP wire
  contracts.
- Do not add a production dependency without explicit user approval, even when
  ADR-0041 accepts it as the required implementation choice.
- A zero-match filtered test is not evidence.

## Already-complete policy additions

Use `already_complete` only when committed live evidence plus the task's exact
validation proves every acceptance criterion. Existing one-shot Workspace
builds and snapshot watches do not prove live file watching or rebuild
orchestration. Do not create an empty commit.

## Failure and integration-review gates

Stop after the first missing prerequisite, implementation, validation, staging,
commit, or review failure. Run Task 6 only after Tasks 1-5 are committed or
proven `already_complete`. Only a non-blocking Task 6 decision plus successful
complete validation may complete Sprint 19, make Sprint 20 eligible, and retire
the exact Sprint 18 suite.

## Final report additions

Report the ordered task table, exact commits and subjects, start/end/elapsed
times, available token telemetry, validation results, changed and preserved
paths, integration-review decision, Sprint 18 suite retirement result, Sprint
20 eligibility, `.codex/` preservation, and final `git status --short`.
