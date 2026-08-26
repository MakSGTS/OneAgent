# Investigate MCP Semantic Tools

Use the investigation profile/template and read the Sprint 29 Roadmap plan,
ADR-0040, ADR-0044, ADR-0049, ADR-0050, current MCP/Runtime/Graph/Analysis/Tool
Policy sources and tests, Sprint 28 review, and official MCP `2026-07-28` tools
specification and schema.

Create only `docs/architecture/mcp-semantic-tools-investigation.md`.

Record confirmed repository and normative facts for: exact `tools` capability,
`tools/list`, `tools/call`, tool metadata/schema/annotations, protocol versus
tool errors, content and structured content; owners and dependency direction;
the six-tool catalog; async sequential dispatch; immutable workspace startup;
Tool Policy request/evaluation/execution; bounds, ordering, redaction and
failures; compatibility; deterministic unit/fixture/process oracles; and
explicit deferrals. Separate facts, candidates, decisions, unknowns, and
unsupported assumptions. Do not change Rust, Cargo, ADRs, current-state docs,
Roadmap, prompts, or review artifacts.

Validation: repeat source/consumer/dependency searches, reconcile every schema
field with official sources, audit links and questions, and run
`git diff --check`. Commit: `Investigate Sprint 29 MCP semantic tools`.
