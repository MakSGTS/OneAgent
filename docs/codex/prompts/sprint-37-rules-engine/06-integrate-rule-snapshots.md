# Integrate Sprint 37 Rule Snapshots

Continue OneAgent development.

## Reporting

- Prompt and repository artifacts: English.
- User-visible reports: Russian.

## Profiles and template

- `docs/codex/profiles/rules-engine-implementation.md`
- `docs/codex/profiles/runtime-service-implementation.md`
- `docs/codex/templates/rules-engine-task.md`

## Required workflows

- `docs/codex/workflows/implementation.md`
- `docs/codex/workflows/rules-engine.md`
- `docs/codex/workflows/diagnostics-engine.md`
- `docs/codex/workflows/runtime-service.md`
- `docs/codex/workflows/persistent-state.md`

## Authoritative documents

- `docs/Roadmap.md`
- `docs/Architecture.md`
- `docs/architecture/semantic-model-2.md`
- `docs/architecture/rules-engine-investigation.md`
- `docs/adr/0039-workspace-service.md`
- `docs/adr/0042-persistent-cache.md`
- `docs/adr/0051-mcp-semantic-tools.md`
- `docs/adr/0054-lsp-adapter.md`
- `docs/adr/0058-diagnostics-engine.md`
- `docs/adr/0059-rules-engine.md`

## Prerequisite

Task 5 is committed, its complete validation passes, and the accepted engine
execution and diagnostic integration are complete.

## Task

Compose the accepted Rules Engine result into immutable Workspace Configuration
snapshots and prove rebuild, cache, watching, observation, reporting, lifecycle,
and consumer compatibility. Add no new protocol or IDE capability unless
ADR-0059 explicitly requires a bounded migration.

## Required behavior and evidence

- Construct registry, plan, execution result, and derived diagnostic report at
  the exact ADR-0059 immutable composition boundary from canonical evidence and
  accepted configuration.
- Publish a complete internally consistent snapshot only after all accepted
  validation. A registry, plan, execution, diagnostic, or bound failure cannot
  publish a partially mismatched graph/report/result snapshot.
- Preserve startup, watcher generation, replacement, observation, cancellation,
  shutdown, readiness, failure, and cleanup ownership with no detached task,
  mutable singleton, hidden source read, or timing-dependent outcome.
- Implement cache serialization, deterministic recomputation, compatibility
  invalidation, or explicit non-persistence exactly as ADR-0059 accepts. Prove
  cold/warm equality, corruption/write-failure recovery, watched rebuild,
  semantic/configuration/registry invalidation, and fresh-process reuse where
  applicable.
- Preserve ADR-0058 diagnostic identity, suppression, ordering, summary, and
  complete-report semantics when rule-produced evidence is composed with
  existing Graph evidence.
- Keep MCP and LSP domain normalization out of handlers. If accepted rule
  diagnostics flow through existing projections, prove exact seven-tool
  catalog, Tool Policy, schema/capability truth, revision parity, bounds,
  confinement, sensitive-data, lifecycle, and public-process compatibility.
  Advertise no rule-management surface unless ADR-0059 explicitly accepts it.
- Add focused tests for empty/default and configured rules, successful and
  failing execution, cancellation, exact/over bounds, reordered/repeated
  builds, cold/warm cache, corruption/recovery, invalidation, watcher
  replacement, atomic failure, observation, MCP/LSP projection compatibility,
  shutdown, and cleanup.
- Preserve raw diagnostics, validation, graph reports/diffs/queries, HTTP/CLI/
  VS Code/EDT behavior, adapters, and Coverage outside accepted additive
  snapshot fields or exact migrations.

## Excluded scope

New MCP tools, rule-management protocol, LSP capability, IDE UI, external
configuration grammar, plugin loading, remote rules, mutable documents,
automatic fixes, source edits, new background service, unapproved dependency,
current-state documentation, and Sprint completion.

## Validation

Run non-zero focused Workspace snapshot/service/cache/watching/rebuild/
observation/failure/cancellation/shutdown/repetition tests; affected Graph,
Diagnostics Engine, Runtime, MCP/LSP, HTTP/CLI, adapter, and public-process
compatibility tests; then the canonical full Rust workspace gate and
`git diff --check`.

## Suggested commit message

`Integrate Sprint 37 rule snapshots`

## Final report additions

Report composition and state ownership, registry/configuration inputs,
snapshot atomicity, cache/rebuild/invalidation policy, lifecycle and cleanup,
diagnostic and protocol compatibility, exact focused/public test counts,
preserved behavior, API/dependency impact, and full validation outcomes.
