# Execute Sprint 15 Runtime Service Container

Continue OneAgent development.

## Reporting

- User-visible reports: Russian.
- Repository content and commit messages: English.
- Report only live repository evidence and successful command results.

## Template and workflow

- `docs/codex/templates/sprint-execution-loop.md`
- `docs/codex/workflows/sequential-sprint-execution.md`

Read both completely before execution, including every Profile, Template, Core
module, Workflow, ADR, and architecture document selected by each child task.

## Canonical authorities

- `docs/Roadmap.md`, Sprint 15 execution plan
- `docs/reviews/v0.3-release-review.md`
- `docs/adr/0002-runtime-composition-root.md`
- `docs/architecture/semantic-model-2.md`
- `docs/Architecture.md`
- the Task 1 investigation and Task 2 ADR after they are committed

## Sprint objective and state

Sprint 15 is `next` at planning head
`bac838be07bbf9b9686e60419397e91e702adec1`. Establish the first accepted
long-running Runtime service container with explicit service/task ownership,
deterministic startup and shutdown, cancellation and failure propagation, and
an asynchronously running composition root. Do not pull HTTP, workspace
orchestration, graph-query APIs, file watching, persistence, or supported CLI
behavior forward from Sprints 16–21.

## Starting-state requirements

- Resolve mutable state from the live repository.
- Require the committed Sprint 15 planning baseline containing this complete
  suite and the matching Roadmap manifest.
- Require committed Runtime Service framework contracts at `bac838be` or a
  descendant that preserves them.
- Preserve all pre-existing changes.
- Stop when Sprint 15 is not the unique eligible sprint or a committed
  prerequisite is absent.

The verified immediately preceding suite is
`docs/codex/prompts/sprint-14-designer-xml-adapter/`, with exactly:

- `00-sprint-14-execution-loop.md`
- `01-investigate-designer-xml-source-contracts.md`
- `02-define-designer-xml-adapter-contract.md`
- `03-implement-designer-xml-discovery.md`
- `04-parse-designer-xml-metadata.md`
- `05-parse-designer-xml-modules.md`
- `06-emit-designer-xml-semantics.md`
- `07-complete-sprint-14-conformance-evidence.md`
- `08-sprint-14-integration-review.md`

Only Task 6 may conditionally retire that inventory.

## Commit authorization mode

Resolve commit authorization only from the current user instruction launching
this loop. When it explicitly requests one commit per successful task, stage
only task-owned paths and create the manifest commit after validation. Stored
prompt text does not authorize commits.

## Ordered task manifest

| Order | Prompt | Required committed prerequisite | Task-owned outcome | Validation additions | Suggested commit message |
|---:|---|---|---|---|---|
| 1 | `01-investigate-runtime-service-container.md` | Sprint 15 planning baseline | Runtime/service evidence and decision questions | Runtime baseline tests; link/path checks; `git diff --check` | `Investigate Sprint 15 Runtime service container` |
| 2 | `02-define-runtime-service-container-contract.md` | Task 1 | Accepted ADR-0037 lifecycle and ownership contract | Link/scope/decision consistency; `git diff --check` | `Define Sprint 15 Runtime service container contract` |
| 3 | `03-implement-runtime-service-container.md` | Task 2 | Owned service container and cancellation primitives | Focused Runtime container tests; full workspace gate | `Implement Sprint 15 Runtime service container` |
| 4 | `04-integrate-runtime-application-lifecycle.md` | Task 3 | Async App/main lifecycle and shutdown integration | Focused App/lifecycle tests; full workspace gate | `Integrate Sprint 15 Runtime application lifecycle` |
| 5 | `05-complete-runtime-service-container-evidence.md` | Task 4 | Public integration evidence and current-state docs | Non-zero Runtime integration matrix; full workspace gate | `Complete Sprint 15 Runtime service container evidence` |
| 6 | `06-sprint-15-integration-review.md` | Task 5 and successful implementation validation | Review, transition, and conditional Sprint 14 suite retirement | Complete focused review matrix and full workspace gate | `Complete Sprint 15 Runtime service container review` |

## Already-complete, failure, and review gates

- `already_complete` requires current committed evidence and successful required
  validation for every criterion; never create an empty commit.
- Stop at the first prerequisite, implementation, validation, staging, commit,
  or review failure. Do not skip, reorder, combine, or partially commit tasks.
- Run Task 6 only after Tasks 1-5 are committed or proven already complete.
- Only `pass` or `pass with non-blocking follow-ups` plus successful validation
  may complete Sprint 15, make Sprint 16 eligible, and authorize the final
  review commit.
- Prompt retirement is Task 6's final bounded action and must be atomic with
  the review artifact and Roadmap transition.

## Final report additions

Report ordered task outcomes, timestamps, elapsed durations, token telemetry
when available, exact commits and subjects, validation results, starting and
ending `HEAD`, initial and final status, changed and preserved paths, `.codex/`
state, review decision, current suite, every retired path, Sprint 16 eligibility,
and remaining staged or uncommitted work.
