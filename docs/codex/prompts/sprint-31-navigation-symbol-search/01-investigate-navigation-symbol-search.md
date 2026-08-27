# Investigate Sprint 31 Navigation and Symbol Search

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
- `docs/adr/0049-tool-execution-policy.md`
- `docs/adr/0050-mcp-server.md`
- `docs/adr/0051-mcp-semantic-tools.md`
- `docs/adr/0052-vscode-extension-foundation.md`
- `docs/reviews/sprint-30-vscode-extension-foundation.md`
- Official stable VS Code 1.134.0 commands, Quick Pick, document-opening,
  text-editor reveal/selection, URI, workspace, testing, and UX documentation,
  with immutable source evidence where version-specific behavior matters.

## Prerequisites / Required gate

- The committed Sprint 31 planning baseline is HEAD.
- Sprint 30 is completed and Sprint 31 is the unique eligible target.
- The accepted desktop VS Code and public `oneagent-mcp` baselines remain
  discoverable and compatible.

## Investigation objective

Create `docs/architecture/navigation-symbol-search-investigation.md` and
update only the Sprint 31 Roadmap state needed to record Task 1 start. Produce
decision-ready evidence for ADR-0053 without production implementation.

## Questions to answer

- Which graph node families are meaningful first-slice symbols, which canonical
  names and kinds are searchable, and what exact matching, case, Unicode,
  ordering, deduplication, ambiguity, empty-query, limit, and truncation rules
  can be proven from current data?
- Which source paths and one-based BSL declaration lines are already retained,
  where are they lost, and which typed source-path/span primitive can preserve
  them without parsing opaque provenance identifiers in a consumer?
- Which EDT and Designer XML node families have reliable file or exact range
  locations, and which must be excluded or returned as non-navigable?
- Should Sprint 31 extend an accepted MCP tool or add bounded read-only tools,
  and how do catalog, schemas, Tool Policy, errors, output bounds, sensitive
  paths, workspace confinement, and protocol compatibility remain truthful?
- Which VS Code commands and Quick Pick flow provide explicit user demand,
  cancellation, repeated invocation, ambiguity handling, URI construction,
  document opening, selection/reveal, and stable failure presentation?
- Which graph, adapter, Runtime, protocol, pure client, Extension Host,
  real-process, cross-platform, package, negative, repeated, and cleanup tests
  provide deterministic non-zero oracles?

## Evidence scope

Inspect common/BSL/graph location types, every producer and consumer of node
provenance, Workspace snapshot roots, Graph Query and MCP projections, Tool
Policy catalog/rules, VS Code client/lifecycle sources, manifests, tests,
tracked EDT/Designer fixtures, current CI/package gates, accepted ADRs, and
pinned official editor sources. Record exact ownership, fields, value
vocabularies, bounds, path normalization and containment behavior, compatibility
impact, dependency impact, rejected candidates, and unresolved choices.

## Excluded

Production code, ADR acceptance, unbounded or fuzzy ranking without a
deterministic contract, filesystem reads after Runtime snapshot construction,
arbitrary provenance disclosure, LSP, definition/reference providers,
diagnostics, chat/context UI, EDT plugin integration, remote/web hosts,
multi-root fan-out, workspace reload/watch changes, automatic connection,
Marketplace work, telemetry, and external-client compatibility.

## Completion Criteria

- Every architecture choice required by Task 2 is decision-ready.
- Real repository-owned positive, negative, ambiguous, missing, incompatible,
  reordered, repeated, and path-confinement evidence exists or the task stops
  with an exact blocker.
- The investigation identifies the smallest coherent first slice and all public
  API/protocol/producer migration impact without inventing source data.
- No production dependency is required, or the task stops for explicit
  approval.

## Task-specific Validation

- Verify every local path and upstream source recorded in the investigation.
- Run focused existing Common, BSL, Graph, Runtime MCP, extension unit,
  Extension Host, and public process baselines selected by the evidence.
- Verify the eight-file Sprint 30 prompt inventory remains unchanged.
- Run `git diff --check`.

## Suggested commit message

`Investigate Sprint 31 navigation and symbol search`

## Final report additions

Report pinned authority, source-location inventory, searchable/navigable
coverage, ownership and compatibility map, dependency impact, exact baseline
test outcomes, remaining unknowns, and ADR readiness.
