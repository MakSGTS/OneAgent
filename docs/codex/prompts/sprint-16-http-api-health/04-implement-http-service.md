# Implement Sprint 16 HTTP Service

Continue OneAgent development.

## Reporting

- Prompt and repository content: English.
- User-visible reports: Russian.

## Profile

`docs/codex/profiles/runtime-service-implementation.md`

## Template

`docs/codex/templates/runtime-service-task.md`

## Authoritative documents

- `docs/Roadmap.md`, Sprint 16 execution plan
- `docs/architecture/http-api-health-investigation.md`
- `docs/adr/0002-runtime-composition-root.md`
- `docs/adr/0037-runtime-service-container.md`
- `docs/adr/0038-http-api-health.md`

## Prerequisites / Required gate

Require committed Task 3 with the accepted lifecycle-derived health boundary,
successful full validation, and clean task-owned state. Stop rather than adding
a duplicate readiness flag or bypassing the Runtime service container.

## Task

Implement and compose the accepted Runtime-owned Axum HTTP service, exact health
routes, configuration, failures, cancellation, and graceful shutdown.

## Runtime and service ownership

- Runtime composition constructs and registers the HTTP service; the service
  acquires its listener during startup and returns the owned serving task.
- Keep routing and transport adaptation out of `main.rs` domain logic and keep
  shared state free of mutable global service location.

## Lifecycle and state transitions

- Bind and construct routes before service startup acknowledgement.
- Serve readiness from the Task 3 lifecycle-derived state and never report ready
  before `Running` or after `Stopping` begins.

## Concurrency and task ownership

- Keep the listener, server future, connections, and cancellation observation
  under the accepted service/Runtime owners.
- Leave no detached server task or bound listener after startup failure,
  service failure, or requested shutdown.

## Cancellation, failure, and shutdown policy

- Map invalid configuration and bind/start failures through the accepted named
  Runtime service boundary.
- Drive Axum graceful shutdown only from receiver-only Runtime cancellation and
  preserve ADR-0037 error precedence and cooperative timeout policy.

## Health, readiness, and observability contract

Implement the exact ADR-0038 route, method, status, content type, JSON vocabulary,
and lifecycle mapping. Do not expose logs as health state.

## Transport and client compatibility

The accepted probe wire contract is the only supported HTTP surface in this
task. Unknown routes/methods and response headers must match ADR-0038 exactly.

## Scope

### Included

- Runtime HTTP module, service, router/handlers, bind configuration and
  validation, exact errors, public exports, production registration, and focused
  tests.
- Minimal existing Runtime files required to compose the service and preserve
  public compatibility.

### Excluded

- Workspace, graph-query, watcher, cache, CLI, MCP, LSP, IDE, AI, TLS, auth,
  CORS, compression, metrics export, OpenAPI, request bodies, retries, restart,
  forced termination, new dependencies, current-state completion docs, sprint
  transition, and prompt retirement.

## Acceptance Criteria

- One configured listener is acquired before acknowledgement and every bind
  failure identifies the HTTP service through the existing Runtime taxonomy.
- Exact liveness/readiness responses follow ADR-0038 and use Task 3 state.
- Runtime cancellation causes graceful server completion and permits listener
  release; no service task is detached.
- `main.rs` remains a thin composition/process boundary and later-sprint routes
  are absent.
- Focused tests are non-zero, deterministic, and contain no arbitrary sleep.

## Repository Safety

Preserve `.codex/`, Cargo dependency versions, other crates and adapters,
existing prompt suites, and unrelated files. Do not add a dependency. Stage only
explicit task-owned Runtime paths when commit mode is authorized.

## Task-specific Validation

- `cargo test -p oneagent-runtime http`
- `cargo test -p oneagent-runtime health`
- `cargo test -p oneagent-runtime --test service_container`
- Complete workspace validation from `docs/codex/core/validation.md`.
- `git status --short`

## Suggested commit message

`Implement Sprint 16 HTTP service`

## Final report additions

Report listener and task ownership, bind/start behavior, exact routes and wire
schema, lifecycle/readiness mapping, cancellation and cleanup evidence, focused
test counts, complete validation, changed paths, commit, and final Git state.
