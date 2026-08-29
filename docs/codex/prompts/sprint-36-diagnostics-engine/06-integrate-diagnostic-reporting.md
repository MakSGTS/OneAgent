# Integrate Sprint 36 Diagnostic Reporting

Continue OneAgent development.

## Reporting

- Prompt and repository artifacts: English.
- User-visible reports: Russian.

## Profiles and template

- `docs/codex/profiles/diagnostics-engine-implementation.md`
- `docs/codex/profiles/mcp-protocol-implementation.md`
- `docs/codex/profiles/runtime-service-implementation.md`
- `docs/codex/templates/diagnostics-engine-task.md`

## Required workflows

- `docs/codex/workflows/mcp-protocol.md`
- `docs/codex/workflows/runtime-service.md`
- `docs/codex/workflows/implementation.md`
- `docs/codex/workflows/diagnostics-engine.md`

## Authoritative documents

- `docs/Roadmap.md`
- `docs/Architecture.md`
- `docs/architecture/semantic-model-2.md`
- `docs/architecture/diagnostics-engine-investigation.md`
- `docs/adr/0049-tool-execution-policy.md`
- `docs/adr/0050-mcp-server.md`
- `docs/adr/0051-mcp-semantic-tools.md`
- `docs/adr/0054-lsp-adapter.md`
- `docs/adr/0057-external-ai-client-compatibility.md`
- `docs/adr/0058-diagnostics-engine.md`

## Prerequisite

Task 5 is committed and its complete validation gate passes.

## Task

Project the accepted immutable Diagnostics Engine result through the existing
MCP `oneagent.diagnostics` tool and LSP pull-document diagnostic capability.
Implement only ADR-0058-approved schema, result, filtering, suppression,
location, ordering, and bound migrations.

## Required behavior and evidence

- Preserve the exact seven-tool MCP catalog order, read-only annotations,
  protocol revision compatibility, Tool Policy authorization/execution, request
  isolation, immutable startup snapshot, and existing non-diagnostic tools.
- Make the `oneagent.diagnostics` input schema, validation, projection,
  summaries, active/suppressed visibility, ordering, bounds, errors, and modern/
  legacy response shapes exactly match ADR-0058. Advertise no unsupported field
  and leak no root, source content, opaque provenance, or rejected input.
- Preserve LSP 3.17 lifecycle, capability truth, URI confinement, position
  encoding, full-report contract, complete-result bound, and protocol-channel
  purity. Project only ADR-approved active results with one exact confined
  location; handle suppressed, unlocated, invalid, and over-bound evidence
  explicitly as accepted.
- Keep domain normalization out of protocol handlers. MCP and LSP consume one
  immutable engine result and do not rerun validators, inspect source files, or
  infer locations.
- Add non-zero in-memory and public-process tests for empty, positive,
  mixed-family, summary, suppression, filter/argument validation, exact/over
  bound, deterministic repetition, missing/invalid location, confinement,
  Tool Policy denial, modern and legacy MCP compatibility, LSP full/empty
  reports, malformed requests, lifecycle, EOF, channel purity, exit, and cleanup.
- Preserve VS Code, EDT, Codex/Cursor, HTTP, CLI, graph, adapter, cache, and all
  other MCP/LSP behavior. Do not claim diagnostics UI or mutable-document
  support.

## Excluded scope

New MCP tools or LSP capabilities, transport/version expansion,
authentication, remote clients, VS Code/EDT UI, push/workspace diagnostics,
mutable documents, Rules Engine, source mutation, external-client reruns not
required by a changed compatibility contract, current-state docs, and Sprint
completion.

## Validation

Run focused protocol, Runtime MCP semantic-tool, MCP stdio/process, LSP
protocol/stdio/process, Tool Policy, Workspace, and compatibility tests with
non-zero matching filters. Audit catalogs, schemas, handlers, policies,
capabilities, locations, bounds, errors, and sensitive-data absence. Then run
the canonical full Rust workspace gate and `git diff --check`.

## Suggested commit message

`Integrate Sprint 36 diagnostic reporting`

## Final report additions

Report MCP schema/result and LSP projection behavior, catalog/capability/Tool
Policy preservation, public-process matrices and counts, bounds/confinement/
cleanup, compatibility, API/dependency impact, and full validation outcomes.
