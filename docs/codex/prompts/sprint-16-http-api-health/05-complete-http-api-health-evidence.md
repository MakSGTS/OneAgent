# Complete Sprint 16 HTTP API and Health Evidence

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
- `docs/architecture/semantic-model-2.md`
- `docs/adr/0002-runtime-composition-root.md`
- `docs/adr/0037-runtime-service-container.md`
- `docs/adr/0038-http-api-health.md`

## Prerequisites / Required gate

Require committed Task 4 with the production HTTP service, successful full
validation, and clean task-owned state. The public listener address or accepted
test seam must permit real loopback client/server requests; handler-only tests
cannot satisfy this task.

## Task

Complete public HTTP/health production evidence and synchronize current-state
documentation without completing Sprint 16.

## Runtime and service ownership

Exercise only the public Runtime composition and HTTP service boundary. Prove
that the Runtime retains listener/task ownership until cooperative shutdown and
that every fresh application owns independent resources.

## Lifecycle and state transitions

Prove the accepted readiness response before serving work when observable,
during `Running`, and after shutdown begins when observable, using explicit
lifecycle and connection coordination rather than elapsed-time races.

## Concurrency and task ownership

Use loopback TCP and bounded channels or lifecycle watches. Prove listener
release/rebind and absence of a surviving HTTP task after `App::run` returns.

## Cancellation, failure, and shutdown policy

Cover named bind failure, requested cancellation, graceful server completion,
connection/resource release, and repeated fresh runs under ADR-0037/0038.

## Health, readiness, and observability contract

Assert the exact status, content type, full JSON body/schema, and stable state
vocabulary for every accepted liveness/readiness lifecycle case.

## Transport and client compatibility

Send actual HTTP requests through a public loopback listener. Cover exact
success routes plus every accepted wrong-method and unknown-route result. A
handler-only or internal-router call is supplementary, not completion evidence.

## Scope

### Included

- A public integration target with real loopback HTTP I/O and non-zero tests.
- Positive, negative, lifecycle, bind-failure, shutdown, cleanup/rebind, and
  repeated-run evidence required by ADR-0038.
- Current-state synchronization in `README.md`, `docs/Architecture.md`, and
  `docs/architecture/semantic-model-2.md` when live implementation proves it.

### Excluded

- New production capabilities or dependencies, implementation refactoring,
  later-sprint routes, TLS/auth/OpenAPI/metrics, release claims, Sprint 16
  completion, Roadmap status transition, and prompt retirement.

## Acceptance Criteria

- A non-zero public integration target proves the complete accepted wire matrix
  through actual loopback client/server entry points on cross-platform Tokio.
- Tests prove lifecycle-derived readiness, bind failure, cooperative shutdown,
  listener release/rebind, repeated fresh runs, and no surviving owned resource.
- Tests use explicit synchronization and bounded hang guards, with no arbitrary
  sleep, real process signal, external service, or platform-specific path.
- Current-state docs describe exactly implemented HTTP behavior and retain all
  later-sprint exclusions; Sprint 16 remains incomplete pending Task 6.
- Full workspace validation succeeds.

## Repository Safety

Preserve `.codex/`, accepted production behavior, dependencies, unrelated crates
and adapters, current prompt suites, and unrelated files. Stage only the public
test and exact current-state docs when commit mode is authorized.

## Task-specific Validation

- `cargo test -p oneagent-runtime --test http_health -- --list`
- `cargo test -p oneagent-runtime --test http_health`
- `cargo test -p oneagent-runtime --test service_container`
- Complete workspace validation from `docs/codex/core/validation.md`.
- Verify every changed documentation claim against the public tests.
- `git status --short`

## Suggested commit message

`Complete Sprint 16 HTTP API and health evidence`

## Final report additions

Report the exact public test matrix and counts, wire responses, lifecycle states,
bind/shutdown/rebind/repeated-run evidence, current-state documentation changes,
complete validation, changed paths, commit, and final Git state.
