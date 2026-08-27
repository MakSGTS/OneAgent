# Define Sprint 31 Navigation and Symbol Search

Continue OneAgent development.

## Reporting

- Prompt and repository artifacts: English.
- User-visible reports: Russian.

## Profile

`docs/codex/profiles/architecture.md`

## Template

`docs/codex/templates/architecture-task.md`

## Authoritative ADRs / architecture documents

- `docs/architecture/navigation-symbol-search-investigation.md`
- `docs/architecture/semantic-model-2.md`
- `docs/adr/0039-workspace-service.md`
- `docs/adr/0040-graph-query-api.md`
- `docs/adr/0049-tool-execution-policy.md`
- `docs/adr/0050-mcp-server.md`
- `docs/adr/0051-mcp-semantic-tools.md`
- `docs/adr/0052-vscode-extension-foundation.md`
- `docs/Roadmap.md`

## Prerequisites / Required gate

Task 1 is committed and its investigation has no blocking unknown or unapproved
production dependency.

## Task

Create and accept `docs/adr/0053-navigation-symbol-search.md`.

## Scope

### Included

Define canonical source-path/span ownership and coordinate conventions;
optional-location representation, equality, ordering, validation, and
serialization impact; accepted source producers and migration behavior;
searchable and navigable node families; query normalization, matching,
ordering, ambiguity, limits, truncation, and errors; Workspace/path confinement;
MCP tool identity or additive operation, schema, Tool Policy classification,
results, sensitive-data and compatibility rules; VS Code command, Quick Pick,
selection/reveal, cancellation, state, failure, ownership, and test contracts;
and exact implementation prerequisites and deferrals.

### Excluded

Production code, source parsing beyond confirmed location projection, fuzzy or
relevance scoring without deterministic evidence, source contents, references
UI, LSP/provider registration, diagnostics, chat/context UI, mutable workspaces,
remote/web/multi-root, automatic Runtime connection, external clients,
Marketplace work, telemetry, and broad performance/security claims.

## Acceptance Criteria

- ADR-0053 records confirmed evidence, alternatives, canonical decisions,
  rejected alternatives, owners, identities, coordinates, bounds, ordering,
  path confinement, errors, protocol compatibility, UI behavior, tests,
  prerequisites, migration impact, and deferred scope.
- Graph, adapter, Runtime/MCP, Tool Policy, and TypeScript ownership remain
  separated; the extension does not infer semantic facts or decode opaque
  provenance identifiers.
- Existing graph identities and accepted six-tool behavior remain compatible,
  with any additive catalog/schema change explicit and fully testable.
- Every implementation task has a repository-owned executable oracle.

## Task-specific Validation

- Validate all links and pinned sources, ADR numbering/index conventions,
  accepted/deferred consistency, public API/consumer inventory, and Roadmap
  agreement.
- Run `git diff --check`.

## Suggested commit message

`Define Sprint 31 navigation and symbol search`

## Final report additions

Report accepted source-location, symbol-query, MCP, Tool Policy, VS Code UX,
compatibility, path-security, dependency, and implementation-gate decisions.
