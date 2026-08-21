# Review Sprint 19 File Watching Integration

Continue OneAgent development.

## Reporting

- Prompt and repository content: English.
- User-visible reports: Russian.

## Profile

`docs/codex/profiles/review.md`

## Template

`docs/codex/templates/review-task.md`

## Authoritative documents

- `docs/Roadmap.md`, Sprint 19 execution plan
- `docs/architecture/file-watching-investigation.md`
- `docs/Architecture.md`
- `docs/architecture/semantic-model-2.md`
- `docs/reviews/sprint-18-graph-query-api.md`
- `docs/adr/0002-runtime-composition-root.md`
- `docs/adr/0037-runtime-service-container.md`
- `docs/adr/0038-http-api-health.md`
- `docs/adr/0039-workspace-service.md`
- `docs/adr/0040-graph-query-api.md`
- `docs/adr/0041-file-watching.md`

## Prerequisites / Required gate

Require committed or proven `already_complete` Tasks 1-5, successful required
focused and complete validation, a clean task-owned tree, and an exact
planning-through-Task-5 range. Stop before review output if any prerequisite or
acceptance evidence is absent.

## Review target

Review the integrated Sprint 19 File Watching baseline against the committed
plan, investigation, ADR-0041, accepted Runtime/Workspace/Graph Query
compatibility, task exclusions, public production evidence, and complete
validation. Do not fix implementation findings in this task.

## Reviewed baseline / commit or diff range

- Planning parent: resolve from the live repository.
- Reviewed range: planning commit through committed Task 5 head.
- Record every commit hash, exact subject, owned paths, and validation outcome.

## Scope

- Watcher ownership, relevant-change normalization, bounded coalescing, typed
  failures, cancellation, and resource cleanup.
- Serialized complete Workspace rebuilds, atomic valid publication, accepted
  invalid-build/recovery behavior, and stable immutable reader observation.
- Lifecycle/health and Graph Query compatibility, dependency approval, public
  EDT/Designer XML evidence, platform coverage, deterministic tests, fixture
  provenance, docs, and deferred-scope conformance.

## Excluded

Implementation fixes, new tests to repair missing evidence, architecture
reselection, new dependencies, graph/parser/adapter semantics, incremental
mutation, persistence, supported CLI, watch-control routes, later integrations,
benchmarks, and unrelated cleanup.

## Review Criteria

- Every committed task owns exactly its planned outcome and satisfies every
  acceptance criterion without pulling excluded scope forward.
- ADR-0041 is implemented exactly; every task/channel/timer/watcher/build
  resource has one owner and terminal behavior.
- Relevant changes deterministically trigger serialized complete builds;
  irrelevant changes do not; bursts/in-flight changes follow the accepted
  policy; only complete valid snapshots become visible atomically.
- Failure, recovery, lifecycle, health, Graph Query, shutdown, cleanup, and
  fresh-run behavior match accepted contracts and public evidence.
- Any new production dependency had explicit approval and is limited to the
  accepted purpose.
- Focused filters execute non-zero tests and the complete workspace validation
  succeeds at the Task 5 head.

## Acceptance evidence matrix

Record at minimum planning readiness, investigation completeness, accepted
architecture, watcher ownership, relevance/normalization, coalescing,
serialization, publication atomicity, failure/recovery, lifecycle/health,
Graph Query compatibility, public EDT/Designer evidence, platform behavior,
dependency approval, cleanup/repetition, documentation, validation, and scope
containment.

## Authorized review outputs and state transition

Only after issuing `pass` or `pass with non-blocking follow-ups` and completing
all required validation:

- create `docs/reviews/sprint-19-file-watching.md`;
- transition Sprint 19 to `completed` in `docs/Roadmap.md` and make Sprint 20
  Persistent Cache the unique `next` target;
- synchronize only the minimum current-state sprint references required by
  that transition;
- conditionally retire the exact verified Sprint 18 prompt suite below in the
  same review commit.

The verified immediately preceding suite is
`docs/codex/prompts/sprint-18-graph-query-api/`, with exactly:

- `00-sprint-18-execution-loop.md`
- `01-investigate-graph-query-api-boundary.md`
- `02-define-graph-query-api-contract.md`
- `03-implement-graph-query-service.md`
- `04-implement-graph-query-http-api.md`
- `05-complete-graph-query-api-evidence.md`
- `06-sprint-18-integration-review.md`

Immediately before deletion, compare the tracked and filesystem inventories.
Delete only those tracked files, explicitly and non-recursively, when the gate
in `docs/codex/prompts/run-next-sprint.md` passes. Stop on any mismatch,
untracked endangered file, retained current-sprint link dependency, blocked
decision, or failed validation. Keep the current Sprint 19 suite and
`run-next-sprint.md`. Include the review artifact, state transition, current-
state synchronization, and exact Sprint 18 deletions atomically in this task's
single commit.

## Task-specific Validation

- Run exact non-zero focused watcher, rebuild, Workspace, Graph Query, health,
  failure/recovery, cancellation, cleanup, repetition, and public integration
  tests named by live evidence.
- Run the canonical complete workspace validation from
  `docs/codex/core/validation.md`.
- Verify every acceptance row, commit/path boundary, dependency approval,
  exclusion, fixture provenance, documentation link, Roadmap transition, and
  previous-suite inventory.
- `git diff --check`
- `git status --short`

## Suggested commit message

`Complete Sprint 19 File Watching review`

## Final report additions

Report reviewed range and commits, findings by severity, missing evidence,
acceptance matrix, exact validation, decision, state transition, every retired
Sprint 18 path or `already_retired`, Sprint 20 eligibility, changed paths,
commit, `.codex/` preservation, and final Git state.
