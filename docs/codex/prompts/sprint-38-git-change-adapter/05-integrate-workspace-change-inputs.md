# Integrate Sprint 38 Workspace Change Inputs

Continue OneAgent development.

## Reporting

- Prompt and repository artifacts: English.
- User-visible reports: Russian.

## Profiles and template

- `docs/codex/profiles/git-change-adapter-implementation.md`
- `docs/codex/profiles/runtime-service-implementation.md`
- `docs/codex/templates/git-change-adapter-task.md`

## Required workflows

- `docs/codex/workflows/implementation.md`
- `docs/codex/workflows/git-change-adapter.md`
- `docs/codex/workflows/runtime-service.md`

## Authoritative documents

- `docs/Roadmap.md`
- `docs/Architecture.md`
- `docs/architecture/semantic-model-2.md`
- `docs/architecture/git-change-adapter-investigation.md`
- `docs/adr/0027-incremental-semantic-index-maintenance.md`
- `docs/adr/0037-runtime-service-container.md`
- `docs/adr/0039-workspace-service.md`
- `docs/adr/0041-file-watching.md`
- `docs/adr/0042-persistent-cache.md`
- `docs/adr/0060-git-change-adapter.md`

## Prerequisite

Task 4 is committed, its complete validation passes, and the accepted Git
reader returns complete deterministic normalized change evidence.

## Task

Map the accepted Git-derived change set into the ADR-0060 source-independent
Workspace change-input boundary and integrate only the accepted complete
rebuild lifecycle. Add no semantic impact or selective graph mutation.

## Required behavior and evidence

- Introduce or use exactly one accepted source-independent Workspace
  change-input contract. Keep Git endpoint/status details out of canonical
  Workspace, Graph, Analysis, and public APIs unless ADR-0060 explicitly owns
  a bounded additive field there.
- Prove that accepted Git-derived relevant changes and equivalent complete
  filesystem end states drive equivalent discovery, adapter builds,
  validation, immutable snapshot results, update status, and supported
  consumer observations.
- Preserve ADR-0041 startup race closure, bounded latest-state coalescing,
  one complete serialized build, follow-up behavior, atomic publication,
  last-valid failure retention, recovery, and equal-result policy.
- Preserve ADR-0042 source-state validation, cache hit/miss/invalidation,
  stable write order, corruption/write-failure recovery, watched replacement,
  and fresh-process equivalence unless ADR-0060 explicitly accepts one exact
  migration.
- Keep every source task, reader/process, channel, blocking build, cancellation
  handle, timeout, and cleanup under the accepted owner. Prove startup failure,
  recoverable failure, cancellation during observation/read/build, shutdown,
  receiver closure, resource release, and repeated fresh runs deterministically.
- Add production-entry-point evidence over disposable copies of tracked EDT and
  Designer fixtures inside temporary Git repositories. Cover accepted relevant
  modifications/additions/removals/rename-equivalent changes, empty or
  irrelevant changes, invalid build/recovery, burst/follow-up, old/new reader
  atomicity, cache reuse, and exact public consumer compatibility.
- Preserve Graph Diff and ADR-0027 only after complete graph construction;
  preserve diagnostics/rules, HTTP/CLI/MCP/LSP/VS Code/EDT behavior, source
  confinement, adapters, and Coverage. Advertise no Git control or impact tool.

## Excluded scope

Incremental Graph/index mutation, changed-entity inference, semantic impact,
diagnostic or rule creation from Git status, selective parsing/building,
partial snapshots, remote Git, repository mutation, new protocol tool or IDE
UI, refactoring, source edits, current-state documentation, and Sprint
completion.

## Validation

Run non-zero focused normalized-input/Workspace-mapping/equivalence/coalescing/
rebuild/publication/failure/recovery/cache/lifecycle/cancellation/shutdown/
repetition tests; affected production Workspace, watching, cache, Graph Query,
HTTP/CLI/MCP/LSP and adapter/public-process compatibility tests; then the
canonical full Rust workspace gate and `git diff --check`.

## Suggested commit message

`Integrate Sprint 38 Workspace change inputs`

## Final report additions

Report Workspace input owner and mapping, Git/filesystem equivalence,
rebuild/coalescing/publication behavior, cache and lifecycle compatibility,
cancellation/cleanup, consumer results, exact focused/public counts,
preserved semantic authority, API/dependency impact, and full validation.
