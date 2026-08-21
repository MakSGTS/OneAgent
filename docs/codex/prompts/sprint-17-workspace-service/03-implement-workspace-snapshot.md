# Implement Sprint 17 Workspace Snapshot

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
- `docs/adr/0002-runtime-composition-root.md`
- `docs/adr/0004-filesystem-workspace-discovery.md`
- `docs/adr/0036-designer-xml-adapter.md`
- `docs/adr/0037-runtime-service-container.md`
- `docs/adr/0038-http-api-health.md`
- `docs/adr/0039-workspace-service.md`

## Prerequisites / Required gate

Require committed Task 2 with accepted ADR-0039, successful documentation
validation, and clean task-owned state. Stop rather than selecting a different
snapshot, dispatch, atomicity, ordering, or ownership contract in code.

## Task

Implement the accepted source-neutral immutable Workspace snapshot and bounded
semantic-build dispatch boundary with focused deterministic tests. Do not
register or run the long-lived Runtime service in this task.

## Runtime and service ownership

- Keep snapshot and build orchestration in the exact layer selected by ADR-0039.
- Preserve adapter ownership of source parsing and graph ownership of semantic
  facts; do not copy semantic authority into Runtime or the workspace model.

## Lifecycle and state transitions

- Implement only the pre-service build/result state required by ADR-0039.
- Do not alter App lifecycle or HTTP readiness until Task 4 owns service
  composition.

## Concurrency and task ownership

- Introduce no detached task or hidden executor.
- If ADR-0039 defines a blocking-build boundary, expose the accepted callable
  seam without spawning unowned work in this task.

## Cancellation, failure, and shutdown policy

- Preserve exact source errors and deterministic result atomicity selected by
  ADR-0039.
- Do not invent retries, partial publication, timeout, abort, or shutdown
  behavior.

## Health, readiness, and observability contract

Expose only the accepted immutable snapshot/build observations. Runtime health
and HTTP response mapping remain unchanged in this task.

## Transport and client compatibility

No transport, endpoint, or supported client behavior is added.

## Scope

### Included

- Minimal source-neutral snapshot/result types, supported format dispatch, and
  deterministic ordered access accepted by ADR-0039.
- Exact domain/adapter/graph path dependencies required by that boundary, using
  workspace members and locked versions only.
- Focused tests for empty and representative builds, ordering, duplicate or
  collision behavior, error preservation, immutability, and repetition as
  applicable to the accepted first slice.
- Minimal Rustdoc and public exports required by Task 4 and public tests.

### Excluded

- Runtime service registration/execution, AppState/lifecycle/readiness changes,
  HTTP routes, file watching, incremental rebuilds, persistent state, CLI,
  semantic graph model changes, adapter parser changes, new external production
  dependencies, current-state docs, sprint transition, and prompt retirement.

## Acceptance Criteria

- One accepted build dispatch maps every supported discovered format to its
  existing production builder without source-format logic leaking into graph or
  transport code.
- The published result shape is immutable, deterministic, and preserves the
  accepted configuration identity, source provenance, diagnostics, and graph
  authority.
- Empty, ordered multi-configuration, supported-format, invalid/unsupported,
  duplicate/collision, adapter failure, and repeated-build outcomes match
  ADR-0039 wherever applicable.
- Failed construction exposes no unsupported partial snapshot.
- Focused tests are non-zero, repository-owned, deterministic, and contain no
  arbitrary sleeps or external services.
- Existing adapter, graph, workspace, Runtime, and HTTP behavior remains green.

## Repository Safety

Preserve `.codex/`, source parser behavior, graph semantics, HTTP composition,
current prompt suites, current-state docs, and unrelated files. Do not add an
external dependency. Stage only exact task-owned manifests, source files, and
focused tests when commit mode is authorized.

## Task-specific Validation

- Run the exact new focused snapshot/build test target or non-zero filter.
- `cargo test -p oneagent-workspace`
- Affected EDT, Designer XML, filesystem, graph, and Runtime focused checks
  identified by the live diff.
- Complete workspace validation from `docs/codex/core/validation.md`.
- `git status --short`

## Suggested commit message

`Implement Sprint 17 Workspace snapshot`

## Final report additions

Report type and dependency ownership, supported build dispatch, result and
atomicity behavior, focused test counts, preserved adapter/graph/HTTP behavior,
complete validation, changed paths, commit, and final Git state.
