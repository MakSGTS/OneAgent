# Implement Sprint 33 Context Runtime Client

Continue OneAgent development.

## Reporting

- Prompt and repository artifacts: English.
- User-visible reports: Russian.

## Profile

`docs/codex/profiles/ide-extension-implementation.md`

## Template

`docs/codex/templates/ide-extension-task.md`

## Authoritative documents

- `docs/adr/0055-ai-chat-context-panel.md`
- `docs/architecture/ai-chat-context-panel-investigation.md`
- `docs/adr/0044-context-engine.md`
- `docs/adr/0050-mcp-server.md`
- `docs/adr/0051-mcp-semantic-tools.md`
- `docs/adr/0053-navigation-symbol-search.md`

## Prerequisites / Required gate

Task 2 and accepted ADR-0055 are committed. The Context wire projection,
bounds, validation, concurrency, failure, and compatibility contracts are fixed.

## Task

Implement the strict TypeScript Context request/result domain and serialized
Runtime client operation over the existing `oneagent.context` tool.

## Scope

### Included

Accepted request types and bounds; exact closed result decoder; node, relation,
rendered-text, accounting, truncation, configuration, and UTF-8 validation;
fail-closed unknown/missing/malformed/over-bound behavior; serialization with
the one-pending-request transport; disconnect/abort cleanup; public exported
client API; unit fixtures; and real `oneagent-mcp` Context process evidence.

### Excluded

Rust or MCP catalog changes, semantic selection UI, webview, Chat API, model
requests, source reads, new dependencies, and documentation completion.

## Acceptance Criteria

- Only ADR-0055-conformant Context values reach extension consumers.
- Context and Symbols operations cannot collide with the single request slot;
  failures and disconnects settle queued/pending work deterministically.
- Exact, boundary, malformed, missing, reordered, repeated, tool-error,
  timeout, exit, and compatibility cases have non-zero evidence.
- Existing connection and symbol behavior remains compatible.

## Task-specific Validation

- Run non-zero focused TypeScript MCP-client/Context unit tests.
- Run non-zero real Runtime Context process and existing symbol-process tests.
- Run extension typecheck and compile; audit unchanged Runtime tool catalog.
- Run `git diff --check`.

## Suggested commit message

`Implement Sprint 33 context Runtime client`

## Final report additions

Report public types, decoder rules, bounds, serialization/cleanup behavior,
process evidence, compatibility, and exact test counts.
