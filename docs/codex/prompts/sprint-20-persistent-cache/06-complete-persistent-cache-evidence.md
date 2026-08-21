# Complete Sprint 20 Persistent Cache Evidence

Continue OneAgent development.

## Reporting

- Prompt and repository content: English.
- User-visible reports: Russian.

## Profile

`docs/codex/profiles/persistent-state-implementation.md`

## Additional workflow

`docs/codex/workflows/runtime-service.md`

## Template

`docs/codex/templates/persistent-state-task.md`

## Authoritative documents

- `docs/Roadmap.md`, Sprint 20 execution plan
- `docs/architecture/persistent-cache-investigation.md`
- `docs/Architecture.md`
- `docs/architecture/semantic-model-2.md`
- `docs/adr/0039-workspace-service.md`
- `docs/adr/0040-graph-query-api.md`
- `docs/adr/0041-file-watching.md`
- `docs/adr/0042-persistent-cache.md`

## Prerequisites / Required gate

Require committed Task 5 with accepted codec, storage/invalidation, and Runtime
cache lifecycle integration, successful focused and complete validation, and
clean task-owned state. Stop if the production entry point or deterministic
public test seam is absent.

## Task

Complete public production evidence for Persistent Cache through tracked EDT and
Designer XML source inputs, cold and warm Runtime starts, File Watching rebuilds,
Workspace publication, Graph Query visibility, incompatibility/corruption/write
failure and recovery, shutdown, cleanup, and repeated fresh runs. Synchronize
current-state documentation without completing Sprint 20.

## Production and ownership evidence

- Exercise only public `oneagent_runtime` composition and the accepted production
  source observation, cache, detector, adapter, Workspace, File Watching, and
  Graph Query paths.
- Keep tracked fixture provenance explicit and mutate source/cache state only in
  disposable temporary roots through accepted public seams.

## Public evidence matrix

- Prove cold miss/build/write and warm exact hit without semantic adapter
  rebuilding for both accepted source formats.
- Prove complete snapshot, graph payload/provenance, diagnostic, request,
  statistic, report, and Graph Query equivalence with a clean build.
- Prove source and semantic-contract invalidation, incompatible/newer/older,
  malformed/truncated/corrupt/semantically-invalid state, write failure, and
  clean-build recovery with exact accepted outcomes.
- Prove a relevant file change produces a complete new semantic snapshot and
  cache entry, an irrelevant cache-owned change does not self-trigger, readers
  observe old-or-new complete values, and subsequent fresh startup restores the
  replacement.
- Prove lifecycle/health/query compatibility, cancellation, shutdown, temporary
  cleanup, closed observers, released resources, and equal fresh repetitions.

## Scope

### Included

- A public Runtime Persistent Cache integration target using the tracked Sprint
  17/19 EDT and Designer fixture copied into temporary source/cache roots.
- Minimal fixture provenance/inventory updates only when evidence proves the
  existing tracked fixture insufficient.
- Synchronization of `README.md`, `docs/Architecture.md`, and
  `docs/architecture/semantic-model-2.md` current implementation text.
- The smallest evidence-proven testability correction consistent with ADR-0042,
  only when public acceptance cannot otherwise be observed.

### Excluded

Architecture or codec/store/runtime redesign, new dependencies, graph/parser/
adapter semantic changes, incremental persistence, cache management API,
supported CLI, cross-process/remote cache, compression/encryption/eviction,
native watchers, benchmarks, completion review, Roadmap transition, and prompt
retirement.

## Acceptance Criteria

- Non-zero public tests traverse production source observation, cache load/write,
  filesystem discovery, both semantic builders on cold paths, Workspace/File
  Watching publication, Graph Query, and Runtime cancellation/shutdown.
- Warm-hit evidence deterministically proves no semantic adapter rebuild while
  preserving every accepted complete snapshot observation and query result.
- The public matrix covers cold/warm, invalidation, incompatible/corrupt/partial,
  write failure, recovery, file-change replacement, no feedback loop, atomic
  visibility, lifecycle, cleanup, and fresh repetition with exact outcomes.
- Test inputs are tracked and provenance-backed; mutations occur only in
  disposable copies and require no network, ignored corpus, host-global state,
  external service, platform-specific timing, or arbitrary sleep.
- Current-state docs accurately describe implemented behavior and retain the
  supported CLI and later integrations as planned. Sprint 20 remains `next`;
  no review artifact or prompt deletion is created.

## Repository Safety

Modify only public Runtime test/fixture evidence and the three named current-
state documents when required. Preserve `.codex/`, prompt suites, Roadmap state,
ADRs, production implementation except for the smallest evidence-proven
testability correction, manifests/lockfile, graph/adapter semantics, HTTP
schemas, and unrelated files. Stage only enumerated task-owned paths.

## Task-specific Validation

- List and run exact non-zero public Persistent Cache integration tests and
  affected Workspace/File Watching/Graph Query/health compatibility targets.
- Verify fixture inventory/provenance, disposable cache isolation, and absence
  of arbitrary sleep evidence.
- Run the canonical complete workspace validation from
  `docs/codex/core/validation.md`.
- Validate documentation links and `git status --short`.

## Suggested commit message

`Complete Sprint 20 Persistent Cache evidence`

## Final report additions

Report the public cold/warm/invalidation/corruption/failure/recovery/watch/query
matrix, complete-state equivalence, fixture provenance, lifecycle compatibility,
cleanup/repetition, documentation changes, focused/full validation, changed
paths, commit, and final Git state.
