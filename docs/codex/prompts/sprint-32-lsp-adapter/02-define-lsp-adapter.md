# Define Sprint 32 LSP Adapter

Continue OneAgent development.

## Reporting

- Prompt and repository artifacts: English.
- User-visible reports: Russian.

## Profile

`docs/codex/profiles/architecture.md`

## Template

`docs/codex/templates/architecture-task.md`

## Authoritative ADRs / architecture documents

- `docs/architecture/lsp-adapter-investigation.md`
- `docs/architecture/semantic-model-2.md`
- `docs/adr/0039-workspace-service.md`
- `docs/adr/0040-graph-query-api.md`
- `docs/adr/0050-mcp-server.md`
- `docs/adr/0051-mcp-semantic-tools.md`
- `docs/adr/0052-vscode-extension-foundation.md`
- `docs/adr/0053-navigation-symbol-search.md`
- `docs/Roadmap.md`

## Prerequisites / Required gate

Task 1 is committed and its investigation has no blocking unknown or unapproved
production dependency.

## Task

Create and accept `docs/adr/0054-lsp-adapter.md`.

## Scope

### Included

Define the exact LSP revision/source authority; crate/process ownership; JSON-
RPC messages and validation precedence; Content-Length framing; lifecycle and
state; roots, URIs, language/document selectors, position encoding and range
conversion; accepted navigation/symbol/diagnostic methods and capabilities;
deterministic projection, ordering and bounds; diagnostic identity/severity/
location behavior; cancellation, errors, shutdown/exit/EOF and channel purity;
sensitive-data policy; compatibility, dependencies, public process evidence;
implementation prerequisites; rejected alternatives; and deferred scope.

### Excluded

Production code, mutable document synchronization or source parsing, unsupported
language features, dynamic registration, multiple workspace roots, sockets/
pipes, IDE-specific UI, MCP changes, external-client compatibility, edits,
telemetry, and broad performance/security claims.

## Acceptance Criteria

- ADR-0054 records one implementable canonical contract with exact owners,
  lifecycle, wire shapes, capabilities, methods, coordinates, URIs, bounds,
  errors, ordering, diagnostic behavior, compatibility, and evidence gates.
- Advertised behavior is limited to immutable canonical graph/diagnostic facts;
  protocol code owns no semantics and Runtime performs no post-start source read.
- Existing MCP, HTTP, CLI, Graph, Workspace, adapters, and VS Code behavior
  remain compatible unless an explicit additive migration is accepted.
- Every implementation task has a repository-owned executable oracle.

## Task-specific Validation

- Validate links and pinned sources, ADR numbering/index conventions, source and
  consumer inventory, accepted/deferred agreement, and Roadmap consistency.
- Run `git diff --check`.

## Suggested commit message

`Define Sprint 32 LSP adapter`

## Final report additions

Report accepted revision, process/lifecycle, method/capability, root/URI/
position, diagnostic, bounds/error, compatibility, dependency, and
implementation-gate decisions.
