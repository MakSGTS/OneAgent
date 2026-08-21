# Define Sprint 15 Runtime Service Container Contract

Continue OneAgent development.

## Reporting

- Prompt and repository content: English.
- User-visible reports: Russian.

## Profile

`docs/codex/profiles/architecture.md`

## Template

`docs/codex/templates/architecture-task.md`

## Authoritative documents

- `docs/Roadmap.md`, Sprint 15 execution plan
- `docs/adr/0002-runtime-composition-root.md`
- `docs/architecture/runtime-service-container-investigation.md`
- `docs/architecture/semantic-model-2.md`
- `docs/Architecture.md`
- `docs/codex/workflows/runtime-service.md`

## Prerequisites / Required gate

Require committed Task 1 evidence sufficient to decide every included lifecycle,
ownership, concurrency, failure, cancellation, shutdown, observability, public
boundary, and testability contract. Stop without edits when evidence is missing.

## Task

Create `docs/adr/0037-runtime-service-container.md` and synchronize only the
planning-level architecture text required to identify the accepted Sprint 15
Runtime boundary.

## Scope

### Included

- Composition-root and public library ownership; service registration and
  identity; task/resource ownership; construction/start/run/stop boundaries.
- Deterministic registration/start/shutdown order, duplicate behavior, partial
  startup rollback, service completion/failure, cancellation propagation, join
  failure, and terminal-state rules.
- App lifecycle integration, injected shutdown testing boundary, main-process
  signal responsibility, and internal observable state permitted before Sprint
  16 health APIs.
- Error taxonomy and propagation, structured observability requirements,
  deterministic test oracle, first production slice, migration impact,
  implementation prerequisites, and deferred scope.
- Rejected alternatives, including detached tasks, adapter-owned composition,
  implicit global state, arbitrary-sleep tests, and pulling HTTP forward.

### Excluded

- Production Rust/Cargo changes, HTTP routes or health schema, workspace and
  graph services, file watching, persistence, supported CLI, MCP/LSP/IDE,
  performance claims, release state, and prompt retirement.

## Acceptance Criteria

- ADR-0037 is accepted, internally consistent, and evidence-backed.
- Every long-lived service, task, channel, cancellation source, and lifecycle
  transition has one explicit owner and terminal behavior.
- Startup failure, running-service failure, requested shutdown, and task join
  failure are distinguishable and cannot leave detached work.
- The App can remain running until an injected or process shutdown source fires;
  tests need no OS signal or arbitrary sleep.
- The decision preserves ADR-0002 composition and immutable shared-state
  constraints or explicitly records a compatible migration.
- Sprint 16–21 capabilities remain deferred and Sprint 15 remains incomplete.

## Repository Safety

Do not modify production code, Cargo files, `.codex/`, prompt suites, or
unrelated documentation. Stage only exact task-owned architecture documents
when commit mode is authorized.

## Task-specific Validation

- Validate links, headings, ADR status, evidence citations, and Roadmap agreement.
- Verify accepted versus deferred scope and unchanged Sprint 15 state.
- `git diff --check`
- `git status --short`

## Suggested commit message

`Define Sprint 15 Runtime service container contract`

## Final report additions

Report accepted ownership, lifecycle, service identity, failure, cancellation,
shutdown, observability, public-boundary, testability, deferred, and migration
contracts; rejected alternatives; validation; commit; and Git state.
