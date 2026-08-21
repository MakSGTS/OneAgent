# Define Sprint 20 Persistent Cache Contract

Continue OneAgent development.

## Reporting

- Prompt and repository content: English.
- User-visible reports: Russian.

## Profile

`docs/codex/profiles/architecture.md`

## Template

`docs/codex/templates/architecture-task.md`

## Authoritative documents

- `docs/Roadmap.md`, Sprint 20 execution plan
- `docs/architecture/persistent-cache-investigation.md`
- `docs/Architecture.md`
- `docs/architecture/semantic-model-2.md`
- `docs/adr/0039-workspace-service.md`
- `docs/adr/0040-graph-query-api.md`
- `docs/adr/0041-file-watching.md`

## Prerequisites / Required gate

Require committed Task 1 evidence that every first-slice decision has a
repository-owned production source and deterministic test oracle. Stop if the
investigation reports missing or conflicting evidence.

## Task

Create and accept `docs/adr/0042-persistent-cache.md`, defining the smallest
cross-platform complete Runtime Workspace Persistent Cache contract. Synchronize
only planning-level architecture text required to make the decision unambiguous.
Implement no production behavior.

## Scope

### Included

- Canonical in-memory semantic authority, persisted representation owner,
  storage/load/write owner, Runtime integration owner, readers, and dependency
  direction.
- Exact complete persisted envelope and payload, checked reconstruction,
  validation, stable schema and semantic-build versions, encoding vocabulary,
  deterministic ordering/bytes guarantees, and intentionally reconstructed or
  excluded state.
- Workspace/cache identity, complete validity and invalidation inputs, ignored
  paths, scan/load/build/write race closure, and exact hit/miss/rejection rules.
- Cache location and path containment, file/directory names, temporary and
  complete replacement behavior, cleanup, permissions/symlink boundary, write
  interruption, and repeated-process behavior.
- Current/older/newer/unknown compatibility, clean-rebuild migration policy,
  missing/malformed/truncated/duplicate/semantically-invalid/unreadable entry
  classification, corruption containment, recovery, and last-valid behavior.
- Runtime startup and post-watch rebuild integration, blocking-work ownership,
  publication, update status, health/readiness, Graph Query consistency,
  cancellation, shutdown, observability, and deterministic test strategy.
- Dependency choice and approval gate, compatibility impact, first production
  slice, rejected alternatives, implementation prerequisites, and deferred
  scope.

### Excluded

Rust implementation, Cargo changes, fixtures, graph/parser/adapter semantic
changes, incremental or partial persistence, cache management HTTP/CLI APIs,
cross-process writers/locking, remote/shared cache, compression, encryption,
eviction, automatic historical migration without real schema evidence, native
watchers, Git/network workspaces, new external dependencies without approval,
benchmarks, performance/security claims, Coverage transitions, sprint
completion, and prompt retirement.

## Acceptance Criteria

- ADR-0042 answers every Task 1 decision question with one canonical contract
  grounded in repository evidence and accepted ADRs.
- `WorkspaceSnapshot` and its canonical `SemanticGraph` values remain the only
  published semantic authority; persisted bytes cannot bypass checked domain
  construction, complete validation, or exact source/semantic validity.
- Envelope/payload fields, version vocabulary, ordering, identity, invalidation,
  location, path containment, replacement, compatibility, corruption,
  clean-rebuild recovery, and every typed outcome are closed and explicit.
- Load/build/write and watcher races, valid-hit and miss behavior, failed writes,
  last-valid publication, cancellation, shutdown, cleanup, and repeated fresh
  runs have one owner and deterministic behavior.
- Runtime lifecycle/health, File Watching coalescing/recovery, Workspace
  publication, and Graph Query wire/single-snapshot compatibility are preserved;
  cache state is not a second readiness label or public transport contract.
- Dependency choice is explicit. If implementation requires a new production
  dependency, Task 3 remains gated on explicit user approval before any manifest
  or lockfile change.
- Public evidence covers both tracked formats, complete snapshot content, cold
  and warm paths, source/semantic invalidation, corruption/incompatibility,
  write failure, recovery, watcher replacement, query equivalence, shutdown,
  cleanup, supported platforms, and fresh repetition.
- Rejected alternatives, first slice, implementation order, Coverage impact,
  compatibility, and Sprint 21/later deferrals are explicit. Sprint 20 remains
  `next` and current-state docs do not claim implementation.

## Repository Safety

Create only `docs/adr/0042-persistent-cache.md` and modify only the minimum
planning-level architecture document if the accepted decision requires it.
Preserve `.codex/`, production code, manifests/lockfile, fixtures, prompt suites,
Roadmap state, current-state implementation claims, and unrelated files. Stage
only explicitly enumerated ADR-owned paths when commit mode is authorized.

## Task-specific Validation

- Verify decision/evidence consistency with Task 1 and all cited public APIs.
- Validate internal links, status, closed authority/schema/identity/invalidation/
  storage/compatibility/corruption/recovery/lifecycle/dependency matrices,
  alternatives, first slice, prerequisites, accepted/deferred scope, and
  Coverage impact.
- `git diff --check`
- `git status --short`

## Suggested commit message

`Define Sprint 20 Persistent Cache contract`

## Final report additions

Report accepted authority, schema/payload, versions, encoding, identity,
invalidation, storage/replacement, compatibility, corruption/recovery, Runtime
lifecycle, dependency/approval, testing, and compatibility contracts; rejected
alternatives; prerequisites; deferred scope; changed paths; validation; commit;
and final Git state.
