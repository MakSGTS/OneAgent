# Define Sprint 16 HTTP API and Health Contract

Continue OneAgent development.

## Reporting

- Prompt and repository content: English.
- User-visible reports: Russian.

## Profile

`docs/codex/profiles/architecture.md`

## Template

`docs/codex/templates/architecture-task.md`

## Authoritative documents

- `docs/Roadmap.md`, Sprint 16 execution plan
- `docs/architecture/semantic-model-2.md`
- `docs/architecture/http-api-health-investigation.md`
- `docs/reviews/sprint-15-runtime-service-container.md`
- `docs/adr/0002-runtime-composition-root.md`
- `docs/adr/0037-runtime-service-container.md`

## Prerequisites / Required gate

Require committed Task 1 evidence sufficient to decide every included bind,
route, method, schema, lifecycle, health, ownership, failure, shutdown,
compatibility, and test contract. Stop without edits when any required decision
lacks executable repository evidence.

## Task

Create `docs/adr/0038-http-api-health.md` and synchronize only the planning-level
Semantic Model and Architecture text required to identify the accepted HTTP
adapter and health/readiness boundary.

## Scope

### Included

- Public bind-address configuration and validation, listener acquisition phase,
  service identity, startup acknowledgement, listener/connection/task ownership,
  cancellation, graceful shutdown, and failure propagation.
- Exact route and method matrix, success and non-success status codes, media
  type, JSON schema and vocabulary, liveness and readiness definitions, and
  unknown route/method behavior.
- Canonical lifecycle-to-health projection with readiness prohibited before
  `Running` and from `Stopping` onward; no second mutable health authority.
- Stable first-slice compatibility boundary, observability, public loopback
  oracle, repeated-run/resource-release evidence, implementation prerequisites,
  rejected alternatives, deferred scope, and documentation completion criteria.

### Excluded

- Rust/Cargo implementation, endpoint tests, current support claims, workspace
  or graph APIs, CLI behavior, TLS, authentication, authorization, OpenAPI,
  metrics, forced termination policy, sprint completion, and prompt retirement.

## Acceptance Criteria

- ADR-0038 is `Accepted` and decides every included wire, health, lifecycle,
  ownership, startup, failure, cancellation, shutdown, and compatibility row.
- The first slice is implementable using only confirmed locked dependencies and
  cross-platform repository-owned test oracles.
- The contract preserves ADR-0002 composition ownership and ADR-0037 structured
  task/cancellation behavior without changing semantic authority.
- Deferred Sprint 17-21 and later transport/security behavior remains explicit.
- Architecture documents describe an accepted plan, not implemented support.

## Repository Safety

Preserve `.codex/`, Rust/Cargo files, existing prompts, unrelated documentation,
and user changes. Stage only the ADR and exact planning-level architecture files
when commit mode is authorized.

## Task-specific Validation

- Validate every cited source and internal Markdown link.
- Compare the decision matrix with Task 1 evidence and ADR-0037 lifecycle/error
  constraints.
- Verify that no current support claim or Sprint 16 completion transition was
  introduced.
- `git diff --check`
- `git status --short`

## Suggested commit message

`Define Sprint 16 HTTP API and health contract`

## Final report additions

Report the accepted bind, route, schema, liveness, readiness, lifecycle,
ownership, failure, shutdown, compatibility, and test contracts; rejected
alternatives; deferred scope; changed paths; commit; and final Git state.
