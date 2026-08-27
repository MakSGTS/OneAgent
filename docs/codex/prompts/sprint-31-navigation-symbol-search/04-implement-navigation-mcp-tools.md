# Implement Sprint 31 Navigation MCP Tools

Continue OneAgent development.

## Reporting

- Prompt and repository artifacts: English.
- User-visible reports: Russian.

## Profile

`docs/codex/profiles/mcp-protocol-implementation.md`

## Template

`docs/codex/templates/mcp-protocol-task.md`

## Authoritative documents

- `docs/adr/0053-navigation-symbol-search.md`
- `docs/architecture/navigation-symbol-search-investigation.md`
- `docs/adr/0039-workspace-service.md`
- `docs/adr/0040-graph-query-api.md`
- `docs/adr/0049-tool-execution-policy.md`
- `docs/adr/0050-mcp-server.md`
- `docs/adr/0051-mcp-semantic-tools.md`
- `docs/adr/0052-vscode-extension-foundation.md`

## Prerequisites / Required gate

Task 3 is committed and its full Rust validation passes. The accepted graph
location model and producer slice are available through public APIs.

## Task

Implement the accepted bounded Runtime/MCP symbol-search and source-navigation
projection.

## Protocol revision and compatibility boundary

Preserve MCP revision `2026-07-28`, ADR-0050 framing and request validation,
the accepted existing tool behavior, and the immutable startup snapshot. Apply
only the additive catalog/schema behavior explicitly accepted by ADR-0053.

## Scope

### Included

Transport-neutral Workspace symbol query if accepted; deterministic symbol
selection, matching, kind filtering, ordering, ambiguity, limits, truncation,
and navigable-location projection; workspace-relative path confinement and
coordinate conversion; accepted read-only MCP tool definitions/dispatch;
Tool Policy registration and canonical request/result bounds; stable errors and
redaction; in-memory protocol, Runtime semantic, public `oneagent-mcp`
list/call, malformed/missing/unknown/duplicate/incompatible/path-escape,
reordered/repeated, result-bound, EOF, and channel-purity evidence.

### Excluded

Source content, arbitrary filesystem/provenance disclosure, graph mutation,
workspace reload/watch changes, concurrent MCP, transport/session changes,
TypeScript/VS Code UI, LSP, diagnostics, references, external-client claims,
remote/authentication, and new dependencies unless separately approved.

## Acceptance Criteria

- Tool discovery, schemas, Tool Policy, handlers, public process behavior, and
  documentation agree exactly and advertise only executable behavior.
- Results are deterministic, bounded, workspace-confined, source-derived, and
  sufficient for the accepted VS Code consumer without duplicating semantics.
- Existing six-tool inputs/results and Runtime/HTTP/CLI behavior remain
  compatible; unsupported and non-navigable symbols follow ADR-0053.
- Public non-zero tests prove positive and negative wire behavior, repetition,
  channel purity, EOF cleanup, and no sensitive absolute-path leakage.

## Task-specific Validation

- Run non-zero focused Graph/Workspace, Tool Policy, protocol dispatch,
  Runtime semantic-tool, MCP stdio, and public `oneagent-mcp` process tests.
- Audit catalog/schema/handler/policy agreement and existing-tool regression.
- Run the canonical full Rust workspace gate and `git diff --check`.

## Suggested commit message

`Implement Sprint 31 navigation MCP tools`

## Final report additions

Report tool/schema compatibility, matching and ordering, location/path
projection, bounds/errors/redaction, Tool Policy behavior, public process test
counts, existing-tool preservation, and deferred protocol scope.
