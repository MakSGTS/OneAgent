# Implement Semantic Graph Tools

Use the MCP Protocol and Runtime Service modules. Read the Sprint plan,
investigation, ADR-0051, ADR-0040, ADR-0049, Runtime workspace/query, Graph
validation, diagnostic, Tool Policy APIs, and relevant public tests.

Implement the accepted graph, query, validation, and diagnostics tools over an
immutable `WorkspaceSnapshot`. Every known call must be represented as a
bounded read-only Tool Policy request and executed only through its gate.
Preserve canonical graph/query vocabularies, stable ordering, result bounds,
closed errors, and path/source redaction. Add only the approved local
`oneagent-tool-policy` Runtime dependency. Do not compose the public process or
implement impact/context.

Run non-zero focused Runtime/Tool Policy/fixture tests, dependency and bypass
audits, then the canonical full workspace gate. Commit:
`Implement Sprint 29 semantic graph tools`.
