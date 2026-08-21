# Implement Sprint 16 Runtime Health State

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

Require committed Task 2 with accepted ADR-0038 and clean task-owned state.
Stop when the accepted lifecycle-to-health projection or ownership boundary is
missing, contradictory, or not implementable through the current Runtime state.

## Task

Implement the accepted transport-neutral Runtime health state and focused tests.
Do not create an HTTP listener or route in this task.

## Runtime and service ownership

- Keep mutable lifecycle authority in `Lifecycle` and immutable shared
  dependencies in `AppState` as accepted by ADR-0002 and ADR-0037.
- Expose only the minimal cloneable or snapshot boundary accepted by ADR-0038;
  do not add a global registry or independently mutable ready flag.

## Lifecycle and state transitions

- Derive every health/readiness result from the canonical lifecycle transition
  source.
- Prove every accepted state mapping, including startup, `Running`, `Stopping`,
  `Stopped`, and fresh repeated application construction.

## Concurrency and task ownership

- Add no spawned task, listener, socket, or detached watcher.
- Preserve watch/channel ownership and terminal behavior defined by ADR-0037.

## Cancellation, failure, and shutdown policy

- Do not change service cancellation, error precedence, cleanup, or shutdown
  timeout behavior.
- The health state must observe shutdown transitions without owning shutdown.

## Health, readiness, and observability contract

Implement only the transport-neutral state and stable vocabulary selected by
ADR-0038. Liveness/readiness HTTP status mapping remains Task 4.

## Transport and client compatibility

No transport or client is introduced. Public API additions must be the smallest
surface needed by the accepted HTTP adapter and public tests.

## Scope

### Included

- Runtime lifecycle observation/state projection and focused unit tests.
- Minimal public exports and Rustdoc required by ADR-0038.
- Preservation tests for existing lifecycle and service-container behavior.

### Excluded

- HTTP modules, Axum router/listener, bind configuration, route/status/JSON
  behavior, production service registration, workspace/graph behavior,
  documentation completion, sprint transition, and prompt retirement.

## Acceptance Criteria

- Health/readiness is deterministic and derived only from canonical lifecycle
  evidence.
- Ready is false before `Running` and from `Stopping` onward; every other
  accepted mapping is exact.
- Existing Runtime lifecycle, service ownership, cancellation, and error
  behavior remains unchanged.
- Focused tests are non-zero and use direct state synchronization without sleeps.
- Public API and Rustdoc expose no HTTP or later-sprint semantic authority.

## Repository Safety

Preserve `.codex/`, HTTP composition, Cargo dependencies, other crates, adapters,
existing prompt suites, and unrelated files. Stage only task-owned Runtime paths
when commit mode is authorized.

## Task-specific Validation

- `cargo test -p oneagent-runtime health`
- `cargo test -p oneagent-runtime lifecycle`
- `cargo test -p oneagent-runtime --test service_container`
- Complete workspace validation from `docs/codex/core/validation.md`.
- `git status --short`

## Suggested commit message

`Implement Sprint 16 Runtime health state`

## Final report additions

Report the state ownership and lifecycle mapping, public API changes, focused
test counts, preserved service behavior, complete validation, changed paths,
commit, and final Git state.
