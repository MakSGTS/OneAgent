# Complete Sprint 18 Graph Query API Evidence

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
- `docs/Architecture.md`
- `README.md`
- `docs/adr/0008-semantic-model-2-knowledge-graph.md`
- `docs/adr/0026-semantic-index-boundary.md`
- `docs/adr/0037-runtime-service-container.md`
- `docs/adr/0038-http-api-health.md`
- `docs/adr/0039-workspace-service.md`
- `docs/adr/0040-graph-query-api.md`

## Prerequisites / Required gate

Require committed Task 4 with the production Graph Query HTTP API, successful
full validation, clean task-owned state, and public seams capable of proving
real Workspace builds and loopback query requests. Handler-only or fake-only
tests cannot satisfy this task.

## Task

Complete public production evidence for the accepted Graph Query API over real
EDT and Designer XML Workspace snapshots, then synchronize current-state
documentation without completing Sprint 18.

## Runtime and service ownership

Exercise only public Runtime composition, production discovery/builders,
immutable Workspace observation, Task 3 query behavior, and the Task 4 listener.
Prove independent ownership for every fresh application.

## Lifecycle and state transitions

Prove accepted query behavior during publication, `Running`, `Stopping`,
snapshot clearing, and terminal shutdown using lifecycle and channel
coordination rather than elapsed-time races.

## Concurrency and task ownership

Use bounded public test coordination and the provenance-backed Sprint 17
production fixture. Prove no request, connection, observer sender, listener,
Workspace task, or Runtime service task survives `App::run`.

## Cancellation, failure, and shutdown policy

Cover every accepted unavailable, selection, validation, bound, and unknown
query failure observable through the public wire boundary. Preserve Workspace
startup atomicity and complete reverse shutdown.

## Health, readiness, and observability contract

Assert exact query results and errors together with unchanged health response
vocabulary and lifecycle authority where applicable.

## Transport and client compatibility

Assert the exact ADR-0040 versioned route, method, parameter, status,
content-type, body, ordering, and stable error-code matrix over raw loopback
HTTP. Do not add a supported CLI or alternate transport.

## Scope

### Included

- A non-zero public integration target, expected at
  `apps/runtime/tests/graph_query_api.rs`, exercising real Runtime -> filesystem
  detector -> EDT/Designer XML builders -> Workspace snapshot -> query service
  -> HTTP server behavior.
- Positive accepted operations, separate configuration selection, exact stable
  identities/relations, missing/invalid/unavailable input, request bounds,
  deterministic ordering, lifecycle/shutdown, cleanup, and repeated fresh-run
  evidence required by ADR-0040.
- Reuse of the provenance-backed Sprint 17 Runtime fixture; new bounded fixture
  files only when Task 1 and ADR-0040 prove an uncovered stable oracle.
- Current-state synchronization in `README.md`, `docs/Architecture.md`, and
  `docs/architecture/semantic-model-2.md` when live implementation proves it.

### Excluded

- New production capability or external dependency, implementation redesign,
  graph semantic changes, file watching/rebuild/cache/CLI behavior, aggregate
  graphs, release claims, Coverage inflation, Sprint 18 completion, Roadmap
  state transition, and prompt retirement.

## Acceptance Criteria

- A non-zero public integration target proves actual production discovery and
  EDT/Designer XML graph construction before public loopback query responses.
- Evidence covers every accepted operation and the complete configuration,
  node, bound, malformed, unavailable, method, path, lifecycle, shutdown,
  ordering, cleanup, and repeated-run matrix with exact stable assertions.
- Both formats remain separate canonical configuration graphs; results preserve
  exact accepted identities and deterministic ordering without merged or
  cross-configuration inference.
- Tests use repository-owned provenance-backed inputs, explicit synchronization,
  raw loopback requests, and bounded hang guards with no arbitrary sleep, real
  signal, external service, ignored corpus dependency, symlink, or
  platform-specific absolute path.
- Health probes remain exact, all listener and snapshot observation resources
  close, and fresh applications produce equal wire observations without shared
  state.
- Current-state docs describe exactly implemented behavior and retain all
  Sprints 19-21 and later exclusions; Sprint 18 remains incomplete pending Task
  6.
- Complete workspace validation succeeds.

## Repository Safety

Preserve `.codex/`, accepted production behavior, dependencies, unrelated
crates/adapters, current prompt suites, and unrelated files. Stage only the
public integration test, explicitly proven fixture files, and exact
current-state docs when commit mode is authorized.

## Task-specific Validation

- `cargo test -p oneagent-runtime --test graph_query_api -- --list`
- `cargo test -p oneagent-runtime --test graph_query_api`
- `cargo test -p oneagent-runtime --test http_health`
- `cargo test -p oneagent-runtime --test workspace_service`
- `cargo test -p oneagent-graph --test query`
- Production EDT, Designer XML, filesystem, workspace, and graph package tests
  required by the accepted matrix.
- Complete workspace validation from `docs/codex/core/validation.md`.
- Verify every changed documentation claim against public tests.
- `git status --short`

## Suggested commit message

`Complete Sprint 18 Graph Query API evidence`

## Final report additions

Report the public test matrix and counts, fixture provenance, both production
format paths, exact operation and wire evidence, bounds/errors,
lifecycle/shutdown cleanup, repeated runs, current-state documentation,
complete validation, changed paths, commit, and final Git state.
