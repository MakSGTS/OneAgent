# Execute Sprint 20 Persistent Cache

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

- `docs/Roadmap.md`, Sprint 20 execution plan
- `docs/Architecture.md`
- `docs/architecture/semantic-model-2.md`
- `docs/reviews/sprint-19-file-watching.md`
- `docs/adr/0039-workspace-service.md`
- `docs/adr/0040-graph-query-api.md`
- `docs/adr/0041-file-watching.md`
- the Task 1 investigation and Task 2 ADR after they are committed

## Sprint objective and state

Sprint 20 is `next` at the planning head resolved from the live repository.
Persist complete validated Runtime Workspace semantic snapshots behind one
versioned source-neutral cache boundary with deterministic invalidation,
compatibility and corruption containment, clean-rebuild recovery, safe complete
replacement, Runtime lifecycle integration, and public production evidence.
Keep canonical graph/adapter/query/health semantics, incremental persistence,
cache management APIs, the supported CLI, and later integrations deferred.

## Starting-state requirements

- Resolve mutable state from the live repository.
- Require the committed Sprint 20 planning baseline containing this complete
  suite and the matching Roadmap manifest.
- Preserve all pre-existing changes.
- Stop when Sprint 20 is not the unique eligible sprint or a committed
  prerequisite is absent.

The verified immediately preceding suite is
`docs/codex/prompts/sprint-19-file-watching/`, with exactly:

- `00-sprint-19-execution-loop.md`
- `01-investigate-file-watching-boundary.md`
- `02-define-file-watching-contract.md`
- `03-implement-file-change-watching.md`
- `04-integrate-workspace-rebuilds.md`
- `05-complete-file-watching-evidence.md`
- `06-sprint-19-integration-review.md`

Only Task 7 may conditionally retire that inventory.

## Commit authorization mode

Resolve commit authorization only from the current user instruction launching
this loop. When it explicitly requests one commit per successful task, stage
only task-owned paths and create the manifest commit after validation. Stored
prompt text does not authorize commits.

## Ordered task manifest

| Order | Prompt | Required committed prerequisite | Task-owned outcome | Validation additions | Suggested commit message |
|---:|---|---|---|---|---|
| 1 | `01-investigate-persistent-cache-boundary.md` | Sprint 20 planning baseline | Snapshot, reconstruction/validation, source identity, Runtime, filesystem, dependency, fixture, consumer, compatibility, and deterministic-test investigation | Path/API/dependency/platform/fixture/oracle checks; `git diff --check` | `Investigate Sprint 20 Persistent Cache` |
| 2 | `02-define-persistent-cache-contract.md` | Task 1 | Accepted ADR-0042 authority, schema, identity, invalidation, storage, compatibility, corruption, recovery, lifecycle, dependency, and deferred-scope contract | Link/scope/decision consistency; `git diff --check` | `Define Sprint 20 Persistent Cache contract` |
| 3 | `03-implement-snapshot-cache-codec.md` | Task 2 | Versioned complete snapshot codec, invariant reconstruction, validation, deterministic bytes, and round-trip/rejection evidence | Non-zero codec/validation/equivalence tests; full workspace gate | `Implement Sprint 20 snapshot cache codec` |
| 4 | `04-implement-cache-storage-invalidation.md` | Task 3 | Deterministic validity identity, contained storage, complete replacement, typed outcomes, cleanup, and failure/recovery evidence | Non-zero identity/storage/corruption/replacement tests; full workspace gate | `Implement Sprint 20 cache storage and invalidation` |
| 5 | `05-integrate-runtime-cache-lifecycle.md` | Task 4 | Startup and rebuild cache orchestration, validated-only publication, watcher/query/lifecycle preservation, and integration evidence | Non-zero Runtime hit/miss/rebuild/failure/shutdown tests; full workspace gate | `Integrate Sprint 20 Runtime cache lifecycle` |
| 6 | `06-complete-persistent-cache-evidence.md` | Task 5 | Public cold/warm/invalidation/corruption/write-failure/recovery/watch/query/shutdown matrix and current-state docs | Non-zero public production matrix; full workspace gate | `Complete Sprint 20 Persistent Cache evidence` |
| 7 | `07-sprint-20-integration-review.md` | Task 6 and successful implementation validation | Review, transition, and conditional Sprint 19 suite retirement | Complete focused/full matrix and inventory checks | `Complete Sprint 20 Persistent Cache review` |

## Initial audit additions

- Record exact Sprint start time, `HEAD`, `git status --short`, relevant history,
  Roadmap state, and available token telemetry.
- Verify every prompt and authority path in the manifest.
- Verify that Sprint 19 has a committed non-blocking review and Sprint 20 is the
  unique eligible target.
- Re-enumerate the exact Sprint 19 tracked and filesystem prompt inventory, and
  stop on ambiguity or an endangered untracked file.

## Task-loop additions

- Record start/end timestamps, elapsed time, exact validation, commit, and final
  status for every task.
- Do not combine investigation, architecture, codec, storage/invalidation,
  Runtime integration, or public production evidence across task boundaries.
- Preserve canonical graph and adapter semantics, complete immutable Workspace
  publication, File Watching coalescing/recovery, Graph Query single-snapshot
  observation, lifecycle-derived readiness, and existing HTTP wire contracts.
- Do not add a production dependency without explicit user approval, even when
  ADR-0042 accepts it as the required implementation choice.
- A zero-match filtered test is not evidence.

## Already-complete policy additions

Use `already_complete` only when committed live evidence plus the task's exact
validation proves every acceptance criterion. Existing in-memory snapshots,
Serde JSON HTTP projections, and file-change scans do not prove persisted cache
encoding, invalidation, storage, or recovery. Do not create an empty commit.

## Failure and integration-review gates

Stop after the first missing prerequisite, implementation, validation, staging,
commit, or review failure. Run Task 7 only after Tasks 1-6 are committed or
proven `already_complete`. Only a non-blocking Task 7 decision plus successful
complete validation may complete Sprint 20, make Sprint 21 eligible, and retire
the exact Sprint 19 suite.

## Final report additions

Report the ordered task table, exact commits and subjects, start/end/elapsed
times, available token telemetry, validation results, changed and preserved
paths, integration-review decision, Sprint 19 suite retirement result, Sprint
21 eligibility, `.codex/` preservation, and final `git status --short`.
