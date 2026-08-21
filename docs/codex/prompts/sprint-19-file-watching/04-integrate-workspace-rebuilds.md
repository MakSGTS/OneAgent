# Integrate Sprint 19 Workspace Rebuilds

Continue OneAgent development.

## Reporting

- Prompt and repository content: English.
- User-visible reports: Russian.

## Profile

`docs/codex/profiles/runtime-service-implementation.md`

## Template

`docs/codex/templates/runtime-service-task.md`

## Authoritative documents

- `docs/Roadmap.md`, Sprint 19 execution plan
- `docs/architecture/file-watching-investigation.md`
- `docs/Architecture.md`
- `docs/architecture/semantic-model-2.md`
- `docs/adr/0037-runtime-service-container.md`
- `docs/adr/0038-http-api-health.md`
- `docs/adr/0039-workspace-service.md`
- `docs/adr/0040-graph-query-api.md`
- `docs/adr/0041-file-watching.md`

## Prerequisites / Required gate

Require committed Task 3 with the accepted observation boundary, successful
focused and complete validation, and clean task-owned state. Stop rather than
changing accepted coalescing, scheduling, publication, failure/recovery, or
lifecycle semantics during orchestration.

## Task

Connect committed file-change observation to serialized complete Workspace
rebuilds and atomic valid snapshot replacement with exact accepted failure and
recovery behavior, stable Graph Query observation, lifecycle preservation, and
focused deterministic integration tests.

## Runtime and service ownership

- Reuse `WorkspaceSnapshotBuilder` and the production detector and builders as
  the sole complete-build path; do not duplicate or reinterpret adapter facts.
- Keep one Runtime owner for observation, rebuild scheduling, blocking build
  handles, publication, cancellation, and terminal snapshot cleanup as defined
  by ADR-0041.
- Keep `WorkspaceSnapshotObserver` and `GraphQueryService` read-only consumers.

## Lifecycle and state transitions

- Preserve initial-build startup behavior and canonical Runtime readiness.
- Serialize accepted rebuild triggers, handle changes that arrive during a
  build exactly as accepted, and publish only one complete valid immutable
  replacement at an atomic boundary.
- Apply exact invalid-build, retained-or-cleared snapshot, repeated-failure,
  recovery, and shutdown behavior from ADR-0041.

## Concurrency and task ownership

- Never run concurrent complete builds for one configured Workspace.
- Own and join every blocking build and coordinator task; do not mutate a
  published graph or expose an in-progress snapshot.
- Prove old-or-new reader visibility through deterministic observation rather
  than timing assumptions.

## Cancellation, failure, and shutdown policy

- Preserve accepted Runtime service failure and reverse cancellation rules.
- On shutdown, stop new rebuild scheduling, resolve or cancel in-flight work as
  accepted, clear publication at the accepted point, close observations, and
  join all owned work.
- Do not hide build or watcher failures behind logs or mutable readiness.

## Health, readiness, and observability contract

- Preserve exact lifecycle-derived health and Sprint 16 wire behavior.
- Preserve Sprint 18 request schemas and error mapping; requests observe one
  immutable old or new snapshot and never a partial build.
- Add only accepted in-process rebuild/publication evidence, not new routes.

## Scope

### Included

- Workspace service/coordinator integration, serialized complete rebuilds,
  atomic valid publication, accepted failure/recovery, observation continuity,
  shutdown cleanup, and focused tests.
- Relevant change to new snapshot, irrelevant/no-op behavior, multiple
  in-flight signals, invalid rebuild, recovery, graph-query visibility,
  cancellation, and repeated fresh-run evidence using deterministic test seams.

### Excluded

New watcher technology, new dependencies, raw event behavior, HTTP route/schema
changes, fixture/current-state documentation changes, graph/parser/adapter
semantics, incremental graph or Semantic Index mutation, persistence/cache,
supported CLI, watch-control APIs, Git/network workspaces, retries beyond the
accepted signal loop, restart, benchmarks, and performance claims.

## Acceptance Criteria

- The existing production complete builder remains the sole source of
  replacement snapshots and no partial or invalid snapshot is published.
- Rebuilds are serialized; accepted in-flight changes are neither silently
  lost nor used to start concurrent builds.
- Readers and Graph Query operations observe one immutable old or new complete
  snapshot, with exact accepted invalid-build retention/clearing and recovery.
- Initial startup, relevant/irrelevant change, burst during build, failure,
  recovery, cancellation, shutdown, observer closure, and repeated fresh runs
  have deterministic non-zero focused tests.
- Health and Graph Query wire compatibility remain unchanged, and no deferred
  persistence, CLI, incremental mutation, or control API is introduced.

## Repository Safety

Modify only the Runtime Workspace/orchestration implementation and focused test
paths required by ADR-0041. Preserve `.codex/`, prompts, Roadmap, ADRs,
manifests/lockfile unless already owned by committed Task 3, tracked production
fixtures, current-state docs, graph/adapter semantics, HTTP schemas, and
unrelated files. Stage only enumerated task-owned paths.

## Task-specific Validation

- Run exact non-zero focused rebuild serialization, atomic publication,
  failure/recovery, query observation, cancellation, cleanup, and repetition
  tests.
- Run affected Runtime package and Graph Query API compatibility tests.
- Run the canonical complete workspace validation from
  `docs/codex/core/validation.md`.
- `git status --short`

## Suggested commit message

`Integrate Sprint 19 Workspace rebuilds`

## Final report additions

Report orchestration ownership, scheduling and serialization, publication and
reader consistency, failure/recovery, lifecycle/query compatibility,
cancellation/cleanup, focused/full validation, changed paths, commit, and final
Git state.
