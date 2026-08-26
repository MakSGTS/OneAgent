# Implement MCP Server Discovery and Dispatch

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
- the exact versioned official MCP sources accepted by ADR-0050

## Prerequisites / Required gate

Task 3 is committed, its protocol-domain tests and full workspace validation
succeeded, and the working tree has no conflicting task-created change.

## Task

Implement only the accepted transport-independent MCP server discovery,
capability, method registration/dispatch, notification, and error behavior.

## Scope

### Included

- Accepted server identity, supported-version list, truthful empty first-slice
  capability set, discovery cache metadata, and deterministic result.
- Accepted method registration/dispatch seam, duplicate registration policy,
  version/metadata checks, unknown method, invalid params, notifications, and
  closed JSON-RPC errors.
- In-memory focused conformance and repeated dispatch evidence.

### Excluded

stdio or other transport, Runtime services, process I/O, semantic tools,
prompts/resources, tool execution/policy mapping, external clients, auth,
current-state docs, or Sprint completion.

## Acceptance Criteria

- Discovery exactly advertises only the accepted revision and no deferred MCP
  capability or method.
- Version and request validation precede method execution as ADR-0050 defines;
  unknown methods and invalid params use exact accepted errors.
- Notifications never receive responses and cannot invoke request-only
  behavior.
- Registration and dispatch are deterministic across insertion order and fresh
  instances; duplicate/ambiguous handlers fail closed.
- Dispatcher owns no I/O, Runtime state, background task, transport framing,
  semantic authority, or real tool action.
- Non-zero tests cover discovery, versions, metadata, method/params/errors,
  notifications, duplicates, reordered registration, and repetition.

## Repository Safety

Enumerate exact protocol source/test paths before editing. Preserve `.codex/`,
manifests unless ADR-required dependency state is incomplete, Runtime, semantic
crates, docs, prompt suites, and unrelated files.

## Task-specific Validation

- List and run non-zero discovery/dispatch/version/error/notification tests.
- Rerun complete `oneagent-protocol` tests.
- Audit advertised capabilities/methods against handlers and deferred absence.
- Run the canonical full workspace gate.

## Suggested commit message

`Implement Sprint 28 MCP server dispatch`

## Final report additions

Report discovery/capability output, dispatch and error precedence, notification
behavior, deterministic evidence, exclusions, commit hash, and final Git state.
