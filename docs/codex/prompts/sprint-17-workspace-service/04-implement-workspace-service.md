# Implement Sprint 17 Workspace Service

Continue OneAgent development.

## Reporting

- Prompt and repository content: English.
- User-visible reports: Russian.

## Profile

`docs/codex/profiles/runtime-service-implementation.md`

## Template

`docs/codex/templates/runtime-service-task.md`

## Authoritative documents

- `docs/Roadmap.md`, Sprint 17 execution plan
- `docs/architecture/workspace-service-investigation.md`
- `docs/architecture/semantic-model-2.md`
- `docs/adr/0002-runtime-composition-root.md`
- `docs/adr/0004-filesystem-workspace-discovery.md`
- `docs/adr/0036-designer-xml-adapter.md`
- `docs/adr/0037-runtime-service-container.md`
- `docs/adr/0038-http-api-health.md`
- `docs/adr/0039-workspace-service.md`

## Prerequisites / Required gate

Require committed Task 3 with the accepted immutable snapshot/build boundary,
successful full validation, and clean task-owned state. Stop rather than adding
a second snapshot, mutable readiness label, or hidden adapter construction.

## Task

Implement and compose the Runtime-owned Workspace service, configured discovery,
initial semantic build, accepted snapshot publication, lifecycle/readiness,
failure, cancellation, and shutdown behavior.

## Runtime and service ownership

- Construct dependencies at the Runtime composition root and register one
  uniquely named Workspace service through ADR-0037.
- Keep filesystem discovery and source builders in adapters, immutable shared
  dependencies in the accepted state owner, and semantic facts in their
  canonical graph objects.

## Lifecycle and state transitions

- Follow ADR-0039 for configuration validation, discovery/build start,
  publication, startup acknowledgement, `Running`, cancellation, and stopped
  state.
- Integrate readiness only through the accepted owned lifecycle/snapshot
  evidence; preserve ADR-0038 liveness and exact HTTP wire vocabulary.

## Concurrency and task ownership

- Keep blocking filesystem/parsing work off asynchronous executor threads by
  the exact accepted mechanism.
- Runtime or an accepted structured owner must retain every task handle,
  channel, snapshot sender, and cancellation receiver through completion.

## Cancellation, failure, and shutdown policy

- Preserve ADR-0037 startup rollback, named service failure, error precedence,
  reverse shutdown, and complete join behavior.
- Implement only ADR-0039 cancellation points and atomic publication. Add no
  retry, watcher, restart, forced abort, or new timeout.

## Health, readiness, and observability contract

Make readiness truthful for the accepted Workspace first slice without an
independently mutable status. Expose only bounded immutable observations needed
by public tests and Sprint 18.

## Transport and client compatibility

Do not add Workspace or graph HTTP routes. Existing `/health/live` and
`/health/ready` responses must remain wire-compatible except for lifecycle
timing explicitly governed by ADR-0039.

## Scope

### Included

- Runtime Workspace service, root configuration, dependency construction,
  service registration, initial discovery/build execution, immutable snapshot
  publication/observation, typed errors, lifecycle integration, cancellation,
  cleanup, minimal exports, and focused tests.
- Minimal existing Runtime, manifest, and accepted state files necessary for
  composition.

### Excluded

- Graph-query or Workspace HTTP endpoints, file watching/rebuild triggers,
  incremental graph updates, persistent cache, supported CLI, MCP/LSP/IDE/AI,
  adapter parser or graph semantic changes, dynamic registration, retries,
  restart, forced termination, new external dependencies, production signal
  tests, current-state completion docs, sprint transition, and prompt retirement.

## Acceptance Criteria

- The configured root is validated and discovered once per fresh Runtime run;
  supported configurations build in deterministic order through Task 3.
- Startup acknowledgement, snapshot publication, Runtime `Running`, and
  readiness follow ADR-0039 exactly; no consumer observes unsupported partial
  or stale state.
- Empty, invalid-root, discovery, unsupported-format, duplicate/collision, and
  adapter-build failures have the accepted public Runtime classification and
  cleanup behavior.
- Cancellation and shutdown terminate all owned work and channels; no detached
  build or service task survives `App::run`.
- Existing service-container and exact HTTP health tests remain green.
- Focused tests are non-zero, deterministic, use explicit coordination and
  bounded hang guards, and contain no arbitrary sleep.

## Repository Safety

Preserve `.codex/`, accepted source and graph behavior, existing HTTP wire
contract, current prompt suites, current-state docs, and unrelated files. Do not
add an external dependency. Stage only exact task-owned Runtime/manifests/tests
when commit mode is authorized.

## Task-specific Validation

- Run the exact new non-zero Workspace service unit/integration filters.
- `cargo test -p oneagent-runtime workspace`
- `cargo test -p oneagent-runtime --test service_container`
- `cargo test -p oneagent-runtime --test http_health`
- Affected workspace/filesystem/EDT/Designer XML package tests.
- Complete workspace validation from `docs/codex/core/validation.md`.
- `git status --short`

## Suggested commit message

`Implement Sprint 17 Workspace service`

## Final report additions

Report composition and resource ownership, configuration/discovery/build flow,
snapshot publication and readiness, failures, cancellation/shutdown cleanup,
focused test counts, preserved HTTP behavior, complete validation, changed
paths, commit, and final Git state.
