# Complete Sprint 15 Runtime Service Container Evidence

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
- `docs/architecture/runtime-service-container-investigation.md`
- `docs/adr/0002-runtime-composition-root.md`
- `docs/Architecture.md`
- `docs/Roadmap.md`, Sprint 15 completion gates
- committed Tasks 3 and 4

## Prerequisites / Required gate

Require committed Task 4, successful implementation validation, and clean
task-owned state. Stop if the public Runtime boundary cannot support an
integration oracle without changing ADR-0037.

## Task

Complete public end-to-end Runtime evidence and synchronize current-state
documentation without changing the accepted service-container architecture or
adding later-sprint behavior.

## Runtime and service ownership

Exercise the public production construction and run boundary with deterministic
probe services whose events prove composition-root, container, task, and
shutdown ownership.

## Lifecycle and state transitions

Cover successful long-running execution, ordered shutdown, partial startup
failure, running-service failure, requested cancellation, and repeated fresh
build/run cycles required by ADR-0037.

## Concurrency and task ownership

Assert all probe tasks terminate and no background activity occurs after the
public run future resolves.

## Cancellation, failure, and shutdown policy

Prove exact public result/error distinctions and cleanup for every accepted
terminal path.

## Health, readiness, and observability contract

Verify only internal lifecycle/structured event evidence. Document HTTP health
and readiness as Sprint 16 deferred scope.

## Transport and client compatibility

No network transport or client is included. Public library integration tests
must not bind ports or require external services.

## Scope

### Included

- Non-zero public Runtime integration tests using deterministic probes.
- Cross-platform-compatible ordering, cleanup, failure, cancellation, and
  repeated-run evidence.
- Synchronization of `README.md`, `docs/Architecture.md`, `docs/Roadmap.md`, and
  Semantic Model current-state text only where live behavior requires it.
- Final Sprint 15 acceptance matrix and explicit deferred boundaries without
  marking the sprint completed.

### Excluded

- Architecture changes, silent implementation fixes outside evidence needs,
  HTTP/health routes, real sockets/signals, workspace/graph services, watchers,
  persistence, CLI, MCP/LSP/IDE, performance claims, sprint completion, and
  prompt retirement.

## Acceptance Criteria

- At least one public non-zero integration target proves a genuinely pending
  Runtime remains active until injected shutdown.
- Startup rollback, service failure, requested shutdown, deterministic ordering,
  complete cleanup, and repeated fresh runs are independently observable.
- macOS and Windows CI compatibility is preserved; tests use no Unix-only
  behavior or arbitrary sleeps.
- Documentation describes only implemented behavior and keeps Sprints 16–21
  explicit.
- Full workspace validation succeeds and Sprint 15 remains incomplete pending
  Task 6 review.

## Repository Safety

Modify only focused Runtime tests and exact current-state documents named by
the Change Contract. Preserve prompts, `.codex/`, unrelated crates, and all
deferred product surfaces.

## Task-specific Validation

- Run the exact public Runtime integration test target and verify non-zero tests.
- `cargo test -p oneagent-runtime --no-fail-fast`
- Complete workspace validation from `docs/codex/core/validation.md`.
- Validate current-state documentation links and `git diff --check`.

## Suggested commit message

`Complete Sprint 15 Runtime service container evidence`

## Final report additions

Report the public integration matrix, ownership and cleanup evidence, terminal
outcomes, cross-platform constraints, documentation state, deferred scope,
exact validation, commit, and Git state.
