# Implement Sprint 30 MCP Runtime Client

Continue OneAgent development.

## Reporting

- Prompt and repository artifacts: English.
- User-visible reports: Russian.

## Profile

`docs/codex/profiles/ide-extension-implementation.md`

## Template

`docs/codex/templates/ide-extension-task.md`

## Authoritative documents

- `docs/adr/0052-vscode-extension-foundation.md`
- `docs/adr/0050-mcp-server.md`
- `docs/adr/0051-mcp-semantic-tools.md`
- `docs/architecture/vscode-extension-foundation-investigation.md`
- `apps/runtime/src/bin/oneagent-mcp.rs`
- `apps/runtime/tests/mcp_process.rs`

## Prerequisites / Required gate

Task 3 is committed and its package gate passes from the lockfile.

## Task

Implement the accepted editor-independent MCP Runtime client within
`extensions/vscode/`.

## Scope

### Included

Executable and workspace inputs supplied by the caller; bounded child-process
spawn policy; newline JSON-RPC framing; accepted initialization handshake;
monotonic request correlation; sequential calls; protocol/stderr/output bounds;
closed connection states; typed redacted failures; EOF, unexpected exit, and
shutdown behavior; cancellation of pending work; pure unit fakes; and public
real-process evidence against a built `oneagent-mcp`.

### Excluded

VS Code API imports in the transport owner, configuration reads, commands,
status UI, automatic reconnect unless ADR-0052 requires it, concurrent requests,
semantic reimplementation, Runtime/protocol behavior changes, remote transport,
and downloading or installing the Runtime.

## Acceptance Criteria

- The client implements exactly the ADR-0052 protocol/lifecycle slice and
  preserves ADR-0050/0051 framing and channel purity.
- Inputs and retained outputs are bounded; implicit diagnostics reveal no user
  path, arguments, protocol payload, or source value.
- Startup, initialization, request, malformed response, protocol error, stderr
  overflow, EOF, unexpected exit, graceful stop, forced stop if accepted,
  repetition, and zero-orphan cleanup are deterministic.
- Real-process tests prove the public binary boundary without modifying Rust
  behavior.

## Task-specific Validation

- Run non-zero client unit tests and real `oneagent-mcp` process tests.
- Run the complete extension package gate.
- Run focused Runtime MCP process tests and every Rust gate required by the
  actual diff.
- Run `git diff --check`.

## Suggested commit message

`Implement Sprint 30 MCP runtime client`

## Final report additions

Report state transitions, bounds, failure taxonomy, process ownership, public
process evidence, cleanup, and any Rust validation.
