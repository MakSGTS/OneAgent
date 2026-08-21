# Define Sprint 17 Workspace Service Contract

Continue OneAgent development.

## Reporting

- Prompt and repository content: English.
- User-visible reports: Russian.

## Profile

`docs/codex/profiles/architecture.md`

## Template

`docs/codex/templates/architecture-task.md`

## Authoritative documents

- `docs/Roadmap.md`, Sprint 17 execution plan
- `docs/architecture/workspace-service-investigation.md`
- `docs/architecture/semantic-model-2.md`
- `docs/adr/0002-runtime-composition-root.md`
- `docs/adr/0004-filesystem-workspace-discovery.md`
- `docs/adr/0036-designer-xml-adapter.md`
- `docs/adr/0037-runtime-service-container.md`
- `docs/adr/0038-http-api-health.md`

## Prerequisites / Required gate

Require committed Task 1 evidence that every first-slice decision has a
repository-owned source, production entry point, and test oracle. Stop if the
investigation reports missing or conflicting evidence.

## Task

Create and accept `docs/adr/0039-workspace-service.md`, defining the smallest
Runtime-owned Workspace lifecycle and semantic-build orchestration contract.
Synchronize only planning-level architecture text required to make the decision
unambiguous. Implement no production behavior.

## Scope

### Included

- Composition ownership and dependency direction between Runtime, workspace
  domain ports, filesystem discovery, EDT/Designer XML builders, and graph.
- Workspace root configuration and validation; discovery ordering and supported
  format dispatch.
- Canonical immutable snapshot/result shape, graph authority, diagnostics,
  identity, deterministic ordering, and duplicate/collision policy.
- Initial-build atomicity and partial-failure policy, startup acknowledgement,
  lifecycle-derived readiness, error classification/source chains, and
  observable state.
- Task/resource ownership, blocking-work execution, cancellation, shutdown,
  repeated fresh runs, public test seams, first production slice, migration,
  and implementation prerequisites.
- Explicit rejected alternatives and deferred scope.

### Excluded

- Rust implementation, Cargo changes, fixtures, accepted graph-query API,
  HTTP workspace endpoints, file watching, incremental rebuild triggers,
  persistence/cache, supported CLI behavior, MCP/LSP/IDE/AI, auth/security,
  retries/restart, forced termination, performance targets, Coverage support
  transitions, sprint completion, and prompt retirement.

## Acceptance Criteria

- ADR-0039 answers every Task 1 decision question with one canonical contract
  grounded in repository evidence and existing accepted ADRs.
- The contract preserves one semantic authority per published build, stable
  configuration identity and ordering, source provenance, adapter boundaries,
  ADR-0037 ownership/error semantics, and ADR-0038 readiness truth.
- The initial slice defines exact success, empty, invalid-root, discovery,
  unsupported-format, duplicate/collision, adapter-build, cancellation,
  shutdown, and repeated-run expectations to the extent proven testable.
- Publication and failure atomicity are explicit; no consumer can observe an
  unsupported partial or mutable graph state.
- All tasks, blocking work, channels, snapshot references, and long-lived
  resources have explicit owners and terminal behavior.
- Rejected alternatives, compatibility impact, implementation order, validation
  matrix, current limitations, and Sprints 18-21 deferrals are explicit.
- Current-state documents do not claim implementation and Sprint 17 remains
  `next`.

## Repository Safety

Create only `docs/adr/0039-workspace-service.md` and modify only the minimum
planning-level architecture document if the accepted decision requires it.
Preserve `.codex/`, production code, manifests, fixtures, current prompt suites,
Roadmap state, current-state implementation claims, and unrelated files. Stage
only explicitly enumerated ADR-owned paths when commit mode is authorized.

## Task-specific Validation

- Verify decision/evidence consistency with the Task 1 investigation and all
  cited public APIs.
- Validate internal links, status, alternatives, first slice, implementation
  prerequisites, accepted/deferred scope, and Coverage impact.
- `git diff --check`
- `git status --short`

## Suggested commit message

`Define Sprint 17 Workspace service contract`

## Final report additions

Report the accepted contract, rejected alternatives, dependency and ownership
direction, snapshot/readiness/failure decisions, implementation prerequisites,
deferred scope, changed paths, validation, commit, and final Git state.
