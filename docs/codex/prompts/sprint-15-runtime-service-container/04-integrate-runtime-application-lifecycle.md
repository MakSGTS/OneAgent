# Integrate Sprint 15 Runtime Application Lifecycle

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
- committed Task 3 public Runtime container API

## Prerequisites / Required gate

Require committed Task 3, successful full validation, and clean task-owned
state. Treat ADR-0037 and the committed container boundary as fixed.

## Task

Integrate the accepted service container into `AppBuilder`, `App`, lifecycle
state, and the production entry point so the Runtime remains asynchronously
running until the accepted shutdown or failure outcome and then terminates all
owned work deterministically.

## Runtime and service ownership

Keep dependency construction and service registration in the Runtime
composition root. App owns the built container and immutable shared state.

## Lifecycle and state transitions

Implement the accepted App construction, initialization, running, stopping,
and terminal transitions, including invalid transition and terminal-error
behavior.

## Concurrency and task ownership

Delegate service task ownership to the committed container and retain no
detached work in `main.rs`, App, or adapters.

## Cancellation, failure, and shutdown policy

Connect the injectable test shutdown boundary and production process signal
responsibility exactly as ADR-0037 defines. Propagate service and signal errors
through the accepted Runtime error surface.

## Health, readiness, and observability contract

Emit or expose only accepted lifecycle evidence. HTTP health/readiness remains
Sprint 16 scope.

## Transport and client compatibility

No HTTP route or supported client is included. `main.rs` remains composition
only and contains no service/domain logic.

## Scope

### Included

- AppBuilder/App/lifecycle/state/main integration required by ADR-0037.
- Injectable deterministic shutdown tests, service-failure propagation,
  lifecycle ordering, graceful termination, and repeated fresh-App execution.
- Public exports or compatibility constructors required by the accepted
  migration.

### Excluded

- New service families, HTTP/Axum serving, public health endpoints, workspace
  orchestration, graph queries, file watching, persistence, CLI, signal tests
  that send real OS signals, arbitrary sleeps, and new dependencies.

## Acceptance Criteria

- Production main uses the accepted asynchronous Runtime boundary and remains
  long-running until shutdown or failure.
- Tests inject shutdown deterministically and prove `Running → Stopping →
  Stopped` behavior and complete service cleanup.
- Service failure reaches the App caller with no successful-shutdown claim.
- `main.rs` retains no domain/infrastructure logic beyond configuration,
  construction, signal ownership, and error propagation accepted by ADR-0002.
- Existing Runtime configuration and builder tests remain green or have an
  explicitly accepted migration assertion.

## Repository Safety

Modify only exact `apps/runtime` lifecycle/composition paths named by the
Change Contract. Preserve prompts, `.codex/`, unrelated crates, and deferred
v0.4 capabilities.

## Task-specific Validation

- Focused non-zero App/lifecycle/shutdown/failure tests.
- `cargo test -p oneagent-runtime`
- Complete workspace validation from `docs/codex/core/validation.md`.

## Suggested commit message

`Integrate Sprint 15 Runtime application lifecycle`

## Final report additions

Report App/container ownership, async run boundary, injected and production
shutdown paths, failure propagation, lifecycle evidence, compatibility, exact
validation, commit, and Git state.
