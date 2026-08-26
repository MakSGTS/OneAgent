# Define MCP Semantic Tools

Use the architecture profile/template. Read the Sprint 29 plan, Task 1
investigation, ADR-0040, ADR-0044, ADR-0049, ADR-0050, semantic architecture,
and exact official MCP sources selected by the investigation.

Create only `docs/adr/0051-mcp-semantic-tools.md` with status `Accepted`.
Decide the exact six-tool catalog and names, ownership/dependencies, public
async sequential handler migration, discovery/list/call wire contracts, input
schemas and manual validation, annotations, Tool Policy gate and audit,
workspace snapshot lifecycle, per-tool arguments/projections/bounds, content
and structured results, protocol/tool/startup failures, compatibility,
conformance, and deferred scope. Preserve existing HTTP/Workspace/CLI and graph
semantics. Authorize no dependency beyond the two planned local Runtime edges.

Validation: decision completeness, links, consumer/migration audit, dependency
and scope consistency, `git diff --check`. Commit:
`Define Sprint 29 MCP semantic tools`.
