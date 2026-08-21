# Integrate Sprint 20 Runtime Cache Lifecycle

Continue OneAgent development.

## Reporting

- Prompt and repository content: English.
- User-visible reports: Russian.

## Profile

`docs/codex/profiles/persistent-state-implementation.md`

## Additional workflow

`docs/codex/workflows/runtime-service.md`

## Template

`docs/codex/templates/persistent-state-task.md`

## Authoritative documents

- `docs/Roadmap.md`, Sprint 20 execution plan
- `docs/architecture/persistent-cache-investigation.md`
- `docs/Architecture.md`
- `docs/architecture/semantic-model-2.md`
- `docs/adr/0037-runtime-service-container.md`
- `docs/adr/0038-http-api-health.md`
- `docs/adr/0039-workspace-service.md`
- `docs/adr/0040-graph-query-api.md`
- `docs/adr/0041-file-watching.md`
- `docs/adr/0042-persistent-cache.md`

## Prerequisites / Required gate

Require committed Task 4 with the accepted codec, identity, storage, replacement,
and typed outcomes, successful focused and complete validation, and clean task-
owned state. Stop rather than changing accepted load/build/write, publication,
failure/recovery, watcher, or lifecycle semantics during integration.

## Task

Integrate the committed Persistent Cache into Runtime Workspace startup and
File Watching rebuild orchestration with exact scan/load/build/write race
closure, validated-only immutable publication, last-valid failure behavior,
stable Graph Query observation, lifecycle preservation, and focused
deterministic integration tests.

## Runtime and service ownership

- Keep `WorkspaceService` as the sole owner of source observation, cache
  orchestration, complete semantic builds, writes, publication, cancellation,
  and cleanup; adapters and cache bytes do not become semantic authorities.
- Keep blocking cache/build work off async workers and own/join every operation
  exactly as ADR-0042 accepts.

## Lifecycle and state transitions

- On startup, publish only an exact validated cache hit or a complete validated
  clean build; apply the accepted cold/warm, rejected-entry, write-failure, and
  startup-failure behavior before Runtime becomes `Running`.
- After relevant changes, preserve Task 19 serialization/coalescing, publish only
  complete valid replacements, and persist successful accepted snapshots at the
  exact point defined by ADR-0042.
- Keep cache state out of lifecycle-derived readiness and preserve update-status
  compatibility except for additive accepted observation, if any.

## Cancellation, failure, shutdown, and consumers

- Apply exact load/write/build failure classification, last-valid retention,
  recovery, cancellation, shutdown, cleanup, and receiver-closure behavior.
- Preserve one immutable snapshot per Graph Query request and all existing
  health and Graph Query routes, schemas, errors, limits, and ordering.

## Scope

### Included

- Runtime configuration/composition/service integration required by ADR-0042,
  startup hit/miss/build/write, rejected-cache clean rebuild, post-watch cache
  replacement, recoverable load/write/rebuild behavior, publication, status,
  cancellation, shutdown, cleanup, and focused tests.
- Deterministic seams proving a warm hit avoids adapter rebuilding, races do not
  publish stale state, writes follow only successful builds, changes during work
  receive accepted follow-up handling, failures retain or clear exactly as
  accepted, queries see old-or-new complete state, and fresh runs are independent.

### Excluded

Codec/storage redesign, new dependencies, graph/parser/adapter semantics,
incremental persistence, cache HTTP/CLI APIs, fixture/current-state docs,
cross-process locking, remote cache, eviction, native watchers, supported CLI,
benchmarks, performance/security claims, and prompt/Roadmap changes.

## Acceptance Criteria

- Startup performs the exact ADR-0042 validity/load/build/write sequence and a
  warm exact hit avoids semantic adapter rebuilding while returning a fully
  validated clean-build-equivalent snapshot.
- Rejected, stale, incompatible, corrupt, unreadable, or missing cache state can
  recover through the accepted clean-build path; no rejected or partial state is
  published.
- Successful initial and File Watching replacement builds update cache exactly
  as accepted; failed load/write/build work preserves publication and recovery
  semantics without concurrent builds, feedback loops, or lost follow-ups.
- Lifecycle/health, Graph Query single-snapshot and wire behavior, update
  coalescing/status compatibility, cancellation, shutdown, observer closure,
  and fresh repetition have deterministic non-zero focused tests.
- No cache management transport, supported CLI, incremental persistence, or
  deferred capability is introduced.

## Repository Safety

Modify only Runtime configuration/composition/Workspace integration and focused
test paths required by ADR-0042. Preserve `.codex/`, prompt suites, Roadmap,
ADRs, manifests/lockfile unless already authorized by Task 3, tracked fixtures,
current-state docs, graph/adapter semantics, HTTP schemas, and unrelated files.
Stage only enumerated task-owned paths.

## Task-specific Validation

- List and run exact non-zero focused startup hit/miss/build/write,
  invalidation/corruption recovery, rebuild replacement, race, query, health,
  cancellation, cleanup, and repetition tests.
- Run affected Runtime, File Watching, Workspace, Graph Query, and health tests.
- Run the canonical complete workspace validation from
  `docs/codex/core/validation.md`.
- `git status --short`

## Suggested commit message

`Integrate Sprint 20 Runtime cache lifecycle`

## Final report additions

Report Runtime/cache ownership, startup and rebuild sequences, publication and
reader consistency, failure/recovery, watcher/query/health compatibility,
cancellation/cleanup, focused/full validation, changed paths, commit, and final
Git state.
