# Implement Impact and Context Tools

Use the MCP Protocol and AI Tool Policy modules. Read the Sprint plan,
investigation, ADR-0051, ADR-0044, ADR-0049, Graph Impact and Analysis Context
APIs/tests, and Task 4 implementation.

Add the accepted impact and context executors to the same immutable catalog.
Impact compares explicitly selected configuration snapshots through canonical
Graph diff/Impact. Context resolves the accepted exact seed and bounded policy
through Context Engine. Both remain deterministic, read-only, bounded,
redacted, and Tool Policy-gated. Add only the approved local
`oneagent-analysis` Runtime dependency. Do not change graph/context semantics or
compose process startup.

Run non-zero impact/context/catalog fixture tests, bounds/dependency/bypass
audits, then the canonical full workspace gate. Commit:
`Implement Sprint 29 impact and context tools`.
