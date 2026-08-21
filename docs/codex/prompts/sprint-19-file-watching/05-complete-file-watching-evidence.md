# Complete Sprint 19 File Watching Evidence

Continue OneAgent development.

## Reporting

- Prompt and repository content: English.
- User-visible reports: Russian.

## Profile

`docs/codex/profiles/runtime-service-implementation.md`

## Template

`docs/codex/templates/runtime-service-task.md`

## Authoritative documents

- `docs/Roadmap.md`, Sprint 19 execution plan
- `docs/architecture/file-watching-investigation.md`
- `docs/Architecture.md`
- `docs/architecture/semantic-model-2.md`
- `docs/adr/0037-runtime-service-container.md`
- `docs/adr/0038-http-api-health.md`
- `docs/adr/0039-workspace-service.md`
- `docs/adr/0040-graph-query-api.md`
- `docs/adr/0041-file-watching.md`

## Prerequisites / Required gate

Require committed Task 4 with accepted watching and rebuild orchestration,
successful focused and complete validation, and clean task-owned state. Stop if
the production entry point or deterministic public test seam is absent.

## Task

Complete public production evidence for File Watching through real filesystem
observation, EDT and Designer XML rebuilds, immutable publication, Graph Query
visibility, failure/recovery, Runtime shutdown, cleanup, and repeated fresh
runs. Synchronize current-state documentation without completing Sprint 19.

## Runtime and service ownership

- Exercise the public `oneagent_runtime` surface and production composition;
  do not replace watcher, detector, adapter, Workspace builder, or Graph Query
  boundaries with test-only semantic implementations.
- Keep tracked fixture provenance explicit and mutate only disposable temporary
  copies during tests.

## Lifecycle and state transitions

- Prove the accepted initial build, relevant change, coalesced rebuild,
  invalid-build behavior, recovery, and shutdown sequence.
- Assert lifecycle and publication through events/channels/observations;
  timeouts are hang guards and arbitrary sleeps are not evidence.

## Concurrency and task ownership

- Prove serialized builds, accepted in-flight change handling, old-or-new atomic
  reader visibility, no surviving watcher/build work, closed observers, and
  equal fresh executions.

## Cancellation, failure, and shutdown policy

- Exercise accepted observation and semantic-build failures without exposing a
  partial snapshot or losing the primary Runtime outcome.
- Prove recovery when accepted and complete reverse shutdown cleanup.

## Health, readiness, and observability contract

- Preserve exact health and Graph Query route/schema/error compatibility.
- Use public Graph Query requests to prove semantic visibility only where the
  accepted fixture mutation supplies an exact deterministic oracle.

## Scope

### Included

- Public Runtime integration tests using tracked EDT and Designer XML fixture
  inputs copied to temporary roots.
- Relevant and irrelevant changes, add/modify/remove/rename-equivalent source
  transitions, duplicate/burst coalescing, invalid build and recovery, atomic
  snapshot/query visibility, lifecycle, cleanup, and repeated-run evidence.
- Minimal fixture provenance/inventory updates only when evidence proves the
  current tracked fixture is insufficient.
- Synchronization of `README.md`, `docs/Architecture.md`, and
  `docs/architecture/semantic-model-2.md` current implementation text.

### Excluded

Architecture changes, watcher/rebuild redesign, new dependencies, graph/parser/
adapter semantics, incremental mutation, persistent cache, supported CLI,
watch-control routes, subscriptions/streaming, Git/network workspaces,
benchmarks, completion review, Roadmap transition, and prompt retirement.

## Acceptance Criteria

- Non-zero public tests traverse production filesystem observation, discovery,
  EDT and Designer XML builders, atomic Workspace publication, Graph Query
  observation, and Runtime cancellation/shutdown.
- Evidence covers accepted relevant/irrelevant, add/modify/remove/rename-
  equivalent, burst, invalid/recovery, lifecycle, atomicity, cleanup, and fresh
  repetition rows with exact observable outcomes.
- Test inputs are tracked and provenance-backed; mutations occur only in
  disposable temporary copies and require no network, ignored corpus, external
  service, or arbitrary sleep.
- Current-state docs accurately describe implemented behavior and retain
  persistence, supported CLI, and later integrations as planned.
- Sprint 19 remains `next`; no review artifact or prompt deletion is created.

## Repository Safety

Modify only public Runtime test/fixture evidence and the three named
current-state documents when required. Preserve `.codex/`, prompt suites,
Roadmap state, ADRs, production implementation except for the smallest
evidence-proven testability correction allowed by the accepted contract,
manifests/lockfile, graph/adapter semantics, and unrelated files. Stage only
enumerated task-owned paths.

## Task-specific Validation

- Run exact non-zero public File Watching integration tests and affected
  Workspace/Graph Query/health compatibility targets.
- Verify fixture inventory/provenance and absence of arbitrary sleep evidence.
- Run the canonical complete workspace validation from
  `docs/codex/core/validation.md`.
- Validate documentation links and `git status --short`.

## Suggested commit message

`Complete Sprint 19 File Watching evidence`

## Final report additions

Report the public production matrix, fixture provenance, lifecycle/query
compatibility, failure/recovery, cleanup/repetition, documentation changes,
focused/full validation, changed paths, commit, and final Git state.
