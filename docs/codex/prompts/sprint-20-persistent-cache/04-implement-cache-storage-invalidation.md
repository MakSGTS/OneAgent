# Implement Sprint 20 Cache Storage and Invalidation

Continue OneAgent development.

## Reporting

- Prompt and repository content: English.
- User-visible reports: Russian.

## Profile

`docs/codex/profiles/persistent-state-implementation.md`

## Template

`docs/codex/templates/persistent-state-task.md`

## Authoritative documents

- `docs/Roadmap.md`, Sprint 20 execution plan
- `docs/architecture/persistent-cache-investigation.md`
- `docs/Architecture.md`
- `docs/architecture/semantic-model-2.md`
- `docs/adr/0039-workspace-service.md`
- `docs/adr/0041-file-watching.md`
- `docs/adr/0042-persistent-cache.md`

## Prerequisites / Required gate

Require committed Task 3 with the accepted complete codec, successful focused
and complete validation, and clean task-owned state. Stop rather than changing
accepted identity, invalidation, location, replacement, compatibility,
corruption, or recovery semantics during storage implementation.

## Task

Implement the accepted deterministic cache validity identity and filesystem
store with contained paths, complete safe replacement, typed load/write
outcomes, cleanup, corruption containment, and focused failure/recovery tests.
Do not integrate Runtime startup or snapshot publication in this task.

## Identity and deterministic invalidation

- Derive exact cache identity from every ADR-0042 source-state,
  semantic-contract, schema/build-version, Workspace/configuration, and option
  input.
- Preserve deterministic path/content ordering and invalidate rather than guess
  when evidence is incomplete, ambiguous, mismatched, or unverifiable.
- Keep ignored cache storage from changing the semantic source identity or
  feeding the existing File Watching loop exactly as accepted.

## Filesystem, replacement, and recovery

- Contain every directory, candidate, temporary, and replacement path below the
  accepted cache owner; apply exact permission/symlink/existing-kind behavior.
- Read only exact candidates, decode through Task 3, and classify missing,
  incompatible, corrupt, unreadable, and invalid state without publication.
- Write only complete validated bytes through the accepted replacement and
  cleanup sequence; prove interruption/failure cannot expose a partial current
  entry.

## Scope

### Included

- Validity/fingerprint implementation, cache path derivation, storage reader and
  writer, complete replacement, stale/incompatible/corrupt handling, temporary
  cleanup, typed outcomes, deterministic failure seams, and focused tests.
- Exact source add/modify/remove/rename/reorder/content changes, semantic/schema
  version changes, hit/miss, duplicate candidates, path containment,
  wrong-kind/symlink/permission cases where portable evidence exists, truncated
  and malformed entries, failed/interrupted writes, recovery, and repetition.

### Excluded

Runtime service/configuration/publication integration, watcher scheduling,
Graph Query or health changes, HTTP/CLI cache APIs, production fixture/current-
state docs, graph/parser/adapter semantics, incremental persistence,
cross-process locking, remote cache, compression, encryption, eviction,
benchmarks, and prompt/Roadmap changes.

## Acceptance Criteria

- Cache identity covers every accepted validity input and equal accepted inputs
  produce equal candidates independent of enumeration order, wall clock,
  process, or platform-specific absolute path behavior.
- Every source/semantic/schema change accepted by ADR-0042 causes a deterministic
  miss or rejection; cache-owned files cannot trigger self-rebuild loops or
  contaminate validity.
- Candidate paths remain contained, reads return only completely decoded and
  validated snapshots, and every rejected state has the exact typed outcome.
- Complete replacement and cleanup prevent partial-current publication and have
  deterministic write-failure/interruption/recovery/repetition evidence.
- No Runtime publication, lifecycle/health/query wire change, cache management
  API, or deferred capability is introduced.

## Repository Safety

Modify only cache identity/storage implementation, module wiring, and focused
test paths required by ADR-0042. Preserve `.codex/`, prompts, Roadmap, ADRs,
manifests/lockfile unless explicitly approved and already required by Task 3,
tracked fixtures, current-state docs, Runtime orchestration, graph/adapter
semantics, HTTP schemas, and unrelated files. Stage only enumerated task-owned
paths.

## Task-specific Validation

- List and run exact non-zero focused identity, invalidation, containment,
  hit/miss, corruption/incompatibility, replacement, failure, cleanup, recovery,
  and repetition tests.
- Run affected Runtime package tests.
- Run the canonical complete workspace validation from
  `docs/codex/core/validation.md`.
- `git status --short`

## Suggested commit message

`Implement Sprint 20 cache storage and invalidation`

## Final report additions

Report validity inputs and identity, path containment, storage/replacement,
typed outcomes, corruption/failure/recovery, watcher exclusion, focused/full
validation, changed paths, commit, and final Git state.
