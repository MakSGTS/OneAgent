# Define Sprint 19 File Watching Contract

Continue OneAgent development.

## Reporting

- Prompt and repository content: English.
- User-visible reports: Russian.

## Profile

`docs/codex/profiles/architecture.md`

## Template

`docs/codex/templates/architecture-task.md`

## Authoritative documents

- `docs/Roadmap.md`, Sprint 19 execution plan
- `docs/architecture/file-watching-investigation.md`
- `docs/Architecture.md`
- `docs/architecture/semantic-model-2.md`
- `docs/adr/0002-runtime-composition-root.md`
- `docs/adr/0037-runtime-service-container.md`
- `docs/adr/0038-http-api-health.md`
- `docs/adr/0039-workspace-service.md`
- `docs/adr/0040-graph-query-api.md`

## Prerequisites / Required gate

Require committed Task 1 evidence that every first-slice decision has a
repository-owned production source and deterministic test oracle. Stop if the
investigation reports missing or conflicting evidence.

## Task

Create and accept `docs/adr/0041-file-watching.md`, defining the smallest
cross-platform Runtime File Watching and Workspace rebuild contract.
Synchronize only planning-level architecture text required to make the decision
unambiguous. Implement no production behavior.

## Scope

### Included

- Authority and dependency direction between filesystem observation, Runtime
  service ownership, Workspace complete builds, immutable publication, Graph
  Query observation, HTTP lifecycle compatibility, and future cache/client
  consumers.
- Exact watched root boundary, relevant input families and exclusions,
  normalized change and failure vocabulary, path treatment, duplicate and
  platform-event equivalence, and unsupported cases.
- Bounded coalescing, scheduling, serialization, in-flight change handling,
  overflow or observation failure, rebuild triggering, and cancellation.
- Atomic complete-snapshot publication, reader consistency, invalid-build
  retention or clearing, recovery, repeated failure, and initial-build
  compatibility.
- Exact owner of tasks, channels, timers, native resources, blocking work, and
  terminal cleanup; lifecycle, readiness, health, shutdown, and observability.
- Dependency choice and approval gate, deterministic cross-platform test
  strategy, compatibility/migration, first production slice, rejected
  alternatives, implementation prerequisites, and deferred scope.

### Excluded

Rust implementation, Cargo changes, fixtures, graph/parser/adapter semantic
changes, incremental graph or Semantic Index mutation, persistence/cache,
supported CLI implementation, watch-control routes, subscriptions/streaming,
Git/network workspace ingestion, edit transactions, new external dependencies
without explicit approval, performance targets, Coverage transitions, sprint
completion, and prompt retirement.

## Acceptance Criteria

- ADR-0041 answers every Task 1 decision question with one canonical contract
  grounded in repository evidence and accepted ADRs.
- The contract preserves `SemanticGraph` and production adapters as semantic
  authorities, separate complete immutable configuration snapshots, stable
  Graph Query single-snapshot observation, and ADR-0037/0038/0039/0040
  lifecycle and compatibility rules.
- Watched boundary, relevant and irrelevant changes, normalized signal and
  error vocabulary, coalescing bounds, scheduling, build serialization,
  publication, failure retention or clearing, and recovery are closed and
  explicit for every accepted case.
- Every background task, blocking operation, channel, timer, and resource has
  one owner and terminal cancellation/shutdown behavior; no arbitrary sleeps,
  detached work, mutable readiness authority, or platform event leaks remain.
- Dependency choice is explicit. If implementation requires a new production
  dependency, the ADR records the exact dependency purpose and Task 3 remains
  gated on explicit user approval before any manifest change.
- Public evidence covers tracked EDT and Designer XML inputs, relevant and
  irrelevant changes, add/modify/remove/rename-equivalent transitions, bursts,
  invalid build and recovery, atomic visibility, graph-query observation,
  shutdown, cleanup, supported platforms, and fresh repetition.
- Compatibility impact, rejected alternatives, implementation order, current
  limitations, Coverage impact, and Sprint 20/21 deferrals are explicit.
- Current-state documents do not claim implementation and Sprint 19 remains
  `next`.

## Repository Safety

Create only `docs/adr/0041-file-watching.md` and modify only the minimum
planning-level architecture document if the accepted decision requires it.
Preserve `.codex/`, production code, manifests, lockfile, fixtures, current
prompt suites, Roadmap state, current-state implementation claims, and unrelated
files. Stage only explicitly enumerated ADR-owned paths when commit mode is
authorized.

## Task-specific Validation

- Verify decision/evidence consistency with Task 1 and all cited public APIs.
- Validate internal links, status, closed ownership/change/coalescing/rebuild/
  publication/failure/recovery/lifecycle/dependency matrices, alternatives,
  first slice, implementation prerequisites, accepted/deferred scope, and
  Coverage impact.
- `git diff --check`
- `git status --short`

## Suggested commit message

`Define Sprint 19 File Watching contract`

## Final report additions

Report accepted ownership, watched boundary, normalized changes, coalescing,
rebuild, publication, failure/recovery, lifecycle, dependency/approval, testing,
and compatibility contracts; rejected alternatives; prerequisites; deferred
scope; changed paths; validation; commit; and final Git state.
