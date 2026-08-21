# Define Sprint 18 Graph Query API Contract

Continue OneAgent development.

## Reporting

- Prompt and repository content: English.
- User-visible reports: Russian.

## Profile

`docs/codex/profiles/architecture.md`

## Template

`docs/codex/templates/architecture-task.md`

## Authoritative documents

- `docs/Roadmap.md`, Sprint 18 execution plan
- `docs/architecture/graph-query-api-investigation.md`
- `docs/architecture/semantic-model-2.md`
- `docs/adr/0002-runtime-composition-root.md`
- `docs/adr/0008-semantic-model-2-knowledge-graph.md`
- `docs/adr/0026-semantic-index-boundary.md`
- `docs/adr/0027-incremental-semantic-index-maintenance.md`
- `docs/adr/0037-runtime-service-container.md`
- `docs/adr/0038-http-api-health.md`
- `docs/adr/0039-workspace-service.md`

## Prerequisites / Required gate

Require committed Task 1 evidence that every first-slice decision has a
repository-owned production source and deterministic test oracle. Stop if the
investigation reports missing or conflicting evidence.

## Task

Create and accept `docs/adr/0040-graph-query-api.md`, defining the smallest
stable bounded Runtime Graph Query API contract. Synchronize only planning-level
architecture text required to make the decision unambiguous. Implement no
production behavior.

## Scope

### Included

- Semantic authority and dependency direction between Workspace observation,
  Runtime query behavior, canonical graph queries, HTTP transport, public
  contracts, and future clients.
- Snapshot availability and lifetime, exact configuration selection, stable
  node and edge identity, deterministic ordering, and future replacement
  compatibility without implementing invalidation.
- Exact bounded first-slice operation matrix using only existing proven graph
  semantics; explicit unsupported operations and request-budget policy.
- Stable owned transport-neutral request, result, and typed-error boundary that
  does not expose internal collections or borrowed graph lifetimes.
- Exact versioned HTTP route, method, parameter, status, media-type, JSON
  schema, error-code, fallback, and compatibility matrix.
- Composition and dependency wiring, lifecycle/readiness preservation,
  cancellation and shutdown behavior, observability, deterministic testing,
  migration, first production slice, rejected alternatives, implementation
  prerequisites, and deferred scope.

### Excluded

- Rust implementation, Cargo changes, fixtures, graph model/query semantic
  changes, new query algorithms, file watching/invalidation, persistence/cache,
  supported CLI implementation, MCP/LSP/IDE/AI, authentication/authorization,
  TLS, streaming, general query languages, new external dependencies,
  performance targets, Coverage transitions, sprint completion, and prompt
  retirement.

## Acceptance Criteria

- ADR-0040 answers every Task 1 decision question with one canonical contract
  grounded in repository evidence and existing accepted ADRs.
- The contract preserves `SemanticGraph` as the sole semantic authority,
  separate immutable per-configuration snapshots, exact current query behavior,
  stable identities, deterministic ordering, and ADR-0037/0038/0039 ownership
  and lifecycle rules.
- Every accepted operation, input, bound, result field, enum vocabulary, route,
  method, status, content type, JSON field, and stable error code is closed and
  explicit; unsupported or unknown input behavior is total.
- The transport-neutral boundary owns returned data and typed errors without
  leaking Rust lifetimes, internal collections, adapter state, source error
  prose, or transport concepts.
- HTTP uses the existing listener and accepted composition boundary, adds no
  mutable readiness authority, and defines behavior during unavailable,
  initializing, running, stopping, and cleared snapshot observations.
- Public evidence requirements cover real EDT and Designer XML configurations,
  multi-configuration selection, positive/negative/bounded requests, exact wire
  behavior, ordering, shutdown, cleanup, and fresh repetition.
- Compatibility impact, rejected alternatives, implementation order, current
  limitations, Coverage impact, and Sprints 19-21 deferrals are explicit.
- Current-state documents do not claim implementation and Sprint 18 remains
  `next`.

## Repository Safety

Create only `docs/adr/0040-graph-query-api.md` and modify only the minimum
planning-level architecture document if the accepted decision requires it.
Preserve `.codex/`, production code, manifests, fixtures, current prompt suites,
Roadmap state, current-state implementation claims, and unrelated files. Stage
only explicitly enumerated ADR-owned paths when commit mode is authorized.

## Task-specific Validation

- Verify decision/evidence consistency with the Task 1 investigation and all
  cited public APIs.
- Validate internal links, status, closed operation/route/schema/error matrices,
  alternatives, first slice, implementation prerequisites, accepted/deferred
  scope, dependency impact, and Coverage impact.
- `git diff --check`
- `git status --short`

## Suggested commit message

`Define Sprint 18 Graph Query API contract`

## Final report additions

Report the accepted ownership, selection, operation, result, limit, route,
schema, error, lifecycle, and compatibility contracts; rejected alternatives;
implementation prerequisites; deferred scope; changed paths; validation;
commit; and final Git state.
