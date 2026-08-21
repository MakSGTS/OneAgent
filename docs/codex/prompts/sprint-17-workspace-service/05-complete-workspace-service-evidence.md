# Complete Sprint 17 Workspace Service Evidence

Continue OneAgent development.

## Reporting

- Prompt and repository content: English.
- User-visible reports: Russian.

## Profile

`docs/codex/profiles/runtime-service-implementation.md`

## Template

`docs/codex/templates/runtime-service-task.md`

## Authoritative documents

- `docs/Roadmap.md`, Sprint 17 execution plan
- `docs/architecture/workspace-service-investigation.md`
- `docs/architecture/semantic-model-2.md`
- `docs/Architecture.md`
- `README.md`
- `docs/adr/0002-runtime-composition-root.md`
- `docs/adr/0004-filesystem-workspace-discovery.md`
- `docs/adr/0036-designer-xml-adapter.md`
- `docs/adr/0037-runtime-service-container.md`
- `docs/adr/0038-http-api-health.md`
- `docs/adr/0039-workspace-service.md`

## Prerequisites / Required gate

Require committed Task 4 with the production Workspace service, successful full
validation, clean task-owned state, and a public observation seam capable of
proving real filesystem discovery and semantic builds. Internal fake-only tests
cannot satisfy this task.

## Task

Complete public production evidence for Workspace discovery and EDT/Designer
XML semantic-build orchestration, then synchronize current-state documentation
without completing Sprint 17.

## Runtime and service ownership

Exercise only public Runtime composition plus production detector/builders.
Prove that every fresh application owns its build, published snapshot, service
task, and cancellation resources independently.

## Lifecycle and state transitions

Prove the exact configured, initializing/building, published/ready, stopping,
and stopped observations accepted by ADR-0039 using lifecycle and channel
coordination instead of elapsed-time races.

## Concurrency and task ownership

Use bounded public test coordination and repository-owned temporary fixture
copies or generated bounded fixture roots whose provenance is documented by
Task 1. Prove that no blocking build or channel survives `App::run`.

## Cancellation, failure, and shutdown policy

Cover successful shutdown and every accepted invalid-root, discovery, format,
collision, and adapter-build failure observable through the public Runtime
boundary. Prove atomic publication and cleanup after failed startup.

## Health, readiness, and observability contract

Assert immutable published state, deterministic ordering/counts/identities and
diagnostics, lifecycle-derived readiness, and unchanged exact health wire
responses when applicable.

## Transport and client compatibility

No Workspace or graph endpoint is introduced. Existing public health probes may
be used only to prove readiness integration; snapshot evidence must use the
accepted transport-neutral public Runtime boundary.

## Scope

### Included

- A non-zero public integration target, expected at
  `apps/runtime/tests/workspace_service.rs`, exercising real production
  discovery and EDT/Designer XML builders against provenance-backed
  repository-owned inputs.
- Positive, empty, mixed/multiple, deterministic ordering, failure atomicity,
  readiness, shutdown, cleanup, and repeated fresh-run evidence required by
  ADR-0039.
- A bounded Runtime fixture under `apps/runtime/tests/fixtures/` only when Task
  1 and ADR-0039 prove that reusing an existing fixture directly is unstable or
  cannot cover the accepted matrix.
- Current-state synchronization in `README.md`, `docs/Architecture.md`, and
  `docs/architecture/semantic-model-2.md` when live implementation proves it.

### Excluded

- New production capability or external dependency, implementation redesign,
  graph-query/Workspace HTTP routes, watcher/rebuild/cache/CLI behavior, release
  claims, Coverage support inflation, Sprint 17 completion, Roadmap state
  transition, and prompt retirement.

## Acceptance Criteria

- A non-zero public integration target proves actual Runtime -> filesystem
  detector -> EDT/Designer XML builder orchestration and immutable snapshot
  observation.
- Evidence covers every accepted positive, empty/multiple, invalid/failing,
  atomicity, lifecycle/readiness, shutdown/cleanup, ordering, and repeated-run
  case with exact stable assertions.
- Fixtures are repository-owned, provenance-documented, cross-platform, and do
  not depend on ignored local corpora, external services, symlinks, or
  platform-specific absolute paths.
- Tests use explicit synchronization and bounded hang guards with no arbitrary
  sleep or real process signal.
- Current-state docs describe exactly implemented behavior and retain all Sprint
  18-21 exclusions; Sprint 17 remains incomplete pending Task 6.
- Complete workspace validation succeeds.

## Repository Safety

Preserve `.codex/`, accepted production behavior, dependencies, unrelated
crates/adapters, current prompt suites, and unrelated files. Stage only the
public integration test, explicitly proven fixture files, and exact current-state
docs when commit mode is authorized.

## Task-specific Validation

- `cargo test -p oneagent-runtime --test workspace_service -- --list`
- `cargo test -p oneagent-runtime --test workspace_service`
- `cargo test -p oneagent-runtime --test service_container`
- `cargo test -p oneagent-runtime --test http_health`
- Production EDT, Designer XML, filesystem, workspace, and graph package tests
  required by the accepted matrix.
- Complete workspace validation from `docs/codex/core/validation.md`.
- Verify every changed documentation claim against public tests.
- `git status --short`

## Suggested commit message

`Complete Sprint 17 Workspace service evidence`

## Final report additions

Report the public test matrix and counts, fixture provenance, both production
format paths, snapshot/readiness/failure/atomicity behavior, shutdown and
repeated-run evidence, current-state documentation, complete validation,
changed paths, commit, and final Git state.
