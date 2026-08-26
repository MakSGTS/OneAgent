# Implement the MCP Tool Protocol

Use the MCP Protocol profile/template. Read the Sprint plan, investigation,
accepted ADR-0051, ADR-0050, official MCP sources, protocol implementation and
all consumers/tests.

Implement only the protocol-owned accepted boundary: truthful `tools`
capability, deterministic `tools/list` and `tools/call`, bounded tool metadata,
schemas, annotations and call/result values, exact protocol-versus-tool error
classification, and the accepted public async sequential handler/dispatch
migration. Keep the protocol crate transport-, Runtime-, graph-, Analysis-, and
Tool Policy-independent. Preserve discovery and framing behavior.

Run non-zero protocol unit/public dispatch tests, consumer/API audit, then the
canonical full workspace gate. Commit: `Implement Sprint 29 MCP tool protocol`.
