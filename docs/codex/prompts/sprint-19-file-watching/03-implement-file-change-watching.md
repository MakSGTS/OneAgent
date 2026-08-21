# Implement Sprint 19 File Change Watching

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

Require committed Task 2 with accepted ADR-0041, successful documentation
validation, and clean task-owned state. If ADR-0041 requires a new production
dependency, require explicit current user approval before changing a manifest
or lockfile. Stop rather than substitute another watcher, change vocabulary,
coalescing policy, ownership model, or error behavior in code.

## Task

Implement the accepted Runtime-owned file-change observation boundary with
normalized relevant change signals, bounded coordination, cancellation,
terminal cleanup, typed failures, and focused deterministic tests. Do not
trigger Workspace rebuilds in this task.

## Runtime and service ownership

- Keep filesystem observation source-specific and transport-neutral; it does
  not own semantic facts, snapshots, graph queries, or HTTP behavior.
- Give every watcher task, callback, channel, timer, blocking operation, and
  native or portable resource the exact owner accepted by ADR-0041.
- Expose only the accepted normalized signal/error boundary needed by Task 4;
  do not expose raw platform event types as OneAgent public semantics.

## Lifecycle and state transitions

- Establish observation at the accepted point relative to the initial build
  and Runtime `Running` transition.
- Apply exact relevance, normalization, coalescing, overflow, root-loss, and
  repeated-change behavior from ADR-0041.
- Preserve lifecycle-derived readiness; watcher state is not another mutable
  health authority.

## Concurrency and task ownership

- Use bounded coordination and deterministic event acknowledgements.
- Prevent detached work, unowned callbacks, unbounded queues, concurrent
  semantic builds, and platform-specific ordering assumptions.
- Make burst and in-flight signal behavior observable without arbitrary sleeps.

## Cancellation, failure, and shutdown policy

- Propagate startup/runtime observation failures through the accepted typed
  Runtime boundary.
- Stop accepting change signals on cancellation, release all watcher resources,
  close observers, and join every owned task before service completion.
- Preserve the primary Runtime failure and existing reverse cleanup rules.

## Health, readiness, and observability contract

- Preserve exact Sprint 16 liveness/readiness behavior.
- Expose only the accepted in-process observations or typed errors; do not add
  routes, mutable status labels, logging contracts, metrics, or tracing export.

## Scope

### Included

- Accepted watcher dependency and manifest/lockfile changes only when explicitly
  approved and required by ADR-0041.
- File-change observation module/API, relevance and normalization, bounded
  coalescing, service/resource ownership, cancellation, typed failures, and
  focused unit/integration tests.
- Positive relevant changes, ignored changes, duplicates/bursts, accepted
  rename equivalence, root/error behavior, cancellation, cleanup, and repeated
  fresh construction.

### Excluded

Workspace rebuild execution or snapshot publication, graph-query result
changes, HTTP routes, fixture/current-state documentation changes, graph/parser/
adapter semantics, partial/incremental semantic updates, persistence, supported
CLI, Git/network workspaces, symlink expansion, arbitrary retries, restart,
forced termination, benchmarks, and unsupported performance claims.

## Acceptance Criteria

- The implementation matches every accepted ADR-0041 watcher, relevance,
  normalization, coalescing, error, ownership, lifecycle, and shutdown rule.
- Public OneAgent boundaries contain no raw platform watcher vocabulary or
  borrowed callback lifetime.
- Relevant and irrelevant changes, burst/duplicate behavior, accepted rename
  equivalence, observation failure, cancellation, cleanup, and repeated fresh
  construction have deterministic non-zero focused tests.
- Tests use event handshakes and timeouts only as hang guards; arbitrary sleeps
  are not passing evidence.
- No Workspace rebuild, semantic snapshot publication, graph/API wire change,
  or deferred capability is introduced.

## Repository Safety

Modify only live implementation, manifest/lockfile, and focused test paths
proved necessary by ADR-0041. Preserve `.codex/`, prompt suites, Roadmap, ADRs,
tracked production fixtures, current-state docs, graph and adapter semantics,
and unrelated files. Stage only enumerated task-owned paths.

## Task-specific Validation

- Run exact non-zero focused watcher normalization, relevance, burst, failure,
  cancellation, cleanup, and repetition tests.
- Run affected Runtime package tests.
- Run the canonical complete workspace validation from
  `docs/codex/core/validation.md`.
- `git status --short`

## Suggested commit message

`Implement Sprint 19 file change watching`

## Final report additions

Report dependency approval and changes, watcher/resource ownership, normalized
signals, coalescing and failure behavior, lifecycle/cancellation/cleanup,
focused/full validation, changed paths, commit, and final Git state.
