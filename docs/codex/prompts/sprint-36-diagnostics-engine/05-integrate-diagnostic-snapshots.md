# Integrate Sprint 36 Diagnostic Snapshots

Continue OneAgent development.

## Reporting

- Prompt and repository artifacts: English.
- User-visible reports: Russian.

## Profile and template

- `docs/codex/profiles/runtime-service-implementation.md`
- `docs/codex/templates/runtime-service-task.md`

## Required workflows

- `docs/codex/workflows/implementation.md`
- `docs/codex/workflows/runtime-service.md`

## Authoritative documents

- `docs/Roadmap.md`
- `docs/Architecture.md`
- `docs/architecture/semantic-model-2.md`
- `docs/architecture/diagnostics-engine-investigation.md`
- `docs/adr/0039-workspace-service.md`
- `docs/adr/0042-persistent-workspace-cache.md`
- `docs/adr/0058-diagnostics-engine.md`

## Prerequisite

Task 4 is committed and its complete validation gate passes.

## Task

Compose the accepted Diagnostics Engine result into immutable Workspace
Configuration snapshots and prove rebuild, cache, observation, and lifecycle
compatibility. Do not change MCP or LSP projections yet.

## Runtime and ownership requirements

- Construct the engine result at the exact ADR-0058-selected immutable
  composition boundary from the canonical graph, recoverable diagnostics,
  validation evidence, and accepted configuration context.
- Publish raw producer evidence and the normalized report exactly as accepted.
  Do not introduce lazy post-publication source reads, hidden global state, or
  a second graph/diagnostic authority.
- Preserve atomic Workspace startup and replacement: a failed diagnostic build
  follows the accepted build failure policy and cannot publish a partially
  mismatched graph/report snapshot.
- Preserve watcher generation ownership, cancellation, shutdown, observation,
  health/readiness, and repeated startup behavior. Add no detached task,
  listener, channel, lock, or mutable singleton.
- Implement cache serialization, recomputation, schema migration, or explicit
  non-persistence exactly as ADR-0058 requires. Prove cold/warm equality,
  corruption/write-failure recovery, invalidation, watched replacement, and
  fresh-process reuse where affected.
- Add focused tests for EDT, Designer, empty, mixed-family, suppressed,
  exact/over-bound, reordered, repeated-build, cache round-trip or recompute,
  rebuild, observation, failure atomicity, and cleanup cases.
- Preserve graph, raw diagnostics, reference requests/statistics, reports,
  queries, validation, cache, HTTP/CLI, and existing Runtime semantics outside
  the accepted additive or migrated snapshot fields.

## Excluded scope

MCP/LSP schema or handler changes, new protocols, new diagnostic producers,
Rules Engine, diagnostics UI, mutable documents, new background services,
unapproved Cargo dependencies, current-state documentation, and Sprint
completion.

## Validation

Run non-zero focused Workspace snapshot, service, cache, file-watching,
observation, rebuild, failure, shutdown, and repeated-process tests; existing
Graph/adapter/HTTP/CLI regressions affected by the public snapshot API; then the
canonical full Rust workspace gate and `git diff --check`.

## Suggested commit message

`Integrate Sprint 36 diagnostic snapshots`

## Final report additions

Report composition and state ownership, cache/rebuild policy, atomicity,
lifecycle and cleanup, exact focused tests/counts, public API migration,
preserved behavior, dependency impact, and full validation outcomes.
