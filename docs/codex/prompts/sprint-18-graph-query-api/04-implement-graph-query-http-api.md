# Implement Sprint 18 Graph Query HTTP API

Continue OneAgent development.

## Reporting

- Prompt and repository content: English.
- User-visible reports: Russian.

## Profile

`docs/codex/profiles/runtime-service-implementation.md`

## Template

`docs/codex/templates/runtime-service-task.md`

## Authoritative documents

- `docs/Roadmap.md`, Sprint 18 execution plan
- `docs/architecture/graph-query-api-investigation.md`
- `docs/architecture/semantic-model-2.md`
- `docs/adr/0002-runtime-composition-root.md`
- `docs/adr/0008-semantic-model-2-knowledge-graph.md`
- `docs/adr/0037-runtime-service-container.md`
- `docs/adr/0038-http-api-health.md`
- `docs/adr/0039-workspace-service.md`
- `docs/adr/0040-graph-query-api.md`

## Prerequisites / Required gate

Require committed Task 3 with the accepted transport-neutral query boundary,
successful full validation, and clean task-owned state. Stop rather than
serializing graph internals or selecting a different route, schema, error, or
composition contract.

## Task

Implement and compose the accepted versioned Graph Query HTTP API through the
existing Runtime-owned listener, mapping only Task 3 operations and typed
failures to the exact ADR-0040 wire contract.

## Runtime and service ownership

- Construct Workspace observation and query dependencies at the Runtime
  composition root and inject them through the accepted boundary.
- Keep the existing `HttpService` as sole listener/task owner and the Task 3
  service as the transport-neutral query authority.

## Lifecycle and state transitions

- Preserve ADR-0037 startup, running, stopping, and stopped behavior and
  ADR-0038 lifecycle-derived readiness.
- Apply only the ADR-0040 query availability rule during observable lifecycle
  and snapshot states; do not add mutable readiness or watcher semantics.

## Concurrency and task ownership

- Add no listener, detached task, blocking graph work, or global state.
- Request handlers may clone only accepted immutable state/observers and must
  return without retaining unbounded graph or connection resources.

## Cancellation, failure, and shutdown policy

- Map Task 3 typed failures exactly; never serialize source-chain or diagnostic
  `Display` prose.
- Preserve graceful HTTP cancellation, connection ownership, Workspace reverse
  clearing, and complete Runtime join behavior.

## Health, readiness, and observability contract

Keep `/health/live` and `/health/ready` exact and unchanged. Query routes must
not become a second liveness or readiness authority.

## Transport and client compatibility

Implement the exact accepted versioned path, method, parameter, status, content
type, JSON field/vocabulary, error-code, fallback, and limit matrix. Add no
implicit HEAD, redirect, content negotiation, streaming, or undocumented field.

## Scope

### Included

- Existing HTTP router composition, accepted query state injection, exact
  versioned routes and handlers, request decoding/validation, Task 3 invocation,
  response/error serialization, production composition wiring, minimal public
  exports, and focused handler plus loopback tests.

### Excluded

- New listener or service-container primitive, graph or Workspace mutation,
  new query operations, file watching/invalidation, persistence, supported CLI,
  authentication/authorization, TLS, CORS, rate limiting, streaming, OpenAPI,
  external dependencies, production fixtures beyond focused needs,
  current-state completion docs, sprint transition, and prompt retirement.

## Acceptance Criteria

- Every accepted HTTP request invokes exactly one Task 3 operation against the
  selected immutable configuration and returns the exact closed success schema
  in deterministic order.
- Missing, invalid, unsupported, unavailable, unknown, and bound-violation
  requests map to the exact stable statuses, media types, fields, and error
  codes selected by ADR-0040.
- Wrong methods, unknown paths, trailing slashes, malformed encoding, duplicate
  parameters, and other accepted negative cases have explicit deterministic
  behavior without leaking Axum or Rust implementation details.
- The existing listener remains sole transport owner; startup, cancellation,
  graceful shutdown, address observation, and exact health routes remain
  compatible.
- Focused tests are non-zero and include real loopback requests for the accepted
  success and error matrices, use explicit synchronization and bounded hang
  guards, and contain no arbitrary sleep or external service.
- Complete workspace validation succeeds.

## Repository Safety

Preserve `.codex/`, canonical graph/query semantics, Workspace build behavior,
exact health wire behavior, production fixtures, current prompt suites,
current-state docs, and unrelated files. Do not add an external dependency.
Stage only exact task-owned Runtime source, manifest, export, composition, and
focused-test paths when commit mode is authorized.

## Task-specific Validation

- Run the exact new non-zero Graph Query HTTP unit and loopback filters.
- `cargo test -p oneagent-runtime http::tests`
- `cargo test -p oneagent-runtime --test http_health`
- `cargo test -p oneagent-runtime --test workspace_service`
- `cargo test -p oneagent-graph --test query`
- Complete workspace validation from `docs/codex/core/validation.md`.
- `git status --short`

## Suggested commit message

`Implement Sprint 18 Graph Query HTTP API`

## Final report additions

Report route/schema/error compatibility, composition and resource ownership,
snapshot/lifecycle behavior, preserved health contract, focused public request
counts, complete validation, changed paths, commit, and final Git state.
