# Implement Sprint 15 Runtime Service Container

Continue OneAgent development.

## Reporting

- Repository content and commit message: English.
- User-visible reports: Russian.

## Profile

`docs/codex/profiles/runtime-service-implementation.md`

## Template

`docs/codex/templates/runtime-service-task.md`

## Authoritative documents

- `docs/adr/0037-runtime-service-container.md`
- `docs/adr/0002-runtime-composition-root.md`
- `docs/architecture/runtime-service-container-investigation.md`
- `docs/Architecture.md`

## Prerequisites / Required gate

Require committed Task 2 and clean task-owned state. Treat ADR-0037 as fixed.

## Task

Implement only the accepted reusable Runtime service-container primitives,
service/task ownership, cancellation, deterministic startup/shutdown, failure
propagation, and focused unit evidence. Establish the accepted public library
surface only where ADR-0037 requires it.

## Runtime and service ownership

The Runtime composition root owns registration and the container. The concrete
service, task-handle, shutdown, lifecycle, and error ownership must match
ADR-0037 and remain independent from HTTP or another adapter.

## Lifecycle and state transitions

Implement the container-level constructed/start/running/stop/terminal behavior
and ordering accepted by ADR-0037 without integrating process signals or the
top-level App run loop in this task.

## Concurrency and task ownership

Track every spawned task and resource through the accepted structured owner.
Do not detach tasks or rely on scheduler timing for correctness.

## Cancellation, failure, and shutdown policy

Implement the accepted cancellation propagation, partial-start rollback,
service failure, join failure, and shutdown result contracts exactly.

## Health, readiness, and observability contract

Expose only the internal lifecycle or deterministic probe evidence accepted for
Sprint 15. Do not define an HTTP health/readiness schema.

## Transport and client compatibility

No transport or client behavior is included. Preserve an adapter-independent
public boundary for later sprints where ADR-0037 requires it.

## Scope

### Included

- Runtime service/container/cancellation/error/library modules required by the
  accepted model.
- Duplicate, ordering, startup rollback, service completion/failure,
  cancellation, join, shutdown, cleanup, and repeated-construction unit tests.
- Minimal compatibility changes to existing lifecycle/error exports.

### Excluded

- App/main signal integration, HTTP/Axum routes, workspace or graph services,
  file watching, persistence, CLI, new dependencies, unrelated refactors,
  completion docs, and Coverage claims.

## Acceptance Criteria

- Every registered service and spawned task has deterministic identity,
  ownership, ordering, and terminal handling matching ADR-0037.
- Partial startup and runtime failure cannot leave detached tasks.
- Requested cancellation is distinct from service failure and shutdown results
  are deterministic.
- Focused tests use explicit synchronization or controlled execution, not
  arbitrary sleeps, and every named filter matches tests.
- Existing configuration/builder/lifecycle behavior remains compatible outside
  the accepted migration.

## Repository Safety

Modify only exact `apps/runtime` paths named by the task Change Contract.
Preserve prompts, `.codex/`, unrelated crates, and deferred adapters.

## Task-specific Validation

- Focused non-zero `oneagent-runtime` service/container tests.
- `cargo test -p oneagent-runtime`
- Complete workspace validation from `docs/codex/core/validation.md`.

## Suggested commit message

`Implement Sprint 15 Runtime service container`

## Final report additions

Report public/container ownership, task/resource inventory, startup/rollback,
cancellation/failure/shutdown behavior, deterministic tests, compatibility,
exact validation, commit, and Git state.
