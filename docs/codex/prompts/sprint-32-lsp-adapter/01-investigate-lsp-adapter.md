# Investigate Sprint 32 LSP Adapter

Continue OneAgent development.

## Reporting

- Prompt and repository artifacts: English.
- User-visible reports: Russian.

## Profile

`docs/codex/profiles/investigation.md`

## Template

`docs/codex/templates/investigation-task.md`

## Authoritative documents

- `docs/Roadmap.md`
- `docs/Architecture.md`
- `docs/architecture/semantic-model-2.md`
- `docs/adr/0039-workspace-service.md`
- `docs/adr/0040-graph-query-api.md`
- `docs/adr/0050-mcp-server.md`
- `docs/adr/0051-mcp-semantic-tools.md`
- `docs/adr/0052-vscode-extension-foundation.md`
- `docs/adr/0053-navigation-symbol-search.md`
- `docs/reviews/sprint-31-navigation-symbol-search.md`
- Official LSP 3.17 specification and immutable meta-model/specification sources.

## Prerequisites / Required gate

- The committed Sprint 32 planning baseline is HEAD.
- Sprint 31 is completed and Sprint 32 is the unique eligible target.
- Typed graph locations, Workspace roots, recoverable diagnostics, and public
  process fixtures remain discoverable.

## Investigation objective

Create `docs/architecture/lsp-adapter-investigation.md` and update only the
Sprint 32 Roadmap state needed to record Task 1 start. Produce decision-ready
evidence for ADR-0054 without production implementation.

## Questions to answer

- Which immutable LSP 3.17 sources govern framing, JSON-RPC, initialize/
  initialized/shutdown/exit, capabilities, URIs, position encoding, symbols,
  navigation, diagnostics, cancellation, errors, and resource bounds?
- Which exact methods can truthfully project current graph locations and
  recoverable diagnostics without reading mutable documents or decoding opaque
  provenance, and which advertised capabilities must remain absent?
- How do Workspace cwd/root URI, Configuration roots, file URIs, lexical
  confinement, UTF-8/UTF-16 positions, client capabilities, and initialization
  compatibility map to the immutable snapshot?
- Which protocol, Runtime, graph, adapter, raw-process, editor-client, malformed,
  missing, ambiguous, incompatible, reordered, repeated, EOF, shutdown, and
  cleanup cases provide deterministic repository-owned oracles?
- Can existing serde/serde_json/Tokio and framework contracts implement the
  slice without a new production dependency?

## Evidence scope

Inspect protocol and Runtime ownership, all source-location and diagnostic
definitions/producers/consumers, Workspace snapshot/cache roots, MCP semantic
projections, extension compatibility assumptions, Cargo/CI/package boundaries,
tracked mixed and negative fixtures, existing tests, accepted ADRs, and pinned
official LSP sources. Record exact fields, vocabularies, bounds, ordering,
identity, lifecycle, compatibility, redaction, dependency, and test-oracle
evidence.

## Excluded

Production code, ADR acceptance, mutable document synchronization, source-text
parsing after startup, completion/hover/references/rename/code actions, edits,
dynamic registration, remote transports, multiple workspaces, IDE-specific UI,
MCP behavior changes, external-client claims, telemetry, and broad performance/
security claims.

## Completion Criteria

- Every architecture choice required by Task 2 is decision-ready.
- The smallest truthful method/capability slice has positive and negative
  executable oracles and explicit unsupported behavior.
- Lifecycle, framing, roots/URIs, coordinates, diagnostics, bounds, errors,
  compatibility, and dependency impact are fully inventoried.
- No new production dependency is required, or execution stops for approval.

## Task-specific Validation

- Verify every local path and pinned official source recorded.
- Run focused existing protocol, Runtime Workspace/MCP-process, Graph,
  adapter-location, diagnostic, and extension compatibility baselines selected
  by the evidence.
- Verify the exact eight-file Sprint 31 prompt inventory.
- Run `git diff --check`.

## Suggested commit message

`Investigate Sprint 32 LSP adapter`

## Final report additions

Report protocol authority, method candidates, ownership/compatibility map,
location/diagnostic coverage, dependency impact, exact baseline checks,
unresolved unknowns, and ADR readiness.
