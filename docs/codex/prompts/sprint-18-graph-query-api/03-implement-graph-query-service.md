# Implement Sprint 18 Graph Query Service

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
- `docs/adr/0008-semantic-model-2-knowledge-graph.md`
- `docs/adr/0026-semantic-index-boundary.md`
- `docs/adr/0037-runtime-service-container.md`
- `docs/adr/0038-http-api-health.md`
- `docs/adr/0039-workspace-service.md`
- `docs/adr/0040-graph-query-api.md`

## Prerequisites / Required gate

Require committed Task 2 with accepted ADR-0040, successful documentation
validation, and clean task-owned state. Stop rather than selecting different
operations, bounds, result ownership, errors, or snapshot semantics in code.

## Task

Implement the accepted transport-neutral Runtime Graph Query service over one
selected immutable Workspace configuration, with owned deterministic results,
typed failures, exact request bounds, and focused tests. Do not add HTTP routes
in this task.

## Runtime and service ownership

- Keep `SemanticGraph` and its current query facade as the sole semantic
  authority; the Runtime boundary may only select, delegate, and project.
- Consume Workspace observation through the exact ADR-0040 construction and
  ownership seam without copying or mutating canonical graph state.

## Lifecycle and state transitions

- Implement only the accepted snapshot availability and query observation
  behavior for existing lifecycle states.
- Do not alter Runtime lifecycle or health readiness and do not implement
  watcher replacement or stale-state policy.

## Concurrency and task ownership

- Introduce no detached task, hidden executor, listener, or global mutable
  registry.
- Keep every observer clone and returned owned result under the exact bounded
  lifetime selected by ADR-0040.

## Cancellation, failure, and shutdown policy

- Return only the accepted typed unavailable, missing, invalid, unsupported, or
  bounded-input outcomes.
- Preserve Workspace shutdown clearing and add no retries, blocking work,
  timeout, abort, mutation, or failure-text serialization.

## Health, readiness, and observability contract

Expose only the accepted transport-neutral query observations. Existing health
and readiness behavior remains unchanged.

## Transport and client compatibility

Implement no HTTP, CLI, MCP, LSP, or IDE mapping. Public Runtime types must
remain transport-neutral and match ADR-0040 compatibility and ownership rules.

## Scope

### Included

- Accepted Runtime query request/result/error types, selected-configuration and
  selected-node lookup, bounded graph-operation delegation, deterministic owned
  projection, minimal public exports, and focused positive/negative tests.
- Only existing Runtime or local crate areas and manifests explicitly required
  by ADR-0040; no new external production dependency.

### Excluded

- HTTP routes or schemas, new listener/service task, graph mutation or semantic
  inference, query algorithm expansion, aggregate graphs, cross-configuration
  traversal, watcher/invalidation, persistence, supported CLI, current-state
  completion docs, sprint transition, and prompt retirement.

## Acceptance Criteria

- The boundary selects exactly one immutable configuration through stable
  identity and delegates each accepted operation to existing canonical graph
  query behavior.
- Every accepted result is owned, deterministic, bounded, and contains exactly
  the ADR-0040 fields and vocabularies without borrowed graph references,
  transport values, internal collections, or source-specific state.
- Snapshot unavailable, empty Workspace, unknown configuration, unknown node,
  invalid value, unsupported operation, and bound violation behavior is total
  and matches the accepted typed error taxonomy where applicable.
- No query mutates graph or Workspace state, changes semantic identity, changes
  graph query behavior, or creates a second query/index authority.
- Focused tests are non-zero and cover positive, missing, invalid, boundary,
  ordering, duplicate-name or multi-relation cases where applicable, and equal
  repeated requests without arbitrary sleeps or external services.
- Existing graph, Workspace, service-container, and HTTP health behavior remains
  green; complete workspace validation succeeds.

## Repository Safety

Preserve `.codex/`, graph semantics and public query compatibility, source
adapters, HTTP routes, production fixtures, current prompt suites, current-state
docs, and unrelated files. Do not add an external dependency. Stage only exact
task-owned source, manifest, export, and focused-test paths when commit mode is
authorized.

## Task-specific Validation

- Run the exact new non-zero Graph Query service unit or integration filters.
- `cargo test -p oneagent-graph --test query`
- `cargo test -p oneagent-runtime workspace::tests`
- `cargo test -p oneagent-runtime --test workspace_service`
- Complete workspace validation from `docs/codex/core/validation.md`.
- `git status --short`

## Suggested commit message

`Implement Sprint 18 Graph Query service`

## Final report additions

Report semantic and snapshot authority, request/result/error types, accepted
operations and limits, deterministic focused test counts, preserved graph and
Workspace behavior, complete validation, changed paths, commit, and final Git
state.
