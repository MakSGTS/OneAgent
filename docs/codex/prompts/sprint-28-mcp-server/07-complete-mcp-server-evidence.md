# Complete MCP Server Evidence

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
- `docs/architecture/semantic-model-2.md`
- `docs/Architecture.md`
- `README.md`
- the exact versioned official MCP sources accepted by ADR-0050

## Prerequisites / Required gate

Task 6 is committed, every focused protocol/transport/lifecycle test and full
workspace gate succeeded, and the working tree has no conflicting task-created
change.

## Task

Complete non-zero public library and real-executable MCP conformance, affected
compatibility evidence, and truthful current-state documentation.

## Scope

### Included

- Public tests using only exported protocol/Runtime/process boundaries and
  platform-neutral child-process pipes or injected streams.
- Complete accepted protocol revision, discovery, message/error/bound,
  dispatch, framing, channel-purity, EOF/cancellation/failure/shutdown, cleanup,
  and repeated fresh-run matrix.
- Dependency/public-surface/capability/method/transport/ignored-test/deferred-
  scope audits and affected Runtime/CLI compatibility.
- Synchronization of only `README.md`, `docs/Architecture.md`, and
  `docs/architecture/semantic-model-2.md` after executable evidence passes.

### Excluded

New architecture or production behavior, semantic tools/capabilities, legacy
or alternate protocol versions, HTTP/SSE/custom transports, auth, live external
clients, remote services, real tool effects, Coverage changes, Roadmap state
transition, prompt retirement, or Sprint completion.

## Acceptance Criteria

- Public non-zero tests prove every accepted ADR-0050 wire, discovery,
  dispatch, stdio, lifecycle, failure, cleanup, and repetition criterion.
- The real executable stdout contains only expected newline-delimited JSON-RPC
  responses; stderr and exit behavior match the accepted matrix.
- Every advertised version, method, and capability maps to implementation and
  tests; every deferred capability is absent.
- Existing Runtime service/HTTP/Workspace/Graph Query and CLI behavior remains
  compatible; no graph, Context, Tool Policy, provider, or adapter code changes.
- Current-state docs describe exactly the implemented first slice and explicit
  deferrals without marking Sprint 28 completed.
- Full workspace validation passes.

## Repository Safety

Before editing, enumerate exact public-test and three documentation paths.
Preserve `.codex/`, production code, Roadmap, prompt suites, ADR/investigation,
Coverage Registries, unrelated docs, and unrelated files.

## Task-specific Validation

- List every protocol unit/public/transport/process test and reject zero-match
  targets.
- Run complete `oneagent-protocol`, affected `oneagent-runtime`, and applicable
  CLI compatibility tests.
- Run dependency, public API, protocol version, capability, method, JSON-RPC
  error, bounds, framing, channel-purity, task/process, ignored-test, live/
  external-state, real-effect, and deferred-scope audits.
- Verify current-state document links and claims.
- Run the canonical full workspace gate.

## Suggested commit message

`Complete Sprint 28 MCP server evidence`

## Final report additions

Report public matrix counts/outcomes, executable behavior, capability/method
audit, compatibility, documentation changes, exclusions, commit hash, and final
Git state.
