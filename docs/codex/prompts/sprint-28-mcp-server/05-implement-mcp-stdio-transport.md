# Implement the MCP stdio Transport

Continue OneAgent development.

## Reporting

- Prompt and repository changes: English.
- User-visible reports: Russian.

## Profile

`docs/codex/profiles/mcp-protocol-implementation.md`

## Template

`docs/codex/templates/mcp-protocol-task.md`

## Authoritative MCP specification, schema, ADRs, and architecture documents

- `docs/Roadmap.md`, Sprint 28 execution plan
- `docs/adr/0050-mcp-server.md`
- `docs/architecture/mcp-server-investigation.md`
- `docs/adr/0037-runtime-service-container.md`
- the exact versioned official stdio specification accepted by ADR-0050

## Prerequisites / Required gate

Task 4 is committed, its complete protocol tests and full workspace validation
succeeded, and the working tree has no conflicting task-created change.

## Task

Implement only the accepted newline-framed stdio-compatible stream adapter over
injected asynchronous input/output and the transport-independent dispatcher.

## Scope

### Included

- Bounded UTF-8 line reads, one-message-per-line framing, embedded-newline and
  malformed-frame handling, deterministic response writes and flush policy.
- Protocol-only output, caller-owned diagnostic separation, input ordering or
  accepted concurrency, EOF/disconnect, cancellation, read/write failure,
  cleanup, and repeated fresh execution.
- In-memory transport tests through injected streams.

### Excluded

Ownership of real process stdin/stdout/stderr, Runtime registration, executable
composition, semantic tools, HTTP/SSE/custom transports, external clients,
auth, logging policy beyond channel separation, current-state docs, or Sprint
completion.

## Acceptance Criteria

- Every accepted frame contains exactly one valid JSON-RPC message and every
  emitted response is exactly one compact JSON value plus one newline.
- Oversized, invalid UTF-8, empty/malformed, and embedded-newline inputs follow
  ADR-0050 failure/error rules without partial dispatch or unbounded buffering.
- Notifications produce no output; request response order/concurrency matches
  the accepted contract.
- No log, banner, diagnostic, source error, or unrelated byte reaches protocol
  output.
- EOF, cancellation, reader failure, writer failure, and dispatcher failure
  terminate with the accepted outcome and leave no detached task or retained
  stream state.
- Non-zero tests prove framing, bounds, positive/negative messages, channel
  purity, EOF/cancellation/failure/cleanup, ordering, and repetition.

## Repository Safety

Enumerate exact protocol source/test paths before editing. Preserve `.codex/`,
process entry points, Runtime composition, semantic crates, docs, prompt suites,
and unrelated files.

## Task-specific Validation

- List and run non-zero stdio framing/transport/cleanup tests.
- Rerun complete `oneagent-protocol` tests and dispatch regressions.
- Audit protocol output writes, bounds, task spawning, stream ownership,
  channel separation, and deferred transport absence.
- Run the canonical full workspace gate.

## Suggested commit message

`Implement Sprint 28 MCP stdio transport`

## Final report additions

Report framing/bounds, channel ownership, EOF/cancellation/failure behavior,
resource cleanup, tests, exclusions, commit hash, and final Git state.
