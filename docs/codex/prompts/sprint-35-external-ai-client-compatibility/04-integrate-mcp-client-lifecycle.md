# Integrate Sprint 35 MCP Client Lifecycle

Continue OneAgent development.

## Reporting

- Prompt and repository artifacts: English.
- User-visible reports: Russian.

## Profile and template

- `docs/codex/profiles/mcp-protocol-implementation.md`
- `docs/codex/templates/mcp-protocol-task.md`

## Required workflows

- `docs/codex/workflows/mcp-protocol.md`
- `docs/codex/workflows/runtime-service.md`
- `docs/codex/workflows/implementation.md`

## Authoritative documents

- `docs/Roadmap.md`
- `docs/Architecture.md`
- `docs/architecture/external-ai-client-compatibility-investigation.md`
- `docs/adr/0050-mcp-server.md`
- `docs/adr/0051-mcp-semantic-tools.md`
- `docs/adr/0057-external-ai-client-compatibility.md`

## Prerequisite

Task 3 is committed and its complete validation gate passes.

## Task

Compose the accepted protocol session into each production `oneagent-mcp` stdio
connection and prove the complete lifecycle through the public process boundary.

## Required behavior and evidence

- Create exactly one isolated negotiated session for each stdio connection or
  process lifetime, with ownership and mutability fixed by ADR-0057.
- Route initialize, initialized, tools/list, tools/call, accepted notifications,
  shutdown/exit where applicable, malformed messages, EOF, and process
  termination through the accepted state machine without stdout contamination.
- Preserve newline framing, request correlation, bounded input/output/stderr,
  deterministic failures, flush behavior, no arbitrary sleeps, and clean
  release of every owned resource.
- Keep `oneagent-mcp` cwd/workspace semantics, startup graph snapshot, immutable
  seven-tool catalog, Tool Policy decisions, semantic results, and existing
  modern client behavior unchanged.
- Update only consumers, fixtures, and tests required by the accepted lifecycle
  migration. Do not add another transport or change client installations.
- Add public-process evidence for each accepted protocol revision and exact
  Codex/Cursor request shape: initialize, initialized, list, representative
  successful and domain-failing calls, invalid order, duplicate initialize,
  unknown method, malformed input, EOF, shutdown/exit where accepted, repeated
  processes, two-session isolation, stderr cleanliness, exit status, and modern
  regression. Each claimed row must execute through the production binary.

## Excluded scope

Checking in external-client artifacts, global client configuration, additional
clients, new transport, authentication, tool/schema expansion, semantic changes,
unrelated Runtime routes, current-state documentation, and Sprint completion.

## Validation

Run focused Runtime MCP/library and public `oneagent-mcp` process targets with
non-zero matching tests, explicit lifecycle/version/framing/cleanup/repetition
filters, existing VS Code/LSP/EDT MCP compatibility tests affected by the
boundary, then the canonical full Rust workspace gate and `git diff --check`.

## Suggested commit message

`Integrate Sprint 35 MCP client lifecycle`

## Final report additions

Report session ownership, transport changes, exact public-process matrix and
counts, stdout/stderr/EOF/shutdown/cleanup behavior, preserved clients and tool
semantics, dependency/API impact, and full validation outcomes.
