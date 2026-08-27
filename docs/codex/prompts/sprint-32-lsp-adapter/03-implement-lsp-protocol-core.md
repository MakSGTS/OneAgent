# Implement Sprint 32 LSP Protocol Core

Continue OneAgent development.

## Reporting

- Prompt and repository artifacts: English.
- User-visible reports: Russian.

## Profile

`docs/codex/profiles/implementation.md`

## Template

`docs/codex/templates/implementation-task.md`

## Authoritative documents

- `docs/adr/0054-lsp-adapter.md`
- `docs/architecture/lsp-adapter-investigation.md`
- `docs/adr/0050-mcp-server.md`

## Prerequisites / Required gate

Task 2 and accepted ADR-0054 are committed. The protocol revision, lifecycle,
messages, capabilities, validation precedence, framing boundary, bounds, and
errors are fixed.

## Task

Implement the accepted transport-independent LSP protocol core.

## Scope

### Included

Accepted request/notification/response identifiers and envelopes; duplicate-
key and depth/size validation; lifecycle state machine; initialize capability
validation and truthful server capabilities; registered accepted method
dispatch; stable errors; cancellation behavior if accepted; deterministic
encoding; protocol-only fixtures; public APIs and exhaustive unit/integration
tests; and exact compatibility with existing MCP modules.

### Excluded

I/O framing, Tokio/process ownership, Workspace construction, semantic symbol
or diagnostic projection, source reads, VS Code code, MCP behavior changes,
unsupported LSP methods, external clients, and new dependencies unless approved.

## Acceptance Criteria

- Public protocol values and dispatch exactly implement ADR-0054 and cannot
  advertise an unregistered capability.
- Validation/error/lifecycle precedence is deterministic for positive,
  malformed, missing, duplicate, unknown, reordered, repeated, pre-initialize,
  shutdown, and exit cases.
- Existing MCP protocol behavior and tests remain unchanged.
- Focused protocol tests execute non-zero cases and the full Rust gate passes.

## Task-specific Validation

- Run non-zero focused `oneagent-protocol` LSP and MCP regression tests.
- Audit capability/handler/error agreement.
- Run the canonical Rust workspace gate and `git diff --check`.

## Suggested commit message

`Implement Sprint 32 LSP protocol core`

## Final report additions

Report public types/APIs, lifecycle and validation behavior, capabilities,
errors/bounds, focused test counts, dependency impact, and preserved MCP behavior.
